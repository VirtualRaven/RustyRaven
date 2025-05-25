use dioxus::prelude::*;

const LOGO: Asset = asset!("/assets/SJF-logo.svg");


#[component]
pub fn Footer() -> Element {

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
                li {
                    a {
                        href: "/admin/products",
                        "Inloggning"
                    }
                }
            },
            img {
                src: LOGO
            },
        }
    }
}