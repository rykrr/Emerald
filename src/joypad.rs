use std::cell::RefCell;
use std::rc::Rc;
use crate::{Address, Attach, Bus, BusListener, Byte, Debugger};

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

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Button {
    Start = BUTTON_START,
    Select = BUTTON_SELECT,
    A = BUTTON_A,
    B = BUTTON_B
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Direction {
    Down = DPAD_DOWN,
    Up = DPAD_UP,
    Left = DPAD_LEFT,
    Right = DPAD_RIGHT
}

#[derive(Copy, Clone)]
pub enum JoypadButtons {
    Button(Button),
    Direction(Direction),
    Pause
}

pub trait JoypadDriver {
    fn get_buttons(&mut self) -> Vec<JoypadButtons>;
}

pub struct Joypad {
    joypad_register: Byte,

    // High nibble is buttons, low nibble is direction.
    // Unlike JOYP, 1 = pressed, 0 = not pressed.
    buttons: Byte,
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
                    output |= self.buttons >> BUTTONS_SHIFT;
                }

                if self.joypad_register & SELECT_DPAD == 0 {
                    output |= self.buttons >> DPAD_SHIFT;
                }

                (self.joypad_register & 0xF0) | (!output & 0x0F)
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
            buttons: 0
        }
    }

    pub fn update(&mut self, driver: &mut dyn JoypadDriver, debugger: &mut Debugger) {
        let mut buttons: u8 = 0;
        let mut directions: u8 = 0;
        driver.get_buttons().iter().for_each(|input | {
            match input {
                JoypadButtons::Button(button) => buttons |= *button as u8,
                JoypadButtons::Direction(direction) => directions |= *direction as u8,
                JoypadButtons::Pause => debugger.stop(),
            }
        });
        self.buttons = (buttons << BUTTONS_SHIFT) | (directions << DPAD_SHIFT);
    }
}