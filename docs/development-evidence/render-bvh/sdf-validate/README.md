# U4 SDF validation window (`2026-10-31-render-sdf-validate`) — executed evidence

Executed under **criterion `render-sdf-criterion-v2.2`** (signed 2026-11-01T21:42:50+08:00);
window plan v4 + baseline v4 double-signed 2026-11-02T08:29:18+08:00; window run 2026-11-02
13:16–13:26 (+08:00, 12:00–14:00 idle band).

## Result

- **attempt5 = VALID ROUND**: `PRECHECK_OK` + `OUTLINE_LISTS_OK=2/2` + `GUARDS_OK=94/94` +
  `WINDOW_RUN_DONE`, EXIT=0; **9/9 entities exit 0** (body_wall 48.4 s).
- Regular pre-window monitor 60/60 (94 samples, util 1–64 %, free_min 12,435 MiB, no exception).
- `zero_rust_compile=True`, post-window exe recheck 9/9, outline lists pre/post 2/2 no drift,
  floor/increment not breached, residual processes 0.
- attempt1–4 are **INVALID rounds**, archived under `../sdf-validate-window/attempts/` and never
  reused; their freezes are void.

## Freeze identity

- guards sha256 `9e000bfc1f8438a4bd6fd5f8c8ccf214c2ab43aee7c1ed1fce78ed51e7646c32` / 37,000 B,
  aggregate `034c7551226e321fab83b0ec4ef92a2dd82e9077ee3f31e7ad88f9a5d3b305b9`,
  `criterion_version=render-sdf-criterion-v2.2`, 94 = 9 exe + 83 inputs + 2 lists; the frozen file
  is committed at `scripts/validate/render-bvh/guards-sdf-validate.json` and copied in
  `../sdf-validate-window/attempts/attempt5/guards-sdf-validate.json`.
- Lists: recipe `cfd1038a…` / 10,099 B / 183 rows / exact max 73 ULP; aa2 `14893fc6…` / 9,509 B /
  172 rows / exact max 91 ULP; camera 0 has 0 sphere pixels.

## QA verdict

`VALID ROUND, releasable; 0 blocker / 0 fix / 2 observations (both addressed)` — independent
recomputation at **2026-11-02T16:13:11+08:00** (PowerShell 5.1): guards sha `9e000bfc…`, 94
entries 0 missing / 0 mismatch, aggregate `034c7551…`, docs copy vs live 30/30 consistent, monitor
verified as real 60/60, 9 entity criteria all correct. See `receipt.json` `/qa_verdict`.

## Release basis and boundaries

- Release basis = **`render_sdf_reference` bitwise** match under criterion v2.2 §1 + the **scene-a
  exact-difference list exemption** (183/172; outside-list bitwise; inside ≤ 128 ULP; B=128 from the
  exact replay). attempt1–4 are void rounds retained for history only.
- C-class evidence (guards/manifest/unchanged-path checks) is integrity evidence only and must not
  be used alone as a pass basis.
- Not claimed: 上游等价 / 精度达标 / 渲染数值验收通过.
- The window changed no criterion numeric values, lists, references, fixtures, product src or
  kernel identity.

## Pointers

- `receipt.json` — full record (criterion lineage, attempt history, attempt5 numbers, guards,
  lists, QA verdict, evidence index, boundaries).
- attempt5 evidence: `../sdf-validate-window/attempts/attempt5/` (facts, guard log/aggregate,
  sha-guards, samples, entity results, monitor CSV, outline outputs, 9 entity logs, manifest).
- criterion: `../sdf-criterion-v2.2/`; window material: `../sdf-validate-window-material-v4/`.
- RCA: `materials/reference-conflict-rca.md`.
