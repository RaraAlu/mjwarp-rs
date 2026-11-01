# U4 attempt5 window material v4 (criterion v2.2)

Material for the attempt5 validation window under the user-signed
`render-sdf-criterion-v2.2` (signed 2026-11-01T21:42:50+08:00, `approved_by=user via Chief`).
**No GPU is used and no window is opened by this material**; the plan and the re-sampled
baseline must be double-signed before the window.

## Contents

| file | content |
| --- | --- |
| `window-plan.txt` | the attempt5 window plan (criterion v2.2, attempt=5, full-parameter commands, conditional monitor, checklist) |
| `baseline-pre-window-v4.txt` | three read-only nvidia-smi points; min free 14461 MiB; floor = 11389 MiB; desktop-util note |
| `receipt.json` | `/task_id`, criterion/clauses, attempt4 INVALID reference, frozen attempt5 guards identity, lists, baseline, script identities, commands, boundaries |
| `manifest.txt` | self-consistent sha256/bytes list of the files above |

## §3d attempt4 INVALID (what this material replaces)

attempt4 (criterion v2.1) ended in `INVALID_ROUND exit10`: 7/9 entities exit 0;
`render_kernel_depth_seg` exit101 and `render_kernel_flags` exit101. Both failures are a
**v2.1 acceptance-surface coverage gap**, not a product regression and not an execution error:

1. the `culling_off` flag-variant arm kept the raw comparison (`variant_depth_diff` exempted only
   `aa2`) → cam1 `183 px / 73 ULP`, first pixel `28969=(169,120)`;
2. the S0b fast-math cross-recipe comparison kept raw → product-vs-S0b cam1
   `4247 / 73 ULP / beyond-4 = 149`, and all 149 pixels are inside the 183 frozen list.

The attempt4 freeze (`13ba1bee../37,000 B`) is **void** and must never be reused. The acceptance
surface was changed only by the separately user-signed criterion v2.2 (clause 1 variant-arm list
reuse + zero-GPU hardened assertion; clause 2 S0b record-only + ">4 ULP ⊆ frozen list"
fail-closed). The complete attempt4 record is archived in
`docs/development-evidence/render-bvh/sdf-validate-window/attempts/attempt4/`.

## attempt5 freeze state (offline, zero GPU)

- Fresh freeze from a fresh compile: **94 = 9 exe + 83 inputs + 2 lists**, all 9 entity exe
  hashes changed vs attempt4.
- `guards-sdf-validate.json`: sha256 `9e000bfc1f8438a4bd6fd5f8c8ccf214c2ab43aee7c1ed1fce78ed51e7646c32`,
  37,000 B, aggregate `034c7551226e321fab83b0ec4ef92a2dd82e9077ee3f31e7ad88f9a5d3b305b9`,
  `criterion_version=render-sdf-criterion-v2.2`, `bound_ulp=128`.
- `-GuardsOnly` with full parameters: `OUTLINE_LISTS_OK=2/2` + `GUARDS_OK=94/94` +
  `GUARDS_ONLY_DONE`, EXIT=0.
- Lists unchanged and byte-identical to the signed prototype: recipe
  `cfd1038a../10,099 B/183` (exact max 73), aa2 `14893fc6../9,509 B/172` (exact max 91).

## Not claimed

This material is a plan and an offline freeze record. It makes no acceptance, parity or
precision statement; U4 stays not passed until attempt5 runs inside a double-signed window.
## §3e attempt5 freeze errata (zero body, zero GPU)

The first attempt=5 freeze wrote `criterion_version=render-sdf-criterion-v2.1` (the freeze default
`-CriterionVersion`, the authoring `CRITERION_VERSION` and the
`tests/support/render_bvh` outline-lists.json version check were still at v2.1). It was discarded
**before any body step** (zero GPU, no entity ran, no body facts), the three constants were updated
to v2.2 (list file headers keep the signed prototype v2.1 bytes) and the freeze re-ran. The list
sha256 values are unchanged (`cfd1038a../10,099 B/183`, `14893fc6../9,509 B/172`); no criterion
value, list, reference, fixture or kernel identity changed. The registered attempt5 guards are
`9e000bfc../37,000 B`, aggregate `034c7551..`, `criterion_version=render-sdf-criterion-v2.2`.
## §3f evidence cross-references and window build rule

- The header references to `sdf-outline-tool-v2.1/` and `sdf-freeze-runner-v2.1/` are the
  **v2.1-time** identities; the CURRENT scripts are listed in `receipt.json` `script_identities`
  (authoring helper `5308e4e4../18,613 B`, freeze `21fe74a5../12,906 B`, runner `2abb8a77../22,910 B`).
- `exe-identities.txt` is the authoritative list of the 9 frozen entity exe sha256/bytes, taken
  from the frozen guards file; `target/v22-new-exe-sha.txt` is a patch-time snapshot only.
- The freeze ran `cargo test --no-run --message-format=json` inside the freeze script
  (`freeze-tool-run.txt`, first line) after an explicit
  `cargo test --locked --features cuda-probe --no-run` returned EXIT=0; exe mtimes 21:50:xx bind
  the freeze to the current source.
- **Window build rule**: after the freeze only `--features cuda-probe` build/test is allowed; the
  default feature set is forbidden inside the window (it recompiles the test crates and replaces
  the frozen exe sha).
- Unit-gate caliber: the `735/0/137` numbers in the slice cards were measured as
  `cargo test --lib --locked --features cuda-probe`; `gates.toml` defines `gate.unit` as
  `cargo test --locked` (default features), which was NOT run for these evidence-only slices and
  the numbers are not quoted as gate.unit results.
