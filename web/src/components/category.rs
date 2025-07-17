use dioxus::prelude::*;

use crate::server::{category::get_children, get_previews};

#[component]
pub fn Category(category_path: ReadOnlySignal<Vec<String>>, id: ReadOnlySignal<u32>) -> Element {

    let children_and_previews: Resource<Result<(sjf_api::category::GetChildrenRsp, sjf_api::product::GetPreviewsResp), ServerFnError>> = use_resource(move || async move {

        let id = Some(*id.read());
        let cs = get_children(id.clone());
        let ps = get_previews(id, false, 100);

        Ok((cs.await?, ps.await?))
    });

    
    rsx! 
    {
        document::Title { "{category_path.last().unwrap() }" }
        crate::components::CategoryBar { path: category_path }


        div {
            div {
                match &*children_and_previews.read()
                {
                    None => rsx! {div { }},
                    Some(Ok((c,p))) => rsx! {


                        match ( (c.children.is_empty(), p.previews.is_empty() ) )
                        {
                            (true,true) => rsx! {
                                    div {
                                        "Ooops här var det tomt för tillfället, men kika gärna tillbaka senare."
                                         Link {to: crate::Route::FrontPage {}, "Tillbaka till början" }
                                    }
                            },
                            (children_empty,previews_empty) => rsx! {

                                if !children_empty
                                {
                                    div {
                                        class: "category-showcase",
                                        div {
                                            for child in &c.children 
                                            {
                                                a {
                                                    href:"{child.1.clone()}/",
                                                    div {
                                                        {child.1.clone()}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                if !previews_empty
                                {
                                    div {
                                        class: "product-previews",
                                        for preview in &p.previews 
                                        {
                                            crate::components::ProductPreview {preview: preview.clone()}
                                        }
                                    }

                                }

                            },
                        }

                    },
                    Some(Err(_)) => rsx! {
                        span { "Kunde inte ladda underkategorier"}
                    }
                }
            }



        }


    }

}