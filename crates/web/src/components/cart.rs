use std::collections::BTreeMap;

use crate::components::MenuState;
use dioxus::prelude::{server_fn::ServerFn, *};
use serde::Serialize;
use sjf_api::{
    checkout::CheckoutRequest,
    product::{Product, ProductId},
};
use dioxus::logger::tracing::warn;

const CART_ICON: Asset = asset!("/assets/cart.png");

pub use u32 as ProductQuantity;

fn cart_name() -> String {
    format!(
        "cart-{}-{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION_MAJOR")
    )
}

#[derive(PartialEq)]
pub struct CartState {
    open: MenuState,
    contents: BTreeMap<ProductId, (Product, ProductQuantity)>,
}

impl CartState {
    pub fn new() -> Self {
        Self {
            open: MenuState::Closed,
            contents: Default::default(),
        }
    }

    #[cfg(feature = "web")]
    fn get_storage() -> Option<web_sys::Storage> {
        use web_sys::window;
        window().map(|w| w.local_storage().unwrap_or_default())?
    }

    #[cfg(feature = "web")]
    pub fn save(&self) {
        use dioxus::logger::tracing::info;
        info!("Saving cart");
        let state: BTreeMap<u32, u32> = self
            .contents
            .iter()
            .map(|(k, (_, q))| (k.clone(), q.clone()))
            .collect();

        Self::get_storage().map(|s| {
            let _ = s.set_item(&cart_name(), &serde_json::to_string(&state)?);
            Ok::<(), serde_json::Error>(())
        });
    }

    #[cfg(feature = "web")]
    pub async fn load() -> Option<Self> {
        use dioxus::logger::tracing::info;
        use web_sys::window;

        if window().map(|w| w.location().pathname().unwrap_or_default() ).unwrap_or_default().starts_with(sjf_api::payment::SUCCESS_PATH)
        {
            let r = Self::get_storage().map( |s| s.remove_item(&cart_name()));
            match r {
                Some(Ok(())) => (),
                Some(Err(_)) => warn!("Failed to clear cart"),
                None => warn!("Failed to get storage to clear cart")
            }
        }

        let unserialize = || {
            if let Some(storage) = Self::get_storage() {
                let data = storage.get_item(&cart_name());
                if let Ok(Some(data)) = data {
                    if let Ok(data) = serde_json::from_str(&data) {
                        let data: BTreeMap<u32, u32> = data;
                        return Some(data);
                    }
                }
            }
            return None;
        };

        if let Some(data) = unserialize() {
            if !data.is_empty() {
                let product_ids = data.keys().cloned().collect();
                let rsp = crate::server::get_specified_products(product_ids).await;
                if let Ok(ps) = rsp {
                    return Some(Self {
                        contents: ps
                            .into_iter()
                            .filter_map(|p| {
                                use std::u32;

                                let previous_quantity = data.get(&p.id).unwrap().clone();
                                let max_quantity = p.stock.clone().unwrap_or(u32::MAX);
                                let new_quantity = {
                                    if previous_quantity > max_quantity {
                                        max_quantity
                                    } else {
                                        previous_quantity
                                    }
                                };

                                if new_quantity > 0 {
                                    Some((p.id.clone(), (p, new_quantity)))
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        open: MenuState::Closed,
                    });
                }
            }
        }
        return None;
    }

    pub fn has_item(&self, id: &ProductId) -> bool {
        self.contents.get(id).is_some()
    }
    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
    pub fn close(&mut self) {
        self.open = MenuState::Closed;
    }

    pub async fn checkout(&self) -> Result<String, ServerFnError> {
        let req = CheckoutRequest {
            order: self
                .contents
                .clone()
                .into_iter()
                .map(|(id, (_, quantity))| (id, quantity))
                .collect(),
        };
        let res = crate::server::checkout(req).await;
        if let Ok(payment_url) = &res {
            let nav = navigator();
            nav.push(NavigationTarget::<crate::Route>::External(
                payment_url.clone(),
            ));
        }
        res
    }

    pub fn add_item(&mut self, product: Product) {
        let key = product.id;
        match self.contents.get_mut(&key) {
            Some(_) => {
                self.inc(&product.id);
            }
            None => {
                self.contents.insert(key, (product, 1));
            }
        }
        self.open = MenuState::Opened;
        self.save();
    }

    pub fn clear(&mut self) {
        self.contents.clear();
        self.save();
    }

    fn inc(&mut self, id: &ProductId) {
        match self.contents.get_mut(id) {
            Some((p, q)) => {
                *q = *q + 1;
                match p.stock {
                    Some(stock) if *q >= stock => *q = stock,
                    _ => {}
                }
            }
            None => {}
        }
        self.save();
    }
    fn dec(&mut self, id: &ProductId) {
        match self.contents.get_mut(id) {
            Some((_, q)) if *q <= 1 => {
                self.contents.remove(id);
            }
            Some((_, q)) => {
                *q = *q - 1;
            }
            None => {}
        }

        if self.contents.is_empty() {
            self.open = MenuState::Closed;
        }
        self.save();
    }

    fn num_items(&self) -> u32 {
        self.contents
            .values()
            .map(|(_, q)| q)
            .fold(0, |acc, q| acc + q)
    }

    fn toggle(&mut self) {
        self.open = self.open.toggle();
    }
}

type CartSignal = Signal<CartState>;
pub fn use_cart() -> CartSignal {
    use_context::<CartSignal>()
}

#[component]
fn CartItem(item_id: ReadOnlySignal<ProductId>) -> Element {
    let mut cart_state = use_cart();

    let cart = cart_state.read();
    let (item, quantity) = cart.contents.get(&*item_id.read()).unwrap();
    rsx! {
        div {
            class: "item",
            if let Some(image) = item.images.first()
            {
                div {
                    class: "image",
                    style:  "background-color: {image.color}",
                    img {
                        src: "{image.sizes.first().unwrap().url}"
                    }
                }
            }
            div {
                span { class: "name", "{item.name}" }
                div {
                    class: "additional",
                    div {
                        class: "quantity-control",
                        div {
                            class: "add",
                            onclick: move |_| {
                                cart_state.with_mut( |c| {c.inc(&item_id.read()); } );
                            },
                            span { "+" }
                        }
                        div {class: "display", span { "{quantity}"} }
                        div {
                            class: "remove",
                            onclick: move |_| {
                                cart_state.with_mut( |c| {c.dec(&item_id.read()); } );
                            },
                            span { "-"}
                        }
                    }
                    div {
                        "{item.price}kr"
                    }
                }
            }
        }
    }
}
#[component]
fn CartCounter() -> Element {
    let cart_state = use_cart();
    let count = use_memo(move || {
        let cart = cart_state.read();
        cart.num_items()
    });

    let class = {
        if *count.read() == 0 {
            "zero count"
        } else {
            "count"
        }
    };

    rsx! {
        div {
            class: class,
            span {"{count}"}
        }
    }
}

#[component]
pub fn CheckoutButton() -> Element {
    enum CheckoutState {
        Idle,
        Pending,
        Error(ServerFnError),
        Changed,
        Accepted,
    }

    let mut state = use_signal(|| CheckoutState::Idle);
    let mut cart_state = use_cart();

    let is_empty = use_memo(move || cart_state.read().is_empty());

    use CheckoutState::*;

    let current_state = &*state.read();
    match current_state {
        Changed | Idle => rsx! {

            if is_empty()
            {
                div {
                    class: "checkout disabled",
                    match current_state
                    {
                        Changed => "Varukorgen har uppdaterats",
                        _ => "Till kassa"
                    }
                }

            }
            else {
                div {
                    class: "checkout",
                    onclick: move |_| async move {

                        state.set(Pending);
                        let cart = &*cart_state.read();
                        match cart.checkout().await
                        {
                            Err(e) => {
                                spawn ( async move {
                                    if let Some(mut c) = CartState::load().await
                                    {
                                        if cart_state.read().contents != c.contents
                                        {
                                            c.open = MenuState::Opened;
                                            cart_state.set(c);
                                            state.set(CheckoutState::Changed);
                                        }
                                    }
                                });
                                state.set(Error(e))
                            },
                            Ok(_) => {
                                state.set(Accepted)
                            }
                        }
                    },
                    match current_state
                    {
                        Changed => "Varukorgen har uppdaterats",
                        _ => "Till kassa"
                    }
                }
            }

        },
        Pending => rsx! {
            div {
                class: "checkout",
                "Processerar..."
            }
        },
        Error(s) => rsx! {
            div {
                class:  "checkout",
                "Misslyckades"
            }
        },
        Accepted => rsx! {
            div {
                class:  "checkout",
                "Omdirigerar till stripe..."
            }
        },
    }
}

#[component]
pub fn CartContents() -> Element {
    let cart_state = use_cart();
    let content_class = use_memo(move || match cart_state.read().open {
        MenuState::Opened => "cart_contents opened",
        _ => "cart_contents",
    });

    let total = use_memo(move || {
        let cart = cart_state.read();
        cart.contents
            .values()
            .map(|(p, q)| q * p.price)
            .fold(0u32, |acc, c| acc + c)
    });

    rsx! {
            div {
                class: "{content_class}",
                div {
                    for id in cart_state.read().contents.keys()
                    {
                        CartItem {key: id, item_id: id.clone()  }
                    }

                    div {
                        class: "total",
                        "Totalt {total}kr"
                    }
                }
                CheckoutButton {}
            }
    }
}

#[component]
pub fn Cart() -> Element {
    let mut cart_state = use_cart();
    rsx! {
        div {
            onclick: move |_| {cart_state.write().toggle(); },
            class: "cart" ,
            CartCounter {}
            img {
                src: CART_ICON
            }

        }
    }
}
