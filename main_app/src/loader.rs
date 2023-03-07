#[cfg(not(feature = "web"))]
use crate::generic_loader_impl::load_raw_bin;
#[cfg(feature = "web")]
use crate::web_setup::loader::download_raw_bin;
#[cfg(not(feature = "web"))]
use farfarbfeld::Decoder;
#[cfg(not(feature = "web"))]
use std::{error::Error, io::Cursor};

use std::collections::HashMap;
#[cfg(feature = "web")]
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u16>,
}

pub struct Assets {
    pub root: String,
    pub textures: HashMap<String, Texture>,
}

impl Assets {
    #[cfg(not(feature = "web"))]
    fn load_texture(&mut self, filename: &str) {
        let texture = self.load_farbfeld(filename);
        let f = match texture {
            Ok(texture) => texture,
            Err(error) => panic!("Problem opening the file: {:?}", error),
        };
        self.textures.insert(filename.to_string(), f);
    }
    /**
     * This is a sample function that loads a couple of textures
     *
     * on the web it simply sends the names to the worker
     * and we process the results in the worker callback.
     */
    #[cfg(not(feature = "web"))]
    pub fn load_some_textures(&mut self) {
        self.load_texture("/images/tile2.ff");
        self.load_texture("/images/green.ff");
        self.load_texture("/images/floortile.ff");
        self.load_texture("/images/tile41.ff");
        self.load_texture("/images/bgr.ff");
        self.load_texture("/images/brick2.ff");
        self.load_texture("/images/arma_32.ff");
    }
    #[cfg(feature = "web")]
    pub fn load_some_textures(&mut self, worker: Rc<RefCell<web_sys::Worker>>) {
        download_raw_bin(worker.clone(), "/images/tile2.ff");
        download_raw_bin(worker.clone(), "/images/green.ff");
        download_raw_bin(worker.clone(), "/images/floortile.ff");
        download_raw_bin(worker.clone(), "/images/tile41.ff");
        download_raw_bin(worker.clone(), "/images/bgr.ff");
        download_raw_bin(worker.clone(), "/images/brick2.ff");
        download_raw_bin(worker.clone(), "/images/arma_32.ff");
    }

    #[cfg(not(feature = "web"))]
    fn load_farbfeld(&self, path: &str) -> Result<Texture, Box<dyn Error>> {
        let raw_bin = load_raw_bin(&(".".to_owned() + path)); //fix path so it finds the files
        let buf = Cursor::new(raw_bin.unwrap()); //this unwrap throws erros if the file doesn't exist
        let mut img = Decoder::new(buf).unwrap(); //this one if the file is invalid
        let (w, h) = img.dimensions();
        let data = img
            .read_image()
            .unwrap()
            .chunks_exact(2)
            .into_iter()
            .map(|a| u16::from_ne_bytes([a[0], a[1]]))
            .collect();
        Ok(Texture {
            width: w,
            height: h,
            data,
        })
    }
}
