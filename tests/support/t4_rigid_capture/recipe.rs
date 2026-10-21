//! 模板重建与产物复制。

use super::*;

// 所有嵌入刚体模板的入口都要重编译，不能只看入口名称。
pub(super) fn recipe(old: &evidence::Artifact) -> Option<(&'static str, String)> {
    if old.entry == "flex_faces" {
        let source = normalized(template(FACE, "FLEX_FACE_CUDA"));
        assert_eq!(
            digest(face_lineage::historical(&source).unwrap().as_bytes()),
            old.source_sha256
        );
        return Some(("flex_faces", source));
    }
    let rigid = sources();
    let (entry, candidates): (_, Vec<String>) = match old.entry.as_str() {
        "rigid_kinematics" => (ENTRY, rigid.into()),
        "attached_kinematics" | "initialize_attached" => {
            let attached = normalized(template(
                include_str!("../../../src/physics/attached.rs"),
                "ATTACHED_CUDA",
            ));
            (
                if old.entry == "attached_kinematics" {
                    "attached_kinematics"
                } else {
                    "initialize_attached"
                },
                rigid.iter().map(|r| format!("{r}\n{attached}")).collect(),
            )
        }
        "camlight_com" | "camlight_rigid" => {
            let fields = normalized(template(RESIDENT, "FIELD_BATCH_CUDA"));
            let camera = normalized(template(
                include_str!("../../../src/physics/camlight.rs"),
                "CAMLIGHT_CUDA",
            ));
            (
                if old.entry == "camlight_com" {
                    "camlight_com"
                } else {
                    "camlight_rigid"
                },
                vec![format!("{fields}\n{}\n{camera}", rigid[0])],
            )
        }
        _ => return None,
    };
    let mut matches = candidates.into_iter().filter(|s| {
        digest(
            before_rotate(&s.replace(
                "if(n==0.0f) return {0,0,0,1};",
                "if(n==0.0f) return {0,0,0,0};",
            ))
            .as_bytes(),
        ) == old.source_sha256
    });
    let source = matches.next().expect("unmapped embedded rigid template");
    assert!(matches.next().is_none());
    Some((entry, source))
}

pub(super) fn rotate_recipe(old: &evidence::Artifact) -> Option<(&'static str, String)> {
    if old.entry == "flex_faces" {
        return None;
    }
    let rigid = sources();
    let (entry, candidates): (_, Vec<String>) = match old.entry.as_str() {
        "rigid_kinematics" => (ENTRY, rigid.into()),
        "attached_kinematics" | "initialize_attached" => {
            let attached = normalized(template(
                include_str!("../../../src/physics/attached.rs"),
                "ATTACHED_CUDA",
            ));
            (
                if old.entry == "attached_kinematics" {
                    "attached_kinematics"
                } else {
                    "initialize_attached"
                },
                rigid.iter().map(|r| format!("{r}\n{attached}")).collect(),
            )
        }
        "camlight_com" | "camlight_rigid" => {
            let fields = normalized(template(RESIDENT, "FIELD_BATCH_CUDA"));
            let camera = normalized(template(
                include_str!("../../../src/physics/camlight.rs"),
                "CAMLIGHT_CUDA",
            ));
            (
                if old.entry == "camlight_com" {
                    "camlight_com"
                } else {
                    "camlight_rigid"
                },
                vec![format!("{fields}\n{}\n{camera}", rigid[0])],
            )
        }
        _ => return None,
    };
    let mut matches = candidates
        .into_iter()
        .filter(|source| digest(before_rotate(source).as_bytes()) == old.source_sha256);
    let source = matches.next().expect("unmapped pre-rotation variant");
    assert!(matches.next().is_none());
    Some((entry, source))
}

/// 逐项重建二十份沿用模板。
/// 它验证当前调用模板没有漂移。
pub(super) fn retained_recipe(old: &evidence::Artifact) -> Option<String> {
    let direct = match old.entry.as_str() {
        "com_velocity" => Some(template(COM_VELOCITY, "COM_VELOCITY_CUDA")),
        "fixed_tendon" => Some(template(FIXED_TENDON, "FIXED_TENDON_CUDA")),
        "flex_edges" => Some(template(FLEX_EDGE, "FLEX_EDGE_CUDA")),
        "flex_faces" => Some(template(FACE, "FLEX_FACE_CUDA")),
        "flex_hessian" | "flex_hessian_validate" => {
            Some(template(FLEX_HESSIAN, "FLEX_HESSIAN_CUDA"))
        }
        "flex_invalidate" | "flex_positions" => Some(template(FLEX, "FLEX_POSITION_CUDA")),
        "rigid_mass_matrix" => Some(template(MASS_MATRIX, "MASS_MATRIX_CUDA")),
        "rigid_mass_solve" => Some(template(MASS_SOLVE, "MASS_SOLVE_CUDA")),
        "rne_bias" => Some(template(RNE_BIAS, "RNE_BIAS_CUDA")),
        "sleep_copy" | "tendon_wake" => Some(template(SLEEP, "TENDON_WAKE_CUDA")),
        "tendon_indices" | "tendon_values" => Some(template(TENDON, "TENDON_CUDA")),
        _ => None,
    };
    if let Some(source) = direct {
        return Some(normalized(source));
    }
    let candidates = match old.entry.as_str() {
        "rigid_com_position" => {
            let com = normalized(template(PHYSICS, "COM_POSITION_CUDA"));
            let fields = normalized(template(RESIDENT, "FIELD_BATCH_CUDA"));
            vec![com.clone(), format!("{fields}\n{com}")]
        }
        "spatial_moment" | "spatial_site" | "spatial_wrap" => {
            let wrap = normalized(template(TENDON_WRAP, "TENDON_WRAP_CUDA"));
            let spatial = normalized(template(SPATIAL_TENDON, "SPATIAL_TENDON_CUDA"));
            vec![format!("{wrap}\n{spatial}")]
        }
        _ => return None,
    };
    let mut matches = candidates
        .into_iter()
        .filter(|source| digest(source.as_bytes()) == old.source_sha256);
    let source = matches.next().expect("unmapped retained template");
    assert!(matches.next().is_none());
    Some(source)
}

pub(super) fn copy_artifact(
    from: &std::path::Path,
    to: &std::path::Path,
    artifact: &evidence::Artifact,
) {
    for ext in ["cu", "ptx", "json"] {
        let name = format!("{}-{}.{}", artifact.entry, artifact.ptx_sha256, ext);
        fs::copy(from.join(&name), to.join(name)).unwrap();
    }
}
