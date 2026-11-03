# Render validation window execution policy (single executor, lock, record-only posts)

Status: adopted 2026-11-03 after the perworld S1 attempt3 parallel-launch incident.
Scope: all render/BVH device validation windows that use `scripts/validate/render-bvh/run-window.ps1`.

## 1. Exactly one executor session

- A window run (monitor + freeze + GuardsOnly + body) is executed by exactly ONE Bot session.
- Group chat posts from the Chief are RECORD ONLY. They are never an execution signal. The executor starts only on
  the direct instruction from the Chief to the executor Bot ("GO"), and the group post must say "record only".
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