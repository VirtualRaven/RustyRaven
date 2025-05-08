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


#[derive(Routable, PartialEq, Clone)]
pub enum Route {
    // if the current location is "/home", render the Home component
    #[route("/")]
    Main {},
    #[route("/admin/products")]
    AdminProduct {},
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

#[cfg(feature="server")]
pub async fn handle_image_get(Path(id): Path<(u32, u32)>) -> impl IntoResponse
{
    use axum::http::StatusCode;


    let image_id: image::ImageId =  id.into();

    use axum::http::header;
    match image::get(image_id).await
    {
        Some(image) => {

            let headers = [  (header::CONTENT_TYPE,"image/jpeg")];

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

    info!("Initializing dioxus...");
    // Connect to the IP and PORT env vars passed by the Dioxus CLI (or your dockerfile)
    let socket_addr =  dioxus::cli_config::fullstack_address_or_localhost();
    use axum::routing::get;
    use dioxus::fullstack::prelude::DioxusRouterExt;

    let dioxus_router = axum::Router::new()
        .serve_dioxus_application(ServeConfigBuilder::new(), App);
    let custom_router= axum::Router::new()
            .route("/images/:image_id}/:variant_id", get(handle_image_get))
            .route("/test1/", get(async || format!("hello world")))
            .route("/test/*key", get(async | Path(id) : Path<(String)> | format!("hello world {id}")));


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
fn Main() -> Element {
    rsx! {
        components::Header {}
        div {
            class: "content"
        }
        components::Footer {}
    }
}
#[component]
fn AdminProduct() -> Element {
    rsx! {
        components::Header {}
        div {
            class: "content",
            components::ProductList {}
        }
        components::Footer {}
    }
}
