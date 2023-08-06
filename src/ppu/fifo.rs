#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum FifoState {
    FetchTileNo,
    FetchTileLo,
    FetchTileHi,
    PushTile,
}

impl FifoState {
    pub fn next(&mut self) {
        *self = FifoState::from_u8(((*self as u8) + 1) % 4);
    }

    pub fn from_u8(value: u8) -> FifoState {
        use FifoState::*;
        match value % 4 {
            0 => FetchTileNo,
            1 => FetchTileLo,
            2 => FetchTileHi,
            3 => PushTile,
            _ => panic!("Impossible Branch"),
        }
    }
}

#[derive(Debug)]
pub struct PixelFifo {
    pixels: [u8; 16],
    pub(crate) size: usize,
    pos: usize,
}

// TODO: Move this to its own file.
impl PixelFifo {
    pub fn new() -> Self {
        Self {
            pixels: [0; 16],
            size: 0,
            pos: 0,
        }
    }

    pub fn clear(&mut self) {
        self.size = 0;
        self.pos = 0;
    }

    pub fn top(&self) -> &u8 {
        return &self.pixels[self.pos];
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.size <= 0 {
            return None;
        }

        let pixel = *self.top();
        self.size -= 1;
        self.pos = (self.pos + 1) % 16;
        Some(pixel)
    }

    pub fn push(&mut self, pixel: u8) {
        if self.size >= 16 {
            return;
        }
        self.pixels[(self.pos + self.size) % 16] = pixel;
        self.size += 1;
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}
