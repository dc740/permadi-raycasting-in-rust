use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

const WIDTH: usize = 320;
const HEIGHT: usize = 200;

fn main() {
    let mut raycast = main_app::game::GameWindow::new(WIDTH, HEIGHT);
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
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    raycast.init();
    raycast.assets.load_some_textures();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        raycast.game_step(&window);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(raycast.get_buffer_to_print(), WIDTH, HEIGHT)
            .unwrap();
    }
}
