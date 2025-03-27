use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

use crate::{ServeConfigBuilder, URI};
use dioxus::{logger::tracing::warn, prelude::*};
use futures::StreamExt;
use reqwest::Response;
use select::document::Document;
use select::predicate::{Attr, Name};
use tokio::net::TcpListener;

thread_local! {
    pub static DB: rusqlite::Connection = {
        let conn = rusqlite::Connection::open("halreslib.sqlite").unwrap();

        conn.execute_batch(
            include_str!("../create.sql")
        ).unwrap();

        conn
    }
}

pub async fn poke_page(uri_entry: &mut URI) -> Result<(), anyhow::Error> {
    match reqwest::get(&uri_entry.url).await {
        Ok(response) if response.status().is_success() => {
            let body = response.text().await?;
            let document = Document::from(body.as_str());
            if let Some(title) = document.find(Name("title")).next() {
                uri_entry.title = title.text();
            }
            if let Some(description) = document.find(Attr("name", "description")).next() {
                if let Some(content) = description.attr("content") {
                    uri_entry.auto_descr = content.to_string();
                }
            }
        }
        Ok(response) => warn!(
            "Error {} while retrieving:\n  {}",
            response.status(),
            uri_entry.url
        ),
        Err(error) => {
            uri_entry.live_status = "0".to_string();
            warn!("No response from URL ({error}):\n  {}", uri_entry.url);
        }
    }
    Ok(())
}

pub fn extract_title_and_content(
    document: &Document,
) -> Result<(Option<String>, Option<&str>), anyhow::Error> {
    let mut title = None;
    let mut content = None;

    if let Some(the_title) = document.find(Name("title")).next() {
        title = Some(the_title.text());
    }
    if let Some(description) = document.find(Attr("name", "description")).next() {
        content = description.attr("content");
    }
    Ok((title, content))
}

pub async fn download_pages(paths: Vec<String>) -> Vec<Result<Response, reqwest::Error>> {
    let fetches = futures::stream::iter(paths.into_iter())
        .map(|page| async move { reqwest::get(page).await })
        .buffer_unordered(16)
        .collect::<Vec<Result<Response, reqwest::Error>>>();
    fetches.await
}

pub fn read_lines(filename: impl AsRef<Path>) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub async fn launch() {
    dioxus::logger::initialize_default();

    let socket_addr = dioxus_cli_config::fullstack_address_or_localhost();

    let router = axum::Router::new()
        .serve_dioxus_application(ServeConfigBuilder::new(), super::App)
        .into_make_service();

    let listener = TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
