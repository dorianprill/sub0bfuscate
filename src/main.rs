type Bit = f32;
const ZERO: Bit = -0.0;
const ONE: Bit = 0.0;

fn not(x: Bit) -> Bit { ZERO - x }
fn or(a: Bit, b: Bit) -> Bit { a - not(b) }
fn and(a: Bit, b: Bit) -> Bit { not(or(not(a), not(b))) }
fn xor(a: Bit, b: Bit) -> Bit { or(and(not(a), b), and(a, not(b))) }
fn adder(a: Bit, b: Bit, c: Bit) -> (Bit, Bit) {
    let s = xor(xor(a, b), c);
    let c = or(and(xor(a, b), c), and(a, b));
    (s, c)
}

// Soft Integer Types
// SoftU23 is used for the SofterF32 Mantissa

type SoftU8 = [Bit; 8];

type SoftU23 = [Bit; 23];

type SoftU32 = [Bit; 32];

pub fn softu32_add(a: SoftU32, b: SoftU32) -> SoftU32 {
    let mut result = [ZERO; 32];
    let mut carry = ZERO;

    for i in 0..32 {
        let (sum, new_carry) = adder(a[i], b[i], carry);
        result[i] = sum;
        carry = new_carry;
    }

    result
}


struct SofterF32 {
    sign: Bit,
    exponent: [Bit; 8],
    fraction: [Bit; 23],
}



pub fn softu8_add(a: SoftU8, b: SoftU8) -> SoftU8 {
    let (s0, c) = adder(a[0], b[0], ZERO);
    let (s1, c) = adder(a[1], b[1], c);
    let (s2, c) = adder(a[2], b[2], c);
    let (s3, c) = adder(a[3], b[3], c);
    let (s4, c) = adder(a[4], b[4], c);
    let (s5, c) = adder(a[5], b[5], c);
    let (s6, c) = adder(a[6], b[6], c);
    let (s7, _) = adder(a[7], b[7], c);
    [s0, s1, s2, s3, s4, s5, s6, s7]
}

pub fn to_softu8(x: u8) -> SoftU8 {
    std::array::from_fn(|i| if (x >> i) & 1 == 1 { ONE } else { ZERO })
}

pub fn from_softu8(x: SoftU8) -> u8 {
    (0..8).filter(|i| x[*i].signum() > 0.0).map(|i| 1 << i).sum()
}


impl SofterF32 {
    fn to_f32(&self) -> f32 {
        let sign = if self.sign.signum() > 0.0 { 0 } else { 1 << 31 };
        let exponent = from_softu8(self.exponent) as u32;
        let fraction = from_softu23(self.fraction);
        let bits = sign | (exponent << 23) | fraction;
        f32::from_bits(bits)
    }

    fn from_f32(x: f32) -> Self {
        let bits = x.to_bits();
        let sign = if bits & (1 << 31) == 0 { ONE } else { ZERO };
        let exponent = to_softu8(((bits >> 23) & 0xFF) as u8);
        let fraction = to_softu23(bits & 0x7FFFFF);
        SofterF32 { sign, exponent, fraction }
    }
}

/// Converts a u32 into a Bit-array to be used as mantissa in our SofterF32 type
fn to_softu23(x: u32) -> [Bit; 23] {
    std::array::from_fn(|i| if (x >> i) & 1 == 1 { ONE } else { ZERO })
}

/// Converts a mantissa Bit-array from our SofterF32 type into a u32
fn from_softu23(x: [Bit; 23]) -> u32 {
    (0..23).filter(|i| x[*i].signum() > 0.0).map(|i| 1 << i).sum()
}



// Convert a regular f32 to a SofterF32 representation
fn to_softerf32(x: f32) -> SofterF32 {
    let bits: u32 = x.to_bits();
    let sign_bit = if bits >> 31 == 0 { ONE } else { ZERO };
    let exponent_bits = to_softu8(((bits >> 23) & 0xFF) as u8);
    let mut fraction_bits = [ZERO; 23];
    if from_softu8(exponent_bits) != 0 {  // Check for non-zero exponent (ignoring denormalized numbers)
        fraction_bits[22] = ONE;  // Set the implicit bit
    }
    for i in 0..23 {
        fraction_bits[i] = if (bits >> i) & 1 == 1 { ONE } else { ZERO };
    }

    SofterF32 {
        sign: sign_bit,
        exponent: exponent_bits,
        fraction: fraction_bits,
    }
}

// Convert a SofterF32 to a regular f32
fn from_softerf32(x: SofterF32) -> f32 {
    let mut bits: u32 = 0;
    bits |= (from_softu8(x.exponent) as u32) << 23;
    for i in 0..23 {
        if x.fraction[i].signum() > 0.0 {
            bits |= 1 << i;
        }
    }
    if x.sign.signum() <= 0.0 {
        bits |= 1 << 31;
    }
    f32::from_bits(bits)
}

fn softu23_add(a: SoftU23, b: SoftU23) -> SoftU23 {
    let mut result = [ZERO; 23];
    let mut carry = ZERO;

    for i in (0..23).rev() {
        let (sum, new_carry) = adder(a[i], b[i], carry);
        result[i] = sum;
        carry = new_carry;
    }

    result
}



fn softerf32_add(a: SofterF32, b: SofterF32) -> SofterF32 {
    let mut result = SofterF32 { sign: ONE, exponent: [ZERO; 8], fraction: [ZERO; 23] };

    let mut a_exp = from_softu8(a.exponent) as isize;
    let mut b_exp = from_softu8(b.exponent) as isize;

    // Copies for adjusting the fractions without mutating the original parameters
    let mut a_fraction = a.fraction;
    let mut b_fraction = b.fraction;

    // Align the numbers by exponents.
    let diff = a_exp - b_exp;
    if diff > 0 {
        for _ in 0..diff {
            b_fraction = shift_right(b_fraction); // Shift 'b' instead of 'a'
        }
    } else {
        for _ in 0..-diff {
            a_fraction = shift_right(a_fraction); // Shift 'a'
        }
    }

    
    // After aligning by exponents in softerf32_add
    println!("A mantissa after alignment: {:?}", a_fraction);
    println!("B mantissa after alignment: {:?}", b_fraction);
    
    // Set implicit bits immediately after conversion
    a_fraction[22] = if from_softu8(a.exponent) == 0 { ZERO } else { ONE };
    b_fraction[22] = if from_softu8(b.exponent) == 0 { ZERO } else { ONE };

    // Add the mantissas. Assuming they have the same sign for simplicity.
    let mantissa_sum = softu23_add(a_fraction, b_fraction);
    if is_overflow(mantissa_sum) {
        result.fraction = shift_right(mantissa_sum);
        a_exp += 1;
    } else {
        result.fraction = mantissa_sum;
    }
    
    // After adding mantissas
    println!("Mantissa sum: {:?}", mantissa_sum);

    result.exponent = to_softu8(a_exp as u8);
    result.sign = a.sign;  // Assuming same sign for simplicity.

    // This is a very basic version and does not handle normalization, rounding, etc.
    result
}


fn shift_right(x: [Bit; 23]) -> [Bit; 23] {
    let mut result = [ZERO; 23];
    for i in (1..23).rev() {
        result[i] = x[i - 1];
    }
    result[0] = ZERO;
    result
}

fn is_overflow(x: [Bit; 23]) -> bool {
    // Check the highest bit to see if there was an overflow.
    x[22].signum() > 0.0
}


fn main() {
    // compute SoftInt sum on the FPU
    let a = to_softu8(23);
    let b = to_softu8(19);
    println!("{}", from_softu8(softu8_add(a, b)));  // Outputs: 42

    // compute SofterFloat sum with SoftInts
    let f1 = 2.0f32; 
    let f2 = 1.8f32;

    let a_f32 = to_softerf32(f1);
    let b_f32 = to_softerf32(f2);
    let sum_f32 = softerf32_add(a_f32, b_f32);

    println!("{:.2}", from_softerf32(sum_f32));  // Outputs a result close to 3.8
}