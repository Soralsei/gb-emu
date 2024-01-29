#![allow(dead_code)]
const ZERO_FLAG: u8 = 7;
const SUB_FLAG: u8 = 6;
const HALF_CARRY_FLAG: u8 = 5;
const CARRY_FLAG: u8 = 4;

#[derive(Debug)]
pub struct Flags {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl std::fmt::Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "flags")
    }
}

impl std::convert::From<&Flags> for u8 {
    fn from(flag: &Flags) -> u8 {
        (flag.zero as u8) << ZERO_FLAG
            | (flag.subtract as u8) << SUB_FLAG
            | (flag.half_carry as u8) << HALF_CARRY_FLAG
            | (flag.carry as u8) << CARRY_FLAG
    }
}

impl std::convert::From<u8> for Flags {
    fn from(flag: u8) -> Flags {
        Flags {
            zero: (flag & (1 << ZERO_FLAG)) != 0,
            subtract: (flag & (1 << SUB_FLAG)) != 0,
            half_carry: (flag & (1 << HALF_CARRY_FLAG)) != 0,
            carry: (flag & (1 << CARRY_FLAG)) != 0,
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
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xd8,
            f: (Flags::from(0xb0)),
            h: 0x01,
            l: 0x4d,
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
            Reg16::AF => {
                self.set_af(value)
            }
            Reg16::BC => {
                self.set_bc(value);
            }
            Reg16::DE => {
                self.set_de(value);
            }
            Reg16::HL => {
                self.set_hl(value);
            }
            Reg16::SP => {
                self.sp = value;
            }
            Reg16::PC => {
                self.pc = value;
            }
        }
    }

    #[inline(always)]
    fn single_to_double(&self, a: u8, b: u8) -> u16 {
        (a as u16) << 8 | (b as u16)
    }

    #[inline(always)]
    fn af(&self) -> u16 {
        self.single_to_double(self.a, u8::from(&self.f))
    }

    #[inline(always)]
    fn bc(&self) -> u16 {
        self.single_to_double(self.b, self.c)
    }

    #[inline(always)]
    fn de(&self) -> u16 {
        self.single_to_double(self.d, self.e)
    }

    #[inline(always)]
    fn hl(&self) -> u16 {
        self.single_to_double(self.h, self.l)
    }

    #[inline(always)]
    fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = Flags::from((value & 0xFF) as u8);
    }

    #[inline(always)]
    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    #[inline(always)]
    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    #[inline(always)]
    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }
}

impl std::fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Registers (a: {}, f: {}, b: {}, c: {}, d: {}, e: {}, h: {}, l: {})",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l
        )
    }
}
