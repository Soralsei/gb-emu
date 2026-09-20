use crate::types::{Condition, Operand, Reg16, Reg8};

/// Spelling of a value as an execution-side Rust expression (`Reg8::A`).
pub trait RustEmitter {
    fn emit_rust(&self) -> String;
}

/// Spelling of an operand as a const for the disasm table (`Operand::Mem(Reg16::HL)`).
pub trait ConstEmitter {
    fn emit_const(&self) -> String;
}

impl RustEmitter for Reg8 {
    fn emit_rust(&self) -> String {
        use Reg8::*;
        match self {
            A => "Reg8::A",
            B => "Reg8::B",
            C => "Reg8::C",
            D => "Reg8::D",
            E => "Reg8::E",
            F => "Reg8::F",
            H => "Reg8::H",
            L => "Reg8::L",
            W => "Reg8::W",
            Z => "Reg8::Z",
        }
        .into()
    }
}

impl RustEmitter for Reg16 {
    fn emit_rust(&self) -> String {
        use Reg16::*;
        match self {
            AF => "Reg16::AF",
            BC => "Reg16::BC",
            DE => "Reg16::DE",
            HL => "Reg16::HL",
            SP => "Reg16::SP",
            PC => "Reg16::PC",
            WZ => "Reg16::WZ",
        }
        .into()
    }
}

impl RustEmitter for Condition {
    fn emit_rust(&self) -> String {
        match self {
            Condition::Unconditional => "Condition::Unconditional",
            Condition::NotZero => "Condition::NotZero",
            Condition::Zero => "Condition::Zero",
            Condition::NotCarry => "Condition::NotCarry",
            Condition::Carry => "Condition::Carry",
        }
        .into()
    }
}

impl<T: RustEmitter> RustEmitter for Option<T> {
    fn emit_rust(&self) -> String {
        self.as_ref().map_or(String::new(), RustEmitter::emit_rust)
    }
}

/// Mirrors `Operand` one-to-one into the runtime enum used by the disasm table.
impl ConstEmitter for Operand {
    fn emit_const(&self) -> String {
        match self {
            Operand::Reg8(r) => format!("Operand::Reg8({})", r.emit_rust()),
            Operand::Reg16(r) => format!("Operand::Reg16({})", r.emit_rust()),
            Operand::Imm8 => "Operand::Imm8".into(),
            Operand::Rel8 => "Operand::Rel8".into(),
            Operand::Imm16 => "Operand::Imm16".into(),
            Operand::Mem(r) => format!("Operand::Mem({})", r.emit_rust()),
            Operand::MemImm16 => "Operand::MemImm16".into(),
            Operand::HighC => "Operand::HighC".into(),
            Operand::HighImm8 => "Operand::HighImm8".into(),
            Operand::Cond(c) => format!("Operand::Cond({})", c.emit_rust()),
            Operand::Num(n) => format!("Operand::Num({n})"),
        }
    }
}
