// @generated
#![allow(unused)]
use std::io::stdin;
use std::ops::{Shl, Shr};
use std::ptr::null;

use super::cpu::{self, Cpu, DMem, Dst, Imm16, Imm8, Mem, Src};
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
    pub fn from_opcode(opcode: Opcode) -> Instruction {
        match opcode {
            Opcode::Unprefixed(op) => Instruction::from_opcode_unprefixed(op),
            Opcode::Prefixed(op) => Instruction::from_opcode_prefixed(op),
        }
    }

    fn from_opcode_unprefixed(opcode: u8) -> Instruction {
        match opcode {
            0x00 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "nop",
                execute: |_: &mut Cpu| nop(),
            },
            0x01 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "ld bc, d16",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::BC, Imm16),
            },
            0x02 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld (bc), a",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::BC), Reg8::A),
            },
            0x03 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "inc bc",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::BC),
            },
            0x04 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "inc b",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::B),
            },
            0x05 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "dec b",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::B),
            },
            0x06 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld b, d8",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Imm8),
            },
            0x07 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "rlca",
                execute: |cpu: &mut Cpu| rlca(cpu),
            },
            0x08 => Instruction {
                cycles: Cycles::Unconditional(20),
                mnemonic: "ld (d16), sp",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Imm16), Reg16::SP),
            },
            0x09 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "add hl, bc",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::BC),
            },
            0x0A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld a, (bc)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Reg16::BC)),
            },
            0x0B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "dec bc",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::BC),
            },
            0x0C => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "inc c",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::C),
            },
            0x0D => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "dec c",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::C),
            },
            0x0E => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld c, d8",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Imm8),
            },
            0x0F => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "rrca",
                execute: |cpu: &mut Cpu| rrca(cpu),
            },
            0x10 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "stop 0x00",
                execute: |cpu: &mut Cpu| {
                    cpu.stop();
                    Timing::Normal
                },
            },
            0x11 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "ld de, d16",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::DE, Imm16),
            },
            0x12 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld (de), a",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::DE), Reg8::A),
            },
            0x13 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "inc de",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::DE),
            },
            0x14 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "inc d",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::D),
            },
            0x15 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "dec d",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::D),
            },
            0x16 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld d, d8",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Imm8),
            },
            0x17 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "rla",
                execute: |cpu: &mut Cpu| rla(cpu),
            },
            0x18 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "jr r8",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::Unconditional, Imm8),
            },
            0x19 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "add hl, de",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::DE),
            },
            0x1A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld a, (de)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Reg16::DE)),
            },
            0x1B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "dec de",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::DE),
            },
            0x1C => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "inc e",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::E),
            },
            0x1D => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "dec e",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::E),
            },
            0x1E => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld e, d8",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Imm8),
            },
            0x1F => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "rra",
                execute: |cpu: &mut Cpu| rra(cpu),
            },
            0x20 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 12,
                }),
                mnemonic: "jr nz, r8",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::NotZero, Imm8),
            },
            0x21 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "ld hl, d16",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::HL, Imm16),
            },
            0x22 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ldi (hl), a",
                execute: |cpu: &mut Cpu| ldi(cpu, Mem(Reg16::HL), Reg8::A, Reg16::HL),
            },
            0x23 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "inc hl",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::HL),
            },
            0x24 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "inc h",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::H),
            },
            0x25 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "dec h",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::H),
            },
            0x26 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld h, d8",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Imm8),
            },
            0x27 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "daa",
                execute: |cpu: &mut Cpu| daa(cpu),
            },
            0x28 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 12,
                }),
                mnemonic: "jr z, r8",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::Zero, Imm8),
            },
            0x29 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "add hl, hl",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::HL),
            },
            0x2A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ldi a, (hl)",
                execute: |cpu: &mut Cpu| ldi(cpu, Reg8::A, Mem(Reg16::HL), Reg16::HL),
            },
            0x2B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "dec hl",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::HL),
            },
            0x2C => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "inc l",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::L),
            },
            0x2D => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "dec l",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::L),
            },
            0x2E => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld l, d8",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Imm8),
            },
            0x2F => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "cpl",
                execute: |cpu: &mut Cpu| cpl(cpu),
            },
            0x30 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 12,
                }),
                mnemonic: "jr nc, r8",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::NotCarry, Imm8),
            },
            0x31 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "ld sp, d16",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::SP, Imm16),
            },
            0x32 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ldd (hl), a",
                execute: |cpu: &mut Cpu| ldd(cpu, Mem(Reg16::HL), Reg8::A, Reg16::HL),
            },
            0x33 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "inc sp",
                execute: |cpu: &mut Cpu| inc16(cpu, Reg16::SP),
            },
            0x34 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "inc (hl)",
                execute: |cpu: &mut Cpu| inc(cpu, Mem(Reg16::HL)),
            },
            0x35 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "dec (hl)",
                execute: |cpu: &mut Cpu| dec(cpu, Mem(Reg16::HL)),
            },
            0x36 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "ld (hl), d8",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Imm8),
            },
            0x37 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "scf",
                execute: |cpu: &mut Cpu| scf(cpu),
            },
            0x38 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 12,
                }),
                mnemonic: "jr c, r8",
                execute: |cpu: &mut Cpu| jr(cpu, Condition::Carry, Imm8),
            },
            0x39 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "add hl, sp",
                execute: |cpu: &mut Cpu| add16(cpu, Reg16::HL, Reg16::SP),
            },
            0x3A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ldd a, (hl)",
                execute: |cpu: &mut Cpu| ldd(cpu, Reg8::A, Mem(Reg16::HL), Reg16::HL),
            },
            0x3B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "dec sp",
                execute: |cpu: &mut Cpu| dec16(cpu, Reg16::SP),
            },
            0x3C => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "inc a",
                execute: |cpu: &mut Cpu| inc(cpu, Reg8::A),
            },
            0x3D => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "dec a",
                execute: |cpu: &mut Cpu| dec(cpu, Reg8::A),
            },
            0x3E => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld a, d8",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Imm8),
            },
            0x3F => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ccf",
                execute: |cpu: &mut Cpu| ccf(cpu),
            },
            0x40 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld b, b",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::B),
            },
            0x41 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld b, c",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::C),
            },
            0x42 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld b, d",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::D),
            },
            0x43 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld b, e",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::E),
            },
            0x44 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld b, h",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::H),
            },
            0x45 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld b, l",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::L),
            },
            0x46 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld b, (hl)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Mem(Reg16::HL)),
            },
            0x47 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld b, a",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::B, Reg8::A),
            },
            0x48 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld c, b",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::B),
            },
            0x49 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld c, c",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::C),
            },
            0x4A => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld c, d",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::D),
            },
            0x4B => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld c, e",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::E),
            },
            0x4C => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld c, h",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::H),
            },
            0x4D => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld c, l",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::L),
            },
            0x4E => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld c, (hl)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Mem(Reg16::HL)),
            },
            0x4F => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld c, a",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::C, Reg8::A),
            },
            0x50 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld d, b",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::B),
            },
            0x51 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld d, c",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::C),
            },
            0x52 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld d, d",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::D),
            },
            0x53 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld d, e",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::E),
            },
            0x54 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld d, h",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::H),
            },
            0x55 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld d, l",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::L),
            },
            0x56 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld d, (hl)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Mem(Reg16::HL)),
            },
            0x57 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld d, a",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::D, Reg8::A),
            },
            0x58 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld e, b",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::B),
            },
            0x59 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld e, c",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::C),
            },
            0x5A => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld e, d",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::D),
            },
            0x5B => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld e, e",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::E),
            },
            0x5C => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld e, h",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::H),
            },
            0x5D => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld e, l",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::L),
            },
            0x5E => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld e, (hl)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Mem(Reg16::HL)),
            },
            0x5F => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld e, a",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::E, Reg8::A),
            },
            0x60 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld h, b",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::B),
            },
            0x61 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld h, c",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::C),
            },
            0x62 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld h, d",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::D),
            },
            0x63 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld h, e",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::E),
            },
            0x64 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld h, h",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::H),
            },
            0x65 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld h, l",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::L),
            },
            0x66 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld h, (hl)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Mem(Reg16::HL)),
            },
            0x67 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld h, a",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::H, Reg8::A),
            },
            0x68 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld l, b",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::B),
            },
            0x69 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld l, c",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::C),
            },
            0x6A => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld l, d",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::D),
            },
            0x6B => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld l, e",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::E),
            },
            0x6C => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld l, h",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::H),
            },
            0x6D => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld l, l",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::L),
            },
            0x6E => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld l, (hl)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Mem(Reg16::HL)),
            },
            0x6F => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld l, a",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::L, Reg8::A),
            },
            0x70 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld (hl), b",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::B),
            },
            0x71 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld (hl), c",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::C),
            },
            0x72 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld (hl), d",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::D),
            },
            0x73 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld (hl), e",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::E),
            },
            0x74 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld (hl), h",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::H),
            },
            0x75 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld (hl), l",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::L),
            },
            0x76 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "halt",
                execute: |cpu: &mut Cpu| halt(cpu),
            },
            0x77 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld (hl), a",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Reg16::HL), Reg8::A),
            },
            0x78 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld a, b",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::B),
            },
            0x79 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld a, c",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::C),
            },
            0x7A => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld a, d",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::D),
            },
            0x7B => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld a, e",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::E),
            },
            0x7C => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld a, h",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::H),
            },
            0x7D => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld a, l",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::L),
            },
            0x7E => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld a, (hl)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Reg16::HL)),
            },
            0x7F => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ld a, a",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Reg8::A),
            },
            0x80 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "add a, b",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::B),
            },
            0x81 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "add a, c",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::C),
            },
            0x82 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "add a, d",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::D),
            },
            0x83 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "add a, e",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::E),
            },
            0x84 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "add a, h",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::H),
            },
            0x85 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "add a, l",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::L),
            },
            0x86 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "add a, (hl)",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Mem(Reg16::HL)),
            },
            0x87 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "add a, a",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Reg8::A),
            },
            0x88 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "adc a, b",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::B),
            },
            0x89 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "adc a, c",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::C),
            },
            0x8A => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "adc a, d",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::D),
            },
            0x8B => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "adc a, e",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::E),
            },
            0x8C => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "adc a, h",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::H),
            },
            0x8D => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "adc a, l",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::L),
            },
            0x8E => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "adc a, (hl)",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Mem(Reg16::HL)),
            },
            0x8F => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "adc a, a",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Reg8::A),
            },
            0x90 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sub b",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::B),
            },
            0x91 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sub c",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::C),
            },
            0x92 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sub d",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::D),
            },
            0x93 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sub e",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::E),
            },
            0x94 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sub h",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::H),
            },
            0x95 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sub l",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::L),
            },
            0x96 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sub (hl)",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Mem(Reg16::HL)),
            },
            0x97 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sub a",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Reg8::A),
            },
            0x98 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sbc a, b",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::B),
            },
            0x99 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sbc a, c",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::C),
            },
            0x9A => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sbc a, d",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::D),
            },
            0x9B => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sbc a, e",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::E),
            },
            0x9C => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sbc a, h",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::H),
            },
            0x9D => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sbc a, l",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::L),
            },
            0x9E => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sbc a, (hl)",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Mem(Reg16::HL)),
            },
            0x9F => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "sbc a, a",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Reg8::A),
            },
            0xA0 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "and b",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::B),
            },
            0xA1 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "and c",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::C),
            },
            0xA2 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "and d",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::D),
            },
            0xA3 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "and e",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::E),
            },
            0xA4 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "and h",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::H),
            },
            0xA5 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "and l",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::L),
            },
            0xA6 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "and (hl)",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Mem(Reg16::HL)),
            },
            0xA7 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "and a",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Reg8::A),
            },
            0xA8 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "xor b",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::B),
            },
            0xA9 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "xor c",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::C),
            },
            0xAA => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "xor d",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::D),
            },
            0xAB => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "xor e",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::E),
            },
            0xAC => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "xor h",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::H),
            },
            0xAD => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "xor l",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::L),
            },
            0xAE => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "xor (hl)",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Mem(Reg16::HL)),
            },
            0xAF => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "xor a",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Reg8::A),
            },
            0xB0 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "or b",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::B),
            },
            0xB1 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "or c",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::C),
            },
            0xB2 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "or d",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::D),
            },
            0xB3 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "or e",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::E),
            },
            0xB4 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "or h",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::H),
            },
            0xB5 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "or l",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::L),
            },
            0xB6 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "or (hl)",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Mem(Reg16::HL)),
            },
            0xB7 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "or a",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Reg8::A),
            },
            0xB8 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "cp b",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::B),
            },
            0xB9 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "cp c",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::C),
            },
            0xBA => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "cp d",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::D),
            },
            0xBB => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "cp e",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::E),
            },
            0xBC => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "cp h",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::H),
            },
            0xBD => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "cp l",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::L),
            },
            0xBE => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "cp (hl)",
                execute: |cpu: &mut Cpu| cp(cpu, Mem(Reg16::HL)),
            },
            0xBF => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "cp a",
                execute: |cpu: &mut Cpu| cp(cpu, Reg8::A),
            },
            0xC0 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 20,
                }),
                mnemonic: "ret nz",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::NotZero),
            },
            0xC1 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "pop bc",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::BC),
            },
            0xC2 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 16,
                }),
                mnemonic: "jp nz, d16",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::NotZero, Imm16),
            },
            0xC3 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "jp d16",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Unconditional, Imm16),
            },
            0xC4 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 24,
                }),
                mnemonic: "call nz, d16",
                execute: |cpu: &mut Cpu| call(cpu, Condition::NotZero, Imm16),
            },
            0xC5 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "push bc",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::BC),
            },
            0xC6 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "add a, d8",
                execute: |cpu: &mut Cpu| add(cpu, Reg8::A, Imm8),
            },
            0xC7 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rst 0x00",
                execute: |cpu: &mut Cpu| rst(cpu, 0x00),
            },
            0xC8 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 20,
                }),
                mnemonic: "ret z",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::Zero),
            },
            0xC9 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "ret",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::Unconditional),
            },
            0xCA => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 16,
                }),
                mnemonic: "jp z, d16",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Zero, Imm16),
            },
            0xCC => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 24,
                }),
                mnemonic: "call z, d16",
                execute: |cpu: &mut Cpu| call(cpu, Condition::Zero, Imm16),
            },
            0xCD => Instruction {
                cycles: Cycles::Unconditional(24),
                mnemonic: "call d16",
                execute: |cpu: &mut Cpu| call(cpu, Condition::Unconditional, Imm16),
            },
            0xCE => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "adc a, d8",
                execute: |cpu: &mut Cpu| adc(cpu, Reg8::A, Imm8),
            },
            0xCF => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rst 0x08",
                execute: |cpu: &mut Cpu| rst(cpu, 0x08),
            },
            0xD0 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 20,
                }),
                mnemonic: "ret nc",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::NotCarry),
            },
            0xD1 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "pop de",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::DE),
            },
            0xD2 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 16,
                }),
                mnemonic: "jp nc, d16",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::NotCarry, Imm16),
            },
            0xD4 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 24,
                }),
                mnemonic: "call nc, d16",
                execute: |cpu: &mut Cpu| call(cpu, Condition::NotCarry, Imm16),
            },
            0xD5 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "push de",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::DE),
            },
            0xD6 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sub d8",
                execute: |cpu: &mut Cpu| sub(cpu, Reg8::A, Imm8),
            },
            0xD7 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rst 0x10",
                execute: |cpu: &mut Cpu| rst(cpu, 0x10),
            },
            0xD8 => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 8,
                    taken: 20,
                }),
                mnemonic: "ret c",
                execute: |cpu: &mut Cpu| ret(cpu, Condition::Carry),
            },
            0xD9 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "reti",
                execute: |cpu: &mut Cpu| reti(cpu),
            },
            0xDA => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 16,
                }),
                mnemonic: "jp c, d16",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Carry, Imm16),
            },
            0xDC => Instruction {
                cycles: Cycles::Conditional(ConditionCycles {
                    not_taken: 12,
                    taken: 24,
                }),
                mnemonic: "call c, d16",
                execute: |cpu: &mut Cpu| call(cpu, Condition::Carry, Imm16),
            },
            0xDE => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sbc a, d8",
                execute: |cpu: &mut Cpu| sbc(cpu, Reg8::A, Imm8),
            },
            0xDF => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rst 0x18",
                execute: |cpu: &mut Cpu| rst(cpu, 0x18),
            },
            0xE0 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "ld (0xff00 + d8), a",
                execute: |cpu: &mut Cpu| ld(cpu, DMem(Imm8), Reg8::A),
            },
            0xE1 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "pop hl",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::HL),
            },
            0xE2 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld (0xff00 + c), a",
                execute: |cpu: &mut Cpu| ld(cpu, DMem(Reg8::C), Reg8::A),
            },
            0xE5 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "push hl",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::HL),
            },
            0xE6 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "and d8",
                execute: |cpu: &mut Cpu| and(cpu, Reg8::A, Imm8),
            },
            0xE7 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rst 0x20",
                execute: |cpu: &mut Cpu| rst(cpu, 0x20),
            },
            0xE8 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "add sp, r8",
                execute: |cpu: &mut Cpu| add_sp(cpu),
            },
            0xE9 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "jp hl",
                execute: |cpu: &mut Cpu| jp(cpu, Condition::Unconditional, Reg16::HL),
            },
            0xEA => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "ld (d16), a",
                execute: |cpu: &mut Cpu| ld(cpu, Mem(Imm16), Reg8::A),
            },
            0xEE => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "xor d8",
                execute: |cpu: &mut Cpu| xor(cpu, Reg8::A, Imm8),
            },
            0xEF => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rst 0x28",
                execute: |cpu: &mut Cpu| rst(cpu, 0x28),
            },
            0xF0 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "ld a, (0xff00 + d8)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, DMem(Imm8)),
            },
            0xF1 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "pop af",
                execute: |cpu: &mut Cpu| pop(cpu, Reg16::AF),
            },
            0xF2 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld a, (0xff00 + c)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, DMem(Reg8::C)),
            },
            0xF3 => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "di",
                execute: |cpu: &mut Cpu| di(cpu),
            },
            0xF5 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "push af",
                execute: |cpu: &mut Cpu| push(cpu, Reg16::AF),
            },
            0xF6 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "or d8",
                execute: |cpu: &mut Cpu| or(cpu, Reg8::A, Imm8),
            },
            0xF7 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rst 0x30",
                execute: |cpu: &mut Cpu| rst(cpu, 0x30),
            },
            0xF8 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "ldhl sp, r8",
                execute: |cpu: &mut Cpu| ldhl(cpu),
            },
            0xF9 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "ld sp, hl",
                execute: |cpu: &mut Cpu| ld(cpu, Reg16::SP, Reg16::HL),
            },
            0xFA => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "ld a, (d16)",
                execute: |cpu: &mut Cpu| ld(cpu, Reg8::A, Mem(Imm16)),
            },
            0xFB => Instruction {
                cycles: Cycles::Unconditional(4),
                mnemonic: "ei",
                execute: |cpu: &mut Cpu| ei(cpu),
            },
            0xFE => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "cp d8",
                execute: |cpu: &mut Cpu| cp(cpu, Imm8),
            },
            0xFF => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rst 0x38",
                execute: |cpu: &mut Cpu| rst(cpu, 0x38),
            },
            _ => ILLEGAL,
        }
    }

    fn from_opcode_prefixed(opcode: u8) -> Instruction {
        match opcode {
            0x00 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rlc b",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::B),
            },
            0x01 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rlc c",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::C),
            },
            0x02 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rlc d",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::D),
            },
            0x03 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rlc e",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::E),
            },
            0x04 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rlc h",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::H),
            },
            0x05 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rlc l",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::L),
            },
            0x06 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rlc (hl)",
                execute: |cpu: &mut Cpu| rlc(cpu, Mem(Reg16::HL)),
            },
            0x07 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rlc a",
                execute: |cpu: &mut Cpu| rlc(cpu, Reg8::A),
            },
            0x08 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rrc b",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::B),
            },
            0x09 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rrc c",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::C),
            },
            0x0A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rrc d",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::D),
            },
            0x0B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rrc e",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::E),
            },
            0x0C => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rrc h",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::H),
            },
            0x0D => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rrc l",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::L),
            },
            0x0E => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rrc (hl)",
                execute: |cpu: &mut Cpu| rrc(cpu, Mem(Reg16::HL)),
            },
            0x0F => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rrc a",
                execute: |cpu: &mut Cpu| rrc(cpu, Reg8::A),
            },
            0x10 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rl b",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::B),
            },
            0x11 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rl c",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::C),
            },
            0x12 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rl d",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::D),
            },
            0x13 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rl e",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::E),
            },
            0x14 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rl h",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::H),
            },
            0x15 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rl l",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::L),
            },
            0x16 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rl (hl)",
                execute: |cpu: &mut Cpu| rl(cpu, Mem(Reg16::HL)),
            },
            0x17 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rl a",
                execute: |cpu: &mut Cpu| rl(cpu, Reg8::A),
            },
            0x18 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rr b",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::B),
            },
            0x19 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rr c",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::C),
            },
            0x1A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rr d",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::D),
            },
            0x1B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rr e",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::E),
            },
            0x1C => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rr h",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::H),
            },
            0x1D => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rr l",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::L),
            },
            0x1E => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "rr (hl)",
                execute: |cpu: &mut Cpu| rr(cpu, Mem(Reg16::HL)),
            },
            0x1F => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "rr a",
                execute: |cpu: &mut Cpu| rr(cpu, Reg8::A),
            },
            0x20 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sla b",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::B),
            },
            0x21 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sla c",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::C),
            },
            0x22 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sla d",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::D),
            },
            0x23 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sla e",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::E),
            },
            0x24 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sla h",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::H),
            },
            0x25 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sla l",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::L),
            },
            0x26 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "sla (hl)",
                execute: |cpu: &mut Cpu| sla(cpu, Mem(Reg16::HL)),
            },
            0x27 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sla a",
                execute: |cpu: &mut Cpu| sla(cpu, Reg8::A),
            },
            0x28 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sra b",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::B),
            },
            0x29 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sra c",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::C),
            },
            0x2A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sra d",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::D),
            },
            0x2B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sra e",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::E),
            },
            0x2C => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sra h",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::H),
            },
            0x2D => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sra l",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::L),
            },
            0x2E => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "sra (hl)",
                execute: |cpu: &mut Cpu| sra(cpu, Mem(Reg16::HL)),
            },
            0x2F => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "sra a",
                execute: |cpu: &mut Cpu| sra(cpu, Reg8::A),
            },
            0x30 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "swap b",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::B),
            },
            0x31 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "swap c",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::C),
            },
            0x32 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "swap d",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::D),
            },
            0x33 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "swap e",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::E),
            },
            0x34 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "swap h",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::H),
            },
            0x35 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "swap l",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::L),
            },
            0x36 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "swap (hl)",
                execute: |cpu: &mut Cpu| swap(cpu, Mem(Reg16::HL)),
            },
            0x37 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "swap a",
                execute: |cpu: &mut Cpu| swap(cpu, Reg8::A),
            },
            0x38 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "srl b",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::B),
            },
            0x39 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "srl c",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::C),
            },
            0x3A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "srl d",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::D),
            },
            0x3B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "srl e",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::E),
            },
            0x3C => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "srl h",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::H),
            },
            0x3D => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "srl l",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::L),
            },
            0x3E => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "srl (hl)",
                execute: |cpu: &mut Cpu| srl(cpu, Mem(Reg16::HL)),
            },
            0x3F => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "srl a",
                execute: |cpu: &mut Cpu| srl(cpu, Reg8::A),
            },
            0x40 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 0, b",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::B),
            },
            0x41 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 0, c",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::C),
            },
            0x42 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 0, d",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::D),
            },
            0x43 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 0, e",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::E),
            },
            0x44 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 0, h",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::H),
            },
            0x45 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 0, l",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::L),
            },
            0x46 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "bit 0, (hl)",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Mem(Reg16::HL)),
            },
            0x47 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 0, a",
                execute: |cpu: &mut Cpu| bit(cpu, 0, Reg8::A),
            },
            0x48 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 1, b",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::B),
            },
            0x49 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 1, c",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::C),
            },
            0x4A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 1, d",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::D),
            },
            0x4B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 1, e",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::E),
            },
            0x4C => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 1, h",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::H),
            },
            0x4D => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 1, l",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::L),
            },
            0x4E => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "bit 1, (hl)",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Mem(Reg16::HL)),
            },
            0x4F => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 1, a",
                execute: |cpu: &mut Cpu| bit(cpu, 1, Reg8::A),
            },
            0x50 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 2, b",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::B),
            },
            0x51 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 2, c",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::C),
            },
            0x52 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 2, d",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::D),
            },
            0x53 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 2, e",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::E),
            },
            0x54 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 2, h",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::H),
            },
            0x55 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 2, l",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::L),
            },
            0x56 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "bit 2, (hl)",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Mem(Reg16::HL)),
            },
            0x57 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 2, a",
                execute: |cpu: &mut Cpu| bit(cpu, 2, Reg8::A),
            },
            0x58 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 3, b",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::B),
            },
            0x59 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 3, c",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::C),
            },
            0x5A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 3, d",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::D),
            },
            0x5B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 3, e",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::E),
            },
            0x5C => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 3, h",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::H),
            },
            0x5D => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 3, l",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::L),
            },
            0x5E => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "bit 3, (hl)",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Mem(Reg16::HL)),
            },
            0x5F => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 3, a",
                execute: |cpu: &mut Cpu| bit(cpu, 3, Reg8::A),
            },
            0x60 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 4, b",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::B),
            },
            0x61 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 4, c",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::C),
            },
            0x62 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 4, d",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::D),
            },
            0x63 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 4, e",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::E),
            },
            0x64 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 4, h",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::H),
            },
            0x65 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 4, l",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::L),
            },
            0x66 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "bit 4, (hl)",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Mem(Reg16::HL)),
            },
            0x67 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 4, a",
                execute: |cpu: &mut Cpu| bit(cpu, 4, Reg8::A),
            },
            0x68 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 5, b",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::B),
            },
            0x69 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 5, c",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::C),
            },
            0x6A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 5, d",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::D),
            },
            0x6B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 5, e",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::E),
            },
            0x6C => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 5, h",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::H),
            },
            0x6D => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 5, l",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::L),
            },
            0x6E => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "bit 5, (hl)",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Mem(Reg16::HL)),
            },
            0x6F => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 5, a",
                execute: |cpu: &mut Cpu| bit(cpu, 5, Reg8::A),
            },
            0x70 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 6, b",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::B),
            },
            0x71 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 6, c",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::C),
            },
            0x72 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 6, d",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::D),
            },
            0x73 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 6, e",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::E),
            },
            0x74 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 6, h",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::H),
            },
            0x75 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 6, l",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::L),
            },
            0x76 => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "bit 6, (hl)",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Mem(Reg16::HL)),
            },
            0x77 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 6, a",
                execute: |cpu: &mut Cpu| bit(cpu, 6, Reg8::A),
            },
            0x78 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 7, b",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::B),
            },
            0x79 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 7, c",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::C),
            },
            0x7A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 7, d",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::D),
            },
            0x7B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 7, e",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::E),
            },
            0x7C => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 7, h",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::H),
            },
            0x7D => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 7, l",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::L),
            },
            0x7E => Instruction {
                cycles: Cycles::Unconditional(12),
                mnemonic: "bit 7, (hl)",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Mem(Reg16::HL)),
            },
            0x7F => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "bit 7, a",
                execute: |cpu: &mut Cpu| bit(cpu, 7, Reg8::A),
            },
            0x80 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 0, b",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::B),
            },
            0x81 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 0, c",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::C),
            },
            0x82 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 0, d",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::D),
            },
            0x83 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 0, e",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::E),
            },
            0x84 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 0, h",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::H),
            },
            0x85 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 0, l",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::L),
            },
            0x86 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "res 0, (hl)",
                execute: |cpu: &mut Cpu| res(cpu, 0, Mem(Reg16::HL)),
            },
            0x87 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 0, a",
                execute: |cpu: &mut Cpu| res(cpu, 0, Reg8::A),
            },
            0x88 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 1, b",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::B),
            },
            0x89 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 1, c",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::C),
            },
            0x8A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 1, d",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::D),
            },
            0x8B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 1, e",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::E),
            },
            0x8C => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 1, h",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::H),
            },
            0x8D => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 1, l",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::L),
            },
            0x8E => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "res 1, (hl)",
                execute: |cpu: &mut Cpu| res(cpu, 1, Mem(Reg16::HL)),
            },
            0x8F => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 1, a",
                execute: |cpu: &mut Cpu| res(cpu, 1, Reg8::A),
            },
            0x90 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 2, b",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::B),
            },
            0x91 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 2, c",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::C),
            },
            0x92 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 2, d",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::D),
            },
            0x93 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 2, e",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::E),
            },
            0x94 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 2, h",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::H),
            },
            0x95 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 2, l",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::L),
            },
            0x96 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "res 2, (hl)",
                execute: |cpu: &mut Cpu| res(cpu, 2, Mem(Reg16::HL)),
            },
            0x97 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 2, a",
                execute: |cpu: &mut Cpu| res(cpu, 2, Reg8::A),
            },
            0x98 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 3, b",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::B),
            },
            0x99 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 3, c",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::C),
            },
            0x9A => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 3, d",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::D),
            },
            0x9B => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 3, e",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::E),
            },
            0x9C => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 3, h",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::H),
            },
            0x9D => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 3, l",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::L),
            },
            0x9E => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "res 3, (hl)",
                execute: |cpu: &mut Cpu| res(cpu, 3, Mem(Reg16::HL)),
            },
            0x9F => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 3, a",
                execute: |cpu: &mut Cpu| res(cpu, 3, Reg8::A),
            },
            0xA0 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 4, b",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::B),
            },
            0xA1 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 4, c",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::C),
            },
            0xA2 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 4, d",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::D),
            },
            0xA3 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 4, e",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::E),
            },
            0xA4 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 4, h",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::H),
            },
            0xA5 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 4, l",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::L),
            },
            0xA6 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "res 4, (hl)",
                execute: |cpu: &mut Cpu| res(cpu, 4, Mem(Reg16::HL)),
            },
            0xA7 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 4, a",
                execute: |cpu: &mut Cpu| res(cpu, 4, Reg8::A),
            },
            0xA8 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 5, b",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::B),
            },
            0xA9 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 5, c",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::C),
            },
            0xAA => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 5, d",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::D),
            },
            0xAB => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 5, e",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::E),
            },
            0xAC => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 5, h",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::H),
            },
            0xAD => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 5, l",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::L),
            },
            0xAE => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "res 5, (hl)",
                execute: |cpu: &mut Cpu| res(cpu, 5, Mem(Reg16::HL)),
            },
            0xAF => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 5, a",
                execute: |cpu: &mut Cpu| res(cpu, 5, Reg8::A),
            },
            0xB0 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 6, b",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::B),
            },
            0xB1 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 6, c",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::C),
            },
            0xB2 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 6, d",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::D),
            },
            0xB3 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 6, e",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::E),
            },
            0xB4 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 6, h",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::H),
            },
            0xB5 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 6, l",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::L),
            },
            0xB6 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "res 6, (hl)",
                execute: |cpu: &mut Cpu| res(cpu, 6, Mem(Reg16::HL)),
            },
            0xB7 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 6, a",
                execute: |cpu: &mut Cpu| res(cpu, 6, Reg8::A),
            },
            0xB8 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 7, b",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::B),
            },
            0xB9 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 7, c",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::C),
            },
            0xBA => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 7, d",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::D),
            },
            0xBB => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 7, e",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::E),
            },
            0xBC => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 7, h",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::H),
            },
            0xBD => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 7, l",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::L),
            },
            0xBE => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "res 7, (hl)",
                execute: |cpu: &mut Cpu| res(cpu, 7, Mem(Reg16::HL)),
            },
            0xBF => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "res 7, a",
                execute: |cpu: &mut Cpu| res(cpu, 7, Reg8::A),
            },
            0xC0 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 0, b",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::B),
            },
            0xC1 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 0, c",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::C),
            },
            0xC2 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 0, d",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::D),
            },
            0xC3 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 0, e",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::E),
            },
            0xC4 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 0, h",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::H),
            },
            0xC5 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 0, l",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::L),
            },
            0xC6 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "set 0, (hl)",
                execute: |cpu: &mut Cpu| set(cpu, 0, Mem(Reg16::HL)),
            },
            0xC7 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 0, a",
                execute: |cpu: &mut Cpu| set(cpu, 0, Reg8::A),
            },
            0xC8 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 1, b",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::B),
            },
            0xC9 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 1, c",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::C),
            },
            0xCA => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 1, d",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::D),
            },
            0xCB => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 1, e",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::E),
            },
            0xCC => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 1, h",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::H),
            },
            0xCD => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 1, l",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::L),
            },
            0xCE => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "set 1, (hl)",
                execute: |cpu: &mut Cpu| set(cpu, 1, Mem(Reg16::HL)),
            },
            0xCF => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 1, a",
                execute: |cpu: &mut Cpu| set(cpu, 1, Reg8::A),
            },
            0xD0 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 2, b",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::B),
            },
            0xD1 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 2, c",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::C),
            },
            0xD2 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 2, d",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::D),
            },
            0xD3 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 2, e",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::E),
            },
            0xD4 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 2, h",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::H),
            },
            0xD5 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 2, l",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::L),
            },
            0xD6 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "set 2, (hl)",
                execute: |cpu: &mut Cpu| set(cpu, 2, Mem(Reg16::HL)),
            },
            0xD7 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 2, a",
                execute: |cpu: &mut Cpu| set(cpu, 2, Reg8::A),
            },
            0xD8 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 3, b",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::B),
            },
            0xD9 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 3, c",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::C),
            },
            0xDA => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 3, d",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::D),
            },
            0xDB => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 3, e",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::E),
            },
            0xDC => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 3, h",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::H),
            },
            0xDD => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 3, l",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::L),
            },
            0xDE => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "set 3, (hl)",
                execute: |cpu: &mut Cpu| set(cpu, 3, Mem(Reg16::HL)),
            },
            0xDF => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 3, a",
                execute: |cpu: &mut Cpu| set(cpu, 3, Reg8::A),
            },
            0xE0 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 4, b",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::B),
            },
            0xE1 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 4, c",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::C),
            },
            0xE2 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 4, d",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::D),
            },
            0xE3 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 4, e",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::E),
            },
            0xE4 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 4, h",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::H),
            },
            0xE5 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 4, l",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::L),
            },
            0xE6 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "set 4, (hl)",
                execute: |cpu: &mut Cpu| set(cpu, 4, Mem(Reg16::HL)),
            },
            0xE7 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 4, a",
                execute: |cpu: &mut Cpu| set(cpu, 4, Reg8::A),
            },
            0xE8 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 5, b",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::B),
            },
            0xE9 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 5, c",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::C),
            },
            0xEA => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 5, d",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::D),
            },
            0xEB => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 5, e",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::E),
            },
            0xEC => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 5, h",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::H),
            },
            0xED => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 5, l",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::L),
            },
            0xEE => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "set 5, (hl)",
                execute: |cpu: &mut Cpu| set(cpu, 5, Mem(Reg16::HL)),
            },
            0xEF => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 5, a",
                execute: |cpu: &mut Cpu| set(cpu, 5, Reg8::A),
            },
            0xF0 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 6, b",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::B),
            },
            0xF1 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 6, c",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::C),
            },
            0xF2 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 6, d",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::D),
            },
            0xF3 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 6, e",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::E),
            },
            0xF4 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 6, h",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::H),
            },
            0xF5 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 6, l",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::L),
            },
            0xF6 => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "set 6, (hl)",
                execute: |cpu: &mut Cpu| set(cpu, 6, Mem(Reg16::HL)),
            },
            0xF7 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 6, a",
                execute: |cpu: &mut Cpu| set(cpu, 6, Reg8::A),
            },
            0xF8 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 7, b",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::B),
            },
            0xF9 => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 7, c",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::C),
            },
            0xFA => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 7, d",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::D),
            },
            0xFB => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 7, e",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::E),
            },
            0xFC => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 7, h",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::H),
            },
            0xFD => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 7, l",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::L),
            },
            0xFE => Instruction {
                cycles: Cycles::Unconditional(16),
                mnemonic: "set 7, (hl)",
                execute: |cpu: &mut Cpu| set(cpu, 7, Mem(Reg16::HL)),
            },
            0xFF => Instruction {
                cycles: Cycles::Unconditional(8),
                mnemonic: "set 7, a",
                execute: |cpu: &mut Cpu| set(cpu, 7, Reg8::A),
            },
            _ => ILLEGAL,
        }
    }
}
