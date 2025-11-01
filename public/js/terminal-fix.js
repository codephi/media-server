// Terminal connection fix for WebSocket
// This module handles the WebSocket connection separately to avoid double echo

class TerminalWebSocketHandler {
    constructor(terminal, tabId) {
        this.terminal = terminal;
        this.tabId = tabId;
        this.ws = null;
        this.connected = false;
        this.reconnectTimeout = null;
        this.inputBuffer = '';
    }

    connect(url = 'ws://localhost:8765') {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            console.log('Already connected');
            return;
        }

        try {
            this.ws = new WebSocket(url);
            this.setupHandlers();
        } catch (error) {
            console.error('Failed to create WebSocket:', error);
            this.handleConnectionError();
        }
    }

    setupHandlers() {
        // Connection opened
        this.ws.onopen = () => {
            console.log('Terminal WebSocket connected');
            this.connected = true;
            
            // Clear any reconnect timeout
            if (this.reconnectTimeout) {
                clearTimeout(this.reconnectTimeout);
                this.reconnectTimeout = null;
            }
            
            // Send initialization
            this.send({
                type: 'init',
                sessionId: this.tabId
            });
            
            // Send terminal size
            this.send({
                type: 'resize',
                cols: this.terminal.cols,
                rows: this.terminal.rows
            });
            
            // Clear terminal and show connected message
            this.terminal.clear();
        };

        // Handle messages from server
        this.ws.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                this.handleMessage(message);
            } catch (e) {
                // If not JSON, treat as raw output
                this.terminal.write(event.data);
            }
        };

        // Connection closed
        this.ws.onclose = () => {
            console.log('Terminal WebSocket disconnected');
            this.connected = false;
            this.handleDisconnection();
        };

        // Connection error
        this.ws.onerror = (error) => {
            console.error('Terminal WebSocket error:', error);
            this.handleConnectionError();
        };
    }

    handleMessage(message) {
        switch (message.type) {
            case 'output':
                // Write output from server
                this.terminal.write(message.data);
                break;
                
            case 'ready':
                this.terminal.writeln('🚀 Terminal conectado!');
                this.terminal.writeln('');
                break;
                
            case 'exit':
                this.terminal.writeln(`\r\nProcesso finalizado com código: ${message.code}`);
                this.connected = false;
                break;
                
            case 'error':
                this.terminal.writeln(`\r\n\x1b[31mErro: ${message.message}\x1b[0m`);
                break;
                
            case 'pong':
                // Ping response, ignore
                break;
                
            default:
                console.warn('Unknown message type:', message.type);
        }
    }

    send(data) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(data));
        }
    }

    sendInput(data) {
        if (this.connected) {
            this.send({
                type: 'input',
                data: data
            });
        }
    }

    resize(cols, rows) {
        if (this.connected) {
            this.send({
                type: 'resize',
                cols: cols,
                rows: rows
            });
        }
    }

    handleDisconnection() {
        this.terminal.writeln('\r\n\x1b[31mConexão perdida com o servidor.\x1b[0m');
        this.terminal.writeln('Tentando reconectar em 3 segundos...');
        
        // Schedule reconnection
        this.reconnectTimeout = setTimeout(() => {
            this.terminal.writeln('Reconectando...');
            this.connect();
        }, 3000);
    }

    handleConnectionError() {
        this.terminal.writeln('\x1b[33m⚠ Servidor de terminal não disponível.\x1b[0m');
        this.terminal.writeln('Verifique se o servidor está rodando:');
        this.terminal.writeln('  ./run_terminal_server.sh');
        
        // Try to reconnect after 5 seconds
        this.reconnectTimeout = setTimeout(() => {
            this.connect();
        }, 5000);
    }

    disconnect() {
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = null;
        }
        
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
        
        this.connected = false;
    }
}

// Export for use in main terminal.js
window.TerminalWebSocketHandler = TerminalWebSocketHandler;