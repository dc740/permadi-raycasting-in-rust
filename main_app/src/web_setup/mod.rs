pub(crate) mod error;
pub(crate) mod loader;

use console_error_panic_hook;
use farfarbfeld::Decoder;
use js_sys::Uint8Array;
use minifb::{Window, WindowOptions};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::panic;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;
use web_sys::MessageEvent;
use web_sys::Worker;

use crate::game::GameWindow;
use crate::loader::Texture;

const WIDTH: usize = 320;
const HEIGHT: usize = 200;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen(start)]
pub fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut raycast = GameWindow::new(WIDTH, HEIGHT);
    let downloaded_textures: Rc<RefCell<HashMap<String, Texture>>> =
        Rc::new(RefCell::new(HashMap::new()));
    let textures_buffer = Rc::clone(&downloaded_textures);
    let mut window = Window::new("Bouncy Box demo", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
    // A reference counted pointer to the closure that will update and render the game
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    // we update the window here just to reference the buffer
    // internally. Next calls to .update() will use the same buffer
    window
        .update_with_buffer(raycast.get_buffer_to_print(), WIDTH, HEIGHT)
        .unwrap();
    let worker_handle = start_file_downloader_worker(textures_buffer);
    raycast.init();

    #[cfg(not(feature = "web"))]
    raycast.assets.load_some_textures();
    #[cfg(feature = "web")]
    raycast.assets.load_some_textures(worker_handle.clone());

    let mut textures_loaded = false;

    // create the closure for updating and rendering the game.
    *g.as_ref().borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if textures_loaded {
            // game step
            raycast.game_step(&window);

            // as the buffer is referenced from inside the ImageData, and
            // we push that to the canvas, so we could call update() and
            // avoid all this. I don't think it's possible to get artifacts
            // on the web side, but I definitely see them on the desktop app
            let result = window.update_with_buffer(raycast.get_buffer_to_print(), WIDTH, HEIGHT);
            match result {
                Ok(_) => {
                    ();
                }
                Err(_) => console::log_1(&"Error updating loop".into()),
            };
        } else {
            //check if there is any new texture available, and move it to the assets
            for (key, value) in downloaded_textures.as_ref().borrow().iter() {
                raycast.assets.textures.insert(key.clone(), value.clone());
            }

            /* clean up afterwards? nah, not filling like it
            match downloaded_textures.borrow_mut() {
                Ok(textures) => {
                    for (key, value) in &mut raycast.assets.textures {
                        textures.remove(&key.as_str()); // only remove the ones we have
                    }
                },
                Err(_) => (),
            };*/
            if raycast.assets.textures.len() >= 7 { //FIXME: Remember to update this when you add more textures (or refactor this hack)
                console::log_1(&"All textures have been loaded. Time to start the game.".into());
                textures_loaded = true;
                worker_handle.as_ref().borrow_mut().terminate();
            }
        }
        // schedule this closure for running again at next frame
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut() + 'static>));

    // start the animation loop
    request_animation_frame(g.borrow().as_ref().unwrap());
}

pub fn start_file_downloader_worker(
    textures_buffer: Rc<RefCell<HashMap<String, Texture>>>,
) -> Rc<RefCell<web_sys::Worker>> {
    // This is not strictly needed but makes debugging a lot easier.
    // Should not be used in productive deployments.
    set_panic_hook();

    // Here, we create our worker. In a larger app, multiple callbacks should be able to interact
    // with the code in the worker. Therefore, we wrap it in `Rc<RefCell>` following the interior
    // mutability pattern. In this example, it would not be needed but we include the wrapping
    // anyway as example.
    let worker_handle = Rc::new(RefCell::new(
        Worker::new("./js/file_download_worker.js").unwrap(),
    ));
    console::log_1(&"Created a new worker from within WASM".into());

    // Pass the worker to the function which sets up the `onchange` callback.
    setup_file_downloader_worker_callback(worker_handle.clone(), textures_buffer);
    worker_handle
}

fn setup_file_downloader_worker_callback(
    worker: Rc<RefCell<web_sys::Worker>>,
    textures_buffer: Rc<RefCell<HashMap<String, Texture>>>,
) {
    // Access worker behind shared handle, following the interior mutability pattern.
    let worker_handle = &*worker.borrow();
    //let _ = worker_handle.post_message(&number.into());
    let persistent_callback_handle = get_on_msg_callback(textures_buffer);

    // Since the worker returns the message asynchronously, we attach a callback to be
    // triggered when the worker returns.
    worker_handle.set_onmessage(Some(persistent_callback_handle.as_ref().unchecked_ref()));

    persistent_callback_handle.forget(); // AFAIK, this needs to be leaked
}

/// Create a closure to act on the message returned by the worker
fn get_on_msg_callback(
    textures_buffer_rc: Rc<RefCell<HashMap<String, Texture>>>,
) -> Closure<dyn FnMut(MessageEvent)> {
    let textures_buffer = Rc::clone(&textures_buffer_rc);
    let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
        console::log_2(&"Received response: ".into(), &event.data().into());

        let result = event.data();
        let uint8_array = Uint8Array::new(&result); //we need to extract the filename from here
        let full_array: Vec<u8> = uint8_array.to_vec();
        let index_element = full_array
            .iter()
            .position(|&x| x == 0x7C) // we search for the pipe character
            .unwrap();
        let filename = &full_array[0..index_element];
        let filename_str = std::str::from_utf8(&filename).expect("invalid utf-8 sequence");
        let blob = &full_array[index_element + 1..];
        let mut buffer = textures_buffer.as_ref().borrow_mut();
        let buf = Cursor::new(blob); //this unwrap throws erros if the file doesn't exist
        let mut img = Decoder::new(buf).unwrap();
        let data = img
            .read_image()
            .unwrap()
            .chunks_exact(2)
            .into_iter()
            .map(|a| a[1])
            .collect();

        let (w, h) = img.dimensions();
        buffer.insert(
            filename_str.to_string(),
            Texture {
                width: w,
                height: h,
                data,
            },
        ); // store the blob, will be parsed later
    }) as Box<dyn FnMut(_)>);

    callback
}

/// Set a hook to log a panic stack trace in JS.
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
