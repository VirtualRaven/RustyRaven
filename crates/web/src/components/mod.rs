
#[derive(PartialEq)]
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

mod about;
pub use about::*;
mod terms;
pub use terms::TermsAndConditions;

mod header;
pub use header::*;
mod footer;
pub use footer::*;
mod cart;
pub use cart::{Cart,CartState,CartContents,use_cart};
mod admin;
pub use admin::product::list::ProductList;
pub use admin::category::CategoryList;
mod close_button;
pub use close_button::CloseButton;
mod image_upload;
pub use image_upload::ImageUploadButton;
mod front_page;
pub use front_page::FrontPage;
pub use front_page::ProductPreview;
mod product;
pub use product::*;
mod category;
pub use category::*;
mod category_bar;
pub use category_bar::*;
mod order;
pub use order::*;