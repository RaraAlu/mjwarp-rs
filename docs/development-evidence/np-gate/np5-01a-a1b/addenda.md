# A-1b QA 跨片复核修正 addendum（卡 QA-ADD，2026-11-11；append-only，不改历史字节）

## G1（必须改判：含 tile 分支的 `identical` 不一致）
- `mass_solve:resident_mass_substitute:accumulation:0` 原记 `identical`（派生文本含 tile 分支）→
  **`not-decidable`**；`not_decidable_reasons: ["no-lineinfo", "frozen-artifact-only"]`；
  note「tile 分支设备级不可判；A1 已 codegen、RE-1 覆盖」。本目录 `evidence.json` 字节不改，以本文件为准。

## 低项（同批修正）
- **词表版本**：现版词表 = `46282084cbe884e53446155f0a2ed44336880367651661376b4a1e5d081f9591` / 4867 B；历史 `fcbec8f9…`/4,560 仅为历史记录。
- **措辞**：统一「**归一化指令流等价**」；不写字节相等/位级等价。
- RE-1 交接不变：tile solve 站点已由 RE-1 定级 `recipe-difference`（MathDx opt-in 注记）。
