use components::FrontPage;
use dioxus::prelude::*;

//use components::{Echo, Hero};

mod components;
mod server;
use dioxus::logger::tracing::{error, info};
#[cfg(feature = "server")]
use sjf_db as db;
#[cfg(feature = "server")]
use sjf_image as image;

cfg_if::cfg_if! {
    if  #[cfg(feature="server")]
    {
        use axum::extract::Path;
        use axum::response::IntoResponse;
        use axum::extract::Request;
        use axum::http::{StatusCode, Uri};
        use axum::middleware::Next;
        use axum::response::Response;
    }
}
//const FAVICON: Asset = asset!("/assets/favicon.ico");

use crate::components::{
    About, Auth, CartState, CategoryList, OrderCanceled, OrderCompleted, TermsAndConditions,
};

#[derive(Routable, PartialEq, Clone)]
pub enum Route {
    #[layout(HeaderFooter)]
    #[route("/")]
    FrontPage {},
    #[route("/admin/products")]
    CategoryList {},
    #[route("/admin/auth")]
    Auth {},
    #[route("/produkter/:..segments")]
    ProductPage { segments: Vec<String> },
    #[nest("/order")]
    #[route("/avbruten/:uuid")]
    OrderCanceled { uuid: String },
    #[route("/klar/:uuid")]
    OrderCompleted { uuid: String },
    #[end_nest]
    #[route("/om")]
    About {},
    #[route("/användarvillkor")]
    TermsAndConditions {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

 cfg_if::cfg_if! 
 {
     if  #[cfg(feature="server")]
     {
        fn is_demo() -> bool { sjf_payment::is_payment_demo() }
     }
     else 
     {
        fn is_demo() -> bool { false }

     }
 }
#[component]
fn HeaderFooter() -> Element {
    use_context_provider(|| Signal::new(None::<sjf_api::category::GetChildrenRsp>));
    use_context_provider(|| Signal::new(components::CartState::new()));

    let mut cart_state = components::use_cart();

    let _ = use_resource(move || async move {
        if let Some(c) = CartState::load().await {
            cart_state.set(c);
        }
    });

    let has_items = use_memo(move || !cart_state.read().is_empty());


    let is_demo_mode = use_server_cached(||  is_demo() );

    let banner_class = {
        if has_items() {
            "banner visible"
        } else {
            "banner"
        }
    };

    rsx! {
        document::Link{
            rel: "preconnect",
            href:"https://fonts.googleapis.com"
        }
        document::Link{
            rel: "preconnect",
            href:"https://fonts.gstatic.com",
            crossorigin: "true"
        }
        document:: Stylesheet {
            href: r"https://fonts.googleapis.com/css2?family=Orbitron:wght@400..900&family=Oxanium:wght@200..800&display=swap"
        }

        components::Header {}
        if is_demo_mode 
        {
            div {
                class: "banner visible",
                "SIDAN ÄR I DEMOLÄGE!"
            }
        }
        div {
            class: "{banner_class}",
            "Fri frakt över 999kr!"
        }
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

use crate::server::{get_category_and_product, get_product};

#[cfg(feature = "server")]
pub async fn handle_image_get(Path(id): Path<(u32, u32)>) -> impl IntoResponse {
    use axum::http::StatusCode;

    let image_id: image::ImageId = id.into();

    use axum::http::header;
    match image::get(image_id).await {
        Some(image) => {
            let headers = [
                (header::CONTENT_TYPE, "image/jpeg"),
                (
                    header::CACHE_CONTROL,
                    "max-age: public, max-age=604800, immutable",
                ),
            ];

            Ok((headers, (*image).clone()))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
#[cfg(feature = "server")]
pub async fn handle_robots_get() -> impl IntoResponse {
    use axum::http::header;
    let headers = [
        (header::CONTENT_TYPE, "text/plain"),
    ];

    let robots_txt = match sjf_payment::is_payment_demo()
    {
        true => include_str!("server/robots/robots_demo.txt"),
        false=> include_str!("server/robots/robots.txt")
    };

    (headers, robots_txt)

}

#[cfg(feature = "server")]
async fn order_middleware(request: Request, next: Next) -> Response {
    let path = request.uri().path();

    if let Some(uuid) = path.strip_prefix(&(sjf_payment::SUCCESS_PATH.to_owned() + "/")) {
        let undo = db::checkout::commit_reservation(uuid.into()).await;
        if let Err(e) = undo {
            error!("Failed to commit order {} from link {}", uuid, e);
        }
    } else if let Some(uuid) = path.strip_prefix(&(sjf_payment::CANCLE_PATH.to_owned() + "/")) {
        let undo = db::checkout::undo_reservation(uuid.into()).await;
        if let Err(e) = undo {
            error!("Failed to undo order {} from link {}", uuid, e);
        }
    }

    let response = next.run(request).await;
    response
}
#[cfg(feature = "server")]
async fn accept_fix(mut request: Request, next: Next) -> Response {
    if let Some(accept) = request.headers_mut().get_mut("Accept")
    {
        if let Ok(str) = accept.to_str()
        {
            if str.contains("*/*")
            {
                use axum::http::HeaderValue;

                let fixed = format!("{}, text/html",str);
                if let Ok(patched) = HeaderValue::from_str(&fixed)
                {
                    *accept = patched;
                }
            }
        }
    }
    let response = next.run(request).await;
    response
}

#[cfg(feature = "server")]
mod metrics {
    use std::{net::SocketAddrV4, time::Duration};

    use axum_prometheus::{
        metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle},
        utils, Handle, PrometheusMetricLayer,
    };

    use super::*;

    pub fn make_prometheus() -> (PrometheusMetricLayer<'static>, PrometheusHandle) {
        let tmp_listen_addr: SocketAddrV4 = "0.0.0.0:0"
            .parse()
            .context("invalid temporary prometheus listen addr")
            .unwrap();
        let builder = PrometheusBuilder::new()
            .with_http_listener(tmp_listen_addr)
            .upkeep_timeout(Duration::from_secs(5));

        let builder = builder
            .set_buckets_for_metric(
                Matcher::Full(
                    axum_prometheus::PREFIXED_HTTP_REQUESTS_DURATION_SECONDS
                        .get()
                        .map_or(axum_prometheus::AXUM_HTTP_REQUESTS_DURATION_SECONDS, |s| {
                            s.as_str()
                        })
                        .to_string(),
                ),
                utils::SECONDS_DURATION_BUCKETS,
            )
            .context("set buckets for prometheus metrics")
            .unwrap();

        let (recorder, _) = builder.build().context("build prometheus").unwrap();

        let handle = recorder.handle();
        let handle = Handle(handle);

        axum_prometheus::metrics::set_global_recorder(recorder)
            .expect("Failed to set global recorder");

        PrometheusMetricLayer::pair_from(handle)
    }
}

#[cfg(feature = "server")]
async fn launch_server() {
    let res = dotenvy::dotenv();

    dioxus::logger::init(dioxus::logger::tracing::Level::INFO).expect("failed to init logger");

    use crate::metrics::make_prometheus;
    use axum_prometheus::PrometheusMetricLayerBuilder;
    let (prometheus_layer, metric_handle) = make_prometheus();

    if let Ok(dot_env) = res {
        info!("Loaded {}", dot_env.to_string_lossy());
    }

    info!("Initializing db...");
    if !db::init().await {
        std::process::exit(1);
    }

    info!("Initializing stripe");
    if let Err(e) = sjf_payment::init().await {
        error!("{}", e);
        std::process::exit(2);
    }

    info!("Initializing object storage...");
    let res = image::init().await;
    if let Err(e) = res {
        use std::error::Error;

        error!("Object storage intialization failed: \n{:#?}", e);
        let mut source = e.source();
        while let Some(s) = source {
            info!("Source {:#?}", s);
            source = s.source();
        }

        error!("{:#?}", e.source());
        std::process::exit(2);
    }

    info!("Initializing dioxus...");
    // Connect to the IP and PORT env vars passed by the Dioxus CLI (or your dockerfile)
    let socket_addr = dioxus::cli_config::fullstack_address_or_localhost();
    let metric_socket_addr = {
        let mut tmp = socket_addr.clone();
        tmp.set_port(5533);
        tmp
    };

    info!("Hosting at {}", socket_addr);

    use axum::routing::get;
    use dioxus::fullstack::prelude::DioxusRouterExt;

    let dioxus_router = axum::Router::new()
        .route("/images/:image_id}/:variant_id", get(handle_image_get))
        .route("/robots.txt", get(handle_robots_get) )
        .serve_dioxus_application(ServeConfigBuilder::new(), App)
        .layer(prometheus_layer)
        .layer(axum::middleware::from_fn(accept_fix))
        .layer(axum::middleware::from_fn(order_middleware))
        .layer(axum::middleware::from_fn(
            sjf_auth::axum::protect_authenticated_routes,
        ))
        .layer(axum::Extension(sjf_auth::state::AuthState::new()))
        .layer(sjf_auth::axum::create_auth_layer());

    let custom_router = axum::Router::new().route(
        "/kubernetes/probes/liveness",
        get(|| async { StatusCode::NO_CONTENT }),
    );

    let router = axum::Router::new()
        .merge(custom_router)
        .merge(dioxus_router)
        .into_make_service();

    let metrics_router = axum::Router::new()
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .into_make_service();

    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
    let metrics_listener = tokio::net::TcpListener::bind(metric_socket_addr)
        .await
        .unwrap();

    let f1 = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let f2 = tokio::spawn(async move {
        axum::serve(metrics_listener, metrics_router).await.unwrap();
    });
    let _ = tokio::join!(f1, f2);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    server_only! {
        let path : String = {
            let ctx = server_context();
            let req = ctx.request_parts();
            req.uri.path().to_string()
        };
        let labels= [("endpoint",path)];
        axum_prometheus::metrics::counter!("not_found_pages", &labels).increment(1);
    }
    rsx! {
        h2 {
            "Oops! Sidan hittades inte :/"
        }
    }
}

#[component]
fn ProductPage(segments: ReadOnlySignal<Vec<String>>) -> Element {
    let error_msg = "Ooops här var det tomt, möjligen kan produkten plockats bort";
    let product = use_server_future(
        move || {
            let segments = segments.clone();
            async move { get_category_and_product(segments().join("/")).await }
        }
    )?;

    match *product.read_unchecked() {
        None => rsx! {
           div { "Laddar produkt..." }
        },
        Some(Ok((i, Some(ref p)))) => rsx! {
            components::Product {product: p.clone() }
        },
        Some(Ok((i, None))) => rsx! {
            components::Category { category_path: segments, id: i  }
        },
        Some(Err(ref e)) => rsx! {
            div { "{error_msg}" }
        },
    }
}
