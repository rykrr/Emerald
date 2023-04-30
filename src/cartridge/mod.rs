mod rom_only;

use std::cell::RefCell;
use crate::BusListener;
use rom_only::RomOnlyCartridge;

use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::rc::Rc;
use log::info;

// Here lies impl Cartridge: BusListener; killed by a lack of trait upcasting support...

#[repr(u8)]
enum CartridgeType {
    ROM_ONLY = 0x00,
}

impl CartridgeType {
    fn from(ty: u8) -> Self {
        match ty {
            ROM_ONLY=> CartridgeType::ROM_ONLY,
            _ => panic!("Cartridge type is not implemented."),
        }
    }

    fn to_cartridge(&self, bytes: Vec<u8>) -> Rc<RefCell<dyn BusListener>> {
        match self {
            CartridgeType::ROM_ONLY => Rc::new(RefCell::new(RomOnlyCartridge::new(bytes))),
        }
    }
}

fn read_cartridge_name(bytes: &Vec<u8>) -> String {
    let mut title = &bytes[0x134..=0x143];
    let end = title.iter().position(|&c| c == 0);

    if let Some(x) = end {
        title = &title[..x];
    }

    let title = std::str::from_utf8(&title).expect("Invalid title.");
    String::from(title)
}

pub fn load(path: &str) -> Rc<RefCell<dyn BusListener>> {
    let file = File::open(path).expect("Failed to open {path}.");
    let mut reader = BufReader::new(file);

    let mut bytes: Vec<u8> = vec![];
    let mut head = reader.by_ref().take(0x150);

    head.read_to_end(&mut bytes).expect("Cartridge file is too small.");

    //let bytes = fs::read(path).expect("Failed to read cartridge.");
    assert!(bytes.len() >= 0x150, "Cartridge is too small.");

    let cartridge_name = read_cartridge_name(&bytes);
    let cartridge_type = CartridgeType::from(bytes[0x147]);

    // TODO: Checksum goes here.

    info!("Loading \"{}\"", cartridge_name);

    reader.read_to_end(&mut bytes).expect("Error reading cartridge.");

    let cartridge = cartridge_type.to_cartridge(bytes);

    cartridge
}
