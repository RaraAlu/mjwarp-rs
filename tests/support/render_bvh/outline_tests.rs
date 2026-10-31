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
        "criterion_version": "render-sdf-criterion-v2.1",
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
