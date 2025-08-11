use std::sync::Arc;

use image::ImageError;
use tracing::warn;
use sjf_db as db;

mod thumbnails;
mod cache;
mod object_storage;
pub use object_storage::init_bucket as init;

pub use thumbnails::upload_image;

#[derive(Debug)]
enum ErrorTypes {
    Image(image::ImageError),
    Io(std::io::Error),
    Internal(std::string::String),
    Sql(db::Error)
}

#[derive(Debug)]
pub struct Error {
    error: ErrorTypes
}

impl From<ImageError> for Error {
    fn from(value: ImageError) -> Self {
        Self {
            error: ErrorTypes::Image(value)
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self {
            error: ErrorTypes::Io(value)
        }
    }
}

impl From<db::Error> for Error {
    fn from(value: db::Error) -> Self {
        Self {
            error: ErrorTypes::Sql(value)
        }
    }
}


#[derive(Ord,Eq,PartialEq, PartialOrd,Clone,Debug)]
pub struct ImageId {
    image_id: u32,
    variant_id: u32
}
impl From<(u32,u32)> for ImageId {
    fn from( (image_id,variant_id): (u32,u32)) -> Self {
        ImageId { image_id, variant_id }
    }
}

impl From<ImageId> for (u32,u32)
{
    fn from(value: ImageId) -> Self {
        (value.image_id,value.variant_id)
    }

}

impl ImageId {
    pub fn resource_path(&self) -> String {
        format!("/images/{}/{}",self.image_id,self.variant_id)
    }
}


pub async fn get(id: ImageId) -> Option<Arc<Vec<u8>>>
{
    match cache::get_image(id.clone()).await
    {
        Some(d) => Some(d) ,
        None => {
            match object_storage::get_image(id.clone()).await 
            {
                Ok(d) => {
                    let rsp = Some(Arc::new(d.clone()));
                    tokio::spawn(
                        async move {
                            cache::add_image(id, d)
                        }
                    );
                    rsp
                },
                Err(e)=> {
                    warn!("Image get error {:#?}",e);
                    None
                }

            }
        }
    }
}






