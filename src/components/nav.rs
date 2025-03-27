use dioxus::prelude::*;

use crate::{import_urls, Route, TitleState};

#[component]
pub fn NavBar() -> Element {
    let title = use_context::<TitleState>();
    rsx! {
        div {
            Link { to: Route::UrlList,
                h1 { "{title.0}" }
            }
            button {
                onclick: move |_event| async move {
                    import_urls().await.unwrap();
                },
                "Load Table"
            }
        }
        Outlet::<Route> {}
    }
}
