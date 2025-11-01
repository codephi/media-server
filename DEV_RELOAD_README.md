# Sistema de Auto-Reload para Desenvolvimento

O media-serve agora inclui um sistema de auto-reload que monitora mudanças em arquivos durante o desenvolvimento e automaticamente recarrega a página no navegador.

## Como Usar

Para habilitar o modo de desenvolvimento com auto-reload, use o parâmetro `--watch`:

```bash
cargo run -- --watch /caminho/para/diretorio
```

Exemplo:
```bash
cargo run -- --watch .
```

## O que é Monitorado

O sistema monitora automaticamente mudanças nos seguintes diretórios:

- `templates/` - Todos os arquivos de template HTML
- `public/` - Arquivos estáticos (CSS, JavaScript, imagens, etc.)

## Como Funciona

1. **Backend**: O sistema usa a crate `notify` para monitorar mudanças no sistema de arquivos
2. **Comunicação**: Utiliza Server-Sent Events (SSE) para comunicação em tempo real entre o servidor e o navegador
3. **Frontend**: Um script JavaScript automaticamente conecta ao endpoint `/_dev/reload` e escuta por eventos de mudança
4. **Auto-reload**: Quando uma mudança é detectada, o navegador é instruído a recarregar a página

## Arquivos Adicionados/Modificados

### Backend
- `src/models/config.rs` - Adicionado parâmetro `--watch` na CLI
- `src/models/watcher.rs` - Implementação do sistema de monitoramento de arquivos
- `src/controllers/dev.rs` - Endpoint SSE para comunicação com o frontend
- `src/controllers/mod.rs` - Estrutura AppState atualizada para suportar broadcasting
- `src/main.rs` - Integração do sistema de watching
- `Cargo.toml` - Dependências adicionadas: `notify`, `async-stream`, `tokio-stream`, `futures`

### Frontend
- `public/js/dev-reload.js` - Script JavaScript para auto-reload
- `templates/base.html` - Inclusão do script de auto-reload

## Logs de Desenvolvimento

Quando o modo watch está ativo, você verá logs como:

```
INFO media_serve: Watch mode enabled - monitoring file changes
INFO media_serve::models::watcher: Watching templates/ directory for changes
INFO media_serve::models::watcher: Watching public/ directory for changes
INFO media_serve: Development reload endpoint available at /_dev/reload
INFO media_serve::models::watcher: File change detected: ["/path/to/changed/file"]
INFO media_serve::controllers::dev: Sending reload event to browser
```

## Detecção Automática

O script JavaScript automaticamente detecta se o modo de desenvolvimento está ativo tentando conectar ao endpoint `/_dev/reload`. Se o endpoint não estiver disponível (modo de produção), o auto-reload é silenciosamente desabilitado.

## Debouncing

O sistema inclui debouncing automático para evitar reloads excessivos quando múltiplos arquivos são modificados em sequência. O delay padrão é de 500ms.

## Reconexão Automática

Se a conexão SSE for perdida, o sistema automaticamente tenta reconectar com backoff exponencial, até um máximo de 3 tentativas.

## Visual Feedback

Quando um reload é acionado, uma pequena notificação visual aparece no canto superior direito da página antes do reload acontecer.

## Exemplo de Uso Completo

```bash
# Terminal 1: Iniciar o servidor em modo watch
cd /seu/projeto
cargo run -- --watch .

# Terminal 2: Fazer mudanças em arquivos
echo "/* Nova mudança */" >> public/css/app.css

# O navegador irá automaticamente recarregar quando você salvar o arquivo
```

Este sistema torna o desenvolvimento muito mais eficiente, eliminando a necessidade de recarregar manualmente o navegador a cada mudança no código.