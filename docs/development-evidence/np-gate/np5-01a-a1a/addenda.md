# A-1a 待复核项裁定回写 addendum（Chief 2026-11-11 修订版；append-only，不改 evidence.json 字节）

本文件是 **append-only 追加**：`evidence.json` / `README.md` / `receipt.json` 原文与字节均不改；本文件与
manifest 重算为派生更新。**以 Chief 2026-11-11 修订裁定为准**（采纳 Arch 建议，替代 18:1x 的前一版）：

## R1 `rigid_mass_solve:intermediate:3`（tile/scalar 路径选择）
- **认可 `engine-semantic`**：同一数学解、不同分解路径、**位级不保证**；**不得当 `identical`**。
- A-1a landing 值即为 `engine-semantic`，无需改判；本节为口径确认。

## R2 `rigid_mass_solve:accumulation:3`（产品 MathDx reciprocal vs 参考 `_qLD_acc` 除法）——**改判**
- **改判为 `recipe-difference`**（替代 landing 时点记录的 `pin-needed`），必带：
  - `value_equivalent: true`
  - `value_equivalence_basis: "analytic, not bit-tested"`
  - `product_form: "预乘 reciprocal 后乘（两次舍入路径）"`
  - `reference_form: "_qLD_acc 每元素除法 tmp=L/diag（一次舍入）"`
  - `why_value_equivalent: "两侧求同一 tmp 值（L/diag）；产品以 reciprocal 预乘近似同一比值。Chief 裁定按 recipe-difference 记，值预期等价、未位级测试"`
  - `bit_tested: false`
- 注记：**MathDx 是产品 opt-in 路线、默认路径不走**；若未来变默认路线需重评。
- **不记 `pin-needed`、不进 deferred 偏离清单**；可在 NP5-04 **观察清单（watch）**记一条。
- A-1a `evidence.json` 字节不改（其中该条仍为 landing 时点 `pin-needed`）；RE-1 卡按本节回写口径引用/覆盖。

## R3 `rigid_mass_solve:accumulation:0/1`（tile 路径参考 unique 内核未 capture）
- **保持 `not-decidable`**；**不引入 `pending-a1` 新词**。
- note 写明：**A1 codegen 已具备，由 RE-1 升级为 device 级**（A1-final 含 tile factor/solve 变体 PTX）。
- A-1a `evidence.json` 字节不改；RE-1 出最终分类（identical / recipe-difference / not-decidable+缺因）。

## R4 `rigid_mass_solve:accumulation:2`（`identical` 前提）
- **认可**；前提文字补「**收缩许可、未证位级相等**」不变。

## 非声明
本 addendum 不改判据/参考/容差，不宣称 NP01 通过/精度完成/位级等价；零 GPU、零窗。

## 5. QA 跨片复核修正（卡 QA-ADD，2026-11-11；append-only，不改历史字节）

来源：QA 对 A-1a/A-1b/RE-0 的复核（PASS + 1 一般项 + 低项），按 Chief 指令在本批以 append-only 处置。

### G1（必须改判：含 tile 分支的 `identical` 不一致）
- **改判**：`mass_solve:resident_mass_factor_派生:accumulation:0` 原记 `identical`（派生文本含 tile 分支）→
  **`not-decidable`**；`not_decidable_reasons: ["no-lineinfo", "frozen-artifact-only"]`；
  note「tile 分支设备级不可判；A1 已 codegen、由 RE-1 覆盖」。
  依据：同一覆盖文本（含 tile 分支）不得在不同站点判成 `identical` 与 `not-decidable` 不一致。
- 本目录 `evidence.json` 字节不改；以本行为准。`rigid_mass_solve:accumulation:0/1/5` 的 tile 设备级结论见 RE-1
  （acc:0 `identical` / acc:1 `not-decidable` 待裁 / a1b acc:5 `recipe-difference`）。

### 低项（同批修正）
- **低① 数字校正**：`evidence.json` 中 `rigid_mass_solve:accumulation:0` 的 `missing_evidence` 原写
  「**2603 entries**」→ 正确为 **303 entries**（6 份 smooth 副本 ×48 + `generated/module.ptx` ×15）。原字节不改，以本行为准。
- **低② 待复核计数一致**：`README.md §3` 列 4 条，而 `evidence.json.requires_decision` 只有 3 条 → 第 4 条为
  `rigid_mass_solve:accumulation:2`（`identical` 前提：上游 `-=` 允许 fmad 收缩、产品显式 `fmaf` 同形；
  补「收缩许可、未证位级相等」）。该条已在 R4 记录，与本行口径一致。
- **低③ 词表版本**：本目录引用的 `fcbec8f9…`/4,560 为历史版本；**现版词表** =
  `46282084cbe884e53446155f0a2ed44336880367651661376b4a1e5d081f9591` / 4867 B（唯一口径），后续卡片一律引用现版。
- **措辞（⑦）**：统一「**归一化指令流等价**」；不写字节相等/位级等价。
