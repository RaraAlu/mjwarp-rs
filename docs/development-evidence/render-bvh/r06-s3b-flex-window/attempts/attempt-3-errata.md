# attempt 3 errata（floor 更正 + 判据定性更正）

任务：`2026-11-04-render-r06-s3b-flex-validate`（窗口执行卡，已 verified）。
本文件为**追加式更正**，不改动任何原记录：`attempts/attempt-3-monitor-pre-window.csv`、`attempts/attempt-3-guards-frozen.json`、`attempts/attempt-3-entity-flex.log` 与 `README.md` 原文一字不动。

## 1. floor 更正：11,682 → **11,688 MiB**

attempt 3 窗前三点的原始采样（`attempts/attempt-3-monitor-pre-window.csv`，08:09:11）：

| 点 | free MiB | util | temp |
| --- | --- | --- | --- |
| 1 | 14764 | 2% | 28°C |
| 2 | 14765 | 2% | 28°C |
| 3 | 14760 | 2% | 28°C |

- `min_free = 14,760`；`floor = max(11,618, min_free − 3,072) = 14,760 − 3,072 = **11,688 MiB**`。
- 原记录（README §7ter）写的 **11,682** 是按 attempt 1 三点（min 14,754 − 3,072）算出的值，被误带入 attempt 3；11,618 只是 Chief 留痕下限（14,690 − 3,072），不是本窗生效值。
- **结论不变**：monitor `free_min = 14,766 ≥ 11,688`，floor 未破。更正只影响留痕算术，不改变任何门禁/判据结果。

## 2. 判据定性更正

原记录只写「VALID_ROUND 判据性红」。按 2026-11-04 诊断证据（`2026-11-04-render-r06-s3b-cam1-rgb-diagnosis`，提交 `348da9fd`），更正为：

> **attempt 3 是 VALID_ROUND，但判据面被 harness D5 相机位姿来源缺陷污染，不作为产品结论。**

- cam1 `rgb max_byte_diff=255` 的根因是**用例输入**缺陷：参考用上游 data 位姿（`d.cam_xpos/d.cam_xmat`）渲染，用例却喂 model `cam_pos0/cam_mat0`；`cam-track`/`cam-flex` 为 mode 3 TARGETBODY，model `cam_mat0` 是单位阵，朝向差 **85.517° / 75.562°**。
- cam0 位姿位级相等故通过，仅为弱正面数据点；cam1/cam2 判据**无效**；rgb/depth/seg 的 flex 数值**仍未评估**。
- **不宣称 flex 光栅化通过，也不宣称不通过**；不放宽任何判据/容差；参考 npz 与产品代码零改动。
- 下一步：D5 修复卡 `2026-11-04-render-r06-s3b-cam-pose-source-fix`（A 类：用例改读参考 data 位姿 + 参考补 `cam-poses.json` f32 位型 provenance + CPU 位级前置断言）→ guards 15→16 与重新真冻结（`2026-11-04-render-r06-s3b-flex-guards-pose-entry`）→ **attempt 4 需 Chief GO**，四前提照旧。

## 3. 边界

本 errata 不重跑、不改判据、不改参考、不改 `src/`；仅更正留痕口径与判据定性。GUARDS 15→16 的扩卡与真冻结不在本文件范围。

签署：Arch 2026-11-04。
