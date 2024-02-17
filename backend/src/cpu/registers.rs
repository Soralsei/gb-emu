use crate::{
    is_bit_set,
    util::bit_operations::*,
};

const ZERO_BIT: u8 = 7;
const SUB_BIT: u8 = 6;
const HALF_CARRY_BIT: u8 = 5;
const CARRY_BIT: u8 = 4;

#[derive(Debug)]
pub struct Flags {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl std::fmt::Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Flags [z: {}, n: {}, hc: {}, c: {}]",
            self.zero, self.subtract, self.half_carry, self.carry
        )
    }
}

impl std::convert::From<&Flags> for u8 {
    fn from(flag: &Flags) -> u8 {
        let mut res = 0;
        res |= (flag.zero as u8) << ZERO_BIT;
        res |= (flag.subtract as u8) << SUB_BIT;
        res |= (flag.half_carry as u8) << HALF_CARRY_BIT;
        res |= (flag.carry as u8) << CARRY_BIT;
        res & 0xF0
    }
}

impl std::convert::From<u8> for Flags {
    fn from(flag: u8) -> Flags {
        Flags {
            zero: is_bit_set!(flag, ZERO_BIT),
            subtract: is_bit_set!(flag, SUB_BIT),
            half_carry: is_bit_set!(flag, HALF_CARRY_BIT),
            carry: is_bit_set!(flag, CARRY_BIT),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

#[derive(Copy, Clone)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

#[derive(Debug)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: Flags,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0x11,
            b: 0x00,
            c: 0x00,
            d: 0xFF,
            e: 0x56,
            f: (Flags::from(0x80)),
            h: 0x00,
            l: 0x0D,
            pc: 0x100,
            sp: 0xfffe,
        }
    }

    #[inline(always)]
    pub fn read_u8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::F => u8::from(&self.f),
            Reg8::H => self.h,
            Reg8::L => self.l,
        }
    }

    #[inline(always)]
    pub fn write_u8(&mut self, reg: Reg8, value: u8) {
        match reg {
            Reg8::A => self.a = value,
            Reg8::B => self.b = value,
            Reg8::C => self.c = value,
            Reg8::D => self.d = value,
            Reg8::E => self.e = value,
            Reg8::F => self.f = Flags::from(value),
            Reg8::H => self.h = value,
            Reg8::L => self.l = value,
        }
    }

    #[inline(always)]
    pub fn read_u16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AF => self.af(),
            Reg16::BC => self.bc(),
            Reg16::DE => self.de(),
            Reg16::HL => self.hl(),
            Reg16::SP => self.sp,
            Reg16::PC => self.pc,
        }
    }

    #[inline(always)]
    pub fn write_u16(&mut self, reg: Reg16, value: u16) {
        match reg {
            Reg16::AF => self.set_af(value),
            Reg16::BC => self.set_bc(value),
            Reg16::DE => self.set_de(value),
            Reg16::HL => self.set_hl(value),
            Reg16::SP => self.sp = value,
            Reg16::PC => self.pc = value,
        }
    }

    #[inline(always)]
    fn af(&self) -> u16 {
        bytes_to_word(self.a, u8::from(&self.f))
    }

    #[inline(always)]
    fn bc(&self) -> u16 {
        bytes_to_word(self.b, self.c)
    }

    #[inline(always)]
    fn de(&self) -> u16 {
        bytes_to_word(self.d, self.e)
    }

    #[inline(always)]
    fn hl(&self) -> u16 {
        bytes_to_word(self.h, self.l)
    }

    #[inline(always)]
    fn set_af(&mut self, value: u16) {
        let (a, f) = word_to_bytes(value);
        self.a = a;
        self.f = Flags::from(f & 0xF0);
    }

    #[inline(always)]
    fn set_bc(&mut self, value: u16) {
        let (b, c) = word_to_bytes(value);
        self.b = b;
        self.c = c;
    }

    #[inline(always)]
    fn set_de(&mut self, value: u16) {
        let (d, e) = word_to_bytes(value);
        self.d = d;
        self.e = e;
    }

    #[inline(always)]
    fn set_hl(&mut self, value: u16) {
        let (h, l) = word_to_bytes(value);
        self.h = h;
        self.l = l;
    }
}

impl std::fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Registers [a: 0x{:02X}, f: {}, b: 0x{:02X}, c: 0x{:02X}, d: 0x{:02X}, e: 0x{:02X}, h: 0x{:02X}, l: 0x{:02X}, sp: 0x{:04X}, pc: 0x{:04X}]",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.pc
        )
    }
}
