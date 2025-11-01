#!/bin/bash
# Setup script for terminal server with virtual environment

echo "📦 Configurando ambiente para o Terminal Server..."

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    echo "Criando ambiente virtual Python..."
    python3 -m venv venv
fi

# Activate virtual environment
source venv/bin/activate

# Install dependencies
echo "Instalando dependências..."
pip install websockets

echo "✅ Configuração concluída!"
echo ""
echo "Para iniciar o servidor de terminal, execute:"
echo "  source venv/bin/activate"
echo "  python3 terminal_server.py"
echo ""
echo "Ou use o script de inicialização:"
echo "  ./run_terminal_server.sh"