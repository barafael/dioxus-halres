mod components;
#[cfg(feature = "server")]
mod server;

use crate::components::*;

use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/main.css");

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(NavBar)]
    #[route("/")]
    DogView,
    #[route("/favorites")]
    Favorites,
    #[route("/:..segments")]
    PageNotFound { segments: Vec<String> },
}

#[derive(serde::Deserialize)]
struct DogApi {
    message: String,
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
    let title = use_signal(|| "HotDog".to_string());
    use_context_provider(|| TitleState(title));

    rsx! {
        document::Stylesheet { href: CSS }
        Router::<Route> {}
    }
}

#[component]
fn DogView() -> Element {
    let mut img_src = use_resource(|| async move {
        reqwest::get("https://dog.ceo/api/breeds/image/random")
            .await
            .unwrap()
            .json::<DogApi>()
            .await
            .unwrap()
            .message
    });

    let update_title = move |_evt| {
        consume_context::<TitleState>()
            .0
            .set("New Title".to_string());
    };

    rsx! {
        div { id: "dogview",
            img {
                src: img_src.cloned().unwrap_or_default(),
                max_height: "300px",
            }
        }
        div { id: "buttons",
            button { onclick: update_title, id: "update_title", "Update title" }
            button { onclick: move |_| img_src.restart(), id: "skip", "skip." }
            button {
                id: "save",
                onclick: move |_| async move {
                    let current = img_src.cloned().unwrap();
                    img_src.restart();
                    _ = save_dog(current).await;
                },
                "save!"
            }
        }
    }
}

#[server]
pub async fn save_dog(url: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    server::DB.with(|f| f.execute("INSERT INTO dogs (url) VALUES (?1)", &[&url]))?;
    Ok(())
}

#[server]
pub async fn list_dogs() -> Result<Vec<(usize, String)>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let dogs = server::DB.with(|f| {
            f.prepare("SELECT id, url FROM dogs ORDER BY id DESC LIMIT 10")
                .unwrap()
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        });
        return Ok(dogs);
    }
    Ok(vec![])
}
