use ::stripe::Client;


mod stripe;
pub use stripe::{checkout,init,CANCLE_PATH,SUCCESS_PATH};

#[derive(thiserror::Error,Debug)]
pub enum PaymentError 
{
    #[error("Sql failed {0}")]
    Sql(#[from] sjf_db::checkout::CheckoutError),
    #[error("Stripe failed {0}")]
    Stripe(#[from] ::stripe::StripeError),
    #[error("Invalid tax rate {0}")]
    InvalidTaxRate(u8),
    #[error("Stripe didn't return a URL for checkout")]
    NoUrl
} 

