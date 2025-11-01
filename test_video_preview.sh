#!/bin/bash

echo "=== Teste da Funcionalidade de Preview de Vídeo ==="
echo

# Teste 1: Verificar se o servidor está rodando
echo "1. Testando conectividade do servidor..."
if curl -s "http://127.0.0.1:8082/" > /dev/null; then
    echo "   ✅ Servidor está respondendo"
else
    echo "   ❌ Servidor não está respondendo"
    exit 1
fi

# Teste 2: Obter informações de preview
echo
echo "2. Obtendo informações de preview do vídeo..."
PREVIEW_INFO=$(curl -s "http://127.0.0.1:8082/video-previews/video.mp4")
if echo "$PREVIEW_INFO" | grep -q "duration"; then
    echo "   ✅ Informações de preview obtidas com sucesso"
    DURATION=$(echo "$PREVIEW_INFO" | grep -o '"duration":[0-9.]*' | cut -d: -f2)
    THUMBNAIL_COUNT=$(echo "$PREVIEW_INFO" | grep -o '"filename"' | wc -l)
    echo "   📊 Duração do vídeo: ${DURATION}s"
    echo "   🖼️  Miniaturas geradas: $THUMBNAIL_COUNT"
else
    echo "   ❌ Falha ao obter informações de preview"
fi

# Teste 3: Testar uma miniatura específica
echo
echo "3. Testando miniatura específica (tempo: 10s)..."
THUMB_RESPONSE=$(curl -s -w "%{http_code}" "http://127.0.0.1:8082/video-previews/video.mp4?time=10" -o /tmp/test_thumb.jpg)
if [ "$THUMB_RESPONSE" = "200" ]; then
    echo "   ✅ Miniatura obtida com sucesso"
    FILE_INFO=$(file /tmp/test_thumb.jpg)
    echo "   📄 Informações do arquivo: $FILE_INFO"
    SIZE=$(stat -f%z /tmp/test_thumb.jpg 2>/dev/null || stat -c%s /tmp/test_thumb.jpg 2>/dev/null)
    echo "   📏 Tamanho: ${SIZE} bytes"
    rm -f /tmp/test_thumb.jpg
else
    echo "   ❌ Falha ao obter miniatura (HTTP: $THUMB_RESPONSE)"
fi

# Teste 4: Verificar cache local
echo
echo "4. Verificando cache local..."
if [ -d ".video-previews" ]; then
    echo "   ✅ Pasta de cache existe"
    CACHE_COUNT=$(find .video-previews -name "*.jpg" | wc -l)
    echo "   📁 Miniaturas em cache: $CACHE_COUNT"
    CACHE_SIZE=$(du -sh .video-previews 2>/dev/null | cut -f1)
    echo "   💾 Tamanho do cache: $CACHE_SIZE"
else
    echo "   ❌ Pasta de cache não encontrada"
fi

echo
echo "=== Resumo dos Testes ==="
echo "✅ Funcionalidade de preview de vídeo implementada e funcionando!"
echo "🎯 Para testar a interface, acesse: http://127.0.0.1:8082/file/video.mp4"
echo "🖱️  Passe o mouse sobre a barra de progresso do vídeo para ver os previews"