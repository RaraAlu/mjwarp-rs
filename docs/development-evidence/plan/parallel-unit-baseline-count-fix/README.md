# 并行计算单元能力基线：数字修正与计数规则附录（证据）

- 任务：`2026-11-12-parallel-unit-baseline-count-fix`（`.normify/changes/2026-11-12-parallel-unit-baseline-count-fix.json`）。
- 日期：2026-11-12（北京时间 18:xx，空闲写段；0 窗、零 GPU）。
- 派工：Chief；只改已验证数字与口径，不改用户文档的结构与判断性内容。
- 改动文件：`docs/implementation-plan.md`（累计 `a831c4eb..HEAD` 为 **+38 / −2 行**；明细见 `receipt.document.diff_stat`）。
- 性质：**纯文档**；`src/`、`tests/`、`fixtures/`、`scripts/` 零改动。

## 1. 改了什么

1. 设备层表中 `G.3` 的量化下限：`数组至少76；总数至少80` → `数组73；总数80（73数组+7标量）`。
2. 「待决策」①：`现有121个手写CUDA内核` → 四口径 `P-DEF 107 / P-PROD 95 / P-UNIQ 107 / P-ENTRY 112（产品100）`，并注明旧数 121 等仅为参考。
3. 节末 append-only 新增「计数规则与复核（2026-11-12）」小节（33 条内容行；含标题与空行共 36 行插入，不含下一节标题行）：上游 U-DEF/U-CALL/U-BODY/U-AST 口径与范围（U-CALL 计宿主与设备侧 `wp.xxx(` 出现次数，注释/docstring 不计）；本地四口径与载体/排除规则；`G.3` 补注上游最大单内核 `_render_megakernel`；不可复现项定稿状态（`B.6 237` 未定，候选 113/179/265；`D.1` U-BODY 726、698 仍不成立差 28；`I.5` 实测 23 种 287 处、19 种 255 处仍不成立差 32 处与 4 种；`F.4 module="unique"` 105，与 `91` 语义不同、`91` 待核；`G.5`/`G.6` 的 24 双规则并存；`G.7` 为块宽取值；`C.7` 为约数）；引用复核/对账材料 sha；末尾一行明确「105 以 F.4 句为准，113 为 B.6 候选」。

未改动：三层划分、“退”的划分、排序归属、PU 任务表、待决策②③等判断性内容；未新增任何“已实现/已验收”类声明。

## 2. 数字与依据

四口径：`P-DEF` 107（`.rs` 90 + `.cu` 17 源定义）、`P-PROD` 95（排除 8 个路径测试 + 4 个文件内 `#[cfg(test)]` 探针）、`P-UNIQ` 107（无重名）、`P-ENTRY` 112（含 `SMALL_ENTRY` 宏展开 5 个；产品口径 100）。旧数 121/125/114/91/108 仅参考；113/105 为“正则/文本级”。

复核材料（target 工作材料，非仓内验收证据）：

| 材料 | sha256 | 字节 |
| --- | --- | ---: |
| `target/impl-plan-review/merged-count-table.md`（终稿合并表，本目录快照同文件） | `6572cdc6ae799a822eded3e16bc5f7b754b534e89dd2bb20d1c373e9dad62307` | 8,159 |
| `target/render-bvh/gl2-ptx-diff/pu-baseline-review/reconcile/reconcile.md`（对账明细） | `c21af22e4b859afd1b2634ecd500006ddd57168dc89cf9a683aa733e123d27b1` | 38,966 |

其余 target 材料 sha 清单见 `provenance.json`；本目录 `merged-count-table.md` 是逐字节快照，快照 sha 与上表一致。

## 3. 校验

- 四门禁：`normify gate 2026-11-12-parallel-unit-baseline-count-fix`（fmt / clippy / clippy-cuda / unit；unit 1410 过 / 0 败）。
- `unchanged src/,tests/,fixtures/,scripts/` 通过。
- `policy_scan`：改动文件 0 命中；`--self-test` 34/34。
- `manifest.txt` 自核（不列自身）；UTF-8 / LF、无 BOM。
- 文档标记机检：`数组至少76` 与 `121个手写CUDA内核` 已清零；新增四口径与小节标记存在。

## 4. 本目录文件

| 文件 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读收据（任务、四口径、文档 sha、门禁、policy_scan） |
| `manifest.txt` | 本目录逐文件 sha256 + bytes，自核（不列自身） |
| `merged-count-table.md` | 终稿合并表逐字节快照（sha `6572cdc6…`） |
| `provenance.json` | 其余 target 复核/对账材料 sha 清单与文档机检结果 |

## 5. 复跑

```powershell
cd D:\Rust\mjwarp-rs
normify show 2026-11-12-parallel-unit-baseline-count-fix
normify gate 2026-11-12-parallel-unit-baseline-count-fix
target\mjwarp-parallel\venv\Scripts\python.exe scripts\np-gate\policy_scan.py docs\implementation-plan.md docs\development-evidence\plan\parallel-unit-baseline-count-fix\README.md docs\development-evidence\plan\parallel-unit-baseline-count-fix\receipt.json
```

## 6. 边界

- 本次复核为**静态扫描**，未运行上游或产品内核验证；不构成能力实现或验收结论。
- 数字口径以 `reconcile.md` §5.2 与合并表 §1/§2 为准；唯一保留待核：`G.5` 的 24（“同缓冲 in/out” vs `launch_tiled` 两规则并存）。
- `target/` 工作材料不是仓内证据；本目录只快照合并表并记录其 sha，其余仅列 sha 供追溯。
