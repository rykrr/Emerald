use std::collections::HashSet;
use std::io;
use std::io::Write;
use std::num::ParseIntError;
use log::info;
use crate::{Address, Bus, CPU, PPU};

pub struct Debugger {
    step: bool,
    step_on_breakpoint: bool,
    breakpoints: HashSet<u16>,
}

fn to_addr(s: Option<&str>) -> Result<Address, String> {
    match s {
        Some(s) => Address::from_str_radix(s, 16).map_err(|e| e.to_string()),
        None => Err(String::from("No Address specified."))
    }
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            step: true,
            step_on_breakpoint: false,
            breakpoints: HashSet::new(),
        }
    }

    pub fn step(&mut self, bus: &mut Bus, cpu: &mut CPU, ppu: &PPU) {
        if self.step || self.breakpoints.contains(&cpu.pc) {
            self.step |= self.step_on_breakpoint;
            println!("\n-- PAUSE ON {:04X} --\n\n{}", cpu.pc, cpu);
            self.prompt(bus, cpu, ppu);
            println!("\n-- -- -- --\n");
        }
    }

    fn prompt(&mut self, bus: &mut Bus, cpu: &mut CPU, ppu: &PPU) {

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .ok()
                .expect("Couldn't read line");

            let mut split = input.split_whitespace();

            if let Some(command) = split.next() {
                match command {
                    "s" => {
                        self.step = true;
                        println!("Stepping enabled.");
                    },
                    "S" => {
                        self.step = false;
                        println!("Stepping disabled.");
                    },
                    "b" => {
                        self.step_on_breakpoint = true;
                        println!("Stepping enabled on next breakpoint.");
                    },
                    "B" => {
                        self.step_on_breakpoint = false;
                        println!("Step on breakpoint disabled.");
                    },
                    "cpu" => println!("\n{cpu}\n"),
                    "ppu" => println!("\n{ppu}\n"),
                    "a" => {
                        match to_addr(split.next()) {
                            Ok(address) => self.add_breakpoint(address),
                            Err(e) => println!("{}", e)
                        }
                    },
                    "d" => {
                        match to_addr(split.next()) {
                            Ok(address) => self.remove_breakpoint(address),
                            Err(e) => println!("{e}")
                        }
                    },
                    "list" => {
                        println!("Breakpoints: ");
                        for address in &self.breakpoints {
                            println!("\t{address:04X}");
                        }
                    },
                    "stack" => {
                        match to_addr(split.next()) {
                            Ok(address) => cpu.print_stack(bus, address),
                            Err(e) => println!("{e}")
                        }
                    },
                    "peek" => {
                        match to_addr(split.next()) {
                            Ok(address) => println!("{:02X}", bus.read_byte(address)),
                            Err(e) => println!("{e}")
                        }
                    }
                    "reset" => {
                        cpu.pc = 0x0100;
                    }
                    "quit" => panic!("QUIT"),
                    _ => {}
                }
            }
            else {
                break;
            }
        }
    }

    pub fn add_breakpoint(&mut self, address: Address) {
        println!("Adding breakpoint {:04X}", address);
        self.breakpoints.insert(address);
    }

    pub fn remove_breakpoint(&mut self, address: Address) {
        print!("Removing breakpoint {:04X}... ", address);
        println!("{}", if self.breakpoints.remove(&address) { "Ok" } else { "Fail" });
    }
}