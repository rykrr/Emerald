#![feature(exclusive_range_pattern)]

mod bus;
use crate::bus::*;

mod clock;
use crate::clock::*;

mod cpu;
use crate::cpu::*;

mod ppu;
use crate::ppu::*;

struct Cartridge {
    data: Vec<u8>
}

impl Cartridge {
    fn new() -> Self {
        Self {
            //data: vec![ 0x81, 0x4F, 0x91, 0x82, 0xA4, 0xB5, 0x76 ]
            //data: vec![0xAF, 0x3C, 0x17, 0x17, 0x17, 0x17, 0x76]
            data: vec![0x3E, 0x06, 0x3D, 0x20, 0xFE, 0x00, 0x76]
        }
    }
}

impl BusListener for Cartridge {
    fn addresses(&self) -> Vec<(Address, Address)> {
        vec![(0x00, 0x10)]
    }

    fn read(&self, address: Address) -> Byte {
        self.data[address as usize]
    }

    fn write(&mut self, _bus: &mut Bus, address: Address, value: Byte) {
        println!("Writing address: {:04X}", address);
        self.data[address as usize] = value
    }
}

use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let mut bus = Bus::new();
    let mut clk = Clock::new();
    let mut cpu = CPU::new();
    let mut ppu = Rc::new(RefCell::new(PPU::new()));
    bus.attach(ppu.clone());

    println!("{}", ppu.borrow());
    bus.write_byte(0xFF41, 0xAA);
    println!("{}", ppu.borrow());

    let mut dum = Rc::new(RefCell::new(Cartridge::new()));
    bus.attach(dum.clone());

    println!("{}", cpu);
    while !cpu.is_halted() {
        cpu.step(&mut bus, &mut clk);
        println!("{}", cpu);
    }
}
