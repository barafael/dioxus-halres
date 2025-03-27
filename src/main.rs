mod components;
#[cfg(feature = "server")]
mod server;

use crate::components::*;

use dioxus::{
    logger::tracing::{debug, error, warn},
    prelude::*,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct URI {
    uri_uuid: String,
    url: String,
    scheme: String,
    host: String,
    path: String,
    live_status: String,
    title: String,
    auto_descr: String,
    man_descr: String,
    crea_user: String,
    crea_time: String,
    modi_user: String,
    modi_time: String,
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(NavBar)]
    #[route("/")]
    Table,
    #[route("/uris")]
    UrlList,
    #[route("/:..segments")]
    PageNotFound { segments: Vec<String> },
}

#[derive(Clone)]
pub struct TitleState(Signal<String>);

fn main() {
    #[cfg(feature = "server")]
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(server::launch());
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    let title = use_signal(|| "Berlin Rust Hack&Learn Web Resources".to_string());
    use_context_provider(|| TitleState(title));

    rsx! {
        Router::<Route> {}
    }
}

#[server]
pub async fn load_uris_from_db() -> Result<Vec<String>, ServerFnError> {
    let uris = server::DB.with(|f| {
        f.prepare("SELECT id, url FROM uris")
            .unwrap()
            .query_map([], |row| Ok(row.get(1)?))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    });
    return Ok(uris);
}

#[server]
pub async fn import_urls() -> Result<(), ServerFnError> {
    use chrono::Local;
    use reqwest::Url;
    use select::document::Document;
    use server::extract_title_and_content;
    use server::{download_pages, read_lines};

    let path = "urls.csv";

    let default_uri_entry = URI {
        uri_uuid: "".to_string(),
        url: "-".to_string(),
        scheme: "-".to_string(),
        host: "-".to_string(),
        path: "-".to_string(),
        live_status: "1".to_string(),
        title: "-".to_string(),
        auto_descr: "-".to_string(),
        man_descr: "".to_string(),
        crea_time: "".to_string(),
        crea_user: "api".to_string(),
        modi_time: "".to_string(),
        modi_user: "api".to_string(),
    };

    let lines = read_lines(path).map_err(|e| ServerFnError::new("Failed to open file"))?;
    let mut entries = Vec::new();
    for line in lines {
        let Ok(line) = line else {
            debug!("Failed to parse line: {line:?}");
            continue;
        };

        let Some(timestamp) = line.split('\t').next() else {
            warn!("No timestamp: {line:?}");
            continue;
        };
        let timestamp = timestamp.parse().unwrap_or(Local::now()).to_rfc3339();
        let Some(url_str) = line.split('\t').nth(1) else {
            warn!("No URL: {line:?}");
            continue;
        };
        let url_str = url_str.trim();
        let Ok(parsed_url) = Url::parse(url_str) else {
            error!("Ill-formed URL: {}", url_str);
            continue;
        };
        entries.push((parsed_url, timestamp));
    }

    let mut uris = Vec::new();
    for (url, timestamp) in entries {
        let mut uri_entry = default_uri_entry.clone();
        uri_entry.url = url.as_str().into();
        uri_entry.uri_uuid = blake3::hash(uri_entry.url.as_bytes()).to_hex().to_string();
        uri_entry.scheme = url.scheme().into();
        uri_entry.host = url.host_str().unwrap_or("-").into();
        uri_entry.path = url.path().into();
        uri_entry.crea_time = timestamp.to_string();
        uri_entry.modi_time = timestamp.to_string();
        uris.push(uri_entry);
    }

    let pages = download_pages(uris.iter().map(|u| u.url.clone()).collect()).await;
    let pages = pages
        .into_iter()
        .filter_map(|page| match page {
            Ok(page) => Some(page),
            Err(error) => {
                warn!(%error, "Fetch failure");
                None
            }
        })
        .collect::<Vec<_>>();

    for (page, uri) in pages.into_iter().zip(uris.iter_mut()) {
        let Ok(body) = page.text().await else {
            uri.live_status = "0".to_string();
            warn!("No response from URL: {:?}", uri.url);
            continue;
        };
        let document = Document::from(body.as_str());
        let (title, content) = extract_title_and_content(&document).unwrap();
        uri.title = title.unwrap_or_else(|| "-".to_string());
        uri.auto_descr = content.unwrap_or_else(|| "-").to_string();
    }

    for (index, uri) in uris.iter().enumerate() {
        server::DB.with(|f| {
            f.prepare(r#"INSERT INTO uris values (?,?,?,?,?,?,?,?,?,?,?,?,?);"#)
                .unwrap()
                .execute(rusqlite::params![
                    index.to_string(),
                    uri.url,
                    uri.scheme,
                    uri.host,
                    uri.path,
                    uri.live_status,
                    uri.title,
                    uri.auto_descr,
                    uri.man_descr,
                    uri.crea_user,
                    uri.crea_time,
                    uri.modi_user,
                    uri.modi_time
                ])
                .unwrap();
        });
    }

    Ok(())
}
