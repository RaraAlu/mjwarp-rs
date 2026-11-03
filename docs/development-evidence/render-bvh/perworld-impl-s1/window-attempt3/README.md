# perworld S1 attempt3 (valid body round; process errata included)

- Task: `2026-10-30-render-perworld-impl-s1` (in_progress at window time; acceptance pending QA verdict)
- Material: `docs/development-evidence/render-bvh/perworld-s1-window-material/` (material v3; signed plan text sha256 `104ca1e5...`; signed §8 baseline 14,802/14,801/14,813; signed floor 11,755)
- Window HEAD: `f122adec` (docs-only ahead of `8559c9b2`; `src`/`tests` identical)
- Opened by: Chief (window authority delegated by the user on 2026-11-03)
- Result: **valid body round, all 11 entities exit 0** — with the process deviations recorded in section 6.

## 1. Pre-window baseline (recorded before the monitor; the number set required by §8)

- 3 read-only `nvidia-smi` points at **2026-11-03T22:56:14.126 / :17.176 / :20.235 +08:00**
- free **14,700 / 14,762 / 14,762 MiB**; util **4 / 2 / 2 %**; temp 27 C; compute-filtered 0; build processes 0
- signed floor (third sign, §8): **11,755 MiB** — derived from the sign-time baseline 14,802/14,801/14,813
  (`max(14,801 - 3,072 = 11,729, attempt2 floor 11,755) = 11,755`); every pre-window point is >= 11,755
- monitor window **22:56:34 - 23:01:34**: **60/60** clean samples (util < 40, no compute apps, no build),
  util range 2-5, **free_min 14,747 MiB**
- runner's own pre-body sample (inside `run-window.ps1` at 23:02:12): 14,809 / 14,809 / 14,809 MiB;
  runner floor `min(10,240, 14,809 - 3,072) = 10,240 MiB` (lower bound only; signed floor 11,755 governs)

## 2. Freeze and guards (two freezes happened; see section 6)

- 96 entries = 11 exes + 83 read-only inputs + 2 criterion-v2.2 outline lists
- aggregate_sha256 `8a4ddc50be2d8e766431533e6ccd8ab4cf5082fdbd63d6638a44b128e9ad4b3e` (identical for both freezes)
- guard file sha256: first freeze `462262ef08d01f74aa19c08aea96fdec85b69baf16027aefb3a2093155a864c3` (23:01:34),
  second freeze `ca3ecd5c407e0e5273a2867794591026d55ee1904f1cf55b689bdc6c8844a15a` (used by the valid body);
  the two freezes differ only in `generated_at` (same entries, same aggregate)
- stash dir `target/render-bvh/perworld-s1-window/frozen-exes`
- body-time GuardsOnly: `OUTLINE_LISTS_OK=2/2`, `GUARDS_OK=96/96`, stash 11/11
- `outline_lists_post=2/2`

## 3. Body (single valid body execution)

- 11/11 entities exit 0, body wall **50.1 s** (23:02:14.776 - 23:03:04.885)
- samples 170, **free_min 14,531 MiB**, util_max 8, used_peak 1,531 MiB
- delta 278 MiB / budget 3,072 MiB; `floor_breached=False`; `increment_exceeded=False`
- `zero_rust_compile=True`, `post_window_exe_recheck=11/11` (mismatches 0), `residual_processes=0`

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

## 4. Criteria and boundaries

- Criterion: rgb <= 1 LSB, seg exact; depth bitwise outside the signed outline lists and `|ULP| <= 128`
  inside them (criterion v2.2 signed lists; `exact_max_ulp` 73/91 are in-list maxima, not widened tolerance).
- L v1 scope is per-world geom poses only; per-world light arrays stay model-level single copy.
- AA / spp>1 is not covered (per-world entry rejects nsamples>1).
- No upstream nworld>1 numeric-equivalence claim; no performance statement.
- Numeric acceptance of this round requires the independent QA verdict; this archive alone does not claim it.

## 5. Session isolation and window lock (process rules adopted after this round)

- Two Arch sessions were active: this chat session froze attempt3 at 23:01:34 and launched a body at ~23:01:57;
  the group session re-froze at ~23:02:0x and launched the valid body at 23:02:12.
- Root cause of the parallel launch: Chief's group-post was treated as an execution signal by the group session
  while the direct instruction channel was still in use. Rule adopted: window execution runs in exactly ONE session;
  group posts are record-only and explicitly marked "record only, execution follows the direct instruction".
- Rule adopted: a window executor must acquire a mutual-exclusion marker before starting
  (`<out-dir>\.window-lock` with pid + start time); if a live marker exists the executor must exit without running.
  The runner lock and `.abort` read-race fixes are tracked separately (CPU-only, post-window).

## 6. Deviations (zero concealment) and errata

1. **Parallel duplicate launch (first body aborted by a runner race).** The first body launch aborted with exit 1 in
   `run-window.ps1:326` because it read `$abortFile` without a `Test-Path` guard while the sampler background job had
   not yet created `.abort` (a harness race, not an entity or criteria failure). No facts file was produced and no
   entity completed in that launch; its partial logs were overwritten by the valid launch. This archive contains the
   valid launch facts only.
2. **Two attempt3 freezes.** Each launch performed its own attempt3 freeze. Both produced 96 entries with the same
   aggregate (`8a4ddc50...`); the guard file sha differs only because `generated_at` differs
   (`462262ef...` vs `ca3ecd5c...`). The valid body used the second freeze (archived guard body `ca3ecd5c...`);
   precheck and body used the same guard file, so the valid body round is internally consistent.
3. **README errata (this file was corrected after the window).** The first version of this README recorded the
   pre-window baseline as 15,068/15,079/15,079 and a derived floor of 11,996 MiB; that did not match the window facts.
   Corrected numbers are in sections 1-2 above: pre-window baseline 14,700/14,762/14,762 MiB (util 4/2/2),
   monitor free_min 14,747 MiB, signed floor 11,755 MiB, runner baseline 14,809 x3, runner floor 10,240 MiB,
   body free_min 14,531 MiB. The first version also claimed "no retry, no re-freeze"; the accurate record is in
   items 1-2 above.
4. No criteria, tolerance, reference or fixture change was made at any point; no threshold was relaxed.
   This errata corrects only the record, not any criterion or result.

## 7. Archive

- This directory holds 24 manifest entries / 25 files: 22 payload files + a copy of the frozen guard body
  (`guards-perworld-s1.json`) + `manifest.txt` (sha256 + bytes per file; the manifest does not register itself).
