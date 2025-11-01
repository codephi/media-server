# Media Serve

Um servidor de arquivos de mídia em Rust com interface web nativa, arquitetura MVC tradicional, navegação de diretórios, visualização de mídia e upload de arquivos.

## Características

- 📁 **Navegação de diretórios** via URL
- 🖼️ **Dois modos de visualização**: Lista e Galeria
- 🎨 **Geração automática de thumbnails** para imagens e vídeos
- 📹 **Players nativos** para vídeo e áudio com streaming HTTP Range
- 🔍 **Visualizador de imagens** com zoom e pan
- 📤 **Upload de múltiplos arquivos**
- 🌙 **Interface dark mode** moderna
- 🔒 **Segurança**: bloqueio de path traversal
- ⚡ **Performance**: streaming de arquivos grandes sem carregar em memória

## Requisitos

- Rust 1.70+ (stable)
- ffmpeg (opcional, para thumbnails de vídeo)

## Instalação

```bash
# Clone o repositório
git clone https://github.com/seu-usuario/media-serve.git
cd media-serve

# Compile o projeto
cargo build --release

# O binário estará em target/release/media-serve
```

## Uso

### Comando básico

```bash
# Servir o diretório atual na porta 8080
media-serve .

# Servir um diretório específico
media-serve ~/Downloads

# Especificar porta customizada
media-serve ~/Videos -p 9090

# Acessível na rede local
media-serve ~/Media --bind 0.0.0.0

# Mostrar arquivos ocultos
media-serve . --show-hidden

# Configuração completa
media-serve ~/Media \
  --port 9090 \
  --bind 0.0.0.0 \
  --thumb-size 400 \
  --show-hidden \
  --log-level debug
```

### Flags disponíveis

- `BASE_DIR` - Diretório a ser servido (obrigatório)
- `-p, --port <PORT>` - Porta do servidor (padrão: 8080)
- `--bind <HOST>` - Host/IP para bind (padrão: 127.0.0.1)
- `--thumb-size <PIXELS>` - Tamanho máximo dos thumbnails (padrão: 320)
- `--show-hidden` - Mostrar arquivos ocultos por padrão
- `--log-level <LEVEL>` - Nível de log: error|warn|info|debug|trace (padrão: info)

## Endpoints

- `/` - Redireciona para /browse/
- `/browse/*path` - Navegar diretórios
- `/file/*path` - Página de visualização de arquivo
- `/download/*path` - Forçar download de arquivo
- `/content/*path` - Conteúdo bruto com suporte a HTTP Range
- `/thumbs/*path` - Thumbnails gerados sob demanda
- `/upload/*path` - Upload de arquivos (POST)
- `/static/*` - Arquivos estáticos (CSS, JS, ícones)

## Funcionalidades

### Navegação
- Breadcrumbs clicáveis para navegação rápida
- URLs diretas para qualquer subdiretório
- Alternância entre modo Lista e Galeria
- Ordenação: diretórios primeiro, depois arquivos (A-Z)

### Visualização de Mídia
- **Imagens**: Visualizador com zoom (scroll) e pan (arrastar)
- **Vídeos**: Player HTML5 com streaming (permite pular para qualquer ponto)
- **Áudio**: Player HTML5 nativo
- **PDF/Texto**: Visualização inline via iframe
- **Outros**: Ícones por tipo de arquivo

### Thumbnails
- Geração automática e cache local
- Imagens: redimensionamento mantendo proporção
- Vídeos: captura de frame em 1 segundo (requer ffmpeg)
- Cache em `<BASE_DIR>/.media-serve/thumbs/`

### Upload
- Upload de múltiplos arquivos simultaneamente
- Resolução automática de conflitos de nome (arquivo(1).jpg)
- Sanitização de nomes de arquivo

## Segurança

- **Path Traversal**: Todos os caminhos são validados e restritos ao diretório base
- **Symlinks**: Bloqueados se apontarem para fora do diretório base
- **Dotfiles**: Ocultos por padrão (pode ser habilitado via flag)
- **Upload**: Nomes de arquivo sanitizados, sem criação de diretórios

## Desenvolvimento

### Estrutura do Projeto

```
media-serve/
├── src/
│   ├── controllers/    # Handlers HTTP (MVC)
│   ├── models/         # Lógica de negócio
│   ├── views/          # Templates Askama
│   └── main.rs         # Entry point
├── templates/          # Templates HTML
├── public/            # Arquivos estáticos
│   ├── css/
│   ├── js/
│   └── icons/
└── Cargo.toml
```

### Tecnologias

- **Framework Web**: Axum
- **Templates**: Askama
- **CLI**: Clap
- **Async Runtime**: Tokio
- **Thumbnails**: image (Rust) + ffmpeg (vídeos)
- **Logging**: tracing

## Licença

MIT

## Contribuições

Contribuições são bem-vindas! Por favor, abra uma issue ou pull request.# media-server
