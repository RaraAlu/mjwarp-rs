# W8-04 CT：P8/P9/P10 准入契约核对（0 窗 CT，只读；零 GPU）

- 任务：`2026-11-11-w8-04-p8-p9-p10-ct`（owner Backend）；阶段：**P8/P9/P10**。
- 性质：**CT 契约核对**——伞卡 acceptance、兼容矩阵行、replication-plan 阶段段、依赖 verified 状态对齐；不生成平台证据、不宣称准入。

## 1. 伞卡

- `2026-10-11-roadmap-p8` status=backlog checks=0（acceptance 8 条）
- `2026-10-11-roadmap-p9` status=backlog checks=0（acceptance 8 条）
- `2026-10-11-roadmap-p10` status=backlog checks=0（acceptance 10 条）

## 2. 阶段卡与依赖 verified 状态

### P8

- 阶段级伞卡：`p8`=backlog/0；`p8-base`=backlog/0；`p8-entry`=backlog/0
- 阶段切片卡：verified **8/12**；未验收依赖 **3** 个

### P9

- 阶段级伞卡：`p9`=backlog/0；`p9-incremental`=backlog/0
- 阶段切片卡：verified **0/0**；未验收依赖 **1** 个

### P10

- 阶段级伞卡：`p10`=backlog/0
- 阶段切片卡：verified **2/5**；未验收依赖 **4** 个


## 3. 兼容矩阵行（24 行）

| 矩阵行 | 功能族 | 阶段 | 矩阵文件锚点 |
| --- | --- | --- | --- |
| A04 | 多流与事件 | P3/P10 | 第 419 行 |
| A05 | JIT或预编译缓存 | P1/P10 | 第 420 行 |
| A06 | 执行图 | P10 | 第 421 行 |
| A07 | 设备数据视图 | P8/P10 | 第 422 行 |
| A08 | 零拷贝优化 | P10 | 第 423 行 |
| A09 | 通用调用方接口 | P9/P10 | 第 424 行 |
| A10 | 记录与性能工具 | P10/P11 | 第 425 行 |
| G25 | 岛与休眠 | P7/P10 | 第 354 行 |
| G28 | 查询 | P7/P8 | 第 357 行 |
| M03 | 资产转换 | P2/P7/P8 | 第 374 行 |
| M05 | 模型上传 | P3/P10 | 第 376 行 |
| M07 | 状态交换 | P3/P9 | 第 378 行 |
| M09 | 参数更新 | P7/P10 | 第 380 行 |
| M10 | Rust回调 | P7/P10 | 第 381 行 |
| R01 | 渲染上下文 | P8 | 第 394 行 |
| R02 | RGB输出 | P8 | 第 395 行 |
| R03 | 深度输出 | P8 | 第 396 行 |
| R04 | 分割输出 | P8 | 第 397 行 |
| R05 | 网格与地形 | P8 | 第 398 行 |
| R06 | 柔性体渲染 | P8 | 第 399 行 |
| R07 | 高斯泼溅 | P8 | 第 400 行 |
| R08 | 异构多相机 | P8 | 第 401 行 |
| R09 | 光照与阴影 | P8 | 第 402 行 |
| R10 | 批量渲染调度 | P8/P10 | 第 403 行 |

replication-plan 阶段段：{"274": "P8 阶段行：批量图像、BVH、设备视图；前置 P2/P3，柔性项需 P7", "275": "P9 阶段行：模型、控制、状态调用契约；前置 P3–P8；交付=独立 Rust 调用方通过接口测试", "276": "P10 阶段行：图重放、缓存失效、性能；前置 P6–P9；交付=图执行等价并提供实测报告"}

## 4. 缺口（6 项）

| id | 类别 | 说明 |
| --- | --- | --- |
| G-NP | NP 门禁 | NP01/NP02 全局未验收 → P8/P9/P10 AD 的 NP 前置不满足 |
| G-W4 | W4 前置 | P8 的 W4 伞卡链（p8-entry/p8-base/p8/r10-p8/r10-p10/m03-p8/m10-p10 全 checks=0）未验收；r07-p8 待 ABI → 片 B 不开窗 |
| G-2REF | 两参考门 | `2026-10-11-two-reference-gates` 为 verified 但 checks=0（无机器检查项）；p9-incremental depends_on 该卡 → P9 的接口测试证据仍缺，只能作为前置状态登记，不构成 P9 准入证据 |
| G-CHAIN | 前置链 | P9 前置 P3–P8、P10 前置 P6–P9 均未准入；P10 还依赖执行图/缓存（A05/A06）与 A09 调用方契约 |
| G-DEPS | 依赖未验收 | P10 组 g25-p10/m10-p10/r10-p10 proposed、a08/a10 相关卡未验收；板面未验收依赖详见 contract.json |
| G-AD | 证据缺口 | 三阶段 AD 均未生成；需 CT 无缺口 + NP 门禁 + 前提阶段 + 窗口 |

**状态：等待 NP 门禁 + W4/前置阶段 + two-reference-gates 实体证据** —— P8 受 W4/r07-p8/NP 阻塞；P9 的 two-reference-gates verified 但 checks=0；P10 前置链未满足 → 均不排窗。

## 5. 结论

CT 完成：P8/P9/P10 伞卡 acceptance、矩阵行（去重）、依赖与前置链已对齐；two-reference-gates 状态如实记为「verified 但 checks=0、无机器检查项」；缺口 6 项。AD 等待 NP 门禁与前置阶段，不宣称准入。

## 6. 验证

- selfcheck（`logs/selfcheck.txt`）：contract 字段/矩阵行/缺口计数/`admission_claim=false`、manifest 4 文件自核、
  policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录 + `.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 7. 边界与非声明

- 零 GPU、零窗；只写本目录 + `.normify/`；不改卡面 acceptance/判据/参考/容差/矩阵/计划文件。
- 不宣称任何阶段准入通过、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布。
