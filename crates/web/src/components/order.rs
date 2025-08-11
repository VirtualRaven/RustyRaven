use dioxus::prelude::*;


#[component]
pub fn OrderCompleted(uuid: ReadOnlySignal<String>) -> Element 
{
    let mut cart_state = crate::components::cart::use_cart();
    let _ = use_resource(move || async move {
        let _ =  uuid.read();
        cart_state.with_mut(|cart|{
            cart.clear();
        });
    });

    rsx! {
        div {
            h1 {
                "Order skapad {uuid}"
            }
            p {
                "Tack för dit köp! Inom kort får du en orderbekräftelse via email!"
            }
        }
    }
}

#[component]
pub fn OrderCanceled(uuid: ReadOnlySignal<String>) -> Element 
{
    rsx! {
        div {
            h1 {
                "Order {uuid} avbruten"
            }
            p {
                "Din order har avbrutits och inget köp har genomförts. Ångrar du dig finns varorna kvar i varukorgen!"
            }
        }
        
    }
}