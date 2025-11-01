use askama_axum::Template;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

use crate::controllers::AppState;
use crate::models::{fs, AppError, Result};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub show_hidden: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub name: String,
    pub rel_path: String,
    pub encoded_path: String,
    pub is_dir: bool,
    pub size: String,
    pub modified: String,
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResultItem>>> {
    let q = query.q.trim();
    if q.is_empty() {
        return Err(AppError::BadRequest("parâmetro 'q' é obrigatório".into()));
    }

    let limit = query.limit.unwrap_or(200).min(10_000);
    let show_hidden = query.show_hidden.unwrap_or(state.config.show_hidden);
    let base = state.config.base_dir_canonical.clone();
    let needle = q.to_lowercase();

    let results =
        tokio::task::spawn_blocking(move || search_fs(&base, &needle, show_hidden, limit))
            .await
            .map_err(|e| AppError::Internal(format!("join error: {}", e)))??;

    Ok(Json(results))
}

fn search_fs(
    base: &PathBuf,
    needle: &str,
    show_hidden: bool,
    limit: usize,
) -> Result<Vec<SearchResultItem>> {
    let mut out = Vec::with_capacity(limit.min(256));
    let mut stack = vec![base.clone()];

    while let Some(dir) = stack.pop() {
        for entry_res in std::fs::read_dir(&dir)? {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
            let name_os = entry.file_name();
            let name = name_os.to_string_lossy().into_owned();

            // Hidden filter
            if !show_hidden && fs::is_hidden(&name) {
                continue;
            }

            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };

            // If directory, consider pushing to stack (skip symlinks)
            if file_type.is_dir() {
                // match directory name
                if name.to_lowercase().contains(needle) {
                    let meta = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
                    let abs = entry.path();
                    let rel = fs::relative_from_base(base, &abs)?;
                    out.push(SearchResultItem {
                        name: name.clone(),
                        rel_path: rel.clone(),
                        encoded_path: fs::url_encode_path(&rel),
                        is_dir: true,
                        size: "-".into(),
                        modified: fs::format_datetime(&time::OffsetDateTime::from(
                            meta.modified()?,
                        )),
                    });
                    if out.len() >= limit {
                        return Ok(out);
                    }
                }

                stack.push(entry.path());
                continue;
            }

            if file_type.is_symlink() {
                continue;
            }

            // Files
            if name.to_lowercase().contains(needle) {
                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let abs = entry.path();
                let rel = fs::relative_from_base(base, &abs)?;
                out.push(SearchResultItem {
                    name: name.clone(),
                    rel_path: rel.clone(),
                    encoded_path: fs::url_encode_path(&rel),
                    is_dir: false,
                    size: fs::format_size(meta.len()),
                    modified: fs::format_datetime(&time::OffsetDateTime::from(meta.modified()?)),
                });
                if out.len() >= limit {
                    return Ok(out);
                }
            }
        }
    }

    Ok(out)
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchPageTemplate {
    breadcrumbs: Vec<crate::models::fs::Breadcrumb>,
    q: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchPageQuery {
    #[serde(default)]
    pub q: String,
}

/// Serves the search page (HTML). The actual results are fetched via /search JSON endpoint.
pub async fn search_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchPageQuery>,
) -> Result<impl axum::response::IntoResponse> {
    let crumbs = crate::models::fs::breadcrumbs("");
    let template = SearchPageTemplate {
        breadcrumbs: crumbs,
        q: query.q.clone(),
    };
    Ok(template)
}
