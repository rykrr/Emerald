mod registers;
mod flag;
mod interrupt;
mod instructions;

use flag::Flag;
use registers::Register;
use interrupt::InterruptRegisters;
pub use interrupt::{InterruptType, interrupt};

use crate::bus::*;
use crate::timer::*;
use std::fmt;
use log::trace;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteDescriptor {
    // General purpose registers
    A,
    B,
    C,
    D,
    E,
    H,
    L,

    // Dereference address in register (i/d = increment/decrement)
    BC,
    DE,
    HL,
    HLi,
    HLd,

    // Indirect = dereference address in immediate
    Immediate,
    Indirect,

    // Dereference address 0xFF00 + immediate and 0xFF00 + C
    HighAddress,
    HighAddressC,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WordDescriptor {
    // General purpose registers
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,

    // Indirect = dereference address in immediate
    Immediate,
    Indirect,
}

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug)]
pub struct CPU {
    // General Purpose Registers
    af: Register,
    bc: Register,
    de: Register,
    hl: Register,

    // Stack Pointer, Program Counter
    sp: Word,
    pub pc: Word,

    halted: bool,

    // A separate struct is used to hold all interrupt registers as
    // the CPU cannot be borrowed mut on the bus while also being stepped.
    interrupt_registers: Rc<RefCell<InterruptRegisters>>,
}

impl CPU {
    pub fn new() -> Self {
        let interrupt_registers = Rc::new(RefCell::new(InterruptRegisters {
            master_enable: false,
            enable: 0xE0,
            flags: 0x00,
        }));

        CPU {
            af: Register::new(0x01B0),
            bc: Register::new(0x0013),
            de: Register::new(0x00D8),
            hl: Register::new(0x014D),
            sp: 0xFFFE,
            pc: 0x0100,
            halted: false,
            interrupt_registers,
        }
    }

    pub fn attach_to_bus(&self, bus: &mut Bus) {
        bus.attach(self.interrupt_registers.clone());
    }

    #[inline(always)]
    fn set_flag_if(&mut self, flag: Flag, cond: bool) {
        if cond {
            self.set_flag(flag)
        } else {
            self.clear_flag(flag);
        }
    }

    #[inline(always)]
    fn set_flag(&mut self, flag: Flag) {
        Flag::set(self.af.right(), flag)
    }

    #[inline(always)]
    fn clear_flag(&mut self, flag: Flag) {
        Flag::clear(self.af.right(), flag)
    }

    #[inline(always)]
    fn test_flag(&mut self, flag: Flag) -> bool {
        Flag::test(self.af.right(), flag)
    }

    #[inline(always)]
    fn clear_all_flags(&mut self) {
        *self.af.right() = 0x00
    }

    #[inline(always)]
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    fn read_byte(&mut self, bus: &Bus, desc: ByteDescriptor) -> u8 {
        use ByteDescriptor::*;
        match desc {
            A => *self.af.left(),
            B => *self.bc.left(),
            C => *self.bc.right(),
            D => *self.de.left(),
            E => *self.de.right(),
            H => *self.hl.left(),
            L => *self.hl.right(),
            BC => bus.read_byte(*self.bc.word()),
            DE => bus.read_byte(*self.de.word()),
            HL => bus.read_byte(*self.hl.word()),
            HLi => {
                let byte = bus.read_byte(*self.hl.word());
                self.hl.inc_word();
                byte
            }
            HLd => {
                let byte = bus.read_byte(*self.hl.word());
                self.hl.dec_word();
                byte
            }
            Immediate => {
                let byte = bus.read_byte(self.pc);
                self.pc += 1;
                byte
            }
            Indirect => {
                let addr = bus.read_word(self.pc);
                self.pc += 2;
                bus.read_byte(addr)
            }
            HighAddress => {
                let addr = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                bus.read_byte(0xFF00 | addr)
            }
            HighAddressC => bus.read_byte(0xFF00 | (*self.bc.right() as u16)),
        }
    }

    fn write_byte(&mut self, bus: &mut Bus, desc: ByteDescriptor, value: u8) {
        use ByteDescriptor::*;
        match desc {
            A => *self.af.left() = value,
            B => *self.bc.left() = value,
            C => *self.bc.right() = value,
            D => *self.de.left() = value,
            E => *self.de.right() = value,
            H => *self.hl.left() = value,
            L => *self.hl.right() = value,
            BC => bus.write_byte(*self.bc.word(), value),
            DE => bus.write_byte(*self.de.word(), value),
            HL => bus.write_byte(*self.hl.word(), value),
            HLi => {
                bus.write_byte(*self.hl.word(), value);
                self.hl.inc_word();
            }
            HLd => {
                bus.write_byte(*self.hl.word(), value);
                self.hl.dec_word();
            }
            Indirect => {
                let addr = bus.read_word(self.pc);
                self.pc += 2;
                bus.write_byte(addr, value);
            }
            HighAddress => {
                let addr = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                bus.write_byte(0xFF00 | addr, value);
            },
            HighAddressC => {
                bus.write_byte(0xFF00 | (*self.bc.right() as u16), value);
            }
            _ => panic!("Invalid write_byte() descriptor: {:?}", desc),
        }
    }

    fn read_word(&mut self, bus: &Bus, desc: WordDescriptor) -> u16 {
        use WordDescriptor::*;
        match desc {
            AF => *self.af.word(),
            BC => *self.bc.word(),
            DE => *self.de.word(),
            HL => *self.hl.word(),
            SP => self.sp,
            PC => self.pc,
            Immediate => {
                let word = bus.read_word(self.pc);
                self.pc += 2;
                word
            }
            Indirect => {
                let addr = bus.read_word(self.pc);
                let word = bus.read_word(addr);
                self.pc += 2;
                word
            }
        }
    }

    fn write_word(&mut self, bus: &mut Bus, desc: WordDescriptor, value: Word) {
        use WordDescriptor::*;
        match desc {
            AF => *self.af.word() = value,
            BC => *self.bc.word() = value,
            DE => *self.de.word() = value,
            HL => *self.hl.word() = value,
            SP => self.sp = value,
            PC => self.pc = value,
            Indirect => {
                let addr = bus.read_word(self.pc);
                self.pc += 2;
                bus.write_word(addr, value);
            }
            _ => panic!("Invalid write_word() descriptor"),
        }
    }

    pub fn print_stack(&mut self, bus: &Bus, stack_base: Address) {
        let mut values: Vec<(Address, Word)> = Vec::new();
        for address in (self.sp..stack_base).step_by(2) {
            values.push((address, bus.read_word(address)));
        }

        // Two sequential loops are used because bus.read_word may output multiple debug lines.
        // This ensures that the output is clean.
        println!("Stack: ");
        for (address, value) in values {
            println!("\t{address:04X}: {value:04X}");
        }
    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let int = *self.interrupt_registers.borrow();
        write! {f,
            concat! {
                "CPU | PC {:04X}  SP {:04X}  FE {:02X}{:02X}  F  {}\n",
                "    | AF {}  BC {}  DE {}  HL {}\n",
            },
            self.pc, self.sp, int.flags, int.enable, Flag::fmt(self.af.value() as u8),
            self.af, self.bc, self.de, self.hl
        }
    }
}

