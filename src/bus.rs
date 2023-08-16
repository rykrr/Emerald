use std::cell::RefCell;
use std::rc::{Rc, Weak};
use log::{debug, trace};

pub type Address = u16;
pub type Byte = u8;
pub type Word = u16;


/// Type for specifying addresses to bind to the bus.
pub enum Attach {
    /// Bind this module to the address range XX00 -> XXFF (inclusive).
    Block(u8),

    /// Bind this module to the address range XX00 -> YYFF (inclusive).
    BlockRange(u8, u8),

    /// Bind a single register FFXX.
    Register(u8),

    /// Bind a range of registers FFXX -> FFYY.
    RegisterRange(u8, u8),
}

/// The interface for sending and receiving messages on a shared bus.
pub trait BusListener {
    /// This method is called by the bus when the listener is attached to the bus.
    /// Provides a list of blocks/registers that this listener will respond to.
    /// Address ranges should not overlap with another bus listener (except for the boot ROM).
    fn bus_attach(&mut self) -> Vec<Attach>;

    /// This method is invoked when a read_byte command is issued to the bus and the address falls
    /// within an address range specified by the listener.
    fn bus_read(&self, address: Address) -> Byte;

    /// This method is invoked when a write_byte command is issued to the bus and the address falls
    /// within an address range specified by the listener.
    fn bus_write(&mut self, bus: &mut Bus, address: Address, value: Byte);
}

/// A general type for handling different implementations of BusListener
type BusListenerCell = RefCell<dyn BusListener>;

/// The bus facilitates the reading and writing of bytes between multiple components.
pub struct Bus {
    callbacks: Vec<Weak<BusListenerCell>>,
    callback_addresses: [Option<usize>; 0x100],
    register_addresses: [Option<usize>; 0x100],
}

impl Bus {
    pub const fn new() -> Self {
        Self {
            callbacks: Vec::new(),
            callback_addresses: [None; 0x100],
            register_addresses: [None; 0x100],
        }
    }

    fn attach_block(&mut self, block: u8, callback_index: usize) {
        assert!(self.callback_addresses[block as usize].is_none(),
                "Address block {block:02X} is already bound.");
        self.callback_addresses[block as usize] = Some(callback_index);
    }

    fn attach_register(&mut self, register: u8, callback_index: usize) {
        assert!(self.register_addresses[register as usize].is_none(),
        "Register {register:02X} is already bound.");

        assert!(register < 0x80 || register == 0xFF,
                "Invalid register {register:02X} specified. \
                 Valid registers are 0x00->0x80 and 0xF");
        self.register_addresses[register as usize] = Some(callback_index);
    }

    /// Attach a listener to blocks of addresses or registers.
    /// Note that modules attached to 0xFF only listen to 0xFF80..=0xFFFE.
    pub fn attach(&mut self, listener: Rc<BusListenerCell>) {
        let attachments = listener.borrow_mut().bus_attach();
        assert!(!attachments.is_empty(), "Listener declare at least one address!");

        let callback_index = self.callbacks.len();
        self.callbacks.push(Rc::downgrade(&listener));

        for attachment in attachments {
            use Attach::*;
            match attachment {
                Block(block) => {
                    self.attach_block(block, callback_index);
                },
                BlockRange(start, end) => {
                    for block in start..=end {
                        self.attach_block(block, callback_index);
                    }
                },
                Register(register) => {
                    self.attach_register(register, callback_index);
                },
                RegisterRange(start, end) => {
                    for register in start..=end {
                        self.attach_register(register, callback_index);
                    }
                },
            }
        }
    }

    fn resolve_address(&self, address: Address) -> &Weak<BusListenerCell> {
        let callback_index = match address {
            // This range covers the IO registers and Interrupt Enable (IE) register.
            0xFF00..=0xFF7F | 0xFFFF => {
                let register = address as usize & 0xFF;
                assert!(self.register_addresses[register].is_some(),
                        "Failed to resolve {address:04X}, register not attached.");
                self.register_addresses[register].unwrap()
            },
            _ => {
                let block = (address >> 8) as usize & 0xFF;
                assert!(self.callback_addresses[block].is_some(),
                        "Failed to resolve {address:04X}, block not attached.");
                self.callback_addresses[block].unwrap()
            }
        };

        &self.callbacks[callback_index]
    }

    pub fn read_byte(&self, address: Address) -> Byte {
        trace!("Reading from address {:04X}...", address);
        self.resolve_address(address)
            .upgrade()
            .unwrap()
            .borrow()
            .bus_read(address)
    }

    pub fn write_byte(&mut self, address: Address, value: Byte) {
        trace!("Writing value {:02X} to address {:04X}", value, address);
        self.resolve_address(address)
            .upgrade()
            .unwrap()
            .borrow_mut()
            .bus_write(self, address, value)
    }

    pub fn read_word(&self, address: Address) -> Word {
        (self.read_byte(address) as Word) | ((self.read_byte(address + 1) as Word) << 8)
    }

    pub fn write_word(&mut self, address: Address, value: Word) {
        self.write_byte(address, value as Byte);
        self.write_byte(address + 1, (value >> 8) as Byte);
    }
}