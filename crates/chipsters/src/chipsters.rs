use anyhow::{anyhow, Result};
use macroquad::miniquad::window::set_window_size;
use macroquad::prelude::*;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::{path::Path, process::exit};

use chip8::{get_platform, init_default_platform, Chip8};

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

    #[must_use]
    pub fn new() -> Self {
        request_new_screen_size(1200., 600.);
        set_window_size(1200, 600);
        init_default_platform();

        let chip8 = Chip8::default();
        let buffer = Image::gen_image_color(
            get_platform().video_width,
            get_platform().video_height,
            BLACK,
        );
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

    pub fn load<'a>(&'a mut self, rom_path: &'a Path) -> Result<()> {
        if !rom_path.exists() {
            return Err(anyhow!("ROM path does not exist: {}", rom_path.display()));
        }

        self.rom_path = Some(rom_path.to_path_buf());
        if rom_path.is_dir() {
            self.rom_titles = Some(
                rom_path
                    .read_dir()
                    .map_err(|err| anyhow!("Error reading directory: {}", err))?
                    .filter_map(|entry| entry.ok().and_then(|e| e.file_name().into_string().ok()))
                    .collect(),
            );
        } else {
            self.chip8.load_rom(rom_path).map_err(|err| {
                anyhow!("Error loading ROM at path {}: {}", rom_path.display(), err)
            })?;
        }

        Ok(())
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

    pub fn handle_input(&mut self) -> Result<()> {
        if let Some(key) = get_last_key_pressed() {
            match key {
                KeyCode::Escape => {
                    if self.chip8.state == chip8::State::Off || self.rom_titles.is_none() {
                        exit(0);
                    }

                    self.chip8.reset()?;
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
                    self.chip8.reset()?;
                }
                _ => {}
            }
        }

        for (i, key) in Self::CHIP8_KEYS.iter().enumerate() {
            if is_key_down(*key) {
                self.chip8.key_down(i);
            }
        }

        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        match &self.chip8.state {
            chip8::State::Finished => self
                .chip8
                .reset()
                .map_err(|err| anyhow!("Failed to reset: {err}"))?,
            chip8::State::Running => {
                let frame_duration = Duration::from_secs_f64(1.0 / 60.0); // 60 Hz display refresh
                let start = std::time::Instant::now();

                // Run CPU cycles for this frame
                for _i in 0..get_platform().tick_rate {
                    self.chip8.step()?;
                }

                // Sleep for remainder of frame if any
                let elapsed = start.elapsed();
                if elapsed < frame_duration {
                    thread::sleep(frame_duration - elapsed);
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
                            panic!(
                                "Error loading ROM at path {}:  {}",
                                path.to_string_lossy(),
                                e
                            );
                        });
                    }
                }
            }
            _ => {}
        }

        let mut color: Color;

        for y in 0..get_platform().video_height {
            for x in 0..get_platform().video_width {
                color = if self.chip8.has_color(x, y) {
                    WHITE
                } else {
                    BLACK
                };
                self.buffer.set_pixel(u32::from(x), u32::from(y), color);
            }
        }

        Ok(())
    }

    pub async fn draw(&mut self) -> Result<()> {
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
            chip8::State::Finished => self
                .chip8
                .reset()
                .map_err(|err| anyhow!("Failed to reset: {err}"))?,
            chip8::State::Off => self.draw_menu(),
        }

        self.chip8.reset_keys();

        next_frame().await;

        Ok(())
    }
}

impl Default for ChipsteRS {
    fn default() -> Self {
        Self::new()
    }
}
