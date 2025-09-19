use dioxus::prelude::*;

pub const FOOTER_LOGO: Asset = asset!("/assets/SJF-logo.svg");
pub const FOOTER_KLARNA: Asset = asset!("/assets/klarna.png");
pub const FOOTER_SWISH: Asset = asset!("/assets/swish.svg");
pub const FOOTER_MASTERCARD: Asset = asset!("/assets/mastercard.png");
pub const FOOTER_VISA: Asset = asset!("/assets/visa.svg");

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {
            ul {
                li {
                    Link {
                        to: crate::Route::TermsAndConditions {  },
                        "Frakt & Villkor"
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
            div {
                class: "payment-options",
                img {
                    src: FOOTER_KLARNA
                }
                img {
                    src: FOOTER_SWISH
                }
                img {
                    src: FOOTER_MASTERCARD
                }
                img {
                    src: FOOTER_VISA
                }
            }
            img {
                src: FOOTER_LOGO
            },
        }
    }
}
