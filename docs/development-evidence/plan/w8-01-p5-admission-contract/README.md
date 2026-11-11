# W8-01 CT：P5 准入契约核对（0 窗只读；零 GPU）

- 任务：`2026-11-11-w8-01-p5-ct`（owner Backend）；阶段：**P5**；阶段伞卡 `2026-10-11-roadmap-p5`（status=backlog / checks=0）。
- 性质：**CT（契约/覆盖核对）**——把「P5 卡面 acceptance + 兼容矩阵行 + replication-plan 阶段段 + 依赖 verified 状态」对齐；
  **不生成平台证据、不宣称 P5 准入通过**。

## 1. 卡面 acceptance（P5 伞卡，10 条）

- 核对适用切片与矩阵。
- 先完成范围内NP门禁。
- 保留双模式平台证据。
- 按适用范围检查M11。
- 按适用范围保存A11证据。
- 先实现几何参考路径。
- 再实现BVH并行优化。
- 记录真实测试与未测项。
- 先核对NP01与NP02。
- 再验收适用NP03与NP04。

## 2. 阶段卡状态（6 张）

| 卡 | status | checks | depends_on |
| --- | --- | --- | --- |
| `2026-10-11-roadmap-p5` | backlog | 0 | 2026-10-11-roadmap-g07-p5, 2026-10-11-roadmap-g08-p5, 2026-10-11-roadmap-g09-p5, 2026-10-11-roadmap-g10-p5 |
| `2026-10-11-roadmap-g07-p5` | verified | 6 | — |
| `2026-10-11-roadmap-g08-p5` | verified | 6 | 2026-10-11-roadmap-g07-p5 |
| `2026-10-11-roadmap-g09-p5` | verified | 6 | 2026-10-11-roadmap-g07-p5, 2026-10-11-roadmap-g08-p5 |
| `2026-10-11-roadmap-g10-p5` | proposed | 0 | 2026-11-04-g10-p5-family-closeout |
| `2026-11-04-g10-p5-family-closeout` | verified | 2 | 2026-10-25-g10-p5-acceptance-perf-native, 2026-10-25-g10-p5-acceptance-perf-native-independent-check, 2026-10-25-g10-p5-gpu-kernels-independent-check, 2026-10-25-g10-p5-hfield-host-independent-check, 2026-10-25-g10-p5-sdf-host-r2, 2026-10-31-g10-p5-mesh-cap20-check |

- P5 伞卡依赖：{"2026-10-11-roadmap-g07-p5": "verified", "2026-10-11-roadmap-g08-p5": "verified", "2026-10-11-roadmap-g09-p5": "verified", "2026-10-11-roadmap-g10-p5": "proposed"}。
- `g10-p5` 伞卡仍 proposed/0；已由 Chief 批准（B0-2/卡 I）改指 **verified** 的
  `2026-11-04-g10-p5-family-closeout`，但其口径为 **met 8 / partial 4 / unmet 0 + uncovered 6**，
  **明说不构成整族通过** → 不得写成 G10 族通过。

## 3. 覆盖对照（矩阵行 × 证据）

| 矩阵行 | 功能族 | 阶段 | 矩阵文件锚点 | 现有证据 |
| --- | --- | --- | --- | --- |
| G07 | 宽相碰撞 | P5 | 第 336 行 | 卡 g07-p5 verified（6 checks） |
| G08 | 基本几何碰撞 | P5 | 第 337 行 | 卡 g08-p5 verified（6 checks） |
| G09 | 凸体与网格 | P5 | 第 338 行 | 卡 g09-p5 verified（6 checks） |
| G10 | 地形与SDF碰撞 | P5 | 第 339 行 | 卡 g10-p5-family-closeout verified（met 8 / partial 4 / unmet 0 + uncovered 6；明说不构成整族通过） |
| M11 | 错误状态 | 全阶段 | 第 382 行 | 跨阶段诊断/失败隔离验收未收口（P5 范围无独立卡） |
| A11 | 双平台发布 | 全阶段/P11 | 第 426 行 | 双平台发布属 P11 用户级（当前不排窗） |

- replication-plan 阶段段：P5 行（第 271 行）「几何对、宽相、接触、容量；前置 P3/P4；
  首个可观察交付=接触结果与独立几何证据一致」。

## 4. 缺口（6 项）

| id | 类别 | 说明 |
| --- | --- | --- |
| G-NP | NP 门禁 | NP01/NP02 全局未验收（NP5-00…04 为证据卡、NP 伞卡未 verified）→ P5 阶段 AD 不得开窗，结论只能记「等待 NP 门禁」；g10-p5 closeout 的 NP 核对仅为家族范围（NP01 已核对、E 已签 A2、NP02 满足），不改变全局未验收 |
| G-P4 | 阶段前置 | P5 处于 P4 之后（replication-plan P5 行：前置 P3/P4）；P4 阶段伞卡 p4 仍 backlog/0、未验收 → P5 AD 的阶段前提前未满足 |
| G-G10 | 族口径 | G10 伞卡 g10-p5 仍 proposed/0；verified 收口卡只到 met 8 / partial 4 / unmet 0 + uncovered 6（含设备地形接触 overflow 只在 QA 复核、容量截断只在 QA、默认产品管线未接入、mesh polygon>16 设备计数差、深穿透 4/24 用户拍板 A 不修、Linux/T4），不得写成整族通过 |
| G-SRC | 来源指针漂移 | g10-p5 卡面来源 `docs/compatibility-matrix.md:263` 当前为「完整对标分母」段说明，不再指向 G10 行；G10 实际锚点=矩阵第 339 行；语义不变、按矩阵 ID 解析（非内容缺口） |
| G-AD | 证据缺口 | P5 阶段 AD（Windows debug+release 双模式平台证据）未生成；按 w8-breakdown §2 模板，AD 需 CT 无缺口 + 范围内 NP 门禁已过 + 独占窗口 |
| G-PARTIAL | 已知边界 | g07/g08/g09 三张 verified 卡各自 acceptance 已写明未覆盖边界（SLEEP 唤醒、flex/mesh/SDF 深路径、box-box/capsule/cylinder/ellipsoid/cone 凸体路径、CCD/multiccd、EFC/约束求解等）；本 CT 不把它们扩写成 P5 全阶段通过 |

**NP 门禁状态：等待 NP 门禁**；**P4 阶段前置（p4 backlog/0）亦未满足** → P5 AD 片不得开窗。

## 5. 结论

CT 完成：P5 兼容矩阵 6 行、阶段卡 6 张（g07/g08/g09 verified；g10-p5 proposed + family closeout verified 但非整族通过）与 replication-plan P5 段已对齐；缺口 6 项——NP 门禁、P4 阶段前置、G10 族口径、来源指针漂移、AD 证据、已知边界。不构成 P5 准入通过，AD 等待 NP 门禁 + P4。

准入前提四项（CT 无缺口 / NP 门禁 / P4 阶段前置 / 窗口）当前**全部未满足**，不得写成准入通过。

## 6. 验证

- selfcheck（`logs/selfcheck.txt`）：contract/矩阵行/阶段卡/缺口计数/`np_gate_status=等待 NP 门禁`/
  `admission_claim=false`、manifest 4 文件自核、policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录 + `.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 7. 边界与非声明

- 零 GPU、零窗；只写本目录 + `.normify/`；不改任何卡 acceptance/判据/参考/容差/矩阵/计划文件。
- 不宣称 P5 准入通过、不宣称 G10 整族通过、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布。
