use crate::{
    backend::{CreateUrlRequest, create_url, get_stats},
    components::*,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use time::Date;

const HOME_CSS: Asset = asset!("/assets/styling/home.css");

#[component]
pub fn Home() -> Element {
    let mut nano_url = use_signal(|| None::<String>);
    let mut long_url = use_signal(String::new);
    let mut alias = use_signal(|| None::<String>);
    let mut expiration = use_signal(|| None::<Date>);
    let mut stats = use_resource(move || async move { get_stats().await });

    let mut long_url_error = use_signal(|| None::<String>);
    let mut alias_error = use_signal(|| None::<String>);
    let toast = use_toast();

    let on_submit = move |e: FormEvent| {
        e.prevent_default();

        let url_err = validate_url(&long_url.read()).map(String::from);
        let alias_err = alias
            .read()
            .clone()
            .and_then(|a| validate_alias(&a).map(String::from));
        long_url_error.set(url_err.clone());
        alias_error.set(alias_err.clone());

        if url_err.is_some() || alias_err.is_some() {
            return;
        }

        spawn(async move {
            match create_url(CreateUrlRequest {
                url: long_url.cloned(),
                alias: alias.cloned(),
                expiration: expiration.cloned().map(|d| d.to_string()),
            })
            .await
            {
                Ok(link) => {
                    nano_url.set(Some(link));
                    stats.restart();
                    long_url.set(String::new());
                    alias.set(None);
                    expiration.set(None);
                    long_url_error.set(None);
                    alias_error.set(None);
                    toast.success("Link created!".into(), ToastOptions::default());
                }
                Err(e) => toast.error(e.to_string(), ToastOptions::default()),
            }
        });
    };

    rsx! {
        document::Stylesheet { href: HOME_CSS }
        div { id: "home",
            Card { id: "nano-card",
                CardHeader {
                    CardTitle { "Create a Nano Url" }
                }
                CardContent {
                    form { id: "url-form", onsubmit: on_submit,
                        div { class: "nano-form-fields",
                            div { class: "nano-field",
                                Label { html_for: "long-url", "Long Url" }
                                Input {
                                    id: "long-url",
                                    placeholder: "https://example.com",
                                    value: long_url,
                                    oninput: move |e: FormEvent| {
                                        let v = e.value();
                                        if v.is_empty() {
                                            long_url_error.set(None);
                                        } else {
                                            long_url_error.set(validate_url(&v).map(String::from));
                                        }
                                        long_url.set(v);
                                    },
                                }
                                if let Some(err) = long_url_error.read().as_deref() {
                                    p { class: "field-error", "{err}" }
                                }
                            }
                            div { class: "nano-field",
                                Label { html_for: "alias", "Alias (optional)" }
                                Input {
                                    id: "alias",
                                    placeholder: "Alias",
                                    value: alias,
                                    oninput: move |e: FormEvent| {
                                        let v = e.value();
                                        if v.is_empty() {
                                            alias.set(None);
                                            alias_error.set(None);
                                        } else {
                                            alias_error.set(validate_alias(&v).map(String::from));
                                            alias.set(Some(v));
                                        }
                                    },
                                }
                                if let Some(err) = alias_error.read().as_deref() {
                                    p { class: "field-error", "{err}" }
                                }
                            }
                            div { class: "nano-field",
                                Label { html_for: "expiration", "Expiration (optional)" }
                                DatePicker {
                                    id: "expiration",
                                    selected_date: expiration,
                                    on_value_change: move |v| {
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
            if let Some(nano_url) = nano_url.read().as_deref() {
                Card { id: "link-card",
                    CardHeader {
                        CardTitle { "Your Nano Link" }
                    }
                    CardContent {
                        a { class: "link", href: "{nano_url}", "{nano_url}" }
                    }
                }
            }
            Card { id: "stats-card",
                CardHeader {
                    CardTitle { "Stats" }
                }
                CardContent {
                    {
                        match &*stats.read() {
                            Some(Ok(s)) => rsx! {
                                p { class: "stats", "Total Links Created: {s.total_links}" }
                                p { class: "stats", "Total Redirects: {s.total_redirects}" }
                            },
                            _ => rsx! {
                                p { class: "stats", "Total Links Created: …" }
                                p { class: "stats", "Total Redirects: …" }
                            },
                        }
                    }
                }
            }
        }
    }
}

fn validate_url(url: &str) -> Option<&'static str> {
    if url.trim().is_empty() {
        Some("URL is required")
    } else if url.contains(char::is_whitespace) {
        Some("URL must not contain whitespace")
    } else {
        None
    }
}

fn validate_alias(alias: &str) -> Option<&'static str> {
    if alias.len() < 5 {
        Some("Alias must be at least 5 characters")
    } else if !alias
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        Some("Alias can only contain letters, numbers, - and _")
    } else {
        None
    }
}
