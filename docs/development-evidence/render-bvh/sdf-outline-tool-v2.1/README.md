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
| `scene-a-sphere-outline-<arm>.txt` | **signed proposal prototype, byte-identical**: 9 columns `pixel x y ref_seg_geom ref_seg_type ref_depth_bits formA_bits formB_bits ulp_ab`; header = criterion line, reference path, reference sha256, camera/raster/spp, columns |
| `outline-bounds-<arm>.txt` | analytic-bound sidecar: header + one row per listed pixel (`pixel bound_raw margin`); recorded only, explicitly not used to determine B |
| `outline-lists.json` | machine-readable freeze contract: `criterion_version`, `bound_ulp`, `bound_definition`, per arm `{arm, entities, path, bytes, sha256, prototype_byte_identical, bounds_sidecar{path,bytes,sha256}, rows, first_pixel, first_xy, exact_max_ulp, spp, raster, sphere_pixels, reference, reference_sha256, analytic_bound{...}}` |
| `b-recompute.txt` | exact max vs signed B verdict + prototype identity + analytic bounds recorded only |
| `selfcheck.txt` | ray/form-A/form-B self-check, prototype byte identity and attempt3 consistency |
| `open-items.txt` | freeze-step prerequisites owned by the window runner |
| `sha256sums.txt` | sha256 of every emitted file |

Exit codes: **0** = freeze verdict OK (lists byte-identical to the prototype, `exact_max <= 128`
and self-check pass); **4** = exact max > 128 (v2.1 freeze failure; raising B forbidden);
**5** = internal self-check failed (including a prototype byte-identity mismatch); **6** =
pinned input drift.

Self-check reproduces the frozen data (see `logs/tool-selfcheck.txt`): rays vs S5 fixture
**256/256** bitwise; form A vs frozen reference depth bits **747/747** (recipe) and **745/745**
(aa2); list **183 px / exact max 73 ULP** (first 28969 = (169,120)) and **172 px / exact max 91
ULP** (first 29448 = (168,122)); camera 0 sphere pixels **0**; the two lists are **byte-identical
to the signed proposal prototype** (`cfd1038a.. / 10,099 B / 183 rows`, `14893fc6.. / 9,509 B /
172 rows`); two runs byte-identical, 8 sha re-verified by the wrapper.

## Contract output sample (self-check run, `target/render-bvh/sdf-validate-window/attempt4/outline-v21-selftest/`)

| file | bytes | sha256 |
| --- | --- | --- |
| `scene-a-sphere-outline-recipe.txt` | 10,099 | `cfd1038a1cc50326de51a25495db68cd8b25c0983121dea88a2fb89ca83724ee` |
| `scene-a-sphere-outline-aa2.txt` | 9,509 | `14893fc6a2479e8e8f5e9cc07cabe72fd9f37aea638dda25fc26839d59ec8eab` |
| `outline-bounds-recipe.txt` | 5,027 | `8a80fb0a43cbe1c2805ce7307c1ed62f0f7f1f42cf2ded621dcec3d658cf7948` |
| `outline-bounds-aa2.txt` | 4,734 | `f23ca697f34ebd75a21c618ae5c309cfc710d2f2ad82349f864f02dcb30fc303` |
| `outline-lists.json` | 2,808 | `a25dbb0d3a2c02fa2d3583e5c0e814109d394f46dba4bf016248846188e1ed39` |
| `b-recompute.txt` | 1,994 | `fe1b62053c7939792ce37eb098e0a65ed35ed9dfcf82dd766dd8489e564a17c9` |
| `selfcheck.txt` | 1,051 | `35a6a71d530bc2fc7b2cfa4402d2c488a06b6ab65ae2be5937823f245ec54106` |
| `open-items.txt` | 632 | `8af93f6776451893ff0f32b71c719f84b9eb94bdf1a4991ebf0a5737e4f3b322` |
| `sha256sums.txt` | 706 | `acaec0020207b23cb19bdd6a5367e6322a55eb05d5186daf69c630b9b03bc997` |

The two list files are the signed proposal prototype byte-for-byte (same 183/172 rows, same
membership, same first pixels and exact maxima 73/91); the analytic `bound_raw`/`margin` columns
moved to the `outline-bounds-<arm>.txt` sidecars, and the freeze step consumes `outline-lists.json`.

## Boundaries

Zero GPU, no freeze, no window, no body. Product build/tests/cargo gates/product scripts never
reference the tool (grep proof in `authoring-exception-v21.txt`). Not claimed: no acceptance,
precision, parity or pass statement; U4 stays not passed until attempt4 actually runs under the
signed criterion.
