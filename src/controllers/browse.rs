use askama_axum::Template;
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
};
use serde::{Deserialize, Deserializer};
use std::sync::Arc;

use crate::controllers::AppState;
use crate::models::{fs, media, Result};

#[derive(Debug, Deserialize)]
pub struct BrowseQuery {
    #[serde(default = "default_view")]
    view: String,
    #[serde(default, deserialize_with = "deserialize_bool_from_anything")]
    show_hidden: Option<bool>,
}

fn default_view() -> String {
    "list".to_string()
}

fn deserialize_bool_from_anything<'de, D>(deserializer: D) -> std::result::Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        None => Ok(None),
        Some(s) => match s.as_str() {
            "true" | "1" | "yes" | "on" => Ok(Some(true)),
            "false" | "0" | "no" | "off" | "" => Ok(Some(false)),
            _ => Ok(None),
        },
    }
}

#[derive(Template)]
#[template(path = "browse.html")]
struct BrowseTemplate {
    items: Vec<ItemView>,
    breadcrumbs: Vec<fs::Breadcrumb>,
    current_path: String,
    view_mode: String,
    show_hidden: bool,
    thumb_size: u32,
}

#[derive(Debug)]
struct ItemView {
    name: String,
    rel_path: String,
    encoded_path: String,
    is_dir: bool,
    size: String,
    modified: String,
    icon: String,
    has_thumbnail: bool,
    media_kind: media::MediaKind,
}

impl From<fs::Item> for ItemView {
    fn from(item: fs::Item) -> Self {
        let path = if item.is_dir {
            item.rel_path.clone()
        } else {
            item.rel_path.clone()
        };
        
        let (_, media_kind) = if !item.is_dir {
            let full_path = std::path::Path::new(&item.rel_path);
            media::detect(full_path)
        } else {
            ("".to_string(), media::MediaKind::Other)
        };
        
        let icon = if item.is_dir {
            "folder.svg".to_string()
        } else {
            media_kind.icon_name().to_string()
        };
        
        Self {
            name: item.name,
            encoded_path: fs::url_encode_path(&path),
            rel_path: path,
            is_dir: item.is_dir,
            size: if item.is_dir {
                "-".to_string()
            } else {
                fs::format_size(item.size)
            },
            modified: fs::format_datetime(&item.modified),
            icon,
            has_thumbnail: !item.is_dir && media_kind.has_thumbnail(),
            media_kind,
        }
    }
}

/// Redirect root to /browse/
pub async fn root_redirect() -> impl IntoResponse {
    Redirect::permanent("/browse/")
}

/// Browse directory
pub async fn browse(
    State(state): State<Arc<AppState>>,
    path: Option<Path<String>>,
    Query(query): Query<BrowseQuery>,
) -> Result<impl IntoResponse> {
    let path = path.map(|Path(p)| p).unwrap_or_default();
    let path = path.trim_matches('/');
    let show_hidden = query.show_hidden.unwrap_or(state.config.show_hidden);
    
    // List directory contents
    let items = fs::list_dir(&state.config.base_dir_canonical, path, show_hidden)?;
    let breadcrumbs = fs::breadcrumbs(path);
    
    // Convert items to view models
    let items: Vec<ItemView> = items.into_iter().map(ItemView::from).collect();
    
    let template = BrowseTemplate {
        items,
        breadcrumbs,
        current_path: path.to_string(),
        view_mode: query.view,
        show_hidden,
        thumb_size: state.config.thumb_size,
    };
    
    Ok(template)
}