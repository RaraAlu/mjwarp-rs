use super::*;

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn matches_native_edges_across_uneven_worlds_and_repeated_updates() {
    let s = TransferSession::new(0).unwrap();
    let b = base();
    let e = fixture();
    let p = plan(&s, &b, &e);
    let (mut count, mut max) = (0, 0.0_f64);
    for nw in [1, 2, 5, 513] {
        let mut d = p.create_data(nw).unwrap();
        for round in 0..4 {
            for w in 0..nw {
                write(&mut d, w, &e["cases"][(w + round * 3) % 12]);
            }
            p.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            for w in 0..nw {
                max = max.max(native(&out, w, &e["cases"][(w + round * 3) % 12]));
                count += 1;
            }
            p.update_flex_edges(&mut d).unwrap();
            let repeated = d.readback().unwrap();
            for w in 0..nw {
                assert_eq!(bits(&out, w), bits(&repeated, w));
            }
        }
    }
    assert_eq!(count, 2084);
    println!(
        "G01-flex-edge states={count} scalars={} max_abs_error={max}",
        count * 578
    );
}
