use crate::cpu::{interrupt, InterruptType};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};
use crate::clock::ClockListener;


use crate::bus::*;

const TIMER_CONTROL_ENABLE: u8 = 1 << 2;
const TIMER_CONTROL_CLOCK_SELECT_MASK: u8 = 2;

const TIMER_CLOCK_BITS: [u16; 4] = [ 1 << 10, 1 << 7, 1 << 5, 1 << 3 ];


pub struct Timer {
    cycles: u16, // DIV is high nibble
    counter_register: Byte, // TIMA
    modulo_register: Byte, // TMA
    control_register: Byte, // TAC
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            cycles: 0,
            counter_register: 0,
            modulo_register: 0,
            control_register: 0,
        }
    }
}

impl BusListener for Timer {
    fn bus_attach(&mut self) -> Vec<Attach> {
        vec![Attach::RegisterRange(4, 7)]
    }

    fn bus_read(&self, address: Address) -> Byte {
        match address {
            0xFF04 => (self.cycles >> 8) as Byte,
            0xFF05 => self.counter_register,
            0xFF06 => self.modulo_register,
            0xFF07 => self.control_register,
            _ => panic!("{} is not a timer register!", address)
        }
    }

    fn bus_write(&mut self, bus: &mut Bus, address: Address, value: Byte) {
        match address {
            0xFF04 => self.cycles = 0,
            0xFF05 => self.counter_register = value,
            0xFF06 => self.modulo_register = value,
            0xFF07 => self.control_register = value,
            _ => panic!("{} is not a timer register!", address)
        }
    }
}

impl ClockListener for Timer {
    fn callback(&mut self, bus: &mut Bus, cycles: u8) {
        for _ in 0..cycles {
            if self.cycles == 0xFFFF {
                self.cycles = 0;
            }
            else {
                self.cycles += 1;
            }

            if self.control_register & TIMER_CONTROL_ENABLE != 0 {
                let clock_select = self.control_register & TIMER_CONTROL_CLOCK_SELECT_MASK;

                if TIMER_CLOCK_BITS[clock_select as usize] != 0 {
                    // if counter overflows...
                    if self.counter_register == 0xFF {
                        self.counter_register = self.modulo_register; // Set TIMA to TMA
                        interrupt(bus, InterruptType::Timer);
                    }
                    else {
                        self.counter_register += 1;
                    }
                }
            }
        }
    }
}

use std::fmt;

impl fmt::Debug for Timer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error")
    }
}
