use std::path::PathBuf;
use std::{path::Path, process::exit};

use macroquad::miniquad::window::set_window_size;
use macroquad::prelude::*;

use chip8::Chip8;

#[derive(Debug)]
pub struct ChipsteRS {
    pub chip8: Chip8,
    buffer: Image,
    texture: Texture2D,
    rom_path: Option<PathBuf>,
    rom_titles: Option<Vec<String>>,
    rom_cursor: usize,
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

    pub fn new() -> Self {
        request_new_screen_size(1200., 600.);
        set_window_size(1200, 600);

        let chip8 = Chip8::new();
        let buffer =
            Image::gen_image_color(chip8::VIDEO_WIDTH as u16, chip8::VIDEO_HEIGHT as u16, BLACK);
        let texture = Texture2D::from_image(&buffer);
        texture.set_filter(FilterMode::Nearest);

        Self {
            chip8,
            buffer,
            texture,
            rom_titles: None,
            rom_path: None,
            rom_cursor: 0,
        }
    }

    pub fn load<'a>(&'a mut self, rom_path: &'a Path) {
        if !rom_path.exists() {
            exit(1)
        }

        self.rom_path = Some(rom_path.to_path_buf());
        if rom_path.is_dir() {
            self.rom_titles = Some(
                rom_path
                    .read_dir()
                    .unwrap()
                    .map(|entry| entry.unwrap().file_name().into_string().unwrap())
                    .collect(),
            );
        } else {
            self.chip8.load_rom(rom_path).unwrap_or_else(|e| {
                println!("Error loading ROM at path {}", e);
            });
        }
    }

    pub fn draw_menu(&self) {
        if let Some(rom_titles) = &self.rom_titles {
            let mut color: Color;
            for (i, rom_title) in rom_titles[self.rom_cursor..].iter().enumerate() {
                color = if i == 0 { BLUE } else { WHITE };
                draw_text(
                    format!(
                        "{}/{} {}",
                        self.rom_cursor + i + 1,
                        rom_titles.len(),
                        rom_title
                    )
                    .as_str(),
                    screen_width() / 2.0 - 100.0,
                    30. + ((i + 1) as f32 * 30.0),
                    40.0,
                    color,
                );
            }
        }
    }

    pub fn handle_input(&mut self) {
        if let Some(key) = get_last_key_pressed() {
            match key {
                KeyCode::Escape => {
                    if self.chip8.state == chip8::State::Off || self.rom_titles.is_none() {
                        exit(0);
                    }

                    self.chip8.reset();
                    self.chip8.state = chip8::State::Off;
                }
                KeyCode::Space => {
                    self.chip8.state = match self.chip8.state {
                        chip8::State::Running => chip8::State::Paused,
                        chip8::State::Paused => chip8::State::Running,
                        _ => self.chip8.state,
                    }
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
                for _i in 0..10 {
                    self.chip8.step();
                }
            }
            chip8::State::Off => {
                if let Some(rom_titles) = &self.rom_titles {
                    if is_key_pressed(KeyCode::Up) {
                        self.rom_cursor =
                            (self.rom_cursor + rom_titles.len() - 1) % rom_titles.len();
                    } else if is_key_pressed(KeyCode::Down) {
                        self.rom_cursor =
                            (self.rom_cursor + rom_titles.len() + 1) % rom_titles.len();
                    } else if is_key_pressed(KeyCode::Left) {
                        self.rom_cursor = if self.rom_cursor as i32 - 10 <= 0 {
                            rom_titles.len() - 1
                        } else {
                            self.rom_cursor - 10
                        }
                    } else if is_key_pressed(KeyCode::Right) {
                        self.rom_cursor = if self.rom_cursor + 10 >= rom_titles.len() - 1 {
                            0
                        } else {
                            self.rom_cursor + 10
                        }
                    } else if is_key_pressed(KeyCode::Enter) {
                        let path = &self
                            .rom_path
                            .clone()
                            .unwrap()
                            .join(&rom_titles[self.rom_cursor]);
                        self.chip8.load_rom(path).unwrap_or_else(|e| {
                            println!("Error loading ROM at path {}", e);
                        });
                    }
                }
            }
            _ => return,
        }

        let mut color: Color;

        for y in 0..chip8::VIDEO_HEIGHT {
            for x in 0..chip8::VIDEO_WIDTH {
                color = if self.chip8.has_color(x, y) {
                    WHITE
                } else {
                    BLACK
                };
                self.buffer.set_pixel(x as u32, y as u32, color);
            }
        }
    }

    pub async fn draw(&mut self) {
        clear_background(BLACK);

        match self.chip8.state {
            chip8::State::Running => {
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
            chip8::State::Paused => draw_text_ex(
                "Paused",
                screen_width() / 2.0 - 100.0,
                screen_height() / 2.0 - 100.0,
                TextParams {
                    font_size: 100,
                    font_scale: 1.0,
                    color: WHITE,
                    ..Default::default()
                },
            ),
            chip8::State::Finished => self.chip8.reset(),
            chip8::State::Off => self.draw_menu(),
        }

        self.chip8.reset_keys();

        next_frame().await
    }
}
