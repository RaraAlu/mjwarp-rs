//! Env-gated scene-a sphere-outline exemption support (criterion render-sdf-criterion-v2.1).
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

/// Parse one frozen list; panics (fail-closed) on any inconsistency.
pub fn parse_list(text: &str, arm: Arm, path: &std::path::Path) -> OutlineList {
    let mut header_rows: Option<usize> = None;
    let mut exact_max: Option<i64> = None;
    let mut bound: Option<i64> = None;
    let mut entries = BTreeMap::new();
    let mut data_rows = 0usize;
    let mut max_pixel_seen = 0usize;
    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('#') {
            let rest = rest.trim();
            if let Some(value) = rest.strip_prefix("rows:") {
                header_rows = Some(
                    value
                        .trim()
                        .parse()
                        .unwrap_or_else(|_| panic!("{}: bad `# rows` header", path.display())),
                );
            } else if let Some(value) = rest.strip_prefix("exact_max_ulp:") {
                exact_max = Some(
                    value
                        .trim()
                        .parse()
                        .unwrap_or_else(|_| panic!("{}: bad `# exact_max_ulp`", path.display())),
                );
            } else if let Some(value) = rest.strip_prefix("bound_ulp:") {
                let digits: String = value
                    .trim()
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                bound = Some(
                    digits
                        .parse()
                        .unwrap_or_else(|_| panic!("{}: bad `# bound_ulp`", path.display())),
                );
            }
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
    let rows =
        header_rows.unwrap_or_else(|| panic!("{}: missing `# rows:` header", path.display()));
    let exact_max = exact_max
        .unwrap_or_else(|| panic!("{}: missing `# exact_max_ulp:` header", path.display()));
    let bound =
        bound.unwrap_or_else(|| panic!("{}: missing `# bound_ulp:` header", path.display()));
    assert_eq!(
        rows,
        data_rows,
        "{}: `# rows: {rows}` does not match {data_rows} data rows (fail-closed)",
        path.display()
    );
    assert_eq!(
        entries.len(),
        data_rows,
        "{}: {data_rows} data rows contain duplicate pixel indices ({} unique, fail-closed)",
        path.display(),
        entries.len()
    );
    assert_eq!(
        bound,
        BOUND_ULP,
        "{}: `# bound_ulp: {bound}` is not the signed {BOUND_ULP}",
        path.display()
    );
    assert!(
        exact_max <= BOUND_ULP,
        "{}: exact_max_ulp {exact_max} exceeds the signed bound {BOUND_ULP}",
        path.display()
    );
    assert!(
        max_pixel_seen < 76_800,
        "{}: pixel index {max_pixel_seen} outside the scene-a ray range",
        path.display()
    );
    OutlineList {
        arm,
        path: path.to_path_buf(),
        rows,
        exact_max_ulp: exact_max,
        bound_ulp: bound,
        entries,
    }
}

fn load_uncached(arm: Arm) -> OutlineList {
    let path = list_dir().join(arm.list_file());
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "outline criterion is enabled but the {} list is missing/unreadable at {}: {error}",
            arm.label(),
            path.display()
        )
    });
    parse_list(&text, arm, &path)
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

/// Criterion-v2.1 depth comparison for one camera. `None` arm or disabled env => `None`
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

/// Drop-in for `diff_depth` applying the env-gated v2.1 exemption. `None` arm keeps the raw
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
        "criterion v2.1 outline violation on cam{} (arm {}): outside_list_mismatch={} \
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

/// Variant-aware comparison for `render_kernel_flags`: the aa2 variant consumes the aa2 list,
/// every other variant keeps the raw comparison.
pub fn variant_depth_diff(
    actual: &[f32],
    camera: &RefCamera,
    variant: &str,
    aa_variant: &str,
) -> DepthDiff {
    let arm = if variant == aa_variant {
        Some(Arm::Aa2)
    } else {
        None
    };
    diff_depth_with_outline(actual, camera, arm)
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
        "criterion v2.1 outline violation on cam{} (arm recipe): outside_list_mismatch={} \
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
mod tests {
    use super::*;

    const HEADER: &str = "# scene-a sphere-outline exact-difference list\n\
                          # camera: 1 raster: 240x180\n\
                          # columns: pixel x y ref_seg_geom ref_seg_type ref_depth_bits formA_bits formB_bits ulp_ab bound_raw margin\n";

    fn sample(rows: usize, exact_max: i64, bound: i64) -> String {
        let mut text = String::from(HEADER);
        text.push_str(&format!(
            "# rows: {rows}\n# exact_max_ulp: {exact_max}\n# bound_ulp: {bound}\n"
        ));
        for index in 0..rows {
            text.push_str(&format!(
                "{} 0 0 2 5 1 2 3 {} 10.0 11.5\n",
                index + 100,
                exact_max.min(index as i64 + 1)
            ));
        }
        text
    }

    #[test]
    fn parses_valid_list() {
        let path = std::path::Path::new("recipe.txt");
        let parsed = parse_list(&sample(3, 73, 128), Arm::Recipe, path);
        assert_eq!(parsed.rows, 3);
        assert_eq!(parsed.exact_max_ulp, 73);
        assert_eq!(parsed.entries.len(), 3);
        assert!(parsed.entries.contains_key(&100));
    }

    #[test]
    #[should_panic(expected = "does not match")]
    fn rejects_row_count_mismatch() {
        let text = sample(3, 73, 128).replace("# rows: 3", "# rows: 2");
        parse_list(&text, Arm::Recipe, std::path::Path::new("bad.txt"));
    }

    #[test]
    #[should_panic(expected = "duplicate pixel indices")]
    fn rejects_duplicate_pixel_rows() {
        let mut text = sample(2, 73, 128);
        text.push_str("100 0 0 2 5 1 2 3 1 10.0 11.5\n");
        text = text.replace("# rows: 2", "# rows: 3");
        parse_list(&text, Arm::Recipe, std::path::Path::new("bad.txt"));
    }

    #[test]
    #[should_panic(expected = "signed 128")]
    fn rejects_wrong_bound() {
        parse_list(
            &sample(2, 73, 512),
            Arm::Recipe,
            std::path::Path::new("bad.txt"),
        );
    }

    #[test]
    #[should_panic(expected = "exceeds the signed bound")]
    fn rejects_exact_max_over_bound() {
        parse_list(
            &sample(2, 512, 128),
            Arm::Recipe,
            std::path::Path::new("bad.txt"),
        );
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
}
