use crate::{Address, Bus, BusListener, Byte, Attach};

const RAM_BASE_ADDRESS: Address = 0xC000;
const BANKED_RAM_BASE_ADDRESS: Address = 0xD000;
const ECHO_RAM_BASE_ADDRESS: Address = 0xE000;

pub struct RAM {
    data: [Byte; 0x2000],
}

impl RAM {
    pub fn new() -> Self {
        Self {
            data: [0; 0x2000]
        }
    }
}

impl BusListener for RAM {
    fn bus_attach(&mut self) -> Vec<Attach> {
        vec![Attach::BlockRange(0xC0, 0xFD)]
    }

    fn bus_read(&self, address: Address) -> Byte {
        match address {
            0xC000..=0xDFFF => self.data[(address - RAM_BASE_ADDRESS) as usize],
            0xE000..=0xFDFF => self.data[(address - ECHO_RAM_BASE_ADDRESS) as usize],
            _ => panic!("Segfault {address:4X}")
        }
    }

    fn bus_write(&mut self, _bus: &mut Bus, address: Address, value: Byte) {
        match address {
            0xC000..=0xDFFF => self.data[(address - RAM_BASE_ADDRESS) as usize] = value,
            0xE000..=0xFDFF => self.data[(address - ECHO_RAM_BASE_ADDRESS) as usize] = value,
            _ => panic!("Segfault {address:4X}")
        }
    }
}


pub struct DummyRAM {
    pub data: Vec<u8>,
    locked: bool,
    lock_value: Byte,
    address_range: (u8, u8),
    is_register: bool
}

impl DummyRAM {
    pub fn new(start: u8, end: u8, is_register: bool) -> Self {
        assert!(start <= end, "Start address must precede end address.");

        let size = ((end - start) + 1) as usize;
        let size = if is_register { size } else { size * 0x100 };

        Self {
            data: vec![0; size as usize],
            locked: false,
            lock_value: 0,
            address_range: (start, end),
            is_register,
        }
    }

    /// Writes will do nothing and all reads will return $value
    pub fn lock_with_value(&mut self, value: Byte) {
        self.locked = true;
        self.lock_value = value;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }
}

impl BusListener for DummyRAM {
    fn bus_attach(&mut self) -> Vec<Attach> {
        if self.is_register {
            //if self.address_range.0 == self.address_range.1
            vec![Attach::RegisterRange(self.address_range.0, self.address_range.1)]
        }
        else {
            vec![Attach::BlockRange(self.address_range.0, self.address_range.1)]
        }
    }

    fn bus_read(&self, address: Address) -> Byte {
        if self.locked {
            return self.lock_value;
        }

        let base = if self.is_register {
            0xFF00u16 + (self.address_range.0 as Address)
        }
        else {
            (self.address_range.0 as Address) << 8
        };

        self.data[(address - base) as usize]
    }

    fn bus_write(&mut self, _bus: &mut Bus, address: Address, value: Byte) {
        let base = if self.is_register {
            0xFF00u16 | (self.address_range.0 as Address)
        }
        else {
            (self.address_range.0 as Address) << 8
        };

        self.data[(address - base) as usize] = value;
    }
}

pub struct RegisterHoles;

impl RegisterHoles {
    pub fn new() -> Self {
        Self {}
    }
}

impl BusListener for RegisterHoles {
    fn bus_attach(&mut self) -> Vec<Attach> {
        vec![
            Attach::RegisterRange(0x08, 0x0E),
            //Attach::RegisterRange(0x27, 0x2F),
            Attach::RegisterRange(0x4C, 0x7F),
        ]
    }

    fn bus_read(&self, address: Address) -> Byte {
        0
    }

    fn bus_write(&mut self, _bus: &mut Bus, _address: Address, _value: Byte) {

    }
}
