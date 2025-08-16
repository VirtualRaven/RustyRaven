use dioxus::prelude::*;

pub const FOOTER_LOGO: Asset = asset!("/assets/SJF-logo.svg");

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {
            ul {
                li {
                    Link {
                        to: crate::Route::TermsAndConditions {  },
                        "Frakt & Vilkor"
                    }
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
                src: FOOTER_LOGO
            },
        }
    }
}
