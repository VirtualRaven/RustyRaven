
use dioxus::logger::tracing::{info, warn};
use dioxus::prelude::*;

use crate::components::admin::product::product::ProductDetails;

use crate::server::{self, Product};

#[derive(PartialEq, Clone, Props)]
pub struct ProductRowProps {
    onclick: EventHandler<MouseEvent>,
    product: ReadOnlySignal<Product>
}

#[component]
pub fn ProductRow(props: ProductRowProps) -> Element {


    let product = &*props.product.read();
    let id = product.id.map( |x| x.to_string() ).unwrap_or("".into());
    let price = product.price.to_string();
    let quantity = product.quantity.map(|x| x.to_string()+"st" ).unwrap_or("Obegränsad".into());

    rsx! {
        tr {
                td { 
                    a {
                        onclick: move |evt| props.onclick.call(evt),
                        "{product.name}"
                    }

                },
                td { "{price}kr"},
                td { "{quantity}"}
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ProductTableProps {
    onedit: EventHandler<Product>,
    products: ReadOnlySignal<Vec<Product>>
}

#[component]
pub fn ProductTable(props: ProductTableProps) -> Element
{
    let products = props.products.clone();
    rsx! {
        table {
            tr {
                th {"Namn"},
                th {"Pris"},
                th {"Kvantitet"}
            }
            for product in products.iter() {

                ProductRow { 
                    onclick: { 
                        let product: Product = (*product).clone();
                        move |_| {  props.onedit.call(product.clone()); }
                    },
                    product: product.clone()
                }
            }
        }
    }

}

#[component]
pub fn ProductList(category: ReadOnlySignal<u32>) -> Element {

    let mut selected_product: Signal<Option<Product>> = use_signal(|| None);
    let products : Signal<Vec<Product>> = use_signal(|| vec![]);
    let update_counter : Signal<u32> = use_signal(|| 0);


    let products= use_resource(move || async move {
       info!("list update {}",update_counter.read());
       server::get_products(category.read().clone()).await 
    
    });



    let mut inspector = use_signal(|| false);

    rsx! {

        div {
            class: "product_list",
        match &*products.read_unchecked() {
            Some(Ok(products)) => rsx! {
                if products.len() > 0 
                {
                    ProductTable{
                        onedit: move |product| {
                            selected_product.set(Some(product));
                        },
                        products: products.clone()
                    }
                }
            },
            Some(Err(e)) =>  {
                warn!("Failed to get products {:#?}",e);
                rsx! {
                    h2 {
                        "Kunde inte ladda produkter!"
                    }
                }
            },
            None => rsx! {
                h2 { "Laddar..."} 
            }
        },


        button {
            onclick: move |_| {
                selected_product.set(Some(Product::new(category.read().clone())) );

            },
            "Lägg till produkt"
        }
        ProductDetails {product: selected_product , update_counter,category }

    }}
}