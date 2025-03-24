use dioxus::prelude::*;

#[component]
pub fn Favorites() -> Element {
    let mut favorites = use_resource(crate::list_dogs).suspend()?;

    rsx! {
        div { id: "favorites",
            div { id: "favorites-container",
                for (id , url) in favorites().unwrap() {
                    div { key: id, class: "favorite-dog",
                        img { src: "{url}" }
                    }
                }
            }
        }
    }
}
