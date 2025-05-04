use dioxus::prelude::*;

use crate::components::{self, MenuState};

const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const MAIN_SCSS: Asset = asset!("/assets/styling/main.scss");
const RESET_CSS: Asset = asset!("/assets/styling/reset.css");

#[component]
pub fn Header() -> Element {



    let mut menu_state = use_signal(|| MenuState::Closed);


    let menu_class =  use_memo( move ||  {
        match *menu_state.read() {
        MenuState::Opened => "menu-open",
        _ => ""
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: RESET_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: MAIN_SCSS }
        header {
            div {
                nav {
                    onclick: move |_| {
                        menu_state.with_mut(|state| {*state=state.toggle();} );
                    },
                    "Menu"
                }
                h2 {
                    "SJF CONCEPT"
                },
                components::Cart {}
            },
            ul {  
                class: "{menu_class}",
                li { 
                    Link {to: crate::Route::Main {}, "Hem" }
                }
                li { 
                    Link {to: crate::Route::Main {}, "Kläder" }
                }
                li { 
                    Link {to: crate::Route::Main {}, "Gravyr" }
                }
                li { 
                    Link {to: crate::Route::Main {}, "Övrigt" }
                }
            }
        }
    }
}