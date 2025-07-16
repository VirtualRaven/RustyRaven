use components::FrontPage;
use dioxus::{prelude::*};

//use components::{Echo, Hero};

mod components;
mod server;
#[cfg(feature="server")]
use sjf_image as image;
#[cfg(feature="server")]
use sjf_db as db;
use dioxus::logger::tracing::info;

//const FAVICON: Asset = asset!("/assets/favicon.ico");


use crate::components::CategoryList;
#[derive(Routable, PartialEq, Clone)]
pub enum Route {
    #[layout(HeaderFooter)]
    #[route("/")]
    FrontPage {},
    #[route("/admin/products")]
    CategoryList {},
    #[route("/produkter/:..segments")]
    ProductPage { segments: Vec<String> },
    #[route("/:..segments")]
    NotFound { segments: Vec<String> }
}

#[component]
fn HeaderFooter() -> Element {
    use_context_provider(|| Signal::new(None::<sjf_api::category::GetChildrenRsp>) );
    rsx! {
        components::Header {}
        div {
            class: "content",
            Outlet::<Route> {}
        }
        components::Footer {}
    }
}

cfg_if::cfg_if! {
    if  #[cfg(feature="server")]
    {
        fn main() {
            tokio::runtime::Runtime::new().unwrap().block_on(launch_server());
        }

    }
    else 
    {
        fn main() 
        {
            use dioxus::logger::tracing::Level;
            dioxus::logger::init(Level::INFO).expect("failed to init logger");
            dioxus::launch(App);
        }

    }
}

#[cfg(feature="server")]
use axum::extract::Path;
#[cfg(feature="server")]
use axum::response::IntoResponse;

use crate::server::get_product;

#[cfg(feature="server")]
pub async fn handle_image_get(Path(id): Path<(u32, u32)>) -> impl IntoResponse
{
    use axum::http::StatusCode;


    let image_id: image::ImageId =  id.into();

    use axum::http::header;
    match image::get(image_id).await
    {
        Some(image) => {

            let headers = [  (header::CONTENT_TYPE,"image/jpeg"), (header::CACHE_CONTROL, "max-age: public, max-age=604800, immutable")];

            Ok((headers, (*image).clone()))


        }
        None => {
            Err(StatusCode::NOT_FOUND)
        }
    }
}




#[cfg(feature="server")]
async fn launch_server() {
    // Connect to dioxus' logging infrastructure
    dioxus::logger::initialize_default();
    
    info!("Initializing db...");
    db::init().await;
    
    info!("Initializing object storage...");
    image::init().await.unwrap();

    info!("Initializing dioxus...");
    // Connect to the IP and PORT env vars passed by the Dioxus CLI (or your dockerfile)
    let socket_addr =  dioxus::cli_config::fullstack_address_or_localhost();
    use axum::routing::get;
    use dioxus::fullstack::prelude::DioxusRouterExt;

    let dioxus_router = axum::Router::new()
        .serve_dioxus_application(ServeConfigBuilder::new(), App);
    let custom_router= axum::Router::new()
            .route("/images/:image_id}/:variant_id", get(handle_image_get));

    let router = axum::Router::new()
        .merge(custom_router)
        .merge(dioxus_router)
        .into_make_service();
    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}


#[component]
fn NotFound(segments : Vec<String>) -> Element {
    rsx! {
        h2 {
            "Oops! Sidan hittades inte :/"
        }
    }
}

#[component]
fn ProductPage(segments: Vec<String> ) -> Element {

    let error_msg = "Ooops här var det tomt, möjligen kan produkten plockats bort";
    let id : Option<String> = segments.last().cloned();
    let product = use_resource(  { let id = id.clone(); move || {to_owned![id];  async move {
        
        if  id.is_none() {
            return Err(())
        }

        match  id.unwrap().parse()
        {
            Ok(i) => get_product(i).await.map_err(|_| () ),
            Err(_) => Err(())
        }
    
    }}});
    
    if id.is_none()
    {
        rsx! {
            div { "{error_msg}" }
        }
    }
    else {

        match *product.read() {
            None => rsx! {
               div { "Laddar..." }
            },
            Some(Ok(ref p)) => rsx! {
                components::Product {product: p.clone() }
            },
            Some(Err(e)) => rsx! 
            {
                div { "{error_msg}" }
            }
        }
    }

}

