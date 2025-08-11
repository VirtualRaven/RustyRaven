
use std::collections::BTreeMap;

use dioxus::prelude::{server_fn::ServerFn, *};
use crate::components::{MenuState};
use sjf_api::{checkout::CheckoutRequest, product::{Product, ProductId}};

const CART_ICON: Asset = asset!("/assets/cart.png");

pub use u32 as ProductQuantity;


pub struct CartState {
    open: MenuState,
    contents: BTreeMap<ProductId, (Product , ProductQuantity) >
}

impl CartState {
    
    pub fn new() -> Self {
        Self {
            open:     MenuState::Closed,
            contents: Default::default()
        }
    }

    pub fn has_item(&self, id: &ProductId) -> bool
    {
        self.contents.get(id).is_some()
    }
    pub fn is_empty(&self) -> bool
    {
        self.contents.is_empty()
    }

    pub async fn checkout(&self) -> Result<String,ServerFnError>
    {
        let req  = CheckoutRequest { order: self.contents.clone().into_iter().map(|(id,(_,quantity))| { (id,quantity) }).collect() };
        let res = crate::server::checkout(req).await;
        if let Ok(payment_url) = &res 
        {
             let nav = navigator();
             nav.push( NavigationTarget::<crate::Route>::External(payment_url.clone()) );
        }
        res
    }

    pub fn add_item(&mut self, product: Product)
    {
        let key = product.id;
        match self.contents.get_mut(&key)
        {
            Some(_) =>  { self.inc(&product.id); }
            None => {
                self.contents.insert( key, (product,1) );
            }
        }
        self.open = MenuState::Opened;
    } 

    pub fn clear(&mut self)
    {
        self.contents.clear();
    }

    fn inc(&mut self, id: &ProductId)
    {
        match self.contents.get_mut(id)
        {
            Some((p,q)) =>  { 
                *q=*q+1;
                match p.stock 
                {
                    Some(stock) if *q >= stock => { *q = stock }
                    _ => { }
                }
            }
            None => {}
        }
    }
    fn dec(&mut self, id: &ProductId)
    {
        match self.contents.get_mut(id)
        {
            Some((_,q)) if *q <= 1 => {
                self.contents.remove(id);
            }
            Some((_,q)) =>  { *q=*q-1; }
            None => {}
        }

        if self.contents.is_empty()
        {
            self.open = MenuState::Closed;
        }
    }

    pub fn load() -> Self {
        Self::new()
    }

    fn num_items(&self) -> u32 
    {
        self.contents
        .values()
        .map(|(_,q)| q)
        .fold(0, |acc,q| acc+q )
    }

    fn toggle(&mut self)
    {
        self.open = self.open.toggle();
    }

}

type CartSignal = Signal<CartState>;
pub fn  use_cart() -> CartSignal  {  use_context::<CartSignal>() }



#[component]
fn CartItem(item_id: ReadOnlySignal<ProductId> ) -> Element {
    let mut cart_state = use_cart();


    let cart = cart_state.read();
    let (item,quantity) = cart.contents.get(&*item_id.read()).unwrap();
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
    let count = use_memo( move || {
        let cart = cart_state.read();
        cart.num_items()
    });


    let class = {
        if *count.read() == 0 
        {
            "zero count"
        }
        else {"count"}
    };

    rsx! {
        div {
            class: class,
            span {"{count}"}
        }
    }

}

#[component]
pub fn CheckoutButton()-> Element 
{
    enum CheckoutState {
        Idle,
        Pending,
        Error(ServerFnError),
        Accepted
    }

    let mut state = use_signal(|| CheckoutState::Idle );
    let cart_state = use_cart();

    let is_empty = use_memo(move || {
        cart_state.read().is_empty()
    });


    use CheckoutState::*;


    let current_state = &*state.read();
    match current_state {
        Idle => rsx!{

            if is_empty()
            {
                div {
                    class: "checkout disabled",
                    "Till kassa"
                }

            }
            else {
                div {
                    class: "checkout",
                    onclick: move |_| async move {

                        state.set(Pending);
                        let cart_state = &*cart_state.read();
                        match cart_state.checkout().await
                        {
                            Err(e) => {
                                state.set(Error(e))
                            },
                            Ok(_) => {
                                state.set(Accepted)
                            }
                        }
                    },
                    "Till kassa"
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
        }
    }




}

#[component]
pub fn CartContents() -> Element {
    let cart_state = use_cart();
    let content_class =  use_memo( move ||  {
        match cart_state.read().open {
        MenuState::Opened => "cart_contents opened",
        _ => "cart_contents"
        }
    });

    let total = use_memo(move || {
        let cart = cart_state.read();
        cart.contents.values().map( |(p,q)| q * p.price ).fold(0u32, |acc,c| acc+c)
    } );

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

