# U4 attempt4 window material v2

Task: `2026-10-31-render-sdf-validate-window-material-v2` (Arch; offline, zero GPU)
Executed task: `2026-10-31-render-sdf-validate` (GPU full load; user double-sign required)

This directory holds the attempt4 window material under the signed `render-sdf-criterion-v2`
(B=128 ULP, signed 2026-10-31T11:04:37+08:00, approved_by=user via Chief).

- `window-plan.txt` — full material: three-question header (aligned with `criteria-purpose-audit/standard.md`),
  bitwise SDF entity criteria, criterion v2 controlled exemption and freeze-time B recomputation, 9 entities,
  conditional monitor protocol, preparation checklist, pre-window baseline rule, open items.
- `receipt.json` — machine-readable summary + signatures + kernel identity + next steps.
- `baseline-pre-window.txt` — pre-window GPU baseline (three read-only `nvidia-smi` points,
  free 8400/8400/8400 MiB, floor = 8400 - 3072 = 5328 MiB) and the window-time re-sample rule.
- `authoring-exception.txt` — offline authoring-exception record for the freeze-step dump tool:
  boundary basis, pinned interpreter/package versions, upstream identity, script sha256,
  zero-product-usage grep proof.
- `dump-tool-selfcheck.txt` — captured wrapper log of the tool self-check (pinned-env line,
  both script hashes, two-run byte-identical determinism, independently re-verified sha256sums,
  exit code 4 = criterion verdict fails).
- `outline-replay-findings.txt` — freeze-blocking facts from the exact f32 replay: the v2
  section-2 predicate covers only 9/183 (recipe) and 18/172 (aa2) affected pixels, and the v2
  section-3 recomputation gives 512+ ULP (> signed 128).

## Freeze-step tool (declared for the double-sign card)

The freeze step runs `scripts/authoring/author-scene-a-sphere-outline.ps1` (+ `.py` helpers),
an **offline authoring-class pinned-Python tool**: zero GPU, outside the product chain, never
referenced by product build/tests/cargo gates/product scripts. The wrapper re-checks the pinned
environment and the upstream identity before running and fails closed; it refuses to write under
`fixtures/`. This is stated explicitly in `window-plan.txt` section 7 and in the
`authoring-exception.txt` record.

Self-check result: rays 256/256 bitwise vs the S5 fixture, form A vs the frozen recipe
reference 747/747 bitwise, form B affected 183 px / max 73 (exactly attempt3), aa2 arm
(sub-sample 0 on the doubled grid) form A 745/745 bitwise and form B affected 172 px / max 91
(exactly attempt3), cam0 sphere pixels 0, two runs byte-identical.

## Status / blocker

Material only; the window is NOT opened and will not be opened under the signed criterion-v2
as it stands: the freeze step fails on both the list-coverage rule and the B recomputation rule
(see `outline-replay-findings.txt`). That is a criterion question - a list-predicate, bound or B
change needs a new criterion version and a fresh user signature - so Arch records the facts and
does not decide it.

Boundaries: zero GPU, no freeze, no window, criteria/references/code unchanged; the window only
opens after the user double-signs the plan and baseline, and after the criterion blocker is
decided.
