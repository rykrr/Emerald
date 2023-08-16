use std::fmt;
use std::sync::Condvar;
use crate::bus::*;
use crate::clock::*;

union Register {
    byte: (u8, u8),
    word: u16
}

impl Register {
    #[inline(always)]
    fn new(word: Word) -> Self {
        Register { word: word }
    }

    #[inline(always)]
    fn right(&mut self) -> &mut u8 {
        unsafe { &mut self.byte.0 }
    }

    #[inline(always)]
    fn left(&mut self) -> &mut u8 {
        unsafe { &mut self.byte.1 }
    }

    #[inline(always)]
    fn word(&mut self) -> &mut u16 {
        unsafe { &mut self.word }
    }

    #[inline(always)]
    fn value(&self) -> u16 {
        unsafe { self.word }
    }

    #[inline(always)]
    fn inc_word(&mut self) {
        unsafe { self.word += 1 };
    }

    #[inline(always)]
    fn dec_word(&mut self) {
        unsafe { self.word -= 1 };
    }

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X}", unsafe { self.word })
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}

impl fmt::Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}

#[derive(Copy, Clone)]
#[allow(unused)]
#[repr(u8)]
pub enum Flag {
    Zero = 0x80,
    Subtract = 0x40,
    HalfCarry = 0x20,
    Carry = 0x10
}

impl Flag {
    pub fn fmt(flags: u8) -> String {
        use Flag::*;

        const LITERALS: [(Flag, char); 4] = [
            (Zero, 'Z'),
            (Subtract, 'N'),
            (HalfCarry, 'H'),
            (Carry, 'C'),
        ];

        let mut out: [char; 4] = ['-', '-', '-', '-'];

        for i in 0..4 {
            if Flag::test(&flags, LITERALS[i].0) {
                out[i] = LITERALS[i].1;
            }
        }

        String::from_iter(out)
    }

    fn set(flags: &mut u8, flag: Flag) {
        *flags |= flag as u8
    }

    fn clear(flags: &mut u8, flag: Flag) {
        *flags &= !(flag as u8)
    }

    fn test(flags: &u8, flag: Flag) -> bool {
        flags & (flag as u8) != 0
    }
}

#[allow(unused)]
#[derive(Clone, Copy, PartialEq)]
pub enum ByteDescriptor {
    // General purpose registers
    A, B, C, D, E, H, L,

    // Dereference address in register (i/d = increment/decrement)
    BC, DE, HL, HLi, HLd, 

    // Indirect = dereference address in immediate
    Immediate, Indirect,

    // Dereference address 0xFF00 + immediate and 0xFF00 + C
    HighAddress, HighAddressC
}

#[allow(unused)]
#[derive(Clone, Copy, PartialEq)]
pub enum WordDescriptor {
    // General purpose registers
    AF, BC, DE, HL, SP, PC,

    // Indirect = dereference address in immediate
    Immediate, Indirect
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum Direction {
    Left, Right
}

/*
#[repr(u8)]
pub enum Interrupt {
    VBlank,
    HBlank,
    Joypad,
    Timer
}
*/

#[derive(Debug)]
pub struct CPU {
    // General Purpose Registers
    af: Register,
    bc: Register,
    de: Register,
    hl: Register,

    // Stack Pointer, Program Counter
    sp: u16,
    pc: u16,

    // Interrupts
    interrupt_master_enable: bool, // IME
    interrupt_enable: u8, // IE
    interrupt_flags: u8, // IF

    // CPU State
    halted: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            af: Register::new(0x01B0),
            bc: Register::new(0x0013),
            de: Register::new(0x00DE),
            hl: Register::new(0x014D),
            sp: 0xFFFE,
            pc: 0x0000,
            interrupt_master_enable: true,
            interrupt_enable: 0xFF,
            interrupt_flags: 0x00,
            halted: false,
        }
    }

    #[inline(always)]
    fn set_flag_if(&mut self, flag: Flag, cond: bool) {
        if cond {
            self.set_flag(flag)
        }
        else {
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
            },
            HLd => {
                let byte = bus.read_byte(*self.hl.word());
                self.hl.dec_word();
                byte
            },
            Immediate => {
                let byte = bus.read_byte(self.pc);
                self.pc += 1;
                byte
            },
            Indirect => {
                let addr = bus.read_word(self.pc);
                self.pc += 2;
                bus.read_byte(addr)
            },
            HighAddress => {
                let addr = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                bus.read_byte(0xFF00 | addr)
            },
            HighAddressC => bus.read_byte(0xFF00 | (*self.bc.right() as u16))
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
            },
            HLd => {
                bus.write_byte(*self.hl.word(), value);
                self.hl.dec_word();
            },
            Indirect => {
                let addr = bus.read_word(self.pc);
                self.pc += 2;
                bus.write_byte(addr, value);
            },
            HighAddress => {
                let addr = bus.read_byte(self.pc) as u16;
                self.pc += 1;
                bus.write_byte(0xFF00 | addr, value);
            },
            _ => panic!("Invalid write_byte() descriptor")
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
            },
            Indirect => {
                let addr = bus.read_word(self.pc);
                let word = bus.read_word(addr);
                self.pc += 2;
                word
            },
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
            },
            _ => panic!("Invalid write_word() descriptor"),
        }
    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!{f,
            concat! {
                "CPU | PC {:04X}  SP {:04X}  F  {}\n",
                "    | AF {}  BC {}  DE {}  HL {}\n",
            },
            self.pc, self.sp, Flag::fmt(self.af.value() as u8),
            self.af, self.bc, self.de, self.hl
        }
    }
}

impl CPU {
    #[inline(always)]
    pub fn ld(&mut self, bus: &mut Bus, dest: ByteDescriptor, src: ByteDescriptor) {
        let value = self.read_byte(bus, src);
        self.write_byte(bus, dest, value);
    }

    #[inline(always)]
    pub fn ld_16(&mut self, bus: &mut Bus, dest: WordDescriptor, src: WordDescriptor) {
        let value = self.read_word(bus, src);
        self.write_word(bus, dest, value);
    }

    fn test_condition(&mut self, condition: Option<(Flag, bool)>) -> bool {
        match condition {
            Some((flag, value)) => self.test_flag(flag) == value,
            None => true
        }
    }

    #[inline(always)]
    pub fn jp(&mut self, bus: &mut Bus, condition: Option<(Flag, bool)>) -> bool {
        let address = self.read_word(bus, WordDescriptor::Immediate);
        if self.test_condition(condition) {
            self.pc = address;
            return true;
        }
        return false;
    }

    #[inline(always)]
    pub fn jr(&mut self, bus: &mut Bus, condition: Option<(Flag, bool)>) -> bool {
        let offset = self.read_byte(bus, ByteDescriptor::Immediate) as i8;
        if self.test_condition(condition) {
            println!("JR {:02}", offset);
            if offset < 0 {
                self.pc -= ((offset - 1) * -1) as u16;
            }
            else {
                self.pc += offset as u16;
            }
            return true;
        }
        return false;
    }

    pub fn call(&mut self, bus: &mut Bus, condition: Option<(Flag, bool)>) -> bool {
        if self.test_condition(condition) {
            self.push(bus, WordDescriptor::PC);
            self.pc = self.read_word(bus, WordDescriptor::Immediate);
            return true;
        }
        return false;
    }

    #[inline(always)]
    pub fn rst(&mut self, bus: &mut Bus, address: Address) {
        self.push(bus, WordDescriptor::PC);
        self.pc = address;
    }

    pub fn ret(&mut self, bus: &mut Bus, condition: Option<(Flag, bool)>) -> bool {
        if self.test_condition(condition) {
            self.pop(bus, WordDescriptor::PC);
            return true;
        }
        return false;
    }

    #[inline(always)]
    pub fn reti(&mut self, bus: &mut Bus) {
        self.interrupt_master_enable = true;
        self.pop(bus, WordDescriptor::PC);
    }

    #[inline(always)]
    pub fn push(&mut self, bus: &mut Bus, src: WordDescriptor) {
        self.sp -= 2;
        let mut value = self.read_word(bus, src);
        if src == WordDescriptor::AF {
            value &= 0xFFF0;
        }
        bus.write_word(self.sp, value);
    }

    #[inline(always)]
    pub fn pop(&mut self, bus: &mut Bus, dest: WordDescriptor) {
        let value = bus.read_word(self.sp);
        self.write_word(bus, dest, value);
        self.sp += 2;
    }

    pub fn add(&mut self, bus: &mut Bus, register: ByteDescriptor, with_carry: bool) {
        let a_value = *self.af.left();
        let value = self.read_byte(bus, register);
        let carry = if with_carry && self.test_flag(Flag::Carry) { 1 } else { 0 };
        let (result, carry) = a_value.overflowing_add(value + carry);

        self.clear_all_flags();
        self.set_flag_if(Flag::HalfCarry, (a_value ^ value ^ result) & 0x10 != 0);
        self.set_flag_if(Flag::Carry, carry);
        self.set_flag_if(Flag::Zero, result == 0);

        *self.af.left() = result;
    }

    pub fn add_16(&mut self, bus: &mut Bus, register: WordDescriptor) {
        let hl_value = self.hl.value();
        let value= self.read_word(bus, register);
        let (result, carry) = hl_value.overflowing_add(value);

        self.clear_flag(Flag::Subtract);
        self.set_flag_if(Flag::HalfCarry, (hl_value ^ value ^ result) & 0x1000 != 0);
        self.set_flag_if(Flag::Carry, carry);
        *self.hl.word() = result;
    }

    pub fn relative_sp(&mut self, bus: &mut Bus) -> Word {
        let sp_value = self.sp;
        let value= self.read_byte(bus, ByteDescriptor::Immediate) as i8;

        self.clear_all_flags();

        if value >= 0 {
            let value = value as u16;
            let (result, carry) = sp_value.overflowing_add(value);
            self.set_flag_if(Flag::HalfCarry, (sp_value ^ value ^ result) & 0x10 != 0);
            self.set_flag_if(Flag::Carry, carry);
            result
        }
        else {
            let value = (-1 * value) as u16;
            let (result, carry) = sp_value.overflowing_sub(value);
            self.set_flag_if(Flag::HalfCarry, result & 0x0F <= sp_value & 0x0F);
            self.set_flag_if(Flag::Carry, carry);
            result
        }
    }

    pub fn add_sp(&mut self, bus: &mut Bus) {
        self.sp = self.relative_sp(bus);
    }

    pub fn ld_sp(&mut self, bus: &mut Bus) {
        let address = self.relative_sp(bus);
        self.write_word(bus, WordDescriptor::HL, address);
    }

    pub fn sub(&mut self, bus: &mut Bus, register: ByteDescriptor, with_carry: bool) {
        let value = self.read_byte(bus, register);
        let a_value = *self.af.left();
        let carry = if with_carry && self.test_flag(Flag::Carry) { 1 } else { 0 };
        let (result, carry) = a_value.overflowing_sub(value + carry);

        self.clear_all_flags();
        self.set_flag(Flag::Subtract);
        self.set_flag_if(Flag::HalfCarry, result & 0x0F <= a_value & 0x0F);
        self.set_flag_if(Flag::Carry, carry);
        self.set_flag_if(Flag::Zero, result == 0);
        *self.af.left() = result;
    }

    pub fn cp(&mut self, bus: &mut Bus, register: ByteDescriptor) {
        let value = self.read_byte(bus, register);
        let a_value = *self.af.left();
        let (result, carry) = a_value.overflowing_sub(value);

        self.clear_all_flags();
        self.set_flag(Flag::Subtract);
        self.set_flag_if(Flag::HalfCarry, (a_value ^ value ^ result) & 0x10 != 0);
        self.set_flag_if(Flag::Carry, carry);
        self.set_flag_if(Flag::Zero, result == 0);
    }

    #[inline(always)]
    pub fn cpl(&mut self) {
        *self.af.left() = !*self.af.left();
        self.set_flag(Flag::Subtract);
        self.set_flag(Flag::HalfCarry);
    }

    pub fn ccf(&mut self) {
        if self.test_flag(Flag::Carry) {
            self.clear_flag(Flag::Carry);
        }
        else {
            self.set_flag(Flag::Carry);
        }
        self.clear_flag(Flag::HalfCarry);
        self.clear_flag(Flag::Subtract);
    }

    pub fn scf(&mut self) {
        self.set_flag(Flag::Carry);
        self.clear_flag(Flag::HalfCarry);
        self.clear_flag(Flag::Subtract);
    }

    pub fn inc(&mut self, bus: &mut Bus, register: ByteDescriptor) {
        let value = self.read_byte(bus, register);
        let (result, carry) = value.overflowing_add(1);

        self.clear_flag(Flag::Subtract);
        self.set_flag_if(Flag::HalfCarry, value & 0x0F == 0x0F);
        self.set_flag_if(Flag::Carry, carry);
        self.write_byte(bus, register, result);
    }

    pub fn inc_16(&mut self, bus: &mut Bus, register: WordDescriptor) {
        let (value, _) = self.read_word(bus, register).overflowing_add(1);
        self.write_word(bus, register, value);
    }

    pub fn dec(&mut self, bus: &mut Bus, register: ByteDescriptor) {
        let value = self.read_byte(bus, register);
        let (result, _) = value.overflowing_sub(1);

        self.set_flag(Flag::Subtract);
        self.set_flag_if(Flag::HalfCarry, (value ^ result) & 0x10 != 0);
        self.set_flag_if(Flag::Zero, result == 0);

        self.write_byte(bus, register, result);
    }

    pub fn dec_16(&mut self, bus: &mut Bus, register: WordDescriptor) {
        let (value, _) = self.read_word(bus, register).overflowing_sub(1);
        self.write_word(bus, register, value);
    }

    pub fn and(&mut self, bus: &mut Bus, register: ByteDescriptor) {
        let value = self.read_byte(bus, register);
        let result = *self.af.left() & value;

        self.clear_all_flags();
        self.set_flag(Flag::HalfCarry);
        self.set_flag_if(Flag::Zero, result == 0);
        *self.af.left() = result;
    }

    pub fn or(&mut self, bus: &mut Bus, register: ByteDescriptor) {
        let value = self.read_byte(bus, register);
        let result = *self.af.left() | value;

        self.clear_all_flags();
        self.set_flag_if(Flag::Zero, result == 0);
        *self.af.left() = result;
    }

    pub fn xor(&mut self, bus: &mut Bus, register: ByteDescriptor) {
        let value = self.read_byte(bus, register);
        let result = *self.af.left() ^ value;

        self.clear_all_flags();
        self.set_flag_if(Flag::Zero, result == 0);
        *self.af.left() = result;
    }

    pub fn rotate(&mut self, bus: &mut Bus, register: ByteDescriptor,
                  direction: Direction, with_carry: bool, with_zero: bool) {
        let value = self.read_byte(bus, register);
        let (mut result, carry) = match direction {
            Direction::Left => value.overflowing_shl(1),
            Direction::Right => value.overflowing_shr(1)
        };

        if (with_carry && self.test_flag(Flag::Carry)) || (!with_carry && carry) {
            match direction {
                Direction::Left => result |= 1,
                Direction::Right => result |= 0x80
            }
        }

        self.clear_all_flags();
        self.set_flag_if(Flag::Carry, carry);
        self.set_flag_if(Flag::Zero, with_zero && result == 0);
        self.write_byte(bus, register, result);
    }

    pub fn shift(&mut self, bus: &mut Bus, register: ByteDescriptor,
                 direction: Direction, arithmetic_shift: bool) {
        let value = self.read_byte(bus, register);
        let (result, carry) = match direction {
            Direction::Left => value.overflowing_shl(1),
            Direction::Right => {
                let msb = if arithmetic_shift { 1u8 } else { 0u8 } << 7;
                let (result, carry) = value.overflowing_shr(1);
                (msb | result, carry)
            }
        };

        self.clear_all_flags();
        self.set_flag_if(Flag::Zero, result == 0);
        self.write_byte(bus, register, result);
    }

    pub fn swap(&mut self, bus: &mut Bus, register: ByteDescriptor) {
        let a_value = *self.af.left();
        let tmp_value = self.read_byte(bus, register);
        self.clear_all_flags();
        self.set_flag_if(Flag::Zero, tmp_value == 0);
        *self.af.left() = tmp_value;
        self.write_byte(bus, register, a_value);
    }

    #[inline(always)]
    pub fn test_bit(&mut self, bus: &mut Bus, register: ByteDescriptor, bit: u8) {
        let value = self.read_byte(bus, register) & (1u8 << bit);
        self.clear_flag(Flag::Subtract);
        self.set_flag(Flag::HalfCarry);
        self.set_flag_if(Flag::Zero, value == 0);
    }

    #[inline(always)]
    pub fn set_bit(&mut self, bus: &mut Bus, register: ByteDescriptor, bit: u8) {
        let value = self.read_byte(bus, register) | (1u8 << bit);
        self.write_byte(bus, register, value);
    }

    #[inline(always)]
    pub fn clear_bit(&mut self, bus: &mut Bus, register: ByteDescriptor, bit: u8) {
        let value = self.read_byte(bus, register) & !(1u8 << bit);
        self.write_byte(bus, register, value);
    }

    pub fn daa(&mut self) {
        let mut value = *self.af.left();

        if self.test_flag(Flag::Subtract) {
            if self.test_flag(Flag::Carry) {
                value = value.overflowing_sub(0x60).0;
            }
            if self.test_flag(Flag::HalfCarry) {
                value = value.overflowing_sub(0x06).0;
            }
        }
        else {
            if self.test_flag(Flag::Carry) || value > 0x99 {
                value = value.overflowing_add(0x60).0;
                self.set_flag(Flag::Carry);
            }
            if self.test_flag(Flag::HalfCarry) || value & 0x0F > 0x09 {
                value = value.overflowing_add(0x06).0;
            }
        }

        self.set_flag_if(Flag::Zero, value == 0);
        self.clear_flag(Flag::HalfCarry);
        *self.af.left() = value;
    }
}

impl CPU {
    pub fn step(&mut self, bus: &mut Bus, clock: &mut Clock) {
        clock.cycle_start();
        clock.increment(self.step_instruction(bus));
        clock.cycle_end();
    }

    fn step_instruction(&mut self, bus: &mut Bus) -> u8 {
        let opcode = self.read_byte(bus, ByteDescriptor::Immediate);
        // println!("PC {:04X} OP {:02X}", self.pc - 1, opcode);

        type Byte = ByteDescriptor;
        type Word = WordDescriptor;

        match opcode {
            0x00 => 1, // NOP
            0x01 => { self.ld_16(bus, Word::BC, Word::Immediate); 3 }, // ld BC, u16
            0x02 => { self.ld(bus, Byte::BC, Byte::A); 2 }, // ld (BC), A
            0x03 => { self.inc_16(bus, Word::BC); 2 }, // inc BC
            0x04 => { self.inc(bus, Byte::B); 1 }, // inc B
            0x05 => { self.dec(bus, Byte::B); 1 }, // dec B
            0x06 => { self.ld(bus, Byte::B, Byte::Immediate); 2 }, // ld B, u8
            0x07 => { self.rotate(bus, Byte::A, Direction::Left, true, false); 1 }, // rlca

            0x08 => { self.ld_16(bus, Word::Indirect, Word::SP); 5}, // ld (u16), SP
            0x09 => { self.add_16(bus, Word::BC); 2 }, // add HL, BC
            0x0A => { self.ld(bus, Byte::A, Byte::BC); 2 }, // ld A, (BC)
            0x0B => { self.dec_16(bus, Word::BC); 2 }, // dec BC
            0x0C => { self.inc(bus, Byte::C); 1 }, // inc C
            0x0D => { self.dec(bus, Byte::C); 1 }, // dec C
            0x0E => { self.ld(bus, Byte::C, Byte::Immediate); 2 }, // ld C, u8
            0x0F => { self.rotate(bus, Byte::A, Direction::Right, true, false); 1 }, // rrca

          //0x10 => { self.stopped = 1 }, // STOP
            0x11 => { self.ld_16(bus, Word::DE, Word::Immediate); 3 }, // ld DE, u16
            0x12 => { self.ld(bus, Byte::DE, Byte::A); 2 }, // ld (DE), A
            0x13 => { self.inc_16(bus, Word::DE); 2 }, // inc DE
            0x14 => { self.inc(bus, Byte::D); 1 }, // inc D
            0x15 => { self.dec(bus, Byte::D); 1 }, // dec D
            0x16 => { self.ld(bus, Byte::D, Byte::Immediate); 2 }, // ld D, u8
            0x17 => { self.rotate(bus, Byte::A, Direction::Left, false, false); 1 }, // rla

            0x18 => { self.jr(bus, None); 3}, // jr i8
            0x19 => { self.add_16(bus, Word::DE); 2 }, // add HL, DE
            0x1A => { self.ld(bus, Byte::A, Byte::DE); 2 }, // ld A, (DE)
            0x1B => { self.dec_16(bus, Word::DE); 2 }, // dec DE
            0x1C => { self.inc(bus, Byte::E); 1 }, // inc E
            0x1D => { self.dec(bus, Byte::E); 1 }, // dec E
            0x1E => { self.ld(bus, Byte::E, Byte::Immediate); 2 }, // ld C, u8
            0x1F => { self.rotate(bus, Byte::A, Direction::Right, false, false); 1 }, // rra

            0x20 => if self.jr(bus, Some((Flag::Zero, false))) { 3 } else { 2 }, // jr NZ, i8
            0x21 => { self.ld_16(bus, Word::HL, Word::Immediate); 3 }, // ld HL, u16
            0x22 => { self.ld(bus, Byte::HLi, Byte::A); 2 }, // ld (HL+), A
            0x23 => { self.inc_16(bus, Word::HL); 2 }, // inc HL
            0x24 => { self.inc(bus, Byte::H); 1 }, // inc H
            0x25 => { self.dec(bus, Byte::H); 1 }, // dec H
            0x26 => { self.ld(bus, Byte::H, Byte::Immediate); 2 }, // ld H, u8
            0x27 => { self.daa(); 1 }, // daa

            0x28 => if self.jr(bus, Some((Flag::Zero, true))) { 3 } else { 2 }, // jr Z, i8
            0x29 => { self.add_16(bus, Word::HL); 2 }, // add HL, HL
            0x2A => { self.ld(bus, Byte::A, Byte::HLi); 2 }, // ld A, (HL+)
            0x2B => { self.dec_16(bus, Word::HL); 2 }, // dec HL
            0x2C => { self.inc(bus, Byte::L); 1 }, // inc L
            0x2D => { self.dec(bus, Byte::L); 1 }, // dec L
            0x2E => { self.ld(bus, Byte::L, Byte::Immediate); 2 }, // ld L, u8
            0x2F => { self.cpl(); 1 }, // cpl

            0x30 => if self.jr(bus, Some((Flag::Carry, false))) { 3 } else { 2 }, // jr NC, i8
            0x31 => { self.ld_16(bus, Word::SP, Word::Immediate); 3 }, // ld SP, u16
            0x32 => { self.ld(bus, Byte::HLd, Byte::A); 2 }, // ld (HL-), A
            0x33 => { self.inc_16(bus, Word::SP); 2 }, // inc SP
            0x34 => { self.inc(bus, Byte::HL); 1 }, // inc (HL)
            0x35 => { self.dec(bus, Byte::HL); 1 }, // dec (HL)
            0x36 => { self.ld(bus, Byte::HL, Byte::Immediate); 2 }, // ld (HL), u8
            0x37 => { self.scf(); 1 }, // scf

            0x38 => if self.jr(bus, Some((Flag::Carry, true))) { 3 } else { 2 }, // jr C, i8
            0x39 => { self.add_16(bus, Word::SP); 2 }, // add HL, SP
            0x3A => { self.ld(bus, Byte::A, Byte::HLd); 2 }, // ld A, (HL-)
            0x3B => { self.dec_16(bus, Word::SP); 2 }, // dec SP
            0x3C => { self.inc(bus, Byte::A); 1 }, // inc A
            0x3D => { self.dec(bus, Byte::A); 1 }, // dec A
            0x3E => { self.ld(bus, Byte::A, Byte::Immediate); 2 }, // ld L, u8
            0x3F => { self.ccf(); 1 }, // ccf

            0x40 => { self.ld(bus, Byte::B, Byte::B); 1 },
            0x41 => { self.ld(bus, Byte::B, Byte::C); 1 },
            0x42 => { self.ld(bus, Byte::B, Byte::D); 1 },
            0x43 => { self.ld(bus, Byte::B, Byte::E); 1 },
            0x44 => { self.ld(bus, Byte::B, Byte::H); 1 },
            0x45 => { self.ld(bus, Byte::B, Byte::L); 1 },
            0x46 => { self.ld(bus, Byte::B, Byte::HL); 2 },
            0x47 => { self.ld(bus, Byte::B, Byte::A); 1 },

            0x48 => { self.ld(bus, Byte::C, Byte::B); 1 },
            0x49 => { self.ld(bus, Byte::C, Byte::C); 1 },
            0x4A => { self.ld(bus, Byte::C, Byte::D); 1 },
            0x4B => { self.ld(bus, Byte::C, Byte::E); 1 },
            0x4C => { self.ld(bus, Byte::C, Byte::H); 1 },
            0x4D => { self.ld(bus, Byte::C, Byte::L); 1 },
            0x4E => { self.ld(bus, Byte::C, Byte::HL); 2 },
            0x4F => { self.ld(bus, Byte::C, Byte::A); 1 },

            0x50 => { self.ld(bus, Byte::D, Byte::B); 1 },
            0x51 => { self.ld(bus, Byte::D, Byte::C); 1 },
            0x52 => { self.ld(bus, Byte::D, Byte::D); 1 },
            0x53 => { self.ld(bus, Byte::D, Byte::E); 1 },
            0x54 => { self.ld(bus, Byte::D, Byte::H); 1 },
            0x55 => { self.ld(bus, Byte::D, Byte::L); 1 },
            0x56 => { self.ld(bus, Byte::D, Byte::HL); 2 },
            0x57 => { self.ld(bus, Byte::D, Byte::A); 1 },

            0x58 => { self.ld(bus, Byte::E, Byte::B); 1 },
            0x59 => { self.ld(bus, Byte::E, Byte::C); 1 },
            0x5A => { self.ld(bus, Byte::E, Byte::D); 1 },
            0x5B => { self.ld(bus, Byte::E, Byte::E); 1 },
            0x5C => { self.ld(bus, Byte::E, Byte::H); 1 },
            0x5D => { self.ld(bus, Byte::E, Byte::L); 1 },
            0x5E => { self.ld(bus, Byte::E, Byte::HL); 2 },
            0x5F => { self.ld(bus, Byte::E, Byte::A); 1 },

            0x60 => { self.ld(bus, Byte::H, Byte::B); 1 },
            0x61 => { self.ld(bus, Byte::H, Byte::C); 1 },
            0x62 => { self.ld(bus, Byte::H, Byte::D); 1 },
            0x63 => { self.ld(bus, Byte::H, Byte::E); 1 },
            0x64 => { self.ld(bus, Byte::H, Byte::H); 1 },
            0x65 => { self.ld(bus, Byte::H, Byte::L); 1 },
            0x66 => { self.ld(bus, Byte::H, Byte::HL); 2 },
            0x67 => { self.ld(bus, Byte::H, Byte::A); 1 },

            0x68 => { self.ld(bus, Byte::L, Byte::B); 1 },
            0x69 => { self.ld(bus, Byte::L, Byte::C); 1 },
            0x6A => { self.ld(bus, Byte::L, Byte::D); 1 },
            0x6B => { self.ld(bus, Byte::L, Byte::E); 1 },
            0x6C => { self.ld(bus, Byte::L, Byte::H); 1 },
            0x6D => { self.ld(bus, Byte::L, Byte::L); 1 },
            0x6E => { self.ld(bus, Byte::L, Byte::HL); 2 },
            0x6F => { self.ld(bus, Byte::L, Byte::A); 1 },

            0x70 => { self.ld(bus, Byte::HL, Byte::B); 1 },
            0x71 => { self.ld(bus, Byte::HL, Byte::C); 1 },
            0x72 => { self.ld(bus, Byte::HL, Byte::D); 1 },
            0x73 => { self.ld(bus, Byte::HL, Byte::E); 1 },
            0x74 => { self.ld(bus, Byte::HL, Byte::H); 1 },
            0x75 => { self.ld(bus, Byte::HL, Byte::L); 1 },
            0x76 => { self.halted = true; 1 },
            0x77 => { self.ld(bus, Byte::HL, Byte::A); 1 },

            0x78 => { self.ld(bus, Byte::A, Byte::B); 1 },
            0x79 => { self.ld(bus, Byte::A, Byte::C); 1 },
            0x7A => { self.ld(bus, Byte::A, Byte::D); 1 },
            0x7B => { self.ld(bus, Byte::A, Byte::E); 1 },
            0x7C => { self.ld(bus, Byte::A, Byte::H); 1 },
            0x7D => { self.ld(bus, Byte::A, Byte::L); 1 },
            0x7E => { self.ld(bus, Byte::A, Byte::HL); 2 },
            0x7F => { self.ld(bus, Byte::A, Byte::A); 1 },

            0x80 => { self.add(bus, Byte::B, false); 1 },
            0x81 => { self.add(bus, Byte::C, false); 1 },
            0x82 => { self.add(bus, Byte::D, false); 1 },
            0x83 => { self.add(bus, Byte::E, false); 1 },
            0x84 => { self.add(bus, Byte::H, false); 1 },
            0x85 => { self.add(bus, Byte::L, false); 1 },
            0x86 => { self.add(bus, Byte::HL, false); 2 },
            0x87 => { self.add(bus, Byte::A, false); 1 }, // Flags 1100

            0x88 => { self.add(bus, Byte::B, true); 1 },
            0x89 => { self.add(bus, Byte::C, true); 1 },
            0x8A => { self.add(bus, Byte::D, true); 1 },
            0x8B => { self.add(bus, Byte::E, true); 1 },
            0x8C => { self.add(bus, Byte::H, true); 1 },
            0x8D => { self.add(bus, Byte::L, true); 1 },
            0x8E => { self.add(bus, Byte::HL, true); 2 },
            0x8F => { self.add(bus, Byte::A, true); 1 },

            0x90 => { self.sub(bus, Byte::B, false); 1 },
            0x91 => { self.sub(bus, Byte::C, false); 1 },
            0x92 => { self.sub(bus, Byte::D, false); 1 },
            0x93 => { self.sub(bus, Byte::E, false); 1 },
            0x94 => { self.sub(bus, Byte::H, false); 1 },
            0x95 => { self.sub(bus, Byte::L, false); 1 },
            0x96 => { self.sub(bus, Byte::HL, false); 2 },
            0x97 => { self.sub(bus, Byte::A, false); 1 },

            0x98 => { self.sub(bus, Byte::B, true); 1 },
            0x99 => { self.sub(bus, Byte::C, true); 1 },
            0x9A => { self.sub(bus, Byte::D, true); 1 },
            0x9B => { self.sub(bus, Byte::E, true); 1 },
            0x9C => { self.sub(bus, Byte::H, true); 1 },
            0x9D => { self.sub(bus, Byte::L, true); 1 },
            0x9E => { self.sub(bus, Byte::HL, true); 2 },
            0x9F => { self.sub(bus, Byte::A, true); 1 },

            0xA0 => { self.and(bus, Byte::B); 1 },
            0xA1 => { self.and(bus, Byte::C); 1 },
            0xA2 => { self.and(bus, Byte::D); 1 },
            0xA3 => { self.and(bus, Byte::E); 1 },
            0xA4 => { self.and(bus, Byte::H); 1 },
            0xA5 => { self.and(bus, Byte::L); 1 },
            0xA6 => { self.and(bus, Byte::HL); 2 },
            0xA7 => { self.and(bus, Byte::A); 1 },

            0xA8 => { self.xor(bus, Byte::B); 1 },
            0xA9 => { self.xor(bus, Byte::C); 1 },
            0xAA => { self.xor(bus, Byte::D); 1 },
            0xAB => { self.xor(bus, Byte::E); 1 },
            0xAC => { self.xor(bus, Byte::H); 1 },
            0xAD => { self.xor(bus, Byte::L); 1 },
            0xAE => { self.xor(bus, Byte::HL); 2 },
            0xAF => { self.xor(bus, Byte::A); 1 },

            0xB0 => { self.or(bus, Byte::B); 1 },
            0xB1 => { self.or(bus, Byte::C); 1 },
            0xB2 => { self.or(bus, Byte::D); 1 },
            0xB3 => { self.or(bus, Byte::E); 1 },
            0xB4 => { self.or(bus, Byte::H); 1 },
            0xB5 => { self.or(bus, Byte::L); 1 },
            0xB6 => { self.or(bus, Byte::HL); 2 },
            0xB7 => { self.or(bus, Byte::A); 1 },

            0xB8 => { self.cp(bus, Byte::B); 1 },
            0xB9 => { self.cp(bus, Byte::C); 1 },
            0xBA => { self.cp(bus, Byte::D); 1 },
            0xBB => { self.cp(bus, Byte::E); 1 },
            0xBC => { self.cp(bus, Byte::H); 1 },
            0xBD => { self.cp(bus, Byte::L); 1 },
            0xBE => { self.cp(bus, Byte::HL); 2 },
            0xBF => { self.cp(bus, Byte::A); 1 },

            0xC0 => if self.ret(bus, Some((Flag::Zero, false))) { 4 } else { 2 },
            0xC1 => { self.pop(bus, Word::BC); 3 },
            0xC2 => if self.jp(bus, Some((Flag::Zero, false))) { 4 } else { 3 },
            0xC3 => { self.jp(bus, None); 4 },
            0xC4 => if self.call(bus, Some((Flag::Zero, false))) { 6 } else { 3 },
            0xC5 => { self.push(bus, Word::BC); 4 },
            0xC6 => { self.add(bus, Byte::Immediate, false); 2 },
            0xC7 => { self.rst(bus, 0x00); 4 },

            0xC8 => if self.ret(bus, Some((Flag::Zero, true))) { 4 } else { 2 },
            0xC9 => { self.ret(bus, None); 4 },
            0xCA => if self.jp(bus, Some((Flag::Zero, true))) { 4 } else { 3 },
            0xCB => self.cb(bus) + 1,
            0xCC => if self.call(bus, Some((Flag::Zero, true))) { 6 } else { 3 },
            0xCD => { self.call(bus, None); 6 },
            0xCE => { self.add(bus, Byte::Immediate, true); 2 },
            0xCF => { self.rst(bus, 0x08); 4 },

            0xD0 => if self.ret(bus, Some((Flag::Carry, false))) { 4 } else { 2 },
            0xD1 => { self.pop(bus, Word::DE); 3 },
            0xD2 => if self.jp(bus, Some((Flag::Carry, false))) { 4 } else { 3 },
         // 0xD3 => Illegal
            0xD4 => if self.call(bus, Some((Flag::Carry, false))) { 6 } else { 3 },
            0xD5 => { self.push(bus, Word::DE); 4 },
            0xD6 => { self.sub(bus, Byte::Immediate, false); 2 },
            0xD7 => { self.rst(bus, 0x10); 4 },

            0xD8 => if self.ret(bus, Some((Flag::Carry, true))) { 4 } else { 2 },
            0xD9 => { self.ret(bus, None); 4 },
            0xDA => if self.jp(bus, Some((Flag::Carry, true))) { 4 } else { 3 },
         // 0xDB => Illegal
            0xDC => if self.call(bus, Some((Flag::Carry, true))) { 6 } else { 3 },
         // 0xDD => Illegal
            0xDE => { self.sub(bus, Byte::Immediate, true); 2 },
            0xDF => { self.rst(bus, 0x18); 4 },

            0xE0 => { self.ld(bus, Byte::HighAddress, Byte::A); 3 }
            0xE1 => { self.pop(bus, Word::HL); 3 },
            0xE2 => { self.ld(bus, Byte::HighAddressC, Byte::A); 2 }
         // 0xE3 => Illegal
         // 0xE4 => Illegal
            0xE5 => { self.push(bus, Word::HL); 4 },
            0xE6 => { self.and(bus, Byte::Immediate); 2 },
            0xE7 => { self.rst(bus, 0x20); 4 },

            0xE8 => { self.add_sp(bus); 4 }
            0xE9 => { self.pc = self.hl.value(); 1 },
            0xEA => { self.ld(bus, Byte::Indirect, Byte::A); 4 },
         // 0xEB => Illegal
         // 0xEC => Illegal
         // 0xED => Illegal
            0xEE => { self.xor(bus, Byte::Immediate); 2 },
            0xEF => { self.rst(bus, 0x28); 4 },

            0xF0 => { self.ld(bus, Byte::A, Byte::HighAddress); 3 }
            0xF1 => { self.pop(bus, Word::AF); 3 },
            0xF2 => { self.ld(bus, Byte::A, Byte::HighAddressC); 2 }
            0xF3 => { self.interrupt_master_enable = false; 1 },
         // 0xF4 => Illegal
            0xF5 => { self.push(bus, Word::AF); 4 },
            0xF6 => { self.or(bus, Byte::Immediate); 2 },
            0xF7 => { self.rst(bus, 0x20); 4 },

            0xF8 => { self.ld_sp(bus); 3 }
            0xF9 => { self.ld_16(bus, Word::SP, Word::HL); 2 }
            0xFA => { self.ld(bus, Byte::A, Byte::Indirect); 4 },
            0xFB => { self.interrupt_master_enable = true; 1 },
         // 0xFC => Illegal
         // 0xFD => Illegal
            0xFE => { self.cp(bus, Byte::Immediate); 2 },
            0xFF => { self.rst(bus, 0x38); 4 },

            _ => panic!("Illegal Instruction ({:02X})", opcode)
        }
    }

    fn cb(&mut self, bus: &mut Bus) -> u8 {
        let opcode = self.read_byte(bus, ByteDescriptor::Immediate);
        // println!("PC {:04X} CB {:02X}", self.pc - 1, opcode);

        type Byte = ByteDescriptor;
        type Word = WordDescriptor;

        // TODO: Write a macro that to reduce repetition
        match opcode {
            0x00 => { self.rotate(bus, Byte::B,  Direction::Left, true, true); 2 },
            0x01 => { self.rotate(bus, Byte::C,  Direction::Left, true, true); 2 },
            0x02 => { self.rotate(bus, Byte::D,  Direction::Left, true, true); 2 },
            0x03 => { self.rotate(bus, Byte::E,  Direction::Left, true, true); 2 },
            0x04 => { self.rotate(bus, Byte::H,  Direction::Left, true, true); 2 },
            0x05 => { self.rotate(bus, Byte::L,  Direction::Left, true, true); 2 },
            0x06 => { self.rotate(bus, Byte::HL, Direction::Left, true, true); 4 },
            0x07 => { self.rotate(bus, Byte::A,  Direction::Left, true, true); 2 },

            0x08 => { self.rotate(bus, Byte::B,  Direction::Right, true, true); 2 },
            0x09 => { self.rotate(bus, Byte::C,  Direction::Right, true, true); 2 },
            0x0A => { self.rotate(bus, Byte::D,  Direction::Right, true, true); 2 },
            0x0B => { self.rotate(bus, Byte::E,  Direction::Right, true, true); 2 },
            0x0C => { self.rotate(bus, Byte::H,  Direction::Right, true, true); 2 },
            0x0D => { self.rotate(bus, Byte::L,  Direction::Right, true, true); 2 },
            0x0E => { self.rotate(bus, Byte::HL, Direction::Right, true, true); 4 },
            0x0F => { self.rotate(bus, Byte::A,  Direction::Right, true, true); 2 },

            0x10 => { self.rotate(bus, Byte::B,  Direction::Left, false, true); 2 },
            0x11 => { self.rotate(bus, Byte::C,  Direction::Left, false, true); 2 },
            0x12 => { self.rotate(bus, Byte::D,  Direction::Left, false, true); 2 },
            0x13 => { self.rotate(bus, Byte::E,  Direction::Left, false, true); 2 },
            0x14 => { self.rotate(bus, Byte::H,  Direction::Left, false, true); 2 },
            0x15 => { self.rotate(bus, Byte::L,  Direction::Left, false, true); 2 },
            0x16 => { self.rotate(bus, Byte::HL, Direction::Left, false, true); 4 },
            0x17 => { self.rotate(bus, Byte::A,  Direction::Left, false, true); 2 },

            0x18 => { self.rotate(bus, Byte::B,  Direction::Right, false, true); 2 },
            0x19 => { self.rotate(bus, Byte::C,  Direction::Right, false, true); 2 },
            0x1A => { self.rotate(bus, Byte::D,  Direction::Right, false, true); 2 },
            0x1B => { self.rotate(bus, Byte::E,  Direction::Right, false, true); 2 },
            0x1C => { self.rotate(bus, Byte::H,  Direction::Right, false, true); 2 },
            0x1D => { self.rotate(bus, Byte::L,  Direction::Right, false, true); 2 },
            0x1E => { self.rotate(bus, Byte::HL, Direction::Right, false, true); 4 },
            0x1F => { self.rotate(bus, Byte::A,  Direction::Right, false, true); 2 },

            0x20 => { self.shift(bus, Byte::B,  Direction::Left, false); 2 },
            0x21 => { self.shift(bus, Byte::C,  Direction::Left, false); 2 },
            0x22 => { self.shift(bus, Byte::D,  Direction::Left, false); 2 },
            0x23 => { self.shift(bus, Byte::E,  Direction::Left, false); 2 },
            0x24 => { self.shift(bus, Byte::H,  Direction::Left, false); 2 },
            0x25 => { self.shift(bus, Byte::L,  Direction::Left, false); 2 },
            0x26 => { self.shift(bus, Byte::HL, Direction::Left, false); 4 },
            0x27 => { self.shift(bus, Byte::A,  Direction::Left, false); 2 },

            0x28 => { self.shift(bus, Byte::B,  Direction::Right, false); 2 },
            0x29 => { self.shift(bus, Byte::C,  Direction::Right, false); 2 },
            0x2A => { self.shift(bus, Byte::D,  Direction::Right, false); 2 },
            0x2B => { self.shift(bus, Byte::E,  Direction::Right, false); 2 },
            0x2C => { self.shift(bus, Byte::H,  Direction::Right, false); 2 },
            0x2D => { self.shift(bus, Byte::L,  Direction::Right, false); 2 },
            0x2E => { self.shift(bus, Byte::HL, Direction::Right, false); 4 },
            0x2F => { self.shift(bus, Byte::A,  Direction::Right, false); 2 },

            0x30 => { self.swap(bus, Byte::B);  2 },
            0x31 => { self.swap(bus, Byte::C);  2 },
            0x32 => { self.swap(bus, Byte::D);  2 },
            0x33 => { self.swap(bus, Byte::E);  2 },
            0x34 => { self.swap(bus, Byte::H);  2 },
            0x35 => { self.swap(bus, Byte::L);  2 },
            0x36 => { self.swap(bus, Byte::HL); 4 },
            0x37 => { self.swap(bus, Byte::A);  2 },

            0x38 => { self.shift(bus, Byte::B,  Direction::Right, true); 2 },
            0x39 => { self.shift(bus, Byte::C,  Direction::Right, true); 2 },
            0x3A => { self.shift(bus, Byte::D,  Direction::Right, true); 2 },
            0x3B => { self.shift(bus, Byte::E,  Direction::Right, true); 2 },
            0x3C => { self.shift(bus, Byte::H,  Direction::Right, true); 2 },
            0x3D => { self.shift(bus, Byte::L,  Direction::Right, true); 2 },
            0x3E => { self.shift(bus, Byte::HL, Direction::Right, true); 4 },
            0x3F => { self.shift(bus, Byte::A,  Direction::Right, true); 2 },

            0x40 => { self.test_bit(bus, Byte::B,  0); 2 },
            0x41 => { self.test_bit(bus, Byte::C,  0); 2 },
            0x42 => { self.test_bit(bus, Byte::D,  0); 2 },
            0x43 => { self.test_bit(bus, Byte::E,  0); 2 },
            0x44 => { self.test_bit(bus, Byte::H,  0); 2 },
            0x45 => { self.test_bit(bus, Byte::L,  0); 2 },
            0x46 => { self.test_bit(bus, Byte::HL, 0); 3 },
            0x47 => { self.test_bit(bus, Byte::A,  0); 2 },
            0x48 => { self.test_bit(bus, Byte::B,  1); 2 },
            0x49 => { self.test_bit(bus, Byte::C,  1); 2 },
            0x4A => { self.test_bit(bus, Byte::D,  1); 2 },
            0x4B => { self.test_bit(bus, Byte::E,  1); 2 },
            0x4C => { self.test_bit(bus, Byte::H,  1); 2 },
            0x4D => { self.test_bit(bus, Byte::L,  1); 2 },
            0x4E => { self.test_bit(bus, Byte::HL, 1); 3 },
            0x4F => { self.test_bit(bus, Byte::A,  1); 2 },
            0x50 => { self.test_bit(bus, Byte::B,  2); 2 },
            0x51 => { self.test_bit(bus, Byte::C,  2); 2 },
            0x52 => { self.test_bit(bus, Byte::D,  2); 2 },
            0x53 => { self.test_bit(bus, Byte::E,  2); 2 },
            0x54 => { self.test_bit(bus, Byte::H,  2); 2 },
            0x55 => { self.test_bit(bus, Byte::L,  2); 2 },
            0x56 => { self.test_bit(bus, Byte::HL, 2); 3 },
            0x57 => { self.test_bit(bus, Byte::A,  2); 2 },
            0x58 => { self.test_bit(bus, Byte::B,  3); 2 },
            0x59 => { self.test_bit(bus, Byte::C,  3); 2 },
            0x5A => { self.test_bit(bus, Byte::D,  3); 2 },
            0x5B => { self.test_bit(bus, Byte::E,  3); 2 },
            0x5C => { self.test_bit(bus, Byte::H,  3); 2 },
            0x5D => { self.test_bit(bus, Byte::L,  3); 2 },
            0x5E => { self.test_bit(bus, Byte::HL, 3); 3 },
            0x5F => { self.test_bit(bus, Byte::A,  3); 2 },
            0x60 => { self.test_bit(bus, Byte::B,  4); 2 },
            0x61 => { self.test_bit(bus, Byte::C,  4); 2 },
            0x62 => { self.test_bit(bus, Byte::D,  4); 2 },
            0x63 => { self.test_bit(bus, Byte::E,  4); 2 },
            0x64 => { self.test_bit(bus, Byte::H,  4); 2 },
            0x65 => { self.test_bit(bus, Byte::L,  4); 2 },
            0x66 => { self.test_bit(bus, Byte::HL, 4); 3 },
            0x67 => { self.test_bit(bus, Byte::A,  4); 2 },
            0x68 => { self.test_bit(bus, Byte::B,  5); 2 },
            0x69 => { self.test_bit(bus, Byte::C,  5); 2 },
            0x6A => { self.test_bit(bus, Byte::D,  5); 2 },
            0x6B => { self.test_bit(bus, Byte::E,  5); 2 },
            0x6C => { self.test_bit(bus, Byte::H,  5); 2 },
            0x6D => { self.test_bit(bus, Byte::L,  5); 2 },
            0x6E => { self.test_bit(bus, Byte::HL, 5); 3 },
            0x6F => { self.test_bit(bus, Byte::A,  5); 2 },
            0x70 => { self.test_bit(bus, Byte::B,  6); 2 },
            0x71 => { self.test_bit(bus, Byte::C,  6); 2 },
            0x72 => { self.test_bit(bus, Byte::D,  6); 2 },
            0x73 => { self.test_bit(bus, Byte::E,  6); 2 },
            0x74 => { self.test_bit(bus, Byte::H,  6); 2 },
            0x75 => { self.test_bit(bus, Byte::L,  6); 2 },
            0x76 => { self.test_bit(bus, Byte::HL, 6); 3 },
            0x77 => { self.test_bit(bus, Byte::A,  6); 2 },
            0x78 => { self.test_bit(bus, Byte::B,  7); 2 },
            0x79 => { self.test_bit(bus, Byte::C,  7); 2 },
            0x7A => { self.test_bit(bus, Byte::D,  7); 2 },
            0x7B => { self.test_bit(bus, Byte::E,  7); 2 },
            0x7C => { self.test_bit(bus, Byte::H,  7); 2 },
            0x7D => { self.test_bit(bus, Byte::L,  7); 2 },
            0x7E => { self.test_bit(bus, Byte::HL, 7); 3 },
            0x7F => { self.test_bit(bus, Byte::A,  7); 2 },

            0x80 => { self.clear_bit(bus, Byte::B,  0); 2 },
            0x81 => { self.clear_bit(bus, Byte::C,  0); 2 },
            0x82 => { self.clear_bit(bus, Byte::D,  0); 2 },
            0x83 => { self.clear_bit(bus, Byte::E,  0); 2 },
            0x84 => { self.clear_bit(bus, Byte::H,  0); 2 },
            0x85 => { self.clear_bit(bus, Byte::L,  0); 2 },
            0x86 => { self.clear_bit(bus, Byte::HL, 0); 4 },
            0x87 => { self.clear_bit(bus, Byte::A,  0); 2 },
            0x88 => { self.clear_bit(bus, Byte::B,  1); 2 },
            0x89 => { self.clear_bit(bus, Byte::C,  1); 2 },
            0x8A => { self.clear_bit(bus, Byte::D,  1); 2 },
            0x8B => { self.clear_bit(bus, Byte::E,  1); 2 },
            0x8C => { self.clear_bit(bus, Byte::H,  1); 2 },
            0x8D => { self.clear_bit(bus, Byte::L,  1); 2 },
            0x8E => { self.clear_bit(bus, Byte::HL, 1); 4 },
            0x8F => { self.clear_bit(bus, Byte::A,  1); 2 },
            0x90 => { self.clear_bit(bus, Byte::B,  2); 2 },
            0x91 => { self.clear_bit(bus, Byte::C,  2); 2 },
            0x92 => { self.clear_bit(bus, Byte::D,  2); 2 },
            0x93 => { self.clear_bit(bus, Byte::E,  2); 2 },
            0x94 => { self.clear_bit(bus, Byte::H,  2); 2 },
            0x95 => { self.clear_bit(bus, Byte::L,  2); 2 },
            0x96 => { self.clear_bit(bus, Byte::HL, 2); 4 },
            0x97 => { self.clear_bit(bus, Byte::A,  2); 2 },
            0x98 => { self.clear_bit(bus, Byte::B,  3); 2 },
            0x99 => { self.clear_bit(bus, Byte::C,  3); 2 },
            0x9A => { self.clear_bit(bus, Byte::D,  3); 2 },
            0x9B => { self.clear_bit(bus, Byte::E,  3); 2 },
            0x9C => { self.clear_bit(bus, Byte::H,  3); 2 },
            0x9D => { self.clear_bit(bus, Byte::L,  3); 2 },
            0x9E => { self.clear_bit(bus, Byte::HL, 3); 4 },
            0x9F => { self.clear_bit(bus, Byte::A,  3); 2 },
            0xA0 => { self.clear_bit(bus, Byte::B,  4); 2 },
            0xA1 => { self.clear_bit(bus, Byte::C,  4); 2 },
            0xA2 => { self.clear_bit(bus, Byte::D,  4); 2 },
            0xA3 => { self.clear_bit(bus, Byte::E,  4); 2 },
            0xA4 => { self.clear_bit(bus, Byte::H,  4); 2 },
            0xA5 => { self.clear_bit(bus, Byte::L,  4); 2 },
            0xA6 => { self.clear_bit(bus, Byte::HL, 4); 4 },
            0xA7 => { self.clear_bit(bus, Byte::A,  4); 2 },
            0xA8 => { self.clear_bit(bus, Byte::B,  5); 2 },
            0xA9 => { self.clear_bit(bus, Byte::C,  5); 2 },
            0xAA => { self.clear_bit(bus, Byte::D,  5); 2 },
            0xAB => { self.clear_bit(bus, Byte::E,  5); 2 },
            0xAC => { self.clear_bit(bus, Byte::H,  5); 2 },
            0xAD => { self.clear_bit(bus, Byte::L,  5); 2 },
            0xAE => { self.clear_bit(bus, Byte::HL, 5); 4 },
            0xAF => { self.clear_bit(bus, Byte::A,  5); 2 },
            0xB0 => { self.clear_bit(bus, Byte::B,  6); 2 },
            0xB1 => { self.clear_bit(bus, Byte::C,  6); 2 },
            0xB2 => { self.clear_bit(bus, Byte::D,  6); 2 },
            0xB3 => { self.clear_bit(bus, Byte::E,  6); 2 },
            0xB4 => { self.clear_bit(bus, Byte::H,  6); 2 },
            0xB5 => { self.clear_bit(bus, Byte::L,  6); 2 },
            0xB6 => { self.clear_bit(bus, Byte::HL, 6); 4 },
            0xB7 => { self.clear_bit(bus, Byte::A,  6); 2 },
            0xB8 => { self.clear_bit(bus, Byte::B,  7); 2 },
            0xB9 => { self.clear_bit(bus, Byte::C,  7); 2 },
            0xBA => { self.clear_bit(bus, Byte::D,  7); 2 },
            0xBB => { self.clear_bit(bus, Byte::E,  7); 2 },
            0xBC => { self.clear_bit(bus, Byte::H,  7); 2 },
            0xBD => { self.clear_bit(bus, Byte::L,  7); 2 },
            0xBE => { self.clear_bit(bus, Byte::HL, 7); 4 },
            0xBF => { self.clear_bit(bus, Byte::A,  7); 2 },

            0xC0 => { self.set_bit(bus, Byte::B,  0); 2 },
            0xC1 => { self.set_bit(bus, Byte::C,  0); 2 },
            0xC2 => { self.set_bit(bus, Byte::D,  0); 2 },
            0xC3 => { self.set_bit(bus, Byte::E,  0); 2 },
            0xC4 => { self.set_bit(bus, Byte::H,  0); 2 },
            0xC5 => { self.set_bit(bus, Byte::L,  0); 2 },
            0xC6 => { self.set_bit(bus, Byte::HL, 0); 4 },
            0xC7 => { self.set_bit(bus, Byte::A,  0); 2 },
            0xC8 => { self.set_bit(bus, Byte::B,  1); 2 },
            0xC9 => { self.set_bit(bus, Byte::C,  1); 2 },
            0xCA => { self.set_bit(bus, Byte::D,  1); 2 },
            0xCB => { self.set_bit(bus, Byte::E,  1); 2 },
            0xCC => { self.set_bit(bus, Byte::H,  1); 2 },
            0xCD => { self.set_bit(bus, Byte::L,  1); 2 },
            0xCE => { self.set_bit(bus, Byte::HL, 1); 4 },
            0xCF => { self.set_bit(bus, Byte::A,  1); 2 },
            0xD0 => { self.set_bit(bus, Byte::B,  2); 2 },
            0xD1 => { self.set_bit(bus, Byte::C,  2); 2 },
            0xD2 => { self.set_bit(bus, Byte::D,  2); 2 },
            0xD3 => { self.set_bit(bus, Byte::E,  2); 2 },
            0xD4 => { self.set_bit(bus, Byte::H,  2); 2 },
            0xD5 => { self.set_bit(bus, Byte::L,  2); 2 },
            0xD6 => { self.set_bit(bus, Byte::HL, 2); 4 },
            0xD7 => { self.set_bit(bus, Byte::A,  2); 2 },
            0xD8 => { self.set_bit(bus, Byte::B,  3); 2 },
            0xD9 => { self.set_bit(bus, Byte::C,  3); 2 },
            0xDA => { self.set_bit(bus, Byte::D,  3); 2 },
            0xDB => { self.set_bit(bus, Byte::E,  3); 2 },
            0xDC => { self.set_bit(bus, Byte::H,  3); 2 },
            0xDD => { self.set_bit(bus, Byte::L,  3); 2 },
            0xDE => { self.set_bit(bus, Byte::HL, 3); 4 },
            0xDF => { self.set_bit(bus, Byte::A,  3); 2 },
            0xE0 => { self.set_bit(bus, Byte::B,  4); 2 },
            0xE1 => { self.set_bit(bus, Byte::C,  4); 2 },
            0xE2 => { self.set_bit(bus, Byte::D,  4); 2 },
            0xE3 => { self.set_bit(bus, Byte::E,  4); 2 },
            0xE4 => { self.set_bit(bus, Byte::H,  4); 2 },
            0xE5 => { self.set_bit(bus, Byte::L,  4); 2 },
            0xE6 => { self.set_bit(bus, Byte::HL, 4); 4 },
            0xE7 => { self.set_bit(bus, Byte::A,  4); 2 },
            0xE8 => { self.set_bit(bus, Byte::B,  5); 2 },
            0xE9 => { self.set_bit(bus, Byte::C,  5); 2 },
            0xEA => { self.set_bit(bus, Byte::D,  5); 2 },
            0xEB => { self.set_bit(bus, Byte::E,  5); 2 },
            0xEC => { self.set_bit(bus, Byte::H,  5); 2 },
            0xED => { self.set_bit(bus, Byte::L,  5); 2 },
            0xEE => { self.set_bit(bus, Byte::HL, 5); 4 },
            0xEF => { self.set_bit(bus, Byte::A,  5); 2 },
            0xF0 => { self.set_bit(bus, Byte::B,  6); 2 },
            0xF1 => { self.set_bit(bus, Byte::C,  6); 2 },
            0xF2 => { self.set_bit(bus, Byte::D,  6); 2 },
            0xF3 => { self.set_bit(bus, Byte::E,  6); 2 },
            0xF4 => { self.set_bit(bus, Byte::H,  6); 2 },
            0xF5 => { self.set_bit(bus, Byte::L,  6); 2 },
            0xF6 => { self.set_bit(bus, Byte::HL, 6); 4 },
            0xF7 => { self.set_bit(bus, Byte::A,  6); 2 },
            0xF8 => { self.set_bit(bus, Byte::B,  7); 2 },
            0xF9 => { self.set_bit(bus, Byte::C,  7); 2 },
            0xFA => { self.set_bit(bus, Byte::D,  7); 2 },
            0xFB => { self.set_bit(bus, Byte::E,  7); 2 },
            0xFC => { self.set_bit(bus, Byte::H,  7); 2 },
            0xFD => { self.set_bit(bus, Byte::L,  7); 2 },
            0xFE => { self.set_bit(bus, Byte::HL, 7); 4 },
            0xFF => { self.set_bit(bus, Byte::A,  7); 2 },

            _ => panic!("Illegal Instruction ({:02X})", opcode)
        }
    }
}