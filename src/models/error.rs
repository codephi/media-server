use askama_axum::Template;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    error_title: String,
    error_description: String,
    error_type: String,
    status_code: u16,
    breadcrumbs: Vec<crate::models::fs::Breadcrumb>,
    suggestions: Vec<String>,
    show_details: bool,
    technical_details: Option<String>,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_title, error_description, error_type, suggestions, technical_details) =
            match &self {
                AppError::BadRequest(msg) => {
                    tracing::warn!("Bad request: {}", msg);
                    (
                        StatusCode::BAD_REQUEST,
                        "Solicitação Inválida".to_string(),
                        msg.clone(),
                        "bad_request".to_string(),
                        vec![
                            "Verifique se os parâmetros estão corretos".to_string(),
                            "Tente novamente com dados válidos".to_string(),
                        ],
                        Some(format!("Erro técnico: {}", msg)),
                    )
                }
                AppError::Forbidden(msg) => {
                    tracing::warn!("Forbidden: {}", msg);
                    (
                        StatusCode::FORBIDDEN,
                        "Acesso Negado".to_string(),
                        "Você não tem permissão para acessar este recurso.".to_string(),
                        "forbidden".to_string(),
                        vec![
                            "Verifique se você tem as permissões necessárias".to_string(),
                            "Entre em contato com o administrador do sistema".to_string(),
                        ],
                        Some(format!("Erro técnico: {}", msg)),
                    )
                }
                AppError::NotFound(msg) => {
                    tracing::debug!("Not found: {}", msg);
                    (
                        StatusCode::NOT_FOUND,
                        "Arquivo Não Encontrado".to_string(),
                        "O arquivo ou diretório que você está procurando não existe.".to_string(),
                        "not_found".to_string(),
                        vec![
                            "Verifique se o caminho está correto".to_string(),
                            "O arquivo pode ter sido movido ou removido".to_string(),
                            "Use a busca para encontrar o que procura".to_string(),
                        ],
                        Some(format!("Erro técnico: {}", msg)),
                    )
                }
                AppError::Internal(msg) => {
                    tracing::error!("Internal error: {}", msg);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Erro Interno do Servidor".to_string(),
                        "Ocorreu um erro interno. Nossa equipe foi notificada.".to_string(),
                        "internal".to_string(),
                        vec![
                            "Tente novamente em alguns instantes".to_string(),
                            "Se o problema persistir, entre em contato conosco".to_string(),
                        ],
                        Some(format!("Erro técnico: {}", msg)),
                    )
                }
                AppError::Io(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                    tracing::debug!("IO not found: {}", e);
                    (
                        StatusCode::NOT_FOUND,
                        "Arquivo Não Encontrado".to_string(),
                        "O arquivo que você está tentando acessar não foi encontrado.".to_string(),
                        "not_found".to_string(),
                        vec![
                            "Verifique se o arquivo existe no local especificado".to_string(),
                            "O arquivo pode ter sido movido ou removido".to_string(),
                        ],
                        Some(format!("Erro de E/S: {}", e)),
                    )
                }
                AppError::Io(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    tracing::error!("Permission denied: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Erro de Permissão".to_string(),
                        "O servidor não tem permissão para acessar este arquivo ou diretório."
                            .to_string(),
                        "permission".to_string(),
                        vec![
                            "Este pode ser um problema temporário de sistema".to_string(),
                            "Verifique se o arquivo não está em uso por outro processo".to_string(),
                            "Tente novamente em alguns instantes".to_string(),
                            "Entre em contato com o administrador se o problema persistir"
                                .to_string(),
                        ],
                        Some(format!("Erro de permissão do sistema: {}", e)),
                    )
                }
                AppError::Io(e) => {
                    tracing::error!("IO error: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Erro de Entrada/Saída".to_string(),
                        "Ocorreu um erro ao acessar o sistema de arquivos.".to_string(),
                        "io_error".to_string(),
                        vec![
                            "Tente novamente em alguns instantes".to_string(),
                            "Verifique se há espaço suficiente em disco".to_string(),
                            "Entre em contato com o administrador se necessário".to_string(),
                        ],
                        Some(format!("Erro de E/S: {}", e)),
                    )
                }
                AppError::Other(e) => {
                    tracing::error!("Other error: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Erro Interno".to_string(),
                        "Ocorreu um erro inesperado. Nossa equipe foi notificada.".to_string(),
                        "other".to_string(),
                        vec![
                            "Tente novamente em alguns instantes".to_string(),
                            "Se o problema persistir, reporte o ocorrido".to_string(),
                        ],
                        Some(format!("Erro geral: {}", e)),
                    )
                }
            };

        // Criar template de erro
        let template = ErrorTemplate {
            error_title,
            error_description,
            error_type,
            status_code: status.as_u16(),
            breadcrumbs: vec![], // Pode ser expandido para incluir breadcrumbs do contexto
            suggestions,
            show_details: cfg!(debug_assertions), // Mostrar detalhes apenas em modo debug
            technical_details,
        };

        // Tentar renderizar o template
        match template.render() {
            Ok(html) => Response::builder()
                .status(status)
                .header("content-type", "text/html; charset=utf-8")
                .body(html.into())
                .unwrap(),
            Err(e) => {
                // Fallback para resposta simples se o template falhar
                tracing::error!("Failed to render error template: {}", e);
                (status, format!("Erro: {}", self)).into_response()
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
