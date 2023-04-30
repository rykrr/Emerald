use std::cell::RefCell;
use std::rc::Rc;
use crate::{Address, Attach, Bus, BusListener, Byte};

const SELECT_BUTTONS: u8 = 1 << 5;
const SELECT_DPAD: u8 = 1 << 4;

const BUTTONS_SHIFT: u8 = 4;
const DPAD_SHIFT: u8 = 0;

pub const BUTTON_START: u8 = 1 << 3;
pub const BUTTON_SELECT: u8 = 1 << 2;
pub const BUTTON_A: u8 = 1 << 1;
pub const BUTTON_B: u8 = 1 << 0;

pub const DPAD_DOWN: u8 = 1 << 3;
pub const DPAD_UP: u8 = 1 << 2;
pub const DPAD_LEFT: u8 = 1 << 1;
pub const DPAD_RIGHT: u8 = 1 << 0;


pub struct Joypad {
    joypad_register: Byte,

    // High nibble is buttons, low nibble is direction.
    // Unlike JOYP, 1 = pressed, 0 = not pressed.
    pressed: Byte,
}

impl BusListener for Joypad {
    fn bus_attach(&mut self) -> Vec<Attach> {
        vec![Attach::Register(0)]
    }

    fn bus_read(&self, address: Address) -> Byte {
        match address {
            0xFF00 => {
                let mut output: u8 = 0;

                // Note: JOYP is inverted; 0 means selected.
                if self.joypad_register & SELECT_BUTTONS == 0 {
                    output |= self.pressed >> BUTTONS_SHIFT;
                }

                if self.joypad_register & SELECT_DPAD == 0 {
                    output |= self.pressed >> BUTTON_SELECT;
                }

                //(self.joypad_register & 0xF0) | (!output & 0x0F)
                0xFF // TODO
            },
            _ => panic!("Not a joypad address {address:04X}")
        }
    }

    fn bus_write(&mut self, bus: &mut Bus, address: Address, value: Byte) {
        let target = match address {
            0xFF00 => self.joypad_register = value & 0xF0, // Last four bytes are R/O
            _ => panic!("Not a joypad address {address:04X}")
        };
    }
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            joypad_register: 0xFF,
            pressed: 0
        }
    }
}