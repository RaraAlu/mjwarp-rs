// QA: per-step comparison of f32::mul_add vs exact single-rounding (values computed offline).
use std::hint::black_box;

fn main() {
    let q = [
        f32::from_bits(0x3f3d3c57),
        f32::from_bits(0x3f01dbed),
        f32::from_bits(0x3e805783),
        f32::from_bits(0x3ebb0653),
    ]
    .map(black_box);
    let [q0, q1, q2, q3] = q;
    let q11 = black_box(q1 * q1);
    let s1 = q0.mul_add(q0, q11);
    let s2 = q2.mul_add(-q2, black_box(s1));
    let s3 = q3.mul_add(-q3, black_box(s2));
    println!("q11={:08x} s1={:08x} s2={:08x} s3={:08x}",
             q11.to_bits(), s1.to_bits(), s2.to_bits(), s3.to_bits());
    // also: single-step cases for offline exact check
    let a = black_box(f32::from_bits(0x3f3d3c57));
    let b = black_box(f32::from_bits(0x3f3d3c57));
    let c = black_box(q11);
    println!("fma(a,a,c)={:08x}", a.mul_add(b, c).to_bits());
}
