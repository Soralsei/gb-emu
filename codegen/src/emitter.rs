use crate::types::{Condition, Operand, Reg16, Reg8};

pub trait MnemonicEmitter {
    fn to_mnemonic(&self) -> String;
}

pub trait RustEmitter {
    fn to_rust(&self) -> String;
}

impl MnemonicEmitter for Reg8 {
    fn to_mnemonic(&self) -> String {
        match self {
            Reg8::A => "a",
            Reg8::B => "b",
            Reg8::C => "c",
            Reg8::D => "d",
            Reg8::E => "e",
            Reg8::F => "f",
            Reg8::H => "h",
            Reg8::L => "l",
        }
        .to_string()
    }
}
impl RustEmitter for Reg8 {
    fn to_rust(&self) -> String {
        match self {
            Reg8::A => "Reg8::A",
            Reg8::B => "Reg8::B",
            Reg8::C => "Reg8::C",
            Reg8::D => "Reg8::D",
            Reg8::E => "Reg8::E",
            Reg8::F => "Reg8::F",
            Reg8::H => "Reg8::H",
            Reg8::L => "Reg8::L",
        }
        .to_string()
    }
}

impl MnemonicEmitter for Reg16 {
    fn to_mnemonic(&self) -> String {
        match self {
            Reg16::AF => "af",
            Reg16::BC => "bc",
            Reg16::DE => "de",
            Reg16::HL => "hl",
            Reg16::SP => "sp",
            Reg16::PC => "pc",
        }
        .to_string()
    }
}

impl RustEmitter for Reg16 {
    fn to_rust(&self) -> String {
        match self {
            Reg16::AF => "Reg16::AF",
            Reg16::BC => "Reg16::BC",
            Reg16::DE => "Reg16::DE",
            Reg16::HL => "Reg16::HL",
            Reg16::SP => "Reg16::SP",
            Reg16::PC => "Reg16::PC",
        }
        .to_string()
    }
}

impl MnemonicEmitter for Condition {
    fn to_mnemonic(&self) -> String {
        match self {
            Condition::Unconditional => "",
            Condition::NotZero => "nz",
            Condition::Zero => "z",
            Condition::NotCarry => "nc",
            Condition::Carry => "c",
        }
        .to_string()
    }
}

impl RustEmitter for Condition {
    fn to_rust(&self) -> String {
        match self {
            Condition::Unconditional => "Condition::Unconditional",
            Condition::NotZero => "Condition::NotZero",
            Condition::Zero => "Condition::Zero",
            Condition::NotCarry => "Condition::NotCarry",
            Condition::Carry => "Condition::Carry",
        }
        .to_string()
    }
}

impl RustEmitter for Operand {
    fn to_rust(&self) -> String {
        match self {
            Operand::R8(reg8) => reg8.to_rust(),
            Operand::R16(reg16) => reg16.to_rust(),
            Operand::Imm8 | Operand::Rel8 => "Imm8".to_string(),
            Operand::Imm16 => "Imm16".to_string(),
            Operand::Cond(condition) => condition.to_rust(),
            Operand::Mem(operand) => format!("Mem({})", operand.to_rust()),
            Operand::HighMem(operand) => format!("DMem({})", operand.to_rust()),
            Operand::Bit(bit) => bit.to_string(),
            Operand::Vector(val) | Operand::Dummy(val) => format!("0x{:02X}", val),
        }
    }
}

impl MnemonicEmitter for Operand {
    fn to_mnemonic(&self) -> String {
        match self {
            Operand::R8(reg8) => reg8.to_mnemonic(),
            Operand::R16(reg16) => reg16.to_mnemonic(),
            Operand::Imm8 => "d8".to_string(),
            Operand::Rel8 => "r8".to_string(),
            Operand::Imm16 => "d16".to_string(),
            Operand::Cond(condition) => condition.to_mnemonic(),
            Operand::Mem(operand) => format!("({})", operand.to_mnemonic()),
            Operand::HighMem(operand) => format!("(0xff00 + {})", operand.to_mnemonic()),
            Operand::Bit(bit) => bit.to_string(),
            Operand::Vector(val) | Operand::Dummy(val) => format!("0x{:02X}", val),
        }
    }
}
