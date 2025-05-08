
use dioxus::logger::tracing::warn;
use dioxus::prelude::*;

use crate::components::product::product::ProductDetails;
const ADMIN_CSS: Asset = asset!("/assets/styling/admin.scss");

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
                td { "{id}"},
                td { "{product.name}"},
                td { "{price}kr"},
                td { "{quantity}"}
                th {
                    a {
                        onclick: move |evt| props.onclick.call(evt),
                        "Ändra"
                    }
                }

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
                th {"Artikel nr"},
                th {"Namn"},
                th {"Pris"},
                th {"Kvantitet"}
                th {""}
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
pub fn ProductList() -> Element {

    let mut selected_product: Signal<Option<Product>> = use_signal(|| None);
    let products : Signal<Vec<Product>> = use_signal(|| vec![]);


    let products= use_resource(move || async move {
       server::get_products().await 
    
    });



    let mut inspector = use_signal(|| false);

    rsx! {
        document::Link { rel: "stylesheet", href: ADMIN_CSS }

        div {
            class: "product_list",
        h2 {
            "Produkt katalog"
        }
        match &*products.read_unchecked() {
            Some(Ok(products)) => rsx! {
                ProductTable{
                    onedit: move |product| {
                        selected_product.set(Some(product));
                    },
                    products: products.clone()
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
                selected_product.set(Some(Default::default()) );

            },
            "Lägg till produkt"
        }
        ProductDetails {product: selected_product  }

    }}
}