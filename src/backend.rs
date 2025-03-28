pub mod downloader;

use crate::ServeConfigBuilder;
use dioxus::prelude::*;
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

pub async fn launch() {
    dioxus::logger::initialize_default();

    let socket_addr = dioxus_cli_config::fullstack_address_or_localhost();

    let router = axum::Router::new()
        .serve_dioxus_application(ServeConfigBuilder::new(), super::App)
        .into_make_service();

    let listener = TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
