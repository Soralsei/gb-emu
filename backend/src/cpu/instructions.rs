#![allow(unused)]
use std::io::stdin;
use std::ops::{Shl, Shr};
use std::ptr::null;

use super::cpu::{self, Cpu, DMem, Dst, Imem16, Imem8, Mem, Src};
use super::operations::*;
use super::registers::{Reg16, Reg8};

pub enum Opcode {
    Unprefixed(u8),
    Prefixed(u8),
}

pub enum Timing {
    Normal,
    Conditional,
}

pub enum Condition {
    Unconditional,
    NotZero,
    Zero,
    NotCarry,
    Carry,
}

impl Condition {
    pub fn eval(&self, cpu: &Cpu) -> bool {
        match self {
            Condition::Unconditional => true,
            Condition::NotZero => !cpu.registers.f.zero,
            Condition::Zero => cpu.registers.f.zero,
            Condition::NotCarry => !cpu.registers.f.carry,
            Condition::Carry => cpu.registers.f.carry,
        }
    }
}

#[derive(PartialEq)]
pub struct ConditionCycles {
    pub not_taken: usize,
    pub taken: usize,
}

#[derive(PartialEq)]
pub enum Cycles {
    Unconditional(usize),
    Conditional(ConditionCycles),
}

#[derive(PartialEq)]
pub struct Instruction {
    pub cycles: Cycles,
    pub mnemonic: &'static str,
    pub execute: fn(&mut Cpu) -> Timing,
}

pub const NOP: Instruction = Instruction {
    cycles: Cycles::Unconditional(4),
    mnemonic: "NOP",
    execute: |_: &mut Cpu| nop(),
};

pub const ILLEGAL: Instruction = Instruction {
    cycles: Cycles::Unconditional(4),
    mnemonic: "ILL",
    execute: |cpu: &mut Cpu| {
        eprintln!(
            "Unknown opcode at address 0x{:04X}",
            cpu.registers.pc.wrapping_sub(1)
        );
        Timing::Normal
    },
};

impl Instruction {
    pub fn get_instruction(opcode: Opcode) -> Option<&'static Instruction> {
        match opcode {
            Opcode::Unprefixed(op) => Instruction::get_unprefixed_instruction(op),
            Opcode::Prefixed(op) => Instruction::get_prefixed_instruction(op),
        }
    }

    fn get_unprefixed_instruction(opcode: u8) -> Option<&'static Instruction> {
        match opcode {
            0x00 => Some(&NOP),
            0x01 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "LD BC,NN",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::BC, Imem16),
            }),
            0x02 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD (BC),A",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::BC), Reg8::A),
            }),
            0x03 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "INC BC",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::BC),
            }),
            0x04 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "INC B",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::B),
            }),
            0x05 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "DEC B",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::B),
            }),
            0x06 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD B,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Imem8),
            }),
            0x07 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "RLCA",
                execute: |cpu: &mut Cpu| rlca(cpu),
            }),
            0x08 => Some(&Instruction {
                cycles: Cycles::Unconditional(20),
                mnemonic: "LD NN,SP",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Imem16), Reg16::SP),
            }),
            0x09 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ADD HL,BC",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::BC),
            }),
            0x0A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD A,(BC)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Reg16::BC)),
            }),
            0x0B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "DEC BC",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::BC),
            }),
            0x0C => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "INC C",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::C),
            }),
            0x0D => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "DEC C",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::C),
            }),
            0x0E => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD C,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Imem8),
            }),
            0x0F => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "RRCA",
                execute: |cpu: &mut Cpu| rrca(cpu),
            }),
            0x10 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "STOP 0",
                execute: |cpu: &mut Cpu| {
                    cpu.stop();
                    Timing::Normal
                },
            }),
            0x11 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "LD DE,NN",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::DE, Imem16),
            }),
            0x12 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD (DE),A",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::DE), Reg8::A),
            }),
            0x13 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "INC DE",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::DE),
            }),
            0x14 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "INC D",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::D),
            }),
            0x15 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "DEC D",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::D),
            }),
            0x16 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD D,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Imem8),
            }),
            0x17 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "RLA",
                execute: |cpu: &mut Cpu| rla(cpu),
            }),
            0x18 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "JR N",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::Unconditional, Imem8),
            }),
            0x19 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ADD HL,DE",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::DE),
            }),
            0x1A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD A,(DE)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Reg16::DE)),
            }),
            0x1B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "DEC DE",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::DE),
            }),
            0x1C => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "INC E",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::E),
            }),
            0x1D => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "DEC E",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::E),
            }),
            0x1E => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD E,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Imem8),
            }),
            0x1F => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "RRA",
                execute: |cpu: &mut Cpu| rra(cpu),
            }),
            0x20 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 12,
                }),
                mnemonic: "JR NZ,N",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::NotZero, Imem8),
            }),
            0x21 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "LD HL,NN",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::HL, Imem16),
            }),
            0x22 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LDI (HL+),A",
                execute: |cpu: &mut Cpu| ldi(cpu, Mem(Reg16::HL), Reg8::A, Reg16::HL),
            }),
            0x23 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "INC HL",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::HL),
            }),
            0x24 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "INC H",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::H),
            }),
            0x25 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "DEC H",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::H),
            }),
            0x26 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD H,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Imem8),
            }),
            0x27 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "DAA",
                execute: |cpu: &mut Cpu| daa(cpu),
            }),
            0x28 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 12,
                }),
                mnemonic: "JR Z,N",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::Zero, Imem8),
            }),
            0x29 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ADD HL,HL",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::HL),
            }),
            0x2A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LDI A,(HL+)",
                execute: |cpu: &mut Cpu| ldi(cpu, Reg8::A, Mem(Reg16::HL), Reg16::HL),
            }),
            0x2B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "DEC HL",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::HL),
            }),
            0x2C => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "INC L",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::L),
            }),
            0x2D => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "DEC L",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::L),
            }),
            0x2E => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD L,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Imem8),
            }),
            0x2F => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "CPL",
                execute: |cpu: &mut Cpu| cpl(cpu),
            }),
            0x30 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 12,
                }),
                mnemonic: "JR NC,N",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::NotCarry, Imem8),
            }),
            0x31 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "LD SP,NN",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::SP, Imem16),
            }),
            0x32 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LDD (HL+),A",
                execute: |cpu: &mut Cpu| ldd(cpu, Mem(Reg16::HL), Reg8::A, Reg16::HL),
            }),
            0x33 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "INC SP",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::SP),
            }),
            0x34 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "INC (HL)",
                execute: |cpu: &mut Cpu| inc(cpu, Mem(Reg16::HL)),
            }),
            0x35 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "DEC (HL)",
                execute: |cpu: &mut Cpu| dec(cpu, Mem(Reg16::HL)),
            }),
            0x36 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "LD (HL),N",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Imem8),
            }),
            0x37 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SCF",
                execute: |cpu: &mut Cpu| scf(cpu),
            }),
            0x38 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 12,
                }),
                mnemonic: "JR CF,N",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::Carry, Imem8),
            }),
            0x39 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ADD HL,SP",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::SP),
            }),
            0x3A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LDD A,(HL)",
                execute: |cpu: &mut Cpu| ldd(cpu, Reg8::A, Mem(Reg16::HL), Reg16::HL),
            }),
            0x3B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "DEC SP",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::SP),
            }),
            0x3C => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "INC A",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::A),
            }),
            0x3D => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "DEC A",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::A),
            }),
            0x3E => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD A,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Imem8),
            }),
            0x3F => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "CCF",
                execute: |cpu: &mut Cpu| ccf(cpu),
            }),
            0x40 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD B,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::B),
            }),
            0x41 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD B,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::C),
            }),
            0x42 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD B,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::D),
            }),
            0x43 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD B,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::E),
            }),
            0x44 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD B,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::H),
            }),
            0x45 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD B,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::L),
            }),
            0x46 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD B,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Mem(Reg16::HL)),
            }),
            0x47 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD B,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::A),
            }),
            0x48 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD C,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::B),
            }),
            0x49 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD C,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::C),
            }),
            0x4A => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD C,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::D),
            }),
            0x4B => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD C,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::E),
            }),
            0x4C => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD C,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::H),
            }),
            0x4D => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD C,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::L),
            }),
            0x4E => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD C,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Mem(Reg16::HL)),
            }),
            0x4F => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD C,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::A),
            }),
            0x50 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD D,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::B),
            }),
            0x51 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD D,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::C),
            }),
            0x52 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD D,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::D),
            }),
            0x53 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD D,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::E),
            }),
            0x54 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD D,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::H),
            }),
            0x55 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD D,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::L),
            }),
            0x56 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD D,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Mem(Reg16::HL)),
            }),
            0x57 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD D,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::A),
            }),
            0x58 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD E,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::B),
            }),
            0x59 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD E,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::C),
            }),
            0x5A => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD E,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::D),
            }),
            0x5B => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD E,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::E),
            }),
            0x5C => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD E,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::H),
            }),
            0x5D => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD E,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::L),
            }),
            0x5E => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD E,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Mem(Reg16::HL)),
            }),
            0x5F => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD E,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::A),
            }),
            0x60 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD H,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::B),
            }),
            0x61 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD H,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::C),
            }),
            0x62 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD H,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::D),
            }),
            0x63 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD H,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::E),
            }),
            0x64 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD H,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::H),
            }),
            0x65 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD H,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::L),
            }),
            0x66 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD H,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Mem(Reg16::HL)),
            }),
            0x67 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD H,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::A),
            }),
            0x68 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD L,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::B),
            }),
            0x69 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD L,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::C),
            }),
            0x6A => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD L,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::D),
            }),
            0x6B => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD L,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::E),
            }),
            0x6C => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD L,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::H),
            }),
            0x6D => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD L,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::L),
            }),
            0x6E => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD L,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Mem(Reg16::HL)),
            }),
            0x6F => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD L,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::A),
            }),
            0x70 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD (HL),B",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::B),
            }),
            0x71 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD (HL),C",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::C),
            }),
            0x72 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD (HL),D",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::D),
            }),
            0x73 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD (HL),E",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::E),
            }),
            0x74 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD (HL),H",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::H),
            }),
            0x75 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD (HL),L",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::L),
            }),
            0x76 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "HALT",
                execute: |cpu: &mut Cpu| halt(cpu),
            }),
            0x77 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD (HL),A",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::A),
            }),
            0x78 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD A,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::B),
            }),
            0x79 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD A,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::C),
            }),
            0x7A => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD A,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::D),
            }),
            0x7B => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD A,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::E),
            }),
            0x7C => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD A,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::H),
            }),
            0x7D => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD A,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::L),
            }),
            0x7E => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD A,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0x7F => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "LD A,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::A),
            }),
            0x80 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADD A,B",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::B),
            }),
            0x81 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADD A,C",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::C),
            }),
            0x82 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADD A,D",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::D),
            }),
            0x83 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADD A,E",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::E),
            }),
            0x84 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADD A,H",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::H),
            }),
            0x85 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADD A,L",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::L),
            }),
            0x86 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ADD A,(HL)",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0x87 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADD A,A",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::A),
            }),
            0x88 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADC A,B",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::B),
            }),
            0x89 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADC A,C",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::C),
            }),
            0x8A => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADC A,D",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::D),
            }),
            0x8B => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADC A,E",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::E),
            }),
            0x8C => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADC A,H",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::H),
            }),
            0x8D => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADC A,L",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::L),
            }),
            0x8E => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ADC A,(HL)",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0x8F => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ADC A,A",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::A),
            }),
            0x90 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SUB B",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::B),
            }),
            0x91 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SUB C",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::C),
            }),
            0x92 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SUB D",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::D),
            }),
            0x93 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SUB E",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::E),
            }),
            0x94 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SUB H",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::H),
            }),
            0x95 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SUB L",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::L),
            }),
            0x96 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SUB (HL)",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0x97 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SUB A",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::A),
            }),
            0x98 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SBC A,B",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::B),
            }),
            0x99 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SBC A,C",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::C),
            }),
            0x9A => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SBC A,D",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::D),
            }),
            0x9B => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SBC A,E",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::E),
            }),
            0x9C => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SBC A,H",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::H),
            }),
            0x9D => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SBC A,L",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::L),
            }),
            0x9E => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SBC A,(HL)",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0x9F => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "SBC A,A",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::A),
            }),
            0xA0 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "AND B",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::B),
            }),
            0xA1 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "AND C",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::C),
            }),
            0xA2 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "AND D",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::D),
            }),
            0xA3 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "AND E",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::E),
            }),
            0xA4 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "AND H",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::H),
            }),
            0xA5 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "AND L",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::L),
            }),
            0xA6 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "AND (HL)",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0xA7 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "AND A",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::A),
            }),
            0xA8 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "XOR B",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::B),
            }),
            0xA9 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "XOR C",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::C),
            }),
            0xAA => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "XOR D",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::D),
            }),
            0xAB => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "XOR E",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::E),
            }),
            0xAC => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "XOR H",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::H),
            }),
            0xAD => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "XOR L",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::L),
            }),
            0xAE => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "XOR (HL)",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0xAF => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "XOR A",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::A),
            }),
            0xB0 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "OR B",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::B),
            }),
            0xB1 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "OR C",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::C),
            }),
            0xB2 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "OR D",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::D),
            }),
            0xB3 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "OR E",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::E),
            }),
            0xB4 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "OR H",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::H),
            }),
            0xB5 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "OR L",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::L),
            }),
            0xB6 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "OR (HL)",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0xB7 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "OR A",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::A),
            }),
            0xB8 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "CP B",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::B),
            }),
            0xB9 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "CP C",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::C),
            }),
            0xBA => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "CP D",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::D),
            }),
            0xBB => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "CP E",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::E),
            }),
            0xBC => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "CP H",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::H),
            }),
            0xBD => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "CP L",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::L),
            }),
            0xBE => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "CP (HL)",
                execute: |cpu: &mut Cpu| cp(cpu, Mem(Reg16::HL)),
            }),
            0xBF => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "CP A",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A),
            }),
            0xC0 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 20,
                }),
                mnemonic: "RET NZ",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::NotZero),
            }),
            0xC1 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "POP BC",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::BC),
            }),
            0xC2 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 16,
                }),
                mnemonic: "JP NZ,NN",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::NotZero, Imem16),
            }),
            0xC3 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "JP NN",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Unconditional, Imem16),
            }),
            0xC4 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 24,
                }),
                mnemonic: "CALL NZ,NN",
                execute: |cpu: &mut Cpu| call(cpu, Condition::NotZero, Imem16),
            }),
            0xC5 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "PUSH BC",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::BC),
            }),
            0xC6 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ADD A,N",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Imem8),
            }),
            0xC7 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RST 0",
                execute: |cpu: &mut Cpu| rst(cpu, 0),
            }),
            0xC8 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 20,
                }),
                mnemonic: "RET Z",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::Zero),
            }),
            0xC9 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RET",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::Unconditional),
            }),
            0xCA => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 16,
                }),
                mnemonic: "JP Z,NN",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Zero, Imem16),
            }),
            0xCC => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 24,
                }),
                mnemonic: "CALL Z,NN",
                execute: |cpu: &mut Cpu| call(cpu, Condition::Zero, Imem16),
            }),
            0xCD => Some(&Instruction {
                cycles: Cycles::Unconditional(24),
                mnemonic: "CALL NN",
                execute: |cpu: &mut Cpu| call(cpu, Condition::Unconditional, Imem16),
            }),
            0xCE => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ADC A,N",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Imem8),
            }),
            0xCF => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RST 8",
                execute: |cpu: &mut Cpu| rst(cpu, 0x08),
            }),
            0xD0 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 20,
                }),
                mnemonic: "RET NC",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::NotCarry),
            }),
            0xD1 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "POP DE",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::DE),
            }),
            0xD2 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 16,
                }),
                mnemonic: "JP NC,NN",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::NotCarry, Imem16),
            }),
            0xD4 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 24,
                }),
                mnemonic: "CALL NC,NN",
                execute: |cpu: &mut Cpu| call(cpu, Condition::NotCarry, Imem16),
            }),
            0xD5 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "PUSH DE",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::DE),
            }),
            0xD6 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SUB N",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Imem8),
            }),
            0xD7 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RST 16",
                execute: |cpu: &mut Cpu| rst(cpu, 0x10),
            }),
            0xD8 => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 20,
                }),
                mnemonic: "RET CF",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::Carry),
            }),
            0xD9 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RETI",
                execute: |cpu: &mut Cpu| reti(cpu),
            }),
            0xDA => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 16,
                }),
                mnemonic: "JP CF,NN",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Carry, Imem16),
            }),
            0xDC => Some(&Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 24,
                }),
                mnemonic: "CALL CF,NN",
                execute: |cpu: &mut Cpu| call(cpu, Condition::Carry, Imem16),
            }),
            0xDE => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SBC A,N",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Imem8),
            }),
            0xDF => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RST 24",
                execute: |cpu: &mut Cpu| rst(cpu, 0x18),
            }),
            0xE0 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "LD (0XFF00+A8),A",
                execute: |cpu: &mut Cpu| ld(cpu, DMem(Imem8), Reg8::A),
            }),
            0xE1 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "POP HL",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::HL),
            }),
            0xE2 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD (0XFF00+C),A",
                execute: |cpu: &mut Cpu| ld(cpu, DMem(Reg8::C), Reg8::A),
            }),
            0xE5 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "PUSH HL",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::HL),
            }),
            0xE6 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "AND N",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Imem8),
            }),
            0xE7 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RST 32",
                execute: |cpu: &mut Cpu| rst(cpu, 0x20),
            }),
            0xE8 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "ADD SP,N",
                execute: |cpu: &mut Cpu| {
                    let test = add_sp(cpu);
                    #[cfg(feature = "debug")]
                    stdin().read_line(&mut String::from(""));
                    test
                },
            }),
            0xE9 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "JP HL",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Unconditional, Reg16::HL),
            }),
            0xEA => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "LD (NN),A",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Imem16), Reg8::A),
            }),
            0xEE => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "XOR N",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Imem8),
            }),
            0xEF => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RST 40",
                execute: |cpu: &mut Cpu| rst(cpu, 0x28),
            }),
            0xF0 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "LD A,(0XFF00+A8)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, DMem(Imem8)),
            }),
            0xF1 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "POP AF",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::AF),
            }),
            0xF2 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD A,(0XFF00+C)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, DMem(Reg8::C)),
            }),
            0xF3 => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "DI",
                execute: |cpu: &mut Cpu| di(cpu),
            }),
            0xF5 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "PUSH AF",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::AF),
            }),
            0xF6 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "OR N",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Imem8),
            }),
            0xF7 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RST 48",
                execute: |cpu: &mut Cpu| rst(cpu, 0x30),
            }),
            0xF8 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "LDHL SP,N",
                execute: |cpu: &mut Cpu| ldhl(cpu),
            }),
            0xF9 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "LD SP,HL",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::SP, Reg16::HL),
            }),
            0xFA => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "LD A,(NN)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Imem16)),
            }),
            0xFB => Some(&Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "EI",
                execute: |cpu: &mut Cpu| ei(cpu),
            }),
            0xFE => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "CP N",
                execute: |cpu: &mut Cpu| cp(cpu, Imem8),
            }),
            0xFF => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RST 56",
                execute: |cpu: &mut Cpu| rst(cpu, 0x38),
            }),
            _ => None,
        }
    }

    fn get_prefixed_instruction(opcode: u8) -> Option<&'static Instruction> {
        match opcode {
            0x00 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RLC B",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::B),
            }),
            0x01 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RLC C",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::C),
            }),
            0x02 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RLC D",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::D),
            }),
            0x03 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RLC E",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::E),
            }),
            0x04 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RLC H",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::H),
            }),
            0x05 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RLC L",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::L),
            }),
            0x06 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RLC (HL)",
                execute: |cpu: &mut Cpu| rlc(cpu, Mem(Reg16::HL)),
            }),
            0x07 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RLC A",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::A),
            }),
            0x08 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RRC B",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::B),
            }),
            0x09 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RRC C",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::C),
            }),
            0x0A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RRC D",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::D),
            }),
            0x0B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RRC E",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::E),
            }),
            0x0C => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RRC H",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::H),
            }),
            0x0D => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RRC L",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::L),
            }),
            0x0E => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RRC (HL)",
                execute: |cpu: &mut Cpu| rrc(cpu, Mem(Reg16::HL)),
            }),
            0x0F => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RRC A",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::A),
            }),
            0x10 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RL B",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::B),
            }),
            0x11 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RL C",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::C),
            }),
            0x12 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RL D",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::D),
            }),
            0x13 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RL E",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::E),
            }),
            0x14 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RL H",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::H),
            }),
            0x15 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RL L",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::L),
            }),
            0x16 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RL (HL)",
                execute: |cpu: &mut Cpu| rl(cpu, Mem(Reg16::HL)),
            }),
            0x17 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RL A",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::A),
            }),
            0x18 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RR B",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::B),
            }),
            0x19 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RR C",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::C),
            }),
            0x1A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RR D",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::D),
            }),
            0x1B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RR E",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::E),
            }),
            0x1C => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RR H",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::H),
            }),
            0x1D => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RR L",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::L),
            }),
            0x1E => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RR (HL)",
                execute: |cpu: &mut Cpu| rr(cpu, Mem(Reg16::HL)),
            }),
            0x1F => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RR A",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::A),
            }),
            0x20 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SLA B",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::B),
            }),
            0x21 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SLA C",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::C),
            }),
            0x22 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SLA D",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::D),
            }),
            0x23 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SLA E",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::E),
            }),
            0x24 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SLA H",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::H),
            }),
            0x25 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SLA L",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::L),
            }),
            0x26 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SLA (HL)",
                execute: |cpu: &mut Cpu| sla(cpu, Mem(Reg16::HL)),
            }),
            0x27 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SLA A",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::A),
            }),
            0x28 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRA B",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::B),
            }),
            0x29 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRA C",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::C),
            }),
            0x2A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRA D",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::D),
            }),
            0x2B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRA E",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::E),
            }),
            0x2C => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRA H",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::H),
            }),
            0x2D => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRA L",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::L),
            }),
            0x2E => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SRA (HL)",
                execute: |cpu: &mut Cpu| sra(cpu, Mem(Reg16::HL)),
            }),
            0x2F => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRA A",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::A),
            }),
            0x30 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SWAP B",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::B),
            }),
            0x31 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SWAP C",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::C),
            }),
            0x32 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SWAP D",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::D),
            }),
            0x33 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SWAP E",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::E),
            }),
            0x34 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SWAP H",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::H),
            }),
            0x35 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SWAP L",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::L),
            }),
            0x36 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SWAP (HL)",
                execute: |cpu: &mut Cpu| swap(cpu, Mem(Reg16::HL)),
            }),
            0x37 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SWAP A",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::A),
            }),
            0x38 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRL B",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::B),
            }),
            0x39 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRL C",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::C),
            }),
            0x3A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRL D",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::D),
            }),
            0x3B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRL E",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::E),
            }),
            0x3C => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRL H",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::H),
            }),
            0x3D => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRL L",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::L),
            }),
            0x3E => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SRL (HL)",
                execute: |cpu: &mut Cpu| srl(cpu, Mem(Reg16::HL)),
            }),
            0x3F => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SRL A",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::A),
            }),
            0x40 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 0,B",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::B),
            }),
            0x41 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 0,C",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::C),
            }),
            0x42 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 0,D",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::D),
            }),
            0x43 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 0,E",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::E),
            }),
            0x44 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 0,H",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::H),
            }),
            0x45 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 0,L",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::L),
            }),
            0x46 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "BIT 0,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Mem(Reg16::HL)),
            }),
            0x47 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 0,A",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::A),
            }),
            0x48 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 1,B",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::B),
            }),
            0x49 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 1,C",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::C),
            }),
            0x4A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 1,D",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::D),
            }),
            0x4B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 1,E",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::E),
            }),
            0x4C => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 1,H",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::H),
            }),
            0x4D => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 1,L",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::L),
            }),
            0x4E => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "BIT 1,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Mem(Reg16::HL)),
            }),
            0x4F => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 1,A",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::A),
            }),
            0x50 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 2,B",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::B),
            }),
            0x51 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 2,C",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::C),
            }),
            0x52 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 2,D",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::D),
            }),
            0x53 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 2,E",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::E),
            }),
            0x54 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 2,H",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::H),
            }),
            0x55 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 2,L",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::L),
            }),
            0x56 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "BIT 2,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Mem(Reg16::HL)),
            }),
            0x57 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 2,A",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::A),
            }),
            0x58 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 3,B",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::B),
            }),
            0x59 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 3,C",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::C),
            }),
            0x5A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 3,D",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::D),
            }),
            0x5B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 3,E",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::E),
            }),
            0x5C => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 3,H",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::H),
            }),
            0x5D => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 3,L",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::L),
            }),
            0x5E => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "BIT 3,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Mem(Reg16::HL)),
            }),
            0x5F => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 3,A",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::A),
            }),
            0x60 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 4,B",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::B),
            }),
            0x61 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 4,C",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::C),
            }),
            0x62 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 4,D",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::D),
            }),
            0x63 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 4,E",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::E),
            }),
            0x64 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 4,H",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::H),
            }),
            0x65 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 4,L",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::L),
            }),
            0x66 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "BIT 4,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Mem(Reg16::HL)),
            }),
            0x67 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 4,A",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::A),
            }),
            0x68 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 5,B",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::B),
            }),
            0x69 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 5,C",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::C),
            }),
            0x6A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 5,D",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::D),
            }),
            0x6B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 5,E",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::E),
            }),
            0x6C => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 5,H",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::H),
            }),
            0x6D => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 5,L",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::L),
            }),
            0x6E => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "BIT 5,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Mem(Reg16::HL)),
            }),
            0x6F => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 5,A",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::A),
            }),
            0x70 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 6,B",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::B),
            }),
            0x71 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 6,C",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::C),
            }),
            0x72 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 6,D",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::D),
            }),
            0x73 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 6,E",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::E),
            }),
            0x74 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 6,H",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::H),
            }),
            0x75 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 6,L",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::L),
            }),
            0x76 => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "BIT 6,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Mem(Reg16::HL)),
            }),
            0x77 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 6,A",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::A),
            }),
            0x78 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 7,B",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::B),
            }),
            0x79 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 7,C",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::C),
            }),
            0x7A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 7,D",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::D),
            }),
            0x7B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 7,E",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::E),
            }),
            0x7C => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 7,H",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::H),
            }),
            0x7D => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 7,L",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::L),
            }),
            0x7E => Some(&Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "BIT 7,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Mem(Reg16::HL)),
            }),
            0x7F => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "BIT 7,A",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::A),
            }),
            0x80 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 0,B",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::B),
            }),
            0x81 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 0,C",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::C),
            }),
            0x82 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 0,D",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::D),
            }),
            0x83 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 0,E",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::E),
            }),
            0x84 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 0,H",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::H),
            }),
            0x85 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 0,L",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::L),
            }),
            0x86 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RES 0,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 0, Mem(Reg16::HL)),
            }),
            0x87 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 0,A",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::A),
            }),
            0x88 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 1,B",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::B),
            }),
            0x89 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 1,C",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::C),
            }),
            0x8A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 1,D",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::D),
            }),
            0x8B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 1,E",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::E),
            }),
            0x8C => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 1,H",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::H),
            }),
            0x8D => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 1,L",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::L),
            }),
            0x8E => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RES 1,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 1, Mem(Reg16::HL)),
            }),
            0x8F => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 1,A",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::A),
            }),
            0x90 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 2,B",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::B),
            }),
            0x91 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 2,C",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::C),
            }),
            0x92 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 2,D",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::D),
            }),
            0x93 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 2,E",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::E),
            }),
            0x94 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 2,H",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::H),
            }),
            0x95 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 2,L",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::L),
            }),
            0x96 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RES 2,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 2, Mem(Reg16::HL)),
            }),
            0x97 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 2,A",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::A),
            }),
            0x98 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 3,B",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::B),
            }),
            0x99 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 3,C",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::C),
            }),
            0x9A => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 3,D",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::D),
            }),
            0x9B => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 3,E",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::E),
            }),
            0x9C => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 3,H",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::H),
            }),
            0x9D => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 3,L",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::L),
            }),
            0x9E => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RES 3,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 3, Mem(Reg16::HL)),
            }),
            0x9F => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 3,A",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::A),
            }),
            0xA0 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 4,B",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::B),
            }),
            0xA1 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 4,C",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::C),
            }),
            0xA2 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 4,D",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::D),
            }),
            0xA3 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 4,E",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::E),
            }),
            0xA4 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 4,H",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::H),
            }),
            0xA5 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 4,L",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::L),
            }),
            0xA6 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RES 4,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 4, Mem(Reg16::HL)),
            }),
            0xA7 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 4,A",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::A),
            }),
            0xA8 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 5,B",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::B),
            }),
            0xA9 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 5,C",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::C),
            }),
            0xAA => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 5,D",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::D),
            }),
            0xAB => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 5,E",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::E),
            }),
            0xAC => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 5,H",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::H),
            }),
            0xAD => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 5,L",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::L),
            }),
            0xAE => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RES 5,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 5, Mem(Reg16::HL)),
            }),
            0xAF => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 5,A",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::A),
            }),
            0xB0 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 6,B",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::B),
            }),
            0xB1 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 6,C",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::C),
            }),
            0xB2 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 6,D",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::D),
            }),
            0xB3 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 6,E",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::E),
            }),
            0xB4 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 6,H",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::H),
            }),
            0xB5 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 6,L",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::L),
            }),
            0xB6 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RES 6,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 6, Mem(Reg16::HL)),
            }),
            0xB7 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 6,A",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::A),
            }),
            0xB8 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 7,B",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::B),
            }),
            0xB9 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 7,C",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::C),
            }),
            0xBA => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 7,D",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::D),
            }),
            0xBB => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 7,E",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::E),
            }),
            0xBC => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 7,H",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::H),
            }),
            0xBD => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 7,L",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::L),
            }),
            0xBE => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "RES 7,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 7, Mem(Reg16::HL)),
            }),
            0xBF => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "RES 7,A",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::A),
            }),
            0xC0 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 0,B",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::B),
            }),
            0xC1 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 0,C",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::C),
            }),
            0xC2 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 0,D",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::D),
            }),
            0xC3 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 0,E",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::E),
            }),
            0xC4 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 0,H",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::H),
            }),
            0xC5 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 0,L",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::L),
            }),
            0xC6 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SET 0,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 0, Mem(Reg16::HL)),
            }),
            0xC7 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 0,A",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::A),
            }),
            0xC8 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 1,B",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::B),
            }),
            0xC9 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 1,C",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::C),
            }),
            0xCA => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 1,D",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::D),
            }),
            0xCB => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 1,E",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::E),
            }),
            0xCC => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 1,H",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::H),
            }),
            0xCD => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 1,L",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::L),
            }),
            0xCE => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SET 1,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 1, Mem(Reg16::HL)),
            }),
            0xCF => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 1,A",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::A),
            }),
            0xD0 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 2,B",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::B),
            }),
            0xD1 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 2,C",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::C),
            }),
            0xD2 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 2,D",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::D),
            }),
            0xD3 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 2,E",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::E),
            }),
            0xD4 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 2,H",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::H),
            }),
            0xD5 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 2,L",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::L),
            }),
            0xD6 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SET 2,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 2, Mem(Reg16::HL)),
            }),
            0xD7 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 2,A",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::A),
            }),
            0xD8 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 3,B",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::B),
            }),
            0xD9 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 3,C",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::C),
            }),
            0xDA => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 3,D",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::D),
            }),
            0xDB => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 3,E",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::E),
            }),
            0xDC => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 3,H",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::H),
            }),
            0xDD => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 3,L",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::L),
            }),
            0xDE => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SET 3,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 3, Mem(Reg16::HL)),
            }),
            0xDF => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 3,A",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::A),
            }),
            0xE0 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 4,B",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::B),
            }),
            0xE1 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 4,C",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::C),
            }),
            0xE2 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 4,D",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::D),
            }),
            0xE3 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 4,E",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::E),
            }),
            0xE4 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 4,H",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::H),
            }),
            0xE5 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 4,L",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::L),
            }),
            0xE6 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SET 4,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 4, Mem(Reg16::HL)),
            }),
            0xE7 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 4,A",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::A),
            }),
            0xE8 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 5,B",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::B),
            }),
            0xE9 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 5,C",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::C),
            }),
            0xEA => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 5,D",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::D),
            }),
            0xEB => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 5,E",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::E),
            }),
            0xEC => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 5,H",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::H),
            }),
            0xED => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 5,L",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::L),
            }),
            0xEE => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SET 5,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 5, Mem(Reg16::HL)),
            }),
            0xEF => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 5,A",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::A),
            }),
            0xF0 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 6,B",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::B),
            }),
            0xF1 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 6,C",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::C),
            }),
            0xF2 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 6,D",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::D),
            }),
            0xF3 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 6,E",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::E),
            }),
            0xF4 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 6,H",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::H),
            }),
            0xF5 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 6,L",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::L),
            }),
            0xF6 => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SET 6,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 6, Mem(Reg16::HL)),
            }),
            0xF7 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 6,A",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::A),
            }),
            0xF8 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 7,B",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::B),
            }),
            0xF9 => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 7,C",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::C),
            }),
            0xFA => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 7,D",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::D),
            }),
            0xFB => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 7,E",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::E),
            }),
            0xFC => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 7,H",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::H),
            }),
            0xFD => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 7,L",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::L),
            }),
            0xFE => Some(&Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "SET 7,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 7, Mem(Reg16::HL)),
            }),
            0xFF => Some(&Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "SET 7,A",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::A),
            }),
            _ => None,
        }
    }
}
