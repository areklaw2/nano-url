use crate::{backend::get_recent_links, components::*};
use dioxus::prelude::*;

const RECENT_CSS: Asset = asset!("/assets/styling/recent.css");

#[component]
pub fn Recent() -> Element {
    let recent = use_resource(move || async move { get_recent_links().await });

    rsx! {
        document::Stylesheet { href: RECENT_CSS }
        div { id: "recent",
            h1 { class: "recent-title", "Recently Created Links" }
            {
                match &*recent.read() {
                    Some(Ok(links)) if !links.is_empty() => rsx! {
                        div { class: "recent-list",
                            for link in links.iter() {
                                Card { key: "{link.short}",
                                    CardContent {
                                        div { class: "recent-item",
                                            a { class: "recent-short", href: "{link.short}", "{link.short}" }
                                            span { class: "recent-url", "{link.url}" }
                                            span { class: "recent-clicks", "{link.clicks} clicks" }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(Ok(_)) => rsx! {
                        p { class: "recent-empty", "No links yet — make one!" }
                    },
                    Some(Err(_)) => rsx! {
                        p { class: "recent-empty", "Failed to load recent links." }
                    },
                    None => rsx! {
                        p { class: "recent-empty", "Loading…" }
                    },
                }
            }
        }
    }
}
