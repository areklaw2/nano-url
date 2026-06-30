use dioxus::prelude::*;

use components::ToastProvider;
use views::{Home, Navbar, Recent};

mod backend;
mod components;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/app/recent")]
    Recent {},
}

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
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: DX_COMPONENT_CSS }
        ToastProvider { Router::<Route> {} }
    }
}
