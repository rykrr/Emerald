#![deny(elided_lifetimes_in_paths)]
//#![feature(exclusive_range_pattern)]
#![allow(non_snake_case)]
#![allow(unused_variables)]

mod bus;
use crate::bus::*;

mod timer;
use crate::timer::*;

mod cpu;
use crate::cpu::*;

mod ppu;
use crate::ppu::*;

mod graphics;

mod cartridge;

mod debug;
mod serial;
mod clock;

mod joypad;
mod ram;

use crate::debug::Debugger;
use crate::joypad::*;
use crate::clock::Clock;
use crate::graphics::*;
use crate::minifb_driver::MiniFbDriver;
use crate::cartridge::load;
use crate::graphics_driver::GraphicsDriver;
use crate::InterruptType::Joypad;
use crate::ram::{DummyRAM, RAM, RegisterHoles};

use std::borrow::BorrowMut;

use std::env;
use std::cell::RefCell;
use std::rc::Rc;

fn rc<T>(t: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(t))
}


struct Options {
    enable_debugger: bool,
    enable_trace: bool,
    enable_serial: bool,
    cartridge_path: String,
}

fn parse_args() -> Options {
    let args: Vec<String> = env::args().collect();

    let mut options = Options {
        enable_debugger: false,
        enable_trace: false,
        enable_serial: false,
        cartridge_path: args.last().expect("Expected a file path.").clone(),
    };

    for arg in &args {
        match arg.as_str() {
            "-d" | "--enable-debugger" => options.enable_debugger = true,
            "-t" | "--enable-trace" => options.enable_trace = true,
            "-s" | "--enable-serial" => options.enable_serial = true,
            _ => {},
        }
    }

    options
}


fn main() {
    env_logger::init();

    let options = parse_args();

    let mut minifb_driver = MiniFbDriver::new(
        DISPLAY_WIDTH as u16,
        DISPLAY_HEIGHT as u16,
        minifb::Scale::X4,
    );

    // The bus handles the "wiring" between all components.
    let mut bus = Bus::new();
    let mut clk = Clock::new();

    let mut cpu = CPU::new();
    cpu.attach_to_bus(&mut bus);

    // Graphics processor.
    let ppu = rc(PPU::new());
    bus.attach(ppu.clone());
    clk.attach(ppu.clone());

    let timer = rc(Timer::new());
    bus.attach(timer.clone());
    clk.attach(timer.clone());

    let cartridge = load(options.cartridge_path.as_str());
    bus.attach(cartridge.clone());

    let joypad = rc(joypad::Joypad::new());
    bus.attach(joypad.clone());

    let audio = rc(DummyRAM::new(0x10, 0x3F, true));
    bus.attach(audio.clone());

    let serial = rc(serial::SerialInterface::new(!options.enable_serial));
    bus.attach(serial.clone());

    let ram = rc(RAM::new());
    bus.attach(ram.clone());

    let unused_registers = rc(RegisterHoles::new());
    bus.attach(unused_registers.clone());

    let hram = rc(DummyRAM::new(0xFF, 0xFF, false));
    bus.attach(hram.clone());

    let mut debugger = Debugger::new(options.enable_debugger);

    // MAIN LOOP //

    if options.enable_trace {
        cpu.print_trace(&bus);
    }

    while !cpu.is_halted() && !minifb_driver.is_closed() {
        // if options.enable_debugger {
        {
            let ppu = &*ppu.as_ref().borrow();
            debugger.step(&mut bus, &mut cpu, ppu);

            if debugger.has_quit() {
                return;
            }
        }

        clk.cycle_start();
        let cycles = cpu.step(&mut bus);

        if options.enable_trace {
            cpu.print_trace(&bus);
        }

        clk.increment(&mut bus, cycles);
        // clk.cycle_end();

        ppu.as_ref()
           .borrow_mut()
           .update(&mut minifb_driver);

        joypad.as_ref()
              .borrow_mut()
              .update(&mut minifb_driver, &mut debugger);
    }
    // END LOOP //
}
