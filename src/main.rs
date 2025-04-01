#[cfg(feature = "server")]
mod backend;
mod components;

use std::{fs::File, time::Instant};

use crate::components::*;

use dioxus::{
    logger::tracing::{info, warn},
    prelude::*,
};

mod hal_resource;

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
        .block_on(backend::launch());
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    let title = use_signal(|| "Berlin Rust Hack&Learn Web Resources".to_string());
    use_context_provider(|| TitleState(title));

    rsx! {
        document::Link {
            href: "https://unpkg.com/tabulator-tables/dist/css/tabulator.min.css",
            rel: "stylesheet",
        }
        Router::<Route> {}
    }
}

#[server]
pub async fn load_uris_from_db() -> Result<Vec<String>, ServerFnError> {
    let uris = backend::DB.with(|f| {
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
    use backend::downloader::{
        Input, create_entries, download_pages, extract_title_and_content, insert_resources,
    };
    use select::document::Document;

    let start = Instant::now();

    let path = "urls.csv";

    let blank_resource = hal_resource::HalResource::default();

    // parse csv file to an [`Input`] struct.
    let file = File::open(path)
        .map_err(|error| ServerFnError::new(format!("Failed to open file ({error})")))?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(file);

    let mut entries: Vec<Input> = Vec::new();
    for result in rdr.deserialize() {
        let record = match result {
            Err(error) => {
                warn!(%error, "Failed to parse record");
                continue;
            }
            Ok(record) => record,
        };
        info!("Parsed record: {:?}", record);
        entries.push(record);
    }

    let mut uris = create_entries(&entries, &blank_resource);

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

    insert_resources(&uris);

    info!(
        "Inserted {} URIs into the database in {}s",
        uris.len(),
        start.elapsed().as_secs_f32()
    );
    Ok(())
}
