// // Turning arbitrary arithmetic into just subtractions of +0.0 and -0.0
// // by leveraging the IEEE 754 floating-point standard and the functional completeness of the logic gates {IMPLY, 0}

// type Bit = f32;
// const ZERO: Bit = -0.0;
// const ONE: Bit = 0.0;

// fn not(x: Bit) -> Bit {
//     ZERO - x
// }
// fn or(a: Bit, b: Bit) -> Bit {
//     a - not(b)
// }
// fn and(a: Bit, b: Bit) -> Bit {
//     not(or(not(a), not(b)))
// }
// fn xor(a: Bit, b: Bit) -> Bit {
//     or(and(not(a), b), and(a, not(b)))
// }
// fn adder(a: Bit, b: Bit, c: Bit) -> (Bit, Bit) {
//     let s = xor(xor(a, b), c);
//     let c = or(and(xor(a, b), c), and(a, b));
//     (s, c)
// }

// // Soft Integer Types
// // SoftU23 is used for the SofterF32 Mantissa

// type SoftU8 = [Bit; 8];

// type SoftU23 = [Bit; 23];

// type SoftU24 = [Bit; 24]; // used for addition and normalization

// type SoftU32 = [Bit; 32];

// pub fn softu32_add(a: SoftU32, b: SoftU32) -> (SoftU32, Bit) {
//     let mut out = [ZERO; 32];
//     let mut carry = ZERO;
//     for i in 0..32 {
//         let (s, c) = adder(a[i], b[i], carry);
//         out[i] = s;
//         carry = c;
//     }
//     (out, carry)
// }

// struct SofterF32 {
//     sign: Bit,
//     exponent: [Bit; 8],
//     fraction: [Bit; 23],
// }

// pub fn softu8_add(a: SoftU8, b: SoftU8) -> SoftU8 {
//     let (s0, c) = adder(a[0], b[0], ZERO);
//     let (s1, c) = adder(a[1], b[1], c);
//     let (s2, c) = adder(a[2], b[2], c);
//     let (s3, c) = adder(a[3], b[3], c);
//     let (s4, c) = adder(a[4], b[4], c);
//     let (s5, c) = adder(a[5], b[5], c);
//     let (s6, c) = adder(a[6], b[6], c);
//     let (s7, _) = adder(a[7], b[7], c);
//     [s0, s1, s2, s3, s4, s5, s6, s7]
// }

// pub fn to_softu8(x: u8) -> SoftU8 {
//     std::array::from_fn(|i| if (x >> i) & 1 == 1 { ONE } else { ZERO })
// }

// pub fn from_softu8(x: SoftU8) -> u8 {
//     (0..8)
//         .filter(|i| x[*i].signum() > 0.0)
//         .map(|i| 1 << i)
//         .sum()
// }

// #[inline(always)]
// fn to_soft_bit(bit_is_one: bool) -> Bit {
//     if bit_is_one {
//         ONE
//     } else {
//         ZERO
//     }
// }

// #[inline(always)]
// pub fn to_softu24(x: u32) -> SoftU24 {
//     let mut out = [ZERO; 24];
//     for i in 0..24 {
//         out[i] = to_soft_bit(((x >> i) & 1) != 0); // LSB -> index 0
//     }
//     out
// }

// #[inline(always)]
// pub fn to_softu32(x: u32) -> SoftU32 {
//     let mut out = [ZERO; 32];
//     for i in 0..32 {
//         out[i] = to_soft_bit(((x >> i) & 1) != 0); // LSB -> index 0
//     }
//     out
// }

// #[inline(always)]
// fn is_one(b: Bit) -> bool {
//     // +0.0 has sign bit 0; -0.0 has sign bit 1
//     (b.to_bits() >> 31) == 0
// }

// #[inline(always)]
// pub fn from_softu24(a: SoftU24) -> u32 {
//     let mut v = 0u32;
//     for i in 0..24 {
//         if is_one(a[i]) {
//             v |= 1u32 << i;
//         }
//     }
//     v
// }

// #[inline(always)]
// pub fn from_softu32(a: SoftU32) -> u32 {
//     let mut v = 0u32;
//     for i in 0..32 {
//         if is_one(a[i]) {
//             v |= 1u32 << i;
//         }
//     }
//     v
// }

// fn with_implicit(frac: SoftU23, exp: SoftU8) -> SoftU24 {
//     // LSB-first: indices 0..22 are fraction bits, index 23 is the implicit 1 for normals
//     let mut sig: SoftU24 = [ZERO; 24];
//     for i in 0..23 {
//         sig[i] = frac[i];
//     }
//     sig[23] = if from_softu8(exp) != 0 { ONE } else { ZERO }; // 0 for subnormals/zero
//     sig
// }

// fn drop_implicit(sig: SoftU24) -> SoftU23 {
//     // drop the top implicit bit; keep the lower 23 as stored fraction
//     let mut frac: SoftU23 = [ZERO; 23];
//     for i in 0..23 {
//         frac[i] = sig[i];
//     }
//     frac
// }

// fn softu24_add(a: SoftU24, b: SoftU24) -> (SoftU24, Bit) {
//     let mut result = [ZERO; 24];
//     let mut carry = ZERO;
//     for i in 0..24 {
//         let (sum, new_carry) = adder(a[i], b[i], carry);
//         result[i] = sum;
//         carry = new_carry;
//     }
//     (result, carry)
// }

// fn shift_right24(x: SoftU24) -> SoftU24 {
//     let mut result = [ZERO; 24];
//     for i in 0..23 {
//         result[i] = x[i + 1];
//     }
//     result[23] = ZERO;
//     result
// }

// impl SofterF32 {
//     fn to_f32(&self) -> f32 {
//         let sign = if self.sign.signum() > 0.0 { 0 } else { 1 << 31 };
//         let exponent = from_softu8(self.exponent) as u32;
//         let fraction = from_softu23(self.fraction);
//         let bits = sign | (exponent << 23) | fraction;
//         f32::from_bits(bits)
//     }

//     fn from_f32(x: f32) -> Self {
//         let bits = x.to_bits();
//         let sign = if bits & (1 << 31) == 0 { ONE } else { ZERO };
//         let exponent = to_softu8(((bits >> 23) & 0xFF) as u8);
//         let fraction = to_softu23(bits & 0x7FFFFF);
//         SofterF32 {
//             sign,
//             exponent,
//             fraction,
//         }
//     }
// }

// /// Converts a u32 into a Bit-array to be used as mantissa in our SofterF32 type
// fn to_softu23(x: u32) -> [Bit; 23] {
//     std::array::from_fn(|i| if (x >> i) & 1 == 1 { ONE } else { ZERO })
// }

// /// Converts a mantissa Bit-array from our SofterF32 type into a u32
// fn from_softu23(x: [Bit; 23]) -> u32 {
//     (0..23)
//         .filter(|i| x[*i].signum() > 0.0)
//         .map(|i| 1 << i)
//         .sum()
// }

// // Convert a regular f32 to a SofterF32 representation
// fn to_softerf32(x: f32) -> SofterF32 {
//     let bits: u32 = x.to_bits();
//     let sign_bit = if bits >> 31 == 0 { ONE } else { ZERO };
//     let exponent_bits = to_softu8(((bits >> 23) & 0xFF) as u8);
//     let mut fraction_bits = [ZERO; 23];
//     // if from_softu8(exponent_bits) != 0 {
//     //     // Check for non-zero exponent (ignoring denormalized numbers)
//     //     fraction_bits[22] = ONE; // Set the implicit bit
//     // }
//     for i in 0..23 {
//         fraction_bits[i] = if (bits >> i) & 1 == 1 { ONE } else { ZERO };
//     }

//     SofterF32 {
//         sign: sign_bit,
//         exponent: exponent_bits,
//         fraction: fraction_bits,
//     }
// }

// // Convert a SofterF32 to a regular f32
// fn from_softerf32(x: SofterF32) -> f32 {
//     let mut bits: u32 = 0;
//     bits |= (from_softu8(x.exponent) as u32) << 23;
//     for i in 0..23 {
//         if x.fraction[i].signum() > 0.0 {
//             bits |= 1 << i;
//         }
//     }
//     if x.sign.signum() <= 0.0 {
//         bits |= 1 << 31;
//     }
//     f32::from_bits(bits)
// }

// fn softu23_add(a: SoftU23, b: SoftU23) -> (SoftU23, Bit) {
//     let mut result = [ZERO; 23];
//     let mut carry = ZERO;
//     for i in 0..23 {
//         let (sum, new_carry) = adder(a[i], b[i], carry);
//         result[i] = sum;
//         carry = new_carry;
//     }
//     (result, carry)
// }

// fn softerf32_add(a: SofterF32, b: SofterF32) -> SofterF32 {
//     // (still assumes same sign; subtraction/opp signs can come later)
//     let mut a_exp = from_softu8(a.exponent) as i32;
//     let mut b_exp = from_softu8(b.exponent) as i32;

//     let mut a_sig = with_implicit(a.fraction, a.exponent);
//     let mut b_sig = with_implicit(b.fraction, b.exponent);

//     // align by shifting the one with the smaller exponent
//     if a_exp > b_exp {
//         for _ in 0..(a_exp - b_exp) {
//             b_sig = shift_right24(b_sig);
//         }
//     } else if b_exp > a_exp {
//         for _ in 0..(b_exp - a_exp) {
//             a_sig = shift_right24(a_sig);
//         }
//     }
//     let mut exp = a_exp.max(b_exp);

//     // add
//     let (mut sum, carry) = softu24_add(a_sig, b_sig);

//     // if carry out, normalize by shifting right once and increment exponent
//     if carry.signum() > 0.0 {
//         sum = shift_right24(sum);
//         exp += 1;
//     } else {
//         // ensure the top bit is the implicit leading 1 for normals; if it isn’t,
//         // you could add a simple left-normalize loop here. for now we keep it minimal.
//     }

//     // pack back: drop the implicit 1 from the stored fraction
//     let fraction = drop_implicit(sum);

//     SofterF32 {
//         sign: a.sign,                   // same-sign fast path
//         exponent: to_softu8(exp as u8), // max exponent (plus any normalization bump)
//         fraction,
//     }
// }

// fn shift_right(x: [Bit; 23]) -> [Bit; 23] {
//     let mut result = [ZERO; 23];
//     // right shift: bit i+1 -> i
//     for i in 0..22 {
//         result[i] = x[i + 1];
//     }
//     // highest bit gets zero on logical right shift
//     result[22] = ZERO;
//     result
// }

// fn is_overflow(x: [Bit; 23]) -> bool {
//     // Check the highest bit to see if there was an overflow.
//     x[22].signum() > 0.0
// }

use sub0bfuscate::softcore::prelude::*;

fn main() {
    // compute SoftInt sum on the FPU
    let a = to_softu8(23);
    let b = to_softu8(19);
    let (sum8, _carry8) = softu8_add(a, b);
    println!("{}", from_softu8(sum8)); // -> 42

    // compute SofterFloat sum with SoftInts
    let f1 = 2.0f32;
    let f2 = 1.8f32;

    let a_f32 = to_softerf32(f1);
    let b_f32 = to_softerf32(f2);
    let sum_f32 = softerf32_add(a_f32, b_f32);

    println!("{:.2}", from_softerf32(sum_f32)); // ~3.80
}
