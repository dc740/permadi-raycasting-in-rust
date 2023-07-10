use crate::loader::Texture;
use farfarbfeld::Decoder;
#[cfg(not(feature = "web"))]
use std::fs::File;
use std::{error::Error, io::Cursor};

#[cfg(not(feature = "web"))]
pub fn load_raw_bin(path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    use std::io::Read; // for the read_to_end
    let mut buffer = Vec::new();
    let new_path = ".".to_owned() + path;
    println!("Loading file {}", new_path);
    let mut file = File::open(new_path)?;
    buffer.clear();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn load_farbfeld(raw_bin: &[u8]) -> Result<Texture, Box<dyn Error>> {
    let buf = Cursor::new(raw_bin);
    let mut img = Decoder::new(buf)?; //this fails if the file is invalid
    let (w, h) = img.dimensions();
    let data = img
        .read_image()
        .unwrap()
        .chunks_exact(2)
        .into_iter()
        .map(|a| a[1]) //we could do .map(|a| u16::from_ne_bytes([a[0], a[1]])) here
        // but we only care about the first 8 bits
        .collect();
    Ok(Texture {
        width: w,
        height: h,
        data,
    })
}
