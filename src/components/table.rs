use dioxus::prelude::*;

#[component]
pub fn Table() -> Element {
    rsx! {
        div { id: "example-table" }
        button {
            onclick: move |_event| async move {
                let _ = document::eval(include_str!("../../assets/table.min.js")).await;
            },
        }
    }
}
