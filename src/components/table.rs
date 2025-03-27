use dioxus::prelude::*;

#[component]
pub fn Table() -> Element {
    let uris = use_server_future(|| crate::load_uris_from_db())?()
        .unwrap()
        .unwrap();

    rsx! {
        div { id: "example-table" }
        button {
            onclick: move |_event| async move {
                document::eval(include_str!("../../assets/table.min.js")).await.unwrap();
            },
        }
    }
}
