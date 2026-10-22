use super::inventory::*;
use super::*;

pub(super) fn boundaries() -> Check {
    for (text, anchors) in [
        (
            MODEL,
            &[
                "let width = count.checked_mul(21).ok_or_else(overflow)?;",
                "\"noncanonical_21_coefficient_range\"",
            ][..],
        ),
        (
            FLEX,
            &["if order != -1 {", "\"unsupported_flex_interpolation\""][..],
        ),
        (
            RESIDENT,
            &[
                "faces: Option<TransferBuffer<f32>>",
                "hessian: Option<TransferBuffer<f32>>",
                "\"resident_flex_hessian_output\"",
                "\"resident_flex_face_output\"",
                "if value != 0 && value != 1",
                "decoded.extend(values.iter().map(|&v| v == 1));",
            ][..],
        ),
        (TRANSFER, &["target.copy_from_slice(staging.values());"][..]),
        (
            HESSIAN,
            &[
                "const int tri[6]={1,2,2,0,0,1};",
                "const int tet[12]={0,1,1,2,2,0,2,3,0,3,1,3};",
            ][..],
        ),
    ] {
        for a in anchors {
            anchor(text, a)?;
        }
    }
    let stage = between(RESIDENT, "pub fn update_flex_hessian(", "\n    ///")?;
    anchor(
        stage,
        "data.ready.require(data.ready.flex, \"flex_positions\")?;",
    )?;
    anchor(
        stage,
        "data.ready.require(data.ready.edges, \"flex_edges\")?;",
    )?;
    require(
        stage.find("device.kernel.launch(").ok_or("matrix stage")?
            < stage
                .find("device.validate_kernel.launch(")
                .ok_or("validity stage")?,
        "publish order",
    )
}
pub(super) type Texts = BTreeMap<String, String>;
pub(super) fn verify_external(i: &Inventory, texts: &Texts) -> Check {
    let get = |p: &str| texts.get(p).ok_or_else(|| format!("external: {p}"));
    for s in &i.external_sources {
        require(digest(get(&s.path)?) == s.sha256, "external drift")?;
    }
    super::super::super::verify_external(
        &super::super::super::inventory(),
        get("upstream/_src/types.py")?,
        get("warp/_src/types.py")?,
    )?;
    for g in &i.groups {
        let section = body(get(&g.external)?, &g.symbol)?;
        for a in &g.upstream {
            anchor(section, a)?;
        }
    }
    let types = get("upstream/_src/types.py")?;
    for a in [
        "class vec6f(wp.types.vector(length=6, dtype=float)):",
        "vec6 = vec6f",
        "class mat43f(wp.types.matrix(shape=(4, 3), dtype=float)):",
        "mat43 = mat43f",
        "class mat63f(wp.types.matrix(shape=(6, 3), dtype=float)):",
        "mat63 = mat63f",
        "class mat66f(wp.types.matrix(shape=(6, 6), dtype=float)):",
        "mat66 = mat66f",
        "FLEX_STIFFNESS_3D = 24 if util_pkg.check_version(\"mujoco>=3.14.1.dev989511280\") else 21",
        "flex_hessian_valid: array(\"nworld\", \"nflex\", bool)",
    ] {
        anchor(types, a)?;
    }
    let data = types.split_once("class Data:").ok_or("Data")?.1;
    for o in &i.outputs {
        anchor(data, &format!("  {}: {}\n", o.name, o.declaration))?;
        anchor(body(get(&o.external)?, &o.symbol)?, &o.upstream_write)?;
    }
    let passive = get("upstream/_src/passive.py")?;
    anchor(
        body(passive, "stretch_stiffness")?,
        "tension[e] = wp.max(tension[e], 0.0)",
    )?;
    for a in [
        "shape=(6, 2), dtype=int)",
        "wp.matrix(1, 2, 2, 0, 0, 1,",
        "wp.matrix(0, 1, 1, 2, 2, 0, 2, 3, 0, 3, 1, 3,",
    ] {
        anchor(body(passive, "stretch_edge_endpoints")?, a)?;
    }
    // 只读源码不能把数值重排及24系数分支判为等价。
    anchor(
        body(passive, "_flex_hessian_elem")?,
        "local_elemid * (FLEX_STIFFNESS_3D if dim == 3 else 21)",
    )?;
    anchor(get("warp/native/builtin.h")?, "#define kEps 0.0f")?;
    Ok(())
}
pub(super) fn verify_generated(cuda: &str, ptx: &[u8]) -> Check {
    let pins: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/precision/g01-zero-quaternion.json"
    ))
    .unwrap();
    require(
        format!("{:x}", Sha256::digest(cuda.as_bytes()))
            == pins["upstream_cuda_sha256"].as_str().unwrap()
            && format!("{:x}", Sha256::digest(ptx))
                == pins["upstream_ptx_sha256"].as_str().unwrap(),
        "upstream artifact drift",
    )?;
    let polar = between(
        cuda,
        "static CUDA_CALLABLE wp::quat_t<wp::float32> mat33_to_quat_polar_0(",
        "\n}",
    )?;
    for a in [
        "wp::mat_t<3, 3, wp::float32> var_F",
        "const wp::int32 var_5 = 50;",
        "const wp::float32 var_34 = 1e-10;",
        "const wp::float32 var_38 = 1e-06;",
        "const wp::float32 var_41 = 0.5;",
        "var_57 = wp::mul(var_56, var_4);",
        "var_58 = wp::normalize(var_57);",
    ] {
        anchor(polar, a)?;
    }
    require(
        !polar.contains("wp::float64") && !polar.contains("wp::float16"),
        "polar scalar",
    )?;
    let face = between(
        cuda,
        "extern \"C\" __global__ void _flex_face_kinematics_9a1620ae_cuda_kernel_forward(",
        "\n}",
    )?;
    for a in [
        "wp::array_t<wp::vec_t<3, wp::float32>> var_face_xpos_out",
        "wp::array_t<wp::quat_t<wp::float32>> var_face_quat_out",
        "var_757 = mat33_to_quat_polar_0(var_756);",
    ] {
        anchor(face, a)?;
    }
    anchor(
        std::str::from_utf8(ptx).map_err(|e| e.to_string())?,
        ".visible .entry _flex_face_kinematics_9a1620ae_cuda_kernel_forward(",
    )?;
    Ok(())
}
pub(super) fn verify_captures(history: &Path, current: &Path) -> Result<usize, String> {
    let base: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/precision/t4-kernels.json"
    ))
    .unwrap();
    let now: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../fixtures/precision/t4-current-kernels.json"
    ))
    .unwrap();
    let names = ["flex_faces", "flex_hessian", "flex_hessian_validate"];
    let rows: Vec<_> = base["artifacts"]
        .as_array()
        .ok_or("artifacts")?
        .iter()
        .filter(|a| names.contains(&a["entry"].as_str().unwrap_or("")))
        .collect();
    require(
        rows.len() == 3
            && rows
                .iter()
                .map(|r| r["entry"].as_str().unwrap())
                .collect::<BTreeSet<_>>()
                == names.into_iter().collect(),
        "capture denominator",
    )?;
    let mut count = 0;
    for dir in [
        history.join("debug-ptx"),
        history.join("release-ptx"),
        current.to_path_buf(),
    ] {
        for old in &rows {
            let matches: Vec<_> = now["artifacts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["entry"] == old["entry"])
                .collect();
            require(matches.len() == 1, "current lineage denominator")?;
            let fresh = matches[0];
            if old["entry"] != "flex_faces" {
                require(fresh == *old, "unrelated capture drift")?;
            }
            let a = if dir == current { fresh } else { *old };
            let entry = a["entry"].as_str().unwrap();
            let hash = a["ptx_sha256"].as_str().ok_or("hash")?;
            require(
                hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
                "path hash",
            )?;
            for (ext, key) in [
                ("cu", "source_sha256"),
                ("ptx", "ptx_sha256"),
                ("json", "receipt_sha256"),
            ] {
                let bytes = std::fs::read(dir.join(format!("{entry}-{hash}.{ext}")))
                    .map_err(|e| e.to_string())?;
                require(
                    format!("{:x}", Sha256::digest(&bytes)) == a[key].as_str().unwrap(),
                    "capture drift",
                )?;
                if ext == "cu" {
                    let (text, name) = if entry == "flex_faces" {
                        (FACE, "FLEX_FACE_CUDA")
                    } else {
                        (HESSIAN, "FLEX_HESSIAN_CUDA")
                    };
                    let text = if entry == "flex_faces" && dir != current {
                        super::super::face_lineage::historical(text)?
                    } else {
                        text.replace("\r\n", "\n")
                    };
                    require(
                        std::str::from_utf8(&bytes).map_err(|e| e.to_string())?
                            == between(&text, &format!("const {name}: &str = r#\""), "\"#;")?,
                        "current full template",
                    )?;
                } else if ext == "json" {
                    let r: serde_json::Value =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    require(
                        r["entry"] == entry
                            && r["passed"] == true
                            && r["precisionGate"] == "f32-physics"
                            && r["fmad"] == true
                            && r["useFastMath"] == false,
                        "receipt options",
                    )?;
                }
            }
            count += 1;
        }
    }
    Ok(count)
}
