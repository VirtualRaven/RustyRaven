use dioxus::prelude::*;

//use components::{Echo, Hero};

mod components;
#[cfg(feature =  "server")]
mod server;

//const FAVICON: Asset = asset!("/assets/favicon.ico");


#[derive(Routable, PartialEq, Clone)]
pub enum Route {
    // if the current location is "/home", render the Home component
    #[route("/")]
    Main {},
}

cfg_if::cfg_if! {
    if  #[cfg(feature="server")]
    {
        #[tokio::main]
        async fn main() {
            pretty_env_logger::init();
            log::info!("Initialized");
            db::init().await;
            dioxus::launch(App);
        }

    }
    else 
    {
        fn main() 
        {
            dioxus::launch(App);
        }

    }
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
