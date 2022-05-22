use std::error::Error;
use std::fs::File;
use std::io::Read;

pub fn load_raw_bin(path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = Vec::new();
    let mut file = File::open(path)?;
    buffer.clear();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
