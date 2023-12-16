use std::{path::Path, process::exit};

use macroquad::miniquad::window::set_window_size;
use macroquad::prelude::*;

use chip8::Chip8;

pub struct ChipsteRS {
    pub chip8: Chip8,
    buffer: Image,
    texture: Texture2D,
}

impl ChipsteRS {
    const CHIP8_KEYS: [KeyCode; 16] = [
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
        KeyCode::Z,
        KeyCode::X,
        KeyCode::C,
        KeyCode::V,
    ];

    pub fn new(rom_path: &Path) -> Self {
        let mut c = Chip8::new();

        if let Some(rom) = Self::validate_rom_path(rom_path) {
            c.load_rom(rom).unwrap_or_else(|e| {
                println!("Error loading ROM at path {}", e);
                exit(1);
            });
        } else {
            exit(1);
        }

        set_window_size(1200, 600);
        request_new_screen_size(1200., 600.);
        let buffer =
            Image::gen_image_color(chip8::VIDEO_WIDTH as u16, chip8::VIDEO_HEIGHT as u16, BLACK);
        let texture = Texture2D::from_image(&buffer);
        texture.set_filter(FilterMode::Nearest);

        Self {
            chip8: c,
            buffer,
            texture,
        }
    }

    fn validate_rom_path(rom_path: &Path) -> Option<&Path> {
        if rom_path.exists() && rom_path.is_file() {
            return Some(rom_path);
        } else {
            println!("{} is not a valid file", rom_path.display());
            None
        }
    }

    pub fn handle_input(&mut self) {
        if let Some(key) = get_last_key_pressed() {
            match key {
                KeyCode::Escape => {
                    self.chip8.state = chip8::State::Off;
                }
                KeyCode::Space => {
                    self.chip8.state = match self.chip8.state {
                        chip8::State::Running => chip8::State::Paused,
                        chip8::State::Paused => chip8::State::Running,
                        _ => chip8::State::Finished,
                    };
                }
                KeyCode::F1 => {
                    self.chip8.reset();
                }
                _ => {}
            }
        }

        for (i, key) in Self::CHIP8_KEYS.iter().enumerate() {
            if is_key_down(*key) {
                self.chip8.key_down(i);
            }
        }
    }

    pub fn update(&mut self) {
        match &self.chip8.state {
            chip8::State::Finished => self.chip8.reset(),
            chip8::State::Running => {
                for _i in 0..5 {
                    self.chip8.step();
                }
            }
            _ => return,
        }

        for y in 0..chip8::VIDEO_HEIGHT {
            for x in 0..chip8::VIDEO_WIDTH {
                if self.chip8.has_color(x, y) {
                    self.buffer.set_pixel(x as u32, y as u32, WHITE);
                } else {
                    self.buffer.set_pixel(x as u32, y as u32, BLACK);
                }
            }
        }
    }

    pub async fn draw(&mut self) {
        clear_background(BLACK);

        if self.chip8.state == chip8::State::Paused {
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
        } else if self.chip8.state == chip8::State::Running {
            self.texture.update(&self.buffer);
            draw_texture_ex(
                &self.texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(screen_width(), screen_height())),
                    ..Default::default()
                },
            );
        }

        self.chip8.reset_keys();

        next_frame().await
    }

    pub fn should_run(&self) -> bool {
        self.chip8.state != chip8::State::Off
    }
}
