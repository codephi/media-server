// Quake-style Terminal Component with Tabs and Persistence
class QuakeTerminal {
    constructor() {
        this.isVisible = false;
        this.tabs = [];
        this.activeTabId = null;
        this.terminalInstances = {};
        this.container = null;
        this.loadState();
        this.init();
    }

    init() {
        this.createTerminalUI();
        this.attachEventListeners();
        this.restoreTabs();
        
        // Load xterm.js dynamically if not already loaded
        if (typeof Terminal === 'undefined') {
            this.loadXtermJS().then(() => {
                this.initializeTabs();
            });
        } else {
            this.initializeTabs();
        }
    }

    loadXtermJS() {
        return new Promise((resolve) => {
            // Load xterm.js CSS
            const cssLink = document.createElement('link');
            cssLink.rel = 'stylesheet';
            cssLink.href = 'https://cdn.jsdelivr.net/npm/xterm@5.3.0/css/xterm.css';
            document.head.appendChild(cssLink);

            // Load xterm.js
            const script = document.createElement('script');
            script.src = 'https://cdn.jsdelivr.net/npm/xterm@5.3.0/lib/xterm.js';
            script.onload = () => {
                // Load xterm addons
                const fitAddon = document.createElement('script');
                fitAddon.src = 'https://cdn.jsdelivr.net/npm/xterm-addon-fit@0.8.0/lib/xterm-addon-fit.js';
                fitAddon.onload = () => {
                    const webLinksAddon = document.createElement('script');
                    webLinksAddon.src = 'https://cdn.jsdelivr.net/npm/xterm-addon-web-links@0.9.0/lib/xterm-addon-web-links.js';
                    webLinksAddon.onload = resolve;
                    document.head.appendChild(webLinksAddon);
                };
                document.head.appendChild(fitAddon);
            };
            document.head.appendChild(script);
        });
    }

    createTerminalUI() {
        // Create main container
        this.container = document.createElement('div');
        this.container.id = 'quake-terminal';
        this.container.className = 'quake-terminal';
        this.container.innerHTML = `
            <div class="terminal-header">
                <div class="terminal-tabs">
                    <div class="tabs-list"></div>
                    <button class="add-tab-btn" title="Nova aba">+</button>
                </div>
                <div class="terminal-controls">
                    <button class="terminal-minimize" title="Minimizar">_</button>
                    <button class="terminal-maximize" title="Maximizar">□</button>
                    <button class="terminal-close" title="Fechar">×</button>
                </div>
            </div>
            <div class="terminal-content"></div>
            <div class="terminal-resize-handle"></div>
        `;
        document.body.appendChild(this.container);

        // Create toggle button
        const toggleBtn = document.createElement('button');
        toggleBtn.id = 'terminal-toggle';
        toggleBtn.className = 'terminal-toggle';
        toggleBtn.innerHTML = '⌨ Terminal';
        toggleBtn.title = 'Toggle Terminal (F12)';
        document.body.appendChild(toggleBtn);
    }

    attachEventListeners() {
        // Toggle button
        document.getElementById('terminal-toggle').addEventListener('click', () => {
            this.toggle();
        });

        // Keyboard shortcut (F12 or Ctrl+`)
        document.addEventListener('keydown', (e) => {
            if (e.key === 'F12' || (e.ctrlKey && e.key === '`')) {
                e.preventDefault();
                this.toggle();
            }
        });

        // Header controls
        this.container.querySelector('.terminal-minimize').addEventListener('click', () => {
            this.minimize();
        });

        this.container.querySelector('.terminal-maximize').addEventListener('click', () => {
            this.maximize();
        });

        this.container.querySelector('.terminal-close').addEventListener('click', () => {
            this.hide();
        });

        // Add tab button
        this.container.querySelector('.add-tab-btn').addEventListener('click', () => {
            this.addTab();
        });

        // Resize handle
        this.initResize();

        // Save state before page unload
        window.addEventListener('beforeunload', () => {
            this.saveState();
        });
    }

    initResize() {
        const handle = this.container.querySelector('.terminal-resize-handle');
        let isResizing = false;
        let startY = 0;
        let startHeight = 0;

        handle.addEventListener('mousedown', (e) => {
            isResizing = true;
            startY = e.clientY;
            startHeight = this.container.offsetHeight;
            document.body.style.userSelect = 'none';
        });

        document.addEventListener('mousemove', (e) => {
            if (!isResizing) return;
            
            const deltaY = startY - e.clientY;
            const newHeight = Math.max(200, Math.min(window.innerHeight - 50, startHeight + deltaY));
            this.container.style.height = `${newHeight}px`;
            
            // Resize all terminal instances
            Object.values(this.terminalInstances).forEach(term => {
                if (term.fitAddon) {
                    term.fitAddon.fit();
                }
            });
        });

        document.addEventListener('mouseup', () => {
            if (isResizing) {
                isResizing = false;
                document.body.style.userSelect = '';
                this.saveState();
            }
        });
    }

    addTab(title = null) {
        const tabId = `tab-${Date.now()}`;
        const tabTitle = title || `Terminal ${this.tabs.length + 1}`;
        
        const tab = {
            id: tabId,
            title: tabTitle,
            content: '',
            scrollback: []
        };
        
        this.tabs.push(tab);
        this.renderTab(tab);
        this.switchToTab(tabId);
        this.createTerminalInstance(tabId);
        this.saveState();
        
        return tabId;
    }

    renderTab(tab) {
        const tabsList = this.container.querySelector('.tabs-list');
        const tabElement = document.createElement('div');
        tabElement.className = 'terminal-tab';
        tabElement.dataset.tabId = tab.id;
        tabElement.innerHTML = `
            <span class="tab-title" contenteditable="false">${tab.title}</span>
            <button class="tab-close">×</button>
        `;
        
        tabElement.addEventListener('click', (e) => {
            if (!e.target.classList.contains('tab-close')) {
                this.switchToTab(tab.id);
            }
        });
        
        tabElement.querySelector('.tab-title').addEventListener('dblclick', (e) => {
            e.target.contentEditable = 'true';
            e.target.focus();
            e.target.select();
        });
        
        tabElement.querySelector('.tab-title').addEventListener('blur', (e) => {
            e.target.contentEditable = 'false';
            tab.title = e.target.textContent;
            this.saveState();
        });
        
        tabElement.querySelector('.tab-close').addEventListener('click', (e) => {
            e.stopPropagation();
            this.closeTab(tab.id);
        });
        
        tabsList.appendChild(tabElement);
    }

    createTerminalInstance(tabId) {
        if (!window.Terminal) return;
        
        const contentArea = this.container.querySelector('.terminal-content');
        
        // Create terminal container for this tab
        const termContainer = document.createElement('div');
        termContainer.className = 'terminal-instance';
        termContainer.dataset.tabId = tabId;
        termContainer.style.display = 'none';
        contentArea.appendChild(termContainer);
        
        // Initialize xterm.js
        const term = new Terminal({
            cursorBlink: true,
            fontSize: 14,
            fontFamily: 'Consolas, Monaco, monospace',
            theme: {
                background: '#1e1e1e',
                foreground: '#d4d4d4',
                cursor: '#d4d4d4',
                black: '#000000',
                red: '#cd3131',
                green: '#0dbc79',
                yellow: '#e5e510',
                blue: '#2472c8',
                magenta: '#bc3fbc',
                cyan: '#11a8cd',
                white: '#e5e5e5',
                brightBlack: '#666666',
                brightRed: '#f14c4c',
                brightGreen: '#23d18b',
                brightYellow: '#f5f543',
                brightBlue: '#3b8eea',
                brightMagenta: '#d670d6',
                brightCyan: '#29b8db',
                brightWhite: '#e5e5e5'
            },
            convertEol: true,
            scrollback: 1000,
            rows: 24,
            cols: 80,
            cursorStyle: 'block',
            bellStyle: 'none'
        });
        
        // Add addons
        const fitAddon = new FitAddon.FitAddon();
        term.loadAddon(fitAddon);
        
        const webLinksAddon = new WebLinksAddon.WebLinksAddon();
        term.loadAddon(webLinksAddon);
        
        term.open(termContainer);
        fitAddon.fit();
        
        // Store instance
        this.terminalInstances[tabId] = {
            terminal: term,
            fitAddon: fitAddon,
            container: termContainer,
            websocket: null,
            connected: false
        };
        
        // Connect to WebSocket server for real terminal
        this.connectToWebSocket(tabId);
    }

    connectToWebSocket(tabId) {
        const instance = this.terminalInstances[tabId];
        if (!instance) return;
        
        const term = instance.terminal;
        
        // Check if TerminalWebSocketHandler is available
        if (window.TerminalWebSocketHandler) {
            // Use the improved handler
            const handler = new TerminalWebSocketHandler(term, tabId);
            instance.wsHandler = handler;
            
            // Connect to server
            handler.connect();
            
            // Handle terminal input
            term.onData(data => {
                handler.sendInput(data);
            });
            
            // Handle terminal resize  
            term.onResize(({ cols, rows }) => {
                handler.resize(cols, rows);
            });
            
            instance.connected = true;
            return;
        }
        
        // Fallback to original implementation if handler not available
        const wsUrl = 'ws://localhost:8765';
        
        try {
            const ws = new WebSocket(wsUrl);
            instance.websocket = ws;
            
            // Connection opened
            ws.onopen = () => {
                console.log('Connected to terminal server');
                instance.connected = true;
                
                // Send initialization message
                ws.send(JSON.stringify({
                    type: 'init',
                    sessionId: tabId
                }));
                
                // Send initial terminal size
                const cols = term.cols;
                const rows = term.rows;
                ws.send(JSON.stringify({
                    type: 'resize',
                    cols: cols,
                    rows: rows
                }));
            };
            
            // Handle messages from server
            ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    
                    switch (message.type) {
                        case 'output':
                            term.write(message.data);
                            break;
                        case 'ready':
                            term.clear();
                            term.writeln('🚀 Terminal conectado!');
                            term.writeln('');
                            break;
                        case 'exit':
                            term.writeln(`\r\nProcesso finalizado com código: ${message.code}`);
                            instance.connected = false;
                            break;
                    }
                } catch (e) {
                    console.error('Error parsing message:', e);
                }
            };
            
            // Connection closed
            ws.onclose = () => {
                console.log('Disconnected from terminal server');
                instance.connected = false;
                term.writeln('\r\n\x1b[31mConexão com servidor perdida.\x1b[0m');
                term.writeln('Tentando reconectar...');
                
                // Try to reconnect after 3 seconds
                setTimeout(() => {
                    if (!instance.connected) {
                        this.connectToWebSocket(tabId);
                    }
                }, 3000);
            };
            
            // Connection error
            ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                // Fallback to simulated terminal
                this.initSimulatedTerminal(term, tabId);
            };
            
            // Handle terminal input - send to server without local echo
            term.onData(data => {
                if (instance.connected && ws.readyState === WebSocket.OPEN) {
                    // Send to server - the server will echo back what should be displayed
                    ws.send(JSON.stringify({
                        type: 'input',
                        data: data
                    }));
                }
            });
            
            // Handle terminal resize
            term.onResize(({ cols, rows }) => {
                if (instance.connected && ws.readyState === WebSocket.OPEN) {
                    ws.send(JSON.stringify({
                        type: 'resize',
                        cols: cols,
                        rows: rows
                    }));
                }
            });
            
        } catch (error) {
            console.error('Failed to connect to WebSocket:', error);
            // Fallback to simulated terminal
            this.initSimulatedTerminal(term, tabId);
        }
    }
    
    initSimulatedTerminal(term, tabId) {
        // Simulated terminal for when WebSocket is not available
        term.writeln('\x1b[33m⚠ Servidor de terminal não disponível.\x1b[0m');
        term.writeln('Usando terminal simulado.');
        term.writeln('Para terminal real, execute: python3 terminal_server.py');
        term.writeln('');
        
        let currentLine = '';
        const prompt = '$ ';
        term.write(prompt);
        
        term.onData(data => {
            const tab = this.tabs.find(t => t.id === tabId);
            
            switch (data) {
                case '\r': // Enter
                    term.writeln('');
                    this.processSimulatedCommand(currentLine, term);
                    currentLine = '';
                    term.write(prompt);
                    break;
                case '\u007F': // Backspace
                    if (currentLine.length > 0) {
                        currentLine = currentLine.slice(0, -1);
                        term.write('\b \b');
                    }
                    break;
                case '\u0003': // Ctrl+C
                    term.writeln('^C');
                    currentLine = '';
                    term.write(prompt);
                    break;
                default:
                    if (data >= ' ') {
                        currentLine += data;
                        term.write(data);
                    }
            }
            
            // Save scrollback
            if (tab) {
                tab.content = currentLine;
                this.saveState();
            }
        });
    }

    processSimulatedCommand(command, term) {
        const cmd = command.trim().toLowerCase();
        const args = cmd.split(' ');
        
        switch (args[0]) {
            case 'help':
                term.writeln('Comandos simulados disponíveis:');
                term.writeln('  help     - Mostra esta ajuda');
                term.writeln('  clear    - Limpa o terminal');
                term.writeln('  date     - Mostra a data e hora atual');
                term.writeln('  echo     - Repete o texto digitado');
                break;
            case 'clear':
                term.clear();
                break;
            case 'date':
                term.writeln(new Date().toLocaleString('pt-BR'));
                break;
            case 'echo':
                term.writeln(args.slice(1).join(' '));
                break;
            case '':
                // Empty command
                break;
            default:
                term.writeln(`\x1b[31mComando simulado não encontrado: ${args[0]}\x1b[0m`);
                term.writeln('Este é um terminal simulado. Execute o servidor para comandos reais.');
        }
    }

    switchToTab(tabId) {
        // Update active tab in UI
        this.container.querySelectorAll('.terminal-tab').forEach(tab => {
            tab.classList.toggle('active', tab.dataset.tabId === tabId);
        });
        
        // Show/hide terminal instances
        this.container.querySelectorAll('.terminal-instance').forEach(instance => {
            instance.style.display = instance.dataset.tabId === tabId ? 'block' : 'none';
        });
        
        this.activeTabId = tabId;
        
        // Fit terminal to container
        if (this.terminalInstances[tabId] && this.terminalInstances[tabId].fitAddon) {
            setTimeout(() => {
                this.terminalInstances[tabId].fitAddon.fit();
            }, 0);
        }
        
        this.saveState();
    }

    closeTab(tabId) {
        if (this.tabs.length <= 1) {
            alert('Você precisa manter pelo menos uma aba aberta.');
            return;
        }
        
        // Remove tab from array
        const tabIndex = this.tabs.findIndex(t => t.id === tabId);
        if (tabIndex !== -1) {
            this.tabs.splice(tabIndex, 1);
        }
        
        // Remove tab element
        const tabElement = this.container.querySelector(`.terminal-tab[data-tab-id="${tabId}"]`);
        if (tabElement) {
            tabElement.remove();
        }
        
        // Destroy terminal instance
        if (this.terminalInstances[tabId]) {
            // Close WebSocket if connected
            if (this.terminalInstances[tabId].wsHandler) {
                this.terminalInstances[tabId].wsHandler.disconnect();
            } else if (this.terminalInstances[tabId].websocket) {
                this.terminalInstances[tabId].websocket.close();
            }
            this.terminalInstances[tabId].terminal.dispose();
            this.terminalInstances[tabId].container.remove();
            delete this.terminalInstances[tabId];
        }
        
        // Switch to another tab if this was active
        if (this.activeTabId === tabId && this.tabs.length > 0) {
            this.switchToTab(this.tabs[0].id);
        }
        
        this.saveState();
    }

    show() {
        this.container.classList.add('visible');
        this.isVisible = true;
        
        // Fit all terminals
        setTimeout(() => {
            Object.values(this.terminalInstances).forEach(term => {
                if (term.fitAddon) {
                    term.fitAddon.fit();
                }
            });
        }, 300);
        
        this.saveState();
    }

    hide() {
        this.container.classList.remove('visible');
        this.isVisible = false;
        this.saveState();
    }

    minimize() {
        this.container.classList.add('minimized');
        this.saveState();
    }

    maximize() {
        this.container.classList.toggle('maximized');
        
        // Refit terminals after maximize/restore
        setTimeout(() => {
            Object.values(this.terminalInstances).forEach(term => {
                if (term.fitAddon) {
                    term.fitAddon.fit();
                }
            });
        }, 300);
        
        this.saveState();
    }

    toggle() {
        if (this.isVisible) {
            this.hide();
        } else {
            this.show();
        }
    }

    saveState() {
        const state = {
            isVisible: this.isVisible,
            tabs: this.tabs,
            activeTabId: this.activeTabId,
            height: this.container ? this.container.style.height : '400px',
            isMaximized: this.container ? this.container.classList.contains('maximized') : false,
            isMinimized: this.container ? this.container.classList.contains('minimized') : false
        };
        
        localStorage.setItem('quakeTerminalState', JSON.stringify(state));
    }

    loadState() {
        const savedState = localStorage.getItem('quakeTerminalState');
        if (savedState) {
            try {
                const state = JSON.parse(savedState);
                this.isVisible = state.isVisible || false;
                this.tabs = state.tabs || [];
                this.activeTabId = state.activeTabId;
                return state;
            } catch (e) {
                console.error('Failed to load terminal state:', e);
            }
        }
        return null;
    }

    restoreTabs() {
        const state = this.loadState();
        
        if (state) {
            // Restore terminal height
            if (state.height) {
                this.container.style.height = state.height;
            }
            
            // Restore maximized/minimized state
            if (state.isMaximized) {
                this.container.classList.add('maximized');
            }
            if (state.isMinimized) {
                this.container.classList.add('minimized');
            }
            
            // Restore visibility
            if (state.isVisible) {
                this.show();
            }
        }
        
        // Create at least one tab if none exist
        if (this.tabs.length === 0) {
            this.addTab('Terminal 1');
        }
    }

    initializeTabs() {
        // Render existing tabs
        this.tabs.forEach(tab => {
            this.renderTab(tab);
            this.createTerminalInstance(tab.id);
        });
        
        // Switch to active tab
        if (this.activeTabId && this.tabs.find(t => t.id === this.activeTabId)) {
            this.switchToTab(this.activeTabId);
        } else if (this.tabs.length > 0) {
            this.switchToTab(this.tabs[0].id);
        }
    }
}

// Initialize terminal when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.quakeTerminal = new QuakeTerminal();
    });
} else {
    window.quakeTerminal = new QuakeTerminal();
}