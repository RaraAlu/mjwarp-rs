# mesh polygon >16 顶点设备容量差距评估（backlog → 评估卡；未实现）

任务：`2026-10-27-mesh-polygon-gt16-device-gap`（Arch 2026-10-27 登记；2026-11-03 先建证据目录，随卡 promote 到 proposed）
性质：**差距评估 + 设计方案**。不改设备代码、不改判据、不开 GPU 窗口、不宣称任何覆盖。本目录先建立，避免 promote 后卡无落点。

## 1. 缺口事实

- 来源：S4 窗 U8 record-only（设备侧 mesh 多接触 count=1 vs 上游 4）、S2 branch-map M-09、S3 捕获参考 `upstream-mesh-limits-gpu.json`（sha `b6abb8fa…`）。
- 触发条件：mesh 面的 polygon 顶点数 > 16 时参与多接触（设备固定容量 `G5_MC_MESH_FACE_MAX=16`，`contact.cu` 的 face1/face2 scratch 按此分配）。
- 影响面：形状/配置、与 `degree>10` 的组合、对既有 M2/M3/S4 判据与证据的影响（是否追溯、是否需重采参考）。
- 上游实现：pinned `collision_convex.py` L1262-1289 / `types.py` L1661-1662 为动态容量；与设备固定容量的差异是本缺口的核心。

## 2. 待评估的两套方案（评估阶段产出，未选定）

| 方案 | 内容 | 代价/风险 |
| --- | --- | --- |
| A1 提高固定上限 | 16 → 20/32（改 `G5_MC_MESH_FACE_MAX`、`contact.cu` scratch、可能动 ABI/`DEVICE_SOURCES`） | 设备代码/判据变更，**需用户批准**；scratch/ABI 容量与内核身份（source/PTX sha）改变，历史证据影响面需盘点 |
| A2 动态容量 + 分派 | 按面顶点数动态分配/分派（对齐上游） | 实现复杂、scratch 管理成本高，需新参考与窗口 |

## 3. 评估交付要求（start 后落）

1. 差距评估：触发条件、影响面、对既有 M2/M3/S4 判据与证据的追溯结论。
2. 设计方案：至少两套（提高固定上限 / 动态容量 + 分派），列 ABI、scratch 容量、`DEVICE_SOURCES`、位级判据与参考重采的代价与风险。
3. 明确「改设备代码或判据需用户批准」；**不 start 实现、不改代码、不开窗口**。

## 4. 非声明

本目录当前只有评估范围与设计方向，**未做评估结论、未实现、未改设备代码、未开 GPU 窗口**；不得据此宣称任何覆盖。
