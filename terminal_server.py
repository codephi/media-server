#!/usr/bin/env python3
"""
WebSocket Terminal Server
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
from typing import Dict, Optional
import websockets

class TerminalSession:
    """Manages a single terminal session"""
    
    def __init__(self, session_id: str, websocket):
        self.session_id = session_id
        self.websocket = websocket
        self.process: Optional[subprocess.Popen] = None
        self.master_fd: Optional[int] = None
        self.slave_fd: Optional[int] = None
        
    async def start(self):
        """Start the terminal process"""
        # Create a pseudo-terminal
        self.master_fd, self.slave_fd = pty.openpty()
        
        # Get user's shell or default to bash
        shell = os.environ.get('SHELL', '/bin/bash')
        
        # Start the shell process
        self.process = subprocess.Popen(
            [shell],
            stdin=self.slave_fd,
            stdout=self.slave_fd,
            stderr=self.slave_fd,
            preexec_fn=os.setsid,
            env={
                **os.environ,
                'TERM': 'xterm-256color',
                'COLORTERM': 'truecolor',
                'COLUMNS': '80',
                'LINES': '24'
            }
        )
        
        # Make master_fd non-blocking
        flags = fcntl.fcntl(self.master_fd, fcntl.F_GETFL)
        fcntl.fcntl(self.master_fd, fcntl.F_SETFL, flags | os.O_NONBLOCK)
        
        # Start reading output
        asyncio.create_task(self.read_output())
        
    async def read_output(self):
        """Read output from the terminal and send to WebSocket"""
        while self.process and self.process.poll() is None:
            try:
                # Check if there's data to read
                r, _, _ = select.select([self.master_fd], [], [], 0.01)
                if r:
                    output = os.read(self.master_fd, 1024)
                    if output:
                        # Send output to WebSocket
                        await self.websocket.send(json.dumps({
                            'type': 'output',
                            'data': output.decode('utf-8', errors='replace')
                        }))
                else:
                    await asyncio.sleep(0.01)
            except (OSError, IOError) as e:
                if e.errno == 5:  # Input/output error
                    break
                elif e.errno == 11:  # Resource temporarily unavailable
                    await asyncio.sleep(0.01)
                else:
                    print(f"Read error: {e}")
                    break
            except Exception as e:
                print(f"Unexpected error reading output: {e}")
                break
        
        # Process ended
        if self.process:
            await self.websocket.send(json.dumps({
                'type': 'exit',
                'code': self.process.returncode
            }))
    
    async def write_input(self, data: str):
        """Write input to the terminal"""
        if self.master_fd:
            try:
                os.write(self.master_fd, data.encode('utf-8'))
            except OSError as e:
                print(f"Write error: {e}")
    
    async def resize(self, cols: int, rows: int):
        """Resize the terminal"""
        if self.master_fd:
            try:
                # Send the terminal size change signal
                winsize = struct.pack('HHHH', rows, cols, 0, 0)
                fcntl.ioctl(self.master_fd, termios.TIOCSWINSZ, winsize)
                
                # Send SIGWINCH to the process group
                if self.process:
                    os.kill(self.process.pid, signal.SIGWINCH)
            except Exception as e:
                print(f"Resize error: {e}")
    
    def cleanup(self):
        """Clean up the terminal session"""
        if self.process:
            try:
                self.process.terminate()
                self.process.wait(timeout=1)
            except subprocess.TimeoutExpired:
                self.process.kill()
            except Exception:
                pass
        
        if self.master_fd:
            try:
                os.close(self.master_fd)
            except Exception:
                pass
        
        if self.slave_fd:
            try:
                os.close(self.slave_fd)
            except Exception:
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
            # Wait for initialization message
            message = await websocket.recv()
            data = json.loads(message)
            
            if data['type'] == 'init':
                session_id = data.get('sessionId', 'default')
                print(f"New terminal session: {session_id}")
                
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
                    data = json.loads(message)
                    
                    if data['type'] == 'input':
                        await session.write_input(data['data'])
                    elif data['type'] == 'resize':
                        await session.resize(data['cols'], data['rows'])
                    elif data['type'] == 'ping':
                        await websocket.send(json.dumps({'type': 'pong'}))
                        
        except websockets.exceptions.ConnectionClosed:
            print(f"Connection closed for session: {session_id}")
        except Exception as e:
            print(f"Error handling connection: {e}")
            import traceback
            traceback.print_exc()
        finally:
            # Clean up session
            if session:
                session.cleanup()
            if session_id and session_id in self.sessions:
                del self.sessions[session_id]
    
    async def start_server(self, host: str = 'localhost', port: int = 8765):
        """Start the WebSocket server"""
        print(f"Starting terminal server on ws://{host}:{port}")
        # Updated for newer websockets version
        async with websockets.serve(
            self.handle_connection, 
            host, 
            port,
            compression=None,  # Disable compression for better latency
            max_size=10 * 1024 * 1024  # 10MB max message size
        ):
            await asyncio.Future()  # Run forever


async def main():
    """Main entry point"""
    server = TerminalServer()
    await server.start_server()


if __name__ == '__main__':
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nShutting down terminal server...")