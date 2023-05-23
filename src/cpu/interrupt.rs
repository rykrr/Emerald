use crate::{Address, Attach, Bus, BusListener, Byte};

#[derive(Clone, Copy, Debug)]
pub struct InterruptRegisters {
    pub master_enable: bool, // IME
    pub enable: Byte, // IE
    pub flags: Byte, // IF
}

impl BusListener for InterruptRegisters {
    fn bus_attach(&mut self) -> Vec<Attach> {
        vec![Attach::Register(0x0F), Attach::Register(0xFF)]
    }

    fn bus_read(&self, address: Address) -> Byte {
        match address {
            0xFF0F => self.flags,
            0xFFFF => self.enable,
            _ => panic!("Address {:4X} is write-only.", address)
        }
    }

    fn bus_write(&mut self, bus: &mut Bus, address: Address, value: Byte) {
        let dest = match address {
            0xFF0F => &mut self.flags,
            0xFFFF => &mut self.enable,
            _ => panic!("Address {:4X} is write-only.", address)
        };
        *dest = value;
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum InterruptType {
    VBlank = 1 << 0,
    LCDStat = 1 << 1,
    Timer = 1 << 2,
    Serial = 1 << 3,
    Joypad = 1 << 4,
}

#[inline(always)]
pub fn interrupt(bus: &mut Bus, interrupt_type: InterruptType) {
    let interrupt = bus.read_byte(0xFF0F);
    bus.write_byte(0xFF0F, interrupt | (interrupt_type as u8));
}
