use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use dioxus::prelude::server_fn::error::NoCustomError;
use serde::{Deserialize, Serialize};
use dioxus::prelude::*;
use dioxus::logger::tracing::{info, warn};
#[cfg(feature="server")]
use sjf_db as db;

#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(bound = "T: Serialize, for<'de2> T: Deserialize<'de2>")]
pub struct AuthenticatedRequest<T>
where
T: Serialize,
for<'de2> T: Deserialize<'de2>
{
    pub data: T 
}


#[derive(Debug,Serialize,Deserialize,Clone,PartialEq)]
pub enum ProductTag {
    Clothing,
    Lamp
}
#[derive(Debug,Serialize,Deserialize,Clone,PartialEq)]
pub struct Product {

    pub id: Option<i32>,
    pub name: String, //VARCHAR(100)
    pub price: u16,
    pub description: String,
    pub quantity: Option<u16>,
    pub product_tag: Option<Vec<ProductTag>>,
    pub images: Option<BTreeSet<u32>>,
    pub tax_rate: u32,
    pub category: u32,
}


impl  Product  {
    pub fn new(category: u32) -> Self 
    {
        Product { id: None, name: String::from(""), price: 0, description: String::from(""), quantity: None, product_tag: None, images: None, tax_rate: 25, category }

    }
}

#[cfg(feature="server")]
fn error_logger<T>(t : Result<T,db::Error>  ) -> Result<T,ServerFnError>
{
    use dioxus::prelude::ServerFnError::ServerError;
    match t 
    {
        Err(e) => {
            warn!("{:#?}", e);
            Err(ServerError("Endpoint failure".into()))
        },
        Ok(rsp) => Ok(rsp)
    }
}

#[cfg(feature="server")]
impl From<db::ProductTag> for ProductTag 
{
    fn from(tag: db::ProductTag) -> Self
    {
        match tag {
            db::ProductTag::Lamp => ProductTag::Lamp,
            db::ProductTag::Clothing => ProductTag::Clothing,
        }
    }

}

#[cfg(feature= "server")]
impl From<db::Product> for Product 
{
    fn from(product: db::Product) -> Self 
    {
        let tags = {
            if product.product_tag.is_empty() { None} 
            else {
                Some(product.product_tag.into_iter().map(|x| x.into()).collect())
            }
        };
        let images = {
            Some(product.images.into_iter().collect())
        };
        Self {
            id: Some(product.id),
            name: product.name,
            price: product.price as u16,
            description: product.description,
            quantity: product.quantity.map(|x| x as u16),
            product_tag: tags,
            images: images,
            tax_rate: product.tax_rate,
            category: product.category 
        }
    }
}

#[cfg(feature="server")]
impl From<ProductTag> for db::ProductTag 
{
    fn from(tag: ProductTag) -> Self
    {
        match tag {
            ProductTag::Lamp => db::ProductTag::Lamp,
            ProductTag::Clothing => db::ProductTag::Clothing,
        }
    }

}

#[cfg(feature="server")]
impl From<Product> for db::Product 
{
    fn from(product: Product) -> Self 
    {
        db::Product {
            id: product.id.unwrap_or(0),
            name: product.name,
            price: product.price as i32,
            created: Default::default(),
            updated: Default::default(),
            description: product.description,
            quantity: product.quantity.map(|x| x as i32),
            product_tag: {
                product.product_tag.unwrap_or(vec![]).into_iter().map(|x|
                    x.into()
                ).collect()
            },
            images: product.images.unwrap_or_default().into_iter().collect(),
            tax_rate: product.tax_rate,
            category: product.category,
        }
    }
}


#[server]
pub async fn get_products(category: u32) -> Result<Vec<Product>,ServerFnError> 
{
    use dioxus::prelude::ServerFnError::ServerError;
    let resp = db::get_products(category).await;

    match resp {
        Ok(products) => {
            Ok(
                products.into_iter().map( |product|{
                    product.into()
                }).collect()
            )

        },
        Err(e) => {
            warn!("serverFn get_products failed: {:#?}", e);
            Err(ServerError( "Product get failed".into()))
        }
    }

}

#[server]
pub async fn get_product_images(req: AuthenticatedRequest<u32> ) -> Result<BTreeMap<u32,Vec<u32>>,ServerFnError> 
{
    use dioxus::prelude::ServerFnError::ServerError;
    match db::image::get_product_images(req.data.clone()).await
    {
        Ok(v) => Ok(v),
        Err(e) => {
            warn!("get_product_images({}) failed with {:#?}",req.data,e);
            Err(ServerError( "get_product_images failed".into()))

        }
    }

}

#[server]
pub async fn store_product(req: AuthenticatedRequest<Product> ) -> Result<i32,ServerFnError> 
{
    //IF authenticated

    let product_id = req.data.id.clone();
    let product: db::Product = req.data.into();
    use dioxus::prelude::ServerFnError::ServerError;
    match product_id
    {
        Some(req_id) => {
            match db::update_product(product).await
            {
                Ok(()) => Ok(req_id),
                Err(e) => {
                    warn!("serverFn store_product failed update: {}", e);
                    Err(ServerError("Store failed".into()))
                }
            }
        },
        None => {
            match db::create_product(product).await 
            {
                Ok(id) => Ok(id),
                Err(e) => {
                    warn!("serverFn store_product failed create: {}", e);
                    Err(ServerError( "Store failed".into()))

                } 
            }
        }
    }

}

#[server]
pub async fn upload_images(req: AuthenticatedRequest<Vec<Vec<u8>>>) -> Result<Vec<(u32,u32)>,ServerFnError>
{
    info!("Uploading {} images", req.data.len());
    use sjf_image as image;
    use futures::future::join_all;
    
    let res: Vec<_> = join_all(req.data.into_iter().map(image::upload_image)).await;

    res.iter().filter_map(|x| x.as_ref().err()).for_each(|e| warn!("upoad_images: {:#?}", e) );
    let sucessful: Vec<_> = res.into_iter().filter_map(|x| x.ok() ).map(|x| x.into() ).collect();


    use dioxus::prelude::ServerFnError::ServerError;
    if sucessful.is_empty()
    {
        Err(ServerError("Image processing failed".into()))
    }
    else
    {
        Ok(sucessful)

    }



}


pub mod category {
    use super::*;
    use dioxus::prelude::server_fn::codec::Json;
    pub use sjf_api::category::*;




#[server]
pub async fn create(req: AuthenticatedRequest<CreateReq>) -> Result<CreateRsp,ServerFnError>
{
    error_logger(db::category::create(req.data).await)
}
#[server(endpoint="get_children",input=dioxus::prelude::server_fn::codec::GetUrl)]
pub async fn get_children( p: Option<u32>) -> Result<GetChildrenRsp,ServerFnError>
{
    error_logger( db::category::get_children(p).await)
}


#[server(input=Json)]
pub async fn update_name(req: AuthenticatedRequest<(u32,String)>) -> Result<(),ServerFnError>
{
    error_logger(db::category::update_name(req.data.0, req.data.1).await)
}

}

use sjf_api::product::{GetPreviewsRequest, GetPreviewsResp};
#[server(endpoint="get_previews",input=dioxus::prelude::server_fn::codec::GetUrl)]
pub async fn get_previews( p: Option<u32>, r: bool) -> Result<GetPreviewsResp,ServerFnError>
{
    let r = GetPreviewsRequest {
        recursive: r,
        category: p,
        limit:100
    };
    error_logger( db::product::get_previews(r).await)
}