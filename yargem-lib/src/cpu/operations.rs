use crate::cpu::registers::Flags;
use crate::is_bit_set;

use std::ops::{Shl, Shr};

pub fn inc(f: &mut Flags, value: u8) -> u8 {
    let result = value.wrapping_add(1);

    f.zero = result == 0;
    f.subtract = false;
    f.half_carry = (value & 0x0f) == 0x0f;

    result
}

pub fn dec(f: &mut Flags, value: u8) -> u8 {
    let result = value.wrapping_sub(1);

    f.zero = result == 0;
    f.subtract = true;
    f.half_carry = (result & 0x0f) == 0x0f;

    result
}

pub fn rlca(f: &mut Flags, value: u8) -> u8 {
    let result = rlc(f, value);
    f.zero = false;

    result
}

pub fn rla(f: &mut Flags, value: u8) -> u8 {
    let result = rl(f, value);
    f.zero = false;

    result
}

pub fn rrca(f: &mut Flags, value: u8) -> u8 {
    let result = rrc(f, value);
    f.zero = false;

    result
}

pub fn rra(f: &mut Flags, value: u8) -> u8 {
    let timing = rr(f, value);
    f.zero = false;

    timing
}

pub fn rlc(f: &mut Flags, value: u8) -> u8 {
    f.carry = is_bit_set!(value, 7);

    let value = value.rotate_left(1);

    f.zero = value == 0;
    f.half_carry = false;
    f.subtract = false;

    value
}

pub fn rl(f: &mut Flags, value: u8) -> u8 {
    let c = is_bit_set!(value, 7);
    let value = value.shl(1);
    let value = value | (f.carry as u8);

    f.carry = c;
    f.zero = value == 0;
    f.half_carry = false;
    f.subtract = false;

    value
}

pub fn rr(f: &mut Flags, value: u8) -> u8 {
    let c = is_bit_set!(value, 0);
    let value = value.shr(1);
    let value = value | ((f.carry as u8) << 7);

    f.carry = c;
    f.zero = value == 0;
    f.half_carry = false;
    f.subtract = false;

    value
}

pub fn rrc(f: &mut Flags, value: u8) -> u8 {
    f.carry = is_bit_set!(value, 0);
    let value = value.rotate_right(1);

    f.zero = value == 0;
    f.half_carry = false;
    f.subtract = false;

    value
}

pub fn add(f: &mut Flags, left: u8, right: u8) -> u8 {
    let (result, carry) = left.overflowing_add(right);

    f.carry = carry;
    f.zero = result == 0;
    f.half_carry = is_bit_set!((right ^ left ^ result), 4);

    f.subtract = false;

    result
}

pub fn add16(f: &mut Flags, left: u16, right: u16) -> u16 {
    let (result, carry) = left.overflowing_add(right);

    f.carry = carry;
    f.half_carry = is_bit_set!((right ^ left ^ result), 12);
    f.subtract = false;

    result
}

pub fn offset_sp(f: &mut Flags, sp: u16, r8: u8) -> u16 {
    let result = sp.wrapping_add_signed(r8 as i8 as i16);
    let lo = sp as u8;

    f.zero = false;
    f.subtract = false;
    f.half_carry = (lo & 0x0F) + (r8 & 0x0F) > 0x0F;
    f.carry = (lo as u16) + (r8 as u16) > 0xFF;

    result
}

pub fn da(f: &mut Flags, value: u8) -> u8 {
    let mut result = value;
    let c = f.carry;
    let hc = f.half_carry;
    let n = f.subtract;

    if !n {
        if c || result > 0x99 {
            result = result.wrapping_add(0x60);
            f.carry = true;
        }
        if hc || (result & 0x0F) > 0x09 {
            result = result.wrapping_add(0x6);
        }
    } else {
        if c {
            result = result.wrapping_sub(0x60);
        }
        if hc {
            result = result.wrapping_sub(0x06);
        }
    }
    f.zero = result == 0;
    f.half_carry = false;

    result
}

pub fn cpl(f: &mut Flags, value: u8) -> u8 {
    f.subtract = true;
    f.half_carry = true;

    !value
}

pub fn scf(f: &mut Flags) {
    f.subtract = false;
    f.half_carry = false;
    f.carry = true;
}

pub fn ccf(f: &mut Flags) {
    f.subtract = false;
    f.half_carry = false;
    f.carry = !f.carry;
}

pub fn adc(f: &mut Flags, a: u8, b: u8) -> u8 {
    let c = f.carry as u8;

    let (result, carry) = b.overflowing_add(a);
    let (result, carry_c) = result.overflowing_add(c);

    f.carry = carry || carry_c;
    f.zero = result == 0;
    f.half_carry = is_bit_set!((a ^ b ^ c ^ result), 4);
    f.subtract = false;

    result
}

pub fn sub(f: &mut Flags, a: u8, b: u8) -> u8 {
    let (result, carry) = a.overflowing_sub(b);

    f.carry = carry;
    f.zero = result == 0;
    f.half_carry = is_bit_set!((a ^ b ^ result), 4);
    f.subtract = true;

    result
}

pub fn sbc(f: &mut Flags, a: u8, b: u8) -> u8 {
    let c = f.carry as u8;
    let (result, carry) = a.overflowing_sub(b);
    let (result, carry_c) = result.overflowing_sub(c);

    f.zero = result == 0;
    f.subtract = true;
    f.carry = carry || carry_c;
    f.half_carry = is_bit_set!((a ^ b ^ c ^ result), 4);

    result
}

pub fn and(f: &mut Flags, a: u8, b: u8) -> u8 {
    let result = a & b;

    f.zero = result == 0;
    f.carry = false;
    f.half_carry = true;
    f.subtract = false;

    result
}

pub fn xor(f: &mut Flags, a: u8, b: u8) -> u8 {
    let result = a ^ b;

    f.zero = result == 0;
    f.carry = false;
    f.half_carry = false;
    f.subtract = false;

    result
}

pub fn or(f: &mut Flags, a: u8, b: u8) -> u8 {
    let result = a | b;

    f.zero = result == 0;
    f.carry = false;
    f.half_carry = false;
    f.subtract = false;

    result
}

/// Argument order matches `sub`: `a` is the accumulator, `b` the operand.
pub fn cp(f: &mut Flags, a: u8, b: u8) {
    let (result, carry) = a.overflowing_sub(b);

    f.zero = result == 0;
    f.subtract = true;
    f.half_carry = (result & 0xf) > (a & 0xf);
    f.carry = carry;
}

pub fn srl(f: &mut Flags, value: u8) -> u8 {
    let result = value >> 1;

    f.zero = result == 0;
    f.carry = is_bit_set!(value, 0);
    f.half_carry = false;
    f.subtract = false;

    result
}

pub fn swap(f: &mut Flags, value: u8) -> u8 {
    let result = value.rotate_right(4);

    f.zero = result == 0;
    f.carry = false;
    f.half_carry = false;
    f.subtract = false;

    result
}

pub fn sra(f: &mut Flags, value: u8) -> u8 {
    let result = (value & 0x80) | (value >> 1);

    f.zero = result == 0;
    f.carry = is_bit_set!(value, 0);
    f.half_carry = false;
    f.subtract = false;

    result
}

pub fn sla(f: &mut Flags, value: u8) -> u8 {
    let result = value << 1;

    f.zero = result == 0;
    f.subtract = false;
    f.half_carry = false;
    f.carry = is_bit_set!(value, 7);

    result
}

pub fn bit(f: &mut Flags, bit: u8, value: u8) {
    f.zero = !is_bit_set!(value, bit);
    f.subtract = false;
    f.half_carry = true;
}

pub fn res(bit: u8, value: u8) -> u8 {
    value & !(1 << bit)
}

pub fn set(bit: u8, value: u8) -> u8 {
    value | (1 << bit)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Flag bits as `Flags`' own u8 conversion orders them, so a whole flag
    // register is one literal and an assertion names every bit at once --
    // including the ones an operation must leave alone.
    const Z: u8 = 0b1000_0000;
    const N: u8 = 0b0100_0000;
    const H: u8 = 0b0010_0000;
    const C: u8 = 0b0001_0000;

    fn flags(bits: u8) -> Flags {
        Flags::from(bits)
    }

    /// `op(&mut flags(before), ..)` -> (result, flags after).
    macro_rules! run {
        ($op:ident($fin:expr $(, $arg:expr)*)) => {{
            let mut f = flags($fin);
            let r = $op(&mut f, $($arg),*);
            (r, u8::from(&f))
        }};
    }

    ///op(...) -> result
    macro_rules! run_noflag {
        ($op:ident($($arg:expr),*)) => {{
            $op($($arg),*)
        }};
    }

    /// For the operations that return nothing and only move flags.
    macro_rules! run_flags {
        ($op:ident($fin:expr $(, $arg:expr)*)) => {{
            let mut f = flags($fin);
            $op(&mut f, $($arg),*);
            u8::from(&f)
        }};
    }

    // ---------------------------------------------------------------- inc/dec

    #[test]
    fn inc_sets_half_carry_at_the_nibble_boundary() {
        assert_eq!(run!(inc(0, 0x00)), (0x01, 0));
        assert_eq!(run!(inc(0, 0x0F)), (0x10, H));
        assert_eq!(run!(inc(0, 0xFF)), (0x00, Z | H));
    }

    #[test]
    fn inc_leaves_carry_alone() {
        // The only 8-bit ALU operation that does not touch C. A `dec`/`inc`
        // between a compare and its branch must not disturb the compare.
        assert_eq!(run!(inc(C, 0x00)), (0x01, C));
        assert_eq!(run!(inc(C, 0xFF)), (0x00, Z | H | C));
    }

    #[test]
    fn dec_counts_down() {
        assert_eq!(run!(dec(0, 0x02)), (0x01, N));
        assert_eq!(run!(dec(0, 0x01)), (0x00, Z | N));
    }

    #[test]
    fn dec_sets_half_carry_when_it_borrows_across_the_nibble() {
        assert_eq!(run!(dec(0, 0x10)), (0x0F, N | H));
        assert_eq!(run!(dec(0, 0x00)), (0xFF, N | H));
    }

    #[test]
    fn dec_leaves_carry_alone() {
        assert_eq!(run!(dec(C, 0x01)), (0x00, Z | N | C));
    }

    // -------------------------------------------------------------- add/adc

    #[test]
    fn add_flags() {
        assert_eq!(run!(add(0, 0x01, 0x01)), (0x02, 0));
        assert_eq!(run!(add(0, 0x0F, 0x01)), (0x10, H));
        assert_eq!(run!(add(0, 0xFF, 0x01)), (0x00, Z | H | C));
        assert_eq!(run!(add(0, 0xF0, 0x10)), (0x00, Z | C));
        // N is cleared even when it was set going in.
        assert_eq!(run!(add(N, 0x01, 0x01)), (0x02, 0));
    }

    #[test]
    fn add_ignores_the_incoming_carry() {
        assert_eq!(run!(add(C, 0x01, 0x01)), (0x02, 0));
    }

    #[test]
    fn adc_adds_the_carry_in() {
        assert_eq!(run!(adc(0, 0x01, 0x01)), (0x02, 0));
        assert_eq!(run!(adc(C, 0x01, 0x01)), (0x03, 0));
        // The carry alone crosses the nibble.
        assert_eq!(run!(adc(C, 0x0F, 0x00)), (0x10, H));
    }

    #[test]
    fn adc_carries_out_of_the_carry_in() {
        // 0xFF + 0x00 does not overflow, but the +1 does. Both adds have to be
        // counted or the carry is lost.
        assert_eq!(run!(adc(C, 0xFF, 0x00)), (0x00, Z | H | C));
        assert_eq!(run!(adc(C, 0xFF, 0xFF)), (0xFF, H | C));
    }

    // -------------------------------------------------------------- sub/sbc

    #[test]
    fn sub_flags() {
        assert_eq!(run!(sub(0, 0x02, 0x01)), (0x01, N));
        assert_eq!(run!(sub(0, 0x01, 0x01)), (0x00, Z | N));
        assert_eq!(run!(sub(0, 0x10, 0x01)), (0x0F, N | H));
        assert_eq!(run!(sub(0, 0x00, 0x01)), (0xFF, N | H | C));
    }

    #[test]
    fn sbc_subtracts_the_carry_in() {
        assert_eq!(run!(sbc(0, 0x02, 0x01)), (0x01, N));
        assert_eq!(run!(sbc(C, 0x02, 0x01)), (0x00, Z | N));
        assert_eq!(run!(sbc(C, 0x00, 0x00)), (0xFF, N | H | C));
        assert_eq!(run!(sbc(C, 0x10, 0x00)), (0x0F, N | H));
    }

    // ------------------------------------------------------------- and/or/xor

    #[test]
    fn and_always_sets_half_carry() {
        // The one logical operation that does, and the reason `and a` is the
        // idiomatic way to clear C while testing for zero.
        assert_eq!(run!(and(0, 0x0F, 0xF0)), (0x00, Z | H));
        assert_eq!(run!(and(C | N, 0xFF, 0x0F)), (0x0F, H));
    }

    #[test]
    fn or_and_xor_clear_every_flag_but_zero() {
        assert_eq!(run!(or(Z | N | H | C, 0x0F, 0xF0)), (0xFF, 0));
        assert_eq!(run!(or(C, 0x00, 0x00)), (0x00, Z));
        assert_eq!(run!(xor(Z | N | H | C, 0x0F, 0xFF)), (0xF0, 0));
        assert_eq!(run!(xor(C, 0xAA, 0xAA)), (0x00, Z));
    }

    // -------------------------------------------------------------------- cp

    #[test]
    fn cp_flags_match_sub_without_producing_a_value() {
        // `cp` and `sub` take the same argument order -- accumulator first.
        // They are the pair most likely to drift apart, so assert rather than
        // duplicate the expected flags.
        for (acc, operand) in [(0x02u8, 0x01u8), (0x01, 0x01), (0x10, 0x01), (0x00, 0x01)] {
            let mut cp_flags = flags(0);
            cp(&mut cp_flags, acc, operand);

            let mut sub_flags = flags(0);
            sub(&mut sub_flags, acc, operand);

            assert_eq!(
                u8::from(&cp_flags),
                u8::from(&sub_flags),
                "cp {operand:#04X} against {acc:#04X}"
            );
        }
    }

    // --------------------------------------------------------------- rotates

    #[test]
    fn rlc_rotates_bit_7_into_bit_0_and_carry() {
        assert_eq!(run!(rlc(0, 0x80)), (0x01, C));
        assert_eq!(run!(rlc(0, 0x01)), (0x02, 0));
        assert_eq!(run!(rlc(C, 0x00)), (0x00, Z));
    }

    #[test]
    fn rl_rotates_through_carry() {
        assert_eq!(run!(rl(0, 0x80)), (0x00, Z | C));
        assert_eq!(run!(rl(C, 0x80)), (0x01, C));
        assert_eq!(run!(rl(C, 0x00)), (0x01, 0));
    }

    #[test]
    fn rrc_rotates_bit_0_into_bit_7_and_carry() {
        assert_eq!(run!(rrc(0, 0x01)), (0x80, C));
        assert_eq!(run!(rrc(0, 0x02)), (0x01, 0));
        assert_eq!(run!(rrc(C, 0x00)), (0x00, Z));
    }

    #[test]
    fn rr_rotates_through_carry() {
        assert_eq!(run!(rr(0, 0x01)), (0x00, Z | C));
        assert_eq!(run!(rr(C, 0x01)), (0x80, C));
        assert_eq!(run!(rr(C, 0x00)), (0x80, 0));
    }

    #[test]
    fn the_accumulator_rotates_never_set_zero() {
        // The whole difference between 0x07/0x17/0x0F/0x1F and their 0xCB
        // twins, and a classic source of a wrong branch after `rra`.
        assert_eq!(run!(rlca(0, 0x00)), (0x00, 0));
        assert_eq!(run!(rla(0, 0x80)), (0x00, C));
        assert_eq!(run!(rrca(0, 0x00)), (0x00, 0));
        assert_eq!(run!(rra(0, 0x01)), (0x00, C));

        // ... where the CB forms do.
        assert_eq!(run!(rlc(0, 0x00)).1, Z);
        assert_eq!(run!(rl(0, 0x80)).1, Z | C);
        assert_eq!(run!(rrc(0, 0x00)).1, Z);
        assert_eq!(run!(rr(0, 0x01)).1, Z | C);
    }

    // ---------------------------------------------------------------- shifts

    #[test]
    fn sla_shifts_bit_7_out() {
        assert_eq!(run!(sla(0, 0x80)), (0x00, Z | C));
        assert_eq!(run!(sla(0, 0x01)), (0x02, 0));
    }

    #[test]
    fn sra_preserves_the_sign_bit() {
        assert_eq!(run!(sra(0, 0x81)), (0xC0, C));
        assert_eq!(run!(sra(0, 0x80)), (0xC0, 0));
        assert_eq!(run!(sra(0, 0x01)), (0x00, Z | C));
    }

    #[test]
    fn srl_shifts_a_zero_in() {
        assert_eq!(run!(srl(0, 0x81)), (0x40, C));
        assert_eq!(run!(srl(0, 0x80)), (0x40, 0));
        assert_eq!(run!(srl(0, 0x01)), (0x00, Z | C));
    }

    #[test]
    fn swap_exchanges_the_nibbles() {
        assert_eq!(run!(swap(0, 0xAB)), (0xBA, 0));
        assert_eq!(run!(swap(C, 0x00)), (0x00, Z));
    }

    // ------------------------------------------------------------ bit/res/set

    #[test]
    fn bit_sets_zero_from_the_complement_and_leaves_carry() {
        assert_eq!(run_flags!(bit(0, 7, 0x80)), H);
        assert_eq!(run_flags!(bit(0, 7, 0x7F)), Z | H);
        assert_eq!(run_flags!(bit(0, 0, 0x01)), H);
        // C is preserved, N cleared, H forced.
        assert_eq!(run_flags!(bit(N | C, 3, 0x08)), H | C);
    }

    #[test]
    fn res_and_set() {
        assert_eq!(run_noflag!(res(3, 0xFF)), 0xF7);
        assert_eq!(run_noflag!(res(3, 0xF7)), 0xF7);
        assert_eq!(run_noflag!(set(3, 0x00)), 0x08);
        assert_eq!(run_noflag!(set(3, 0x08)), 0x08);
    }

    // ------------------------------------------------------------------- daa

    #[test]
    fn daa_after_addition() {
        // 9 + 8 = 17 BCD. The add leaves 0x11 with H set.
        let (sum, after_add) = run!(add(0, 0x09, 0x08));
        assert_eq!((sum, after_add), (0x11, H));
        assert_eq!(run!(da(after_add, sum)), (0x17, 0));

        // 99 + 1 = 100 BCD: wraps to 0x00 and carries.
        let (sum, after_add) = run!(add(0, 0x99, 0x01));
        assert_eq!(run!(da(after_add, sum)), (0x00, Z | C));
    }

    #[test]
    fn daa_after_subtraction() {
        // 1 - 2 = 99 BCD with a borrow.
        let (diff, after_sub) = run!(sub(0, 0x01, 0x02));
        assert_eq!((diff, after_sub), (0xFF, N | H | C));
        assert_eq!(run!(da(after_sub, diff)), (0x99, N | C));
    }

    #[test]
    fn daa_always_clears_half_carry() {
        assert_eq!(run!(da(H, 0x00)).1 & H, 0);
        assert_eq!(run!(da(N | H, 0x00)).1 & H, 0);
    }

    // ----------------------------------------------------------------- add16

    #[test]
    fn add16_carries_out_of_bit_11_and_bit_15() {
        assert_eq!(run!(add16(0, 0x0FFF, 0x0001)), (0x1000, H));
        assert_eq!(run!(add16(0, 0xFFFF, 0x0001)), (0x0000, H | C));
        assert_eq!(run!(add16(0, 0x8000, 0x8000)), (0x0000, C));
    }

    #[test]
    fn add16_leaves_zero_alone() {
        // `add hl,rr` does not touch Z, even when the result is zero.
        assert_eq!(run!(add16(Z, 0x0001, 0x0001)), (0x0002, Z));
        assert_eq!(run!(add16(0, 0xFFFF, 0x0001)).1 & Z, 0);
    }

    // ------------------------------------------------------------- offset_sp

    #[test]
    fn offset_sp_adds_a_positive_displacement() {
        assert_eq!(run!(offset_sp(0, 0x0000, 0x01)), (0x0001, 0));
        assert_eq!(run!(offset_sp(0, 0x000F, 0x01)), (0x0010, H));
        assert_eq!(run!(offset_sp(0, 0x00FF, 0x01)), (0x0100, H | C));
    }

    #[test]
    fn offset_sp_treats_the_displacement_as_signed() {
        // 0xFF is -1, not +255. Sign extension is the whole point of `r8`, and
        // every stack frame in every game depends on it.
        assert_eq!(run!(offset_sp(0, 0xFF00, 0xFF)).0, 0xFEFF);
        assert_eq!(run!(offset_sp(0, 0x0000, 0x80)).0, 0xFF80);
        assert_eq!(run!(offset_sp(0, 0x0100, 0xFF)).0, 0x00FF);
    }

    #[test]
    fn offset_sp_computes_flags_from_the_unsigned_low_byte() {
        // H and C come from the byte addition of SP's low half with r8 taken
        // unsigned -- not from the signed 16-bit result. A negative
        // displacement can therefore still set C.
        assert_eq!(run!(offset_sp(Z | N, 0xFF00, 0xFF)).1, 0);
        assert_eq!(run!(offset_sp(0, 0xFF0F, 0xFF)).1, H | C);
        assert_eq!(run!(offset_sp(0, 0x00FF, 0xFF)).1, H | C);
    }

    #[test]
    fn offset_sp_always_clears_zero() {
        // Even when the result is 0x0000 -- the one 16-bit result that is not
        // tested for zero.
        assert_eq!(run!(offset_sp(Z, 0x0001, 0xFF)).0, 0x0000);
        assert_eq!(run!(offset_sp(Z, 0x0001, 0xFF)).1 & Z, 0);
    }

    // ------------------------------------------------------- cpl / scf / ccf

    #[test]
    fn cpl_complements_and_leaves_zero_and_carry() {
        assert_eq!(run!(cpl(0, 0x00)), (0xFF, N | H));
        assert_eq!(run!(cpl(Z | C, 0xAA)), (0x55, Z | N | H | C));
    }

    #[test]
    fn scf_and_ccf_leave_zero_alone() {
        assert_eq!(run_flags!(scf(N | H)), C);
        assert_eq!(run_flags!(scf(Z)), Z | C);
        assert_eq!(run_flags!(ccf(C)), 0);
        assert_eq!(run_flags!(ccf(Z | N | H)), Z | C);
    }
}
