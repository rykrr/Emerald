use crate::{Address, Attach, Bus, BusListener, Byte};

use log::warn;

const RAM_BIT: u16 = 0x8000;

pub struct RomOnlyCartridge {
    bytes: Vec<u8>
}

impl RomOnlyCartridge {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        let size = bytes.len();
        if size > 0x8000 {
            warn!("Cartridge is larger than 32KiB! (Actual size: {size} bytes)")
        }

        Self {
            bytes
        }
    }
}

impl BusListener for RomOnlyCartridge {
    fn bus_attach(&mut self) -> Vec<Attach> {
        vec![Attach::BlockRange(0, 0x7F), Attach::BlockRange(0xA0, 0xBF)]
    }

    fn bus_read(&self, address: Address) -> Byte {
        if address & RAM_BIT != 0 {
            return 0xFF;
        }
        self.bytes[address as usize]
    }

    fn bus_write(&mut self, bus: &mut Bus, address: Address, value: Byte) {
        if address & RAM_BIT != 0 {
            warn!("Attempted to write to non-existent RAM at {address:04X}.")
        }
        if address == 0x2000 {
            return
        }
        //panic!("Cartridge address {:04X} is read-only!", address);
    }
}