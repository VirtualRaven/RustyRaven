use serde::{Deserialize, Serialize};


#[derive(Debug,Serialize,Deserialize,Clone,PartialEq)]
pub struct ImageVariant 
{
    pub width: u32,
    pub height: u32,
    pub url: String
}
#[derive(Debug,Serialize,Deserialize,Clone,PartialEq)]
pub struct Image 
{
    pub color: String,
    pub sizes: Vec<ImageVariant>
}

impl Image {
    pub fn srcset(&self) -> Option<String>
    {
        self.sizes.iter().map(|v| {
            format!("{} {}w",v.url,v.width)
        })
        .reduce(|acc,b | {
            acc + ", " + &b
        })


    }
}

#[derive(Debug,Serialize,Deserialize,Clone,PartialEq)]
pub struct Preview
{
    pub id: u32,
    pub name: String,
    pub price: u32,
    pub images: Vec<Image>
}



#[derive(Debug,Serialize,Deserialize,Clone,PartialEq)]
pub struct  GetPreviewsRequest 
{
    pub category: Option<u32>,
    pub recursive: bool,
    pub limit: u32,
}

#[derive(Debug,Serialize,Deserialize,Clone,PartialEq)]
pub struct GetPreviewsResp 
{
    pub previews: Vec<Preview>
    


}