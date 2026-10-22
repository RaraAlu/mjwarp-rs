# 交接：G01 双模式验收（QA）

来源任务：`2026-10-22-g01-all-probes-env-wiring`（Backend 收尾）。G01 双模式验收由 QA 接手。

## 一页状态

- 源码身份：repin-9（`f1faa15`）已收口；实现提交 `d5b02d5`（矩阵链对齐）。
- env 接线：`scripts/test-windows-g01.ps1` 已接线 all-probes 环境（见 `env-wiring.json`）。
- 本轮完整双模式实测（同一脚本参数）：
  - debug：fmt 0 / native 27 / host-msvc 709 / g01 7 / native-g01 2 / native-only-g01 1 / resident 102 / **all-probes 339 过 1 败 79 行**。
  - release：同 debug，**all-probes 339 过 1 败 79 行**。
  - 唯一失败：`computes_both_flags_for_nonzero_accelerations_without_old_bias_replay`（RNE 521 native-tolerance blocks；scalars=900288）。
- 结论口径：**不得声明 G01 首次全绿**；只能说「除 RNE 数值探针缺口外全绿」。
- 后续两项 proposed 债务：`2026-10-22-g02-rne-acc-native-tolerance-debt`（RNE 证据/归属）、`2026-10-22-t4-historical-capture-rebaseline`（历史捕获重基线）。

## 复跑命令

```
scripts/test-windows-g01.ps1 -AllFeatures -FullRegression
scripts/test-windows-g01.ps1 -AllFeatures -FullRegression -Release
```

需要 env：`MJWARP_MUJOCO_ROOT`/`MJWARP_MUJOCO_DLL`、CUDA 工具链（PATH/CUDA_PATH）。脚本自身会接好 all-probes 的 T4/gravity/mixed/machine/sparse/collision/native 环境。
迭代时只跑受影响 target（如 `cargo +1.99 test --locked --all-features --test <target> -- --ignored`），不要每轮全脚本。

## 判读

1. 阶段日志：`<run>/<stage>.log`；脚本开头打印各阶段 `passed=…`。
2. all-probes：脚本已加 `cargo --no-fail-fast`（`--` 之前），因此会跑完所有 target。判「除 RNE 外全绿」= `all-probes.log` 汇总 `passed=339 failed=1` 且失败集合恰为 RNE 那一条。
3. 排除清单：`<run>/probe-reports/exclusions.json`，三组单列可审计：
   - W-D5 原 3 项（保持不变，用户 2026-10-22 裁决 A）；
   - 历史捕获身份 6 项（G03 矩阵链对齐有意变更 f32 算序；历史 BASE/模板先于该变更；重生成需被排除重采集工具与冻结捕获流水线）；
   - env/外部产物未接线 1 项（face machine-code，需既有 face-operator 探针重生成 `target/t4-face-operators`）。
   不得把它们视为静默跳过；复跑时计数会相应变化。
4. 瞬态锁：若 all-probes 出现 `os error 32`，脚本会用独立捕获目录重试一次并在 `env-wiring.json` 记 `transient_file_lock_retry`；这不是断言放宽。
5. release 曾出现 `gpu_shared_resident_ellipsoid_matches_independent_reference` 失败，已确认是 physics-precision-evidence 文件写入的 os error 32 瞬态锁，不是 release FMA/精度差异；复跑未复现。若稳定复现再按独立问题登记。

## 建议的验收文档更新（现成文本）

在 `docs/three-layer-acceptance.md` 增「历史捕获身份债务」一节：列 6 项 audit、原因（见上）、证据目录 `target/g01-acceptance/a947ee919daf48699d47016d684ebe70`（debug）与 `target/g01-acceptance/3afadc22c5b74e9b8e6576aafdcad231`（release）、恢复条件（`2026-10-22-t4-historical-capture-rebaseline` 完成重采集/重基线后解除排除）。同时更新 G01 现状为「除 RNE 数值探针缺口外全绿（debug+release 同数）」，W-D4/W-D5/W-D3 保持现状。
