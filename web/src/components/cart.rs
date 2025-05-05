
use dioxus::prelude::*;
use crate::components::MenuState;


const CART_ICON: Asset = asset!("/assets/cart.png");

#[component]
pub fn Cart() -> Element {


    let mut cart_state = use_signal(|| MenuState::Closed);
    


    let menu_class =  use_memo( move ||  {
        match *cart_state.read() {
        MenuState::Opened => "cart-open",
        _ => ""
        }
    });

    rsx! {
        div {
            class: "cart" ,
            img {
                src: CART_ICON
            }

            div {
                class: "cart-contents",
            }
        }
    }

}

