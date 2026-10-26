fn main() {
    let tet: [f32; 12] = [
        -0.004_515_995, 0.05, 0.042_259_388,
        -0.004_515_995, -0.05, 0.042_259_388,
        -0.041_932_628, 0.0, -0.050_476_775,
        0.050_964_616, 0.0, -0.034_041_997,
    ];
    let model: [u32; 12] = [0xbb93faea,0x3d4ccccd,0x3d2d182e,0xbb93faea,0xbd4ccccd,0x3d2d182e,0xbd2bc18c,0x0,0xbd4ec0bc,0x3d50c046,0x0,0xbd0b6f9f];
    for i in 0..12 { println!("{} lit={:08x} model={:08x} {}", i, tet[i].to_bits(), model[i], if tet[i].to_bits()==model[i] {"OK"} else {"DIFF"}); }
}
