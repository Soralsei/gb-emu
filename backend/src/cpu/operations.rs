use crate::is_bit_set;

use super::cpu::{Cpu, Dst, Imem8, Src};
use super::instructions::Condition;
use super::instructions::Timing;
use super::registers::{Reg16, Reg8};
use std::fmt::UpperHex;
use std::ops::{Shl, Shr};

#[inline(always)]
pub fn nop() -> Timing {
    Timing::Normal
}

pub fn ld<T: UpperHex, D: Dst<T>, S: Src<T>>(cpu: &mut Cpu, dst: D, src: S) -> Timing {
    let val: T = src.read(cpu);
    dst.write(cpu, val);
    Timing::Normal
}

pub fn ldi<T: UpperHex, D: Dst<T>, S: Src<T>>(
    cpu: &mut Cpu,
    dst: D,
    src: S,
    to_inc: Reg16,
) -> Timing {
    let t: Timing = ld(cpu, dst, src);
    inc16(cpu, to_inc);
    t
}

pub fn ldd<T: UpperHex, D: Dst<T>, S: Src<T>>(
    cpu: &mut Cpu,
    dst: D,
    src: S,
    to_dec: Reg16,
) -> Timing {
    let t: Timing = ld(cpu, dst, src);
    dec16(cpu, to_dec);
    t
}

pub fn inc<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value: u8 = loc.read(cpu);
    let result = value.wrapping_add(1);
    loc.write(cpu, result);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.subtract = false;
    cpu.registers.f.half_carry = (value & 0x0f) == 0x0f;
    Timing::Normal
}

// no affected flags for inc16
pub fn inc16<L: Dst<u16> + Src<u16> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value: u16 = loc.read(cpu);
    let result = value.wrapping_add(1);
    loc.write(cpu, result);

    Timing::Normal
}

pub fn dec<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value: u8 = loc.read(cpu);
    let result = value.wrapping_sub(1);
    loc.write(cpu, result);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.subtract = true;
    cpu.registers.f.half_carry = (result & 0x0f) == 0x0f;
    Timing::Normal
}

// no affected flags for dec16
pub fn dec16<L: Dst<u16> + Src<u16> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value: u16 = loc.read(cpu);
    let result = value.wrapping_sub(1);
    loc.write(cpu, result);

    Timing::Normal
}

pub fn rlca(cpu: &mut Cpu) -> Timing {
    let timing = rlc(cpu, Reg8::A);
    cpu.registers.f.zero = false;

    timing
}

pub fn rla(cpu: &mut Cpu) -> Timing {
    let timing = rl(cpu, Reg8::A);
    cpu.registers.f.zero = false;

    timing
}

pub fn rrca(cpu: &mut Cpu) -> Timing {
    let timing = rrc(cpu, Reg8::A);
    cpu.registers.f.zero = false;

    timing
}

pub fn rra(cpu: &mut Cpu) -> Timing {
    let timing = rr(cpu, Reg8::A);
    cpu.registers.f.zero = false;

    timing
}

pub fn rlc<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value = loc.read(cpu);
    cpu.registers.f.carry = is_bit_set!(value, 7);

    let value = value.rotate_left(1);

    cpu.registers.f.zero = value == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    loc.write(cpu, value);

    Timing::Normal
}

pub fn rl<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value = loc.read(cpu);
    let c = is_bit_set!(value, 7);
    let value = value.shl(1);
    let value = value | (cpu.registers.f.carry as u8);

    loc.write(cpu, value);

    cpu.registers.f.carry = c;
    cpu.registers.f.zero = value == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn rr<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value = loc.read(cpu);
    let c = is_bit_set!(value, 0);
    let value = value.shr(1);
    let value = value | ((cpu.registers.f.carry as u8) << 7);

    loc.write(cpu, value);

    cpu.registers.f.carry = c;
    cpu.registers.f.zero = value == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn rrc<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value = loc.read(cpu);
    cpu.registers.f.carry = is_bit_set!(value, 0);
    let value = value.rotate_right(1);

    cpu.registers.f.zero = value == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    loc.write(cpu, value);

    Timing::Normal
}

pub fn add<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let value_src = src.read(cpu);
    let value_dest = dest.read(cpu);
    let (result, carry) = value_dest.overflowing_add(value_src);

    dest.write(cpu, result);

    cpu.registers.f.carry = carry;
    cpu.registers.f.zero = result == 0;
    cpu.registers.f.half_carry = is_bit_set!((value_src ^ value_dest ^ result), 4);

    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn add16<D: Dst<u16> + Src<u16> + Copy, S: Src<u16>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let a = src.read(cpu);
    let b = dest.read(cpu);
    let (result, carry) = b.overflowing_add(a);

    dest.write(cpu, result);

    cpu.registers.f.carry = carry;
    cpu.registers.f.half_carry = is_bit_set!((a ^ b ^ result), 12);
;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn offset_sp(cpu: &mut Cpu) -> u16 {
    let a = (Imem8.read(cpu) as i8) as i16;
    let b = cpu.registers.sp as i16;
    let result = b.wrapping_add(a);

    cpu.registers.f.zero = false;
    cpu.registers.f.subtract = false;
    cpu.registers.f.half_carry = is_bit_set!((a ^ b ^ result), 4);
    cpu.registers.f.carry = is_bit_set!((a ^ b ^ result), 8);

    result as u16
}

pub fn add_sp(cpu: &mut Cpu) -> Timing {
    let result = offset_sp(cpu);
    cpu.registers.sp = result;

    Timing::Normal
}

pub fn jr<S: Src<u8>>(cpu: &mut Cpu, cond: Condition, src: S) -> Timing {
    let offset = src.read(cpu) as i8;
    if cond.eval(cpu) {
        let offset = offset as i16;
        cpu.registers.pc = ((cpu.registers.pc as i16).wrapping_add(offset)) as u16;
        Timing::Conditionnal
    } else {
        Timing::Normal
    }
}

pub fn daa(cpu: &mut Cpu) -> Timing {
    let mut a = cpu.registers.a;
    let c = cpu.registers.f.carry;
    let hc = cpu.registers.f.half_carry;
    let n = cpu.registers.f.subtract;

    if !n {
        if c || a > 0x99 {
            a = a.wrapping_add(0x60);
            cpu.registers.f.carry = true;
        }
        if hc || (a & 0x0F) > 0x09 {
            a = a.wrapping_add(0x6);
        }
    } else {
        if c {
            a = a.wrapping_sub(0x60);
        }
        if hc {
            a = a.wrapping_sub(0x06);
        }
    }
    cpu.registers.a = a;
    cpu.registers.f.zero = a == 0;
    cpu.registers.f.half_carry = false;

    Timing::Normal
}

pub fn cpl(cpu: &mut Cpu) -> Timing {
    cpu.registers.a = !cpu.registers.a;
    cpu.registers.f.subtract = true;
    cpu.registers.f.half_carry = true;

    Timing::Normal
}

pub fn scf(cpu: &mut Cpu) -> Timing {
    cpu.registers.f.subtract = false;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.carry = true;

    Timing::Normal
}

pub fn ccf(cpu: &mut Cpu) -> Timing {
    cpu.registers.f.subtract = false;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.carry = !cpu.registers.f.carry;

    Timing::Normal
}

pub fn halt(cpu: &mut Cpu) -> Timing {
    cpu.halted = true;
    // println!("Should implement CPU halting");
    Timing::Normal
}

pub fn adc<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let a = src.read(cpu);
    let b = dest.read(cpu);
    let c = cpu.registers.f.carry as u8;

    let (result, carry) = b.overflowing_add(a);
    let (result, carry_c) = result.overflowing_add(c);

    dest.write(cpu, result);

    cpu.registers.f.carry = carry || carry_c;
    cpu.registers.f.zero = result == 0;
    cpu.registers.f.half_carry = is_bit_set!((a ^ b ^ c ^ result), 4);
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn sub<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let b = src.read(cpu);
    let a = dest.read(cpu);
    let (result, carry) = a.overflowing_sub(b);

    dest.write(cpu, result);

    cpu.registers.f.carry = carry;
    cpu.registers.f.zero = result == 0;
    cpu.registers.f.half_carry = is_bit_set!((a ^ b ^ result), 4);
    cpu.registers.f.subtract = true;

    Timing::Normal
}

pub fn sbc<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let a = dest.read(cpu);
    let b = src.read(cpu);
    let c = cpu.registers.f.carry as u8;
    let (result, carry) = a.overflowing_sub(b);
    let (result, carry_c) = result.overflowing_sub(c);
    dest.write(cpu, result as u8);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.subtract = true;
    cpu.registers.f.carry = carry || carry_c;
    cpu.registers.f.half_carry = is_bit_set!((a ^ b ^ c ^ result), 4);

    Timing::Normal
}

pub fn and<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let a = src.read(cpu);
    let b = dest.read(cpu);

    let result = a & b;

    dest.write(cpu, result);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.carry = false;
    cpu.registers.f.half_carry = true;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn xor<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let a = src.read(cpu);
    let b = dest.read(cpu);

    let result = a ^ b;

    dest.write(cpu, result);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.carry = false;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn or<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let a = src.read(cpu);
    let b = dest.read(cpu);

    let result = a | b;

    dest.write(cpu, result);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.carry = false;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn cp<S: Src<u8>>(cpu: &mut Cpu, src: S) -> Timing {
    let b = src.read(cpu);
    let a = cpu.registers.a;

    // println!("Value B {:02X}", b);
    // println!("Before {}", cpu.registers);

    let (result, carry) = a.overflowing_sub(b);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.subtract = true;
    cpu.registers.f.half_carry = (result & 0xf) > (a & 0xf);
    cpu.registers.f.carry = carry;
    // panic!("After {}", cpu.registers);

    Timing::Normal
}

pub fn ret(cpu: &mut Cpu, cond: Condition) -> Timing {
    if cond.eval(cpu) {
        let pc = cpu.pop16();
        #[cfg(feature = "debug")]
        println!("Returning to address 0x{:04X}", pc);
        cpu.registers.pc = pc;
        return Timing::Conditionnal;
    }
    Timing::Normal
}

pub fn pop<D: Dst<u16>>(cpu: &mut Cpu, dest: D) -> Timing {
    let value = cpu.pop16();
    // println!("Writing {:04X}", value);
    dest.write(cpu, value);

    Timing::Normal
}

pub fn push<S: Src<u16>>(cpu: &mut Cpu, src: S) -> Timing {
    let value = src.read(cpu);
    // println!("Pushing address 0x{:04X} to stack", value);
    cpu.push16(value);
    Timing::Normal
}

pub fn jp<T: Src<u16>>(cpu: &mut Cpu, cond: Condition, target: T) -> Timing {
    let addr = target.read(cpu);
    if cond.eval(cpu) {
        cpu.registers.pc = addr;
        return Timing::Conditionnal;
    }
    Timing::Normal
}

pub fn call<T: Src<u16>>(cpu: &mut Cpu, cond: Condition, target: T) -> Timing {
    let addr = target.read(cpu);
    if cond.eval(cpu) {
        push(cpu, Reg16::PC);
        #[cfg(feature = "debug")]
        {
            println!("\nCalling function at 0x{:04X}", addr);
        }
        cpu.registers.pc = addr;
        return Timing::Conditionnal;
    }
    Timing::Normal
}

pub fn rst(cpu: &mut Cpu, src: u8) -> Timing {
    let pc = cpu.registers.pc;
    cpu.push16(pc);
    cpu.registers.pc = src as u16;

    Timing::Normal
}

pub fn reti(cpu: &mut Cpu) -> Timing {
    ei(cpu);
    ret(cpu, Condition::Unconditional)
}

pub fn ei(cpu: &mut Cpu) -> Timing {
    cpu.set_interrupts(true);
    Timing::Normal
}

pub fn di(cpu: &mut Cpu) -> Timing {
    cpu.set_interrupts(false);
    Timing::Normal
}

pub fn ldhl(cpu: &mut Cpu) -> Timing {
    let sp = offset_sp(cpu);
    Reg16::HL.write(cpu, sp);
    Timing::Normal
}

pub fn srl<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value = loc.read(cpu);
    let result = value >> 1;
    loc.write(cpu, result);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.carry = is_bit_set!(value, 0);
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
    Timing::Normal
}

pub fn swap<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value = loc.read(cpu);
    let result = (value << 4) | (value >> 4);
    loc.write(cpu, result);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.carry = false;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn sra<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value = loc.read(cpu);
    let result = (value & 0x80) | (value >> 1);
    loc.write(cpu, result);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.carry = is_bit_set!(value, 0);
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
    Timing::Normal
}

pub fn sla<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value = loc.read(cpu);
    // eprintln!("Before {}, value {:02X}", cpu.registers, value);
    let result = value << 1;
    loc.write(cpu, result);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.subtract = false;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.carry = is_bit_set!(value, 7);
    Timing::Normal
}

pub fn bit<L: Src<u8>>(cpu: &mut Cpu, bit: u8, loc: L) -> Timing {
    let value = loc.read(cpu);
    cpu.registers.f.zero = !is_bit_set!(value, bit);
    cpu.registers.f.subtract = false;
    cpu.registers.f.half_carry = true;

    Timing::Normal
}

pub fn res<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, bit: u8, loc: L) -> Timing {
    let value = loc.read(cpu);
    let result = value & !(1 << bit);
    loc.write(cpu, result);

    Timing::Normal
}

pub fn set<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, bit: u8, loc: L) -> Timing {
    let value = loc.read(cpu);
    let result = value | (1 << bit);
    loc.write(cpu, result);

    Timing::Normal
}
