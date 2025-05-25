use dioxus::{logger::tracing::info, prelude::*};

use crate::{components::{self, MenuState}, server};

const MAIN_SCSS: Asset = asset!("/assets/styling/main.scss");
const RESET_CSS: Asset = asset!("/assets/styling/reset.css");
const HEADER_LOGO: Asset = asset!("/assets/SJF-logo2.svg");


#[component]
pub fn DynamicMenu() -> Element 
{
    let categories = use_resource(|| async move {
        server::category::get_children(None).await
    });

    info!("{:#?}", categories);

    let categories = categories.read();

    match *categories {
        Some(Ok(ref rsp)) => rsx!{
            for (id,name) in &rsp.children
            {
                li {
                    key: id,
                    "{name}"
                }
            }

        },
        Some(Err(ref e)) => rsx! {
            span {
                "Laddning misslyckades!"
            }
        },
        None => rsx! {
            span {
                "Laddar!"
            }
        }
    }


}

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
        document::Link { rel: "stylesheet", href: MAIN_SCSS }
        header {
            div {
                nav {
                    onclick: move |_| {
                        menu_state.with_mut(|state| {*state=state.toggle();} );
                    },
                    div {},
                    div {},
                    div {}
                }
                img {
                    class: "logo",
                    src: HEADER_LOGO
                },
                components::Cart {}
            },
            ul {  
                class: "menu {menu_class}",
                li { 
                    Link {to: crate::Route::Main {}, "Hem" }
                }

                DynamicMenu {}

                li { 
                    Link {to: crate::Route::Main {}, "Om SJF" }
                }
            }
        }
    }
}