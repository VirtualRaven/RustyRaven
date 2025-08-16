use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread::current;

use dioxus::html::details::open;
use dioxus::html::FileEngine;
use dioxus::logger::tracing::{info, warn};
use dioxus::prelude::*;
use dioxus::signals::Signal;

use crate::components::admin::category;
use crate::server::{AuthenticatedRequest, Product};
use crate::{components, server};

#[derive(PartialEq, Clone, Debug)]
struct Category {
    id: u32,
    depth: u32,
    name: String,
    parent: Option<u32>,
}

#[component]
fn CategoryRemovalButton(
    category: ReadOnlySignal<Category>,
    delete_fn: EventHandler<()>,
) -> Element {
    enum State {
        Idle,
        Removing,
        Error,
        Removed,
    };

    let mut state = use_signal(|| State::Idle);

    let current_state = &*state.read();
    match current_state {
        State::Idle => rsx! {
            span {
               onclick: move |e| async move {
                   e.stop_propagation();

                   state.set(State::Removing);
                   let res = server::category::delete( AuthenticatedRequest { data: sjf_api::category::DeleteReq { id: category().id } }  ).await;
                   if let  Ok(_) = res
                   {
                       state.set(State::Removed);
                       delete_fn.call(());
                   }
                   else{
                    state.set(State::Error);
                   }

               },
               "Ta bort!"
            }
        },

        State::Removing => rsx! {
            span {
                "Tar bort..."
            }
        },

        State::Error => rsx! {
            span {
                "Misslyckades"
            }
        },
        State::Removed => rsx! {
            span {
                "Bort plockad"
            }
        },
    }
}

#[component]
fn CategoryEntry(category: ReadOnlySignal<Category>, delete_fn: EventHandler<()>) -> Element {
    let mut categories: Signal<Vec<Category>> = use_signal(|| Vec::new());
    let mut name: Signal<String> = use_signal(move || category.read().name.clone());
    let mut editable: Signal<bool> = use_signal(|| false);
    let mut menu_open: Signal<bool> = use_signal(|| false);
    let mut expanded: Signal<bool> = use_signal(|| false);

    let _ = use_resource(move || async move {
        match server::category::get_children(Some(category.read().id.clone())).await {
            Ok(rsp) => {
                let parent_depth = category.read().depth;
                let parent_id = category.read().id;
                categories
                    .write()
                    .extend(rsp.children.into_iter().map(|(id, name)| Category {
                        id,
                        name,
                        depth: parent_depth + 1,
                        parent: Some(parent_id.clone()),
                    }));
            }
            Err(_) => warn!("Failed to load children!"),
        }
    });

    rsx! {
        div {
            key: category.read().id,
            id: "{category.read().id}",
            class: "categoryItem",
            div {
                class: "categoryRow",
                span {
                    onclick: move |_| { expanded.with_mut(| o| {*o = !*o;} ); },
                    "V"
                }
                if *editable.read()
                {
                    input {type:"text",
                        onfocusout: move |_| {editable.set(false); },
                        onmounted: move |element| async move { let _ =  element.set_focus(true).await; },
                        onchange: move |evt|  async move {
                                name.set(evt.value());
                                server::category::update_name(AuthenticatedRequest { data: (category.read().id, name.read().clone() ) }).await;

                        },
                        value: "{name.read()}"
                    }
                }
                else {
                    h3 {
                        onclick: move |_| {editable.set(true)},
                        "{name.read()}"}
                }
                div {
                    onclick: move |_| { menu_open.with_mut(| o| {*o = !*o;} ); },
                    class: "categoryMenu",
                    div {
                        class: if *menu_open.read() {"show"},
                        span {
                            onclick: move |_| async move {

                                let parent = Some(category.read().id);
                                let name = String::from("Ny kategori");
                                let rsp = server::category::create(
                                    AuthenticatedRequest {
                                        data: server::category::CreateReq {
                                            name: name.clone(),
                                            parent: parent.clone()
                                        }
                                    }
                                ).await;

                                match rsp
                                {
                                    Ok(rsp) => {
                                        categories.write().push(Category { id: rsp.id, depth: rsp.depth, name, parent });
                                        expanded.set(true);
                                    },
                                    Err(e) => warn!("{:#?}", e)
                                };
                            },
                            "Skapa ny underkategori"
                        }
                        span {
                            onclick: move |_| {editable.set(true); },
                            "Redigera"
                        }

                        CategoryRemovalButton { category,delete_fn  }

                    }

                }
            }

            if *expanded.read()
            {
                for category in categories.read().iter()
                {
                    CategoryEntry { key: "{category.id}", category: category.clone(),
                        delete_fn: {
                            let current_id = category.id;
                            move || {
                                categories.write().retain(|c| {c.id != current_id} );
                            }
                        }
                    }
                }
                components::ProductList { category: category.read().id  }
            }

        }
    }
}

pub const ADMIN_CSS: Asset = asset!("/assets/styling/admin.scss");

#[component]
pub fn CategoryList() -> Element {
    let mut categories: Signal<Vec<Category>> = use_signal(|| Vec::new());

    let loaded_categories = use_resource(move || async move {
        match crate::server::auth::is_authenticated().await {
            Ok(true) => match server::category::get_children(None).await {
                Ok(rsp) => {
                    categories
                        .write()
                        .extend(rsp.children.into_iter().map(|(id, name)| Category {
                            id,
                            name,
                            depth: 0,
                            parent: None,
                        }));
                    Ok(())
                }
                Err(_) => {
                    warn!("Failed to load children!");
                    Err(())
                }
            },
            Ok(false) | Err(_) => {
                let nav = navigator();
                nav.push(NavigationTarget::<crate::Route>::Internal(
                    crate::Route::Auth {},
                ));
                Err(())
            }
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: ADMIN_CSS }

        div {
            class: "product_list",
        h2 {
            "Produkt katalog"
        }
        match &*loaded_categories.read_unchecked() {
            Some(Ok(())) => rsx! {
                for category in categories.read().iter()
                {
                    CategoryEntry {
                        key: "{category.id}",
                        category: category.clone(),
                        delete_fn: {
                            let current_id = category.id;
                            move || {
                                categories.write().retain(|c| {  c.id != current_id} );
                            }
                        }
                    }
                }
            },
            Some(Err(())) =>  {
                rsx! {
                    h2 {
                        "Kunde inte ladda kategorier!"
                    }
                }
            },
            None => rsx! {
                h2 { "Laddar..."}
            }
        },


        button {
            onclick: move |_| async move {
                let name = String::from("Ny kategori") ;
                let rsp = server::category::create(
                    AuthenticatedRequest {
                        data: server::category::CreateReq {
                            name: name.clone(),
                            parent: None
                        }
                    }
                ).await;

                match rsp
                {
                    Ok(rsp) => categories.write().push(Category { id: rsp.id, depth: rsp.depth, parent: None,name }),
                    Err(e) => warn!("{:#?}", e)
                };
            },
            "Lägg till kategori"
        }
    }}
}
