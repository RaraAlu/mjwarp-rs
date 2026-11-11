# RE-1 裁定回写 addendum（append-only，不改历史字节）

## R5（Chief 2026-11-11 裁定）：`acc:1` 从 `not-decidable` 改判 `engine-semantic`

- 站点：`mass_solve:rigid_mass_solve:accumulation:1`（tile 对角步：产品 `rsqrt.approx` vs 参考 scalar `sqrt.rn`/`rcp.rn`）。
- 原 landing 值（`evidence.json` / `receipt.json` 字节不改）：`not-decidable` + `requires_decision` 一条。
- **改判：`engine-semantic`**，字段：
  - `why_engine_semantic`：「产品 opt-in MathDx tile 路径 `rsqrt.approx` vs 参考默认 scalar 路径 `sqrt.rn`/`rcp.rn`；
    实现路线不同、数值非严格等价（approx vs exact）」
  - `bit_level_guarantee`: `false`
  - note：「MathDx 为 opt-in、默认路径不走；若未来变默认路线需在 E 指标定稿后重评（并可能转为 pin/deferred 偏离项）」
- **NP5-04 记 watch 项**（不是 deferred 偏离清单；除非默认路线切换）。
- **不得**用 `recipe-difference`（其 `value_equivalent` 必须为 true，此处 approx vs exact 不成立）、
  **不得**用 `pin-needed`（无 pin 规格；vendor MathDx PTX 不动）。
- 生效方式：本文件为 append-only 口径，**取代** `evidence.json.requires_decision` 中该条；历史字节不改，
  manifest 已按本文件重算。RE-1 其余分类（acc:0 `identical`、a1b acc:5 `recipe-difference`）不变。

## 过程事件记录（写锁纪律）

- 18:15:09 QA 持锁期间，我未先核锁状态即执行 `normify fix <RE-1 卡>`，致本卡 JSON 出现一处本地修改
  （未提交、未再写仓）；18:24:33 QA 释放锁后我 acquire，并在持锁状态补做本 addenda / manifest 重算 /
  gate / done / 提交。该过程事件按纪律记录于此，供 QA 复核引用。
