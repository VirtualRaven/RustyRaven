use dioxus::prelude::*;

#[component]
pub fn ProductImages(images: ReadOnlySignal<Vec<sjf_api::product::Image>>) -> Element {

    let mut selected_image = use_signal(|| images().first().unwrap().clone());

    rsx! {
        div {
            class: "product-images",
            if images().len() > 1 
            {
                div {
                    class: "preview",
                    div {
                        for image in images()
                        {
                            div {
                                class: "image_container",
                                onclick: { to_owned![image]; move |_| {
                                    *selected_image.write() = image.clone();
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
                    style: "background-color: #{selected_image().color}",
                    img {
                        srcset: "{selected_image().srcset().unwrap() }"
                    }

                }
            }
        }
    }

}
#[component]
pub fn Product(product: ReadOnlySignal<sjf_api::product::Product>) -> Element {
    
    rsx! 
    {
        document::Title { "{product().name}" }
        crate::components::CategoryBar { path: product().category_name }


        div {
            class: "product",



            div {
                class:"split-when-large",
                div {
                    ProductImages {images: product().images}
                }

                div {
                    class: "product-details",
                    h2 { "{product().name}"}
                    span { class: "price", "{product().price}kr"}
                    button { "Lägg i varukorg" }
                    p {
                        "{product().description}"
                    }
                }

            }


        }


    }

}