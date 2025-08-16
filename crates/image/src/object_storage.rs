use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::env;
use std::time::Duration;

use crate::ImageId;
use thiserror::Error;

use aws_sdk_s3::{
    Client,
    config::http::HttpResponse,
    error::SdkError,
    operation::{
        create_bucket::CreateBucketError, get_object::GetObjectError,
        list_buckets::ListBucketsError, put_object::PutObjectError,
    },
    primitives::{ByteStream, ByteStreamError},
    types::{CreateBucketConfiguration, LocationInfo},
    waiters::bucket_exists,
};

const BUCKET_NAME: &str = "sjf-images-bucket";
static CLIENT: OnceCell<Box<Client>> = OnceCell::new();

#[derive(Error, Debug)]
pub enum ObjectStorageError {
    #[error("Failed to list buckets: {0}")]
    ListBucketError(#[from] SdkError<ListBucketsError, HttpResponse>),
    #[error("Failed Create bucket: {0}")]
    CreateBucketError(#[from] SdkError<CreateBucketError, HttpResponse>),
    #[error("Failed object from bucket: {0}")]
    GetObjectError(#[from] SdkError<GetObjectError, HttpResponse>),
    #[error("Failed object from bucket: {0}")]
    PutObjectError(#[from] SdkError<PutObjectError, HttpResponse>),
    #[error("ByteStreamError: {0}")]
    ByteStreamError(#[from] ByteStreamError),
}

fn client() -> &'static Box<Client> {
    CLIENT.get().expect("Client should have been intialized")
}

pub async fn init_bucket() -> Result<(), ObjectStorageError> {
    use aws_sdk_s3::config::Credentials;
    use aws_sdk_s3::config::Region;
    let key_id = dotenvy::var("S3_ACCESS_KEY_ID")
        .expect("Environment variable S3_ACCESS_KEY_ID is required");
    let secret_key = dotenvy::var("S3_SECRET_ACCESS_KEY")
        .expect("Environment variable S#_SECRET_ACCESS_KEY is required");
    let cred = Credentials::new(&key_id, &secret_key, None, None, "loaded-from-custom-env");
    let endpoint = dotenvy::var("OBJECT_STORAGE_URI")
        .expect("Environment variable OBJECT_STORAGE_URI required")
        .to_owned();
    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(endpoint)
        .credentials_provider(cred)
        .region(Region::new("eu-central-1"))
        .force_path_style(true) // apply bucketname as path param instead of pre-domain
        .behavior_version_latest()
        .build();

    let client = aws_sdk_s3::Client::from_conf(s3_config);

    let resp = client.list_buckets().send().await?;
    let bucket_exists = resp
        .buckets()
        .iter()
        .find(|b| b.name().unwrap_or_default() == BUCKET_NAME)
        .is_some();

    if !bucket_exists {
        client
            .create_bucket()
            .set_bucket(Some(BUCKET_NAME.into()))
            .send()
            .await?;
    }

    CLIENT.set(client.into()).unwrap();

    Ok(())
}

fn id_filename(id: ImageId) -> String {
    format!("{}-{}.jpeg", id.image_id, id.variant_id)
}

pub async fn get_image(id: ImageId) -> Result<Vec<u8>, ObjectStorageError> {
    let rsp = client()
        .get_object()
        .bucket(BUCKET_NAME)
        .set_key(Some(id_filename(id)))
        .send()
        .await?;
    Ok(rsp.body.collect().await?.to_vec())
}

pub async fn put_image(id: ImageId, data: Vec<u8>) -> Result<(), ObjectStorageError> {
    client()
        .put_object()
        .set_bucket(Some(BUCKET_NAME.into()))
        .set_key(Some(id_filename(id)))
        .set_content_type(Some("image/jpeg".into()))
        .set_body(Some(data.into()))
        .send()
        .await?;
    Ok(())
}
