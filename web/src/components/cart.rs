
use dioxus::prelude::*;
use crate::components::MenuState;



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
            span {
                "CART!!!"
            }

            div {
                class: "cart-contents",
            }
        }
    }

}

