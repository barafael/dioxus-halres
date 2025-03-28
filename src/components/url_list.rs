use dioxus::prelude::*;

#[component]
pub fn UrlList() -> Element {
    let uris = use_server_future(crate::load_uris_from_db)?()
        .unwrap()
        .unwrap();

    rsx! {
        h3 { "urls" }
        ul {
            for url in uris {
                li { "{url}" }
            }
        }
    }
}
