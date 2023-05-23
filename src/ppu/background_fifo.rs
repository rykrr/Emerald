use crate::{Address, Byte, Registers};
use crate::ppu::fifo::{FifoState, FifoState::*, PixelFifo};
use crate::ppu::{LCDC_TILE_DATA_SELECT, LCDC_TILE_MAP_SELECT, LCDC_WINDOW_TILE_MAP_SELECT, Point, TILE_DATA_BLOCK_BASE, TILE_MAP_HI_BASE, TILE_MAP_LO_BASE};

#[derive(Clone, Copy, PartialEq)]
enum TileDataBlock {
    BLK0,
    BLK1,
}

// TODO: Move this to its own file.
#[derive(Debug)]
pub struct BackgroundFifo {
    fifo: PixelFifo,
    pub(crate) state: FifoState,
    offset: Point,

    // Column
    column: u8,

    // Discard columns when scrolled
    discard_columns: u8,

    tile_data: (u8, u8),
    tile_data_address: Address,
}

impl BackgroundFifo {
    pub(crate) fn new() -> Self {
        Self {
            fifo: PixelFifo::new(),
            state: FetchTileNo,
            offset: Point { x: 0, y: 0 },

            column: 0,
            discard_columns: 0,

            tile_data: (0, 0),
            tile_data_address: 0,
        }
    }

    pub fn step(&mut self, vram: &[Byte], registers: Registers) {
        use FifoState::*;
        match &self.state {
            FetchTileNo => {
                // Check whether the window is enabled and the column is within window bounds
                // TODO: Implement window.
                /*
                let window_active = registers.LCDC & LCDC_WINDOW_ENABLE != 1
                    && (self.column as i8) <= (registers.WX as i8) - 7
                    && self.column <= registers.WX + DISPLAY_WIDTH;
                 */
                let window_active = false;

                // Select active tile map (either tile map of the window or the background)
                let tile_map_select = if window_active {
                    LCDC_WINDOW_TILE_MAP_SELECT
                } else {
                    LCDC_TILE_MAP_SELECT
                };

                // Translate map selection to address
                let tile_map_base = if registers.LCDC & tile_map_select == 0 {
                    TILE_MAP_LO_BASE
                } else {
                    TILE_MAP_HI_BASE
                };

                // Discard first SCX columns
                if self.column != 0 {
                    self.discard_columns = registers.SCX;
                }

                self.offset = Point {
                    x: (self.column as u16 + registers.SCX as u16) % 0xFF,
                    y: (registers.LY as u16 + registers.SCY as u16) % 0xFF,
                };

                //let map_index = ((self.offset.y >> 3) << 5) + (self.offset.x >> 3);
                let map_index: u16 = ((self.offset.y & 0xF8) << 2) + (self.offset.x >> 3);

                // Lookup the tile number to load from the appropriate tile map.
                let tile_no: u8 = vram[(tile_map_base + map_index) as usize];

                // Each tile takes up 16 bytes, so tile_no is multiplied by 16.
                // Each pixel takes up 2 bits, so the y offset must be multiplied by 2.
                let tile_index: u16 = ((tile_no as u16) << 4) + ((self.offset.y % 8) << 1);

                self.tile_data_address = if registers.LCDC & LCDC_TILE_DATA_SELECT == 0 {
                    // Tile Block 1: Integer Indexing
                    // Treat tile_no as a signed int.
                    if tile_no & 0x80 == 0 {
                        TILE_DATA_BLOCK_BASE[1] + tile_index
                    }
                    else {
                        // Remove the sign bit before subtracting the index from the base address.
                        TILE_DATA_BLOCK_BASE[1] - (tile_index ^ 0x0800)
                    }
                } else {
                    // Tile Block 0: Natural Indexing
                    TILE_DATA_BLOCK_BASE[0] + tile_index as u16
                };
            }
            FetchTileLo => {
                self.tile_data.0 = vram[self.tile_data_address as usize];
            }
            FetchTileHi => {
                self.tile_data.1 = vram[(self.tile_data_address + 1) as usize];
            }
            PushTile => {
                if self.fifo.size > 8 {
                    return;
                }

                for mask_bit in (0..8).rev() {
                    if self.discard_columns != 0 {
                        self.discard_columns -= 1;
                        continue;
                    }

                    let pixel = (self.tile_data.1.overflowing_shr(mask_bit).0 & 1) << 1
                        | (self.tile_data.0.overflowing_shr(mask_bit).0 & 1);

                    self.fifo.push(pixel);
                }

                self.column += 8;
            }
        }

        self.state.next();
    }

    pub fn pop(&mut self) -> Option<u8> {
        self.fifo.pop()
    }

    pub fn reset(&mut self) {
        self.fifo.clear();
        self.state = FetchTileNo;
        self.column = 0;
    }
}
