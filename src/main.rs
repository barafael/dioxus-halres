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
    use backend::downloader::{insert_resources, make_entry};
    let start = Instant::now();

    let path = "urls.csv";

    // parse csv file to an [`Input`] struct.
    let file = File::open(path)
        .map_err(|error| ServerFnError::new(format!("Failed to open file ({error})")))?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(file);

    let (sender, mut receiver) = halres_downloader::run(64, 12);

    for record in rdr.deserialize() {
        match record {
            Ok(record) => sender.send(record).await.unwrap(),
            Err(error) => {
                warn!(%error, "invalid record");
                continue;
            }
        }
    }
    drop(sender);

    let mut uris = vec![];
    while let Some(res) = receiver.recv().await {
        uris.push(res);
    }

    let uris: Vec<_> = uris.into_iter().map(make_entry).collect();

    insert_resources(&uris);

    info!(
        "Inserted {} URIs into the database in {}s",
        uris.len(),
        start.elapsed().as_secs_f32()
    );
    Ok(())
}
