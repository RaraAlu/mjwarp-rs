# NP01 E 指标定稿（A2）：同码跨后端核对

任务：`2026-11-05-precision-np01-e-criterion-a2`（owner Arch）。日期：2026-11-05。
**边界：只落定义与核对方法；不加新数值、不改已签 atol、不改任何已 verified 证据语义；零 GPU。**

## 1. 签名记录

- **日期**：2026-11-05
- **签署人**：用户
- **选项**：**①「同码跨后端核对」**（签名级）
- **转达**：Chief 转达（仓内无独立用户消息快照；与既有签名卡同口径处理）
- **落点**：`docs/numerical-precision-contract.md` 新增「### E指标定稿（NP01，用户2026-11-05拍板选项①）」，紧跟「接触字段容差分档」之后。

## 2. 口径（逐条写死）

1. **判据写法**：每项（路径 × 字段）写成「**实测 worst ≤ 已签 atol**」的核对。
2. **不用 E 比值作判据**：以同码跨后端基线作参考时 `E_mjwarp ≡ 0`，`1.05 × E_mjwarp` 退化，E 比值恒成立、等于没有判据。
3. **范围**：CONVEX / SDF 两路径 × 字段块 **dist / pos / frame**（**normal 并入 frame**）。
4. **样本**：同码跨后端 **97 个标量**（QA 归档 `g6_tolerance_candidates.same_code_backend_baseline`）。
5. **明确不宣称 native 口径**：契约 native 侧「仅诊断」语义保留（2026-10-25 选项 B 不变），本口径不用于 native 侧放行。
6. **不加新数值、不改已签 atol**：既有 atol/rtol 表与实测 worst 数值保持原样。
7. **新旧措辞规则**：自 A2 起后续收口写「NP01 已核对；E 指标用户 2026-11-05 已签（同码跨后端核对，A2 `<提交号>`）」；本卡之前已 verified 的历史 summary 中「E 指标 1 项待用户签」保留为当时快照，不回改。

## 3. 依据指针（只引用，不改写数字）

- `docs/development-evidence/np01-e-recompute/README.md` §三「候选①：同码跨后端（warp CPU vs warp CUDA f32）基线」：
  - 依据 QA 归档 `g6_tolerance_candidates.same_code_backend_baseline`（**97 个标量**）；
  - 6/6 项实测 worst ≤ 已签 atol，**最紧余量 1.80×**（frame/SDF）；
  - 该证据同时记录了退化理由与「定稿必须写成 atol 核对」的结论。
- 容差表来源：`docs/numerical-precision-contract.md` 接触字段容差分档（atol：dist CONVEX `1e-6` / SDF `2e-6`；pos CONVEX `1e-6` / SDF `1e-6`；frame·normal CONVEX `2e-6` / SDF `5e-5`；结构字段零容差）——**本卡不改这些数字**。

## 4. 契约改动范围

- **只新增**一个小节（定义 + 核对方法 + 措辞规则）；被删改既有行 = 0。
- 不触碰：`E_mjwarp`/`redline` 公式节、接触字段 native 侧选项 B 节、容差分档表、其它任何章节。

## 5. 非宣称

- 不宣称 native 口径通过；
- 不宣称 E 比值作为判据；
- 不放宽任何 atol/rtol，不新增数值；
- 不宣称 NP01 / NP03 / NP04 整族通过；
- 本卡不改变任何已 verified 证据的语义与数字。

## 6. 影响面

- **后续收口**：r01-p8 / R06 / G10 等后续收口证据的 NP01 措辞写「已签（A2）」，不再写「待签」（其中 G10 收口卡由同批 fix 更新）。
- **历史不回改**：A2 之前已 verified 的 summary 保持「待签」快照。
