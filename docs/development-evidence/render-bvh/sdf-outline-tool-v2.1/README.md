# scene-a sphere-outline authoring tool v2.1 contract

Task: `2026-11-01-render-sdf-outline-tool-v21-contract` (Arch; offline, zero GPU)
Basis: signed `render-sdf-criterion-v2.1` (2026-11-01T02:42:29+08:00, approved_by=user via Chief,
evidence `docs/development-evidence/render-bvh/sdf-criterion-v2.1/`).

This slice updates the offline authoring tool to the v2.1 contract. It changes only
`scripts/authoring/**` (plus this evidence); it does not touch product `src/`, tests, fixtures,
criteria values, references or the kernel.

## Tool

- wrapper: `scripts/authoring/author-scene-a-sphere-outline.ps1`
- helper: `scripts/authoring/author-scene-a-sphere-outline.py`
- replay: `scripts/authoring/scene_a_outline_replay.py`

The wrapper is unchanged in behaviour: pinned interpreter/package versions and upstream identity
are re-checked (fail-closed), it refuses to write under `fixtures/`, runs the helper twice and
requires byte-identical outputs, and independently re-verifies every emitted sha256.

## v2.1 contract

Per arm (`recipe` = spp 1, `aa2` = spp 2 sub-sample 0 on the doubled grid):

| output | content |
| --- | --- |
| `scene-a-sphere-outline-<arm>.txt` | exact-difference list: `pixel x y ref_seg_geom ref_seg_type ref_depth_bits formA_bits formB_bits ulp_ab bound_raw margin`, sorted by pixel; header carries criterion version, reference path + sha256, camera/raster, rows, `exact_max_ulp`, `bound_ulp=128` |
| `outline-lists.json` | machine-readable freeze contract: `criterion_version`, `bound_ulp`, `bound_definition`, per arm `{arm, entities, path, bytes, sha256, rows, first_pixel, first_xy, exact_max_ulp, spp, raster, sphere_pixels, reference, reference_sha256, analytic_bound{bound_raw/margin min/median/max + max observed ratios}}` |
| `b-recompute.txt` | exact max vs signed B verdict + analytic bounds recorded only |
| `selfcheck.txt` | ray/form-A/form-B self-check and attempt3 consistency |
| `open-items.txt` | freeze-step prerequisites owned by the window runner |
| `sha256sums.txt` | sha256 of every emitted file |

Exit codes: **0** = freeze verdict OK (`exact_max <= 128` and self-check pass); **4** = exact max
> 128 (v2.1 freeze failure; raising B forbidden); **5** = internal self-check failed; **6** =
pinned input drift.

Self-check reproduces the frozen data (see `logs/tool-selfcheck.txt`): rays vs S5 fixture
**256/256** bitwise; form A vs frozen reference depth bits **747/747** (recipe) and **745/745**
(aa2); list **183 px / exact max 73 ULP** (first 28969 = (169,120)) and **172 px / exact max 91
ULP** (first 29448 = (168,122)); camera 0 sphere pixels **0**; two runs byte-identical, 6 sha
re-verified by the wrapper.

## Contract output sample (self-check run, `target/render-bvh/sdf-validate-window/attempt4/outline-v21-selftest/`)

| file | bytes | sha256 |
| --- | --- | --- |
| `scene-a-sphere-outline-recipe.txt` | 13,944 | `43ffaf1b2f8b2dc9b195e7cf78f2e2b69253f2b3d3bd4a975c39d80ff5f68b3c` |
| `scene-a-sphere-outline-aa2.txt` | 13,130 | `0c37a22022fdb76d852728e7b7552638fc75152f0b5d76fb4b5d2d262dc6bb06` |
| `outline-lists.json` | 2,356 | `58c57bb369a51bef1510a1109de61d7112d69031954cd50bcd380fb7f41a8c28` |
| `b-recompute.txt` | 1,407 | `9df88b2725c54ab31796db42b33094c0d47eeb5c47f94db3b991240cc4eb58b6` |
| `selfcheck.txt` | 787 | `44ad3afb9407eec46eea5c8d6511ba91867d05d65ad704d3392ca1e99f4ca755` |
| `open-items.txt` | 632 | `8af93f6776451893ff0f32b71c719f84b9eb94bdf1a4991ebf0a5737e4f3b322` |
| `sha256sums.txt` | 525 | `45375a6e9cbc58dab8fc28ea10a5d8910b9b3638f811c84a7569c69d30887e12` |

The 11-column list format differs from the `prototype-lists/` proposal prototype (9 columns, no
bounds); list membership, first pixel and exact max are unchanged (same 183/172 and 73/91), the
prototype stays as proposal evidence, and the freeze step consumes `outline-lists.json`.

## Boundaries

Zero GPU, no freeze, no window, no body. Product build/tests/cargo gates/product scripts never
reference the tool (grep proof in `authoring-exception-v21.txt`). Not claimed: no acceptance,
precision, parity or pass statement; U4 stays not passed until attempt4 actually runs under the
signed criterion.
