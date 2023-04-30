use std::fs;
use crate::{Address, Attach, Bus, BusListener, Byte};

#[derive(Clone)]
pub struct BootRom {
    data: Vec<u8>,
}

impl BootRom {
    pub fn new(rom_file: &str) -> Self {
        Self {
            data: fs::read(rom_file).expect("Failed to read boot ROM."),
        }
    }
}

impl BusListener for BootRom {
    fn bus_attach(&mut self) -> Vec<Attach> {
        vec![Attach::Block(0x00), Attach::Register(0x50)]
    }

    fn bus_read(&self, address: Address) -> Byte {
        match address {
            0x0000..=0x00FF => self.data[address as usize],
            0xFF50 => panic!("Address {:4X} is write-only.", address),
            _ => panic!("Boot ROM does not own address {:4X}.", address),
        }
    }

    fn bus_write(&mut self, bus: &mut Bus, address: Address, value: Byte) {
        if address == 0xFF50 {
            // self.bus_detach(bus);
            return
        }
        panic!("Boot ROM is Read-Only!")
    }
}