#!/bin/bash
# Start script for media-serve with terminal server

echo "🚀 Starting Media Serve with Terminal Server..."

# Check if python-websockets is installed
if ! python3 -c "import websockets" 2>/dev/null; then
    echo "⚠️  python-websockets não está instalado."
    echo "Por favor, instale com: sudo pacman -S python-websockets"
    echo "Ou use um ambiente virtual:"
    echo "  python3 -m venv venv"
    echo "  source venv/bin/activate"
    echo "  pip install websockets"
    echo ""
    echo "Continuando sem servidor de terminal..."
    SKIP_TERMINAL=1
fi

# Start terminal server in background if websockets is available
if [ -z "$SKIP_TERMINAL" ]; then
    echo "Starting terminal server on ws://localhost:8765..."
    python3 terminal_server.py &
    TERMINAL_PID=$!
    echo "Terminal server started with PID: $TERMINAL_PID"
    sleep 1
fi

# Start main server (adjust this command based on your actual server)
echo "Starting media-serve..."
if [ -f "server.py" ]; then
    python3 server.py
elif [ -f "app.py" ]; then
    python3 app.py
elif [ -f "main.py" ]; then
    python3 main.py
else
    echo "⚠️  Não foi encontrado arquivo principal do servidor."
    echo "Por favor, ajuste este script com o comando correto."
fi

# Clean up on exit
if [ ! -z "$TERMINAL_PID" ]; then
    echo "Stopping terminal server..."
    kill $TERMINAL_PID 2>/dev/null
fi