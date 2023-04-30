use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::{Address, Attach, Bus, BusListener, Byte};
use crate::clock::ClockListener;

const SERIAL_CONTROL_TRANSFER_START: u8 = 1 << 7; // 1 = Active or Requested
const SERIAL_CONTROL_CLOCK_SPEED: u8 = 1 << 1; // 1 = Fast
const SERIAL_CONTROL_SHIFT_CLOCK: u8 = 1 << 0; // 0 = Ext Clock, 1 = Int Clock

const SERIAL_LO_SPEED_CYCLES: u16 = 512;
const SERIAL_HI_SPEED_CYCLES: u16 = 16;

#[derive(Clone)]
pub struct SerialInterface {
    quiet: bool,
    cycles: u16,
    transfer_data: u8,
    transfer_control: u8,
}

impl SerialInterface {
    pub fn new(quiet: bool) -> Self {
        Self {
            quiet: quiet,
            cycles: 0,
            transfer_data: 0,
            transfer_control: 0,
        }
    }
}

impl BusListener for SerialInterface {
    fn bus_attach(&mut self) -> Vec<Attach> {
        vec![Attach::RegisterRange(1, 2)]
    }

    fn bus_read(&self, address: Address) -> Byte {
        match address {
            0xFF01 => self.transfer_data,
            0xFF02 => self.transfer_control,
            _ => panic!("{} is not part of the serial interface!", address)
        }
    }

    fn bus_write(&mut self, bus: &mut Bus, address: Address, value: Byte) {
        match address {
            0xFF01 => self.transfer_data = value,
            0xFF02 => {
                self.transfer_control = value;
                if !self.quiet && value & SERIAL_CONTROL_TRANSFER_START != 0 {
                    print!("{}", self.transfer_data as char);
                }
            },
            _ => panic!("{} is not part of the serial interface!", address)
        };
    }
}

impl ClockListener for SerialInterface {
    fn callback(&mut self, _bus: &mut Bus, _cycles: u8) {
        return;
        /*
        if self.transfer_control & SERIAL_CONTROL_TRANSFER_START == 0 {
            return;
        }

        let transfer_cycles = if self.transfer_control & SERIAL_CONTROL_CLOCK_SPEED == 0 {
            SERIAL_LO_SPEED_CYCLES
        }
        else {
            SERIAL_HI_SPEED_CYCLES
        };

        self.cycles += cycles as u16;

        if self.cycles < transfer_cycles {
            return;
        }

        self.cycles = 0;
        self.transfer_control ^= SERIAL_CONTROL_TRANSFER_START;
         */
    }
}