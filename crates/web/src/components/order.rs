use dioxus::prelude::*;

#[component]
pub fn OrderCompleted(uuid: ReadOnlySignal<String>) -> Element {
    rsx! {
        div {
            class: "about",
            h2 {"Order skapad!"}

            p {
                "Tack för dit köp! Inom kort får du en orderbekräftelse via email!"
            }

            p { "Har du några tankar eller funderingar så kontakta oss på: sjfconcept@hotmail.com"}
            p {
                class:"order-reference",
                "Referens: {uuid}"
            }

            div {
                img {
                    src:  crate::components::footer::FOOTER_LOGO
                },
            }
        }
    }
}

#[component]
pub fn OrderCanceled(uuid: ReadOnlySignal<String>) -> Element {
    rsx! {
        div {
            class: "about",
            h2 {"Order avbruten!"}

            p {
                "Köpet har avbrutits och du har inte blivit debiterad. Om du skulle ångra dig så finns dina varor kvar i varukorgen redo att beställas."
            }

            p { "Har du några tankar eller funderingar så kontakta oss på: sjfconcept@hotmail.com"}
            p {
                class:"order-reference",
                "Referens: {uuid}"
            }

            div {
                img {
                    src:  crate::components::footer::FOOTER_LOGO
                },
            }
        }

    }
}
