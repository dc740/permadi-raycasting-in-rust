use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use std::time::Instant;
use std::collections::HashMap;
use main_app::loader::{Assets, LocalFileLoader};
const WIDTH: usize = 320;
const HEIGHT: usize = 200;

fn main() {
    let loader = LocalFileLoader{};
    let assets = Assets {
        root: "./".to_string(),
        textures: HashMap::new(),
        resources: None,
        loader: Box::new(loader),
    };
    let mut raycast = main_app::game::GameWindow::new(WIDTH, HEIGHT, assets);
    let mut window = Window::new(
        "Raycast demo",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X4,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    //window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    raycast.init();
    raycast.assets.init();
    raycast.assets.load();
    let mut average_execution_time: u128 = 0;
    let mut fps_counter_reset: u128 = 0;
    let samples = 20;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start = Instant::now();
        raycast.game_step(&window);
        average_execution_time += start.elapsed().as_micros();
        if fps_counter_reset % samples == 0 {
            println!(
                "Frame time {} ms | {} FPS",
                average_execution_time / 1000 / samples,
                1000000 / (average_execution_time / samples)
            );
            average_execution_time = 0;
        }
        // open and close the doors
        raycast.move_doors_demo();
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(raycast.get_buffer_to_print(), WIDTH, HEIGHT)
            .unwrap();
        fps_counter_reset += 1;
    }
}
