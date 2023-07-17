use macroquad::prelude::{Image, ImageFormat};
use std::fs::File;
use std::io::{Error, Read, Write};

fn main() {
    convert_to_raw("icon_16x16").unwrap();
    convert_to_raw("icon_32x32").unwrap();
    convert_to_raw("icon_64x64").unwrap();
}

fn convert_to_raw(path: &str) -> Result<(), Error> {
    let mut bytes = vec![];
    File::open(format!("icon/{}.png", path))?.read_to_end(&mut bytes)?;
    let bytes = Image::from_file_with_format(&bytes, Some(ImageFormat::Png)).bytes;
    File::create(format!("icon/{}.rgba", path))?.write_all(&bytes)?;
    Ok(())
}
