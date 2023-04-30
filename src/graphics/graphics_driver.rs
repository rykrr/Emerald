use std::cell::RefCell;

pub struct Point {
    pub x: u16,
    pub y: u16,
}

pub fn encode_rgb(r: u8, g: u8, b: u8) -> u32 {
    return (r as u32) << 16 | (g as u32) << 8 | (b as u32);
}

pub trait GraphicsDriver {
    fn draw(&mut self, y: u16, x: u16, colour: u32);
    fn render(&mut self, pixel_buffer: &[u32]);
    fn is_closed(&self) -> bool;
}

pub type GraphicsDriverCell = Box<dyn GraphicsDriver>;
