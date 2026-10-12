//! 追踪状态写入与打包存储。
//! 静态证据不替代GPU实测。
use super::{Check, Stamp, between, digest, require};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path};

const FIXTURE: &str = include_str!("../../fixtures/precision/g01-state-packing.json");
const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
const SOURCES: &[(&str, &str)] = &[
    ("src/physics/resident.rs", RESIDENT),
    (
        "src/physics/mod.rs",
        include_str!("../../src/physics/mod.rs"),
    ),
    (
        "src/physics/attached.rs",
        include_str!("../../src/physics/attached.rs"),
    ),
    (
        "src/physics/sleep.rs",
        include_str!("../../src/physics/sleep.rs"),
    ),
    (
        "src/physics/flex.rs",
        include_str!("../../src/physics/flex.rs"),
    ),
    (
        "src/physics/flex_hessian.rs",
        include_str!("../../src/physics/flex_hessian.rs"),
    ),
    (
        "src/runtime/transfer.rs",
        include_str!("../../src/runtime/transfer.rs"),
    ),
];
const REMAINING: [&str; 6] = [
    "per-field-model-packing",
    "physical-locals-and-accumulators",
    "complete-output-mapping",
    "other-physics-state-paths",
    "upstream-generated-kernels",
    "measured-performance",
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Upstream {
    owner: String,
    name: String,
    declaration: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    owner: String,
    name: String,
    host_type: String,
    device_owner: String,
    device_field: String,
    device_type: String,
    classification: String,
    upstream: Vec<Upstream>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Packer {
    name: String,
    metadata: String,
    parameters: String,
    boundary: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    upstream_revision: String,
    product_revision: String,
    product_sources: Vec<Stamp>,
    states: Vec<State>,
    packers: Vec<Packer>,
    np01_complete: bool,
    remaining: Vec<String>,
}

fn inventory() -> Inventory {
    serde_json::from_str(FIXTURE).unwrap()
}

fn source(path: &str) -> Result<&'static str, String> {
    SOURCES
        .iter()
        .find_map(|(p, s)| (*p == path).then_some(*s))
        .ok_or_else(|| format!("unreviewed source: {path}"))
}

fn declaration_fields(owner: &str) -> Result<Vec<(String, String)>, String> {
    let body = between(RESIDENT, &format!("struct {owner} {{"), "}")?;
    body.lines()
        .map(str::trim)
        .filter_map(|s| s.strip_prefix("pub "))
        .map(|s| {
            s.strip_suffix(',')
                .and_then(|s| s.split_once(": "))
                .map(|(name, ty)| (name.into(), ty.into()))
                .ok_or("state declaration changed".into())
        })
        .collect()
}

fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1 && i.scope == "g01-state-and-packing-storage",
        "scope",
    )?;
    require(
        i.upstream_revision == super::REVISION
            && i.product_revision == "dcb1fae771c45ca5df823d0b2517f04724f625b2",
        "revision",
    )?;
    require(
        !i.np01_complete && i.remaining.iter().map(String::as_str).eq(REMAINING),
        "remaining precision work",
    )?;
    let paths: BTreeSet<_> = i.product_sources.iter().map(|s| s.path.as_str()).collect();
    require(
        paths.len() == i.product_sources.len()
            && paths == SOURCES.iter().map(|(p, _)| *p).collect(),
        "source coverage",
    )?;
    for stamp in &i.product_sources {
        require(
            digest(source(&stamp.path)?) == stamp.sha256,
            format!("source drift: {}", stamp.path),
        )?;
    }
    let mut expected = Vec::new();
    for owner in ["G01State", "G01SleepState"] {
        for (name, ty) in declaration_fields(owner)? {
            expected.push((owner, name, ty));
        }
    }
    require(
        i.states.len() == 15 && expected.len() == 15,
        "state coverage",
    )?;
    for (s, (owner, name, ty)) in i.states.iter().zip(expected) {
        require(
            s.owner == owner && s.name == name && s.host_type == ty,
            "state identity/type",
        )?;
        let (device_owner, device_field, device_type, classification, fields): (
            _,
            _,
            _,
            _,
            Vec<&str>,
        ) = match name.as_str() {
            "qpos" | "mocap_pos" | "mocap_quat" => (
                "KinematicsData",
                "state",
                "TransferBuffer<f32>",
                "same-scalar",
                vec![&name],
            ),
            "qvel" => (
                "ResidentFlexEdges",
                "qvel",
                "TransferBuffer<f32>",
                "same-scalar",
                vec!["qvel"],
            ),
            "rigid_cache" => (
                "KinematicsData",
                "rigid",
                "TransferBuffer<f32>",
                "same-scalar",
                vec![
                    "xpos", "xquat", "xmat", "xipos", "ximat", "xanchor", "xaxis",
                ],
            ),
            "com_cache" => (
                "KinematicsData",
                "com",
                "TransferBuffer<f32>",
                "derived-cache",
                vec!["body_subtreemass", "subtree_com", "cinert", "cdof"],
            ),
            "world_pose" => (
                "KinematicsData",
                "rigid",
                "TransferBuffer<f32>",
                "world-body-alias",
                vec!["xpos", "xquat"],
            ),
            "attached_cache" => (
                "KinematicsData",
                "attached",
                "TransferBuffer<f32>",
                "same-scalar",
                vec!["geom_xpos", "geom_xmat", "site_xpos", "site_xmat"],
            ),
            "flex_hessian_valid" => (
                "KinematicsData",
                "flex_validity",
                "Option<TransferBuffer<i32>>",
                "bool-i32",
                vec!["flex_hessian_valid"],
            ),
            "sleep" => (
                "ResidentSleep",
                "state",
                "TransferBuffer<i32>",
                "sleep-container",
                vec![],
            ),
            _ if owner == "G01SleepState" => (
                "ResidentSleep",
                "state",
                "TransferBuffer<i32>",
                "same-scalar",
                vec![&name],
            ),
            _ => return Err("unreviewed state route".into()),
        };
        require(
            (
                s.device_owner.as_str(),
                s.device_field.as_str(),
                s.device_type.as_str(),
                s.classification.as_str(),
            ) == (device_owner, device_field, device_type, classification),
            format!("device route: {name}"),
        )?;
        let body = between(RESIDENT, &format!("struct {device_owner} {{"), "}")?;
        require(
            body.lines()
                .any(|l| l.trim() == format!("{device_field}: {device_type},")),
            format!("device storage: {name}"),
        )?;
        require(
            s.upstream.iter().map(|u| u.name.as_str()).eq(fields),
            format!("upstream coverage: {name}"),
        )?;
        for u in &s.upstream {
            require(
                u.owner
                    == if u.name == "body_subtreemass" {
                        "Model"
                    } else {
                        "Data"
                    },
                "upstream owner",
            )?;
            let scalar = match u.name.as_str() {
                "cinert" => {
                    require(u.declaration.ends_with(", vec10)"), "inertia scalar")?;
                    "f32"
                }
                "cdof" => {
                    require(
                        u.declaration.ends_with(", wp.spatial_vector)"),
                        "motion scalar",
                    )?;
                    "f32"
                }
                _ => super::upstream_scalar(&u.declaration)?,
            };
            let expected = if name == "flex_hessian_valid" {
                "bool"
            } else if owner == "G01SleepState" {
                "i32"
            } else {
                "f32"
            };
            require(scalar == expected, format!("upstream scalar: {}", u.name))?;
        }
    }
    // 打包分母来自源码函数集。
    let packers: BTreeSet<_> = RESIDENT
        .lines()
        .filter_map(|l| {
            l.strip_prefix("fn pack_")
                .and_then(|s| s.split_once('('))
                .map(|(n, _)| format!("pack_{n}"))
        })
        .collect();
    require(
        packers.len() == 10
            && i.packers.len() == 10
            && i.packers
                .iter()
                .map(|p| p.name.clone())
                .collect::<BTreeSet<_>>()
                == packers,
        "packer coverage",
    )?;
    for p in &i.packers {
        require(
            p.metadata == "i32"
                && p.parameters
                    == if p.name == "pack_flex_edges" {
                        "none"
                    } else {
                        "f32"
                    },
            "packer scalar",
        )?;
        let boundary = match p.name.as_str() {
            "pack_spatial_tendon" => "f32-pulley-precompute",
            "pack_tendon" | "pack_flex_faces" => "unused-f32-sentinel",
            _ => "typed-copy-and-index-pack",
        };
        require(p.boundary == boundary, "packer boundary")?;
    }
    for anchor in [
        "metadata: Vec<i32>",
        "parameters: Vec<f32>",
        "metadata: session.upload(&self.metadata)?",
        "parameters: session.upload(&self.parameters)?",
        "self.state.write_range(offset, qpos)",
        "self.state.write_range(start + p, pos)?",
        ".write_range(start + self.layout.mocap_pos.total_elements() + q, quat)",
        ".write_range(4 + offset, qvel)",
        "data.rigid.write_range(4, cache)?",
        "data.com.write_range(4, cache)?",
        "data.attached.write_range(4, cache)?",
        "data.rigid.write_range(start, &pose[..3])?",
        ".write_range(start + 3 * data.layout.rigid.nbody, &pose[3..])?",
        "flags.iter().map(|&v| i32::from(v)).collect()",
        ".write_range(start, &row.packed())?",
        "result.extend_from_slice(&self.tree_awake)",
        "result.extend_from_slice(&[self.ntree_awake, self.nbody_awake, self.nv_awake])",
        "metadata.extend(f.tendon_limited.iter().copied().map(i32::from))",
        "i32::from(f.sleep_enabled && !f.island_disabled)",
        "i32::from(f.flex_centered[i])",
        "scale = 1.0 / fields.wrap_prm[i]",
        "*value = scale",
    ] {
        require(
            RESIDENT.contains(anchor),
            format!("reviewed packing route changed: {anchor}"),
        )?;
    }
    let transfer = source("src/runtime/transfer.rs")?;
    for anchor in [
        "source: &[T]",
        "HostStaging::<T>::new",
        "staging.values_mut().copy_from_slice(source)",
        ".memcpy_htod(staging.values(), &mut view)",
    ] {
        require(transfer.contains(anchor), "typed byte transfer")?;
    }
    Ok(())
}

fn verify_upstream(i: &Inventory, text: &str) -> Check {
    let input = super::inventory();
    require(
        digest(text) == input.upstream_types.sha256,
        "upstream type drift",
    )?;
    for s in &i.states {
        for u in &s.upstream {
            let body = match u.owner.as_str() {
                "Model" => between(text, "class Model:", "class Contact:")?,
                "Data" => between(text, "class Data:", "class InverseContext:")?,
                _ => return Err("unreviewed upstream owner".into()),
            };
            let line = format!("  {}: {}", u.name, u.declaration);
            require(
                body.lines().filter(|l| *l == line).count() == 1,
                format!("upstream declaration: {}", u.name),
            )?;
        }
    }
    for anchor in [
        "class vec10f(wp.types.vector(length=10, dtype=float)):",
        "vec10 = vec10f",
    ] {
        require(text.contains(anchor), "upstream inertia type")?;
    }
    Ok(())
}

fn reader_anchors(entry: &str) -> &'static [&'static str] {
    match entry {
        "tendon_wake" => &[
            "if(!m[6]||nt==0) return;",
            "const int* limited=num+nt;",
            "any_awake&&limited[t]",
        ],
        "flex_positions" => &["bool centered", "row[8]!=0", "float* result"],
        "flex_invalidate" => &["int* result", "flags[f]=0;"],
        "flex_hessian" => &["const int* valid", "if(flags[f]==1) continue;"],
        _ => &[],
    }
}

fn verify_reader(entry: &str, text: &str) -> Check {
    require(
        !reader_anchors(entry).is_empty(),
        "unreviewed device reader",
    )?;
    for anchor in reader_anchors(entry) {
        require(
            text.contains(anchor),
            format!("device reader: {entry}: {anchor}"),
        )?;
    }
    Ok(())
}

fn verify_captures(root: &Path) -> Result<usize, String> {
    let kernels: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    let artifacts = kernels["artifacts"].as_array().ok_or("capture inventory")?;
    let mut total = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        let mut entries = BTreeSet::new();
        for artifact in artifacts {
            let entry = artifact["entry"].as_str().ok_or("capture entry")?;
            if reader_anchors(entry).is_empty() {
                continue;
            }
            let hash = artifact["source_sha256"].as_str().ok_or("capture hash")?;
            let ptx_hash = artifact["ptx_sha256"].as_str().ok_or("PTX hash")?;
            require(
                [hash, ptx_hash]
                    .iter()
                    .all(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())),
                "capture path hash",
            )?;
            // 三类产物统一采用PTX哈希命名。
            // CUDA源码仍按源码哈希核验。
            let stem = format!("{entry}-{ptx_hash}");
            for (extension, expected) in [
                ("ptx", ptx_hash),
                (
                    "json",
                    artifact["receipt_sha256"].as_str().ok_or("receipt hash")?,
                ),
            ] {
                let bytes = std::fs::read(root.join(mode).join(format!("{stem}.{extension}")))
                    .map_err(|e| e.to_string())?;
                require(
                    format!("{:x}", Sha256::digest(&bytes)) == expected,
                    "capture companion drift",
                )?;
            }
            let path = root.join(mode).join(format!("{stem}.cu"));
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            require(
                format!("{:x}", Sha256::digest(&bytes)) == hash,
                "captured source drift",
            )?;
            let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
            verify_reader(entry, text)?;
            entries.insert(entry);
            total += 1;
        }
        require(entries.len() == 4, "reader capture coverage")?;
    }
    Ok(total)
}

#[test]
fn covers_state_declarations_and_all_ten_packing_functions() {
    validate(&inventory()).unwrap();
    // 编译器核对公开状态类型。
    // 本测试不构造设备会话。
    let s = mjwarp_rs::physics::G01State::default();
    let _: Vec<f32> = s.qpos;
    let _: Vec<f32> = s.qvel;
    let _: Vec<f32> = s.mocap_pos;
    let _: Vec<f32> = s.mocap_quat;
    let _: Option<Vec<bool>> = s.flex_hessian_valid;
}

#[test]
fn rejects_each_missing_duplicated_or_mistyped_state() {
    for index in 0..15 {
        for mutation in 0..6 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.states.remove(index);
                }
                1 => i.states[index] = i.states[(index + 1) % 15].clone(),
                2 => i.states[index].host_type = "Vec<f64>".into(),
                3 => i.states[index].device_type = "TransferBuffer<f64>".into(),
                4 => i.states[index].classification = "equivalent".into(),
                _ => i.states[index].device_field = "wrong_buffer".into(),
            }
            assert!(validate(&i).is_err(), "state {index}, mutation {mutation}");
        }
    }
}

#[test]
fn rejects_packing_source_or_coverage_drift() {
    for index in 0..10 {
        let mut i = inventory();
        i.packers.remove(index);
        assert!(validate(&i).is_err());
        let mut i = inventory();
        i.packers[index].parameters = "f64".into();
        assert!(validate(&i).is_err());
    }
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    let mut i = inventory();
    i.np01_complete = true;
    assert!(validate(&i).is_err());
    let mut i = inventory();
    i.remaining.clear();
    assert!(validate(&i).is_err());
}

#[test]
fn rejects_mislabelled_cache_components_and_device_readers() {
    for index in 0..15 {
        let mut i = inventory();
        if let Some(first) = i.states[index].upstream.first_mut() {
            first.name = "qacc".into();
            assert!(validate(&i).is_err());
        }
    }
    for (entry, path) in [
        ("tendon_wake", "src/physics/sleep.rs"),
        ("flex_positions", "src/physics/flex.rs"),
        ("flex_invalidate", "src/physics/flex.rs"),
        ("flex_hessian", "src/physics/flex_hessian.rs"),
    ] {
        let text = source(path).unwrap();
        verify_reader(entry, text).unwrap();
        for anchor in reader_anchors(entry) {
            assert!(verify_reader(entry, &text.replace(anchor, "changed_reader")).is_err());
        }
    }
}

#[test]
#[ignore = "reads frozen types and existing CUDA captures, with no GPU execution"]
fn audit_state_packing_sources_and_captured_readers() {
    let i = inventory();
    validate(&i).unwrap();
    let root = std::env::var_os("MJWARP_FROZEN_UPSTREAM_ROOT").expect("set upstream root");
    let upstream = std::fs::read_to_string(Path::new(&root).join("_src/types.py")).unwrap();
    verify_upstream(&i, &upstream).unwrap();
    let warp_root = std::env::var_os("MJWARP_FROZEN_WARP_ROOT").expect("set Warp root");
    let warp = std::fs::read_to_string(Path::new(&warp_root).join("_src/types.py")).unwrap();
    super::verify_external(&super::inventory(), &upstream, &warp).unwrap();
    assert!(warp.contains("class spatial_vectorf(vector(length=6, dtype=float32)):"));
    assert!(warp.contains("spatial_vector = spatial_vectorf"));
    assert!(
        warp.replace("\r\n", "\n")
            .contains("elif dtype is float:\n        dtype = float32")
    );
    assert!(verify_upstream(&i, &(upstream.clone() + "\n")).is_err());
    let mut changed = i.clone();
    changed.states[0].upstream[0].declaration = "array(\"nworld\", \"nv\", float)".into();
    assert!(verify_upstream(&changed, &upstream).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("set capture root");
    let captures = verify_captures(Path::new(&root)).unwrap();
    let report = serde_json::json!({
        "schema":1, "scope":i.scope, "inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision, "upstream_revision":i.upstream_revision,
        "state_declarations":15, "upstream_fields":25, "packing_functions":10,
        "captured_cuda_readers":captures, "gpu_executions":0, "python_executions":0,
        "np01_complete":false, "remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_STATE_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
