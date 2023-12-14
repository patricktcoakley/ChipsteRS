use std::env;

use macroquad::prelude::*;

#[macroquad::main("ChipsteRS")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mut rom_path = String::new();
    match args.len() {
        1 => println!("No ROM path provided"),
        2 => { rom_path = args[1].clone(); }
        _ => println!("Too many arguments provided"),
    }

    let mut c = chip8::Chip8::new();
    c.load_rom(rom_path.as_str());

    let chip8_keys = vec![
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Q,
        KeyCode::W,
        KeyCode::E,
        KeyCode::R,
        KeyCode::A,
        KeyCode::S,
        KeyCode::D,
        KeyCode::F,
    ];

    request_new_screen_size(1200., 600.);

    loop {
        clear_background(BLACK);

        match get_last_key_pressed() {
            None => {}
            Some(key) => match key {
                KeyCode::Escape => break,
                KeyCode::Space => {
                    c.state = match c.state {
                        chip8::state::State::Running => chip8::state::State::Paused,
                        chip8::state::State::Paused => chip8::state::State::Running,
                        _ => chip8::state::State::Stopped,
                    };
                }
                _ => {}
            }
        }


        for (i, key) in chip8_keys.iter().enumerate() {
            c.set_key(i, is_key_down(*key));
        }

        match c.state {
            chip8::state::State::Paused => {
                draw_text_ex(
                    "Paused",
                    screen_width() / 2.0 - 100.0,
                    screen_height() / 2.0 - 100.0,
                    TextParams {
                        font_size: 100,
                        font_scale: 1.0,
                        color: WHITE,
                        ..Default::default()
                    },
                )
            }
            _ => {
                for _i in 0..5 {
                    c.step();
                }
            }
        }


        let pixel_size = ((screen_width() / chip8::VIDEO_HEIGHT as f32) * 0.5).floor();

        for y in 0..chip8::VIDEO_HEIGHT {
            for x in 0..chip8::VIDEO_WIDTH {
                if c.has_color(x, y) {
                    draw_rectangle(
                        x as f32 * pixel_size,
                        y as f32 * pixel_size,
                        pixel_size,
                        pixel_size,
                        WHITE,
                    );
                }
            }
        }

        for i in 0..chip8_keys.len() {
            c.set_key(i, false);
        }

        next_frame().await
    }

    std::process::exit(0);
}
