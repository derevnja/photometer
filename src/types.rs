use time::{ Timespec };
use std::fmt::{ Show, Formatter, Result };

pub type Id = i64;

#[deriving(Show)]
pub struct PhotoInfo {
    pub id: Id,
    pub upload_time: Timespec,
    pub image_type: ImageType,
    pub width: u32,
    pub height: u32,
    pub name: String,
    pub iso: Option<u32>,
    pub shutter_speed: Option<i32>,
    pub aperture: Option<f32>,
    pub focal_length: Option<u16>,
    pub focal_length_35mm: Option<u16>,
    pub camera_model: Option<String>
}

#[deriving(PartialEq)]
pub enum ImageType {
    Jpeg,
    Png
}

impl Show for ImageType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &ImageType::Jpeg => write!(f, "{}", "jpg".to_string() ),
            &ImageType::Png => write!(f, "{}", "png".to_string() )
        }
    }   
}