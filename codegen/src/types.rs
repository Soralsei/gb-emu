use clap::Parser;
use std::path::PathBuf;

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

pub trait MnemonicEmitter {
    fn to_mnemonic(&self) -> String;
}

pub trait RustEmitter {
    fn to_rust(&self) -> String;
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

pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
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

pub enum Condition {
    Unconditional,
    NotZero,
    Zero,
    NotCarry,
    Carry,
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

enum Operand {
    R8(Reg8),
    R16(Reg16),
    Imm8,
    Imm16,
    Cond(Condition),
    Mem(Box<Operand>),     // (hl) (bc) (a16)
    HighMem(Box<Operand>), // (0xff00+c) (0xff00+a8)
    Bit(u8),               // "0".."7"
    Vector(u8),            // 0x00..0x38 for rst
}

impl RustEmitter for Operand {
    fn to_rust(&self) -> String {
        match self {
            Operand::R8(reg8) => reg8.to_rust(),
            Operand::R16(reg16) => reg16.to_rust(),
            Operand::Imm8 => "Imem8".to_string(),
            Operand::Imm16 => "Imem16".to_string(),
            Operand::Cond(condition) => condition.to_rust(),
            Operand::Mem(operand) => format!("Mem({})", operand.to_rust()),
            Operand::HighMem(operand) => format!("DMem({})", operand.to_rust()),
            Operand::Bit(bit) => format!("{}", bit),
            Operand::Vector(dst) => format!("0x{:02X}", dst),
        }
    }
}

impl MnemonicEmitter for Operand {
    fn to_mnemonic(&self) -> String {
        todo!()
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
