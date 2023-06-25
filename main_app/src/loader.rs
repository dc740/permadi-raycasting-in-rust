#[cfg(not(feature = "web"))]
use crate::generic_loader_impl::{load_raw_bin, load_farbfeld};
#[cfg(feature = "web")]
use crate::web_setup::loader::download_raw_bin;

use serde::{Serialize, Deserialize};

use std::collections::HashMap;
#[cfg(feature = "web")]
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub struct Assets {
    pub root: String,
    pub resources: Option<ResourceIndex>,
    pub textures: HashMap<u32, Texture>,
    pub loader: Box<dyn FileLoader>,
}

#[derive(Serialize, Deserialize)]
pub struct ResourceIndex {
    pub images: Vec<ResourceImage>,
}

#[derive(Serialize, Deserialize)]
pub struct ResourceImage {
    pub id: u32,
    pub name: String,
    pub path: String,
}

pub trait FileLoader {
    /**
     * Loads all textures detailed in the index file
     */
    fn load_textures(&mut self, resource_index: &ResourceIndex, textures: &mut HashMap<u32, Texture>);
    /**
     * Downloads the index file that contains the list of
     * textures to download
     */
    fn load_index_file(&mut self) -> Option<ResourceIndex>;
}

#[cfg(not(feature = "web"))]
pub struct LocalFileLoader {
}

#[cfg(not(feature = "web"))]
impl FileLoader for LocalFileLoader {
    fn load_textures(&mut self, resource_index: &ResourceIndex, textures: &mut HashMap<u32, Texture>){
        for img in &resource_index.images {
            let raw_bin = load_raw_bin(&img.path); //TODO: improve fix to path so it finds the files and works with web
            let texture = load_farbfeld(&raw_bin.unwrap()); //this unwrap throws erros if the file doesn't exist

            let f = match texture {
                Ok(texture) => texture,
                Err(error) => panic!("Problem opening the file: {:?}", error),
            };
            textures.insert(img.id, f);
        }
    }
    fn load_index_file(&mut self) -> Option<ResourceIndex>{
        let raw_bin = load_raw_bin(&("/resources.json".to_owned())).unwrap();
        let resources_str = std::str::from_utf8(&raw_bin).unwrap();
        Some(serde_json::from_str(&resources_str).unwrap())
    }
}

#[cfg(feature = "web")]
pub struct WebFileLoader {
    pub worker : Rc<RefCell<web_sys::Worker>>,
}

#[cfg(feature = "web")]
impl FileLoader for WebFileLoader {
    fn load_textures(&mut self, resource_index: &ResourceIndex, _textures: &mut HashMap<u32, Texture>){
        for img in &resource_index.images {
            download_raw_bin(self.worker.clone(), &img.path);
            // TODO: move farbled loading and texture inserts here.
            // It is currently setup in the web module, with the worker
            // callback.
            // load_farbfeld(...)
            //textures.insert(img.path[1..].to_string(), f);
        }
    }
    fn load_index_file(&mut self) -> Option<ResourceIndex> {
        download_raw_bin(self.worker.clone(), &("/resources.json".to_owned()));
        None
    }
}

impl Assets {
    pub fn init(&mut self){
        self.resources = self.loader.load_index_file();
    }


    pub fn load(&mut self){
        if let Some(resources) = &self.resources {
            self.loader.load_textures(&resources, &mut self.textures)
        } else {
            panic!("Resources file not loaded");
        }
    }
}
