use image2::{color, ImageBuf};
use serde::{Deserialize, Serialize};
use type_uuid::*;

#[derive(TypeUuid, Serialize, Deserialize, Debug)]
#[uuid = "d4079e74-3ec9-4ebc-9b77-a87cafdfdada"]
pub enum ImageAsset {
    Rgb8(ImageBuf<u8, color::Rgb>),
    // ...
}
