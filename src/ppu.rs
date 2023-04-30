use crate::bus::*;
use crate::cpu::{interrupt, InterruptType};
use crate::ppu::FifoState::{FetchTileHi, FetchTileNo, PushTile};
use crate::clock::ClockListener;
use std::fmt;

use std::cell::RefCell;
use std::rc::Rc;

use crate::graphics_driver::GraphicsDriver;
use crate::ppu::Mode::OAM;

pub const DISPLAY_WIDTH: u8 = 160;
pub const DISPLAY_HEIGHT: u8 = 144;
const PITCH: usize = DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize;
const VIRTUAL_DISPLAY_HEIGHT: u8 = 154;

const LCDC_DISPLAY_ENABLE: u8 = 1 << 7;
const LCDC_WINDOW_TILE_MAP_SELECT: u8 = 1 << 6;
const LCDC_WINDOW_ENABLE: u8 = 1 << 5;
const LCDC_TILE_DATA_SELECT: u8 = 1 << 4;
const LCDC_TILE_MAP_SELECT: u8 = 1 << 3;

const LCDC_SPRITE_SIZE: u8 = 1 << 2; // 1: Double height
const LCDC_SPRITE_ENABLE: u8 = 1 << 1;
// const LCDC_BG_VS_WINDOW_PRIORITY: u8 = 1 << 1;
const MAX_SPRITES_PER_LINE: usize = 10;

const STAT_LYC_INTERRUPT: u8 = 1 << 6;
const STAT_OAM_INTERRUPT: u8 = 1 << 5;
const STAT_VBLANK_INTERRUPT: u8 = 1 << 4;
const STAT_HBLANK_INTERRUPT: u8 = 1 << 3;
const STAT_LYC_FLAG: u8 = 1 << 2;
const STAT_MODE_MASK: u8 = 0x03;

const VRAM_BASE_ADDRESS: Address = 0x8000;
const TILE_MAP_LO_BASE: Address = 0x1800; // VRAM Relative Address; Bus Address 0x9800;
const TILE_MAP_HI_BASE: Address = 0x1C00; // VRAM Relative Address; Bus Address 0x9C00;

const TILE_DATA_BLOCK_BASE: [Address; 3] = [0x0000, 0x0800, 0x1000]; // VRAM Relative Addresses

// Timings from "The Ultimate Game Boy Talk"
const OAM_CYCLES: u16 = 20;
const DRAW_CYCLES: u16 = 43;
const HBLANK_CYCLES: u16 = 51;
const VBLANK_LINE_CYCLES: u16 = 114;
const SCREEN_CYCLES: u16 = VBLANK_LINE_CYCLES * VIRTUAL_DISPLAY_HEIGHT as u16;


#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum Mode {
    HBlank = 0,
    VBlank,
    OAM,
    Draw,
}

#[derive(Debug, Copy, Clone)]
struct Point {
    x: u16,
    y: u16,
}

#[derive(Debug, Copy, Clone)]
pub struct Registers {
    LCDC: Byte,
    STAT: Byte,
    SCY: Byte,
    SCX: Byte,

    LX: Byte,
    // Fake register specifying X position of renderer
    LY: Byte,
    LYC: Byte,

    WY: Byte,
    WX: Byte,

    BGP: Byte,
    OBP0: Byte,
    OBP1: Byte,

    dma_active: bool,
    dma_address: Byte,
    dma_counter: Byte,
}

#[derive(Debug)]
pub struct PPU {
    on: bool,
    mode: Mode,

    clock: u16,

    pixel_buffer: [u32; PITCH],
    palette_buffer: [u32; 4],
    render_flag: bool,

    VRAM: [Byte; 0x2000],
    OAM: [Byte; 0x100],

    registers: Registers,

    bgfifo: BackgroundFifo,
    spfifo: SpriteFifo,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            on: false,
            mode: Mode::VBlank,

            clock: 0,

            pixel_buffer: [0x00; PITCH],
            palette_buffer: [0xFFFFFF, 0xC0C0C0, 0x404040, 0x000000],
            render_flag: true,

            VRAM: [0; 0x2000],
            OAM: [0; 0x100],

            registers: Registers {
                LCDC: 0,
                STAT: Mode::VBlank as u8,
                SCY: 0,
                SCX: 0,

                LX: 0,
                LY: 0,
                LYC: 0,

                WY: 0,
                WX: 0,

                BGP: 0,
                OBP0: 0,
                OBP1: 0,

                dma_active: false,
                dma_counter: 0,
                dma_address: 0,
            },

            bgfifo: BackgroundFifo::new(),
            spfifo: SpriteFifo::new(),
        }
    }

    pub fn update<'a>(&mut self, driver: &mut (dyn GraphicsDriver + 'a)) {
        let start = crate::graphics_driver::Point { x: 0, y: 0 };
        let end = crate::graphics_driver::Point {
            x: DISPLAY_WIDTH as u16,
            y: DISPLAY_HEIGHT as u16,
        };


        if self.render_flag {
            if !self.on {
                let screen: [u32; PITCH] = [0; PITCH];
                driver.render(&screen);
            }
            else {
                driver.render(&self.pixel_buffer);
            }

            self.render_flag = false;
        }
    }

    fn set_mode(&mut self, bus: &mut Bus, mode: Mode) {
        self.mode = mode;

        // Clear previous mode flag
        self.registers.STAT &= 0xFF ^ STAT_MODE_MASK;

        // Set current mode flag
        self.registers.STAT |= mode as u8;

        const INTERRUPT_SOURCE_FLAGS: [u8; 3] = [
            STAT_HBLANK_INTERRUPT,
            STAT_VBLANK_INTERRUPT,
            STAT_OAM_INTERRUPT
        ];

        match mode {
            // Draw does not have an associated interrupt.
            Mode::Draw => return,
            Mode::VBlank => interrupt(bus, InterruptType::VBlank),
            _ => {},
        }

        if self.registers.STAT & INTERRUPT_SOURCE_FLAGS[mode as usize] != 0 {
            interrupt(bus, InterruptType::LCDStat);
        }

    }

    pub fn reset(&mut self, bus: &mut Bus) {
        self.set_mode(bus, Mode::OAM);
        self.registers.LY = 0;
        self.clock = 0;
    }
}

impl fmt::Display for PPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! {f,
               concat! {
               "PPU | MODE {:6?}\n",
               "    | LCDC {:02X}  STAT {:02X}\n",
               "    |  SCY {:02X}   SCX {:02X}\n",
               "    |   LY {:02X}   LYC {:02X}\n",
               "    |   LY {:02X}    LX {:02X}\n",
               "    |   WY {:02X}    WX {:02X}\n\n",
               "BGF | MODE {:?}\n"
               },
               self.mode,
               self.registers.LCDC, self.registers.STAT, self.registers.SCY, self.registers.SCX,
               self.registers.LY, self.registers.LYC, self.registers.LY, self.registers.LX,
               self.registers.WY, self.registers.WX,
               self.bgfifo.state,
        }
    }
}

impl BusListener for PPU {
    fn bus_attach(&mut self) -> Vec<Attach> {
        vec![
            Attach::BlockRange(0x80, 0x9F), // VRAM
            Attach::Block(0xFE), // OAM Sprite Memory (Note that OAM is only up to 0xFE9F)
            Attach::RegisterRange(0x40, 0x4B), // LCD Position / Palettes / DMA Transfer Start Address
            // Attach::Register(0x4F), // VRAM Bank Selector
            // Attach::RegisterRange(0x51, 0x55), // HDMA 1-5
            // Attach::RegisterRange(0x68, 0x6B), // CGB Palletes
        ]
    }

    fn bus_read(&self, address: Address) -> Byte {
        // TODO: Prevent access during OAM or Draw.
        match address {
            0x8000..=0x9FFF => self.VRAM[(address - 0x8000) as usize],
            0xFE00..=0xFE9F => self.OAM[(address - 0xFE00) as usize],

            0xFEA0..=0xFEFF => 0, // This range is unusable

            0xFF40 => self.registers.LCDC,
            0xFF41 => self.registers.STAT,

            0xFF42 => self.registers.SCY,
            0xFF43 => self.registers.SCX,

            //0xFF44 => 0x90, //DEBUG//
            0xFF44 => self.registers.LY,
            0xFF45 => self.registers.LYC,

            0xFF46 => self.registers.dma_address,
            0xFF47 => self.registers.BGP,
            0xFF48 => self.registers.OBP0,
            0xFF49 => self.registers.OBP1,

            0xFF4A => self.registers.WY,
            0xFF4B => self.registers.WX,

            // 0xFF4F | 0xFF51..=0xFF55 | 0xFF68..=0xFF6B => 0x00, // TODO

            _ => panic!("PPU Address ({:04X}) Not Implemented", address),
        }
    }

    fn bus_write(&mut self, _bus: &mut Bus, address: Address, value: Byte) {
        match address {
            // 0xFF4F | 0xFF51..=0xFF55 | 0xFF68..=0xFF6B => return, // TODO
            0xFEA0..=0xFEFF => return, // This range is unusable
            0xFF41 => {
                // Lower 3 bits of STAT are read-only mode indicators.
                let stat = self.registers.STAT;
                self.registers.STAT = (value & 0xF8) | (stat & 0x07);
                return;
            }
            _ => {},
        }

        let ptr = match address {
            0x8000..=0x9FFF => &mut self.VRAM[(address - 0x8000) as usize],
            0xFE00..=0xFE9F => &mut self.OAM[(address - 0xFE00) as usize],

            0xFF40 => &mut self.registers.LCDC,
            // 0xFF41 HANDLED ABOVE //

            0xFF42 => &mut self.registers.SCY,
            0xFF43 => &mut self.registers.SCX,

            // 0xFF44 (LY) is READ ONLY //
            0xFF45 => &mut self.registers.LYC,

            0xFF47 => &mut self.registers.BGP,
            0xFF48 => &mut self.registers.OBP0,
            0xFF49 => &mut self.registers.OBP1,

            0xFF4A => &mut self.registers.WY,
            0xFF4B => &mut self.registers.WX,

            // Writing to the DMA Transfer Register initializes transfer
            0xFF46 => {
                self.registers.dma_active = true;
                self.registers.dma_counter = 0;
                assert!(value <= 0xF1);
                &mut self.registers.dma_address
            },

            _ => panic!("PPU Address ({:04X}) Not Implemented", address),
        };
        *ptr = value;
    }
}

impl ClockListener for PPU {
    fn callback(&mut self, bus: &mut Bus, cycles: u8) {
        if self.registers.LCDC & LCDC_DISPLAY_ENABLE == 0 {
            self.on = false;
            self.clock += cycles as u16;

            if SCREEN_CYCLES < self.clock {
                self.clock -= SCREEN_CYCLES;
                self.render_flag = true;
            }

            return;
        }
        else if !self.on {
            self.reset(bus);
            self.on = true;
        }

        // DMA Transfer Loop
        for _ in 0..cycles {
            // DMA may terminate in the middle of this loop
            if !self.registers.dma_active {
                break;
            }

            let dma_counter = self.registers.dma_counter as u16;
            let data = bus.read_byte(((self.registers.dma_address as Address) << 8) | dma_counter);
            self.OAM[dma_counter as usize] = data;

            self.registers.dma_counter += 1;
            self.registers.dma_active = self.registers.dma_counter < DISPLAY_WIDTH;
        }

        self.clock += cycles as u16;

        use Mode::*;
        match self.mode {
            OAM => {
                if self.clock < OAM_CYCLES {
                    self.spfifo.scan_next_oam_table_entry(&self.OAM, &self.registers);
                    return;
                }

                self.clock -= OAM_CYCLES;
                self.set_mode(bus, Draw);
            }
            Draw => {
                // Render cycle: Push pixels onto the screen
                for _ in 0..(cycles << 1) {
                    self.bgfifo.step(&self.VRAM, self.registers);
                    self.spfifo.step(&self.VRAM, self.registers);

                    for _ in 0..2 {
                        // TODO: Window Handling

                        if DISPLAY_WIDTH <= self.registers.LX {
                            break;
                        }

                        let mut pixel_index = 0u8;

                        match self.bgfifo.pop() {
                            None => break,
                            Some(index) => pixel_index = index,
                        }

                        // TODO: Sprite priority.
                        match self.spfifo.pop(self.registers.LX) {
                            None => {},
                            Some(index) => pixel_index = index,
                        }

                        let pixel = self.palette_buffer[pixel_index as usize];
                        let buffer_index = (self.registers.LY as u16 * DISPLAY_WIDTH as u16) + self.registers.LX as u16;

                        self.pixel_buffer[buffer_index as usize] = pixel;
                        self.registers.LX += 1;
                    }
                }

                if self.registers.LX < DISPLAY_WIDTH || self.clock < DRAW_CYCLES {
                    return;
                }

                self.clock -= DRAW_CYCLES;
                self.set_mode(bus, HBlank);
            }
            HBlank => {
                if self.clock < HBLANK_CYCLES {
                    return;
                }

                self.clock -= HBLANK_CYCLES;

                if self.registers.LY == self.registers.LYC {
                    // Set the LYC flag
                    self.registers.STAT |= STAT_LYC_FLAG;

                    if self.registers.STAT & STAT_LYC_INTERRUPT != 0 {
                        interrupt(bus, InterruptType::LCDStat);
                    }
                }
                else {
                    // Clear the LYC flag.
                    self.registers.STAT &= 0xFF ^ STAT_LYC_FLAG;
                }

                self.bgfifo.reset();
                self.spfifo.reset();
                self.registers.LX = 0;
                self.registers.LY += 1;

                if self.registers.LY >= DISPLAY_HEIGHT {
                    self.set_mode(bus, VBlank);
                }
                else {
                    self.set_mode(bus, OAM);
                }
            }
            VBlank => {
                if self.clock < VBLANK_LINE_CYCLES {
                    return;
                }

                self.clock -= VBLANK_LINE_CYCLES;
                self.registers.LY += 1;

                if self.registers.LY < VIRTUAL_DISPLAY_HEIGHT {
                    return;
                }

                self.render_flag = true;

                self.registers.LY = 0;
                self.set_mode(bus, OAM);
            }
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum FifoState {
    FetchTileNo,
    FetchTileLo,
    FetchTileHi,
    PushTile,
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
            _ => panic!("Impossible Branch"),
        }
    }
}

#[derive(Debug)]
struct PixelFifo {
    pixels: [u8; 16],
    size: usize,
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
}

#[derive(Clone, Copy, PartialEq)]
enum TileDataBlock {
    BLK0,
    BLK1,
}

// TODO: Move this to its own file.
#[derive(Debug)]
struct BackgroundFifo {
    fifo: PixelFifo,
    state: FifoState,
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
struct SpriteFifo {
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
    fn new() -> Self {
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

    fn scan_next_oam_table_entry(&mut self, oam: &[Byte], registers: &Registers) {
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

    fn step(&mut self, vram: &[Byte], registers: Registers) {

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
