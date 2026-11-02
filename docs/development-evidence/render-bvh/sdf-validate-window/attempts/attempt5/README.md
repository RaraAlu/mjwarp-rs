# U4 attempt5 evidence (VALID ROUND; criterion v2.2)

attempt=5, criterion `render-sdf-criterion-v2.2` (signed 2026-11-01T21:42:50+08:00),
window plan v4 + baseline v4 double-signed 2026-11-02T08:29:18+08:00, run 2026-11-02
13:16–13:26 (+08:00) inside the 12:00–14:00 idle band.

**Runner verdict: `PRECHECK_OK` → `OUTLINE_LISTS_OK=2/2` → `GUARDS_OK=94/94` → `WINDOW_RUN_DONE`,
EXIT=0.** This file records the measured facts; it makes no acceptance, parity or precision
statement (U4 acceptance status is decided by the criteria owner).

## Window path and monitor

- Pre-window monitor: **regular** `monitor-until-clean.ps1 -Required 60 -MaxMinutes 10`
  (PS 5.1, `MaxUtil=40`, no `-OpenUtilLimit`), CSV `monitor-pre-window-attempt5.csv`:
  **94 samples, longest_clean=60, ready=True, util 1–64 %, free_min 12,435 MiB, EXIT=0**.
  No exception path was used (`authoring_exception_used=false` in facts); `ge40`/exception
  monitors are not part of this window.
- Body-time baseline re-sample (manually checked before the body): free 12,947 / 12,947 /
  12,947 MiB, used 3,115, util 4 / 4 / 4, min free **12,947 ≥ signed floor 10,376 MiB**;
  no compute app held GPU memory, no cargo/rustc/nvcc/link process. (The runner's own floor at
  this baseline is min(10240, 12947−3072) = 9,875 MiB, which is LOOSER than the signed floor;
  the signed floor governed and was met.)
- First 12:00 open attempt was stopped manually: free 1,747 MiB < floor (external desktop load);
  see `window-open-attempt.txt`. The window was opened only after the GPU was released.
  Archive anchors for the released state: the body-time re-sample in `baseline.txt`
  (2026-11-02T13:24:53/56/59, free 12,947 MiB x3, util 4/4/4) and the regular-monitor CSV points
  13:23:31.643 (free 12,942 MiB, util 5) and 13:23:41.809 (free 12,942 MiB, util 4). The earlier
  13:14 observation existed only in the local watch log and is NOT cited as evidence.

## Freeze identity (frozen guards, also committed at `scripts/validate/render-bvh/guards-sdf-validate.json`)

- guards sha256 **`9e000bfc1f8438a4bd6fd5f8c8ccf214c2ab43aee7c1ed1fce78ed51e7646c32`**, 37,000 B,
  aggregate **`034c7551226e321fab83b0ec4ef92a2dd82e9077ee3f31e7ad88f9a5d3b305b9`**,
  `criterion_version=render-sdf-criterion-v2.2`, `bound_ulp=128`,
  94 = 9 exe + 83 inputs + 2 lists.
- `sha_guards=94/94 (9 exes + 83 read-only inputs), mismatches=0`; post-window exe recheck 9/9.
- Lists unchanged and byte-identical to the signed prototype: recipe
  `cfd1038a1cc50326de51a25495db68cd8b25c0983121dea88a2fb89ca83724ee` / 10,099 B / 183 rows
  (exact max 73 ULP), aa2 `14893fc6a2479e8e8f5e9cc07cabe72fd9f37aea638dda25fc26839d59ec8eab` /
  9,509 B / 172 rows (exact max 91 ULP); pre-body and post-window checks 2/2, no drift.

## Body (9 entities, env-gated criterion v2.2)

| entity | kind | exit | wall ms |
| --- | --- | --- | --- |
| render_sdf_reference | new | 0 | 7938 |
| render_context_lifecycle | regression | 0 | 7914 |
| render_e2e | regression | 0 | 8630 |
| render_kernel_depth_seg | regression | 0 | 4394 |
| render_kernel_flags | regression | 0 | 9279 |
| render_kernel_rgb | regression | 0 | 3673 |
| render_mesh_bvh | regression | 0 | 977 |
| render_r10_multi_world | regression | 0 | 2710 |
| render_skybox_reference | regression | 0 | 1610 |

- `body_wall_s=48.4`, `attempt_valid_rounds=5`, `guard_aborted=False`.
- `zero_rust_compile=True` (deps 152/152, deps max write 2026-11-01 21:51:29), post-window exe
  recheck 9/9, `residual_processes=0`.
- Samples 162, `used_peak_mib=3385`, `free_min_mib=12677`, `util_max_percent=13`,
  `util_streak_ge_80=0`, `increment_exceeded=False`, `floor_breached=False`.
- Clause 1/2 of criterion v2.2 were enforced by the env-gated test support
  (`MJWARP_RENDER_OUTLINE_LIST_DIR` + `MJWARP_REQUIRE_OUTLINE_CRITERION=1`); the product↔S0b
  comparison was record-only and the >4 ULP subset rule stayed fail-closed.

## Files

| file | content |
| --- | --- |
| `facts.txt` / `guard.log` | runner facts and guard log (`PRECHECK_OK`/`OUTLINE_LISTS_OK`/`GUARDS_OK`/`WINDOW_RUN_DONE`) |
| `entity-results.csv` | 9 entity rows (all exit 0) |
| `sha-guards.txt` | 92 sha lines (94 entries minus the 2 lists, verified separately) |
| `samples.csv` | 0.25 s GPU sampler for the body |
| `baseline.txt` / `machine-load.txt` | body-time baseline and machine load |
| `monitor-pre-window-attempt5.csv` | regular pre-window monitor (60/60 clean, exit 0) |
| `window-open-attempt.txt` | the 12:00 manual stop by the signed floor (external load) |
| `guard_aggregate.json` | guard aggregate record (entries/counts and the sampler interval; the same guard numbers also appear in `facts.txt`) |
| `guards-sdf-validate.json` | the frozen attempt5 guards (also committed under scripts/) |
| `outline/` | the 9 authoring outputs: two 9-column lists, `outline-lists.json`, two bounds sidecars, determinism/sha/selfcheck/recompute/open-items reports |
| `logs/` | authoring, freeze, guards-only, monitor, body and the 9 entity logs |

## Boundaries / not claimed

No acceptance, parity or precision statement is made by this evidence; it records the runner
verdict and the measured facts under the signed criterion v2.2. The criterion numbers, lists,
references, fixtures, product src and kernel identity are unchanged by this window.
