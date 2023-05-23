use crate::{Address, Byte, Registers};
use crate::ppu::fifo::{FifoState, FifoState::*, PixelFifo};
use crate::ppu::{LCDC_SPRITE_ENABLE, LCDC_SPRITE_SIZE, LCDC_TILE_DATA_SELECT, LCDC_TILE_MAP_SELECT, LCDC_WINDOW_TILE_MAP_SELECT, MAX_SPRITES_PER_LINE, Point, TILE_DATA_BLOCK_BASE, TILE_MAP_HI_BASE, TILE_MAP_LO_BASE};

#[derive(Debug, Copy, Clone)]
struct OamEntry {
    x: u8,
    y: u8,
    tile: u8,
    attributes: u8,
}

impl OamEntry {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            tile: 0,
            attributes: 0,
        }
    }
}

// TODO: Move this to its own file.
#[derive(Debug)]
pub struct SpriteFifo {
    fifo: PixelFifo,
    entry_fifo: PixelFifo,

    state: FifoState,

    // OAM Entries
    oam_table: [OamEntry; MAX_SPRITES_PER_LINE],
    oam_table_size: usize,
    oam_entry_index: usize,
    oam_scan_index: usize,

    // Column
    column: u8,

    // Discard columns when scrolled
    discard_columns: u8,

    tile_data: (u8, u8),
    tile_data_address: Address,
}

impl SpriteFifo {
    pub(crate) fn new() -> Self {
        Self {
            fifo: PixelFifo::new(),
            entry_fifo: PixelFifo::new(),

            state: FetchTileNo,

            oam_table: [OamEntry::new(); MAX_SPRITES_PER_LINE],
            oam_table_size: 0,
            oam_entry_index: 0, // OAM entry to read from while drawing
            oam_scan_index: 0, // OAM entry to read from memory

            column: 0,
            discard_columns: 0,

            tile_data: (0, 0),
            tile_data_address: 0,
        }
    }

    pub(crate) fn scan_next_oam_table_entry(&mut self, oam: &[Byte], registers: &Registers) {
        if self.oam_scan_index >= 40 {
            return;
        }

        let index = self.oam_scan_index << 2;
        self.oam_scan_index += 1;

        let oam_entry = OamEntry {
            y: oam[index],
            x: oam[index + 1],
            tile: oam[index + 2],
            attributes: oam[index + 3],
        };

        // Sprite is not visible y.
        if oam_entry.y == 0 || oam_entry.x >= 160 {
            return;
        }

        // Sprite is not visible x.
        if oam_entry.x == 0 || oam_entry.y >= 168 {
            return;
        }

        // Is entry on this line?
        if registers.LY < oam_entry.y - 16 {
            return;
        }

        let sprite_height = if registers.LCDC & LCDC_SPRITE_SIZE != 0 { 16 } else { 8 };

        if registers.LY >= (oam_entry.y - 16) + sprite_height {
            return;
        }

        println!("{:02X} {:02X} {:02X}", oam_entry.y, oam_entry.x, oam_entry.tile);

        // Sort entries by x value.
        let mut tmp_entry = oam_entry;
        for i in 0..self.oam_table_size {
            let oam_entry = self.oam_table[i];
            if oam_entry.x < tmp_entry.x {
                continue;
            }
            self.oam_table[i] = tmp_entry;
            tmp_entry = oam_entry;
        }
        self.oam_table[self.oam_table_size] = tmp_entry;
        self.oam_table_size += 1;
    }

    pub(crate) fn step(&mut self, vram: &[Byte], registers: Registers) {

        if registers.LCDC & LCDC_SPRITE_ENABLE == 0 {
            return;
        }

        use FifoState::*;
        match &self.state {
            FetchTileNo => {
                if self.oam_table_size <= self.oam_entry_index {
                    return;
                }

                let oam_entry = self.oam_table[self.oam_entry_index];

                for pixel_offset in 0..8 {
                    if self.column < oam_entry.x {
                        self.column += 1;
                        break;
                    }

                    if pixel_offset == 7 {
                        return;
                    }
                }

                if oam_entry.x < 8 {
                    self.discard_columns = 8 - oam_entry.x;
                }

                let tile_no: u8 = oam_entry.tile;

                // Each tile takes up 16 bytes, so tile_no is multiplied by 16.
                // Each pixel takes up 2 bits, so the y offset must be multiplied by 2.
                // Sprites can be 16 pixels tall and are offset by this height.
                let entry_y = oam_entry.y as i16 - 16;
                let height = (registers.LY as i16) - entry_y;
                self.tile_data_address = ((tile_no as u16) << 4) + ((height as u16) << 1);
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

                    // TODO add priority and palette information.
                    let pixel = (self.tile_data.1.overflowing_shr(mask_bit).0 & 1) << 1
                        | (self.tile_data.0.overflowing_shr(mask_bit).0 & 1);

                    self.entry_fifo.push(self.oam_table[self.oam_entry_index].x + mask_bit as u8);
                    self.fifo.push(pixel);
                }

                self.oam_entry_index += 1;
                self.column += 8 - self.discard_columns;
            }
        }

        self.state.next();
    }

    pub fn pop(&mut self, x: u8) -> Option<u8> {
        let ex = *self.entry_fifo.top();
        if x < ex {
            return None;
        }
        self.entry_fifo.pop();
        self.fifo.pop()
    }

    pub fn reset(&mut self) {
        self.fifo.clear();
        self.entry_fifo.clear();
        self.state = FetchTileNo;
        self.column = 0;
        self.oam_entry_index = 0;
        self.oam_scan_index = 0;
        self.oam_table_size = 0;
    }
}
