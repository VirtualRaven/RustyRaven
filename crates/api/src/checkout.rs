use std::collections::BTreeMap;


use serde::{Deserialize, Serialize};
pub use u32 as ProductId;
pub use u32 as ProductQuantity ;

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct CheckoutRequest 
{
    pub order: BTreeMap<ProductId,ProductQuantity>
}