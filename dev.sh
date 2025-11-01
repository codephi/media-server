#!/bin/bash

# Script para desenvolvimento com hot-reload completo
# Este script usa cargo-watch para recompilar quando templates ou código mudam

set -e

# Verificar se cargo-watch está instalado
if ! command -v cargo-watch &> /dev/null; then
    echo "🔧 cargo-watch não encontrado. Instalando..."
    cargo install cargo-watch
fi

echo "🚀 Iniciando servidor de desenvolvimento com hot-reload completo..."
echo "📁 Diretório: $(pwd)"
echo "🎯 O servidor será recompilado automaticamente quando templates ou código mudarem"
echo ""

# Usar cargo-watch para monitorar mudanças e reiniciar o servidor
cargo watch \
    --watch src \
    --watch templates \
    --watch public \
    --watch Cargo.toml \
    --clear \
    --exec "run -- . --watch"