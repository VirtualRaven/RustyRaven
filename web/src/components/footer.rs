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
                    Link {
                        to: "https://www.instagram.com/sjfconcept/",
                        "@sjfconcept"
                    }
                }
                li {
                    Link {
                        to: crate::Route::CategoryList {},
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