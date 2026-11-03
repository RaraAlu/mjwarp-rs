use super::*;

const PROTOTYPE_DIR: &str = "docs/development-evidence/render-bvh/";
const PROTOTYPE_RECIPE: &str =
    "sdf-validate-window-material-v2/prototype-lists/scene-a-sphere-outline-recipe.txt";

fn sample_list(rows: usize) -> String {
    let mut text = String::from(
        "# scene-a sphere-outline exact-diff list (criterion v2.1 proposal format)\n\
         # reference: fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json\n\
         # reference sha256: d6ab6dde\n\
         # camera: 1 raster: 240x180 spp: 1\n\
         # columns: pixel x y ref_seg_geom ref_seg_type ref_depth_bits formA_bits formB_bits ulp_ab\n",
    );
    for index in 0..rows {
        text.push_str(&format!(
            "{} 0 0 2 5 1 2 3 {} 10.0 11.5\n",
            index + 100,
            index as i64 + 1
        ));
    }
    text
}

fn list_sha(list_text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(list_text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn sample_json(
    rows: usize,
    exact_max: u64,
    bound: u64,
    list_text: &str,
    first_pixel: u64,
) -> String {
    serde_json::json!({
        "schema": 1,
        "criterion_version": "render-sdf-criterion-v2.2",
        "bound_ulp": bound,
        "camera": 1,
        "arms": [{
            "arm": "recipe",
            "entities": ["render_e2e"],
            "path": "scene-a-sphere-outline-recipe.txt",
            "bytes": list_text.len() as u64,
            "sha256": list_sha(list_text),
            "prototype_byte_identical": true,
            "bounds_sidecar": {"path": "outline-bounds-recipe.txt", "bytes": 1, "sha256": "00"},
            "rows": rows as u64,
            "first_pixel": first_pixel,
            "first_xy": [0, 0],
            "exact_max_ulp": exact_max,
            "spp": 1,
            "raster": "240x180",
            "sphere_pixels": 747,
            "reference": "fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json",
            "reference_sha256": "d6ab6dde"
        }]
    })
    .to_string()
}

fn dir() -> std::path::PathBuf {
    std::env::temp_dir()
}

#[test]
fn parses_prototype_list_with_json_metadata() {
    let text = sample_list(3);
    let json = sample_json(3, 3, 128, &text, 100);
    let parsed = parse_list_with_json(&text, &json, Arm::Recipe, &dir());
    assert_eq!(parsed.rows, 3);
    assert_eq!(parsed.exact_max_ulp, 3);
    assert_eq!(parsed.bound_ulp, 128);
    assert_eq!(parsed.entries.len(), 3);
    assert!(parsed.entries.contains_key(&100));
}

#[test]
#[should_panic(expected = "criterion_version is not")]
fn rejects_missing_json_metadata() {
    let text = sample_list(2);
    parse_list_with_json(&text, "{}", Arm::Recipe, &dir());
}

#[test]
#[should_panic(expected = "data rows")]
fn rejects_json_rows_mismatch() {
    let text = sample_list(3);
    let json = sample_json(2, 3, 128, &text, 100);
    parse_list_with_json(&text, &json, Arm::Recipe, &dir());
}

#[test]
#[should_panic(expected = "duplicate pixel indices")]
fn rejects_duplicate_pixel_rows() {
    let mut text = sample_list(2);
    text.push_str("100 0 0 2 5 1 2 3 1 10.0 11.5\n");
    let json = sample_json(3, 3, 128, &text, 100);
    parse_list_with_json(&text, &json, Arm::Recipe, &dir());
}

#[test]
#[should_panic(expected = "signed 128")]
fn rejects_wrong_bound() {
    let text = sample_list(2);
    let json = sample_json(2, 2, 512, &text, 100);
    parse_list_with_json(&text, &json, Arm::Recipe, &dir());
}

#[test]
#[should_panic(expected = "exceeds the signed bound")]
fn rejects_exact_max_over_bound() {
    let text = sample_list(2);
    let json = sample_json(2, 512, 128, &text, 100);
    parse_list_with_json(&text, &json, Arm::Recipe, &dir());
}

#[test]
fn ulp_distance_matches_f32_ordering() {
    assert_eq!(ulp_distance(1.0f32.to_bits(), 1.0f32.to_bits()), 0);
    assert_eq!(
        ulp_distance((1.0f32 + f32::EPSILON).to_bits(), 1.0f32.to_bits()),
        1
    );
    assert_eq!(ulp_distance((-1.0f32).to_bits(), (-1.0f32).to_bits()), 0);
    assert_eq!(
        ulp_distance((-1.0f32).to_bits(), (-0.999_999_94f32).to_bits()),
        1
    );
}

fn prototype_text() -> String {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let list_path = manifest.join(PROTOTYPE_DIR).join(PROTOTYPE_RECIPE);
    std::fs::read_to_string(&list_path).unwrap_or_else(|error| {
        panic!("prototype list missing at {}: {error}", list_path.display())
    })
}

/// Env-gated path: both variables set, list + metadata present in the list dir -> parse OK.
/// This is the only test in this module that touches the process environment.
#[test]
fn env_enabled_loads_prototype_list_and_json() {
    let text = prototype_text();
    let dir = std::env::temp_dir().join(format!("mjwarp-outline-env-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(dir.join(Arm::Recipe.list_file()), text.as_bytes()).expect("list");
    std::fs::write(
        dir.join(LISTS_JSON),
        sample_json(183, 73, 128, &text, 28969),
    )
    .expect("json");
    // SAFETY: this is the only test in this binary that mutates the process environment.
    unsafe {
        std::env::set_var(ENV_LIST_DIR, &dir);
        std::env::set_var(ENV_REQUIRE, "1");
    }
    assert!(enabled(), "both env vars set must enable the criterion");
    let parsed = load_from_dir(&dir, Arm::Recipe);
    // SAFETY: same single-owner contract as above.
    unsafe {
        std::env::remove_var(ENV_LIST_DIR);
        std::env::remove_var(ENV_REQUIRE);
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(parsed.rows, 183);
    assert_eq!(parsed.exact_max_ulp, 73);
    assert_eq!(parsed.entries.len(), 183);
}

/// 9-column list present but metadata JSON absent -> panic (fail-closed).
#[test]
#[should_panic(expected = "outline-lists.json is missing")]
fn env_enabled_missing_json_panics() {
    let text = prototype_text();
    let dir =
        std::env::temp_dir().join(format!("mjwarp-outline-env-missing-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(dir.join(Arm::Recipe.list_file()), text.as_bytes()).expect("list");
    let _parsed = load_from_dir(&dir, Arm::Recipe);
    let _ = std::fs::remove_dir_all(&dir);
}

/// CPU-only regression on the committed prototype list (signed 9-column format) with the
/// metadata JSON the authoring tool emits; proves the body-time parser accepts the real
/// list, which the synthetic fixtures alone could not guarantee.
#[test]
fn parses_committed_prototype_list_recipe() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let list_path = manifest.join(PROTOTYPE_DIR).join(PROTOTYPE_RECIPE);
    let text = std::fs::read_to_string(&list_path).unwrap_or_else(|error| {
        panic!("prototype list missing at {}: {error}", list_path.display())
    });
    let json = sample_json(183, 73, 128, &text, 28969);
    let parsed = parse_list_with_json(&text, &json, Arm::Recipe, &dir());
    assert_eq!(parsed.rows, 183);
    assert_eq!(parsed.exact_max_ulp, 73);
    assert_eq!(parsed.entries.len(), 183);
    assert!(parsed.entries.contains_key(&28969));
}

fn rs_sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = std::fs::read_dir(dir).expect("read tests source dir");
    for entry in entries {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rs_sources(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// `open` 指向 `(`；按括号配对返回该调用表达式的文本。
fn call_text(text: &str, open: usize) -> &str {
    let mut depth = 0usize;
    for (offset, ch) in text[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    &text[open..]
}

/// 测试支撑护栏：不经 criterion v2.2 清单豁免的裸 depth 比较（`diff_depth` 调用）只允许出现在
/// 白名单文件里；新增必须先在这里登记并写明理由，评审可见。带清单豁免的 helper 与
/// `depth_stats` 也不得传 `None`（arm 已改为必填 `Arm`，编译期兜底；本断言防回归到 `Option` 写法）。
#[test]
fn bare_depth_diff_call_sites_are_allowlisted() {
    // 拼接构造，避免扫描器命中本文件自身的字面量。
    let bare = ["diff", "_depth("].concat();
    let outlined = ["diff_depth", "_with_outline("].concat();
    let stats_fn = ["depth", "_stats("].concat();
    let allowlist = [
        "tests/render_kernel_depth_seg.rs", // 合成扰动 depth，判据是 mismatch 计数本身
        "tests/support/render_bvh/hfield_scene.rs", // hfield 参考无球面轮廓，位级比较
        "tests/support/render_bvh/outline.rs", // 豁免 helper 自身先算原始 diff
        "tests/support/render_bvh/r08.rs",  // 多世界逐相机 depth 位级比较
        "tests/support/render_bvh/r08_gpu.rs", // 同上（GPU 变体）
        "tests/support/render_bvh/s7b1.rs", // 泛化比较，随后 adjust_recipe_diff 调整
        "tests/support/render_bvh/sdf_compare.rs", // SDF 参考比较，无球面轮廓清单
    ];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rs_sources(&root.join("tests"), &mut files);
    files.sort();
    let mut bare_sites = Vec::new();
    for path in &files {
        let rel = path
            .strip_prefix(root)
            .expect("tests source under manifest dir")
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(path).expect("read test source");
        if text.contains(bare.as_str()) {
            bare_sites.push(rel.clone());
        }
        for needle in [outlined.as_str(), stats_fn.as_str()] {
            let mut from = 0usize;
            while let Some(index) = text[from..].find(needle) {
                let open = from + index + needle.len() - 1;
                let call = call_text(&text, open);
                assert!(
                    !call.contains("None"),
                    "{rel}: `{needle}` 调用传了 None（arm 必填）: {call}"
                );
                from = open + 1;
            }
        }
    }
    bare_sites.sort();
    assert_eq!(
        bare_sites, allowlist,
        "裸 depth 比较的白名单不符：新增裸比较必须先在这里登记并写明理由"
    );
}
