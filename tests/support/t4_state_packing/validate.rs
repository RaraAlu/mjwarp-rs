//! 状态声明与打包函数校验。

use super::*;

pub(super) fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1 && i.scope == "g01-state-and-packing-storage",
        "scope",
    )?;
    require(
        i.upstream_revision == super::super::REVISION
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
            body.lines().any(|l| {
                l.trim().trim_start_matches("pub(super) ")
                    == format!("{device_field}: {device_type},")
            }),
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
                _ => super::super::upstream_scalar(&u.declaration)?,
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
            let l = l.trim_start_matches("pub(super) ");
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
        "metadata: SleepCopyMetadataBuffer::upload(session, metadata)?",
        "parameters: session.upload(&parameters)?",
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
