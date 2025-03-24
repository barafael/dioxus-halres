use dioxus::prelude::*;

use crate::{Route, TitleState};

#[component]
pub fn NavBar() -> Element {
    let title = use_context::<TitleState>();
    rsx! {
        div {
            Link { to: Route::UrlList,
                h1 { "{title.0}" }
            }
        }
        Outlet::<Route> {}
    }
}
