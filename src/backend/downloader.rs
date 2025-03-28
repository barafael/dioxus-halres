use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

use crate::hal_resource::HalResource;
use chrono::{Local, NaiveDate};
use dioxus::logger::tracing::{debug, error, warn};
use futures::StreamExt;
use reqwest::Response;
use select::document::Document;
use select::predicate::{Attr, Name};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub timestamp: NaiveDate,
    pub url: Url,
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

pub(crate) fn create_entries(entries: &[Input], blank_resource: HalResource) -> Vec<HalResource> {
    let mut uris = Vec::new();
    for Input { url, timestamp } in entries {
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
    uris
}

pub(crate) fn insert_resources(uris: &[HalResource]) {
    for (index, uri) in uris.iter().enumerate() {
        super::DB.with(|f| {
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
}
