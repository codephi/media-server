#!/usr/bin/env python3
"""
WebSocket Terminal Server v2 - Improved version
Provides real terminal access through WebSocket connection
"""

import asyncio
import json
import os
import pty
import select
import subprocess
import termios
import struct
import fcntl
import signal
import sys
from typing import Dict, Optional

try:
    import websockets
except ImportError:
    print("ERROR: websockets module not found!")
    print("Install with: pip install websockets")
    sys.exit(1)


class TerminalSession:
    """Manages a single terminal session"""
    
    def __init__(self, session_id: str, websocket):
        self.session_id = session_id
        self.websocket = websocket
        self.process: Optional[subprocess.Popen] = None
        self.master_fd: Optional[int] = None
        self.slave_fd: Optional[int] = None
        self.read_task = None
        
    async def start(self):
        """Start the terminal process"""
        try:
            # Create a pseudo-terminal
            self.master_fd, self.slave_fd = pty.openpty()
            
            # Set terminal attributes to handle echo properly
            attrs = termios.tcgetattr(self.slave_fd)
            # Ensure echo is enabled on the PTY (the shell will handle echo)
            attrs[3] = attrs[3] | termios.ECHO | termios.ICANON
            termios.tcsetattr(self.slave_fd, termios.TCSANOW, attrs)
            
            # Get user's shell or default to bash
            shell = os.environ.get('SHELL', '/bin/bash')
            
            # Set terminal size
            cols = 80
            rows = 24
            winsize = struct.pack('HHHH', rows, cols, 0, 0)
            fcntl.ioctl(self.slave_fd, termios.TIOCSWINSZ, winsize)
            
            # Start the shell process
            self.process = subprocess.Popen(
                [shell, '-l'],  # -l for login shell
                stdin=self.slave_fd,
                stdout=self.slave_fd,
                stderr=self.slave_fd,
                preexec_fn=os.setsid,
                env={
                    **os.environ,
                    'TERM': 'xterm-256color',
                    'COLORTERM': 'truecolor',
                }
            )
            
            # Close slave_fd in parent process
            os.close(self.slave_fd)
            self.slave_fd = None
            
            # Make master_fd non-blocking
            flags = fcntl.fcntl(self.master_fd, fcntl.F_GETFL)
            fcntl.fcntl(self.master_fd, fcntl.F_SETFL, flags | os.O_NONBLOCK)
            
            # Start reading output
            self.read_task = asyncio.create_task(self.read_output())
            
            print(f"Terminal session {self.session_id} started with PID {self.process.pid}")
            
        except Exception as e:
            print(f"Error starting terminal session: {e}")
            await self.send_error(f"Failed to start terminal: {e}")
            raise
    
    async def send_error(self, message: str):
        """Send error message to client"""
        try:
            await self.websocket.send(json.dumps({
                'type': 'error',
                'message': message
            }))
        except:
            pass
    
    async def read_output(self):
        """Read output from the terminal and send to WebSocket"""
        buffer = b''
        
        while self.process and self.process.poll() is None:
            try:
                # Use select to check for available data
                r, _, _ = select.select([self.master_fd], [], [], 0.01)
                
                if r:
                    try:
                        data = os.read(self.master_fd, 4096)
                        if data:
                            buffer += data
                            
                            # Try to decode and send
                            try:
                                text = buffer.decode('utf-8')
                                buffer = b''
                                await self.websocket.send(json.dumps({
                                    'type': 'output',
                                    'data': text
                                }))
                            except UnicodeDecodeError:
                                # If we can't decode, maybe we're in the middle of a multi-byte character
                                if len(buffer) > 4096:
                                    # Force send with replacement if buffer is too large
                                    text = buffer.decode('utf-8', errors='replace')
                                    buffer = b''
                                    await self.websocket.send(json.dumps({
                                        'type': 'output',
                                        'data': text
                                    }))
                    except BlockingIOError:
                        pass
                    except OSError as e:
                        if e.errno in (5, 11):  # EIO or EAGAIN
                            await asyncio.sleep(0.01)
                        else:
                            print(f"Read error: {e}")
                            break
                else:
                    await asyncio.sleep(0.01)
                    
            except Exception as e:
                print(f"Unexpected error in read_output: {e}")
                break
        
        # Send any remaining buffer
        if buffer:
            try:
                text = buffer.decode('utf-8', errors='replace')
                await self.websocket.send(json.dumps({
                    'type': 'output',
                    'data': text
                }))
            except:
                pass
        
        # Process ended
        if self.process:
            returncode = self.process.returncode if self.process.returncode is not None else -1
            try:
                await self.websocket.send(json.dumps({
                    'type': 'exit',
                    'code': returncode
                }))
            except:
                pass
            print(f"Terminal session {self.session_id} ended with code {returncode}")
    
    async def write_input(self, data: str):
        """Write input to the terminal"""
        if self.master_fd and self.process and self.process.poll() is None:
            try:
                os.write(self.master_fd, data.encode('utf-8'))
            except OSError as e:
                print(f"Write error: {e}")
                await self.send_error(f"Write error: {e}")
    
    async def resize(self, cols: int, rows: int):
        """Resize the terminal"""
        if self.master_fd and self.process and self.process.poll() is None:
            try:
                # Send the terminal size change
                winsize = struct.pack('HHHH', rows, cols, 0, 0)
                fcntl.ioctl(self.master_fd, termios.TIOCSWINSZ, winsize)
                
                # Send SIGWINCH to the process
                os.kill(self.process.pid, signal.SIGWINCH)
                
                print(f"Resized terminal {self.session_id} to {cols}x{rows}")
            except Exception as e:
                print(f"Resize error: {e}")
    
    async def cleanup(self):
        """Clean up the terminal session"""
        print(f"Cleaning up terminal session {self.session_id}")
        
        # Cancel read task
        if self.read_task:
            self.read_task.cancel()
            try:
                await self.read_task
            except asyncio.CancelledError:
                pass
        
        # Terminate process
        if self.process:
            try:
                if self.process.poll() is None:
                    self.process.terminate()
                    try:
                        self.process.wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        self.process.kill()
                        self.process.wait(timeout=1)
            except Exception as e:
                print(f"Error terminating process: {e}")
        
        # Close file descriptors
        if self.master_fd:
            try:
                os.close(self.master_fd)
            except:
                pass
        
        if self.slave_fd:
            try:
                os.close(self.slave_fd)
            except:
                pass


class TerminalServer:
    """WebSocket server for terminal sessions"""
    
    def __init__(self):
        self.sessions: Dict[str, TerminalSession] = {}
    
    async def handle_connection(self, websocket):
        """Handle a new WebSocket connection"""
        session_id = None
        session = None
        
        try:
            print(f"New connection from {websocket.remote_address}")
            
            # Wait for initialization message with timeout
            try:
                message = await asyncio.wait_for(websocket.recv(), timeout=5.0)
            except asyncio.TimeoutError:
                print("Connection timeout - no init message received")
                await websocket.send(json.dumps({
                    'type': 'error',
                    'message': 'Connection timeout - no initialization message'
                }))
                return
            
            data = json.loads(message)
            
            if data.get('type') != 'init':
                await websocket.send(json.dumps({
                    'type': 'error',
                    'message': 'Expected init message'
                }))
                return
            
            session_id = data.get('sessionId', f'session_{id(websocket)}')
            print(f"Creating terminal session: {session_id}")
            
            # Clean up existing session with same ID if it exists
            if session_id in self.sessions:
                print(f"Cleaning up existing session: {session_id}")
                await self.sessions[session_id].cleanup()
                del self.sessions[session_id]
            
            # Create new session
            session = TerminalSession(session_id, websocket)
            self.sessions[session_id] = session
            
            # Start the terminal
            await session.start()
            
            # Send ready message
            await websocket.send(json.dumps({
                'type': 'ready',
                'sessionId': session_id
            }))
            
            # Handle messages
            async for message in websocket:
                try:
                    data = json.loads(message)
                    
                    if data['type'] == 'input':
                        await session.write_input(data.get('data', ''))
                    elif data['type'] == 'resize':
                        cols = data.get('cols', 80)
                        rows = data.get('rows', 24)
                        await session.resize(cols, rows)
                    elif data['type'] == 'ping':
                        await websocket.send(json.dumps({'type': 'pong'}))
                except json.JSONDecodeError as e:
                    print(f"Invalid JSON received: {e}")
                    await websocket.send(json.dumps({
                        'type': 'error',
                        'message': f'Invalid JSON: {e}'
                    }))
                except Exception as e:
                    print(f"Error handling message: {e}")
                    await websocket.send(json.dumps({
                        'type': 'error',
                        'message': f'Error: {e}'
                    }))
                    
        except websockets.exceptions.ConnectionClosed:
            print(f"Connection closed for session: {session_id}")
        except Exception as e:
            print(f"Error handling connection: {e}")
            import traceback
            traceback.print_exc()
        finally:
            # Clean up session
            if session:
                await session.cleanup()
            if session_id and session_id in self.sessions:
                del self.sessions[session_id]
                print(f"Session {session_id} removed")
    
    async def start_server(self, host: str = '0.0.0.0', port: int = 8765):
        """Start the WebSocket server"""
        print(f"Starting terminal server on ws://{host}:{port}")
        print("Press Ctrl+C to stop")
        
        # Start server with proper configuration
        async with websockets.serve(
            self.handle_connection,
            host,
            port,
            compression=None,  # Disable compression for lower latency
            max_size=10 * 1024 * 1024,  # 10MB max message size
            ping_interval=20,  # Send ping every 20 seconds
            ping_timeout=10,  # Wait 10 seconds for pong
        ):
            try:
                await asyncio.Future()  # Run forever
            except KeyboardInterrupt:
                print("\nShutting down...")


async def main():
    """Main entry point"""
    server = TerminalServer()
    
    # Parse command line arguments
    import argparse
    parser = argparse.ArgumentParser(description='WebSocket Terminal Server')
    parser.add_argument('--host', default='0.0.0.0', help='Host to bind to (default: 0.0.0.0)')
    parser.add_argument('--port', type=int, default=8765, help='Port to bind to (default: 8765)')
    args = parser.parse_args()
    
    try:
        await server.start_server(args.host, args.port)
    except OSError as e:
        if e.errno == 98:  # Address already in use
            print(f"Error: Port {args.port} is already in use")
            print("Try a different port with --port option")
        else:
            raise


if __name__ == '__main__':
    try:
        # Set up asyncio for proper signal handling
        if sys.platform != 'win32':
            # Unix/Linux: Use uvloop if available for better performance
            try:
                import uvloop
                uvloop.install()
                print("Using uvloop for better performance")
            except ImportError:
                pass
        
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nShutting down terminal server...")
    except Exception as e:
        print(f"Fatal error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)