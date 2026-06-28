use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::Links { id: 1 }, "Recent Links" }
        }

        Outlet::<Route> {}
    }
}
