import io

p = r'D:\Rust\mjwarp-rs\tests\g10_p5_nvrtc_launcher.rs'
src = io.open(p, encoding='utf-8', newline='').read()
start = src.index('#[test]\nfn world_order_normalization_turns_row_swap_into_zero_diff()')

new_test = r'''#[test]
fn world_order_normalization_turns_row_swap_into_zero_diff() {
    // Chief 条件 2：同一份 dump（仓库内 `ellipsoid-box/out1`）构造 attempt1 观测到的「两行整体互换」，
    // 旧口径（原始行序逐字段比）应复现 attempt1 的 14 条差异；新口径（G5 行序归一化后比）应为 0 条。
    let expected = cases_root().join("ellipsoid-box").join("out1");
    let efc_width = read_efc_width(&expected);
    let ref_worldid = read_i32(&expected, "worldid");
    let rows = ref_worldid.len();
    assert_eq!(rows, 2, "ellipsoid-box dump 应为 2 行");
    let swap_row = |row: usize| -> usize {
        match row {
            0 => 1,
            1 => 0,
            other => other,
        }
    };
    let swap_f = |values: &[f32], width: usize| -> Vec<f32> {
        let mut out = Vec::with_capacity(values.len());
        for row in 0..rows {
            let source = swap_row(row);
            out.extend_from_slice(&values[source * width..(source + 1) * width]);
        }
        out
    };
    let swap_i = |values: &[i32], width: usize| -> Vec<i32> {
        let mut out = Vec::with_capacity(values.len());
        for row in 0..rows {
            let source = swap_row(row);
            out.extend_from_slice(&values[source * width..(source + 1) * width]);
        }
        out
    };

    let ref_dist = read_f32(&expected, "dist");
    let ref_pos = read_f32(&expected, "pos");
    let ref_frame = read_f32(&expected, "frame");
    let ref_includemargin = read_f32(&expected, "includemargin");
    let ref_friction = read_f32(&expected, "friction");
    let ref_solref = read_f32(&expected, "solref");
    let ref_solreffriction = read_f32(&expected, "solreffriction");
    let ref_solimp = read_f32(&expected, "solimp");
    let ref_adhesion = read_f32(&expected, "adhesion");
    let ref_dim = read_i32(&expected, "dim");
    let ref_geom = read_i32(&expected, "geom");
    let ref_efc = read_i32(&expected, "efc_address");
    let ref_type = read_i32(&expected, "type");
    let ref_id = read_i32(&expected, "geomcollisionid");
    let ref_nacon = read_i32(&expected, "nacon");
    let ref_nccd = read_i32(&expected, "nccd");
    let ref_overflow: Vec<u32> = read_i32(&expected, "overflow")
        .iter()
        .map(|value| *value as u32)
        .collect();

    let dev = CollisionGpuOutput {
        nacon: ref_nacon[0],
        nccd: ref_nccd[0],
        overflow: vec![0_u32; ref_overflow.len()],
        dist: swap_f(&ref_dist, 1),
        pos: swap_f(&ref_pos, 3),
        frame: swap_f(&ref_frame, 9),
        includemargin: swap_f(&ref_includemargin, 1),
        friction: swap_f(&ref_friction, 5),
        solref: swap_f(&ref_solref, 2),
        solreffriction: swap_f(&ref_solreffriction, 2),
        solimp: swap_f(&ref_solimp, 5),
        adhesion: swap_f(&ref_adhesion, 1),
        dim: swap_i(&ref_dim, 1),
        geom: swap_i(&ref_geom, 2),
        efc_address: swap_i(&ref_efc, efc_width),
        worldid: swap_i(&ref_worldid, 1),
        contact_type: swap_i(&ref_type, 1),
        geomcollisionid: swap_i(&ref_id, 1),
    };

    // 旧口径：原始行序逐字段比（attempt1 的失败集合应被复现）。
    let mut old = CaseReport::new("ellipsoid-box", CollisionKernel::Contact, &dev);
    check_dist(&mut old, "dist", &dev.dist, &ref_dist);
    check_vectors(&mut old, "pos", &dev.pos, &ref_pos, 3, false);
    check_vectors(&mut old, "frame", &dev.frame, &ref_frame, 9, true);
    check_exact_f32(&mut old, "includemargin", &dev.includemargin, &ref_includemargin);
    check_exact_f32(&mut old, "friction", &dev.friction, &ref_friction);
    check_exact_f32(&mut old, "solref", &dev.solref, &ref_solref);
    check_exact_f32(
        &mut old,
        "solreffriction",
        &dev.solreffriction,
        &ref_solreffriction,
    );
    check_exact_f32(&mut old, "solimp", &dev.solimp, &ref_solimp);
    check_exact_f32(&mut old, "adhesion", &dev.adhesion, &ref_adhesion);
    check_exact(&mut old, "dim", &dev.dim, &ref_dim);
    check_exact(&mut old, "geom", &dev.geom, &ref_geom);
    check_exact(&mut old, "efc_address", &dev.efc_address, &ref_efc);
    check_exact(&mut old, "worldid", &dev.worldid, &ref_worldid);
    check_exact(&mut old, "geomcollisionid", &dev.geomcollisionid, &ref_id);
    check_exact(&mut old, "contact_type", &dev.contact_type, &ref_type);

    // 新口径：G5 行序归一化（worldid 分组 + 组内 sort_contacts 键）后逐字段比。
    let device_order = g5_contact_order(&dev.worldid, &dev.dist, &dev.pos, &dev.geomcollisionid);
    let reference_order = g5_contact_order(&ref_worldid, &ref_dist, &ref_pos, &ref_id);
    let dev_f = |values: &[f32], width: usize| permute_rows_f32(values, width, &device_order);
    let ref_f = |values: &[f32], width: usize| permute_rows_f32(values, width, &reference_order);
    let dev_i = |values: &[i32], width: usize| permute_rows_i32(values, width, &device_order);
    let ref_i = |values: &[i32], width: usize| permute_rows_i32(values, width, &reference_order);

    let mut new = CaseReport::new("ellipsoid-box", CollisionKernel::Contact, &dev);
    check_dist(&mut new, "dist", &dev_f(&dev.dist, 1), &ref_f(&ref_dist, 1));
    check_vectors(
        &mut new,
        "pos",
        &dev_f(&dev.pos, 3),
        &ref_f(&ref_pos, 3),
        3,
        false,
    );
    check_vectors(
        &mut new,
        "frame",
        &dev_f(&dev.frame, 9),
        &ref_f(&ref_frame, 9),
        9,
        true,
    );
    check_exact_f32(
        &mut new,
        "includemargin",
        &dev_f(&dev.includemargin, 1),
        &ref_f(&ref_includemargin, 1),
    );
    check_exact_f32(
        &mut new,
        "friction",
        &dev_f(&dev.friction, 5),
        &ref_f(&ref_friction, 5),
    );
    check_exact_f32(&mut new, "solref", &dev_f(&dev.solref, 2), &ref_f(&ref_solref, 2));
    check_exact_f32(
        &mut new,
        "solreffriction",
        &dev_f(&dev.solreffriction, 2),
        &ref_f(&ref_solreffriction, 2),
    );
    check_exact_f32(&mut new, "solimp", &dev_f(&dev.solimp, 5), &ref_f(&ref_solimp, 5));
    check_exact_f32(
        &mut new,
        "adhesion",
        &dev_f(&dev.adhesion, 1),
        &ref_f(&ref_adhesion, 1),
    );
    check_exact(&mut new, "dim", &dev_i(&dev.dim, 1), &ref_i(&ref_dim, 1));
    check_exact(&mut new, "geom", &dev_i(&dev.geom, 2), &ref_i(&ref_geom, 2));
    check_exact(
        &mut new,
        "efc_address",
        &dev_i(&dev.efc_address, efc_width),
        &ref_i(&ref_efc, efc_width),
    );
    check_exact(
        &mut new,
        "worldid",
        &dev_i(&dev.worldid, 1),
        &ref_i(&ref_worldid, 1),
    );
    check_exact(
        &mut new,
        "geomcollisionid",
        &dev_i(&dev.geomcollisionid, 1),
        &ref_i(&ref_id, 1),
    );
    check_exact(
        &mut new,
        "contact_type",
        &dev_i(&dev.contact_type, 1),
        &ref_i(&ref_type, 1),
    );

    println!(
        "normalize_proof: raw_order_failures={} sorted_failures={} rows={rows} max_dist_ulp(old)={} max_pos_ulp(old)={} max_frame_ulp(old)={}",
        old.failures.len(),
        new.failures.len(),
        old.max_dist_ulp,
        old.max_pos_ulp,
        old.max_frame_ulp
    );
    assert_eq!(
        old.failures.len(),
        14,
        "旧口径应复现 attempt1 的 14 条差异，实际 {} 条：{:?}",
        old.failures.len(),
        old.failures
    );
    assert_eq!(
        new.failures.len(),
        0,
        "新口径应为 0 条差异，实际 {:?}",
        new.failures
    );
}
'''
io.open(p, 'w', encoding='utf-8', newline='\n').write(src[:start] + new_test)
print('proof test rewritten with real comparator (old vs new)')
