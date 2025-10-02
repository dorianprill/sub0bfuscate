pub mod bitops;
pub mod softerfloat;
pub mod softuint;

// Optional: a small prelude for ergonomic imports.
pub mod prelude {
    pub use super::bitops::{adder, and, not, or, xor, Bit, BitExt, ONE, ZERO};
    pub use super::softerfloat::{from_softerf32, softerf32_add, to_softerf32, SofterF32};
    pub use super::softuint::{
        from_softu23, from_softu24, from_softu32, from_softu8, shift_right23, shift_right24,
        shift_right32, softu23_add, softu24_add, softu32_add, softu8_add, to_softu23, to_softu24,
        to_softu32, to_softu8, SoftU23, SoftU24, SoftU32, SoftU8,
    };
}
