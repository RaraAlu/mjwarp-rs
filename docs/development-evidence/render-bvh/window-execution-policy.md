# Render validation window execution policy (single executor, lock, record-only posts)

Status: adopted 2026-11-03 after the perworld S1 attempt3 parallel-launch incident.
Scope: all render/BVH device validation windows that use `scripts/validate/render-bvh/run-window.ps1`.

## 1. Exactly one executor session

- A window run (monitor + freeze + GuardsOnly + body) is executed by exactly ONE Bot session.
- Group chat posts from the Chief are RECORD ONLY. They are never an execution signal. The executor starts only on
  the direct instruction from the Chief to the executor Bot ("GO"), and the group post must say "record only".
- Window opening authority belongs to the Chief (delegated by the user on 2026-11-03). Once the four preconditions in
  section 3 hold, the Chief posts the GO and the record-only trail; no per-window user double-sign is needed.
  Only three things still go to the user: relaxing a criterion, changing a reference, and an external release claim.
- Before starting, the executor states in the group: plan version + three-point baseline + signed floor + expected
  guard count, and that it is the only executor.

## 2. Mutual exclusion marker (`.window-lock`)

- The runner acquires `<out-dir>/.window-lock` (content: pid, start time, out dir) before any window step.
- If the marker exists and its pid is alive, the runner prints `WINDOW_LOCK_HELD pid=...` and exits 17 without running.
- A marker whose pid is dead is stale and is removed before proceeding.
- Every exit path and the normal end release the lock.
- Operational rule: a second session never deletes a live marker; it stops and reports.

## 3. Preconditions (all must hold before opening)

1. Implementation and window material closed; independent QA pass on the material (and on the implementation for the
   entity set) recorded.
2. Three-point `nvidia-smi` baseline sampled within 5 minutes before opening; every point `free >= signed floor`;
   compute-apps filtered = 0; build processes = 0; worktree clean.
3. `floor = max(this baseline's derived value (min_free - 3072), previously signed floor)` - the stricter value governs.
4. Failure means stop: no retry, no re-freeze to "make it pass", no threshold relaxation. The attempt is archived
   verbatim (INVALID_ROUND or valid round), including facts/logs/guards body and a manifest.

## 4. Runner state files (write-order rules)

- The GPU sampler runs as a background job; it creates `.abort` and appends to `samples.csv` before sampling.
- The body waits (max 5 s) for `.abort` to appear; all reads of `.abort` are guarded with `Test-Path`.
- `.stop-sampler` is written by the main flow only after the body loop; the sampler exits on it.
- A missing state file is never a terminating error; it is treated as "no abort" and recorded as such.

## 5. Attempt bookkeeping

- Two freezes are not permitted within one attempt. If a launch is aborted by a harness bug, it is recorded as a
  process deviation and the next launch is a NEW attempt with a fresh freeze and a fresh direct GO.
- Window evidence: `facts.txt`, `entity-results.csv`, `guard.log`, `guard_aggregate.json`, `samples.csv`,
  `sha-guards.txt`, monitor CSV, baseline snapshot, guards body copy, entity logs, manifest (not self-referencing).

## 6. Write lock precedes the window lock

- The executor first takes the repository write lock (`.normify/locks/repo-write.lock` via
  `scripts/validate/repo-write-lock.ps1 -Action acquire -Owner <session>`), then enters the `<out-dir>/.window-lock`
  flow. Both layers must pass; the write lock guards any repo write (including evidence archiving and commits).
- The lock file is runtime state under `.normify/locks/` and is git-ignored; `-Action release` before finishing so the
  worktree returns to clean (windows require a clean worktree).
- Policy: `docs/development-evidence/write-lock-policy.md`.

## 7. Guards invalidation (any test-support or source change)

- A freeze binds the 11 test executables by sha256. ANY change under `tests/` (including test-support helpers) or
  under `src/` changes executable identities, so all 96 guard entries become stale as a set - the guards file bytes may
  stay unchanged while the exes differ.
- Therefore: after such a change the next window is a NEW attempt with a fresh freeze; the previous guard set is never
  reused, and a previous attempt's numeric evidence is never reused for the new tree.
- Current state (2026-11-03): `ebba11f7` changed `tests/support/render_bvh/` (Arm required + bare-`diff_depth`
  allowlist). 11 of the 96 guard entries (the test exes) are stale from that commit on; the 83 read-only inputs and the
  2 signed criterion lists are unaffected. The perworld S1 attempt3 valid-round evidence stays bound to `f122adec`
  (its `src/` is identical and the test-support change is criterion-neutral), but it does not carry over to a re-run.

## 8. Freeze expectation for the next window

- Re-freeze with `scripts/validate/render-bvh/freeze-guards-perworld-s1.ps1 -Attempt <n>
  -CriterionVersion render-sdf-criterion-v2.2 -BoundUlp 128`; expected `96 = 11 exe + 83 inputs + 2 lists`,
  `GuardsOnly 96/96`, `OUTLINE_LISTS_OK=2/2`.
- Record the new guards sha256 and aggregate in the attempt facts; do not compare them against the attempt3 values.
