use std::fmt;
use crate::bus::*;
use crate::ppu::FifoState::{FetchTileHi, FetchTileNo, PushTile};

const DISPLAY_WIDTH : u8 = 160;
const DISPLAY_HEIGHT: u8 = 144;

const LCDC_TILE_MAP_SELECT          : u8 = 0x08;
const LCDC_TILE_DATA_SELECT         : u8 = 0x10;
const LCDC_WINDOW_TILE_MAP_SELECT   : u8 = 0x30;
const LCDC_WINDOW_ENABLE            : u8 = 0x20;

const TILE_MAP_LO_BASE: Address = 0x9800;
const TILE_MAP_HI_BASE: Address = 0x9C00;

const TILE_DATA_BLOCK_BASE: [Address; 3] = [ 0x8000, 0x8800, 0x9000 ];

#[derive(Debug)]
#[derive(Copy, Clone)]
#[repr(u8)]
enum Mode {
    HBlank,
    VBlank,
    OAM,
    Draw
}

impl Mode {
    fn next(&mut self) {
        *self = Mode::from_u8(((*self as u8) + 1) % 4);
    }

    fn from_u8(value: u8) -> Mode {
        use Mode::*;
        match value % 4 {
            0 => HBlank,
            1 => VBlank,
            2 => OAM,
            3 => Draw,
            _ => panic!("Impossible Branch")
        }
    }
}


#[derive(Clone)]
struct Point {
    x: u8,
    y: u8
}

#[derive(Debug)]
pub struct PPU {
    mode: Mode,

    VRAM: [Byte; 0x1FFF],

    LCDC: Byte,
    STAT: Byte,
    SCY: Byte,
    SCX: Byte,

    LY: Byte,
    LYC: Byte,

    WY: Byte,
    WX: Byte,

    BGP: Byte,
    OBP0: Byte,
    OBP1: Byte,

    DMA: Byte,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            mode: Mode::VBlank,
            VRAM: [0; 0x1FFF],

            LCDC: 0,
            STAT: 0,
            SCY: 0,
            SCX: 0,

            LY: 0,
            LYC: 0,

            WY: 0,
            WX: 0,

            BGP: 0,
            OBP0: 0,
            OBP1: 0,

            DMA: 0,
        }
    }
}

impl fmt::Display for PPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!{f,
               concat! {
               "PPU | MODE {:6?}\n",
               "    | LCDC {:02X}  STAT {:02X}\n",
               "    |  SCY {:02X}   SCX {:02X}\n",
               "    |   LY {:02X}   LYC {:02X}\n",
               "    |   WY {:02X}    WX {:02X}\n"
               },
               self.mode,
               self.LCDC, self.STAT, self.SCY, self.SCX,
               self.LY, self.LYC, self.WY, self.WX
        }
    }
}

impl BusListener for PPU {
    fn addresses(&self) -> Vec<(Address, Address)> {
        vec![
            (0x8000, 0x9FFF), // VRAM
            (0xFE00, 0xFE9F), // OAM Sprite Memory
            (0xFF40, 0xFF4B), // LCD Position / Palettes / DMA Transfer Start Address
            (0xFF4F, 0xFF4F), // VRAM Bank Selector
            (0xFF51, 0xFF55), // HDMA 1-5
            (0xFF68, 0xFF6B), // CGB Palletes
        ]
    }

    fn read(&self, address: Address) -> Byte {
        match address {
            0x8000..0x9FFF => self.VRAM[(address - 0x8000) as usize],
            0xFF40 => self.LCDC,
            0xFF41 => self.STAT,

            0xFF42 => self.SCY,
            0xFF43 => self.SCX,

            0xFF44 => self.LY,
            0xFF45 => self.LYC,

            0xFF4A => self.WY,
            0xFF4B => self.WX,

            0xFF47 => self.BGP,
            0xFF48 => self.OBP0,
            0xFF49 => self.OBP1,

            0xFF46 => self.DMA,
            _ =>  panic!("PPU Address Not Implemented")
        }
    }

    fn write(&mut self, _bus: &mut Bus, address: Address, value: Byte) {
        println!("Writing address: {:04X}", address);
        let ptr = match address {
            0x8000..0x9FFF => &mut self.VRAM[(address - 0x8000) as usize],
            0xFF40 => &mut self.LCDC,
            0xFF41 => &mut self.STAT,

            0xFF42 => &mut self.SCY,
            0xFF43 => &mut self.SCX,

         // 0xFF44 => &mut self.LY,
            0xFF45 => &mut self.LYC,

            0xFF4A => &mut self.WY,
            0xFF4B => &mut self.WX,

            0xFF47 => &mut self.BGP,
            0xFF48 => &mut self.OBP0,
            0xFF49 => &mut self.OBP1,

            0xFF46 => &mut self.DMA,
            _ =>  panic!("PPU Address Not Implemented")
        };
        *ptr = value;
    }
}

impl PPU {

}

#[repr(u8)]
#[derive(Copy, Clone)]
enum FifoState {
    FetchTileNo,
    FetchTileLo,
    FetchTileHi,
    PushTile
}

impl FifoState {
    fn next(&mut self) {
        *self = FifoState::from_u8(((*self as u8) + 1) % 4);
    }

    fn from_u8(value: u8) -> FifoState {
        use FifoState::*;
        match value % 4 {
            0 => FetchTileNo,
            1 => FetchTileLo,
            2 => FetchTileHi,
            3 => PushTile,
            _ => panic!("Impossible Branch")
        }
    }
}

struct PixelFifo {
    state: FifoState,
    pixels: [u8; 16],
    size: usize,
    pos: usize
}

impl PixelFifo {
    pub fn new() -> Self {
        Self {
            state: FifoState::FetchTileNo,
            pixels: [0; 16],
            size: 0,
            pos: 0
        }
    }

    pub fn clear(&mut self) {
        self.state = FifoState::FetchTileLo;
        self.size = 0;
    }

    pub fn top(&self) -> &u8 {
        return &self.pixels[self.pos];
    }

    pub fn pop(&mut self) -> u8 {
        let pixel = *self.top();
        if self.size <= 0 {
            panic!("Buffer Underflow");
        }

        self.size -= 1;
        self.pos += 1 % 16;
        pixel
    }

    pub fn push(&mut self, pixel: u8) {
        if self.size >= 16 {
            return
        }
        self.pixels[(self.pos + self.size) % 16] = pixel
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TileDataBlock {
    BLK0, BLK1
}

struct BackgroundFifo {
    fifo: PixelFifo,
    offset: Point,

    // Column
    column: u8,

    // Discard columns when scrolled
    discard_columns: u8,

    tile_data: (u8, u8),
    tile_data_address: Address,
}

impl BackgroundFifo {
    fn new() -> Self {
        Self {
            fifo: PixelFifo::new(),
            offset: Point { x: 0, y: 0 },

            column: 0,
            discard_columns: 0,

            tile_data: (0, 0),
            tile_data_address: 0,
        }
    }

    pub fn run(&mut self, ppu: &PPU, bus: &Bus) {
        use FifoState::*;
        match &self.fifo.state {
            FetchTileNo => {
                // Whether the window is enabled and the column is within window bounds
                let window_active = ppu.LCDC & LCDC_WINDOW_ENABLE == 1
                                            && self.column <= ppu.WX - 7
                                            && self.column <= ppu.WX + DISPLAY_WIDTH;

                // Select active tile map (either tile map of the window or the background)
                let tile_map_select = ppu.LCDC & if window_active {
                    LCDC_WINDOW_TILE_MAP_SELECT
                }
                else {
                    LCDC_TILE_MAP_SELECT
                };

                // Translate map selection to address
                let tile_map_base = match tile_map_select {
                    0 => TILE_MAP_LO_BASE,
                    1 => TILE_MAP_HI_BASE,
                    _ => panic!("Invalid")
                };

                // Discard first SCX columns
                if self.column == 0 {
                    self.discard_columns = ppu.SCX;
                }

                self.offset = Point {
                    x: (self.column + ppu.SCX) % 0xFF,
                    y: (ppu.LY + ppu.SCY) % 0xFF
                };

                let map_index = (((self.offset.y >> 3) << 5) + (self.offset.x >> 3)) as u16;

                let tile_no = bus.read_byte(tile_map_base + map_index);
                let tile_index = (tile_no << 4) + ((self.offset.y % 8) << 1);

                self.tile_data_address = if ppu.LCDC & LCDC_TILE_DATA_SELECT == 1 {
                    // Tile Block 0: Natural Indexing
                    TILE_DATA_BLOCK_BASE[0] + (tile_index as u16)
                }
                else {
                    // Tile Block 1: Integer Indexing
                    (TILE_DATA_BLOCK_BASE[1] as i16 + tile_index as i16) as u16
                };
            },
            FetchTileLo => {
                self.tile_data.0 = bus.read_byte(self.tile_data_address);
            },
            FetchTileHi => {
                self.tile_data.1 = bus.read_byte(self.tile_data_address + 1);
            },
            PushTile => {
                if self.fifo.size > 8 {
                    return;
                }

                for mask_bit in (0..8).rev() {
                    if self.discard_columns != 0 {
                        self.discard_columns -= 1;
                        continue;
                    }

                    let pixel = (self.tile_data.0 >> (mask_bit - 1) & 2)
                              | (self.tile_data.1 >> mask_bit & 1);

                    self.fifo.push(pixel);
                }

                self.offset.x += 8;
            }
        }

        self.fifo.state.next();
    }
}