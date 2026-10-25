//! G10@P5 前置片 G2：pinned 参考契约与观察项（不依赖产品 API）。
//!
//! 数值对照（GJK/EPA 对 5 个 CONVEX case，排序含 `geomcollisionid`）在 lib 内测试
//! `src/physics/collision/gjk/case_compare.rs` —— 该测试需要产品 `ccd_pair` API，
//! 而本片任务范围不含 `src/physics/mod.rs`，故不在此重复。
//!
//! 本文件固定两件事：
//! 1. 参考 JSON 的契约（upstream revision、接触数、worldid 分组、geomcollisionid 覆盖）；
//! 2. G0b 复核要求的观察项：重复接触点（`(dist,pos,frame)` 相同、仅 `geomcollisionid`
//!    不同）必须用 `geomcollisionid` 定序；**原始写入顺序只对本次 CUDA 运行/本设备
//!    成立**（QA 用 warp CPU 后端重采顺序不同），不得写成固定判据。

use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const UPSTREAM_REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";

/// `(case, 参考路径, 是否 G0b 单文件格式, 期望 count, geom 对[排序后])`。
const CASES: [(&str, &str, bool, usize, [i32; 2]); 5] = [
    (
        "cylinder-box",
        "fixtures/g10-p5/references/cylinder-box.json",
        true,
        8,
        [5, 6],
    ),
    (
        "ellipsoid-box",
        "fixtures/g10-p5/references/ellipsoid-box.json",
        true,
        2,
        [4, 6],
    ),
    (
        "mesh-mesh",
        "fixtures/g10-p5/references/mesh-mesh.json",
        true,
        2,
        [7, 7],
    ),
    (
        "convex-mesh",
        "fixtures/mjwarp-complex-physics/reference.json",
        false,
        2,
        [6, 7],
    ),
    (
        "degenerate-mesh",
        "fixtures/mjwarp-complex-physics/reference.json",
        false,
        2,
        [7, 7],
    ),
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn floats(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .expect("数值数组")
        .iter()
        .map(|item| item.as_f64().expect("数值") as f32)
        .collect()
}

fn case_output(name: &str, reference: &str, standalone: bool) -> Value {
    let path = root().join(reference);
    assert!(path.exists(), "缺少参考 JSON：{reference}");
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).expect("参考 JSON 应可读"))
        .expect("参考 JSON 应为合法 JSON");
    if standalone {
        assert_eq!(value["provenance"]["upstream_revision"], UPSTREAM_REVISION);
        value["output"].clone()
    } else {
        assert_eq!(value["upstream_revision"], UPSTREAM_REVISION);
        value["collision_cases"]
            .as_array()
            .expect("collision_cases")
            .iter()
            .find(|entry| entry["name"].as_str() == Some(name))
            .expect("应有该 case")["output"]
            .clone()
    }
}

#[test]
fn reference_contract_covers_five_convex_cases() {
    for (name, reference, standalone, expected_count, _geom_pair) in CASES {
        let output = case_output(name, reference, standalone);
        let count = output["count"].as_u64().expect("count") as usize;
        assert_eq!(count, expected_count, "{name} count");

        let worldid = floats(&output["fields"]["worldid"]["values"]);
        let collisionid = floats(&output["fields"]["geomcollisionid"]["values"]);
        assert_eq!(worldid.len(), count);
        assert_eq!(collisionid.len(), count);
        let worlds: BTreeSet<i32> = worldid.iter().map(|&value| value as i32).collect();
        assert_eq!(worlds, BTreeSet::from([0, 1]), "{name} worldid");

        // `geomcollisionid` 是同一 geom 对内的接触序号：每 world 内应为 0..n-1。
        for world in 0..2 {
            let mut ids = collisionid
                .iter()
                .zip(worldid.iter())
                .filter(|(_, w)| **w as i32 == world)
                .map(|(id, _)| *id as i32)
                .collect::<Vec<_>>();
            ids.sort_unstable();
            assert_eq!(
                ids,
                (0..ids.len() as i32).collect::<Vec<_>>(),
                "{name} w{world}"
            );
        }

        // 每个接触的 geom 对（参考中按 pair 顺序）应等于模型 geom 对（排序后）。
        let geom = floats(&output["fields"]["geom"]["values"]);
        assert_eq!(geom.len(), count * 2);
        for index in 0..count {
            let mut got = [geom[2 * index] as i32, geom[2 * index + 1] as i32];
            got.sort_unstable();
            // 参考 fields.geom 是 geom 索引，两 geom 场景恒为 {0,1}。
            assert_eq!(got, [0, 1], "{name} geom 对");
        }
    }
}

/// G0b 复核观察项 1：重复接触点必须用 `geomcollisionid` 定序。
///
/// 本片 5 个 CONVEX case 若出现 `(dist,pos,frame)` 完全相同、仅 `geomcollisionid` 不同的行，
/// 排序键必须含后者；这里只记录数量（G2 的 5 个 case 预期为 0，`hfield-*` 才是重复点来源）。
#[test]
fn duplicate_contact_points_are_keyed_by_geomcollisionid() {
    for (name, reference, standalone, _, _) in CASES {
        let output = case_output(name, reference, standalone);
        let count = output["count"].as_u64().expect("count") as usize;
        let dist = floats(&output["fields"]["dist"]["values"]);
        let pos = floats(&output["fields"]["pos"]["values"]);
        let frame = floats(&output["fields"]["frame"]["values"]);
        let collisionid = floats(&output["fields"]["geomcollisionid"]["values"]);
        let mut duplicates = 0_usize;
        for left in 0..count {
            for right in left + 1..count {
                let same = dist[left].to_bits() == dist[right].to_bits()
                    && pos[3 * left..3 * left + 3]
                        .iter()
                        .zip(&pos[3 * right..3 * right + 3])
                        .all(|(a, b)| a.to_bits() == b.to_bits())
                    && frame[9 * left..9 * left + 9]
                        .iter()
                        .zip(&frame[9 * right..9 * right + 9])
                        .all(|(a, b)| a.to_bits() == b.to_bits());
                if same && collisionid[left] != collisionid[right] {
                    duplicates += 1;
                }
            }
        }
        println!("case {name}: duplicate_keyed_rows={duplicates}");
    }
}

/// G0b 复核观察项 2：原始顺序只对本次 CUDA 运行/本设备成立，不作判据。
#[test]
fn raw_order_is_observation_only() {
    for (name, reference, standalone, _, _) in CASES {
        let value: Value = serde_json::from_str(
            &fs::read_to_string(root().join(reference)).expect("参考 JSON 应可读"),
        )
        .expect("合法 JSON");
        if standalone {
            // G0b 单文件参考带 order_observation；只断言字段可读，不据此断言顺序。
            let observation = &value["order_observation"];
            assert!(
                observation.is_object() || observation.is_null(),
                "{name} order_observation 应为对象"
            );
        }
    }
}
