#[derive(Copy, Clone)]
#[allow(unused)]
#[repr(u8)]
pub enum Flag {
    Zero = 0x80,
    Subtract = 0x40,
    HalfCarry = 0x20,
    Carry = 0x10,
}

impl Flag {
    pub fn fmt(flags: u8) -> String {
        use Flag::*;

        const LITERALS: [(Flag, char); 4] =
            [(Zero, 'Z'), (Subtract, 'N'), (HalfCarry, 'H'), (Carry, 'C')];

        let mut out: [char; 4] = ['-', '-', '-', '-'];

        for i in 0..4 {
            if Flag::test(&flags, LITERALS[i].0) {
                out[i] = LITERALS[i].1;
            }
        }

        String::from_iter(out)
    }

    pub fn set(flags: &mut u8, flag: Flag) {
        *flags |= flag as u8
    }

    pub fn clear(flags: &mut u8, flag: Flag) {
        *flags &= !(flag as u8)
    }

    pub fn test(flags: &u8, flag: Flag) -> bool {
        flags & (flag as u8) != 0
    }
}
