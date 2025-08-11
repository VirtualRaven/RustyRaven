use serde::{Deserialize, Serialize};


#[derive(Serialize,Deserialize,Debug,PartialEq,Clone)]
pub struct CreateReq
{
    pub name: String,
    pub parent: Option<u32>
}

#[derive(Serialize,Deserialize,Debug,PartialEq,Clone)]
pub struct CreateRsp 
{
    pub id: u32,
    pub depth: u32
}


#[derive(Serialize,Deserialize,Debug,PartialEq,Clone)]
pub struct DeleteReq 
{
    pub id: u32
}

#[derive(Serialize,Deserialize,Debug,PartialEq,Clone)]
pub struct GetChildrenRsp 
{
    pub children: Vec<(u32,String)>
}