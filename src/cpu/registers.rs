use std::fmt;
use crate::Word;

pub union Register {
    byte: (u8, u8),
    word: u16,
}

impl Register {
    #[inline(always)]
    pub fn new(word: Word) -> Self {
        Register { word: word }
    }

    #[inline(always)]
    pub fn right(&mut self) -> &mut u8 {
        unsafe { &mut self.byte.0 }
    }

    #[inline(always)]
    pub fn left(&mut self) -> &mut u8 {
        unsafe { &mut self.byte.1 }
    }

    #[inline(always)]
    pub fn word(&mut self) -> &mut u16 {
        unsafe { &mut self.word }
    }

    #[inline(always)]
    pub fn value(&self) -> u16 {
        unsafe { self.word }
    }

    #[inline(always)]
    pub fn inc_word(&mut self) {
        unsafe {
            match self.word {
                0xFFFF => self.word = 0,
                _ => self.word += 1,
            }
        };
    }

    #[inline(always)]
    pub fn dec_word(&mut self) {
        unsafe {
            match self.word {
                0x0 => self.word = 0xFFFF,
                _ => self.word -= 1,
            }
        };
    }

    pub fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04X}", unsafe { self.word })
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}

impl fmt::Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}
