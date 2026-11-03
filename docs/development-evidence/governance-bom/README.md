# 证据/fixture 的 UTF-8 BOM 审计（只读，不改历史字节）

任务：`2026-11-04-governance-evidence-bom-audit`（QA）。工具：`scripts/validate/check-evidence-bom.ps1`（PS 5.1、ASCII-only、**只读**）。

## 1. 政策

- **`.ps1` 的 BOM 有意保留**：Windows PowerShell 5.1 读「UTF-8 无 BOM + 中文」脚本会按 ANSI 解码而解析失败（先例 `2026-11-04-prepare-scripts-bom-hardening-np02` 正是给两个扫描脚本加 BOM）。审计对 `.ps1` 只报 `INFO`，**不得建议去掉**。
- **其它文本证据（`.txt/.log/.json/.md/.patch/.tsv/.cpp/.rs` 等）一律不带 BOM**：BOM 会让「按行/按首字节」解析器误判（典型：`manifest.txt` 首字节必须是 `#`，有了 BOM 就变成 `EF`）。
- **绝不重写历史文件**：改证据字节属「需用户签」范围；本卡只审计、只登记、只给新文件规范。
- **新写/改写的证据文件不得带 BOM**（尤其 `manifest.txt` 第 1 字节必须是 `#`）。

## 2. 实测基线（2026-11-04，`docs/evidence/` + `docs/development-evidence/` + `fixtures/`）

| 项 | 数量 |
| --- | --- |
| 扫描文件 | 13,316 |
| 带 BOM 文件合计 | **145** |
| ├ 其中 `.ps1`（INFO，有意保留） | 16 |
| └ 其中非 `.ps1`（WARN，历史存量，不重写） | 129 |
| `manifest.txt` 首字节非 `#`（WARN） | 4 |

按扩展名：

| 扩展名 | 数量 | 处理 |
| --- | --- | --- |
| `.cpp` | 1 | WARN（历史存量） |
| `.json` | 5 | WARN（历史存量） |
| `.log` | 56 | WARN（历史存量） |
| `.md` | 2 | WARN（历史存量） |
| `.patch` | 6 | WARN（历史存量） |
| `.ps1` | 16 | INFO（PS 5.1 需要） |
| `.rs` | 1 | WARN（历史存量） |
| `.tsv` | 1 | WARN（历史存量） |
| `.txt` | 57 | WARN（历史存量） |

**基线漂移记录**：本卡登记时基线为 **145**（WARN 129 / INFO 16）。相对于 S3b② 材料里记录的 144（128/16）多出的 1 个是 `docs/development-evidence/render-bvh/r06-s3b-flex-harness/compile.log`（提交 `0a3058ee`，S3b③ 采集/harness 新证据；加入后基线由 144 变 145（非 .ps1 WARN 128→129））。按政策「新文件不得带 BOM」，建议该文件属主另行剥 BOM（**不影响历史 144 个文件**）；剥离前 S3b② 开窗基线应按 **129+4** 执行，剥离后再回到 128。

## 3. `manifest.txt` 首字节断言（可粘贴）

```powershell
# manifest 第 1 字节必须是 '#'（0x23）；带 UTF-8 BOM 时为 0xEF，断言会失败
$bytes = [System.IO.File]::ReadAllBytes('docs/development-evidence/<dir>/manifest.txt')
if ($bytes[0] -ne 0x23) { throw 'manifest 首字节不是 #（检查是否被写成 UTF-8 BOM）' }
```

当前不满足的历史文件（只登记，不重写；新文件必须满足）：

- `docs/development-evidence/mesh-polygon-gt16-host-diagnostics-check/manifest.txt`（带 BOM）
- `docs/development-evidence/np01-field-tables-b1-b2a-check/manifest.txt`（带 BOM）
- `docs/development-evidence/np01-field-tables-b1b-check/manifest.txt`（带 BOM）
- `docs/development-evidence/render-bvh/r05-summary/manifest.txt`（无 BOM 但首字节不是 `#`，该目录使用目录相对路径且无表头）

## 4. 用法与接入口径

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/check-evidence-bom.ps1
# 需要严格模式（出现 WARN 即失败）：加 -Strict
```

- 输出：每行 `WARN  <path>` / `INFO  <path>`，末尾给 `BOM files total=`、按扩展名计数、`RESULT:`；**本脚本不写任何文件**（含不写日志）。
- 接进 S3b② 开窗前检查项：跑完 EOL 规范化（`scripts/validate/normalize-evidence-eol.ps1`，先 `-DryRun`）后、开窗材料冻结前跑本脚本；**若 `.ps1` 以外的 WARN 数超过本节基线（129）或出现新的 manifest 违规，就停止开窗并报 Chief**。
- 证据写入者自查：新建 `.txt/.log/.json/.md/.patch/.tsv` 一律 UTF-8 无 BOM；`manifest.txt` 首行以 `#` 开头。

## 5. 存量清单（只登记，不改动）

### `.cpp` — 1 个 WARN（历史存量）

- `docs/development-evidence/g03-mathdx-alignment/tools/oracle.cpp`

### `.json` — 5 个 WARN（历史存量）

- `docs/development-evidence/g01-new-tests-wiring/checks.json`
- `docs/development-evidence/g01-new-tests-wiring/env-wiring.json`
- `docs/development-evidence/g01-new-tests-wiring/exclusions.json`
- `docs/development-evidence/g10-p5-gjk-epa-independent-check/delivered-unit-gate-735abaf.json`
- `docs/development-evidence/layers-m3-sparse-mass-copy/kernel-comparison.json`

### `.log` — 56 个 WARN（历史存量）

- `docs/development-evidence/g01-gate-script-sync/runs/attempt-01-g01-full.log`
- `docs/development-evidence/g01-gate-script-sync/runs/attempt-02-g01-full.log`
- `docs/development-evidence/g01-gate-script-sync/runs/attempt-03-all-probes.log`
- `docs/development-evidence/g10-p5-branch-closure-capture/logs/capture-attempt1-compare.log`
- `docs/development-evidence/g10-p5-branch-closure-capture/logs/capture-attempt1-guard.log`
- `docs/development-evidence/g10-p5-branch-closure-capture/logs/capture-attempt1-run-a.log`
- `docs/development-evidence/g10-p5-branch-closure-capture/logs/capture-attempt1-run-b.log`
- `docs/development-evidence/g10-p5-fixtures-independent-check/logs/01-build-312.log`
- `docs/development-evidence/g10-p5-fixtures-independent-check/logs/02-dump-315.log`
- `docs/development-evidence/g10-p5-fixtures-independent-check/logs/03-parity-compare.log`
- `docs/development-evidence/g10-p5-fixtures-independent-check/logs/04-audit-existing.log`
- `docs/development-evidence/g10-p5-fixtures-independent-check/logs/05-verify-references.log`
- `docs/development-evidence/g10-p5-fixtures-independent-check/logs/07-recapture-diff.log`
- `docs/development-evidence/g10-p5-fixtures-independent-check/logs/08-evidence-hash.log`
- `docs/development-evidence/g10-p5-fixtures/gpu-run.log`
- `docs/development-evidence/g10-p5-gjk-epa-independent-check/unit-full.log`
- `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/device-logic-intermediates-after-fix.log`
- `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/device-logic-intermediates.log`
- `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/host-intermediates.log`
- `docs/development-evidence/g10-p5-model-fields-independent-check/parity-test.log`
- `docs/development-evidence/g10-p5-model-fields-independent-check/unit-full.log`
- `docs/development-evidence/layers-m3/runs/work-20-gpu-tests.log`
- `docs/development-evidence/layers-m3/runs/work-21-gpu-tests.log`
- `docs/development-evidence/layers-m3/runs/work-22-gpu-tests.log`
- `docs/development-evidence/layers-m3/runs/work-23-gpu-tests.log`
- `docs/development-evidence/layers-m3/runs/work-24-gpu-tests.log`
- `docs/development-evidence/mesh-multicontact-m0a-fix-mesh-poly-shapes/logs/clippy-cuda.log`
- `docs/development-evidence/mesh-multicontact-m0a-fix-mesh-poly-shapes/logs/clippy.log`
- `docs/development-evidence/mesh-multicontact-m0a-fix-mesh-poly-shapes/logs/green-new-rule.log`
- `docs/development-evidence/mesh-multicontact-m0a-fix-mesh-poly-shapes/logs/nvrtc-smoke.log`
- `docs/development-evidence/mesh-multicontact-m0a-fix-mesh-poly-shapes/logs/red-old-rule.log`
- `docs/development-evidence/render-bvh/independent-check/gpu-after.log`
- `docs/development-evidence/render-bvh/independent-check/gpu-before.log`
- `docs/development-evidence/render-bvh/perf/gpu-run.log`
- `docs/development-evidence/render-bvh/perf/regression.log`
- `docs/development-evidence/render-bvh/perf/repeat-run.log`
- `docs/development-evidence/render-bvh/r06-s3b-flex-harness/compile.log`
- `docs/development-evidence/render-bvh/s2b-independent-check/product-gpu-test.log`
- `docs/development-evidence/render-bvh/s3-independent-check/product-gpu-test.log`
- `docs/development-evidence/render-bvh/s4-independent-check/product-gpu-test.log`
- `docs/development-evidence/render-bvh/s5-independent-check/product-gpu-test.log`
- `docs/development-evidence/render-bvh/s6b-independent-check/product-gpu-test.log`
- `docs/development-evidence/render-bvh/s7a-independent-check/gpu-roundtrip.log`
- `docs/development-evidence/render-bvh/s7a-independent-check/w15-test.log`
- `docs/development-evidence/render-bvh/s7b1-independent-check/gpu-test.log`
- `docs/development-evidence/render-bvh/s7b1-independent-check/s6b-regression.log`
- `docs/development-evidence/render-bvh/s8-e2e/gpu-run.log`
- `docs/development-evidence/render-bvh/s8-e2e/regression.log`
- `docs/development-evidence/t4-source-hash-repin/runs/run-01-repin-tests.log`
- `docs/evidence/launcher-baseline/validation/adapter-gpu-tests.log`
- `docs/evidence/launcher-baseline/validation/clippy.log`
- `docs/evidence/launcher-baseline/validation/diff-scope.log`
- `docs/evidence/launcher-baseline/validation/fmt.log`
- `docs/evidence/launcher-baseline/validation/gpu-baseline-test.log`
- `docs/evidence/launcher-baseline/validation/host-tests.log`
- `docs/evidence/launcher-baseline/validation/kernel-host-tests.log`

### `.md` — 2 个 WARN（历史存量）

- `docs/development-evidence/mesh-multicontact-m1-observation-governance/README.md`
- `docs/development-evidence/np01-field-tables-b1b-check/README.md`

### `.patch` — 6 个 WARN（历史存量）

- `docs/development-evidence/evidence-eol-hygiene/materials/attributes-diff.patch`
- `docs/development-evidence/g10-p5-mesh-cap20/materials/src-diff.patch`
- `docs/development-evidence/render-bvh/sdf-host-unlock-redo/materials/src-diff.patch`
- `docs/development-evidence/render-bvh/sdf-sphere-a-order-refix/materials/src-diff.patch`
- `docs/development-evidence/render-bvh/sphere-dot-a-order-fix/materials/src-diff.patch`
- `docs/development-evidence/render-bvh/sphere-dot-a-order-revert/materials/src-diff.patch`

### `.ps1` — 16 个 INFO（`.ps1`，保留）

- `docs/development-evidence/evidence-bin-eol-hygiene/verify-bin-checkout.ps1`
- `docs/development-evidence/g10-p5-acceptance-map/perf/build_perf_probe.ps1`
- `docs/development-evidence/g10-p5-acceptance-map/perf/run_window.ps1`
- `docs/development-evidence/g10-p5-acceptance-map/perf/split_perf_probe.ps1`
- `docs/development-evidence/g10-p5-ccd-overflow-bit-fix/build_probes.ps1`
- `docs/development-evidence/g10-p5-ccd-overflow-bit-fix/run_window.ps1`
- `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/build_probe.ps1`
- `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/run_window.ps1`
- `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/run_window_phase2.ps1`
- `docs/development-evidence/g10-p5-multicontact-cuda/run_window.ps1`
- `docs/development-evidence/g10-p5-nvrtc-product-launcher/run_window.ps1`
- `docs/development-evidence/g10-p5-primitive-narrowphase-cuda/run_window.ps1`
- `docs/development-evidence/g10-p5-sdf-descent-sign-branch-test/pre_fix_reverse_check.ps1`
- `docs/development-evidence/g10-p5-sdf-primitives-sign/pre_fix_reverse_check.ps1`
- `docs/development-evidence/render-bvh/sdf-host-unlock-redo-check/materials/selfcheck.ps1`
- `docs/development-evidence/render-bvh/sdf-sphere-a-order-refix-check/materials/selfcheck.ps1`

### `.rs` — 1 个 WARN（历史存量）

- `docs/development-evidence/g10-p5-gjk-epa-independent-check/harness/main.rs`

### `.tsv` — 1 个 WARN（历史存量）

- `docs/development-evidence/render-bvh-precheck/upstream-references.tsv`

### `.txt` — 57 个 WARN（历史存量）

- `docs/development-evidence/evidence-eol-hygiene/logs/verify-eol.txt`
- `docs/development-evidence/g01-new-tests-wiring/all-probes-summary.txt`
- `docs/development-evidence/g10-p5-ccd-overflow-bit-fix/window-log.txt`
- `docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-gpu/build-recipe.txt`
- `docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-gpu/window-probe-rerun-log.txt`
- `docs/development-evidence/g10-p5-mesh-cap20/logs/no-run.txt`
- `docs/development-evidence/g10-p5-mesh-cap20/logs/nvrtc-identity.txt`
- `docs/development-evidence/g10-p5-mesh-cap20/materials/validate.txt`
- `docs/development-evidence/g10-p5-nvrtc-product-launcher-independent-check/window-log.txt`
- `docs/development-evidence/g10-p5-nvrtc-product-launcher-independent-check/window-log2.txt`
- `docs/development-evidence/g10-p5-primitive-narrowphase-cuda-independent-check/window-log.txt`
- `docs/development-evidence/mesh-polygon-gt16-host-diagnostics-check/manifest.txt`
- `docs/development-evidence/np01-field-tables-b1-b2a-check/manifest.txt`
- `docs/development-evidence/np01-field-tables-b1b-check/manifest.txt`
- `docs/development-evidence/render-bvh/perf/nvidia-smi-samples.txt`
- `docs/development-evidence/render-bvh/r06-flex-datasource/native-verify.txt`
- `docs/development-evidence/render-bvh/r06-flex-datasource/repin-after-pass.txt`
- `docs/development-evidence/render-bvh/r06-flex-datasource/repin-before-fail-identity.txt`
- `docs/development-evidence/render-bvh/r06-flex-datasource/repin-before-fail.txt`
- `docs/development-evidence/render-bvh/r06-s2c-flex-shell/native-smoke.txt`
- `docs/development-evidence/render-bvh/r06-s2c-flex-shell/native-topology-tests.txt`
- `docs/development-evidence/render-bvh/r06-s2c-flex-shell/repin-after-pass.txt`
- `docs/development-evidence/render-bvh/r06-s2c-flex-shell/repin-before-fail.txt`
- `docs/development-evidence/render-bvh/r06-s3a-flex-device/unit-tests.txt`
- `docs/development-evidence/render-bvh/r06-s3pre-flexhost-wiring/flex-host-unit-tests.txt`
- `docs/development-evidence/render-bvh/s2b-independent-check/ptx-fma-summary.txt`
- `docs/development-evidence/render-bvh/s2b-independent-check/unit-tests-summary.txt`
- `docs/development-evidence/render-bvh/s3-independent-check/gpu-window.txt`
- `docs/development-evidence/render-bvh/s3-independent-check/ptx-fma-summary.txt`
- `docs/development-evidence/render-bvh/s3-independent-check/unit-tests-summary.txt`
- `docs/development-evidence/render-bvh/s4-independent-check/gpu-window.txt`
- `docs/development-evidence/render-bvh/s4-independent-check/ptx-fma-summary.txt`
- `docs/development-evidence/render-bvh/s4-independent-check/unit-tests-summary.txt`
- `docs/development-evidence/render-bvh/s5-independent-check/gpu-window.txt`
- `docs/development-evidence/render-bvh/s5-independent-check/unit-tests-summary.txt`
- `docs/development-evidence/render-bvh/s6a-independent-check/gpu-window.txt`
- `docs/development-evidence/render-bvh/s6a-independent-check/no-gpu-proof.txt`
- `docs/development-evidence/render-bvh/s6a-independent-check/unit-tests-summary.txt`
- `docs/development-evidence/render-bvh/s6b-independent-check/gpu-window.txt`
- `docs/development-evidence/render-bvh/s6b-independent-check/unit-tests-summary.txt`
- `docs/development-evidence/render-bvh/s7a-independent-check/gpu-window.txt`
- `docs/development-evidence/render-bvh/s7a-independent-check/unit-tests-summary.txt`
- `docs/development-evidence/render-bvh/s7b1-independent-check/gpu-window.txt`
- `docs/development-evidence/render-bvh/s7b1-independent-check/unit-tests-summary.txt`
- `docs/development-evidence/render-bvh/sdf-criterion-v2/materials/per-entity-affected.txt`
- `docs/development-evidence/render-bvh/sdf-host-unlock-redo/logs/no-run.txt`
- `docs/development-evidence/render-bvh/sdf-host-unlock-redo/materials/kernel-identity-unchanged.txt`
- `docs/development-evidence/render-bvh/sdf-host-unlock-redo/materials/validate.txt`
- `docs/development-evidence/render-bvh/sdf-sphere-a-order-refix/logs/no-run.txt`
- `docs/development-evidence/render-bvh/sdf-sphere-a-order-refix/materials/kernel-identity.txt`
- `docs/development-evidence/render-bvh/sdf-sphere-a-order-refix/materials/validate.txt`
- `docs/development-evidence/render-bvh/sphere-dot-a-order-fix/logs/kernel-identity.txt`
- `docs/development-evidence/render-bvh/sphere-dot-a-order-fix/logs/no-run.txt`
- `docs/development-evidence/render-bvh/sphere-dot-a-order-fix/materials/validate.txt`
- `docs/development-evidence/render-bvh/sphere-dot-a-order-revert/logs/kernel-identity-three-generations.txt`
- `docs/development-evidence/render-bvh/sphere-dot-a-order-revert/logs/no-run.txt`
- `docs/development-evidence/render-bvh/sphere-dot-a-order-revert/materials/validate.txt`

## 6. 边界

- 零 GPU、零网络；只读扫描，**未修改任何文件字节**。
- 不重写 145 个历史文件（含 16 个 `.ps1` 保留项）；不宣称清理完成。
- 审计脚本自身为 ASCII-only，PS 5.1 `-File` 实跑通过。

