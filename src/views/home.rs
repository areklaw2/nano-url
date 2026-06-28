use crate::components::*;
use dioxus::prelude::*;
use time::Date;

const HOME_CSS: Asset = asset!("/assets/styling/home.css");

#[component]
pub fn Home() -> Element {
    let mut long_url = use_signal(String::new);
    let mut expiration = use_signal(|| None::<Date>);
    let total_links_created = use_signal(|| 0u32);
    let total_redirects = use_signal(|| 0u32);

    rsx! {
        div { id: "home",
            document::Stylesheet { href: HOME_CSS }
            Card { id: "nano-card",
                CardHeader {
                    CardTitle { "Create a Nano Url" }
                }
                CardContent {
                    form { id: "url-form",
                        div { class: "nano-form-fields",
                            div { class: "nano-field",
                                Label { html_for: "long-url", "Long Url" }
                                Input {
                                    id: "long-url",
                                    placeholder: "https://example.com",
                                    value: long_url,
                                    oninput: move |e: FormEvent| long_url.set(e.value()),
                                }
                            }
                            div { class: "nano-field",
                                Label { html_for: "expiration", "Expiration (optional)" }
                                DatePicker {
                                    id: "expiration",
                                    selected_date: expiration,
                                    on_value_change: move |v| {
                                        println!("Selected date changed: {:?}", v);
                                        expiration.set(v);
                                    },
                                }
                            }
                        }
                    }
                }
                CardFooter { id: "nano-footer",
                    Button { form: "url-form", id: "nano-submit", "Make Nano" }
                }
            }

            div { id: "stats",
                "Stats"
                ul {
                    li { "Total Links Created {total_links_created}" }
                    li { "Total Redirects {total_redirects}" }
                }
            }
        }
    }
}
