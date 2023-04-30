extern crate minifb;

use crate::graphics::graphics_driver::*;
use minifb::{Scale, Window, WindowOptions};

pub struct MiniFbDriver {
    window: Window,
    height: u16,
    width: u16,
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
    }

    fn is_closed(&self) -> bool {
        !self.window.is_open()
    }
}
