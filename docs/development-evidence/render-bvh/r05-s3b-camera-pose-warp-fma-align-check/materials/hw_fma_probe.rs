// QA: hardware-FMA implementation of the host contraction order, evaluated on all 1003
// reference quaternions. This mirrors the production path of crate::math::fma::fma_f32
// (inlined `vfmadd231ss` when CPUID FMA is present) without touching the crate.
use std::arch::x86_64::*;
use std::io::{BufRead, BufWriter, Write};

#[target_feature(enable = "fma")]
unsafe fn fma(a: f32, b: f32, c: f32) -> f32 {
    _mm_cvtss_f32(_mm_fmadd_ss(_mm_set_ss(a), _mm_set_ss(b), _mm_set_ss(c)))
}

#[target_feature(enable = "fma")]
unsafe fn warp(quat: [u32; 4]) -> [u32; 9] {
    let q0 = f32::from_bits(quat[0]);
    let q1 = f32::from_bits(quat[1]);
    let q2 = f32::from_bits(quat[2]);
    let q3 = f32::from_bits(quat[3]);
    let q01 = q0 * q1;
    let q02 = q0 * q2;
    let q03 = q0 * q3;
    let q11 = q1 * q1;
    let e0 = fma(-q3, q3, fma(-q2, q2, fma(q0, q0, q11)));
    let e4 = fma(-q3, q3, fma(q2, q2, fma(q0, q0, -q11)));
    let e8 = fma(q3, q3, fma(-q2, q2, fma(q0, q0, -q11)));
    let two = 2.0f32;
    [
        e0.to_bits(),
        (two * fma(q1, q2, -q03)).to_bits(),
        (two * fma(q1, q3, q02)).to_bits(),
        (two * fma(q1, q2, q03)).to_bits(),
        e4.to_bits(),
        (two * fma(q2, q3, -q01)).to_bits(),
        (two * fma(q1, q3, -q02)).to_bits(),
        (two * fma(q2, q3, q01)).to_bits(),
        e8.to_bits(),
    ]
}

fn main() {
    assert!(std::is_x86_feature_detected!("fma"), "no hardware fma");
    let path = std::env::args().nth(1).expect("quats.txt");
    let out = std::env::args().nth(2).expect("out");
    let f = std::fs::File::open(path).unwrap();
    let mut w = BufWriter::new(std::fs::File::create(out).unwrap());
    for line in std::io::BufReader::new(f).lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let q: Vec<u32> = line
            .split_whitespace()
            .map(|t| u32::from_str_radix(t, 16).unwrap())
            .collect();
        let r = unsafe { warp([q[0], q[1], q[2], q[3]]) };
        let s: Vec<String> = r.iter().map(|b| format!("{:08x}", b)).collect();
        writeln!(w, "{}", s.join(" ")).unwrap();
    }
}
