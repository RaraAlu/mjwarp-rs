# U4 attempt4 evidence (INVALID_ROUND; criterion v2.1 acceptance-surface gap)

attempt=4, criterion `render-sdf-criterion-v2.1` (signed 2026-11-01T02:42:29+08:00),
run 2026-11-01 11:07–11:24 (+08:00). **Verdict: INVALID_ROUND exit10 — not a pass,
not a product regression; the frozen guards are void.**

## §3d attempt4 INVALID (record)

- Body: `PRECHECK_OK` → `OUTLINE_LISTS_OK=2/2` → `GUARDS_OK=94/94` → `INVALID_ROUND exit10`.
  7/9 entities exit 0; `render_kernel_depth_seg` exit101 and `render_kernel_flags` exit101.
- Root cause: criterion **v2.1 acceptance-surface coverage gap** (two comparison points were
  deliberately left un-exempted by the v2.1 implementation):
  1. `render_kernel_flags`: the `culling_off` variant arm kept the raw comparison
     (`variant_depth_diff` exempted only `aa2`) → cam1 `depth_mismatch=183 max_ulp=73`
     first pixel `28969=(169,120)` vs the `<=4 ULP` assertion;
  2. `render_kernel_depth_seg`: the S0b fast-math cross-recipe comparison kept raw
     (`compare_reference` exempts only `label=="recipe"`) → product-vs-S0b cam1
     `mismatched=4247 max_ulp=73 histogram [38953,3677,382,12,27,149] beyond4=149`;
     the "cross-recipe >4 ULP must be 0" and the old histogram fixture assertions failed.
- Offline read-only analysis (`target/v22-precheck.md`; QA independently reproduced every
  number): the 149 >4 ULP pixels are **all inside** the 183 frozen recipe list (149/149, zero
  outside); `culling_off` camera-1 **sphere 747 pixels are seg+depth bitwise identical** to the
  recipe reference (non-sphere: 38 depth diffs, 29 with seg diffs, all outside the list).
  ⇒ Neither failure indicates a product regression; both are acceptance-surface gaps.
- Discipline was clean throughout: `zero_rust_compile=True` (deps 152/152, max write
  2026-11-01 03:11:35), outline lists pre/post `2/2` without drift, post-window exe recheck
  `9/9`, `free_min=13191 MiB`, `floor_breached=False`, `increment_exceeded=False`,
  `residual_processes=0`, `util_max=8`.
- Handling: attempt4 freeze (`guards-sdf-validate.json` sha
  `13ba1bee2f2691e9d93e4d1128c9c193eaa6b4229ea62c2924e0cfea4bdbfd88`, 37,000 B;
  aggregate `c039998a1c682a4dba1729f1006f12a7e9e190f018a61afd78a220d9504757dd`) is **VOID**;
  no retry, no re-freeze, no numeric relaxation. The acceptance surface was changed only by the
  separately user-signed `render-sdf-criterion-v2.2` (2026-11-01T21:42:50+08:00):
  clause 1 (variant arms reuse the recipe list; hardened zero-GPU static assertion) and
  clause 2 (S0b cross-recipe record-only + fail-closed ">4 ULP ⊆ frozen list").

## Files

| file | content |
| --- | --- |
| `invalid-round-record.txt` | the full machine-readable INVALID round record (root cause, discipline, policy) |
| `guards-sdf-validate.json` | attempt4 frozen guards (94 entries; sha `13ba1bee../37,000 B`; VOID, archived for evidence only) |
| `facts.txt` / `guard.log` | runner facts (body) and guard log (`PRECHECK_OK`/`OUTLINE_LISTS_OK`/`GUARDS_OK`/`INVALID_ROUND exit10`) |
| `entity-results.csv` | 9 entity rows with exit codes and wall times |
| `sha-guards.txt` | 92 sha lines (94 entries minus the 2 lists, which are verified separately) |
| `samples.csv` | 0.25 s GPU sampler for the body |
| `baseline.txt` / `machine-load.txt` | body-time baseline and machine load |
| `monitor-pre-window-attempt4.csv` | regular pre-window monitor (60/60 clean, exit 0) |
| `logs/` | authoring tool, freeze, guards-only, monitor and the 9 entity logs (`logs/guards-only-baseline.txt` / `logs/guards-only-machine-load.txt` preserve the guards-only samples) |
| `notes-errata.txt` | invalid invocation record + runner-floor caliber |

## Boundaries / not claimed

- This directory is an **invalid-round archive**: it must not be quoted as a normal-threshold
  pass, an acceptance, a parity statement or a precision result.
- The two signed lists, the criterion numbers (B=128, 183/172, exact max 73/91) and all
  references/fixtures/kernel identities are unchanged by this archive.
