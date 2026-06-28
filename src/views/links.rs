use crate::Route;
use dioxus::prelude::*;

/// The Blog page component that will be rendered when the current route is `[Route::Blog]`
///
/// The component takes a `id` prop of type `i32` from the route enum. Whenever the id changes, the component function will be
/// re-run and the rendered HTML will be updated.
#[component]
pub fn Links() -> Element {
    rsx! {
        div { id: "links",

            // Content
            h1 { "This is blog !" }
            p {
                "In blog #, we show how the Dioxus router works and how URL parameters can be passed as props to our route components."
            }

        // Navigation links
        // The `Link` component lets us link to other routes inside our app. It takes a `to` prop of type `Route` and
        }
    }
}
