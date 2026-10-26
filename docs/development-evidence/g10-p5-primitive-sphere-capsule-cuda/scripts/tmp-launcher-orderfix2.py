import io

p = r'D:\Rust\mjwarp-rs\tests\g10_p5_nvrtc_launcher.rs'
src = io.open(p, encoding='utf-8', newline='').read()

# 1) replace helper section: stable_world_order -> G5 sort-key order (worldid, dist, pos[0..2], geomcollisionid)
old_start = src.index('/// 行序归一化：多 world 的接触写入依赖共享 `atomicAdd`')
old_end = src.index('fn compare_case(')
new_helpers = r'''/// 行序归一化（G5 口径）：先按 `worldid` 分组，组内按 G5 `sort_contacts` 排序键
/// （`dist → pos[0..2] → geomcollisionid`）稳定排序，再逐字段比较。行序不作判据（多 world 的接触
/// 写入依赖共享 `atomicAdd`，跨 world 行序无保证）；判据本身不变（位级 / 4 ULP / 结构零容差）。
fn g5_contact_order(
    worldid: &[i32],
    dist: &[f32],
    pos: &[f32],
    geomcollisionid: &[i32],
) -> Vec<usize> {
    let mut order: Vec<usize> = (0..worldid.len()).collect();
    order.sort_by(|&a, &b| {
        worldid[a]
            .cmp(&worldid[b])
            .then(dist[a].total_cmp(&dist[b]))
            .then(pos[a * 3].total_cmp(&pos[b * 3]))
            .then(pos[a * 3 + 1].total_cmp(&pos[b * 3 + 1]))
            .then(pos[a * 3 + 2].total_cmp(&pos[b * 3 + 2]))
            .then(geomcollisionid[a].cmp(&geomcollisionid[b]))
    });
    order
}

fn permute_rows_f32(values: &[f32], width: usize, order: &[usize]) -> Vec<f32> {
    let mut out = Vec::with_capacity(values.len());
    for &row in order {
        out.extend_from_slice(&values[row * width..(row + 1) * width]);
    }
    out
}

fn permute_rows_i32(values: &[i32], width: usize, order: &[usize]) -> Vec<i32> {
    let mut out = Vec::with_capacity(values.len());
    for &row in order {
        out.extend_from_slice(&values[row * width..(row + 1) * width]);
    }
    out
}

'''
src = src[:old_start] + new_helpers + src[old_end:]

# 2) replace order construction inside compare_case
old_block = '''    // 按 worldid 稳定排序两侧行（跨 world 行序非判据；同一 world 内仍逐位比较）。
    let expected_worldid = read_i32(&expected, "worldid");
    if output.worldid.len() != rows || expected_worldid.len() != rows {
        report.length("worldid", output.worldid.len(), rows);
        report.length("worldid(ref)", expected_worldid.len(), rows);
        return report;
    }
    let device_order = stable_world_order(&output.worldid);
    let reference_order = stable_world_order(&expected_worldid);
'''
new_block = '''    // 行序归一化（G5 口径：worldid 分组 + 组内 sort_contacts 键）；判据与字段集不变。
    let expected_worldid = read_i32(&expected, "worldid");
    let expected_dist_raw = read_f32(&expected, "dist");
    let expected_pos_raw = read_f32(&expected, "pos");
    let expected_id_raw = read_i32(&expected, "geomcollisionid");
    if output.worldid.len() != rows
        || expected_worldid.len() != rows
        || output.dist.len() != rows
        || expected_dist_raw.len() != rows
        || output.pos.len() != rows * 3
        || expected_pos_raw.len() != rows * 3
        || output.geomcollisionid.len() != rows
        || expected_id_raw.len() != rows
    {
        report.length("rows(normalize)", output.worldid.len(), rows);
        return report;
    }
    let device_order = g5_contact_order(
        &output.worldid,
        &output.dist,
        &output.pos,
        &output.geomcollisionid,
    );
    let reference_order = g5_contact_order(
        &expected_worldid,
        &expected_dist_raw,
        &expected_pos_raw,
        &expected_id_raw,
    );
'''
assert src.count(old_block) == 1, src.count(old_block)
src = src.replace(old_block, new_block)

# 3) replace the CPU helper test with the dump-based proof (Chief condition 2) + a small unit check
old_test_start = src.index('#[test]\nfn normalize_rows_by_world_keeps_within_world_order()')
old_test_end = src.index('\n}\n', src.index('permute_rows_i32(&ids, 1, &order), vec![8, 7, 9]')) + 3
new_test = r'''#[test]
fn world_order_normalization_turns_row_swap_into_zero_diff() {
    // Chief 条件 2：用同一份 dump（仓库内 `ellipsoid-box/out1`）构造 attempt1 观测到的「两行整体互换」，
    // 旧口径（原始行序逐字段比）应出现 14 条差异，新口径（G5 行序归一化后比）应为 0 差异、0 ULP。
    let expected = cases_root().join("ellipsoid-box").join("out1");
    let worldid = read_i32(&expected, "worldid");
    let dist = read_f32(&expected, "dist");
    let pos = read_f32(&expected, "pos");
    let frame = read_f32(&expected, "frame");
    let ids = read_i32(&expected, "geomcollisionid");
    let rows = worldid.len();
    assert_eq!(rows, 2, "ellipsoid-box dump 应为 2 行");
    let swap = |index: usize| -> usize {
        match index {
            0 => 1,
            1 => 0,
            other => other,
        }
    };
    let dev_worldid: Vec<i32> = (0..rows).map(|i| worldid[swap(i)]).collect();
    let dev_dist: Vec<f32> = (0..rows).map(|i| dist[swap(i)]).collect();
    let dev_pos: Vec<f32> = (0..rows * 3).map(|i| pos[swap(i / 3) * 3 + i % 3]).collect();
    let dev_frame: Vec<f32> = (0..rows * 9).map(|i| frame[swap(i / 9) * 9 + i % 9]).collect();
    let dev_ids: Vec<i32> = (0..rows).map(|i| ids[swap(i)]).collect();

    // 旧口径：原始行序逐字段比较（worldid/dist 位级、pos/frame 位级）。
    let mut raw_diff = 0_usize;
    for index in 0..rows {
        if dev_worldid[index] != worldid[index] {
            raw_diff += 1;
        }
        if dev_dist[index].to_bits() != dist[index].to_bits() {
            raw_diff += 1;
        }
        for cell in 0..9 {
            if dev_frame[index * 9 + cell].to_bits() != frame[index * 9 + cell].to_bits() {
                raw_diff += 1;
            }
        }
        for axis in 0..3 {
            if dev_pos[index * 3 + axis].to_bits() != pos[index * 3 + axis].to_bits() {
                raw_diff += 1;
            }
        }
    }
    assert_eq!(
        raw_diff, 14,
        "旧口径在行交换 dump 上应出现 attempt1 同型的 14 条差异"
    );

    // 新口径：G5 行序归一化后逐字段比较 -> 0 差异。
    let device_order = g5_contact_order(&dev_worldid, &dev_dist, &dev_pos, &dev_ids);
    let reference_order = g5_contact_order(&worldid, &dist, &pos, &ids);
    let sorted_dev_worldid = permute_rows_i32(&dev_worldid, 1, &device_order);
    let sorted_ref_worldid = permute_rows_i32(&worldid, 1, &reference_order);
    let sorted_dev_dist = permute_rows_f32(&dev_dist, 1, &device_order);
    let sorted_ref_dist = permute_rows_f32(&dist, 1, &reference_order);
    let sorted_dev_pos = permute_rows_f32(&dev_pos, 3, &device_order);
    let sorted_ref_pos = permute_rows_f32(&pos, 3, &reference_order);
    let sorted_dev_frame = permute_rows_f32(&dev_frame, 9, &device_order);
    let sorted_ref_frame = permute_rows_f32(&frame, 9, &reference_order);
    let sorted_dev_ids = permute_rows_i32(&dev_ids, 1, &device_order);
    let sorted_ref_ids = permute_rows_i32(&ids, 1, &reference_order);
    assert_eq!(sorted_dev_worldid, sorted_ref_worldid);
    assert_eq!(sorted_dev_ids, sorted_ref_ids);
    assert!(sorted_dev_dist
        .iter()
        .zip(&sorted_ref_dist)
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    assert!(sorted_dev_pos
        .iter()
        .zip(&sorted_ref_pos)
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    assert!(sorted_dev_frame
        .iter()
        .zip(&sorted_ref_frame)
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    println!(
        "normalize_proof: raw_order_diff={raw_diff} sorted_diff=0 max_ulp=0 (ellipsoid-box dump, rows={rows})"
    );
}
'''
src = src[:old_test_start] + new_test + src[old_test_end:]
io.open(p, 'w', encoding='utf-8', newline='\n').write(src)
print('patched v2: G5 sort-key order + dump-based proof test')
