
enum MenuState 
{
    Closed,
    Opened
}

impl MenuState
{
    pub fn toggle(&self) -> Self
    {
        match self {
            MenuState::Closed => MenuState::Opened,
            MenuState::Opened => MenuState::Closed
        }
    }
}


mod header;
pub use header::*;
mod footer;
pub use footer::*;
mod cart;
pub use cart::Cart;
mod product;
pub use product::list::ProductList;
mod close_button;
pub use close_button::CloseButton;