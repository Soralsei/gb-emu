#![allow(unused)]
use std::ops::{Shl, Shr};

use super::cpu::{self, Cpu, Dst, Imem16, Imem8, Mem, DMem, Src};
use super::registers::{Reg16, Reg8};
use super::operations::*;

pub enum Opcode {
    Unprefixed(u8),
    Prefixed(u8),
}

pub enum Timing {
    Normal,
    Conditionnal,
    Prefixed,
}

pub enum Condition {
    Unconditional,
    NotZero,
    Zero,
    NotCarry,
    Carry,
}

impl Condition {
    pub fn eval(self, cpu: &Cpu) -> bool {
        match self {
            Condition::Unconditional => true,
            Condition::NotZero => !cpu.registers.f.zero,
            Condition::Zero => cpu.registers.f.zero,
            Condition::NotCarry => !cpu.registers.f.carry,
            Condition::Carry => cpu.registers.f.carry,
        }
    }
}

pub struct Instruction {
    pub c_cycles: u8,
    pub conditional_c_cycles: Option<u8>,
    pub mnemonic: &'static str,
    pub execute: fn(&mut Cpu) -> Timing,
}

impl Instruction {
    pub fn get_instruction(opcode: Opcode) -> Option<&'static Instruction> {
        match opcode {
            Opcode::Unprefixed(op) => Instruction::get_unprefixed_instruction(op),
            Opcode::Prefixed(op) => todo!("Implement Prefixed opcode decoding"),
        }
    }

    fn get_unprefixed_instruction(opcode: u8) -> Option<&'static Instruction> {
        match opcode {
            0x00 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "NOP",
                execute: |cpu: &mut Cpu| nop(),
            }),
            0x01 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "LD BC,NN",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::BC, Imem16),
            }),
            0x02 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD (BC),A",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::BC), Reg8::A),
            }),
            0x03 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "INC BC",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::BC),
            }),
            0x04 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "INC B",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::B),
            }),
            0x05 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "DEC B",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::B),
            }),
            0x06 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD B,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Imem8),
            }),
            0x07 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "RLCA",
                execute: |cpu: &mut Cpu| rlca(cpu),
            }),
            0x08 => Some(&Instruction {
                c_cycles: 20,
                conditional_c_cycles: None,
                mnemonic: "LD NN,SP",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Imem16), Reg16::SP),
            }),
            0x09 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "ADD HL,BC",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::BC),
            }),
            0x0A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD A,(BC)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Reg16::BC)),
            }),
            0x0B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "DEC BC",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::BC),
            }),
            0x0C => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "INC C",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::C),
            }),
            0x0D => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "DEC C",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::C),
            }),
            0x0E => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD C,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Imem8),
            }),
            0x0F => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "RRCA",
                execute: |cpu: &mut Cpu| rrca(cpu),
            }),
            0x10 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "STOP 0",
                execute: |cpu: &mut Cpu| {
                    cpu.stop();
                    Timing::Normal
                },
            }),
            0x11 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "LD DE,NN",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::DE, Imem16),
            }),
            0x12 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD (DE),A",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::DE), Reg8::A),
            }),
            0x13 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "INC DE",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::DE),
            }),
            0x14 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "INC D",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::D),
            }),
            0x15 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "DEC D",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::D),
            }),
            0x16 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD D,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Imem8),
            }),
            0x17 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "RLA",
                execute: |cpu: &mut Cpu| rla(cpu),
            }),
            0x18 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "JR N",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::Unconditional, Imem8),
            }),
            0x19 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "ADD HL,DE",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::DE),
            }),
            0x1A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD A,(DE)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Reg16::DE)),
            }),
            0x1B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "DEC DE",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::DE),
            }),
            0x1C => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "INC E",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::E),
            }),
            0x1D => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "DEC E",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::E),
            }),
            0x1E => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD E,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Imem8),
            }),
            0x1F => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "RRA",
                execute: |cpu: &mut Cpu| rra(cpu),
            }),
            0x20 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: Some(12),
                mnemonic: "JR NZ,N",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::NotZero, Imem8),
            }),
            0x21 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "LD HL,NN",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::HL, Imem16),
            }),
            0x22 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LDI (HL+),A",
                execute: |cpu: &mut Cpu| ldi(cpu, Mem(Reg16::HL), Reg8::A, Reg16::HL),
            }),
            0x23 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "INC HL",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::HL),
            }),
            0x24 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "INC H",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::H),
            }),
            0x25 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "DEC H",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::H),
            }),
            0x26 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD H,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Imem8),
            }),
            0x27 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "DAA",
                execute: |cpu: &mut Cpu| daa(cpu),
            }),
            0x28 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: Some(12),
                mnemonic: "JR Z,N",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::Zero, Imem8),
            }),
            0x29 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "ADD HL,HL",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::HL),
            }),
            0x2A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LDI A,(HL+)",
                execute: |cpu: &mut Cpu| ldi(cpu, Reg8::A, Mem(Reg16::HL), Reg16::HL),
            }),
            0x2B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "DEC HL",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::HL),
            }),
            0x2C => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "INC L",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::L),
            }),
            0x2D => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "DEC L",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::L),
            }),
            0x2E => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD L,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Imem8),
            }),
            0x2F => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "CPL",
                execute: |cpu: &mut Cpu| cpl(cpu),
            }),
            0x30 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: Some(12),
                mnemonic: "JR NC,N",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::NotCarry, Imem8),
            }),
            0x31 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "LD SP,NN",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::SP, Imem16),
            }),
            0x32 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LDD (HL+),A",
                execute: |cpu: &mut Cpu| ldd(cpu, Mem(Reg16::HL), Reg8::A, Reg16::HL),
            }),
            0x33 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "INC SP",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::SP),
            }),
            0x34 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "INC (HL)",
                execute: |cpu: &mut Cpu| inc(cpu, Mem(Reg16::HL)),
            }),
            0x35 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "DEC (HL)",
                execute: |cpu: &mut Cpu| dec(cpu, Mem(Reg16::HL)),
            }),
            0x36 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "LD (HL),N",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Imem8),
            }),
            0x37 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SCF",
                execute: |cpu: &mut Cpu| scf(cpu),
            }),
            0x38 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: Some(12),
                mnemonic: "JR CF,N",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::Carry, Imem8),
            }),
            0x39 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "ADD HL,SP",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::SP),
            }),
            0x3A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LDD A,(HL)",
                execute: |cpu: &mut Cpu| ldd(cpu, Reg8::A, Mem(Reg16::HL), Reg16::HL),
            }),
            0x3B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "DEC SP",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::SP),
            }),
            0x3C => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "INC A",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::A),
            }),
            0x3D => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "DEC A",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::A),
            }),
            0x3E => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD A,N",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Imem8),
            }),
            0x3F => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "CCF",
                execute: |cpu: &mut Cpu| ccf(cpu),
            }),
            0x40 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD B,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::B),
            }),
            0x41 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD B,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::C),
            }),
            0x42 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD B,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::D),
            }),
            0x43 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD B,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::E),
            }),
            0x44 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD B,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::H),
            }),
            0x45 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD B,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::L),
            }),
            0x46 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD B,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Mem(Reg16::HL)),
            }),
            0x47 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD B,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::A),
            }),
            0x48 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD C,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::B),
            }),
            0x49 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD C,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::C),
            }),
            0x4A => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD C,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::D),
            }),
            0x4B => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD C,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::E),
            }),
            0x4C => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD C,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::H),
            }),
            0x4D => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD C,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::L),
            }),
            0x4E => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD C,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Mem(Reg16::HL)),
            }),
            0x4F => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD C,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::A),
            }),
            0x50 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD D,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::B),
            }),
            0x51 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD D,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::C),
            }),
            0x52 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD D,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::D),
            }),
            0x53 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD D,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::E),
            }),
            0x54 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD D,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::H),
            }),
            0x55 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD D,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::L),
            }),
            0x56 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD D,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Mem(Reg16::HL)),
            }),
            0x57 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD D,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::A),
            }),
            0x58 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD E,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::B),
            }),
            0x59 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD E,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::C),
            }),
            0x5A => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD E,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::D),
            }),
            0x5B => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD E,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::E),
            }),
            0x5C => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD E,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::H),
            }),
            0x5D => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD E,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::L),
            }),
            0x5E => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD E,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Mem(Reg16::HL)),
            }),
            0x5F => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD E,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::A),
            }),
            0x60 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD H,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::B),
            }),
            0x61 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD H,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::C),
            }),
            0x62 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD H,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::D),
            }),
            0x63 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD H,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::E),
            }),
            0x64 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD H,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::H),
            }),
            0x65 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD H,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::L),
            }),
            0x66 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD H,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Mem(Reg16::HL)),
            }),
            0x67 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD H,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::A),
            }),
            0x68 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD L,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::B),
            }),
            0x69 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD L,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::C),
            }),
            0x6A => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD L,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::D),
            }),
            0x6B => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD L,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::E),
            }),
            0x6C => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD L,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::H),
            }),
            0x6D => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD L,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::L),
            }),
            0x6E => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD L,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Mem(Reg16::HL)),
            }),
            0x6F => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD L,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::A),
            }),
            0x70 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD (HL),B",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::B),
            }),
            0x71 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD (HL),C",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::C),
            }),
            0x72 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD (HL),D",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::D),
            }),
            0x73 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD (HL),E",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::E),
            }),
            0x74 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD (HL),H",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::H),
            }),
            0x75 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD (HL),L",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::L),
            }),
            0x76 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "HALT",
                execute: |cpu: &mut Cpu| halt(cpu),
            }),
            0x77 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD (HL),A",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::A),
            }),
            0x78 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD A,B",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::B),
            }),
            0x79 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD A,C",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::C),
            }),
            0x7A => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD A,D",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::D),
            }),
            0x7B => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD A,E",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::E),
            }),
            0x7C => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD A,H",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::H),
            }),
            0x7D => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD A,L",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::L),
            }),
            0x7E => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD A,(HL)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0x7F => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "LD A,A",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::A),
            }),
            0x80 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADD A,B",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::B),
            }),
            0x81 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADD A,C",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::C),
            }),
            0x82 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADD A,D",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::D),
            }),
            0x83 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADD A,E",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::E),
            }),
            0x84 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADD A,H",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::H),
            }),
            0x85 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADD A,L",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::L),
            }),
            0x86 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "ADD A,(HL)",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0x87 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADD A,A",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::A),
            }),
            0x88 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADC A,B",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::B),
            }),
            0x89 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADC A,C",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::C),
            }),
            0x8A => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADC A,D",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::D),
            }),
            0x8B => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADC A,E",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::E),
            }),
            0x8C => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADC A,H",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::H),
            }),
            0x8D => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADC A,L",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::L),
            }),
            0x8E => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "ADC A,(HL)",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0x8F => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "ADC A,A",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::A),
            }),
            0x90 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SUB B",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::B),
            }),
            0x91 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SUB C",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::C),
            }),
            0x92 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SUB D",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::D),
            }),
            0x93 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SUB E",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::E),
            }),
            0x94 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SUB H",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::H),
            }),
            0x95 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SUB L",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::L),
            }),
            0x96 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SUB (HL)",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0x97 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SUB A",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::A),
            }),
            0x98 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SBC A,B",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::B),
            }),
            0x99 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SBC A,C",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::C),
            }),
            0x9A => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SBC A,D",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::D),
            }),
            0x9B => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SBC A,E",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::E),
            }),
            0x9C => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SBC A,H",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::H),
            }),
            0x9D => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SBC A,L",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::L),
            }),
            0x9E => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SBC A,(HL)",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0x9F => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "SBC A,A",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::A),
            }),
            0xA0 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "AND B",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::B),
            }),
            0xA1 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "AND C",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::C),
            }),
            0xA2 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "AND D",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::D),
            }),
            0xA3 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "AND E",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::E),
            }),
            0xA4 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "AND H",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::H),
            }),
            0xA5 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "AND L",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::L),
            }),
            0xA6 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "AND (HL)",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0xA7 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "AND A",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::A),
            }),
            0xA8 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "XOR B",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::B),
            }),
            0xA9 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "XOR C",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::C),
            }),
            0xAA => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "XOR D",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::D),
            }),
            0xAB => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "XOR E",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::E),
            }),
            0xAC => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "XOR H",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::H),
            }),
            0xAD => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "XOR L",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::L),
            }),
            0xAE => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "XOR (HL)",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0xAF => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "XOR A",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::A),
            }),
            0xB0 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "OR B",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::B),
            }),
            0xB1 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "OR C",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::C),
            }),
            0xB2 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "OR D",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::D),
            }),
            0xB3 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "OR E",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::E),
            }),
            0xB4 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "OR H",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::H),
            }),
            0xB5 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "OR L",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::L),
            }),
            0xB6 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "OR (HL)",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0xB7 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "OR A",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::A),
            }),
            0xB8 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "CP B",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A, Reg8::B),
            }),
            0xB9 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "CP C",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A, Reg8::C),
            }),
            0xBA => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "CP D",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A, Reg8::D),
            }),
            0xBB => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "CP E",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A, Reg8::E),
            }),
            0xBC => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "CP H",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A, Reg8::H),
            }),
            0xBD => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "CP L",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A, Reg8::L),
            }),
            0xBE => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "CP (HL)",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A, Mem(Reg16::HL)),
            }),
            0xBF => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "CP A",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A, Reg8::A),
            }),
            0xC0 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: Some(20),
                mnemonic: "RET NZ",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::NotZero),
            }),
            0xC1 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "POP BC",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::BC),
            }),
            0xC2 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: Some(16),
                mnemonic: "JP NZ,NN",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::NotZero, Imem16),
            }),
            0xC3 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "JP NN",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Unconditional, Imem16),
            }),
            0xC4 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: Some(24),
                mnemonic: "CALL NZ,NN",
                execute: |cpu: &mut Cpu| call(cpu, Condition::NotZero, Imem16),
            }),
            0xC5 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "PUSH BC",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::BC),
            }),
            0xC6 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "ADD A,N",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Imem8),
            }),
            0xC7 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RST 0",
                execute: |cpu: &mut Cpu| rst(cpu, 0),
            }),
            0xC8 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: Some(20),
                mnemonic: "RET Z",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::Zero),
            }),
            0xC9 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RET",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::Unconditional),
            }),
            0xCA => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: Some(16),
                mnemonic: "JP Z,NN",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Zero, Imem16),
            }),
            0xCC => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: Some(24),
                mnemonic: "CALL Z,NN",
                execute: |cpu: &mut Cpu| call(cpu, Condition::Zero, Imem16),
            }),
            0xCD => Some(&Instruction {
                c_cycles: 24,
                conditional_c_cycles: None,
                mnemonic: "CALL NN",
                execute: |cpu: &mut Cpu| call(cpu, Condition::Unconditional, Imem16),
            }),
            0xCE => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "ADC A,N",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Imem8),
            }),
            0xCF => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RST 8",
                execute: |cpu: &mut Cpu| rst(cpu, 8),
            }),
            0xD0 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: Some(20),
                mnemonic: "RET NC",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::NotCarry),
            }),
            0xD1 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "POP DE",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::DE),
            }),
            0xD2 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: Some(16),
                mnemonic: "JP NC,NN",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::NotCarry, Imem16),
            }),
            0xD4 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: Some(24),
                mnemonic: "CALL NC,NN",
                execute: |cpu: &mut Cpu| call(cpu, Condition::NotCarry, Imem16),
            }),
            0xD5 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "PUSH DE",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::DE),
            }),
            0xD6 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SUB N",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Imem8),
            }),
            0xD7 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RST 16",
                execute: |cpu: &mut Cpu| rst(cpu, 16),
            }),
            0xD8 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: Some(20),
                mnemonic: "RET CF",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::Carry),
            }),
            0xD9 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RETI",
                execute: |cpu: &mut Cpu| reti(cpu),
            }),
            0xDA => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: Some(16),
                mnemonic: "JP CF,NN",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Carry, Imem16),
            }),
            0xDC => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: Some(24),
                mnemonic: "CALL CF,NN",
                execute: |cpu: &mut Cpu| call(cpu, Condition::Carry, Imem16),
            }),
            0xDE => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SBC A,N",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Imem8),
            }),
            0xDF => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RST 24",
                execute: |cpu: &mut Cpu| rst(cpu, 24),
            }),
            0xE0 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "LD (0XFF00+A8),A",
                execute: |cpu: &mut Cpu| ld(cpu, DMem(Imem8), Reg8::A),
            }),
            0xE1 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "POP HL",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::HL),
            }),
            0xE2 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD (0XFF00+C),A",
                execute: |cpu: &mut Cpu| ld(cpu, DMem(Reg8::C), Reg8::A),
            }),
            0xE5 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "PUSH HL",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::HL),
            }),
            0xE6 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "AND N",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Imem8),
            }),
            0xE7 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RST 32",
                execute: |cpu: &mut Cpu| rst(cpu, 32),
            }),
            0xE8 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "ADD SP,N",
                execute: |cpu: &mut Cpu| add_sp(cpu, Imem8),
            }),
            0xE9 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "JP HL",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Unconditional, Reg16::HL),
            }),
            0xEA => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "LD NN,A",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Imem16), Reg8::A),
            }),
            0xEE => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "XOR N",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Imem8),
            }),
            0xEF => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RST 40",
                execute: |cpu: &mut Cpu| rst(cpu, 40),
            }),
            0xF0 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "LD A,(0XFF00+A8)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, DMem(Imem8)),
            }),
            0xF1 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "POP AF",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::AF),
            }),
            0xF2 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD A,(0XFF00+C)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, DMem(Reg8::C)),
            }),
            0xF3 => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "DI",
                execute: |cpu: &mut Cpu| di(cpu),
            }),
            0xF5 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "PUSH AF",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::AF),
            }),
            0xF6 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "OR N",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Imem8),
            }),
            0xF7 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RST 48",
                execute: |cpu: &mut Cpu| rst(cpu, 48),
            }),
            0xF8 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "LDHL SP,N",
                execute: |cpu: &mut Cpu| ldhl(cpu),
            }),
            0xF9 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "LD SP,HL",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::SP, Reg16::HL),
            }),
            0xFA => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "LD A,NN",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Imem16)),
            }),
            0xFB => Some(&Instruction {
                c_cycles: 4,
                conditional_c_cycles: None,
                mnemonic: "EI",
                execute: |cpu: &mut Cpu| ei(cpu),
            }),
            0xFE => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "CP N",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A, Imem8),
            }),
            0xFF => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RST 56",
                execute: |cpu: &mut Cpu| rst(cpu, 56),
            }),
            _ => panic!("Invalid OPCODE 0x{:02X}", opcode),
        }
    }

    fn get_prefixed_instruction(opcode: u8) -> Option<&'static Instruction>  {
        match opcode {
            0x00 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RLC B",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::B)
            }),
            0x01 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RLC C",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::C)
            }),
            0x02 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RLC D",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::D)
            }),
            0x03 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RLC E",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::E)
            }),
            0x04 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RLC H",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::H)
            }),
            0x05 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RLC L",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::L)
            }),
            0x06 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RLC (HL)",
                execute: |cpu: &mut Cpu| rlc(cpu, Mem(Reg16::HL))
            }),
            0x07 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RLC A",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::A)
            }),
            0x08 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RRC B",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::B)
            }),
            0x09 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RRC C",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::C)
            }),
            0x0A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RRC D",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::D)
            }),
            0x0B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RRC E",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::E)
            }),
            0x0C => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RRC H",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::H)
            }),
            0x0D => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RRC L",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::L)
            }),
            0x0E => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RRC (HL)",
                execute: |cpu: &mut Cpu| rrc(cpu, Mem(Reg16::HL))
            }),
            0x0F => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RRC A",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::A)
            }),
            0x10 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RL B",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::B)
            }),
            0x11 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RL C",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::C)
            }),
            0x12 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RL D",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::D)
            }),
            0x13 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RL E",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::E)
            }),
            0x14 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RL H",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::H)
            }),
            0x15 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RL L",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::L)
            }),
            0x16 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RL (HL)",
                execute: |cpu: &mut Cpu| rl(cpu, Mem(Reg16::HL))
            }),
            0x17 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RL A",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::A)
            }),
            0x18 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RR B",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::B)
            }),
            0x19 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RR C",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::C)
            }),
            0x1A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RR D",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::D)
            }),
            0x1B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RR E",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::E)
            }),
            0x1C => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RR H",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::H)
            }),
            0x1D => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RR L",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::L)
            }),
            0x1E => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RR (HL)",
                execute: |cpu: &mut Cpu| rr(cpu, Mem(Reg16::HL))
            }),
            0x1F => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RR A",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::A)
            }),
            0x20 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SLA B",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::B)
            }),
            0x21 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SLA C",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::C)
            }),
            0x22 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SLA D",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::D)
            }),
            0x23 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SLA E",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::E)
            }),
            0x24 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SLA H",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::H)
            }),
            0x25 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SLA L",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::L)
            }),
            0x26 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SLA (HL)",
                execute: |cpu: &mut Cpu| sla(cpu, Mem(Reg16::HL))
            }),
            0x27 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SLA A",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::A)
            }),
            0x28 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRA B",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::B)
            }),
            0x29 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRA C",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::C)
            }),
            0x2A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRA D",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::D)
            }),
            0x2B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRA E",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::E)
            }),
            0x2C => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRA H",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::H)
            }),
            0x2D => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRA L",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::L)
            }),
            0x2E => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SRA (HL)",
                execute: |cpu: &mut Cpu| sra(cpu, Mem(Reg16::HL))
            }),
            0x2F => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRA A",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::A)
            }),
            0x30 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SWAP B",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::B)
            }),
            0x31 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SWAP C",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::C)
            }),
            0x32 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SWAP D",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::D)
            }),
            0x33 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SWAP E",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::E)
            }),
            0x34 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SWAP H",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::H)
            }),
            0x35 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SWAP L",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::L)
            }),
            0x36 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SWAP (HL)",
                execute: |cpu: &mut Cpu| swap(cpu, Mem(Reg16::HL))
            }),
            0x37 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SWAP A",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::A)
            }),
            0x38 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRL B",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::B)
            }),
            0x39 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRL C",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::C)
            }),
            0x3A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRL D",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::D)
            }),
            0x3B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRL E",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::E)
            }),
            0x3C => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRL H",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::H)
            }),
            0x3D => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRL L",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::L)
            }),
            0x3E => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SRL (HL)",
                execute: |cpu: &mut Cpu| srl(cpu, Mem(Reg16::HL))
            }),
            0x3F => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SRL A",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::A)
            }),
            0x40 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 0,B",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::B)
            }),
            0x41 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 0,C",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::C)
            }),
            0x42 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 0,D",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::D)
            }),
            0x43 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 0,E",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::E)
            }),
            0x44 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 0,H",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::H)
            }),
            0x45 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 0,L",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::L)
            }),
            0x46 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "BIT 0,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Mem(Reg16::HL))
            }),
            0x47 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 0,A",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::A)
            }),
            0x48 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 1,B",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::B)
            }),
            0x49 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 1,C",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::C)
            }),
            0x4A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 1,D",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::D)
            }),
            0x4B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 1,E",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::E)
            }),
            0x4C => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 1,H",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::H)
            }),
            0x4D => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 1,L",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::L)
            }),
            0x4E => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "BIT 1,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Mem(Reg16::HL))
            }),
            0x4F => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 1,A",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::A)
            }),
            0x50 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 2,B",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::B)
            }),
            0x51 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 2,C",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::C)
            }),
            0x52 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 2,D",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::D)
            }),
            0x53 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 2,E",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::E)
            }),
            0x54 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 2,H",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::H)
            }),
            0x55 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 2,L",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::L)
            }),
            0x56 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "BIT 2,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Mem(Reg16::HL))
            }),
            0x57 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 2,A",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::A)
            }),
            0x58 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 3,B",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::B)
            }),
            0x59 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 3,C",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::C)
            }),
            0x5A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 3,D",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::D)
            }),
            0x5B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 3,E",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::E)
            }),
            0x5C => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 3,H",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::H)
            }),
            0x5D => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 3,L",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::L)
            }),
            0x5E => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "BIT 3,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Mem(Reg16::HL))
            }),
            0x5F => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 3,A",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::A)
            }),
            0x60 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 4,B",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::B)
            }),
            0x61 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 4,C",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::C)
            }),
            0x62 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 4,D",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::D)
            }),
            0x63 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 4,E",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::E)
            }),
            0x64 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 4,H",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::H)
            }),
            0x65 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 4,L",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::L)
            }),
            0x66 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "BIT 4,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Mem(Reg16::HL))
            }),
            0x67 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 4,A",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::A)
            }),
            0x68 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 5,B",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::B)
            }),
            0x69 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 5,C",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::C)
            }),
            0x6A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 5,D",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::D)
            }),
            0x6B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 5,E",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::E)
            }),
            0x6C => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 5,H",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::H)
            }),
            0x6D => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 5,L",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::L)
            }),
            0x6E => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "BIT 5,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Mem(Reg16::HL))
            }),
            0x6F => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 5,A",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::A)
            }),
            0x70 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 6,B",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::B)
            }),
            0x71 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 6,C",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::C)
            }),
            0x72 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 6,D",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::D)
            }),
            0x73 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 6,E",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::E)
            }),
            0x74 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 6,H",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::H)
            }),
            0x75 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 6,L",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::L)
            }),
            0x76 => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "BIT 6,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Mem(Reg16::HL))
            }),
            0x77 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 6,A",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::A)
            }),
            0x78 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 7,B",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::B)
            }),
            0x79 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 7,C",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::C)
            }),
            0x7A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 7,D",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::D)
            }),
            0x7B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 7,E",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::E)
            }),
            0x7C => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 7,H",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::H)
            }),
            0x7D => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 7,L",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::L)
            }),
            0x7E => Some(&Instruction {
                c_cycles: 12,
                conditional_c_cycles: None,
                mnemonic: "BIT 7,(HL)",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Mem(Reg16::HL))
            }),
            0x7F => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "BIT 7,A",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::A)
            }),
            0x80 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 0,B",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::B)
            }),
            0x81 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 0,C",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::C)
            }),
            0x82 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 0,D",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::D)
            }),
            0x83 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 0,E",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::E)
            }),
            0x84 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 0,H",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::H)
            }),
            0x85 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 0,L",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::L)
            }),
            0x86 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RES 0,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 0, Mem(Reg16::HL))
            }),
            0x87 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 0,A",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::A)
            }),
            0x88 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 1,B",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::B)
            }),
            0x89 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 1,C",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::C)
            }),
            0x8A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 1,D",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::D)
            }),
            0x8B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 1,E",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::E)
            }),
            0x8C => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 1,H",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::H)
            }),
            0x8D => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 1,L",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::L)
            }),
            0x8E => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RES 1,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 1, Mem(Reg16::HL))
            }),
            0x8F => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 1,A",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::A)
            }),
            0x90 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 2,B",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::B)
            }),
            0x91 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 2,C",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::C)
            }),
            0x92 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 2,D",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::D)
            }),
            0x93 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 2,E",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::E)
            }),
            0x94 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 2,H",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::H)
            }),
            0x95 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 2,L",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::L)
            }),
            0x96 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RES 2,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 2, Mem(Reg16::HL))
            }),
            0x97 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 2,A",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::A)
            }),
            0x98 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 3,B",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::B)
            }),
            0x99 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 3,C",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::C)
            }),
            0x9A => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 3,D",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::D)
            }),
            0x9B => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 3,E",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::E)
            }),
            0x9C => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 3,H",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::H)
            }),
            0x9D => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 3,L",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::L)
            }),
            0x9E => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RES 3,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 3, Mem(Reg16::HL))
            }),
            0x9F => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 3,A",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::A)
            }),
            0xA0 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 4,B",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::B)
            }),
            0xA1 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 4,C",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::C)
            }),
            0xA2 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 4,D",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::D)
            }),
            0xA3 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 4,E",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::E)
            }),
            0xA4 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 4,H",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::H)
            }),
            0xA5 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 4,L",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::L)
            }),
            0xA6 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RES 4,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 4, Mem(Reg16::HL))
            }),
            0xA7 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 4,A",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::A)
            }),
            0xA8 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 5,B",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::B)
            }),
            0xA9 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 5,C",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::C)
            }),
            0xAA => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 5,D",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::D)
            }),
            0xAB => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 5,E",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::E)
            }),
            0xAC => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 5,H",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::H)
            }),
            0xAD => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 5,L",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::L)
            }),
            0xAE => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RES 5,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 5, Mem(Reg16::HL))
            }),
            0xAF => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 5,A",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::A)
            }),
            0xB0 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 6,B",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::B)
            }),
            0xB1 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 6,C",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::C)
            }),
            0xB2 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 6,D",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::D)
            }),
            0xB3 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 6,E",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::E)
            }),
            0xB4 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 6,H",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::H)
            }),
            0xB5 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 6,L",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::L)
            }),
            0xB6 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RES 6,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 6, Mem(Reg16::HL))
            }),
            0xB7 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 6,A",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::A)
            }),
            0xB8 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 7,B",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::B)
            }),
            0xB9 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 7,C",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::C)
            }),
            0xBA => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 7,D",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::D)
            }),
            0xBB => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 7,E",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::E)
            }),
            0xBC => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 7,H",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::H)
            }),
            0xBD => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 7,L",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::L)
            }),
            0xBE => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "RES 7,(HL)",
                execute: |cpu: &mut Cpu| res(cpu, 7, Mem(Reg16::HL))
            }),
            0xBF => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "RES 7,A",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::A)
            }),
            0xC0 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 0,B",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::B)
            }),
            0xC1 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 0,C",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::C)
            }),
            0xC2 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 0,D",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::D)
            }),
            0xC3 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 0,E",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::E)
            }),
            0xC4 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 0,H",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::H)
            }),
            0xC5 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 0,L",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::L)
            }),
            0xC6 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SET 0,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 0, Mem(Reg16::HL))
            }),
            0xC7 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 0,A",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::A)
            }),
            0xC8 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 1,B",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::B)
            }),
            0xC9 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 1,C",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::C)
            }),
            0xCA => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 1,D",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::D)
            }),
            0xCB => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 1,E",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::E)
            }),
            0xCC => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 1,H",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::H)
            }),
            0xCD => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 1,L",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::L)
            }),
            0xCE => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SET 1,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 1, Mem(Reg16::HL))
            }),
            0xCF => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 1,A",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::A)
            }),
            0xD0 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 2,B",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::B)
            }),
            0xD1 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 2,C",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::C)
            }),
            0xD2 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 2,D",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::D)
            }),
            0xD3 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 2,E",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::E)
            }),
            0xD4 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 2,H",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::H)
            }),
            0xD5 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 2,L",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::L)
            }),
            0xD6 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SET 2,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 2, Mem(Reg16::HL))
            }),
            0xD7 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 2,A",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::A)
            }),
            0xD8 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 3,B",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::B)
            }),
            0xD9 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 3,C",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::C)
            }),
            0xDA => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 3,D",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::D)
            }),
            0xDB => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 3,E",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::E)
            }),
            0xDC => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 3,H",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::H)
            }),
            0xDD => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 3,L",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::L)
            }),
            0xDE => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SET 3,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 3, Mem(Reg16::HL))
            }),
            0xDF => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 3,A",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::A)
            }),
            0xE0 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 4,B",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::B)
            }),
            0xE1 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 4,C",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::C)
            }),
            0xE2 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 4,D",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::D)
            }),
            0xE3 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 4,E",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::E)
            }),
            0xE4 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 4,H",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::H)
            }),
            0xE5 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 4,L",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::L)
            }),
            0xE6 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SET 4,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 4, Mem(Reg16::HL))
            }),
            0xE7 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 4,A",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::A)
            }),
            0xE8 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 5,B",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::B)
            }),
            0xE9 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 5,C",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::C)
            }),
            0xEA => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 5,D",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::D)
            }),
            0xEB => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 5,E",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::E)
            }),
            0xEC => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 5,H",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::H)
            }),
            0xED => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 5,L",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::L)
            }),
            0xEE => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SET 5,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 5, Mem(Reg16::HL))
            }),
            0xEF => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 5,A",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::A)
            }),
            0xF0 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 6,B",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::B)
            }),
            0xF1 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 6,C",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::C)
            }),
            0xF2 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 6,D",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::D)
            }),
            0xF3 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 6,E",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::E)
            }),
            0xF4 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 6,H",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::H)
            }),
            0xF5 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 6,L",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::L)
            }),
            0xF6 => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SET 6,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 6, Mem(Reg16::HL))
            }),
            0xF7 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 6,A",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::A)
            }),
            0xF8 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 7,B",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::B)
            }),
            0xF9 => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 7,C",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::C)
            }),
            0xFA => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 7,D",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::D)
            }),
            0xFB => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 7,E",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::E)
            }),
            0xFC => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 7,H",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::H)
            }),
            0xFD => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 7,L",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::L)
            }),
            0xFE => Some(&Instruction {
                c_cycles: 16,
                conditional_c_cycles: None,
                mnemonic: "SET 7,(HL)",
                execute: |cpu: &mut Cpu| set(cpu, 7, Mem(Reg16::HL))
            }),
            0xFF => Some(&Instruction {
                c_cycles: 8,
                conditional_c_cycles: None,
                mnemonic: "SET 7,A",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::A)
            }),
            _ => panic!("Invalid OPCODE 0x{:02X}", opcode),
        }
    }
}