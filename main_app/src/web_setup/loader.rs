use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Worker;

pub fn download_raw_bin(worker_handle: Rc<RefCell<Worker>>, path: &str) {
    let _ = worker_handle
        .borrow_mut()
        .post_message(&JsValue::from_str(&path));
}
