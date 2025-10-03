use super::bitops::adder;
use super::bitops::{Bit, BitExt, ZERO};

// Type aliases (LSB-first arrays)
pub type SoftU8 = [Bit; 8];
pub type SoftU23 = [Bit; 23];
pub type SoftU24 = [Bit; 24];
pub type SoftU32 = [Bit; 32];

// --- encode/decode ---

#[inline(never)]
#[allow(clippy::needless_range_loop)]
pub fn to_softu_generic<const N: usize>(x: u64) -> [Bit; N] {
    let mut out = [ZERO; N];
    for i in 0..N {
        out[i] = Bit::from_bool(((x >> i) & 1) != 0);
    }
    out
}

#[inline(never)]
pub fn to_softu8(x: u8) -> SoftU8 {
    to_softu_generic::<8>(x as u64)
}
#[inline(never)]
pub fn to_softu23(x: u32) -> SoftU23 {
    to_softu_generic::<23>(x as u64)
}
#[inline(never)]
pub fn to_softu24(x: u32) -> SoftU24 {
    to_softu_generic::<24>(x as u64)
}
#[inline(never)]
pub fn to_softu32(x: u32) -> SoftU32 {
    to_softu_generic::<32>(x as u64)
}

#[inline(never)]
#[allow(clippy::needless_range_loop)]
pub fn from_softu_generic<const N: usize>(a: [Bit; N]) -> u64 {
    let mut v = 0u64;
    for i in 0..N {
        if a[i].sign_bit_is_one() {
            v |= 1u64 << i;
        }
    }
    v
}

#[inline(never)]
pub fn from_softu8(a: SoftU8) -> u8 {
    from_softu_generic(a) as u8
}
#[inline(never)]
pub fn from_softu23(a: SoftU23) -> u32 {
    from_softu_generic(a) as u32
}
#[inline(never)]
pub fn from_softu24(a: SoftU24) -> u32 {
    from_softu_generic(a) as u32
}
#[inline(never)]
pub fn from_softu32(a: SoftU32) -> u32 {
    from_softu_generic(a) as u32
}

// --- adders (return (sum, carry)) ---

#[inline(never)]
pub fn softu_add_generic<const N: usize>(a: [Bit; N], b: [Bit; N]) -> ([Bit; N], Bit) {
    let mut out = [ZERO; N];
    let mut carry = ZERO;
    for i in 0..N {
        let (s, c) = adder(a[i], b[i], carry);
        out[i] = s;
        carry = c;
    }
    (out, carry)
}

#[inline(never)]
pub fn softu8_add(a: SoftU8, b: SoftU8) -> (SoftU8, Bit) {
    softu_add_generic(a, b)
}
#[inline(never)]
pub fn softu23_add(a: SoftU23, b: SoftU23) -> (SoftU23, Bit) {
    softu_add_generic(a, b)
}
#[inline(never)]
pub fn softu24_add(a: SoftU24, b: SoftU24) -> (SoftU24, Bit) {
    softu_add_generic(a, b)
}
#[inline(never)]
pub fn softu32_add(a: SoftU32, b: SoftU32) -> (SoftU32, Bit) {
    softu_add_generic(a, b)
}

// --- shifts (LSB-first) ---

#[inline(never)]
#[allow(clippy::manual_memcpy)] // easier to read this way
pub fn shift_right_generic<const N: usize>(x: [Bit; N]) -> [Bit; N] {
    let mut r = [ZERO; N];
    for i in 0..N - 1 {
        r[i] = x[i + 1];
    }
    r
}

#[inline(never)]
pub fn shift_right23(x: SoftU23) -> SoftU23 {
    shift_right_generic(x)
}
#[inline(never)]
pub fn shift_right24(x: SoftU24) -> SoftU24 {
    shift_right_generic(x)
}
#[inline(never)]
pub fn shift_right32(x: SoftU32) -> SoftU32 {
    shift_right_generic(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_softu8_simple() {
        let a = to_softu8(23);
        let b = to_softu8(19);
        let (c, _carry) = softu8_add(a, b); // <— destructure
        let c_u8 = from_softu8(c);
        assert_eq!(c_u8, 42);
    }

    #[test]
    fn add_softu23_simple() {
        let a = to_softu23(0b10101); // 21
        let b = to_softu23(0b01011); // 11
        let (c, carry) = softu23_add(a, b);
        let c_u32 = from_softu23(c) + if carry.signum() > 0.0 { 1 << 23 } else { 0 };
        assert_eq!(c_u32, 32);
    }

    #[test]
    fn add_softu24_simple() {
        // 21 + 11 = 32 (small, easy sanity check; LSB-first representation)
        let a = to_softu24(0b10101); // 21
        let b = to_softu24(0b01011); // 11
        let (c, carry) = softu24_add(a, b);
        let c_u32 = from_softu24(c) + if carry.signum() > 0.0 { 1 << 24 } else { 0 };
        assert_eq!(c_u32, 32);
    }

    #[test]
    fn add_softu24_carry_out() {
        // (2^24 - 1) + 1 = 2^24 -> result wraps to 0 with carry=1
        let a = to_softu24((1u32 << 24) - 1);
        let b = to_softu24(1);
        let (c, carry) = softu24_add(a, b);
        assert_eq!(from_softu24(c), 0);
        assert!(
            carry.signum() > 0.0,
            "expected carry-out=1 (negative zero), got {:?}",
            carry
        );
    }

    #[test]
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn add_softu32_simple() {
        // 23 + 19 = 42
        let a = to_softu32(23);
        let b = to_softu32(19);
        let (c, carry) = softu32_add(a, b);
        let sum = (from_softu32(c) as u64) + if carry.signum() > 0.0 { 1u64 << 32 } else { 0 };
        assert_eq!(sum, 42);
        assert!(!(carry.signum() > 0.0), "no carry expected");
    }

    #[test]
    fn add_softu32_carry_out() {
        // u32::MAX + 1 = 2^32 -> wraps to 0 with carry=1
        let a = to_softu32(u32::MAX);
        let b = to_softu32(1);
        let (c, carry) = softu32_add(a, b);
        assert_eq!(from_softu32(c), 0);
        assert!(carry.signum() > 0.0, "expected carry-out=1 (negative zero)");
    }
}
