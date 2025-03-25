mod components;
#[cfg(feature = "server")]
mod server;

use crate::components::*;

use dioxus::prelude::*;

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
pub async fn first_uris_in_db() -> Result<Vec<String>, ServerFnError> {
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
