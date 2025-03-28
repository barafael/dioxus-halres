mod components;
#[cfg(feature = "server")]
mod server;

use std::time::Instant;

use crate::components::*;

use dioxus::{
    logger::tracing::{info, warn},
    prelude::*,
};

mod resource;

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
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    });
    Ok(uris)
}

#[server]
pub async fn import_urls() -> Result<(), ServerFnError> {
    use select::document::Document;
    use server::{create_entries, download_pages, extract_title_and_content, read_lines};

    let start = Instant::now();

    let path = "urls.csv";

    let blank_resource = resource::Resource::default();

    let lines = read_lines(path)
        .map_err(|error| ServerFnError::new(format!("Failed to open file ({error})")))?;
    let entries = create_entries(lines).await;
    let mut uris = Vec::new();
    for (url, timestamp) in entries {
        let mut uri_entry = blank_resource.clone();
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
        let (title, content) = extract_title_and_content(&document);
        uri.title = title.unwrap_or_else(|| "-".to_string());
        uri.auto_descr = content.unwrap_or("-").to_string();
    }

    for (index, uri) in uris.iter().enumerate() {
        server::DB.with(|f| {
            f.prepare("INSERT INTO uris values (?,?,?,?,?,?,?,?,?,?,?,?,?);")
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

    info!(
        "Inserted {} URIs into the database in {}s",
        uris.len(),
        start.elapsed().as_secs_f32()
    );
    Ok(())
}
