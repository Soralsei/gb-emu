use clap::Parser;
use std::{num::ParseIntError, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ConditionalTime {
    // Order matters here, since the yaml declares them in this order
    pub taken: usize,
    pub not_taken: usize,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Time {
    One(usize),
    Two(ConditionalTime),
}

#[derive(Debug, Deserialize)]
pub struct Instruction {
    pub code: u16,
    pub operator: String,
    pub operands: Vec<String>,
    pub bits: usize,
    pub size: usize,
    pub time: Time,
    pub z: String,
    pub n: String,
    pub h: String,
    pub c: String,
}

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

impl TryFrom<&str> for Reg8 {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "a" => Ok(Reg8::A),
            "b" => Ok(Reg8::B),
            "c" => Ok(Reg8::C),
            "d" => Ok(Reg8::D),
            "e" => Ok(Reg8::E),
            "f" => Ok(Reg8::F),
            "h" => Ok(Reg8::H),
            "l" => Ok(Reg8::L),
            _ => Err(format!("Unknown register '{}'", value)),
        }
    }
}

pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

impl TryFrom<&str> for Reg16 {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "af" => Ok(Reg16::AF),
            "bc" => Ok(Reg16::BC),
            "de" => Ok(Reg16::DE),
            "hl" => Ok(Reg16::HL),
            "sp" => Ok(Reg16::SP),
            "pc" => Ok(Reg16::PC),
            _ => Err(format!("Unknown 16 bits register '{}'", value)),
        }
    }
}

pub enum Condition {
    Unconditional,
    NotZero,
    Zero,
    NotCarry,
    Carry,
}

impl TryFrom<&str> for Condition {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "nz" => Ok(Condition::NotZero),
            "z" => Ok(Condition::Zero),
            "nc" => Ok(Condition::NotCarry),
            "cf" => Ok(Condition::Carry),
            _ => Err(format!("Unknown condition {}", value)),
        }
    }
}

pub enum Operand {
    R8(Reg8),
    R16(Reg16),
    Imm8,
    Rel8,
    Imm16,
    Cond(Condition),
    Mem(Box<Operand>),     // (hl) (bc) (a16)
    HighMem(Box<Operand>), // (0xff00+c) (0xff00+a8)
    Bit(u8),               // "0".."7"
    Vector(u8),            // 0x00..0x38 for rst
    Dummy(u8),             // dummy variant for stop N
}

impl Operand {
    pub fn from_bit(value: &str) -> Result<Self, String> {
        let bit: u8 = match value.parse() {
            Ok(val) => val,
            Err(e) => return Err(format!("failed to parse bit {}: {}", value, e)),
        };
        match bit {
            0..=7 => Ok(Self::Bit(bit)),
            _ => Err(format!("invalid bit {}", value)),
        }
    }

    // The yaml spells reset vectors in hex ("0x00".."0x38"), but accept plain
    // decimal too so the two notations can't silently diverge.
    pub fn from_vector(value: &str) -> Result<Self, ParseIntError> {
        match value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")) {
            Some(hex) => Ok(Self::Vector(u8::from_str_radix(hex, 16)?)),
            None => Ok(Self::Vector(value.parse()?)),
        }
    }
}

impl TryFrom<&str> for Operand {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(reg) = Reg8::try_from(value) {
            return Ok(Operand::R8(reg));
        }
        if let Ok(reg) = Reg16::try_from(value) {
            return Ok(Operand::R16(reg));
        }
        if let Ok(cond) = Condition::try_from(value) {
            return Ok(Operand::Cond(cond));
        }
        if let Some(inner) = value.strip_prefix('(').and_then(|v| v.strip_suffix(')')) {
            return match inner.strip_prefix("0xff00+") {
                Some(hi) => Ok(Operand::HighMem(Box::new(Operand::try_from(hi)?))),
                None => Ok(Operand::Mem(Box::new(Operand::try_from(inner)?))),
            };
        }
        match value {
            "d8" | "a8" => Ok(Operand::Imm8),
            "r8" => Ok(Operand::Rel8),
            "d16" | "a16" => Ok(Operand::Imm16),
            _ => Err(()),
        }
    }
}

pub enum Cycles {
    Unconditional(usize),
    Conditional(usize, usize),
}

#[derive(Debug, Serialize)]
pub struct InstructionTemplate {
    pub code: String,     // "01"
    pub mnemonic: String, // "LD BC, NN"
    pub cycles: String,   // "Cycles::Unconditional(8)"
    pub call: String,     // "ld(cpu, Reg16::BC, Imem16)"
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct GenerateArgs {
    #[arg(long)]
    pub template_path: PathBuf,
    #[arg(long)]
    pub oplist: PathBuf,
    #[arg(long = "out")]
    pub output: PathBuf,
}
