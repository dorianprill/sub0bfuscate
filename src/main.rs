// // Turning arbitrary arithmetic into just subtractions of +0.0 and -0.0
// // by leveraging the IEEE 754 floating-point standard and the functional completeness of the logic gates {IMPLY, 0}


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
