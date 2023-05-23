mod fifo;
mod background_fifo;
mod sprite_fifo;

use crate::bus::*;
use crate::cpu::{interrupt, InterruptType};
use crate::clock::ClockListener;
use std::fmt;

use std::cell::RefCell;
use std::rc::Rc;

use crate::graphics_driver::GraphicsDriver;
use crate::ppu::background_fifo::BackgroundFifo;
use crate::ppu::Mode::OAM;
use crate::ppu::sprite_fifo::SpriteFifo;

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
