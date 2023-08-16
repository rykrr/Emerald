extern crate minifb;

use crate::graphics::graphics_driver::*;
use minifb::{Key, Scale, Window, WindowOptions};
use crate::{JoypadButtons, JoypadDriver};
use crate::joypad::{Button, Direction};

pub struct MiniFbDriver {
    window: Window,
    height: u16,
    width: u16,
    disable_pause: bool,
}

impl MiniFbDriver {
    pub fn new(width: u16, height: u16, scale: Scale) -> Self {
        let mut window = Window::new(
            "Emerald",
            width as usize,
            height as usize,
            WindowOptions {
                scale,
                ..WindowOptions::default()
            },
        )
        .expect("Fatal: Failed to initialize window.");

        Self {
            window,
            height,
            width,
            disable_pause: false
        }
    }
}

impl GraphicsDriver for MiniFbDriver {
    fn draw(&mut self, y: u16, x: u16, colour: u32) {
        todo!()
    }

    fn render(&mut self, pixel_buffer: &[u32]) {
        let buffer = unsafe { pixel_buffer.align_to::<u32>().1 };

        self.window
            .update_with_buffer(buffer, self.width as usize, self.height as usize)
            .unwrap();

        self.disable_pause = false;
    }

    fn is_closed(&self) -> bool {
        !self.window.is_open()
    }
}

impl JoypadDriver for MiniFbDriver {
    fn get_buttons(&mut self) -> Vec<JoypadButtons> {
        self.window.get_keys().iter().filter_map(|&key| {
            match key {
                Key::W => Some(JoypadButtons::Direction(Direction::Up)),
                Key::A => Some(JoypadButtons::Direction(Direction::Left)),
                Key::S => Some(JoypadButtons::Direction(Direction::Down)),
                Key::D => Some(JoypadButtons::Direction(Direction::Right)),
                Key::Q => Some(JoypadButtons::Button(Button::A)),
                Key::E => Some(JoypadButtons::Button(Button::B)),
                Key::Key1 => Some(JoypadButtons::Button(Button::Start)),
                Key::Key2 => Some(JoypadButtons::Button(Button::Select)),
                Key::F2 => {
                    if self.disable_pause {
                        return None;
                    }
                    self.disable_pause = true;
                    Some(JoypadButtons::Pause)
                },
                _ => None
            }
        }
        ).collect::<Vec<JoypadButtons>>()
    }
}
