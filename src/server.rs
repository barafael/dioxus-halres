use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

use crate::ServeConfigBuilder;
use chrono::Local;
use dioxus::{
    logger::tracing::{debug, error, warn},
    prelude::*,
};
use futures::StreamExt;
use reqwest::{Response, Url};
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

pub fn extract_title_and_content(document: &Document) -> (Option<String>, Option<&str>) {
    let mut title = None;
    let mut content = None;

    if let Some(the_title) = document.find(Name("title")).next() {
        title = Some(the_title.text());
    }
    if let Some(description) = document.find(Attr("name", "description")).next() {
        content = description.attr("content");
    }
    (title, content)
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

pub async fn create_entries(
    lines: impl Iterator<Item = Result<String, std::io::Error>>,
) -> Vec<(Url, String)> {
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
        let timestamp = timestamp
            .parse()
            .inspect_err(|error| {
                warn!(%error, "Failed to parse timestamp: {line:?}, assuming 'now'");
            })
            .unwrap_or_else(|_| Local::now())
            .to_rfc3339();
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
    entries
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
