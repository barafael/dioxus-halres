use dioxus::prelude::*;

use crate::{Route, TitleState};

#[component]
pub fn NavBar() -> Element {
    let title = use_context::<TitleState>();
    rsx! {
        div { id: "title",
            Link { to: Route::DogView,
                h1 { "{title.0}" }
            }
            Link { to: Route::Favorites, id: "heart", "♥️" }
        }
        Outlet::<Route> {}
    }
}
