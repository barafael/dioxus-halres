use dioxus::prelude::*;

#[component]
pub fn UrlList() -> Element {
    let uris = use_server_future(|| crate::first_uris_in_db())?()
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
