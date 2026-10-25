// QA asm probe: compile the product fma snapshot (non-test build) and check the
// emitted assembly really contains the hardware FMA instruction.
include!("fma_snapshot.rs");

use std::hint::black_box;

fn main() {
    let a = black_box(1.0000001f32);
    let b = black_box(0.9999999f32);
    let c = black_box(-1.0f32);
    let mut acc = 0.0f32;
    let mut index = 0u32;
    while index < 8 {
        acc = black_box(fma_f32(a + index as f32, b, c + acc));
        index += 1;
    }
    println!("asm_probe acc_bits={:08x}", acc.to_bits());
}
