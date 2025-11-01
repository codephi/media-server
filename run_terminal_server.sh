#!/bin/bash
# Run script for terminal server

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "⚠️  Ambiente virtual não encontrado."
    echo "Execute primeiro: ./setup_terminal.sh"
    exit 1
fi

# Activate virtual environment and run server
source venv/bin/activate
echo "🚀 Iniciando Terminal Server em ws://localhost:8765..."
python3 terminal_server_v2.py
