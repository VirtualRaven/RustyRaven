use dioxus::prelude::*;

#[component]
pub fn ProductImages(images: ReadOnlySignal<Vec<sjf_api::product::Image>>) -> Element {
    let mut selected_image = use_signal(|| (0usize, images().first().unwrap().clone()));

    rsx! {
        div {
            class: "product-images",
            if images().len() > 1
            {
                div {
                    class: "preview",
                    div {
                        for (index,image) in images().iter().enumerate()
                        {
                            div {
                                class: "image_container",
                                onclick: { to_owned![image]; move |_| {
                                    *selected_image.write() = (index,image.clone());
                                }},
                                key: "{image.sizes.first().unwrap().url}",
                                style: "background-color: #{image.color}",
                                img {
                                    srcset: "{image.srcset().unwrap() }"
                                }
                            }
                        }
                    }
                }
            }
            div {
                class: "main",
                div {
                    class: "image_container",
                    style: "background-color: #{selected_image().1.color}",
                    div {
                        class: "overlay",
                        div {
                            onclick: move |_| {
                                let (index, _) = *selected_image.read();
                                let new_index = {
                                    match index {
                                        0 => images.read().len()-1,
                                        i => i-1
                                    }
                                };

                                *selected_image.write() = {
                                    let new_image = (*images.read()).get(new_index).unwrap().clone();
                                    (new_index,new_image)
                                };
                            },
                            div {}
                        }
                        div {}
                        div {
                            onclick: move |_| {
                                let (index, _) = *selected_image.read();
                                let new_index = {
                                    let max_len = images.read().len()-1;
                                    match index {
                                        i if i == max_len => 0,
                                        i => i+1
                                    }
                                };

                                *selected_image.write() = {
                                    let new_image = (*images.read()).get(new_index).unwrap().clone();
                                    (new_index,new_image)
                                };
                            } ,
                            div {}
                        },
                    }
                    img {
                        srcset: "{selected_image().1.srcset().unwrap() }"
                    }

                }
            }
        }
    }
}
#[component]
pub fn AddToCartButton(product: ReadOnlySignal<sjf_api::product::Product>) -> Element {
    let mut cart = crate::components::cart::use_cart();

    let added = use_memo(move || {
        let cart = cart.read();
        let product = product.read();
        let id = product.id;
        cart.has_item(&id)
    });

    let stock = product.read().stock;

    rsx! {

        if let Some(0) = stock
        {
            div {
                class: "outofstock",
                "Slutsåld!"
            }
        }
        else {
            button {
                onclick: move |_| {
                    cart.with_mut(move |cart|
                        cart.add_item(product.read().clone())
                    );
                },
                if added() {
                    "Tillagd!"
                }
                else
                {
                    "Lägg i varukorg"
                }
            }
            if let Some(s) = stock
            {
                if s < 10
                {
                    span {
                        class: "lowstock",
                        "Endast {s} kvar i lager"
                    }
                }
            }

        }

    }
}

#[component]
pub fn Product(product: ReadOnlySignal<sjf_api::product::Product>) -> Element {
    rsx! {
        document::Title { "SJF Concept - {product().name}" }
        crate::components::CategoryBar { path: product().category_name }


        div {
            class: "product",
            div {
                class:"split-when-large",
                div {
                    ProductImages {images: product().images  }
                }

                div {
                    class: "product-details",
                    h2 { "{product().name}"}
                    span { class: "price", "{product().price}kr"}
                    AddToCartButton { product  }
                    p {
                        "{product().description}"
                    }
                }

            }


        }


    }
}
