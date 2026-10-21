# G01门禁脚本同步

任务：`2026-10-22-g01-gate-script-sync`。
脚本过期非本任务引入：源自 `runtime/transfer/kernel.rs`、`resident.rs` 等历史拆分与后续测试重构。
本任务只同步精确计数、输出标记与 all-probes 的环境接线；不删除断言、不放宽数值容差、不改成模糊匹配。

## 同步项

| 项 | before | after | 语义证据 |
| --- | --- | --- | --- |
| adapter 过滤计数（3 个脚本） | 5 / 3 / 3 | 9 / 9 / 9 | `runtime::transfer::kernel::tests` 在 `kernel.rs` 拆分后匹配 `tests_abi`、`tests_adapter`、`tests_guards`、`tests_precision` 共 9 项；断言仍是精确相等 |
| 内侧运动标记 | `G01-geom-tendon kind=motion`（已不发射） | 解析 `NP03_DETAIL` 中 `MuJoCo3.12.0:inside-cylinder-motion` 记录 | 运动断言现由 `Audit("G01-geom-motion")` 的 NP03 审计输出承载；按 9 条原生记录与最大绝对误差统计，计数仍精确为 9 |
| 主 G01 的 resident 期望值 | 98 | 102 | 由 adapter 同步（+4 项）派生；resident 步骤仍以报告合计精确断言 |

## all-probes 环境接线

在 `scripts/test-windows-g01.ps1` 的 all-probes 段新增临时目录与报告路径，运行后还原：

- `MJWARP_APPLIED_FORCE_REPORT`、`MJWARP_APPLIED_FORCE_PUBLISH_REPORT`、`MJWARP_MUL_M_REPORT`、`MJWARP_RESIDENT_MIXED_REPORT`、`MJWARP_SMOOTH_FORCE_REPORT`。
- `MJWARP_PHYSICS_PTX_DIR`（独立 capture 目录）。

所有路径位于本次门禁运行目录下的临时子目录，断言未改动。

## 验证

| 轮次 | 内容 | 结果 |
| --- | --- | --- |
| attempt-01 | g01-full 第一次 | 261s，gpu_mem 5532→5756 MiB；failed at adapter 5 vs 9 |
| attempt-02 | g01-full 第二次 | 182s，gpu_mem 5521→5772 MiB；failed at 内侧运动 0 vs 9 |
| attempt-03 | 环境接线后 lib ignored（same flags as all-probes） | 110 passed / 10 failed / 0 ignored，114.76s；五个报告类探针全部通过 |

失败摘录见 `runs/`。

## 既有债务清单

1. `physics::gravity_compensation::tests::gpu_matches_independent_multi_body_reduction_certificate`
   冻结位证书差异 `left: 2, right: 0`。非本任务改动；基线判定另立 `2026-10-22-g01-numerics-audit`。
2. `MJWARP_PHYSICS_PTX_DIR` 双向语义冲突：RNE/mass 包装测试要求该目录存在且为空（再建子目录），
   recapture 测试要求该路径不存在（拒绝覆盖）。同一环境变量无法同时满足，属 G01 门禁设计债务。
3. 受 2 影响仍红：`mass_solve::tests_device_wrapper` 1 项、`rne` 4 项、`rne_bias` 1 项、`recapture` 3 项。

## 未验收项

- 未宣称 NP01 至 NP04、精度、GPU 性能、Linux 或 release。
- all-probes 仍未全绿；上述债务不在本任务内修复。
