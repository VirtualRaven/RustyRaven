use dioxus::html::details::open;
use dioxus::signals::Signal;
use dioxus::prelude::*;
use dioxus::logger::tracing::{info, warn};

use crate::components;
use crate::server::Product;

#[component]
pub fn SaveButton( product : Signal<Product>) -> Element {

    enum State {
        Idle,
        Saving,
        Saved(Product),
        Error(Product)
    }


    let mut saving_state = use_signal(|| State::Idle);
    let is_unedited = use_memo(move || *product.read() == Default::default() );
    

    let changed_since_saved = use_memo( move || {
         let current_product = &*product.read();
         match *saving_state.read() 
         {
             State::Saved(ref saved_product) => saved_product != current_product,
             State::Error(ref failed_product)=> failed_product != current_product,
             _ => false
         }
    });

    //Reset internal state if product changes
    use_effect( move ||  {
        if *changed_since_saved.read() {
            saving_state.set(State::Idle);
        }
    });


    let button_text = use_memo(move || {
        match *saving_state.read()
        {
            State::Idle => "Spara",
            State::Saving => "Sparar...",
            State::Saved(_) => "Sparad!",
            State::Error(_) => "Sparning misslyckades"
        }
    } );


    let button_class = match *saving_state.read()
    {
        State::Error(_) => "red",
        _ => "green"
    };

    rsx! {
        button {
            disabled: is_unedited,
            class: "{button_class}",
            onclick: move |_| { 
                to_owned![product,saving_state];
                spawn( async move  {  
                    use crate::server::*;
                    saving_state.set(State::Saving);
                    let rsp = store_product(
                        AuthenticatedRequest {
                            data: product.read().clone()
                        }
                    ).await;

                    match rsp {
                        Ok(id) => {
                            product.write().id = Some(id);
                            saving_state.set(State::Saved(product.read().clone()));
                        },
                        Err(e) => {
                            saving_state.set(State::Error(product.read().clone()) );
                            warn!("Failed to save changes {:#?}", e);
                        }
                    }
                });
            },
            {button_text}
        }
    }

}

#[component]
fn ProductName(product: Signal<Product> )-> Element
{

    rsx! {
            label {
                "Produkt namn"
            }
            input {
                type: "text",
                value: "{product.read().name}",
                oninput: move |evt| {
                    product.write().name = {
                        if evt.value().len() > 100 
                        {
                            evt.value()[0..100].to_string()
                        }
                        else {
                            evt.value()
                        }
                    };                    


                }
            },
    }
}
#[component]
fn ProductInventory(product: Signal<Product> )-> Element
{
    rsx!{ 
                div {
                    label {
                        "Lager status"
                    }
                    input {
                        type: "checkbox",
                        initial_value: product.read().quantity.is_some(),
                        oninput: move |evt| {
                            info!("checkbox click {}",evt.value());
                            let is_set: bool = evt.value() == "true";
                            if is_set {
                                product.with_mut( |product| {
                                    product.quantity = Some(1)
                                })
                            }
                            else { 
                                product.with_mut( |product| {
                                    product.quantity = None
                                })
                            }

                        }

                    },
                    input {
                        disabled: if product.read().quantity.is_none() {true},
                        value: product.read().quantity.map(|x| x.to_string() ).unwrap_or(String::from("")),
                        oninput: move |evt|{
                            product.write().quantity = Some(
                            match evt.value().parse::<u16>()
                            {
                                Ok(quantity) => quantity,
                                Err(e) => 0
                            });

                        },
                        type: "text"
                    }
                },
    }
}

#[component]
fn ProductPrice(product: Signal<Product> )-> Element
{
    rsx! {
        label {
            "Pris"
        }
        input {
            type: "text",
            value: "{product.read().price}kr",
            oninput: move |evt| {
                product.write().price = {
                    let value = evt.value();
                    let value = value.strip_suffix("kr").unwrap_or(&value);
                    match value.parse::<u16>()
                    {
                        Ok(price) => price,
                        Err(e) => 0
                    }
                };                    


            }
        },

    }
}

#[component]
fn ProductDescription(product: Signal<Product> )-> Element
{
    rsx! {
            label {
                "Beskrivning"
            }
            textarea {
                value: "{product.read().description}",
                oninput: move |evt| {
                    product.write().description = evt.value()
                }
            }
    }
}

#[derive(PartialEq, Clone, Props)]
struct FormFieldProps {
    product: Product
}



#[component]
fn FormFields(props: FormFieldProps) -> Element {


    let mut product =use_signal(move || props.product);

    rsx! {
            ProductName {product}
            div {
                ProductPrice {product},
                ProductInventory {product}
            },
            ProductDescription {product}
            div {
                class: "flex-start-container button-row",
                button {
                    disabled: product.read().id.is_none(),
                    class: "red",
                    "Ta bort!"
                }, 
                button {
                    disabled: product.read().id.is_none(),
                    onclick: move |_| {
                        product.set(Default::default())
                    },
                    "Ny produkt"
                }, 
                SaveButton {  product: product }

            }

    }
}


#[component]
pub fn ProductDetails( product : Signal<Option<Product>> ) -> Element {

    match &*product.read()
    {
        None => rsx ! {},
        Some(p) => rsx! {
            div {
                class: "overlay",
                onclick: move |_| {product.set(None); }
            },
            form {
                id: "product_editor",
                onsubmit: |evt| { evt.stop_propagation(); },
                div {
                    class: "flex-start-container",
                    id: "editor_title",
                    h2 {
                        "Redigera produkt"
                    },
                    components::CloseButton {
                        onclick: move |_| {product.set(None); },
                    },
                },
                FormFields { product: p.clone() }
            }
        }
    }

}