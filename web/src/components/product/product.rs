use std::collections::BTreeMap;
use std::sync::Arc;

use dioxus::html::details::open;
use dioxus::html::FileEngine;
use dioxus::signals::Signal;
use dioxus::prelude::*;
use dioxus::logger::tracing::{info, warn};

use crate::{components, server};
use crate::server::{AuthenticatedRequest, Product};

use super::list::ProductList;

#[component]
pub fn SaveButton( product : Signal<Product>, update_counter: Signal<u32>) -> Element {

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
                            update_counter.with_mut(|c| {(*c)+=1;} );
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
        div {
            class: "inputsection",
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
}
#[component]
fn ProductInventory(product: Signal<Product> )-> Element
{
    rsx!{ 
                div {
                    class: "inputsection",
                    label {
                        "Lager status"
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
                    },
                    input {
                        type: "checkbox",
                        checked: product.read().quantity.is_some(),
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
                },
    }
}

#[component]
fn ProductPrice(product: Signal<Product> )-> Element
{
    rsx! {
        div {
            class: "inputsection",
            label {
                "Pris (ink moms)"
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

#[component]
fn ProductTax(product: Signal<Product>) -> Element {
    rsx! {
        div {
            class: "inputsection",
            label {
                for: "producttax",
                "Moms"
            }
            select {  
                onchange: move |e| {
                    product.write().tax_rate = e.value().parse().unwrap();
                },
                id: "producttax",
                option { value: 0,"0%"}
                option { value: 6,"6%"}
                option { value: 12,"12%"}
                option { value: 25,"25%"}
            }
        }
    }
}


#[component]
fn ImageUploadButton(product: Signal<Product>, thumbnails: Signal<BTreeMap<u32,u32>> )-> Element
{

    let mut is_loading  = use_signal(|| false);

    let read_files = move |file_engine: Arc<dyn FileEngine>| async move {
        let files = file_engine.files();
        let mut file_contents : Vec<Vec<u8>> = Vec::new();
        file_contents.reserve(files.len());

        for file_name in &files {
            if let Some(contents) = file_engine.read_file(file_name).await {
                info!("Read {} {}", file_name,contents.len());
                file_contents.push(contents);
            }
            else {
                info!("Failed to read {}",file_name);
            }
        }

        if !file_contents.is_empty()
        {
            is_loading.set(true);
            let resp = server::upload_images( 
                AuthenticatedRequest {
                    data: file_contents
                }
            ).await;
            is_loading.set(false);
            info!("Server responded with {:#?}",resp);
            
            match resp {
                Ok(mut images) => {
                    
                    let mut image_ids: Vec<_> = images.iter().map(|(a,_)| a.clone()).collect();
                    images.into_iter().for_each(
                        |(image,variant)| {
                            thumbnails.write().insert(image,variant);
                        }
                    );

                    let updated_images = match product.write().images.take() {
                        Some(mut existing_images) => {
                            existing_images.append(&mut image_ids);


                            existing_images
                        }
                        None=> image_ids
                    };

                    product.write().images=Some(updated_images);

                },
                Err(e) => {

                }
            }
        }


    };
    let upload_file = move |evt: FormEvent| async move {
        if let Some(file_engine) = evt.files() {
            read_files(file_engine).await;
        }
    };
    rsx! {
        div {
            class: "imageuploadcontainer",
            label {
                class: if *is_loading.read() { "loading"},
                for: "fileupload",
                div {}
                div {}
                span { class: "loader"}
            }
            input {
                disabled: if *is_loading.read() {"true"},
                id:"fileupload",
                type:"file",
                accept: "image/*",
                multiple: true,
                onchange: upload_file
            }

        }
    }
}

#[component]
fn ProductImage(image_id: u32, product: Signal<Product>, thumbnails: Signal<BTreeMap<u32,u32>>)-> Element {

    rsx! {
        div {
            onclick: { let image_id = image_id.clone(); move |_| {
                product.write().images.as_mut().unwrap().retain(|x| *x !=image_id );
                thumbnails.write().remove(&image_id);
            }},
            class: "imagecontainer",
            key: image_id,
            img {
                src: if let Some(thumbnail_id) = thumbnails.read().get(&image_id) { "/images/{image_id}/{thumbnail_id}" }
            }
            div {
                div {}
                div {}
            }
        }
    }
}


#[component]
fn ProductImages(product: Signal<Product> )-> Element
{
    let mut thumbnails: Signal<BTreeMap<u32, u32>> = use_signal(move || BTreeMap::new());

    let resource = use_resource(move || async move
    {
        if let Some(id)= product.read().id
        {
            if let Ok(v)= server::get_product_images( AuthenticatedRequest { data: (id as u32) } ).await
            {
                thumbnails.with_mut(|t|
                {
                    v.into_iter().filter(|(_,v)| !v.is_empty() ).for_each(|(k,mut v) |  {t.insert(k, v.remove(0) );} );
                });

            }
        }
    }
    );

    rsx!{
        div {
            class: "imagesection",
            if let Some(ref images) = product.read().images
            {
                for image in images {
                    ProductImage {image_id: *image, product , thumbnails }
                }
            },
            ImageUploadButton {product, thumbnails }
        }
    }
}


#[derive(PartialEq, Clone, Props)]
struct FormFieldProps {
    product: Product,
    update_counter: Signal<u32>
}



#[component]
fn FormFields(props: FormFieldProps) -> Element {


    let mut product =use_signal(move || props.product);

    rsx! {
            ProductName {product}
            div {
                ProductPrice {product},
                ProductTax {product  },
                ProductInventory {product}
            },
            ProductImages{product},
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
                SaveButton {  product: product, update_counter: props.update_counter }

            }

    }
}



#[component]
pub fn ProductDetails( product : Signal<Option<Product>>, update_counter: Signal<u32> ) -> Element {

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
                FormFields { product: p.clone(), update_counter }
            }
        }
    }

}