# perworld S1 attempt3 (valid window)

- Task: `2026-10-30-render-perworld-impl-s1` (in_progress at window time; acceptance pending QA verdict)
- Material: `docs/development-evidence/render-bvh/perworld-s1-window-material/` (material v3; signed plan text sha256 `1fb883a4...`)
- Window HEAD: `f122adec` (docs-only ahead of `8559c9b2`; `src`/`tests` identical)
- Opened by: Chief (window authority delegated by the user on 2026-11-03)
- Result: **valid round, all 11 entities exit 0**

## Floor

- Signed floor (third sign): 11,755 MiB
- Pre-window baseline: free 15,068 / 15,079 / 15,079 MiB at 22:56:22/24/25, util 2%, 27 C
- Derived from this baseline: 15,068 - 3,072 = 11,996 MiB
- Floor used (strict): **11,996 MiB**

## Monitor

- 60/60 clean samples (`util < 40`, no compute apps, no build), 22:56:34 - 23:01:34
- util range 2-5, free_min 14,747 MiB

## Freeze and guards

- 96 entries = 11 exes + 83 read-only inputs + 2 criterion-v2.2 outline lists
- aggregate_sha256 `8a4ddc50be2d8e766431533e6ccd8ab4cf5082fdbd63d6638a44b128e9ad4b3e`
- guards_sha256 `ca3ecd5c407e0e5273a2867794591026d55ee1904f1cf55b689bdc6c8844a15a`
- stash dir `target/render-bvh/perworld-s1-window/frozen-exes`
- GuardsOnly: `OUTLINE_LISTS_OK=2/2`, `GUARDS_OK=96/96`, `GUARDS_ONLY_DONE`

## Body

- 11/11 entities exit 0, body wall 50.1 s (23:02:14 - 23:03:04)
- samples 170, free_min 14,531 MiB, util_max 8, used_peak 1,531 MiB
- delta 278 MiB / budget 3,072 MiB, floor_breached False
- zero_rust_compile True, post-window exe recheck 11/11 (mismatches 0), residual processes 0

| entity | kind | exit | wall_ms |
| --- | --- | --- | --- |
| render_per_world | new | 0 | 1632 |
| render_context_lifecycle | regression | 0 | 7741 |
| render_e2e | regression | 0 | 8874 |
| render_hfield_reference | regression | 0 | 2728 |
| render_kernel_depth_seg | regression | 0 | 4316 |
| render_kernel_flags | regression | 0 | 9677 |
| render_kernel_rgb | regression | 0 | 3713 |
| render_mesh_bvh | regression | 0 | 997 |
| render_r10_multi_world | regression | 0 | 2693 |
| render_shadow_reference | regression | 0 | 4509 |
| render_skybox_reference | regression | 0 | 1608 |

## Deviations

None. No retry, no re-freeze, no threshold or tolerance change.

## Claims and boundaries

- L v1 scope is per-world geom poses only; per-world light arrays stay model-level single copy.
- AA / spp>1 is not covered (per-world entry rejects nsamples>1).
- No upstream nworld>1 numeric-equivalence claim; no performance statement.
- Criterion: rgb <= 1 LSB, seg exact; depth bitwise outside the signed lists and `|ULP| <= 128` inside them
  (criterion v2.2 signed lists, `exact_max_ulp` 73/91 are in-list maxima, not widened tolerance).
- Numeric acceptance of this round requires the independent QA verdict; this archive alone does not claim it.

## Archive

- This directory contains the attempt3 payload (22 files), a copy of the frozen guards body, and
  `manifest.txt` (sha256 + bytes per file; the manifest does not register itself).
