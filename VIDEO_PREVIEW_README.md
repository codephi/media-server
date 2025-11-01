# Funcionalidade de Preview de Vídeo

## Visão Geral

Esta implementação adiciona preview de miniaturas de vídeo ao passar o mouse sobre a barra de progresso do player de vídeo.

## Como Funciona

### Backend (Rust)

1. **Módulo `video_previews.rs`**: Gerencia a criação e servimento das miniaturas de preview
   - Usa `ffprobe` para obter a duração do vídeo
   - Gera até 100 miniaturas distribuídas uniformemente pelo vídeo
   - Armazena as miniaturas em uma pasta oculta `.video-previews` no mesmo diretório do arquivo de vídeo
   - Cada pasta de cache é nomeada com hash do caminho do arquivo para evitar conflitos

2. **Controller `video_previews.rs`**: Serve as miniaturas via HTTP
   - Endpoint `/video-previews/{path}` - retorna informações do preview (JSON)
   - Endpoint `/video-previews/{path}?time={segundos}` - retorna miniatura mais próxima ao tempo especificado

3. **Cache de Miniaturas**:
   - Cada vídeo tem sua pasta de cache no formato `.video-previews/{hash}/`
   - Arquivo `info.json` contém metadados (duração, intervalo, lista de miniaturas)
   - Miniaturas em formato JPEG (160x90px) com nomes baseados no timestamp

### Frontend (JavaScript/CSS)

1. **JavaScript (`video-preview.js`)**:
   - Detecta movimento do mouse sobre a área de controles do vídeo
   - Calcula o tempo correspondente à posição do mouse
   - Carrega e exibe a miniatura correspondente em um tooltip
   - Gerencia cache de imagens blob para evitar vazamentos de memória

2. **CSS**:
   - Estilo para o tooltip de preview
   - Posicionamento responsivo que se adapta à viewport

## Estrutura de Arquivos

```
/caminho/para/video.mp4
/caminho/para/.video-previews/
  └── {hash_do_caminho}/
      ├── info.json
      ├── thumb_00000.00.jpg
      ├── thumb_00001.50.jpg
      └── ...
```

## Dependências

- **ffmpeg**: Para gerar miniaturas de vídeo
- **ffprobe**: Para obter metadados do vídeo (incluído com ffmpeg)

## Configuração

A funcionalidade é ativada automaticamente quando:
1. O ffmpeg está disponível no sistema
2. O arquivo é um vídeo suportado
3. O usuário passa o mouse sobre a barra de progresso

## Performance

- As miniaturas são geradas de forma lazy (apenas quando necessário)
- Sistema de locks evita regeneração simultânea
- Cache persistente - miniaturas são geradas apenas uma vez
- Otimizado para até 100 miniaturas por vídeo
- Compressão JPEG com qualidade balanceada

## Limitações

- Requer ffmpeg instalado no sistema
- Funciona apenas com formatos de vídeo suportados pelo ffmpeg
- As miniaturas são armazenadas localmente (não adequado para armazenamento distribuído sem modificações)

## Extensões Futuras

- Suporte a diferentes resoluções de miniatura
- Configuração do número máximo de miniaturas
- Limpeza automática de cache antigo
- Suporte a sprites de imagem para melhor performance