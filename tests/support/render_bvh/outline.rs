//! Env-gated scene-a sphere-outline exemption support (criterion render-sdf-criterion-v2.2).
//!
//! Semantics:
//!   - disabled (neither env var set): callers keep the raw product comparison, so plain
//!     `cargo test` / CI behaviour is unchanged;
//!   - enabled (`MJWARP_RENDER_OUTLINE_LIST_DIR` + `MJWARP_REQUIRE_OUTLINE_CRITERION=1`):
//!     - camera 0 stays bitwise (it has 0 sphere pixels in scene-a);
//!     - camera 1: pixels outside the frozen list must be bitwise equal; inside the list
//!       `|ULP| <= 128`; the list file must exist, its `# rows` header must equal the
//!       data-row count, `# bound_ulp` must be 128 and `# exact_max_ulp` must be <= 128;
//!     - any violation panics (fail-closed) instead of silently relaxing;
//!   - setting only one of the two env vars panics (misconfiguration).
//!
//! The frozen lists are produced by `scripts/authoring/author-scene-a-sphere-outline.ps1`
//! (`scene-a-sphere-outline-recipe.txt` / `...-aa2.txt`); the window runner exports the env
//! vars and the expected list sha256 (verified by the runner itself).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use mjwarp_rs::render::kernel::output::{DepthDiff, RefCamera, diff_depth};

pub const ENV_LIST_DIR: &str = "MJWARP_RENDER_OUTLINE_LIST_DIR";
pub const ENV_REQUIRE: &str = "MJWARP_REQUIRE_OUTLINE_CRITERION";
pub const BOUND_ULP: i64 = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    Recipe,
    Aa2,
}

impl Arm {
    pub fn list_file(self) -> &'static str {
        match self {
            Arm::Recipe => "scene-a-sphere-outline-recipe.txt",
            Arm::Aa2 => "scene-a-sphere-outline-aa2.txt",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Arm::Recipe => "recipe",
            Arm::Aa2 => "aa2",
        }
    }
}

/// True when the env-gated criterion is required; false when the window is not running.
pub fn enabled() -> bool {
    let dir = std::env::var(ENV_LIST_DIR)
        .ok()
        .filter(|value| !value.is_empty());
    let require = std::env::var(ENV_REQUIRE)
        .ok()
        .filter(|value| !value.is_empty());
    match (dir, require) {
        (Some(_), Some(value)) if value == "1" => true,
        (None, None) => false,
        (Some(_), Some(value)) => panic!(
            "{ENV_REQUIRE} must be 1 when {ENV_LIST_DIR} is set (got {value:?}); refusing to relax"
        ),
        _ => panic!(
            "outline criterion misconfiguration: set both {ENV_LIST_DIR} and {ENV_REQUIRE}=1, or neither"
        ),
    }
}

fn list_dir() -> PathBuf {
    PathBuf::from(std::env::var(ENV_LIST_DIR).expect("checked by enabled()"))
}

fn ulp_distance(left: u32, right: u32) -> i64 {
    let ordered = |bits: u32| -> i64 {
        if bits & 0x8000_0000 == 0 {
            i64::from(bits)
        } else {
            -i64::from(bits & 0x7fff_ffff)
        }
    };
    (ordered(left) - ordered(right)).abs()
}

#[derive(Debug)]
pub struct OutlineList {
    pub arm: Arm,
    pub path: PathBuf,
    pub rows: usize,
    pub exact_max_ulp: i64,
    pub bound_ulp: i64,
    /// pixel index -> frozen `ulp_ab` value
    pub entries: BTreeMap<usize, i64>,
}

/// Parse the 9-column data rows of one frozen list (signed prototype format).
///
/// The prototype list carries no metadata header; rows / exact_max_ulp / bound_ulp come from
/// the machine-readable `outline-lists.json` written by the authoring tool next to the lists
/// (single source of truth for freeze, runner and body-time support).
fn parse_rows(text: &str, path: &std::path::Path) -> (BTreeMap<usize, i64>, usize, usize) {
    let mut entries = BTreeMap::new();
    let mut data_rows = 0usize;
    let mut max_pixel_seen = 0usize;
    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        assert!(
            fields.len() >= 9,
            "{} line {}: expected >= 9 columns, got {}",
            path.display(),
            line_no + 1,
            fields.len()
        );
        let pixel: usize = fields[0]
            .parse()
            .unwrap_or_else(|_| panic!("{}: bad pixel index", path.display()));
        let ulp: i64 = fields[8]
            .parse()
            .unwrap_or_else(|_| panic!("{}: bad ulp_ab", path.display()));
        entries.insert(pixel, ulp);
        max_pixel_seen = max_pixel_seen.max(pixel);
        data_rows += 1;
    }
    (entries, data_rows, max_pixel_seen)
}

/// Name of the machine-readable metadata file written next to the lists.
pub const LISTS_JSON: &str = "outline-lists.json";

fn json_u64(doc: &serde_json::Value, key: &str, path: &std::path::Path) -> u64 {
    doc.get(key)
        .and_then(|value| value.as_u64())
        .unwrap_or_else(|| panic!("{}: missing/invalid `{key}`", path.display()))
}

/// Parse one frozen list together with its metadata JSON; panics (fail-closed) on any
/// inconsistency: missing JSON, arm entry missing, rows != data rows, duplicate pixels,
/// exact max > 128, bound != 128, list bytes/sha or first pixel mismatch.
pub fn parse_list_with_json(
    list_text: &str,
    json_text: &str,
    arm: Arm,
    dir: &std::path::Path,
) -> OutlineList {
    let path = dir.join(arm.list_file());
    let json_path = dir.join(LISTS_JSON);
    let (entries, data_rows, max_pixel_seen) = parse_rows(list_text, &path);
    let list_bytes = list_text.len();
    let list_sha = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(list_text.as_bytes());
        format!("{:x}", hasher.finalize())
    };
    let doc: serde_json::Value = serde_json::from_str(json_text)
        .unwrap_or_else(|error| panic!("{}: invalid JSON: {error}", json_path.display()));
    assert_eq!(
        doc.get("criterion_version").and_then(|v| v.as_str()),
        Some("render-sdf-criterion-v2.2"),
        "{}: criterion_version is not render-sdf-criterion-v2.2",
        json_path.display()
    );
    let bound = json_u64(&doc, "bound_ulp", &json_path);
    assert_eq!(
        bound,
        BOUND_ULP as u64,
        "{}: bound_ulp {bound} is not the signed {BOUND_ULP}",
        json_path.display()
    );
    let arms = doc
        .get("arms")
        .and_then(|value| value.as_array())
        .unwrap_or_else(|| panic!("{}: missing `arms`", json_path.display()));
    let entry = arms
        .iter()
        .find(|entry| entry.get("arm").and_then(|v| v.as_str()) == Some(arm.label()))
        .unwrap_or_else(|| panic!("{}: no entry for arm {}", json_path.display(), arm.label()));
    let rows = json_u64(entry, "rows", &json_path) as usize;
    let exact_max = json_u64(entry, "exact_max_ulp", &json_path) as i64;
    let first_pixel = json_u64(entry, "first_pixel", &json_path) as usize;
    assert_eq!(
        rows,
        data_rows,
        "{}: arm {} rows {rows} != {data_rows} data rows (fail-closed)",
        json_path.display(),
        arm.label()
    );
    assert_eq!(
        entries.len(),
        data_rows,
        "{}: {data_rows} data rows contain duplicate pixel indices ({} unique, fail-closed)",
        json_path.display(),
        entries.len()
    );
    assert!(
        exact_max <= BOUND_ULP,
        "{}: arm {} exact_max_ulp {exact_max} exceeds the signed bound {BOUND_ULP}",
        json_path.display(),
        arm.label()
    );
    assert_eq!(
        entry.get("sha256").and_then(|v| v.as_str()),
        Some(list_sha.as_str()),
        "{}: arm {} list sha256 mismatch (list bytes changed vs the freeze-time record)",
        json_path.display(),
        arm.label()
    );
    assert_eq!(
        json_u64(entry, "bytes", &json_path) as usize,
        list_bytes,
        "{}: arm {} list bytes mismatch",
        json_path.display(),
        arm.label()
    );
    assert_eq!(
        first_pixel,
        entries.keys().next().copied().unwrap_or(usize::MAX),
        "{}: arm {} first_pixel mismatch",
        json_path.display(),
        arm.label()
    );
    assert!(
        max_pixel_seen < 76_800,
        "{}: pixel index {max_pixel_seen} outside the scene-a ray range",
        path.display()
    );
    OutlineList {
        arm,
        path,
        rows,
        exact_max_ulp: exact_max,
        bound_ulp: bound as i64,
        entries,
    }
}

/// Load one arm's frozen list + metadata from a directory (env-gated callers use `list_dir()`).
pub fn load_from_dir(dir: &std::path::Path, arm: Arm) -> OutlineList {
    let list_path = dir.join(arm.list_file());
    let json_path = dir.join(LISTS_JSON);
    let list_text = std::fs::read_to_string(&list_path).unwrap_or_else(|error| {
        panic!(
            "outline criterion is enabled but the {} list is missing/unreadable at {}: {error}",
            arm.label(),
            list_path.display()
        )
    });
    let json_text = std::fs::read_to_string(&json_path).unwrap_or_else(|error| {
        panic!(
            "outline criterion is enabled but {} is missing/unreadable at {}: {error}",
            LISTS_JSON,
            json_path.display()
        )
    });
    parse_list_with_json(&list_text, &json_text, arm, dir)
}

fn load_uncached(arm: Arm) -> OutlineList {
    load_from_dir(&list_dir(), arm)
}

pub fn list(arm: Arm) -> &'static OutlineList {
    static RECIPE: OnceLock<OutlineList> = OnceLock::new();
    static AA2: OnceLock<OutlineList> = OnceLock::new();
    match arm {
        Arm::Recipe => RECIPE.get_or_init(|| load_uncached(Arm::Recipe)),
        Arm::Aa2 => AA2.get_or_init(|| load_uncached(Arm::Aa2)),
    }
}

#[derive(Clone, Debug, Default)]
pub struct OutlineStats {
    pub list_pixels: usize,
    pub outside_mismatch: usize,
    pub in_list_differing: usize,
    pub in_list_max_ulp: i64,
    pub violations: usize,
}

impl OutlineStats {
    pub fn in_list_over_bound(&self) -> usize {
        self.violations.saturating_sub(self.outside_mismatch)
    }
}

/// Criterion-v2.2 depth comparison for one camera. `None` arm or disabled env => `None`
/// (caller keeps the raw comparison).
pub fn depth_stats(
    actual: &[f32],
    expected: &[f32],
    camera_index: usize,
    arm: Option<Arm>,
) -> Option<OutlineStats> {
    if !enabled() {
        return None;
    }
    let arm = arm?;
    assert_eq!(
        actual.len(),
        expected.len(),
        "outline: depth length mismatch"
    );
    if camera_index != 1 {
        let outside = actual
            .iter()
            .zip(expected)
            .filter(|(left, right)| left.to_bits() != right.to_bits())
            .count();
        return Some(OutlineStats {
            outside_mismatch: outside,
            violations: outside,
            ..OutlineStats::default()
        });
    }
    let frozen = list(arm);
    let mut stats = OutlineStats {
        list_pixels: frozen.rows,
        ..OutlineStats::default()
    };
    for (pixel, (left, right)) in actual.iter().zip(expected).enumerate() {
        if left.to_bits() == right.to_bits() {
            continue;
        }
        let ulp = ulp_distance(left.to_bits(), right.to_bits());
        match frozen.entries.get(&pixel) {
            Some(_) => {
                stats.in_list_differing += 1;
                stats.in_list_max_ulp = stats.in_list_max_ulp.max(ulp);
                if ulp > BOUND_ULP {
                    stats.violations += 1;
                }
            }
            None => {
                stats.outside_mismatch += 1;
                stats.violations += 1;
            }
        }
    }
    println!(
        "outline[{}] cam{}: list_rows={} outside_mismatch={} in_list_differing={} \
         in_list_max_ulp={} violations={} exact_max_ulp={} list={}",
        arm.label(),
        camera_index,
        frozen.rows,
        stats.outside_mismatch,
        stats.in_list_differing,
        stats.in_list_max_ulp,
        stats.violations,
        frozen.exact_max_ulp,
        frozen.path.display()
    );
    Some(stats)
}

/// Drop-in for `diff_depth` applying the env-gated v2.2 exemption. `None` arm keeps the raw
/// comparison. Panics on any violation; on success the returned numbers express the criterion
/// (outside-list mismatches, in-list values logged separately).
pub fn diff_depth_with_outline(actual: &[f32], camera: &RefCamera, arm: Option<Arm>) -> DepthDiff {
    let raw = diff_depth(actual, camera).expect("depth diff");
    if !enabled() || arm.is_none() {
        return raw;
    }
    let arm = arm.expect("checked");
    let expected = camera.depth_values();
    let Some(stats) = depth_stats(actual, &expected, camera.index, Some(arm)) else {
        return raw;
    };
    assert_eq!(
        stats.violations,
        0,
        "criterion v2.2 outline violation on cam{} (arm {}): outside_list_mismatch={} \
         in_list_over_bound={} in_list_max_ulp={} (list {} px, exact max {}): {}",
        camera.index,
        arm.label(),
        stats.outside_mismatch,
        stats.in_list_over_bound(),
        stats.in_list_max_ulp,
        stats.list_pixels,
        list(arm).exact_max_ulp,
        list(arm).path.display()
    );
    let mut adjusted = raw;
    adjusted.mismatched = stats.outside_mismatch;
    adjusted.beyond_ulp = 0;
    adjusted.max_ulp = 0;
    adjusted
}

/// Recipe-arm comparison for a single camera (used by rgb/r10/e2e/skybox call sites).
pub fn recipe_depth_diff(actual: &[f32], camera: &RefCamera) -> DepthDiff {
    diff_depth_with_outline(actual, camera, Some(Arm::Recipe))
}

/// Variant-aware comparison for `render_kernel_flags` (criterion v2.2 clause 1).
///
/// Every flag variant compares against its own same-recipe (`use_fast_math=False`) camera-1
/// reference, whose sphere-outline depth bits are bitwise identical to the recipe reference
/// (verified offline: culling_off 747/747 sphere pixels identical), so all variants consume the
/// recipe list; the aa2 arm keeps the aa2 list (spp=2, sub-sample 0). The active fail-closed rule
/// is the same as the recipe arm: outside-list bitwise, in-list `|ULP| <= 128`.
pub fn variant_depth_diff(
    actual: &[f32],
    camera: &RefCamera,
    variant: &str,
    aa_variant: &str,
) -> DepthDiff {
    let arm = if variant == aa_variant {
        Arm::Aa2
    } else {
        Arm::Recipe
    };
    diff_depth_with_outline(actual, camera, Some(arm))
}

/// Adjust a raw `diff_depth` result for the recipe comparison (used by
/// `s7b1::compare_reference`). Returns true when the exemption was applied; panics on violation.
pub fn adjust_recipe_diff(
    diff: &mut DepthDiff,
    actual: &[f32],
    expected: &[f32],
    camera_index: usize,
) -> bool {
    let Some(stats) = depth_stats(actual, expected, camera_index, Some(Arm::Recipe)) else {
        return false;
    };
    assert_eq!(
        stats.violations,
        0,
        "criterion v2.2 outline violation on cam{} (arm recipe): outside_list_mismatch={} \
         in_list_over_bound={} in_list_max_ulp={} (list {} px, exact max {})",
        camera_index,
        stats.outside_mismatch,
        stats.in_list_over_bound(),
        stats.in_list_max_ulp,
        stats.list_pixels,
        list(Arm::Recipe).exact_max_ulp
    );
    diff.mismatched = stats.outside_mismatch;
    diff.beyond_ulp = 0;
    diff.max_ulp = 0;
    true
}

#[cfg(test)]
#[path = "outline_tests.rs"]
mod tests;
