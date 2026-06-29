use dioxus::prelude::*;

use components::ToastProvider;
use views::{Home, Links, Navbar};

mod backend;
mod components;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/app/links")]
    Links {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const DX_COMPONENT_CSS: Asset = asset!("/assets/dx-components-theme.css");

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        use dioxus::server::axum::routing::get;
        let router = dioxus::server::router(App).route("/{hash}", get(backend::redirect));
        Ok(router)
    });
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: DX_COMPONENT_CSS }
        ToastProvider { Router::<Route> {} }
    }
}
