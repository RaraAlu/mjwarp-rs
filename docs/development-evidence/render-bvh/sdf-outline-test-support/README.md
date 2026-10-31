# env-gated scene-a sphere-outline test support (criterion v2.1)

Task: `2026-11-01-render-sdf-outline-test-support` (Arch; offline, zero GPU)
Basis: signed `render-sdf-criterion-v2.1` (2026-11-01T02:42:29+08:00) and
`sdf-outline-tool-v2.1/` (contract + self-check evidence).

## Purpose

Make the six scene-a regression entities consume the frozen exact-difference lists at body
time, without changing the product `src/` and without changing any assertion text. Behaviour is
**env-gated**: plain `cargo test` / CI is unchanged; only the validation-window runner exports
the two variables.

Metadata source of truth: the 9-column list files stay the signed proposal prototype
byte-for-byte (no extra header lines). `rows`, `exact_max_ulp` and the signed `bound_ulp` are
read from the machine-readable `outline-lists.json` that the authoring tool writes next to the
lists, and cross-checked against the list (data rows, unique pixels, sha256, bytes, first pixel).
Freeze, runner and body-time support all consume the same JSON record, so the three cannot drift.

Env contract (window runner sets them; see slice 4):

| variable | meaning |
| --- | --- |
| `MJWARP_RENDER_OUTLINE_LIST_DIR` | directory holding `scene-a-sphere-outline-recipe.txt` / `...-aa2.txt` |
| `MJWARP_REQUIRE_OUTLINE_CRITERION` | must be `1` when the list dir is set |

Fail-closed matrix (all panics, never silent relaxation):

| condition | behaviour |
| --- | --- |
| only one env var set, or `REQUIRE != 1` | panic (misconfiguration) |
| list file missing / unreadable | panic |
| `# rows:` != number of data rows | panic |
| `# bound_ulp:` != 128 | panic |
| `outline-lists.json` missing / invalid | panic |
| JSON arm entry missing, or JSON rows != data rows | panic |
| duplicate pixel indices (rows != unique pixels) | panic |
| JSON / list sha256, bytes or first pixel mismatch | panic |
| JSON `bound_ulp` != 128 or `exact_max_ulp` > 128 | panic |
| malformed row / pixel index out of range | panic |
| cam 1 pixel outside the list with a non-zero delta | panic (violation) |
| cam 1 in-list pixel > 128 ULP | panic (violation) |
| cam 0 any delta != 0 | panic (bitwise, unchanged) |
| no env vars | raw product comparison (zero behaviour change) |

Enforcement point is the panic in `outline::diff_depth_with_outline` / `outline::depth_stats`;
the helper returns numbers re-expressed for the criterion (`mismatched = outside-list
mismatches`, `max_ulp = 0`, `beyond_ulp = 0` when no violation) so that the existing assertion
text does not become the enforcement point. The real values are printed by the
`outline[<arm>] cam1: ...` log line (list rows, outside mismatch, in-list differing count,
in-list max ULP, violations, exact max, list path).

## Wiring (files touched)

New: `tests/support/render_bvh/outline.rs` (module included from `s7b1.rs` via
`#[path = "outline.rs"] pub mod outline;`, so every render test that already includes s7b1 gets
it without extra module declarations).

| file | change |
| --- | --- |
| `tests/support/render_bvh/s7b1.rs` | expose `pub mod outline`; `compare_reference` applies the list only for `label == "recipe"` (the cross-recipe `s0b-fast-math` comparison stays raw) |
| `tests/support/render_bvh/e2e.rs` | `assert_depth_seg_same_recipe` uses `outline::diff_depth_with_outline(..., Arm::Recipe)` |
| `tests/support/render_bvh/skybox.rs` | `compare` uses `outline::diff_depth_with_outline(..., Arm::Recipe)` |
| `tests/render_kernel_rgb.rs` | one call swap to the outline-aware helper (Arm::Recipe) |
| `tests/render_r10_multi_world.rs` | one call swap (Arm::Recipe) |
| `tests/render_kernel_flags.rs` | aa2 variant arm -> Arm::Aa2, baseline arm -> Arm::Recipe, other variants keep the raw comparison |

Product `src/` untouched; fixtures/criteria values/kernel untouched; assertion text unchanged.

## Unit tests (run without GPU, no env)

`tests/support/render_bvh/outline.rs` (module) + `outline_tests.rs` add ten non-ignored tests:
valid parse, missing-JSON panic, JSON-row mismatch panic, duplicate-pixel panic, wrong-bound
panic, exact-max-over-bound panic, ULP ordering, a CPU-only regression on the **committed
prototype 9-column list** (183 rows / exact max 73), an env-enabled success path (both variables
set, prototype list + JSON in a temp dir) and an env-enabled missing-JSON panic. Captured log:
`logs/outline-tests.txt` (`10 passed; 0 failed`).

## Boundaries

Zero GPU, no freeze, no window. This slice only wires the consumer; the guards file, env export
and list-immutability checks are slice 4, and the window material/plan is slice 5. Not claimed:
no acceptance/precision/pass statement; U4 stays not passed until attempt4 runs.
