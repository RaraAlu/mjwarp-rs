# U4 attempt4 window material v3

Task: `2026-11-01-render-sdf-attempt4-material-v3` (Arch; offline, zero GPU)
Executed task: `2026-10-31-render-sdf-validate` (GPU full load; user double-sign required)

Material for the attempt4 window under the signed `render-sdf-criterion-v2.1`
(2026-11-01T02:42:29+08:00, B=128 ULP, exact-difference list exemption).

- `window-plan.txt` — full plan: three-question header, criterion v2.1 (bitwise SDF entity + exact-difference
  lists outside/inside rules), 9 entities, freeze = 94 entries (9 exe + 83 inputs + 2 lists) with the
  authoring tool step, conditional monitor protocol, preparation checklist, baseline rule, freeze-step tool
  declaration and env-gated body consumption, boundaries.
- `baseline-pre-window-v3.txt` — pre-window GPU baseline (three read-only nvidia-smi points, floor =
  min(free) - 3072 MiB) and the window-time re-sample rule; measured immediately before signing.
- `receipt.json` — machine-readable summary (criterion, entities, freeze counts/fields, baseline, blockers).

Prerequisites landed before this material: U1b redo, a-order refix (kernel identity 10070226../7782aeb1..
/268,589), criterion v2.1 signed and recorded (`sdf-criterion-v2.1/`), authoring tool contract
(`sdf-outline-tool-v2.1/`), env-gated test support (`sdf-outline-test-support/`), freeze/runner extension
(`sdf-freeze-runner-v2.1/`). The previous v2 material is kept as history; this v3 material supersedes it
for attempt4.

Boundaries: zero GPU, no freeze, no window; the window only opens after the user double-signs this plan
and the baseline. Criteria values, references, fixtures, product src and kernel identity are unchanged.
