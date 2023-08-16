use std::cell::RefCell;
use std::rc::{Rc, Weak};
//use crate::cpu::Interrupt;

pub type Address = u16;
pub type Byte = u8;
pub type Word = u16;

pub trait BusListener {
    fn addresses(&self) -> Vec<(Address, Address)>;
    fn read(&self, address: Address) -> Byte;
    fn write(&mut self, bus: &mut Bus, address: Address, value: Byte);
}

type BusListenerCell = RefCell<dyn BusListener>;

struct Callback {
    pub base: Address,
    pub last: Address,
    pub listener: Weak<BusListenerCell>,
}

pub struct Bus {
    callbacks: Vec<Callback>,
}

impl Bus {
    pub fn new() -> Self {
        Bus { callbacks: Vec::new() }
    }

    pub fn attach(&mut self, listener: Rc<BusListenerCell>) {
        for (base, last) in listener.borrow().addresses() {
            assert!(base <= last);
            self.callbacks.push(Callback {
                base: base,
                last: last,
                listener: Rc::downgrade(&listener),
            });
        }
    }

    pub fn read_byte(&self, address: Address) -> Byte {
        // println!("Reading address: {:04X}", address);
        for c in &self.callbacks {
            if c.base <= address && address <= c.last {
                return c.listener
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .read(address);
            }
        }
        panic!("Failed to read!")
    }

    pub fn write_byte(&mut self, address: Address, value: Byte) {
        // println!("Address: {:04X} Value: {:02X}", address, value);
        for c in &mut self.callbacks {
            if c.base <= address && address <= c.last {
                return c.listener
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .write(self, address, value);
            }
        }
        panic!("Segfault {:04X}", address)
    }

    pub fn read_word(&self, address: Address) -> Word {
        (self.read_byte(address) as Word) | ((self.read_byte(address + 1) as Word) << 8)
    }

    pub fn write_word(&mut self, address: Address, value: Word) {
        self.write_byte(address, value as Byte);
        self.write_byte(address + 1, (value >> 8) as Byte);
    }

    /*
    pub fn interrupt(&mut self, interrupt: Interrupt) {
        self.write_byte(0xFF0F, interrupt as Byte);
    }
    */

}

use std::fmt;
impl fmt::Debug for Bus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error")
    }
}
