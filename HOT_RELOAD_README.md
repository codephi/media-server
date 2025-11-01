# 🔥 Hot Reload no Media Server

## 🎯 Problema Identificado

O sistema de `--watch` estava detectando mudanças nos templates, mas o conteúdo não atualizava porque:

**Askama compila templates em tempo de build, não em tempo de execução.**

Isso significa que quando você altera um arquivo `.html` em `templates/`, o template já está compilado no binário e não reflete as mudanças até uma nova compilação.

## ✅ Soluções Implementadas

### 1. Script de Desenvolvimento Completo (Recomendado)

```bash
# Use este script para desenvolvimento com hot-reload completo
./dev.sh
```

**O que faz:**
- Usa `cargo-watch` para monitorar mudanças
- Recompila automaticamente quando templates ou código mudam
- Reinicia o servidor automaticamente
- Funciona com templates, CSS, JS e código Rust

**Instala automaticamente** o `cargo-watch` se necessário.

### 2. Sistema de Watch Interno Melhorado

Se você usar `cargo run -- . --watch`, agora o sistema:

✅ **Assets (CSS/JS)**: Recarrega a página automaticamente  
⚠️ **Templates (HTML)**: Mostra aviso visual que recompilação é necessária

## 🚀 Como Usar

### Para Desenvolvimento (Recomendado)
```bash
./dev.sh
```

### Para Produção
```bash
cargo run -- /caminho/para/media
```

### Para Debug do Watch
```bash
cargo run -- . --watch
```

## 📋 Tipos de Arquivo Monitorados

| Tipo | Extensão | Ação |
|------|----------|------|
| **Templates** | `.html` | ⚠️ Aviso + Sugestão de recompilação |
| **Estilos** | `.css` | 🔄 Reload automático |
| **Scripts** | `.js`, `.ts` | 🔄 Reload automático |
| **Código** | `.rs` | 🔄 Recompilação (apenas com `./dev.sh`) |

## 🔧 Como Funciona

### Sistema Interno (`--watch`)
1. **Detecção**: Monitora `templates/`, `public/` 
2. **Categorização**: Distingue entre assets e templates
3. **Notificação**: Envia eventos SSE específicos para o browser
4. **Ação**: Reload para assets, aviso para templates

### Script de Desenvolvimento (`./dev.sh`)
1. **Cargo Watch**: Monitora código e templates
2. **Recompilação**: Rebuild automático do projeto
3. **Reinício**: Servidor é reiniciado com código atualizado
4. **Hot Reload**: Funciona para tudo

## 🎨 Indicadores Visuais

- 🟢 **Verde**: Reload de assets realizado
- 🟡 **Amarelo**: Template mudou, recompilação necessária

## 💡 Dicas

1. **Use `./dev.sh`** para melhor experiência de desenvolvimento
2. **Templates grandes**: Mudanças em templates grandes podem demorar para recompilar
3. **Assets estáticos**: Mudanças em CSS/JS são instantâneas
4. **Cache do browser**: Use Ctrl+F5 se não ver mudanças