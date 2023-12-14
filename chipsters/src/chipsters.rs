use std::{env, path::Path, process::exit};

use macroquad::{
    color::{BLACK, WHITE},
    miniquad::{window::set_window_size, KeyCode},
    shapes::draw_rectangle,
    text::{draw_text_ex, TextParams},
    window::{clear_background, request_new_screen_size, screen_height, screen_width},
};

pub struct ChipsteRS {
    pub chip8: chip8::Chip8,
}

impl ChipsteRS {
    const CHIP8_KEYS: [KeyCode; 12] = [
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

    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();

        let mut c = chip8::Chip8::new();
        if let Some(rom_path) = Self::get_rom_path(&args) {
            c.load_rom(rom_path).unwrap_or_else(|e| {
                println!("Error loading ROM at path {}", e);
                exit(1);
            });
        } else {
            exit(1);
        }

        set_window_size(1200, 600);
        request_new_screen_size(1200., 600.);
        return Self { chip8: c };
    }

    fn get_rom_path(args: &[String]) -> Option<&Path> {
        match args.len() {
            2 => {
                let rom_path = Path::new(&args[1]);
                if rom_path.exists() && rom_path.is_file() {
                    return Some(rom_path);
                } else {
                    println!("{} is not a valid file", rom_path.display());
                }
            }
            _ => {
                println!("Usage: chipsters <rom_path>");
            }
        }
        None
    }

    pub fn handle_input(&mut self) {
        if let Some(key) = macroquad::input::get_last_key_pressed() {
            match key {
                macroquad::miniquad::KeyCode::Escape => {
                    self.chip8.state = chip8::state::State::Off;
                }
                macroquad::miniquad::KeyCode::Space => {
                    self.chip8.state = match self.chip8.state {
                        chip8::state::State::Running => chip8::state::State::Paused,
                        chip8::state::State::Paused => chip8::state::State::Running,
                        _ => chip8::state::State::Finished,
                    };
                }
                macroquad::miniquad::KeyCode::F1 => {
                    self.chip8.reset();
                }
                _ => {}
            }
        }

        for (i, key) in Self::CHIP8_KEYS.iter().enumerate() {
            if macroquad::input::is_key_down(*key) {
                self.chip8.key_down(i);
            }
        }
    }

    pub fn update(&mut self) {
        match self.chip8.state {
            chip8::State::Finished => self.chip8.reset(),
            chip8::State::Running => {
                for _i in 0..5 {
                    self.chip8.step();
                }
            }
            _ => return,
        }
    }

    pub fn draw(&mut self) {
        clear_background(BLACK);
        if self.chip8.state == chip8::state::State::Paused {
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
            );
            return;
        }

        let pixel_size = ((screen_width() / chip8::VIDEO_HEIGHT as f32) * 0.5).floor();

        for y in 0..chip8::VIDEO_HEIGHT {
            for x in 0..chip8::VIDEO_WIDTH {
                if self.chip8.has_color(x, y) {
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

        self.chip8.reset_keys();
    }

    pub fn should_run(&self) -> bool {
        self.chip8.state != chip8::state::State::Off
    }
}
