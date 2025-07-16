use dioxus::{html::i, logger::tracing::info, prelude::*};
use sjf_api::product::{Image, Preview};

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
            a {
                href: previewr.product_url(),
                class: "product_preview",
                div {
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
            }
        )

    } 
}


#[component]
fn Latest() -> Element
{
    
    let previews: Resource<Result<(sjf_api::product::GetPreviewsResp, Preview,Image), ServerFnError>> = use_resource(|| async  {
         let res = server::get_previews(None,true,4).await?;

        use rand::distr::Distribution;
        let mut  rng = rand::rng();

        let random_index: usize =
        {
           let len = res.previews.len();
           let between = rand::distr::Uniform::try_from(0..len).unwrap();
           between.sample(&mut rng)
        };

        let highlight: Preview = res.previews[random_index].clone();

        let random_image_index: usize = {
            let len = highlight.images.len();
           let between = rand::distr::Uniform::try_from(0..len).unwrap();
           between.sample(&mut rng)
        };

        let highlighted_image = highlight.images[random_image_index].clone();

        Ok((res,highlight,highlighted_image))
    });


    match &*previews.read_unchecked() 
    {
        Some(Ok(( rsp,highlight,highlighted_image))) => rsx! {
            h1 {
                "Senaste nytt!"
            }
            div {
                class: "product-previews",

                for preview in &rsp.previews
                {
                    ProductPreview { key: "{preview.id}", preview: preview.clone()  }
                }
            }

            a {
                class: "product-highlight",
                href: highlight.product_url(),
                div {
                        img {
                            srcset: "{highlighted_image.srcset().unwrap() }"
                        }

                }
                div {
                    class: "info",
                    h1 {
                        "{highlight.name}"
                    }
                    h2 {
                        "{highlight.price}kr"
                    }
                }
                

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
fn ProductCategories() -> Element 
{
    
    let mut context = use_context::< crate::components::CategorySignal>();

    rsx! {
        if let Some(ref cs) = context() 
        {
            div {
                class: "category-showcase",
                h2 { "Utforska"}
                div {
                    for (_,c) in &cs.children {
                        div {
                            class: "category",
                            "{c}"
                        }
                    }
                }

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
            ProductCategories {}

        }
    }

}