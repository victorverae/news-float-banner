use regex::Regex;
use rss::extension::{Extension, ExtensionMap};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    pub title: String,
    pub link: String,
    pub state: NewsState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NewsState {
    Breaking,
    Alert,
    Economy,
    Conflict,
    Normal,
}

fn classify_news_state(title: &str) -> NewsState {
    let t = title.to_lowercase();
    if t.contains("breaking")
        || t.contains("última hora")
        || t.contains("ultima hora")
        || t.contains("urgent")
        || t.contains("alert")
    {
        return NewsState::Breaking;
    }
    if t.contains("war")
        || t.contains("guerra")
        || t.contains("conflict")
        || t.contains("attack")
        || t.contains("missile")
    {
        return NewsState::Conflict;
    }
    if t.contains("econom")
        || t.contains("market")
        || t.contains("stock")
        || t.contains("bank")
        || t.contains("trade")
    {
        return NewsState::Economy;
    }
    if t.contains("warning")
        || t.contains("crisis")
        || t.contains("climate")
        || t.contains("storm")
    {
        return NewsState::Alert;
    }
    NewsState::Normal
}

fn img_src_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?i)src\s*=\s*["']([^"']+)["']"#).expect("img src regex")
    })
}

/// Primer <img src="..."> en HTML (p. ej. descripción del RSS de la BBC).
fn first_img_src_from_html(html: &str) -> Option<String> {
    img_src_regex()
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| s.starts_with("http"))
}

/// Hijos anidados en un [`Extension`] (`media:group`, etc.).
fn image_url_from_extension_children(map: &BTreeMap<String, Vec<Extension>>) -> Option<String> {
    for exts in map.values() {
        for ext in exts {
            if let Some(u) = ext.attrs.get("url") {
                if u.starts_with("http") {
                    return Some(u.clone());
                }
            }
            if let Some(u) = image_url_from_extension_children(&ext.children) {
                return Some(u);
            }
        }
    }
    None
}

/// Recorre `media:thumbnail` y extensiones similares (attrs `url`).
fn image_url_from_extensions(map: &ExtensionMap) -> Option<String> {
    for inner in map.values() {
        for exts in inner.values() {
            for ext in exts {
                if let Some(u) = ext.attrs.get("url") {
                    if u.starts_with("http") {
                        return Some(u.clone());
                    }
                }
                if let Some(u) = image_url_from_extension_children(&ext.children) {
                    return Some(u);
                }
            }
        }
    }
    None
}

fn extract_image_url(item: &rss::Item) -> Option<String> {
    if let Some(u) = image_url_from_extensions(item.extensions()) {
        return Some(u);
    }

    if let Some(enc) = item.enclosure() {
        let mime = enc.mime_type();
        if mime.starts_with("image/") {
            let u = enc.url();
            if u.starts_with("http") {
                return Some(u.to_string());
            }
        }
    }

    if let Some(desc) = item.description() {
        if let Some(u) = first_img_src_from_html(desc) {
            return Some(u);
        }
    }

    if let Some(content) = item.content() {
        if let Some(u) = first_img_src_from_html(content) {
            return Some(u);
        }
    }

    None
}

#[tauri::command]
async fn fetch_world_news() -> Result<Vec<NewsItem>, String> {
    const FEED_URL: &str = "https://feeds.bbci.co.uk/news/world/rss.xml";
    let client = reqwest::Client::builder()
        .user_agent("PublicidadTicker/0.1 (Tauri; educational)")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(FEED_URL)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let channel = rss::Channel::read_from(&bytes[..]).map_err(|e| e.to_string())?;

    let mut items: Vec<NewsItem> = channel
        .items()
        .iter()
        .take(24)
        .filter_map(|item| {
            let title = item.title()?.trim().to_string();
            if title.is_empty() {
                return None;
            }
            let link = item.link().unwrap_or("#").to_string();
            let state = classify_news_state(&title);
            let image_url = extract_image_url(item);
            Some(NewsItem {
                title,
                link,
                state,
                image_url,
            })
        })
        .collect();

    if items.is_empty() {
        items.push(NewsItem {
            title: "No se pudieron cargar noticias.".into(),
            link: "#".into(),
            state: NewsState::Alert,
            image_url: None,
        });
    }

    Ok(items)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![fetch_world_news])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
