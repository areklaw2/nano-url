use crate::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Stylesheet { href: NAVBAR_CSS }
        div { id: "navbar",
            Link { to: Route::Home {},
                h1 { " Nano Url" }
            }
            Link { to: Route::Recent {}, "Recent Links" }
        }

        Outlet::<Route> {}
    }
}
