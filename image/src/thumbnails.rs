use image::{DynamicImage, ImageDecoder, ImageEncoder, ImageError, ImageReader };
use log::{info, warn};
use std::{collections::VecDeque, io::Write };
use sjf_db as db;

use crate::ImageId;


#[derive(Clone,Debug,PartialEq)]
enum ThumbnailSize {
    Small,
    Medium,
    Large,
    Huge
}

impl Into<u32> for ThumbnailSize {
    fn into(self) -> u32 {
        match self {
            ThumbnailSize::Small  => 256,
            ThumbnailSize::Medium => 512,
            ThumbnailSize::Large  => 2048,
            ThumbnailSize::Huge   => 3000,
        }
    }
}
impl Into<&'static str > for ThumbnailSize {
    fn into(self) -> &'static str {
        match self {
            ThumbnailSize::Small  => "small",
            ThumbnailSize::Medium => "medium",
            ThumbnailSize::Large  => "large",
            ThumbnailSize::Huge   => "huge",
        }
    }
}

struct LoadedImage {
    image: DynamicImage,
    icc_profile: Option<Vec<u8>>
}

fn load_image(bytes: Vec<u8> ) -> Result<LoadedImage,ImageError>
{

    let reader = ImageReader::new(std::io::Cursor::new(bytes))
    .with_guessed_format().expect("Cursor io never fails");

    let mut decoder = reader.into_decoder()?;

    let icc_profile = decoder.icc_profile()?;

    Ok (
        LoadedImage {
            image: DynamicImage::from_decoder(decoder)?,
            icc_profile
        }
    )
}
fn generate_thumbnail(image: &DynamicImage, size: ThumbnailSize ) -> DynamicImage  {

    let size: u32 = size.into();
    image.resize(
            size,
            size,
            image::imageops::FilterType::Lanczos3,
    )
}


pub async fn upload_image(bytes: Vec<u8>) -> Result<ImageId,crate::Error>
{
    let image = load_image(bytes)?;
    use image::codecs::jpeg::JpegEncoder;


    use ThumbnailSize::*;
    let desired_sizes = [Small,
    Medium,
    Large,
    Huge];
    

    let hex_color = {
        let rgb_img = image.image.thumbnail(300,300) .into_rgb8();
        let (width, height) = rgb_img.dimensions();

        let (mut r, mut g, mut b) = (0u128, 0u128, 0u128);
        for (_, row) in rgb_img.enumerate_rows() {
            let (mut rr, mut rg, mut rb) = (0u128, 0u128, 0u128);
            for (_, _, pixel) in row {
                rr += u128::from(pixel[0]);
                rg += u128::from(pixel[1]);
                rb += u128::from(pixel[2]);
            }
            r += rr / u128::from(width);
            g += rg / u128::from(width);
            b += rb / u128::from(width);
        }
        r /= u128::from(height);
        g /= u128::from(height);
        b /= u128::from(height);

        String::from(&format!("{:#08x}", r << 16 | g << 8 | b << 0)[2..])
    };

    struct EncodedImageData {
        data: Vec<u8>,
        width: u32,
        height: u32,
        
    }

    let mut init: Vec<EncodedImageData> = Vec::new();
    init.reserve(desired_sizes.len());
    let images: Result<_,crate::Error> = 
     desired_sizes
    .into_iter()
    .map(|size|  generate_thumbnail(&image.image, size)  )
    .try_fold(init,|mut acc: Vec<EncodedImageData> ,thumbnail| {
        let mut w = std::io::Cursor::new(Vec::new());
        let mut encoder = JpegEncoder::new_with_quality(&mut w, 80);
        if let Some(ref profile) = image.icc_profile
        {
            encoder.set_icc_profile(profile.clone());
        }
        
        thumbnail.write_with_encoder(encoder)?;
        w.flush()?;

        acc.push(
            EncodedImageData {
                data: w.into_inner(),                
                width: thumbnail.width(),
                height: thumbnail.height(),
            }
        );
        Ok(acc)
    });

    let mut images = images?;

    let req = {
        let mut vs : VecDeque<_> = images.iter().map(|img|
                db::image::ImageInsertVariant {
                    width: img.width as i32,
                    height: img.height as i32
                }
            ).collect();
        db::image::ImageInsertRequest {
            avg_color: hex_color,
            variants: [
                vs.pop_front().unwrap(), 
                vs.pop_front().unwrap(), 
                vs.pop_front().unwrap(), 
                vs.pop_front().unwrap()
            ]
        }
    };


    
    let mut image_ids = db::image::insert_image(req).await?;




    let id = { 
        let id = image_ids.last().unwrap().clone();
        ImageId {
            image_id: id.image_id as u32,
            variant_id: id.variant_id as u32
        }
    };
    let data = images.last().unwrap().data.clone();

    let result = Ok(id.clone());


    info!("Uploaded image");
    tokio::spawn(
        async move {
            crate::cache::add_image(id, data).await;
        }
    );

    tokio::spawn(
        async move {
            let iter = image_ids
                .into_iter()
                .zip(images.into_iter());
                for (id,image) in iter {
                    match crate::object_storage::put_image((id.image_id as u32 ,id.variant_id as u32).into(), &image.data).await
                    {
                        Ok(()) => (),
                        Err(e) => warn!("Image put error {:#?}",e)
                    }
                }
        }
    );

    return result;

}  



