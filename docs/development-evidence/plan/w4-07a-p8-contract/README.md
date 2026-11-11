# W4-07A P8 全部渲染与查询准入核对（CT）（0 窗 CT，只读；零 GPU）

- 任务：`2026-11-11-w4-07a-p8-contract`（owner Backend）；伞卡：`2026-10-11-roadmap-p8`（**status=backlog / checks=0**）。
- 性质：**CT 契约核对**——卡面 acceptance、兼容矩阵行、replication-plan 阶段段、依赖 verified 状态对齐；不生成平台证据、不宣称准入。

## 1. 卡面 acceptance（8 条）

- 核对适用切片与矩阵。
- 先完成范围内NP门禁。
- 保留双模式平台证据。
- 按适用范围检查M11。
- 按适用范围保存A11证据。
- 记录真实测试与未测项。
- 先核对NP01与NP02。
- 再验收适用NP03与NP04。

## 2. 依赖 verified 状态（4 项）

| 依赖 | 卡 id | status |
| --- | --- | --- |
| `base` | `2026-10-11-roadmap-p8-base` | backlog |
| `p8` | `2026-10-11-roadmap-m03-p8` | proposed |
| `p8` | `2026-10-11-roadmap-g28-p8` | verified |
| `closeout` | `2026-11-04-render-r06-flex-closeout` | verified |

## 3. 兼容矩阵行（15 行）

| 矩阵行 | 功能族 | 阶段 | 矩阵文件锚点 |
| --- | --- | --- | --- |
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
| G28 | 查询 | P7/P8 | 第 357 行 |
| M03 | 资产转换 | P2/P7/P8 | 第 374 行 |
| M11 | 错误状态 | 全阶段 | 第 382 行 |
| A11 | 双平台发布 | 全阶段/P11 | 第 426 行 |
| A07 | 设备数据视图 | P8/P10 | 第 422 行 |

replication-plan 阶段段：{"274": "P8 阶段行：批量图像、BVH、设备视图；前置 P2/P3，柔性项需 P7；交付=图像布局与完成事件正确"}

## 4. 缺口（7 项）

| id | 类别 | 说明 |
| --- | --- | --- |
| G-BASE | 前置未验收 | p8-base 伞卡 backlog/0 未验收（其 CT=W4-06A，r07-p8 仍 proposed）→ p8 全量准入前置不满足 |
| G-M03 | 前置未验收 | m03-p8 伞卡 proposed/0 未验收（其 CT=W4-02A，资产转换契约断言未落地、NP 门禁未过） |
| G-R07 | 渲染 ABI | r07-p8 proposed（待 ABI 用户签）→ R07 行不具备准入证据；W4-B0 不 waive |
| G-NP | NP 门禁 | NP01/NP02 未验收 → P8 全量准入的 NP 前置未满足 |
| G-AD | 证据缺口 | 双模式平台证据与 A11/M11 范围检查未做；本片只做 CT |
| G-G28 | 已验收前置 | g28-p8 verified（6 checks）、r06-flex-closeout verified 可作输入；但不足以替代 p8-base/m03-p8 前置 |
| G-CLAIM | 声明边界 | 即使全部前置齐，也只能写阶段准入；**不得**写「全部渲染已通过/等价/发布」（W4-07 卡面明示） |

**状态：等待前置验收 + NP 门禁 + ABI 裁决** —— p8-base（W4-06A）与 m03-p8（W4-02A）均未验收、r07-p8 待 ABI、NP01/NP02 未验收 → 不得准入。

## 5. 结论

CT 完成：p8 的 4 项依赖中仅 g28-p8 与 r06-flex-closeout verified；p8-base 与 m03-p8 均未验收，r07-p8 待 ABI、NP 门禁未过；缺口 7 项，不构成 P8 全量准入。

## 6. 验证

- selfcheck（`logs/selfcheck.txt`）：contract 字段/矩阵行/缺口计数/`admission_claim=false`、manifest 4 文件自核、
  policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录 + `.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 7. 边界与非声明

- 零 GPU、零窗；只写本目录 + `.normify/`；不改卡面 acceptance/判据/参考/容差/矩阵/计划文件。
- 不宣称任何阶段准入通过、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布。
