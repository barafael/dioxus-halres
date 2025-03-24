use dioxus::prelude::*;

#[component]
pub fn PageNotFound(segments: Vec<String>) -> Element {
    rsx! {
        div { id: "page-not-found",
            h1 { "404" }
            p { "Page not found." }
            ul {
                for segment in &segments {
                    li { "{segment}" }
                }
            }
        }
    }
}
