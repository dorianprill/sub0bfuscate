use super::bitops::{Bit, BitExt, ONE, ZERO};
use super::softuint::{from_softu8, to_softu8, SoftU23, SoftU8};
use super::softuint::{shift_right24, softu24_add, SoftU24};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SofterF32 {
    pub sign: Bit,
    pub exponent: SoftU8,  // stored exponent (no bias removed)
    pub fraction: SoftU23, // stored 23 bits (no implicit 1 here)
}

// pack/unpack with your existing approach
#[allow(clippy::needless_range_loop)]
#[inline(never)]
pub fn to_softerf32(x: f32) -> SofterF32 {
    let bits = x.to_bits();
    let sign = if (bits >> 31) != 0 { ZERO } else { ONE }; // - => 0, + => 1
    let exp = to_softu8(((bits >> 23) & 0xff) as u8);
    let mut frac = [ZERO; 23];
    for i in 0..23 {
        frac[i] = if ((bits >> i) & 1) != 0 { ONE } else { ZERO };
    }
    SofterF32 {
        sign,
        exponent: exp,
        fraction: frac,
    }
}

#[inline(never)]
pub fn from_softerf32(s: SofterF32) -> f32 {
    let mut bits: u32 = 0;

    if s.sign.sign_bit_is_zero() {
        // -0.0 encodes “negative”
        bits |= 1 << 31;
    }
    bits |= (from_softu8(s.exponent) as u32) << 23;
    for i in 0..23 {
        if s.fraction[i].sign_bit_is_one() {
            // also use the trait here if you like
            bits |= 1 << i;
        }
    }
    f32::from_bits(bits)
}

// simple add (same-sign fast path), using 24-bit significands — minimal from earlier
#[allow(clippy::comparison_chain)]
#[no_mangle]
#[inline(never)]
pub extern "C" fn softerf32_add(a: SofterF32, b: SofterF32) -> SofterF32 {
    let a_exp = from_softu8(a.exponent) as i32;
    let b_exp = from_softu8(b.exponent) as i32;

    // build 24-bit significands with implicit 1 for normals
    fn with_implicit(frac: SoftU23, exp: SoftU8) -> SoftU24 {
        let mut sig = [ZERO; 24];
        sig[..23].copy_from_slice(&frac);
        sig[23] = if from_softu8(exp) != 0 { ONE } else { ZERO };
        sig
    }
    fn drop_implicit(sig: SoftU24) -> SoftU23 {
        let mut f = [ZERO; 23];
        f.copy_from_slice(&sig[..23]);
        f
    }

    let mut a_sig = with_implicit(a.fraction, a.exponent);
    let mut b_sig = with_implicit(b.fraction, b.exponent);

    // align
    if a_exp > b_exp {
        for _ in 0..(a_exp - b_exp) {
            b_sig = shift_right24(b_sig);
        }
    } else if b_exp > a_exp {
        for _ in 0..(b_exp - a_exp) {
            a_sig = shift_right24(a_sig);
        }
    }
    let mut exp = a_exp.max(b_exp);

    // add
    let (mut sum, carry) = softu24_add(a_sig, b_sig);
    // carry = 1 iff it's +0.0 (sign bit 0) in our encoding
    if carry.sign_bit_is_one() {
        sum = shift_right24(sum);
        exp += 1;
    }

    SofterF32 {
        sign: a.sign, // assumes same sign
        exponent: to_softu8(exp as u8),
        fraction: drop_implicit(sum),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn add_softerf32_simple() {
        let a = to_softerf32(17.3);
        let b = to_softerf32(24.7);
        let c = softerf32_add(a, b);
        let c_f32 = from_softerf32(c);
        assert!((c_f32 - 42.0).abs() < 0.001);
    }
}
