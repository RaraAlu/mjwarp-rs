//! 追踪其余打包器的字段路由。
//! 私有表示不充当完整上游ABI。

use super::face_lineage;

/// 柔体新增切片；旧打包选择及其分母保持原样。
mod flex_internal {
    use super::super::{Check, Stamp, between, digest, require};
    use serde::Deserialize;
    use sha2::{Digest, Sha256};
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::Path,
    };

    const FIXTURE: &str = include_str!("../../docs/evidence/t4-g03-retest/flex-internal/map.json");
    const FLEX: &str = include_str!("../../src/physics/flex.rs");
    const EDGE: &str = include_str!("../../src/physics/flex_edge.rs");
    const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
    const MODEL: &str = include_str!("../../src/model/flex.rs");
    const TRANSFER: &str = include_str!("../../src/runtime/transfer.rs");
    const SOURCES: &[(&str, &str)] = &[
        ("src/physics/flex.rs", FLEX),
        ("src/physics/flex_edge.rs", EDGE),
        ("src/physics/resident.rs", RESIDENT),
        ("src/model/flex.rs", MODEL),
        ("src/runtime/transfer.rs", TRANSFER),
    ];
    const EXTERNAL: &[&str] = &[
        "upstream/_src/smooth.py",
        "upstream/_src/math.py",
        "upstream/_src/support.py",
        "upstream/_src/types.py",
        "warp/_src/types.py",
    ];
    const STEPS: &[&str] = &[
        "node-transform|same-scalar-order-unverified|f32|src/physics/flex.rs|upstream/_src/smooth.py|_flex_nodes|wp.vec3|3|3",
        "vertex-direct|same-scalar-order-unverified|f32|src/physics/flex.rs|upstream/_src/smooth.py|_flex_vertices|wp.vec3|1|2",
        "cell-lookup|f32-to-i32|f32|src/physics/flex.rs|upstream/_src/smooth.py|_flex_vertices|float|2|2",
        "local-clamp|finite-domain|f32|src/physics/flex.rs|upstream/_src/smooth.py|_flex_vertices|wp.vec3|2|2",
        "basis|same-scalar-order-unverified|f32|src/physics/flex.rs|upstream/_src/support.py|eval_basis_trilinear|float|1|1",
        "interpolate|same-loop-order|f32|src/physics/flex.rs|upstream/_src/smooth.py|_flex_vertices|wp.vec3|3|5",
        "invalidate|private-bool-adapter|i32|src/physics/flex.rs|upstream/_src/smooth.py|flex|bool|2|2",
        "edge-difference|same-scalar|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|wp.vec3|1|2",
        "edge-length|same-scalar-order-unverified|f32|src/physics/flex_edge.rs|upstream/_src/math.py|normalize_with_norm|float|1|1",
        "edge-normalize|zero-preserved|f32|src/physics/flex_edge.rs|upstream/_src/math.py|normalize_with_norm|wp.vec3|1|3",
        "edge-offset|same-scalar|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|wp.vec3|2|2",
        "point-velocity|same-scalar-order-unverified|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|wp.vec3|2|2",
        "jacobian|same-endpoint-order|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|float|3|4",
        "edge-velocity|same-row-order|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|float|2|3",
        "sparse-pool|extra-tail-clear|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|float|1|2",
        "edge-early-return|same-velocity-only|f32|src/physics/flex_edge.rs|upstream/_src/smooth.py|_flex_edges|float|1|3",
    ];
    const OUTPUTS: &[&str] = &[
        "flexnode_xpos|flexnode_xpos|f32|wp.vec3|src/physics/flex.rs|_flex_nodes|array(\"nworld\", \"nflexnode\", wp.vec3)|float* nx=result+4+(unsigned long long)w*(3ull*nn+3ull*nv);|flex_transform(nx+3*i,local,xp,xm,nodebody[i],row[8]!=0||zero);|&v[..3 * self.layout.nn]|flexnode_xpos_out[worldid, nodeid] = xpos",
        "flexvert_xpos|flexvert_xpos|f32|wp.vec3|src/physics/flex.rs|_flex_vertices|array(\"nworld\", \"nflexvert\", wp.vec3)|float* vx=nx+3*nn;|for(int d=0;d<3;++d) vx[3*v+d]=p[d];|&v[3 * self.layout.nn..]|flexvert_xpos_out[worldid, vertid] = result",
        "flexedge_length|flexedge_length|f32|float|src/physics/flex_edge.rs|_flex_edges|array(\"nworld\", \"nflexedge\", float)|float* len=out+4+(unsigned long long)w*(2ull*ne+nnz);|len[e]=norm;|&v[..ne]|flexedge_length_out[worldid, edgeid] = edge_length",
        "flexedge_velocity|flexedge_velocity|f32|float|src/physics/flex_edge.rs|_flex_edges|array(\"nworld\", \"nflexedge\", float)|float* vel=len+ne;|vel[e]+=j*qv[d];|&v[ne..2 * ne]|flexedge_velocity_out[worldid, edgeid] = vel",
        "flexedge_jacobian|flexedge_J|f32|float|src/physics/flex_edge.rs|_flex_edges|array(\"nworld\", \"nJfe\", float)|float* jac=vel+ne;|jac[adr+i]=j;|&v[2 * ne..]|flexedge_J_out[worldid, rowadr + i] = J_val",
        "flex_hessian_valid|flex_hessian_valid|bool|bool|src/physics/flex.rs|flex|array(\"nworld\", \"nflex\", bool)|int* flags=result+4+(unsigned long long)w*nf;|flags[f]=0;|&self.validity[self.layout.validity.world_elements(world)?]|d.flex_hessian_valid.zero_()",
    ];
    const REMAINING: &[&str] = &[
        "generated-operator-semantics",
        "sparse-unused-tail-side-effect",
        "face-hessian-and-other-internals",
        "full-pipeline-current-numerics",
        "measured-performance",
    ];
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Step {
        id: String,
        classification: String,
        scalar: String,
        source: String,
        external: String,
        symbol: String,
        upstream_type: String,
        product: Vec<String>,
        upstream: Vec<String>,
    }
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Output {
        name: String,
        upstream_name: String,
        scalar: String,
        upstream_type: String,
        source: String,
        symbol: String,
        declaration: String,
        pointer: String,
        write: String,
        view: String,
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
        steps: Vec<Step>,
        outputs: Vec<Output>,
        np01_complete: bool,
        np02_complete: bool,
        equivalence_eligible: bool,
        remaining: Vec<String>,
    }
    fn inventory() -> Inventory {
        serde_json::from_str(FIXTURE).unwrap()
    }
    fn source(path: &str) -> Result<&'static str, String> {
        SOURCES
            .iter()
            .find_map(|(p, t)| (*p == path).then_some(*t))
            .ok_or_else(|| path.into())
    }
    fn anchor(text: &str, needle: &str) -> Check {
        require(
            !needle.trim().is_empty() && text.contains(needle),
            format!("anchor: {needle}"),
        )
    }
    fn body<'a>(text: &'a str, symbol: &str) -> Result<&'a str, String> {
        text.split_once(&format!("def {symbol}("))
            .map(|(_, s)| s.split("\n\n\n").next().unwrap())
            .ok_or_else(|| symbol.into())
    }
    fn validate(i: &Inventory) -> Check {
        require(
            i.schema == 1
                && i.scope == "g01-flex-position-edge-internal-output"
                && i.product_revision == "e272b1474391c4b875ad89b70a7e865a33eec914"
                && i.upstream_revision == super::super::REVISION
                && !i.np01_complete
                && !i.np02_complete
                && !i.equivalence_eligible
                && i.remaining
                    .iter()
                    .map(String::as_str)
                    .eq(REMAINING.iter().copied()),
            "scope/acceptance",
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
            require(digest(source(&s.path)?) == s.sha256, "product source drift")?;
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
        require(i.steps.len() == STEPS.len(), "step denominator")?;
        for (s, expected) in i.steps.iter().zip(STEPS) {
            require(
                format!(
                    "{}|{}|{}|{}|{}|{}|{}|{}|{}",
                    s.id,
                    s.classification,
                    s.scalar,
                    s.source,
                    s.external,
                    s.symbol,
                    s.upstream_type,
                    s.product.len(),
                    s.upstream.len()
                ) == *expected,
                "step identity/type/boundary",
            )?;
            let text = source(&s.source)?;
            let kernel = between(
                text,
                if s.source.ends_with("flex.rs") {
                    "const FLEX_POSITION_CUDA: &str = r#\""
                } else {
                    "const FLEX_EDGE_CUDA: &str = r#\""
                },
                "\"#;",
            )?;
            for a in &s.product {
                anchor(kernel, a)?;
            }
            require(
                s.upstream.iter().all(|a| !a.trim().is_empty()),
                "empty upstream anchor",
            )?;
        }
        require(i.outputs.len() == OUTPUTS.len(), "output denominator")?;
        for (o, expected) in i.outputs.iter().zip(OUTPUTS) {
            require(
                format!(
                    "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                    o.name,
                    o.upstream_name,
                    o.scalar,
                    o.upstream_type,
                    o.source,
                    o.symbol,
                    o.declaration,
                    o.pointer,
                    o.write,
                    o.view,
                    o.upstream_write
                ) == *expected,
                "output identity/type/route",
            )?;
            let text = source(&o.source)?;
            for a in [
                &o.pointer,
                &o.write,
                &format!("pub {}: &'a [{}]", o.name, o.scalar),
                &format!("{}: {},", o.name, o.view),
            ] {
                anchor(text, a)?;
            }
            require(
                super::super::upstream_scalar(&o.declaration)? == o.scalar,
                "output scalar",
            )?;
        }
        boundaries()
    }
    fn boundaries() -> Check {
        for (text, anchors) in [
            (
                FLEX,
                &[
                    "pub(super) values: Vec<f32>",
                    "pub(super) validity: Vec<bool>",
                    "let v = &self.values[r.start + 4..r.end + 4];",
                ][..],
            ),
            (
                EDGE,
                &[
                    "pub(super) values: Vec<f32>",
                    "let v = &self.values[r.start + 4..r.end + 4];",
                    "const float* qv=qvel+4+(unsigned long long)w*nv;",
                    "while(b>0) {if(b==owner) return true;b=parent[b];}return false;",
                ][..],
            ),
            (
                RESIDENT,
                &[
                    "flex: Option<TransferBuffer<f32>>",
                    "flex_validity: Option<TransferBuffer<i32>>",
                    "values: read_output(out, layout.guarded, \"resident_flex_position_output\")?",
                    "\"resident_flex_edge_output\"",
                    "if value != 0 && value != 1",
                    "decoded.extend(values.iter().map(|&v| v == 1));",
                ][..],
            ),
            (TRANSFER, &["target.copy_from_slice(staging.values());"][..]),
            (
                MODEL,
                &[
                    "\"interpolated_vertex_has_sparse_row\"",
                    "\"endpoint_dof_union_mismatch\"",
                    "// Native nJfe can reserve unused tail slots when passive forces are off.",
                    "if !scaled.is_finite() || scaled < i32::MIN as f32 || scaled >= 2147483648.0",
                ][..],
            ),
        ] {
            for a in anchors {
                anchor(text, a)?;
            }
        }
        // 同一世界先失效，再算节点/顶点，最后计算边。
        let pos = RESIDENT.find("\"flex_invalidate\"").ok_or("invalidation")?;
        let nodes = RESIDENT[pos..]
            .find("\"flex_positions\"")
            .ok_or("positions")?
            + pos;
        require(RESIDENT[nodes..].contains("\"flex_edges\""), "stage order")?;
        Ok(())
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
        // 六维速度也须追到f32定义，不能只核对三维别名。
        for a in [
            "class spatial_vectorf(vector(length=6, dtype=float32)):",
            "spatial_vector = spatial_vectorf",
        ] {
            anchor(get("warp/_src/types.py")?, a)?;
        }
        for a in [
            "qvel_in: wp.array2d[float]",
            "cdof_in: wp.array2d[wp.spatial_vector]",
            "body_isdofancestor: wp.array2d[int]",
        ] {
            anchor(body(get("upstream/_src/smooth.py")?, "_flex_edges")?, a)?;
        }
        for s in &i.steps {
            let section = body(get(&s.external)?, &s.symbol)?;
            for a in &s.upstream {
                anchor(section, a)?;
            }
        }
        let smooth = get("upstream/_src/smooth.py")?;
        let data = get("upstream/_src/types.py")?
            .split_once("class Data:")
            .ok_or("Data")?
            .1;
        for o in &i.outputs {
            anchor(data, &format!("  {}: {}\n", o.upstream_name, o.declaration))?;
            let section = body(smooth, &o.symbol)?;
            anchor(section, &o.upstream_write)?;
            if o.scalar != "bool" {
                anchor(
                    section,
                    &format!("{}_out: wp.array2d[{}]", o.upstream_name, o.upstream_type),
                )?;
            }
        }
        for a in ["if i == 0:", "return 1.0 - s", "return s"] {
            anchor(body(get("upstream/_src/support.py")?, "_phi")?, a)?;
        }
        let edges = body(smooth, "_flex_edges")?;
        require(!edges.contains("zero_()"), "upstream pool-clear assumption")?;
        // 私有祖先遍历与上游预计算掩码分开记账。
        anchor(edges, "if body_isdofancestor[b1, dofid] != 0:")?;
        anchor(edges, "if body_isdofancestor[b2, dofid] != 0:")?;
        let flex = body(smooth, "flex")?;
        let invalidate = flex
            .find("d.flex_hessian_valid.zero_()")
            .ok_or("invalidate")?;
        let nodes = flex.find("_flex_nodes,").ok_or("nodes")?;
        let vertices = flex.find("_flex_vertices,").ok_or("vertices")?;
        let edges = flex.find("_flex_edges,").ok_or("edges")?;
        require(
            invalidate < nodes && nodes < vertices && vertices < edges,
            "upstream stage order",
        )
    }
    fn verify_captures(history: &Path, current: &Path) -> Result<usize, String> {
        let original: serde_json::Value =
            serde_json::from_str(include_str!("../../fixtures/precision/t4-kernels.json")).unwrap();
        let now: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/precision/t4-current-kernels.json"
        ))
        .unwrap();
        let entries = ["flex_invalidate", "flex_positions", "flex_edges"];
        let rows: Vec<_> = original["artifacts"]
            .as_array()
            .ok_or("artifacts")?
            .iter()
            .filter(|a| entries.contains(&a["entry"].as_str().unwrap_or("")))
            .collect();
        require(
            rows.len() == 3
                && rows
                    .iter()
                    .map(|a| a["entry"].as_str().unwrap())
                    .collect::<BTreeSet<_>>()
                    == entries.into_iter().collect(),
            "capture denominator",
        )?;
        let mut count = 0;
        for directory in [
            history.join("debug-ptx"),
            history.join("release-ptx"),
            current.to_path_buf(),
        ] {
            for row in &rows {
                require(
                    now["artifacts"]
                        .as_array()
                        .ok_or("current artifacts")?
                        .iter()
                        .filter(|a| *a == *row)
                        .count()
                        == 1,
                    "current lineage changed",
                )?;
                let entry = row["entry"].as_str().ok_or("entry")?;
                let hash = row["ptx_sha256"].as_str().ok_or("hash")?;
                require(
                    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
                    "path hash",
                )?;
                for (ext, key) in [
                    ("cu", "source_sha256"),
                    ("ptx", "ptx_sha256"),
                    ("json", "receipt_sha256"),
                ] {
                    let bytes = std::fs::read(directory.join(format!("{entry}-{hash}.{ext}")))
                        .map_err(|e| e.to_string())?;
                    require(
                        format!("{:x}", Sha256::digest(&bytes))
                            == row[key].as_str().ok_or("hash")?,
                        "artifact drift",
                    )?;
                    if ext == "cu" {
                        let text = String::from_utf8(bytes)
                            .map_err(|e| e.to_string())?
                            .replace("\r\n", "\n");
                        let (source, constant) = if entry == "flex_edges" {
                            (EDGE, "FLEX_EDGE_CUDA")
                        } else {
                            (FLEX, "FLEX_POSITION_CUDA")
                        };
                        let normalized = source.replace("\r\n", "\n");
                        require(
                            text == between(
                                &normalized,
                                &format!("const {constant}: &str = r#\""),
                                "\"#;",
                            )?,
                            "current product template",
                        )?;
                    } else if ext == "json" {
                        let r: serde_json::Value =
                            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                        require(
                            r["entry"] == entry
                                && r["passed"] == true
                                && r["fmad"] == true
                                && r["useFastMath"] == false
                                && r["precisionGate"] == "f32-physics",
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
    fn covers_sixteen_groups_and_six_output_routes() {
        validate(&inventory()).unwrap();
        assert_eq!((STEPS.len(), OUTPUTS.len()), (16, 6));
    }
    #[test]
    fn rejects_missing_retyped_and_reclassified_flex_groups() {
        for n in 0..STEPS.len() {
            for mutation in 0..5 {
                let mut i = inventory();
                match mutation {
                    0 => {
                        i.steps.remove(n);
                    }
                    1 => i.steps[n].scalar = "f64".into(),
                    2 => i.steps[n].classification = "equivalent".into(),
                    3 => {
                        i.steps[n].product.pop();
                    }
                    _ => i.steps[n].product[0] = "missing-product-expression".into(),
                }
                assert!(validate(&i).is_err(), "{n}/{mutation}");
            }
        }
    }
    #[test]
    fn rejects_output_retyping_and_private_abi_confusion() {
        for n in 0..OUTPUTS.len() {
            for mutation in 0..5 {
                let mut i = inventory();
                match mutation {
                    0 => {
                        i.outputs.remove(n);
                    }
                    1 => i.outputs[n].scalar = "f64".into(),
                    2 => i.outputs[n].view = "&v[..]".into(),
                    3 => i.outputs[n].upstream_name = "wrong-upstream-field".into(),
                    _ => i.outputs[n].pointer = "wrong-pointer".into(),
                }
                assert!(validate(&i).is_err(), "{n}/{mutation}");
            }
        }
    }
    #[test]
    fn rejects_drift_and_premature_flex_acceptance() {
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
    #[ignore = "reads frozen flex sources and existing captures; no Python, GPU or compilation"]
    fn audits_frozen_flex_sources_and_retained_artifacts() {
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
        changed.steps[0].upstream[0] = "wrong-upstream".into();
        assert!(verify_external(&changed, &texts).is_err());
        let mut changed = texts.clone();
        changed
            .get_mut("upstream/_src/support.py")
            .unwrap()
            .push('\n');
        assert!(verify_external(&i, &changed).is_err());
        let mut changed = texts.clone();
        let types = changed.get_mut("upstream/_src/types.py").unwrap();
        *types = types.replace(
            "flexedge_J: array(\"nworld\", \"nJfe\", float)",
            "flexedge_J: array(\"nworld\", \"nJfe\", wp.float64)",
        );
        assert!(verify_external(&i, &changed).is_err());
        let old = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("history");
        let current = std::env::var_os("MJWARP_T4_CURRENT_CAPTURE_ROOT").expect("current");
        let count = verify_captures(Path::new(&old), Path::new(&current)).unwrap();
        let report = serde_json::json!({
            "schema":1,"scope":i.scope,"product_revision":i.product_revision,
            "upstream_revision":i.upstream_revision,"map_sha256":digest(FIXTURE),
            "internal_groups":16,"f32_internal_groups":15,"private_i32_flag_groups":1,
            "output_fields":6,"f32_outputs":5,"bool_outputs":1,
            "historical_capture_variants":6,"retained_current_variants":3,"checked_capture_sets":count,
            "np01_complete":false,"np02_complete":false,"equivalence_eligible":false,
            "np03":"historical-not-rerun","np04":"not-measured","gpu_executions":0,"python_executions":0,
            "recompilations":0,"remaining":i.remaining
        });
        println!("{report}");
        if let Some(path) = std::env::var_os("MJWARP_FLEX_INTERNAL_AUDIT_REPORT") {
            std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
        }
    }
}

use super::{Check, Stamp, between, digest, require};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path};

const FIXTURE: &str = include_str!("../../fixtures/precision/g01-packing-routes.json");
const RESIDENT: &str = include_str!("../../src/physics/resident.rs");
macro_rules! sources {
    ($($path:literal),+ $(,)?) => {
        &[$(($path, include_str!(concat!("../../", $path)))),+]
    };
}
const SOURCES: &[(&str, &str)] = sources![
    "src/physics/resident.rs",
    "src/physics/fixed_tendon.rs",
    "src/physics/spatial_tendon.rs",
    "src/physics/tendon.rs",
    "src/physics/flex.rs",
    "src/physics/flex_edge.rs",
    "src/physics/flex_face.rs",
    "src/physics/flex_hessian.rs",
    "src/physics/sleep.rs",
    "src/model/flex.rs",
    "src/model/flex_hessian.rs",
    "src/model/sleep.rs",
    "src/model/fixed_tendon.rs",
    "src/model/spatial_tendon.rs",
    "src/model/tendon.rs",
];
// 独立冻结分母和表示类别。
// 新增路由须重新审查此表。
const PACKERS: &[(&str, &str, &str, &str)] = &[
    (
        "pack_flex_hessian",
        "flex_hessian",
        "src/physics/flex_hessian.rs",
        "header:header:,flex_dim:copy-i32:flex_dim,flex_vertadr:copy-i32:flex_vertadr,flex_vertnum:copy-i32:flex_vertnum,flex_edgeadr:copy-i32:flex_edgeadr,flex_edgenum:copy-i32:flex_edgenum,flex_elemadr:unused-i32:flex_elemadr,flex_elemnum:copy-i32:flex_elemnum,flex_elemdataadr:copy-i32:flex_elemdataadr,flex_elemedgeadr:copy-i32:flex_elemedgeadr,flex_stiffnessadr:copy-i32:flex_stiffnessadr,active:index-derived:flex_interp+flex_rigid+flex_dim+flex_stiffnessadr+flex_stiffness,flex_elem:copy-i32:flex_elem,flex_elemedge:copy-i32:flex_elemedge,flex_edge:copy-i32:flex_edge,flexedge_length0:copy-f32:flexedge_length0,flex_stiffness:copy-f32:flex_stiffness",
    ),
    (
        "pack_flex_faces",
        "flex_faces",
        "src/physics/flex_face.rs",
        "header:header:,normal-axis:index-derived:flex_face_map+flex_cellnum,flex_face:copy-i32:flex_face,unused:sentinel:",
    ),
    (
        "pack_sleep",
        "tendon_wake",
        "src/physics/sleep.rs",
        "header:header:,wake-gate:index-derived:,tendon_adr:copy-i32:tendon_adr,tendon_num:copy-i32:tendon_num,tendon_limited:bool-i32:tendon_limited,wrap_treeid:index-derived:body_treeid+wrap_type+wrap_objid,tendon_range:copy-f32:tendon_range,tendon_margin:copy-f32:tendon_margin,empty-padding:sentinel:",
    ),
    (
        "pack_flex_edges",
        "flex_edges",
        "src/physics/flex_edge.rs",
        "header:header:,vertex0:index-derived:flex_vertadr+flex_edge,vertex1:index-derived:flex_vertadr+flex_edge,flexedge_J_rowadr:copy-i32:flexedge_J_rowadr,flexedge_J_rownnz:copy-i32:flexedge_J_rownnz,flexedge_J_colind:copy-i32:flexedge_J_colind,flex_vertbodyid:copy-i32:flex_vertbodyid,body_parentid:copy-i32:body_parentid,body_rootid:index-derived:body_parentid+body_rootid,dof_bodyid:copy-i32:dof_bodyid",
    ),
    (
        "pack_flex",
        "flex_positions",
        "src/physics/flex.rs",
        "header:header:,flex_interp:copy-i32:flex_interp,flex_cellnum:copy-i32:flex_cellnum,flex_nodeadr:copy-i32:flex_nodeadr,flex_nodenum:copy-i32:flex_nodenum,flex_vertadr:copy-i32:flex_vertadr,flex_vertnum:copy-i32:flex_vertnum,flex_centered:bool-i32:flex_centered,flex_nodebodyid:copy-i32:flex_nodebodyid,flex_vertbodyid:copy-i32:flex_vertbodyid,flex_vert:copy-f32:flex_vert,flex_vert0:copy-f32:flex_vert0,flex_node:copy-f32:flex_node",
    ),
    (
        "pack_tendon",
        "tendon_values",
        "src/physics/tendon.rs",
        "header:header:,subset-kind:index-derived:,subset-local:index-derived:,subset-offset:index-derived:ten_J_rowadr,ten_J_rowadr:copy-i32:ten_J_rowadr,ten_J_rownnz:copy-i32:ten_J_rownnz,unused:sentinel:",
    ),
    (
        "pack_fixed_tendon",
        "fixed_tendon",
        "src/physics/fixed_tendon.rs",
        "header:header:,tendon_adr:copy-i32:tendon_adr,tendon_num:copy-i32:tendon_num,wrap_prm:copy-f32:wrap_prm,qpos-address:index-derived:wrap_objid+jnt_qposadr,jacobian-slot:index-derived:jnt_dofadr+ten_J_rowadr+ten_J_rownnz+ten_J_colind",
    ),
    (
        "pack_spatial_tendon",
        "spatial_site",
        "src/physics/spatial_tendon.rs",
        "header:header:,body_parentid:copy-i32:body_parentid,body_rootid:index-derived:body_parentid+body_rootid,body_dofadr:index-derived:body_jntadr+jnt_dofadr+body_dofadr,body_dofnum:index-derived:body_jntadr+body_jntnum+jnt_type+body_dofnum,tendon_adr:copy-i32:tendon_adr,tendon_num:copy-i32:tendon_num,ten_J_rowadr:copy-i32:ten_J_rowadr,ten_J_rownnz:copy-i32:ten_J_rownnz,wrap_type:copy-i32:wrap_type,wrap_objid:copy-i32:wrap_objid,wrap-body:index-derived:wrap_type+wrap_objid+site_bodyid+geom_bodyid,side-site:index-derived:wrap_type+wrap_prm,ten_J_colind:copy-i32:ten_J_colind,pulley-scale:f32-precompute:wrap_type+wrap_prm,geom_size:copy-f32:geom_size",
    ),
];
const REMAINING: [&str; 4] = [
    "rigid-camlight-integer-topology",
    "other-physics-internals-and-outputs",
    "upstream-generated-kernels",
    "measured-performance",
];
// 本库按肌腱串行累加。
// 上游按包裹对象原子累加。
const FIXED_INTERNAL: &[(&str, &str)] = &[
    ("float sum=0.0f;", "ten_length_out: wp.array2d[float],"),
    (
        "sum+=coefficients[i]*qpos[qadr[i]];",
        "L = prm * qpos_in[worldid, jnt_qposadr[wrapobjid]]",
    ),
    (
        "jac[jslot[i]]=coefficients[i];",
        "ten_J_out[worldid, rowadr + k] = prm",
    ),
    (
        "length[t]=sum;",
        "wp.atomic_add(ten_length_out[worldid], tenid, L)",
    ),
];

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Upstream {
    name: String,
    declaration: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    id: String,
    kind: String,
    scalar: String,
    upstream: Vec<Upstream>,
    write: String,
    read: String,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Packer {
    name: String,
    entry: String,
    file: String,
    routes: Vec<Route>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema: u32,
    scope: String,
    product_revision: String,
    upstream_revision: String,
    product_sources: Vec<Stamp>,
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
// 固定格式的源码守卫，不冒充语法解析器。
// 来源哈希同时保护完整函数。
fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn pack_body<'a>(resident: &'a str, name: &str) -> Result<&'a str, String> {
    between(resident, &format!("fn {name}("), "\n}")
}
fn cuda(path: &str) -> Result<&'static str, String> {
    between(source(path)?, ": &str = r#\"", "\"#;")
}
fn validate_reader(route: &Route, text: &str) -> Check {
    require(!route.read.is_empty(), "missing reader")?;
    if route.kind == "unused-i32" {
        require(
            route.id == "flex_elemadr"
                && route.read == "unused:r[6]"
                && !compact(text)
                    .match_indices("r[6]")
                    .any(|(index, _)| index == 0 || !text_before_access(text, index)),
            "unused metadata boundary",
        )
    } else {
        require(
            compact(text).contains(&compact(&route.read)),
            format!("reader: {}", route.id),
        )
    }
}

// 排除upper[6]等不同标识符。
fn text_before_access(text: &str, compact_index: usize) -> bool {
    let before = compact(text).as_bytes()[compact_index - 1];
    before.is_ascii_alphanumeric() || before == b'_'
}
fn validate_routes(p: &Packer, expected: &str, resident: &str, reader: &str) -> Check {
    let expected: Vec<_> = expected.split(',').collect();
    require(p.routes.len() == expected.len(), "route coverage")?;
    let body = compact(pack_body(resident, &p.name)?);
    for (r, expected) in p.routes.iter().zip(expected) {
        require(
            format!(
                "{}:{}:{}",
                r.id,
                r.kind,
                r.upstream
                    .iter()
                    .map(|u| u.name.as_str())
                    .collect::<Vec<_>>()
                    .join("+")
            ) == expected,
            "route identity or boundary",
        )?;
        let scalar = if matches!(r.kind.as_str(), "copy-f32" | "f32-precompute" | "sentinel") {
            "f32"
        } else {
            "i32"
        };
        require(r.scalar == scalar, "storage scalar")?;
        require(
            !r.write.trim().is_empty() && body.contains(&compact(&r.write)),
            format!("writer: {}", r.id),
        )?;
        validate_reader(r, reader)?;
        require(
            r.upstream
                .iter()
                .map(|u| u.name.as_str())
                .collect::<BTreeSet<_>>()
                .len()
                == r.upstream.len(),
            "duplicate upstream fields",
        )?;
        if matches!(
            r.kind.as_str(),
            "copy-f32" | "copy-i32" | "bool-i32" | "unused-i32"
        ) {
            require(
                r.upstream.len() == 1 && r.upstream[0].name == r.id,
                "direct field identity",
            )?;
        }
        for u in &r.upstream {
            let upstream_scalar = super::upstream_scalar(&u.declaration)?;
            if matches!(r.kind.as_str(), "copy-f32" | "copy-i32" | "unused-i32") {
                require(upstream_scalar == scalar, "direct scalar")?;
            } else if r.kind == "bool-i32" {
                // tendon_limited在上游采用整数。
                // flex_centered在上游采用布尔。
                require(
                    upstream_scalar
                        == if r.id == "tendon_limited" {
                            "i32"
                        } else {
                            "bool"
                        },
                    "boolean representation",
                )?;
            }
        }
    }
    Ok(())
}
fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1 && i.scope == "g01-remaining-eight-packing-routes",
        "scope",
    )?;
    require(
        i.product_revision == "9db581d509adbaf487cbe0498e0b7c2b9a5caba1"
            && i.upstream_revision == super::REVISION,
        "revision",
    )?;
    require(
        !i.np01_complete && i.remaining.iter().map(String::as_str).eq(REMAINING),
        "remaining work",
    )?;
    require(
        i.product_sources.len() == SOURCES.len()
            && i.product_sources
                .iter()
                .map(|s| s.path.as_str())
                .collect::<BTreeSet<_>>()
                == SOURCES.iter().map(|(p, _)| *p).collect(),
        "source coverage",
    )?;
    for stamp in &i.product_sources {
        let text = if stamp.path == "src/physics/flex_face.rs" {
            face_lineage::historical(source(&stamp.path)?)?
        } else {
            source(&stamp.path)?.to_owned()
        };
        require(
            digest(&text) == stamp.sha256,
            format!("source drift: {}", stamp.path),
        )?;
    }
    require(i.packers.len() == 8, "packer coverage")?;
    for (p, &(name, entry, file, routes)) in i.packers.iter().zip(PACKERS) {
        require(
            (p.name.as_str(), p.entry.as_str(), p.file.as_str()) == (name, entry, file),
            "packer identity",
        )?;
        validate_routes(p, routes, &RESIDENT.replace("\r\n", "\n"), cuda(file)?)?;
    }
    // 派生路线需绑定真正的类型来源。
    for (path, anchors) in [
        (
            "src/model/flex_hessian.rs",
            vec![
                "pub flexedge_length0: Vec<f32>",
                "pub flex_stiffness: Vec<f32>",
                "positions.flex_interp[f] == 0",
                "&& !self.flex_rigid[f]",
                "&& self.flex_dim[f] >= 2",
                "&& self.flex_stiffnessadr[f] >= 0",
                "&& self.flex_stiffness[self.flex_stiffnessadr[f] as usize] != 0.0",
            ],
        ),
        (
            "src/model/sleep.rs",
            vec![
                "pub tendon_limited: Vec<bool>",
                "pub tendon_range: ParameterBatch",
                "pub tendon_margin: ParameterBatch",
                "pub wrap_treeid: Vec<i32>",
                "fields.body_treeid[body as usize]",
            ],
        ),
        (
            "src/model/flex.rs",
            vec![
                "pub flex_centered: Vec<bool>",
                "pub flex_node: Vec<f32>",
                "pub flex_vert: Vec<f32>",
                "pub flex_vert0: Vec<f32>",
            ],
        ),
        (
            "src/model/spatial_tendon.rs",
            vec!["pub wrap_prm: Vec<f32>", "pub geom_size: ParameterBatch"],
        ),
        ("src/model/fixed_tendon.rs", vec!["pub wrap_prm: Vec<f32>"]),
    ] {
        for anchor in anchors {
            require(source(path)?.contains(anchor), "host representation")?;
        }
    }
    let spatial = compact(pack_body(
        &RESIDENT.replace("\r\n", "\n"),
        "pack_spatial_tendon",
    )?);
    for anchor in [
        "host_staging::<f32>(param_count.max(1))",
        "let mut scale = 1.0;",
        "scale = 1.0 / fields.wrap_prm[i];",
        "*value = scale;",
        "fields.wrap_prm[i].round() as i32",
    ] {
        require(spatial.contains(&compact(anchor)), "derived precision")?;
    }
    verify_fixed(cuda("src/physics/fixed_tendon.rs")?)
}
fn verify_fixed(text: &str) -> Check {
    require(
        digest(text) == digest(cuda("src/physics/fixed_tendon.rs")?),
        "fixed template drift",
    )?;
    for &(product, _) in FIXED_INTERNAL {
        require(text.contains(product), "fixed local or accumulator")?;
    }
    Ok(())
}
fn verify_upstream(i: &Inventory, types: &str, warp: &str, smooth: &str) -> Check {
    super::verify_external(&super::inventory(), types, warp)?;
    require(
        digest(smooth) == "6268f0e1b6cb4c9ec6bbd3aa29828dfb36224efb2a3a3e1f54e5f87b6623fdef",
        "upstream smooth drift",
    )?;
    let model = between(types, "class Model:", "class Contact:")?;
    for p in &i.packers {
        for r in &p.routes {
            for u in &r.upstream {
                let line = format!("  {}: {}", u.name, u.declaration);
                require(
                    model.lines().filter(|l| *l == line).count() == 1,
                    format!("upstream field: {}", u.name),
                )?;
            }
        }
    }
    let body = between(smooth, "def _joint_tendon(", "\n\n\n")?;
    for &(_, upstream) in FIXED_INTERNAL {
        require(body.contains(upstream), "upstream fixed expression")?;
    }
    require(
        body.contains("wrap_prm: wp.array[float],") && body.contains("qpos_in: wp.array2d[float],"),
        "fixed input scalars",
    )?;
    Ok(())
}
fn verify_captures(i: &Inventory, root: &Path) -> Result<usize, String> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    let artifacts = manifest["artifacts"]
        .as_array()
        .ok_or("capture inventory")?;
    let mut total = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        for p in &i.packers {
            let found: Vec<_> = artifacts.iter().filter(|a| a["entry"] == p.entry).collect();
            require(found.len() == 1, "missing or duplicate capture")?;
            let a = found[0];
            let ptx = a["ptx_sha256"].as_str().ok_or("PTX hash")?;
            require(
                ptx.len() == 64 && ptx.bytes().all(|b| b.is_ascii_hexdigit()),
                "capture path",
            )?;
            let mut text = String::new();
            for (ext, key) in [
                ("cu", "source_sha256"),
                ("ptx", "ptx_sha256"),
                ("json", "receipt_sha256"),
            ] {
                let path = root.join(mode).join(format!("{}-{ptx}.{ext}", p.entry));
                let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                require(
                    format!("{:x}", Sha256::digest(&bytes)) == a[key].as_str().ok_or("hash")?,
                    "artifact drift",
                )?;
                if ext == "cu" {
                    text = std::str::from_utf8(&bytes)
                        .map_err(|e| e.to_string())?
                        .replace("\r\n", "\n");
                }
                if ext == "json" {
                    let receipt: serde_json::Value =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    require(
                        receipt["entry"] == p.entry
                            && receipt["passed"] == true
                            && receipt["precisionGate"] == "f32-physics"
                            && receipt["fmad"] == true
                            && receipt["useFastMath"] == false,
                        "precision receipt",
                    )?;
                }
            }
            require(
                text.contains(&cuda(&p.file)?.replace("\r\n", "\n")),
                "captured template",
            )?;
            for r in &p.routes {
                validate_reader(r, &text)?;
            }
            total += 1;
        }
    }
    Ok(total)
}

#[test]
fn covers_eight_packers_and_all_82_routes() {
    validate(&inventory()).unwrap();
    assert_eq!(
        inventory()
            .packers
            .iter()
            .map(|p| p.routes.len())
            .sum::<usize>(),
        82
    );
}
#[test]
fn rejects_every_missing_duplicated_or_mistyped_route() {
    for (p, &(_, _, _, expected)) in inventory().packers.iter().zip(PACKERS) {
        for index in 0..p.routes.len() {
            for mutation in 0..8 {
                let mut changed = p.clone();
                match mutation {
                    0 => {
                        changed.routes.remove(index);
                    }
                    1 => changed.routes[index] = p.routes[(index + 1) % p.routes.len()].clone(),
                    2 => changed.routes[index].scalar = "f64".into(),
                    3 => changed.routes[index].kind = "equivalent".into(),
                    4 => changed.routes[index].write = "wrong_writer".into(),
                    5 => changed.routes[index].read = "wrong_reader".into(),
                    6 if !changed.routes[index].upstream.is_empty() => {
                        changed.routes[index].upstream.pop();
                    }
                    _ => changed.routes[index].upstream.push(Upstream {
                        name: "bad".into(),
                        declaration: "f64".into(),
                    }),
                }
                assert!(
                    validate_routes(
                        &changed,
                        expected,
                        &RESIDENT.replace("\r\n", "\n"),
                        cuda(&p.file).unwrap()
                    )
                    .is_err(),
                    "{} route {index} mutation {mutation}",
                    p.name
                );
            }
        }
    }
}
#[test]
fn rejects_source_coverage_and_false_completion() {
    for index in 0..8 {
        let mut i = inventory();
        i.packers.remove(index);
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
    i.np01_complete = false;
    i.remaining.clear();
    assert!(validate(&i).is_err());
}
#[test]
fn rejects_fixed_accumulator_and_unused_slot_drift() {
    let fixed = cuda("src/physics/fixed_tendon.rs").unwrap();
    for &(anchor, _) in FIXED_INTERNAL {
        assert!(verify_fixed(&fixed.replace(anchor, "changed_expression")).is_err());
    }
    assert!(verify_fixed(&fixed.replace("float sum", "double sum")).is_err());
    let i = inventory();
    let route = i.packers[0]
        .routes
        .iter()
        .find(|r| r.kind == "unused-i32")
        .unwrap();
    assert!(validate_reader(route, "int changed=r[6];").is_err());
}
#[test]
#[ignore = "reads frozen sources and existing GPU captures without executing them"]
fn audit_remaining_packing_sources_and_captures() {
    let i = inventory();
    validate(&i).unwrap();
    let root = std::env::var_os("MJWARP_FROZEN_UPSTREAM_ROOT").expect("set upstream root");
    let types = std::fs::read_to_string(Path::new(&root).join("_src/types.py")).unwrap();
    let smooth = std::fs::read_to_string(Path::new(&root).join("_src/smooth.py"))
        .unwrap()
        .replace("\r\n", "\n");
    let warp_root = std::env::var_os("MJWARP_FROZEN_WARP_ROOT").expect("set Warp root");
    let warp = std::fs::read_to_string(Path::new(&warp_root).join("_src/types.py")).unwrap();
    verify_upstream(&i, &types, &warp, &smooth).unwrap();
    let mut changed = i.clone();
    changed.packers[0].routes[1].upstream[0].declaration = "array(\"nflex\", float)".into();
    assert!(verify_upstream(&changed, &types, &warp, &smooth).is_err());
    assert!(verify_upstream(&i, &types, &warp, &(smooth.clone() + "\n")).is_err());
    let root = std::env::var_os("MJWARP_T4_EVIDENCE_ROOT").expect("set evidence root");
    let captures = verify_captures(&i, Path::new(&root)).unwrap();
    let fields: BTreeSet<_> = i
        .packers
        .iter()
        .flat_map(|p| p.routes.iter())
        .flat_map(|r| r.upstream.iter().map(|u| u.name.as_str()))
        .collect();
    let report = serde_json::json!({
        "schema":1, "scope":i.scope, "inventory_sha256":digest(FIXTURE),
        "product_revision":i.product_revision, "upstream_revision":i.upstream_revision,
        "packing_functions":8, "packing_routes":82, "upstream_fields":fields.len(),
        "fixed_internal_groups":4, "captured_variants":captures,
        "gpu_executions":0, "python_executions":0, "np01_complete":false, "np02_complete":false,
        "np03":"historical-unchanged", "np04":"not-measured", "remaining":i.remaining
    });
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_PACKING_ROUTES_AUDIT_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
