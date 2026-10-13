//! 逐字段核对批量打包与质心内部量。
//! 此切片不代表完整精度验收。

/// 面与矩阵新增切片；不执行旧满额选择。
mod flex_matrices {
    use super::super::{Check, Stamp, between, digest, require};
    use serde::Deserialize;
    use sha2::{Digest, Sha256};
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::Path,
    };

    const FIXTURE: &str = include_str!("../../docs/evidence/t4-g03-retest/flex-matrices/map.json");
    const FACE: &str = include_str!("../../src/physics/flex_face.rs");
    const HESSIAN: &str = include_str!("../../src/physics/flex_hessian.rs");
    const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
    const MODEL: &str = include_str!("../../src/model/flex_hessian.rs");
    const FLEX: &str = include_str!("../../src/model/flex.rs");
    const TRANSFER: &str = include_str!("../../src/runtime/transfer.rs");
    const SOURCES: &[(&str, &str)] = &[
        ("src/physics/flex_face.rs", FACE),
        ("src/physics/flex_hessian.rs", HESSIAN),
        ("src/physics/resident.rs", RESIDENT),
        ("src/model/flex_hessian.rs", MODEL),
        ("src/model/flex.rs", FLEX),
        ("src/runtime/transfer.rs", TRANSFER),
    ];
    const EXTERNAL: &[&str] = &[
        "upstream/_src/smooth.py",
        "upstream/_src/support.py",
        "upstream/_src/passive.py",
        "upstream/_src/types.py",
        "warp/_src/types.py",
        "warp/native/quat.h",
        "warp/native/builtin.h",
    ];
    const GROUPS: &[&str] = &[
        "face-nodes|linear-only|f32|src/physics/flex_face.rs|upstream/_src/smooth.py|_flex_face_kinematics|2|2",
        "face-gradient|linear-only|f32|src/physics/flex_face.rs|upstream/_src/smooth.py|_flex_face_kinematics|1|4",
        "face-tangents|same-loop-order|f32|src/physics/flex_face.rs|upstream/_src/smooth.py|_flex_face_kinematics|3|5",
        "face-columns|column-major-private-layout|f32|src/physics/flex_face.rs|upstream/_src/smooth.py|_flex_face_kinematics|4|4",
        "polar-initial-stop|same-constants|f32|src/physics/flex_face.rs|upstream/_src/support.py|mat33_to_quat_polar|3|3",
        "polar-rotation-matrix|different-unit-quaternion-expression|f32|src/physics/flex_face.rs|warp/native/quat.h||3|4",
        "polar-omega|same-scalar-generated-checked|f32|src/physics/flex_face.rs|upstream/_src/support.py|mat33_to_quat_polar|2|3",
        "polar-axis|same-scalar|f32|src/physics/flex_face.rs|upstream/_src/support.py|mat33_to_quat_polar|2|4",
        "polar-product|different-grouping|f32|src/physics/flex_face.rs|warp/native/quat.h||2|2",
        "polar-normalize|divide-versus-reciprocal|f32|src/physics/flex_face.rs|warp/native/quat.h||2|2",
        "cache-clear|valid-skips-clear|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|_flex_hessian_clear|3|3",
        "edge-vectors|same-endpoint-order|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_edge_vectors|2|2",
        "elongation|same-scalar|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_elongation|1|1",
        "metric|21-coefficient-only|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_metric|2|4",
        "tension|same-row-order|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_tension|2|3",
        "material-block|unscaled-only|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_stiffness_block|2|2",
        "geometric-block|zero-sign-evaluation-differs|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|stretch_stiffness_block|3|4",
        "edge-scatter|serial-versus-atomic|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|_flex_hessian_elem|2|3",
        "diagonal-scatter|serial-versus-atomic|f32|src/physics/flex_hessian.rs|upstream/_src/passive.py|_flex_hessian_diag|2|3",
        "cache-publish|private-bool-adapter|i32|src/physics/flex_hessian.rs|upstream/_src/passive.py|flex_hessian|2|1",
        "active-gate|host-derived-bool|bool|src/model/flex_hessian.rs|upstream/_src/passive.py|_flex_hessian_active|5|3",
    ];
    const OUTPUTS: &[&str] = &[
        "face_xpos|f32|src/physics/flex_face.rs|array(\"nworld\", \"nflexface\", 9, wp.vec3)|float* positions=result+4+(unsigned long long)w*31ull*nf;|out[k]=nx[3*n+k];|&values[..27 * self.layout.nf]|upstream/_src/smooth.py|_flex_face_kinematics|face_xpos_out[worldid, face_id, local_idx] = node_pos",
        "face_quat|f32|src/physics/flex_face.rs|array(\"nworld\", \"nflexface\", wp.quat)|float* quats=positions+27*nf;|face_polar(quats+4*f,F);|&values[27 * self.layout.nf..]|upstream/_src/smooth.py|_flex_face_kinematics|face_quat_out[worldid, face_id] = support.mat33_to_quat_polar(F)",
        "flexvert_hessian|f32|src/physics/flex_hessian.rs|array(\"nworld\", \"nflexvert\", vec6)|float* diag=result+4+(unsigned long long)w*(6ull*nv+9ull*ne);|d0[k]-=b[upper[k]];d1[k]-=b[lower[k]];|&v[..6 * self.layout.nv]|upstream/_src/passive.py|_flex_hessian_diag|wp.atomic_add(flexvert_hessian_out, worldid, va + vert[0], d0)",
        "flexedge_hessian|f32|src/physics/flex_hessian.rs|array(\"nworld\", \"nflexedge\", wp.mat33)|float* blocks=diag+6*nv;|blocks[9*edge+k]+=block[k];|&v[6 * self.layout.nv..]|upstream/_src/passive.py|_flex_hessian_elem|wp.atomic_add(flexedge_hessian_out, worldid, edge_id, block)",
    ];
    const REMAINING: &[&str] = &[
        "polar-matrix-product-and-normalization-order",
        "serial-versus-atomic-accumulation",
        "zero-sign-nonfinite-intermediates",
        "upstream-hessian-generated-artifacts",
        "higher-order-faces-and-24-coefficient-models",
        "full-pipeline-current-numerics",
        "measured-performance",
    ];
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Group {
        id: String,
        classification: String,
        scalar: String,
        source: String,
        external: String,
        symbol: String,
        product: Vec<String>,
        upstream: Vec<String>,
    }
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Output {
        name: String,
        scalar: String,
        source: String,
        declaration: String,
        pointer: String,
        write: String,
        view: String,
        external: String,
        symbol: String,
        upstream_write: String,
    }
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Inventory {
        schema: u32,
        scope: String,
        product_revision: String,
        upstream_revision: String,
        product_sources: Vec<Stamp>,
        external_sources: Vec<Stamp>,
        groups: Vec<Group>,
        outputs: Vec<Output>,
        remaining: Vec<String>,
        np01_complete: bool,
        np02_complete: bool,
        equivalence_eligible: bool,
    }
    fn inventory() -> Inventory {
        serde_json::from_str(FIXTURE).unwrap()
    }
    fn anchor(text: &str, needle: &str) -> Check {
        require(
            !needle.trim().is_empty() && text.contains(needle),
            format!("anchor: {needle}"),
        )
    }
    fn source(path: &str) -> Result<&'static str, String> {
        SOURCES
            .iter()
            .find_map(|(p, t)| (*p == path).then_some(*t))
            .ok_or_else(|| path.into())
    }
    fn body<'a>(text: &'a str, symbol: &str) -> Result<&'a str, String> {
        if symbol.is_empty() {
            return Ok(text);
        }
        text.split_once(&format!("def {symbol}("))
            .map(|(_, s)| s.split("\n\n\n").next().unwrap())
            .ok_or_else(|| format!("symbol: {symbol}"))
    }
    fn validate(i: &Inventory) -> Check {
        require(
            i.schema == 1
                && i.scope == "flex-face-hessian-internal"
                && i.product_revision == "bdcc9d97482d629c40bcabfa52c0c159590bcd4d"
                && i.upstream_revision == super::super::REVISION
                && !i.np01_complete
                && !i.np02_complete
                && !i.equivalence_eligible
                && i.remaining
                    .iter()
                    .map(String::as_str)
                    .eq(REMAINING.iter().copied()),
            "boundary",
        )?;
        require(
            i.product_sources.len() == SOURCES.len()
                && i.product_sources
                    .iter()
                    .map(|s| s.path.as_str())
                    .collect::<BTreeSet<_>>()
                    == SOURCES.iter().map(|(p, _)| *p).collect(),
            "product denominator",
        )?;
        for s in &i.product_sources {
            require(digest(source(&s.path)?) == s.sha256, "product drift")?;
        }
        require(
            i.external_sources.len() == EXTERNAL.len()
                && i.external_sources
                    .iter()
                    .map(|s| s.path.as_str())
                    .collect::<BTreeSet<_>>()
                    == EXTERNAL.iter().copied().collect(),
            "external denominator",
        )?;
        require(i.groups.len() == GROUPS.len(), "group denominator")?;
        for (g, expected) in i.groups.iter().zip(GROUPS) {
            require(
                format!(
                    "{}|{}|{}|{}|{}|{}|{}|{}",
                    g.id,
                    g.classification,
                    g.scalar,
                    g.source,
                    g.external,
                    g.symbol,
                    g.product.len(),
                    g.upstream.len()
                ) == *expected,
                "group type/identity",
            )?;
            for a in &g.product {
                anchor(source(&g.source)?, a)?;
            }
            require(
                g.upstream.iter().all(|s| !s.trim().is_empty()),
                "empty upstream anchors",
            )?;
        }
        require(i.outputs.len() == OUTPUTS.len(), "output denominator")?;
        for (o, expected) in i.outputs.iter().zip(OUTPUTS) {
            require(
                format!(
                    "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                    o.name,
                    o.scalar,
                    o.source,
                    o.declaration,
                    o.pointer,
                    o.write,
                    o.view,
                    o.external,
                    o.symbol,
                    o.upstream_write
                ) == *expected,
                "output route",
            )?;
            for a in [
                &o.pointer,
                &o.write,
                &format!("pub {}: &'a [f32]", o.name),
                &format!("{}: {},", o.name, o.view),
            ] {
                anchor(source(&o.source)?, a)?;
            }
        }
        boundaries()
    }
    fn boundaries() -> Check {
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
            stage
                .find("device.kernel.launch_flex_hessian(")
                .ok_or("matrix stage")?
                < stage
                    .find("device.validate_kernel.launch(")
                    .ok_or("validity stage")?,
            "publish order",
        )
    }
    type Texts = BTreeMap<String, String>;
    fn verify_external(i: &Inventory, texts: &Texts) -> Check {
        let get = |p: &str| texts.get(p).ok_or_else(|| format!("external: {p}"));
        for s in &i.external_sources {
            require(digest(get(&s.path)?) == s.sha256, "external drift")?;
        }
        super::super::verify_external(
            &super::super::inventory(),
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
    fn verify_generated(cuda: &str, ptx: &[u8]) -> Check {
        let pins: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/precision/g01-zero-quaternion.json"
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
    fn verify_captures(history: &Path, current: &Path) -> Result<usize, String> {
        let base: serde_json::Value =
            serde_json::from_str(include_str!("../../fixtures/precision/t4-kernels.json")).unwrap();
        let now: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/precision/t4-current-kernels.json"
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
            for a in &rows {
                require(
                    now["artifacts"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|r| *r == *a)
                        .count()
                        == 1,
                    "current lineage",
                )?;
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
                        let text = text.replace("\r\n", "\n");
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
    #[test]
    fn covers_twenty_one_groups_four_float_outputs_and_cache_publication() {
        validate(&inventory()).unwrap();
        assert_eq!((GROUPS.len(), OUTPUTS.len()), (21, 4));
    }
    #[test]
    fn rejects_group_and_output_precision_drift() {
        for n in 0..GROUPS.len() {
            for mutation in 0..4 {
                let mut i = inventory();
                match mutation {
                    0 => {
                        i.groups.remove(n);
                    }
                    1 => i.groups[n].scalar = "f64".into(),
                    2 => i.groups[n].classification = "equivalent".into(),
                    _ => i.groups[n].product[0] = "missing-expression".into(),
                }
                assert!(validate(&i).is_err(), "{n}/{mutation}");
            }
        }
        for n in 0..OUTPUTS.len() {
            for mutation in 0..3 {
                let mut i = inventory();
                match mutation {
                    0 => {
                        i.outputs.remove(n);
                    }
                    1 => i.outputs[n].scalar = "f64".into(),
                    _ => i.outputs[n].view = "&values[..]".into(),
                }
                assert!(validate(&i).is_err(), "{n}/{mutation}");
            }
        }
    }
    #[test]
    fn rejects_source_and_acceptance_boundary_drift() {
        for n in 0..SOURCES.len() {
            let mut i = inventory();
            i.product_sources[n].sha256 = "0".repeat(64);
            assert!(validate(&i).is_err());
        }
        for n in 0..5 {
            let mut i = inventory();
            match n {
                0 => i.np01_complete = true,
                1 => i.np02_complete = true,
                2 => i.equivalence_eligible = true,
                3 => i.remaining.clear(),
                _ => {
                    i.external_sources.pop();
                }
            }
            assert!(validate(&i).is_err());
        }
    }
    #[test]
    #[ignore = "reads frozen face/Hessian sources and cached artifacts; no GPU or Python"]
    fn audits_frozen_matrices_and_generated_face_artifacts() {
        let i = inventory();
        validate(&i).unwrap();
        let mut texts = Texts::new();
        for s in &i.external_sources {
            let (kind, path) = s.path.split_once('/').unwrap();
            let key = if kind == "upstream" {
                "MJWARP_FROZEN_UPSTREAM_ROOT"
            } else {
                "MJWARP_FROZEN_WARP_ROOT"
            };
            let root = std::env::var_os(key).expect(key);
            texts.insert(
                s.path.clone(),
                std::fs::read_to_string(Path::new(&root).join(path))
                    .unwrap()
                    .replace("\r\n", "\n"),
            );
        }
        verify_external(&i, &texts).unwrap();
        let mut changed = i.clone();
        changed.groups[0].upstream[0] = "missing-upstream".into();
        assert!(verify_external(&changed, &texts).is_err());
        let mut changed = texts.clone();
        changed
            .get_mut("upstream/_src/passive.py")
            .unwrap()
            .push('\n');
        assert!(verify_external(&i, &changed).is_err());
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let pins: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/precision/g01-zero-quaternion.json"
        ))
        .unwrap();
        let prefix = root.join(pins["artifact_prefix"].as_str().unwrap());
        // 缓存名含点号；追加后缀，不能替换已有扩展名。
        let cuda = std::fs::read_to_string(format!("{}.cu", prefix.display())).unwrap();
        let ptx = std::fs::read(format!("{}.sm89.ptx", prefix.display())).unwrap();
        verify_generated(&cuda, &ptx).unwrap();
        assert!(
            verify_generated(
                &cuda.replace("var_58 = wp::normalize(var_57);", "missing-normalize"),
                &ptx
            )
            .is_err()
        );
        let mut corrupt = ptx.clone();
        corrupt[0] ^= 1;
        assert!(verify_generated(&cuda, &corrupt).is_err());
        let history = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("historical captures");
        let current = std::env::var_os("MJWARP_T4_CURRENT_CAPTURE_ROOT").expect("current captures");
        let count = verify_captures(Path::new(&history), Path::new(&current)).unwrap();
        let report = serde_json::json!({
            "schema":1,"scope":i.scope,"product_revision":i.product_revision,"upstream_revision":i.upstream_revision,
            "map_sha256":digest(FIXTURE),"internal_groups":21,"float_groups":19,"private_flag_groups":1,
            "host_gate_groups":1,"float_outputs":4,"cache_publication_fields":1,
            "historical_capture_variants":6,"retained_current_variants":3,"checked_capture_sets":count,
            "upstream_face_generated_source_verified":true,"upstream_face_ptx_hash_verified":true,
            "upstream_cuda_sha256":pins["upstream_cuda_sha256"],"upstream_ptx_sha256":pins["upstream_ptx_sha256"],
            "upstream_hessian_generated_verified":false,"np01_complete":false,"np02_complete":false,
            "equivalence_eligible":false,"np03":"historical-not-rerun","np04":"not-measured",
            "gpu_executions":0,"python_executions":0,"recompilations":0,"remaining":i.remaining
        });
        println!("{report}");
        if let Some(path) = std::env::var_os("MJWARP_FLEX_MATRICES_AUDIT_REPORT") {
            std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
        }
    }
}

use super::{Check, Stamp, between, digest, require};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path};

const FIXTURE: &str = include_str!("../../fixtures/precision/g01-field-internal.json");
const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
const PHYSICS: &str = include_str!("../../src/physics/mod.rs");
const PARAMETERS: &str = include_str!("../../src/model/parameters.rs");
const CAMLIGHT: &str = include_str!("../../src/model/camlight.rs");
const SOURCES: &[(&str, &str)] = &[
    ("src/physics/resident.rs", RESIDENT),
    ("src/physics/mod.rs", PHYSICS),
    (
        "src/physics/attached.rs",
        include_str!("../../src/physics/attached.rs"),
    ),
    (
        "src/physics/camlight.rs",
        include_str!("../../src/physics/camlight.rs"),
    ),
    ("src/model/parameters.rs", PARAMETERS),
    ("src/model/camlight.rs", CAMLIGHT),
];
const REMAINING: [&str; 6] = [
    "other-packers-per-field",
    "integer-topology-and-derived-pack-fields",
    "other-kernel-locals-and-accumulators",
    "complete-output-mapping",
    "upstream-generated-kernels",
    "measured-performance",
];
// 人工审查固定对应关系。
// 不把不同顺序冒充逐算子等价。
const STEPS: &[(&str, &str, &str, &str)] = &[
    ("mass-init", "_subtree_com_init", "derived-cache", "float"),
    (
        "weighted-position",
        "_subtree_com_init",
        "same-scalar",
        "wp.vec3",
    ),
    ("mass-accumulate", "_subtree_div", "derived-cache", "float"),
    (
        "position-accumulate",
        "_subtree_com_acc",
        "different-order",
        "wp.vec3",
    ),
    ("normalize", "_subtree_div", "derived-cache", "wp.vec3"),
    ("inertia-offset", "_cinert", "same-scalar", "wp.vec3"),
    ("inertia-rotation", "_cinert", "different-order", "wp.mat33"),
    ("inertia-diagonal", "_cinert", "same-scalar", "vec10"),
    ("inertia-offdiagonal", "_cinert", "same-scalar", "vec10"),
    ("inertia-moments", "_cinert", "same-scalar", "vec10"),
    ("motion-offset", "_cdof", "same-scalar", "wp.vec3"),
    (
        "free-translation",
        "_cdof",
        "same-scalar",
        "wp.spatial_vector",
    ),
    ("motion-axis", "_cdof", "same-scalar", "wp.vec3"),
    ("slide-motion", "_cdof", "same-scalar", "wp.spatial_vector"),
    (
        "rotation-motion",
        "_cdof",
        "same-scalar",
        "wp.spatial_vector",
    ),
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
    group: String,
    variant: String,
    name: String,
    slot: usize,
    descriptor_scalar: String,
    parameter_scalar: String,
    width: String,
    reader: String,
    entry: String,
    upstream_declaration: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    symbol: String,
    classification: String,
    scalar: String,
    upstream_type: String,
    product: Vec<String>,
    upstream: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    product_revision: String,
    upstream_revision: String,
    product_sources: Vec<Stamp>,
    upstream_smooth: Stamp,
    fields: Vec<Field>,
    com_steps: Vec<Step>,
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

fn com_source() -> &'static str {
    between(PHYSICS, "const COM_POSITION_CUDA: &str = r#\"", "\"#;").unwrap()
}

fn kernel(entry: &str) -> Result<(&'static str, &'static str, &'static str), String> {
    match entry {
        "rigid_kinematics" => Ok((PHYSICS, "model", "desc")),
        "rigid_com_position" => Ok((com_source(), "p", "desc")),
        "attached_kinematics" => Ok((source("src/physics/attached.rs")?, "model", "desc")),
        "camlight_rigid" | "camlight_com" => Ok((source("src/physics/camlight.rs")?, "model", "d")),
        _ => Err("unreviewed reader entry".into()),
    }
}

fn reader_anchor(f: &Field) -> Result<String, String> {
    let (_, parameters, base) = kernel(&f.entry)?;
    let descriptor = if f.slot == 0 {
        base.into()
    } else {
        format!("{base}+{}", 2 * f.slot)
    };
    Ok(format!(
        "const float* {}=field_parameter({parameters},{descriptor},w,{});",
        f.reader, f.width
    ))
}

fn variants(text: &str, start: &str, prefix: &str) -> Result<Vec<String>, String> {
    Ok(between(text, start, "];")?
        .lines()
        .filter_map(|line| line.trim().strip_prefix(prefix))
        .map(|line| line.trim_end_matches(',').to_owned())
        .collect())
}

fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1 && i.scope == "g01-batched-parameters-and-com-internals",
        "scope",
    )?;
    require(
        i.product_revision == "dcb1fae771c45ca5df823d0b2517f04724f625b2"
            && i.upstream_revision == super::REVISION,
        "revision",
    )?;
    require(
        !i.np01_complete && i.remaining.iter().map(String::as_str).eq(REMAINING),
        "remaining work",
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
    require(
        i.upstream_smooth.path == "_src/smooth.py"
            && i.upstream_smooth.sha256
                == "6268f0e1b6cb4c9ec6bbd3aa29828dfb36224efb2a3a3e1f54e5f87b6623fdef",
        "smooth source",
    )?;
    let input = super::inventory();
    let mut expected = Vec::new();
    for (group, start) in [
        ("rigid", "const RIGID_PARAMETERS:"),
        ("com", "const COM_PARAMETERS:"),
        ("attached", "const ATTACHED_PARAMETERS:"),
    ] {
        for (slot, variant) in variants(RESIDENT, start, "KinematicsParameter::")?
            .into_iter()
            .enumerate()
        {
            expected.push((group, slot, variant));
        }
    }
    for (slot, variant) in variants(CAMLIGHT, "pub const ALL:", "Self::")?
        .into_iter()
        .enumerate()
    {
        expected.push(("camlight", slot, variant));
    }
    require(
        expected.len() == 23 && i.fields.len() == 23,
        "field coverage",
    )?;
    // 同时覆盖公开枚举，避免新增字段静默漏审。
    let public: BTreeSet<_> = mjwarp_rs::model::KinematicsParameter::ALL
        .into_iter()
        .map(|f| f.name())
        .chain(
            mjwarp_rs::model::CamLightParameter::ALL
                .into_iter()
                .map(|f| f.name()),
        )
        .collect();
    require(
        public.len() == 23
            && i.fields
                .iter()
                .map(|f| f.name.as_str())
                .collect::<BTreeSet<_>>()
                == public,
        "public field coverage",
    )?;
    for (f, (group, slot, variant)) in i.fields.iter().zip(expected) {
        require(
            f.group == group && f.slot == slot && f.variant == variant,
            "packing order",
        )?;
        require(
            f.descriptor_scalar == "i32" && f.parameter_scalar == "f32",
            "packed scalar",
        )?;
        let declaration = input
            .fields
            .iter()
            .find(|p| p.name == f.name)
            .ok_or("unknown input")?;
        require(
            f.upstream_declaration == declaration.upstream_declaration
                && super::upstream_scalar(&f.upstream_declaration)? == "f32",
            "upstream field scalar",
        )?;
        let host = if group == "camlight" {
            CAMLIGHT
        } else {
            PARAMETERS
        };
        require(
            host.contains(&format!("Self::{} => \"{}\",", f.variant, f.name)),
            "host field identity",
        )?;
        let shared = between(host, "pub(crate) fn shared(", "\n}")?;
        require(
            shared.lines().any(|l| {
                l.trim().starts_with(&format!("Self::{} => &", f.variant))
                    && l.trim().ends_with(&format!(".{},", f.name))
            }),
            "shared field source",
        )?;
        let entry = match group {
            "rigid" => "rigid_kinematics",
            "com" => "rigid_com_position",
            "attached" => "attached_kinematics",
            _ if matches!(slot, 2 | 7) => "camlight_com",
            _ => "camlight_rigid",
        };
        require(f.entry == entry && !f.reader.is_empty(), "reader routing")?;
        require(
            kernel(entry)?.0.contains(&reader_anchor(f)?),
            "descriptor reader",
        )?;
    }
    for (start, end) in [
        ("fn pack_model(", "// Descriptors hold"),
        ("fn pack_camlight(", "impl PackedModel"),
    ] {
        let pack = between(RESIDENT, start, end)?;
        for anchor in [
            "host_staging::<i32>(metadata_len)",
            "host_staging::<f32>(parameters_len.max(1))",
            "metadata[cursor] = offset as i32;",
            "metadata[cursor + 1] = parameters.batches(f) as i32;",
            "values[offset..offset + v.len()].copy_from_slice(v);",
            "offset += v.len();",
            "cursor += 2;",
        ] {
            require(pack.contains(anchor), "typed pack route")?;
        }
    }
    for anchor in [
        "parameters: session.upload(&self.parameters)?",
        "return p+desc[0]+(unsigned long long)(world % (unsigned)desc[1])*width;",
    ] {
        require(RESIDENT.contains(anchor), "upload and independent period")?;
    }
    validate_com(i, com_source())
}

fn validate_com(i: &Inventory, text: &str) -> Check {
    require(i.com_steps.len() == STEPS.len(), "COM step coverage")?;
    // 哈希绑定整个模板，局部锚点只提供可读映射。
    require(digest(text) == digest(com_source()), "COM template drift")?;
    for (step, &(id, symbol, classification, ty)) in i.com_steps.iter().zip(STEPS) {
        require(
            (
                step.id.as_str(),
                step.symbol.as_str(),
                step.classification.as_str(),
                step.upstream_type.as_str(),
            ) == (id, symbol, classification, ty)
                && step.scalar == "f32",
            "COM type or boundary",
        )?;
        require(
            !step.product.is_empty() && !step.upstream.is_empty(),
            "COM missing anchors",
        )?;
        for anchor in &step.product {
            require(
                !anchor.trim().is_empty() && text.contains(anchor),
                "COM product anchor",
            )?;
        }
    }
    Ok(())
}

fn verify_upstream(i: &Inventory, smooth: &str, types: &str, warp: &str) -> Check {
    require(
        digest(smooth) == i.upstream_smooth.sha256,
        "upstream smooth drift",
    )?;
    super::verify_external(&super::inventory(), types, warp)?;
    for anchor in [
        "class vec10f(wp.types.vector(length=10, dtype=float)):",
        "vec10 = vec10f",
    ] {
        require(types.contains(anchor), "inertia type")?;
    }
    for anchor in [
        "class spatial_vectorf(vector(length=6, dtype=float32)):",
        "spatial_vector = spatial_vectorf",
        "class mat33f(matrix(shape=(3, 3), dtype=float32)):",
        "mat33 = mat33f",
    ] {
        require(warp.contains(anchor), "internal scalar type")?;
    }
    for step in &i.com_steps {
        let body = smooth
            .split_once(&format!("def {}(", step.symbol))
            .ok_or("upstream symbol")?
            .1;
        let body = body.split("\n\n\n").next().ok_or("upstream body")?;
        for anchor in &step.upstream {
            require(
                !anchor.trim().is_empty() && body.contains(anchor),
                format!("upstream expression: {}", step.id),
            )?;
        }
    }
    Ok(())
}

fn verify_captures(i: &Inventory, root: &Path) -> Result<(usize, usize), String> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    let artifacts = manifest["artifacts"].as_array().ok_or("artifacts")?;
    let mut total = 0;
    let mut batched = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        let mut entries = BTreeSet::new();
        let mut fields = BTreeSet::new();
        for artifact in artifacts {
            let entry = artifact["entry"].as_str().ok_or("entry")?;
            if kernel(entry).is_err() {
                continue;
            }
            let ptx = artifact["ptx_sha256"].as_str().ok_or("PTX hash")?;
            require(
                ptx.len() == 64 && ptx.bytes().all(|b| b.is_ascii_hexdigit()),
                "capture path",
            )?;
            let stem = format!("{entry}-{ptx}");
            let mut cuda = String::new();
            for (ext, hash_key) in [
                ("cu", "source_sha256"),
                ("ptx", "ptx_sha256"),
                ("json", "receipt_sha256"),
            ] {
                let bytes = std::fs::read(root.join(mode).join(format!("{stem}.{ext}")))
                    .map_err(|e| e.to_string())?;
                let expected = artifact[hash_key].as_str().ok_or("artifact hash")?;
                require(
                    format!("{:x}", Sha256::digest(&bytes)) == expected,
                    "artifact drift",
                )?;
                if ext == "cu" {
                    cuda = std::str::from_utf8(&bytes)
                        .map_err(|e| e.to_string())?
                        .to_owned();
                }
                if ext == "json" {
                    let receipt: serde_json::Value =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    require(
                        receipt["entry"] == entry
                            && receipt["passed"] == true
                            && receipt["precisionGate"] == "f32-physics"
                            && receipt["fmad"] == true
                            && receipt["useFastMath"] == false,
                        "precision receipt",
                    )?;
                }
            }
            if entry == "rigid_com_position" {
                require(
                    cuda.replace("\r\n", "\n").contains(com_source()),
                    "captured COM template",
                )?;
            }
            // 只给真正启用批量宏的产物记字段覆盖。
            if cuda
                .lines()
                .any(|l| l.trim() == "#define MJWARP_FIELD_BATCHES")
            {
                for f in i.fields.iter().filter(|f| f.entry == entry) {
                    require(cuda.contains(&reader_anchor(f)?), "captured field reader")?;
                    fields.insert(f.name.as_str());
                }
                batched += 1;
            }
            entries.insert((entry, ptx));
            total += 1;
        }
        require(
            entries.len() == 10 && fields.len() == 23,
            "capture coverage",
        )?;
    }
    Ok((total, batched))
}

#[test]
fn maps_all_batched_fields_and_com_steps() {
    validate(&inventory()).unwrap();
}

#[test]
fn rejects_each_missing_or_misrouted_field() {
    for index in 0..23 {
        for mutation in 0..7 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.fields.remove(index);
                }
                1 => i.fields[index] = i.fields[(index + 1) % 23].clone(),
                2 => i.fields[index].parameter_scalar = "f64".into(),
                3 => i.fields[index].descriptor_scalar = "f32".into(),
                4 => i.fields[index].slot += 1,
                5 => i.fields[index].width = "1".into(),
                _ => i.fields[index].reader = "wrong_reader".into(),
            }
            assert!(validate(&i).is_err(), "field {index}, mutation {mutation}");
        }
    }
}

#[test]
fn rejects_each_missing_or_mistyped_internal_step() {
    for index in 0..STEPS.len() {
        for mutation in 0..6 {
            let mut i = inventory();
            match mutation {
                0 => {
                    i.com_steps.remove(index);
                }
                1 => i.com_steps[index] = i.com_steps[(index + 1) % STEPS.len()].clone(),
                2 => i.com_steps[index].scalar = "f64".into(),
                3 => i.com_steps[index].upstream_type = "wp.float64".into(),
                4 => i.com_steps[index].classification = "equivalent".into(),
                _ => i.com_steps[index].product.clear(),
            }
            assert!(
                validate_com(&i, com_source()).is_err(),
                "step {index}, mutation {mutation}"
            );
        }
    }
}

#[test]
fn rejects_source_drift_and_premature_completion() {
    for index in 0..SOURCES.len() {
        let mut i = inventory();
        i.product_sources[index].sha256 = "0".repeat(64);
        assert!(validate(&i).is_err());
    }
    let i = inventory();
    for (old, new) in [
        ("float v=0.0f;", "double v=0.0;"),
        ("sm[up]+=sm[b]", "sm[up]=sm[b]"),
        ("float t[9]", "double t[9]"),
        ("1.0f", "1.0"),
    ] {
        assert!(validate_com(&i, &com_source().replace(old, new)).is_err());
    }
    let mut i = inventory();
    i.np01_complete = true;
    assert!(validate(&i).is_err());
    i.np01_complete = false;
    i.remaining.clear();
    assert!(validate(&i).is_err());
}

#[test]
#[ignore = "reads frozen sources and existing captures without GPU execution"]
fn audit_field_internal_sources_and_captures() {
    let i = inventory();
    validate(&i).unwrap();
    let root = std::env::var_os("MJWARP_FROZEN_UPSTREAM_ROOT").expect("set upstream root");
    let smooth = std::fs::read_to_string(Path::new(&root).join("_src/smooth.py"))
        .unwrap()
        .replace("\r\n", "\n");
    let types = std::fs::read_to_string(Path::new(&root).join("_src/types.py")).unwrap();
    let warp_root = std::env::var_os("MJWARP_FROZEN_WARP_ROOT").expect("set Warp root");
    let warp = std::fs::read_to_string(Path::new(&warp_root).join("_src/types.py")).unwrap();
    verify_upstream(&i, &smooth, &types, &warp).unwrap();
    let mut changed = i.clone();
    changed.com_steps[0].upstream[0] = "body_mass: wp.array2d[wp.float64],".into();
    assert!(verify_upstream(&changed, &smooth, &types, &warp).is_err());
    assert!(verify_upstream(&i, &(smooth.clone() + "\n"), &types, &warp).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("set capture root");
    let (captures, batched) = verify_captures(&i, Path::new(&root)).unwrap();
    let report = serde_json::json!({
        "schema":1, "scope":i.scope, "inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision, "upstream_revision":i.upstream_revision,
        "batched_fields":i.fields.len(), "com_step_groups":i.com_steps.len(),
        "captured_variants":captures, "batched_variants":batched,
        "gpu_executions":0, "python_executions":0, "np01_complete":false,
        "np02_complete":false, "np03":"historical-unchanged", "np04":"not-measured",
        "remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_FIELD_INTERNAL_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
