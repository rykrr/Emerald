extern crate sdl2;

use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::{pixels::PixelFormatEnum, rect::Rect, Sdl, VideoSubsystem};
use std::rc::Weak;
use std::{cell::RefCell, rc::Rc};

use crate::graphics::graphics_driver::*;

pub struct SdlContext {
    context: Sdl,
    video: VideoSubsystem,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    height: u32,
    width: u32,
    scale: u32,
}

pub struct SdlDriver<'a> {
    texture: Texture<'a>,
}

impl SdlContext {
    pub fn new(width: u16, height: u16, scale: u8) -> SdlContext {
        let context = sdl2::init().unwrap();
        let video = context.video().unwrap();

        let scaled_height: u32 = height as u32 * scale as u32;
        let scaled_width: u32 = width as u32 * scale as u32;

        let mut window = video.window("Hello World", scaled_width, scaled_height);

        let mut canvas = window //video.window("Hello World", scaled_height, scaled_width)
            .position_centered()
            .build()
            .expect("Fatal: Failed to initialize window.")
            .into_canvas()
            .accelerated()
            .build()
            .expect("Fatal: Failed to initialize canvas.");

        let texture_creator = canvas.texture_creator();

        SdlContext {
            context,
            video,
            canvas,
            texture_creator,
            height: height as u32,
            width: width as u32,
            scale: scale as u32,
        }
    }
}

impl<'a> SdlDriver<'a> {
    pub fn new(context: &'a mut SdlContext) -> Self {
        let texture = context
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::ABGR1555, context.width, context.height)
            .expect("Fatal: Failed to initialize texture.");

        SdlDriver { texture }
    }
}

impl<'a> GraphicsDriver for SdlDriver<'a> {
    fn draw(&mut self, y: u16, x: u16, colour: u32) {
        todo!()
    }

    fn copy_pixel_buffer(&mut self, pixel_buffer: &[u32], target: (Point, Point)) {
        let (x0, y0) = (target.0.x as i32, target.0.y as i32);
        let (x1, y1) = (target.1.x as i32, target.1.y as i32);
        let rect = Rect::new(x0, y0, (x1 - x0) as u32, (y1 - y0) as u32);

        self.texture
            .with_lock(rect, |pixels, pitch| {
                pixels.copy_from_slice(unsafe { pixel_buffer.align_to::<u8>().1 })
            })
            .expect("Fatal: Failed to lock texture.");
    }

    fn render(&mut self) -> u8 {
        todo!()
    }
}
