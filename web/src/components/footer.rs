use dioxus::prelude::*;

const LOGO: Asset = asset!("/assets/SJF-logo.svg");


#[component]
pub fn Footer() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        footer {
            ul {
                li {
                    "Kontakt"
                },
                li {
                    "Frakt & Vilkor"
                },
                li {
                    a {
                        href: "https://www.instagram.com/sjfconcept/",
                        "@sjfconcept"
                    }
                }
            },
            img {
                src: LOGO
            },
        }
    }
}