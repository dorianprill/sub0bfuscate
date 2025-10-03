// Bit + helpers live here so every other module can just import one place.

pub type Bit = f32;
pub const ZERO: Bit = -0.0; // encodes bit 0
pub const ONE: Bit = 0.0; // encodes bit 1

// Extension trait for clean bit checks & construction.
pub trait BitExt: Sized {
    /// 1 for +0.0 (ONE), 0 for -0.0 (ZERO)
    fn sign_as_bit(self) -> u32;

    #[inline(always)]
    fn sign_bit_is_one(self) -> bool {
        self.sign_as_bit() == 1
    }

    #[inline(always)]
    fn sign_bit_is_zero(self) -> bool {
        self.sign_as_bit() == 0
    }

    fn from_bool(b: bool) -> Self;
}

impl BitExt for f32 {
    #[inline(always)]
    fn sign_as_bit(self) -> u32 {
        1 ^ ((self.to_bits() >> 31) & 1)
    }

    #[inline(always)]
    fn from_bool(b: bool) -> Self {
        if b {
            0.0
        } else {
            -0.0
        }
    }
}

// Logic gates using only ±0.0 arithmetic.
// RIGHT
#[inline(never)]
pub fn not(x: Bit) -> Bit {
    ZERO - x
}

#[inline(never)]
pub fn or(a: Bit, b: Bit) -> Bit {
    a - not(b)
}
#[inline(never)]
pub fn and(a: Bit, b: Bit) -> Bit {
    not(or(not(a), not(b)))
}
#[inline(never)]
pub fn xor(a: Bit, b: Bit) -> Bit {
    or(and(not(a), b), and(a, not(b)))
}

// Full adder primitive.
#[inline(never)]
pub fn adder(a: Bit, b: Bit, c: Bit) -> (Bit, Bit) {
    let s = xor(xor(a, b), c);
    let cout = or(and(xor(a, b), c), and(a, b));
    (s, cout)
}

#[cfg(test)]
pub mod tests {
    use crate::softcore::bitops::*;
    #[test]
    fn gate_truth_tables() {
        let z = ZERO; // -0.0 (bit 0)
        let o = ONE; // +0.0 (bit 1)

        // not
        assert!(not(z).sign_bit_is_one());
        assert!(not(o).sign_bit_is_zero());

        // and
        assert!(and(z, z).sign_bit_is_zero());
        assert!(and(z, o).sign_bit_is_zero());
        assert!(and(o, z).sign_bit_is_zero());
        assert!(and(o, o).sign_bit_is_one());

        // or
        assert!(or(z, z).sign_bit_is_zero());
        assert!(or(z, o).sign_bit_is_one());
        assert!(or(o, z).sign_bit_is_one());
        assert!(or(o, o).sign_bit_is_one());

        // xor
        assert!(xor(z, z).sign_bit_is_zero());
        assert!(xor(z, o).sign_bit_is_one());
        assert!(xor(o, z).sign_bit_is_one());
        assert!(xor(o, o).sign_bit_is_zero());
    }
}
