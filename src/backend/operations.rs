use super::cpu::{Cpu, Dst, Imem8, Src};
use super::instructions::Timing;
use super::registers::{Reg8, Reg16};
use std::ops::{Shl, Shr};
use super::instructions::Condition;

#[inline(always)]
pub fn nop() -> Timing {
    Timing::Normal
}

pub fn ld<T, D: Dst<T>, S: Src<T>>(cpu: &mut Cpu, dst: D, src: S) -> Timing {
    let val: T = src.read(cpu);
    dst.write(cpu, val);
    Timing::Normal
}

pub fn ldi<T, D: Dst<T>, S: Src<T>>(cpu: &mut Cpu, dst: D, src: S, to_inc: Reg16) -> Timing {
    let t: Timing = ld(cpu, dst, src);
    inc16(cpu, to_inc);
    t
}

pub fn ldd<T, D: Dst<T>, S: Src<T>>(cpu: &mut Cpu, dst: D, src: S, to_dec: Reg16) -> Timing {
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
    cpu.registers.f.carry = value & 0x80 != 0;
    let value = value.rotate_left(1);

    cpu.registers.f.zero = value == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    loc.write(cpu, value);

    Timing::Normal
}

pub fn rl<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value = loc.read(cpu);
    let c = value & 0x80 != 0;
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
    let c = value & 0x01 != 0;
    let value = value.shr(1);
    let value = value | ((cpu.registers.f.carry as u8) << 8);

    loc.write(cpu, value);

    cpu.registers.f.carry = c;
    cpu.registers.f.zero = value == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn rrc<L: Dst<u8> + Src<u8> + Copy>(cpu: &mut Cpu, loc: L) -> Timing {
    let value = loc.read(cpu);
    cpu.registers.f.carry = value & 0x01 != 0;
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
    cpu.registers.f.half_carry = (value_src & 0x0F) + (value_dest & 0x0F) > 0xF;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn add16<D: Dst<u16> + Src<u16> + Copy, S: Src<u16>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let value_src = src.read(cpu);
    let value_dest = dest.read(cpu);
    let (result, carry) = value_dest.overflowing_add(value_src);

    dest.write(cpu, result);

    cpu.registers.f.carry = carry;
    cpu.registers.f.zero = result == 0;
    cpu.registers.f.half_carry = (value_src & 0xFF) + (value_dest & 0xFF) > 0xFF;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn offset_sp<S: Src<u8>>(cpu: &mut Cpu, src: S) -> u16 {
    let value_src = (src.read(cpu) as i16) as i32;
    let value_dest = (cpu.registers.sp as i16) as i32;
    let result = value_dest.wrapping_add(value_src);

    cpu.registers.f.carry = ((value_dest ^ value_src ^ (result & 0xffff)) & 0x100) == 0x100;
    cpu.registers.f.half_carry = ((value_dest ^ value_src ^ (result & 0xffff)) & 0x10) == 0x10;
    cpu.registers.f.zero = result == 0;
    cpu.registers.f.subtract = false;

    result as u16
}

pub fn add_sp<S: Src<u8>>(cpu: &mut Cpu, src: S) -> Timing {
    let result = offset_sp(cpu, src);
    cpu.registers.sp = result as u16;

    Timing::Normal
}

pub fn jr<S: Src<u8>>(cpu: &mut Cpu, cond: Condition, src: S) -> Timing {
    if cond.eval(cpu) {
        let offset = src.read(cpu) as i8;
        let offset = offset as i16;
        cpu.registers.pc = ((cpu.registers.pc as i16).wrapping_add(offset)) as u16;
        Timing::Conditionnal
    } else {
        cpu.registers.pc += 1;
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
    println!("Should implement CPU halting");
    Timing::Normal
}

pub fn adc<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let value_src = src.read(cpu);
    let value_dest = dest.read(cpu);
    let (_result,_carry) = value_dest.overflowing_add(value_src);
    let (result, carry) = value_dest.overflowing_add(cpu.registers.f.carry as u8);

    dest.write(cpu, result);

    cpu.registers.f.carry = carry;
    cpu.registers.f.zero = result == 0;
    cpu.registers.f.half_carry = (value_src & 0x0F) + (value_dest & 0x0F) > 0xF;
    cpu.registers.f.subtract = false;

    Timing::Normal
}

pub fn sub<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let value_src = src.read(cpu);
    let value_dest = dest.read(cpu);
    let (result, carry) = value_dest.overflowing_sub(value_src);

    dest.write(cpu, result);

    cpu.registers.f.carry = carry;
    cpu.registers.f.zero = result == 0;
    cpu.registers.f.half_carry = (value_src & 0x0F) + (value_dest & 0x0F) > 0xF;
    cpu.registers.f.subtract = true;

    Timing::Normal
}

pub fn sbc<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let value_src = src.read(cpu);
    let value_dest = dest.read(cpu);
    let (_result, _carry) = value_dest.overflowing_sub(value_src);
    let (result, carry) = value_dest.overflowing_sub(cpu.registers.f.carry as u8);

    dest.write(cpu, result);

    cpu.registers.f.carry = carry;
    cpu.registers.f.zero = result == 0;
    cpu.registers.f.half_carry = (value_src & 0x0F) + (value_dest & 0x0F) > 0xF;
    cpu.registers.f.subtract = false;

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

pub fn cp<D: Dst<u8> + Src<u8>, S: Src<u8>>(cpu: &mut Cpu, dest: D, src: S) -> Timing {
    let a = src.read(cpu);
    let b = dest.read(cpu);

    let (result, carry) = b.overflowing_sub(a);

    cpu.registers.f.zero = result == 0;
    cpu.registers.f.carry = carry;
    cpu.registers.f.half_carry = (a & 0x0F) + (b & 0x0F) > 0xF;
    cpu.registers.f.subtract = true;

    Timing::Normal
}

pub fn ret(cpu: &mut Cpu, cond: Condition) -> Timing {
    if cond.eval(cpu) {
        let pc = cpu.pop16();
        cpu.registers.pc = pc;
        return Timing::Conditionnal;
    }
    Timing::Normal
}

pub fn pop<D: Dst<u16>>(cpu: &mut Cpu, dest: D) -> Timing {
    let value = cpu.pop16();
    dest.write(cpu, value);
    Timing::Normal
}

pub fn push<S: Src<u16>>(cpu: &mut Cpu, src: S) -> Timing {
    let value = src.read(cpu);
    cpu.push16(value);
    Timing::Normal
}

pub fn jp<T: Src<u16>>(cpu: &mut Cpu, cond: Condition, target: T) -> Timing {
    if cond.eval(cpu) {
        let addr = target.read(cpu);
        cpu.registers.pc = addr;
        return Timing::Conditionnal;
    }
    Timing::Normal
}

pub fn call<T: Src<u16>>(cpu: &mut Cpu, cond: Condition, target: T) -> Timing {
    if cond.eval(cpu) {
        let addr = target.read(cpu);
        push(cpu, Reg16::PC);
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
    let sp = offset_sp(cpu, Imem8);
    Reg16::HL.write(cpu, sp);
    Timing::Normal
}