use dioxus::{html::i, logger::tracing::info, prelude::*};
use sjf_api::product::Preview;

use crate::server;

//#[component]
//fn Heighlights() -> Element
//{
//
//} 
//
//#[component]
//fn categories() -> Element
//{
//
//
//} 

#[component]
fn ProductPreview(preview: ReadOnlySignal<Preview>) -> Element 
{

    let mut loaded = use_signal(||false);
    if preview.read().images.is_empty() 
    {
        rsx! {
            span {
                "No image",
            }
        }

    }
    else 
    {
        let previewr = preview.read();
        let image = previewr.images.first().unwrap();
        let srcset = image.srcset();
    

        rsx!(
            div {
                class: "product_preview",
                div {
                    class: "img_container",
                    style: "background-color: #{image.color}",
                    img {
                        srcset:  srcset,
                        onload: move |_|  {loaded.set(true)}
                    }

                }
                div {
                    class: "footer",
                    span {
                        "{previewr.name}"
                    }
                    span {
                        "{previewr.price}kr"
                    }
                }
            }
        )

    } 
}

#[component]
fn Latest() -> Element
{
    
    let previews = use_resource(|| async  {
         server::get_previews(None,true).await
    });

    match &*previews.read_unchecked() 
    {
        Some(Ok(ref rsp)) => rsx! {
            h1 {
                "Senaste nytt!"
            }
            for preview in &rsp.previews
            {
                ProductPreview { key: "{preview.id}", preview: preview.clone()  }
            }

        },
        Some(Err(_)) => rsx! {

        },
        None=> rsx! {
            span {
                "Loading..."
            }
        }
    } 



} 


#[component]
pub fn FrontPage() -> Element 
{
    rsx! {
        div {
            class: "front_page",
            Latest {  }

        }
    }

}