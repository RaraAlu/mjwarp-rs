# NP5-01a 跨片复核修正（卡 QA-ADD；append-only；零 GPU）

- 任务：`2026-11-11-np5-01a-qa-cross-slice-addenda`（owner Backend；p95；P8-渲染与BVH）。
- 范围：A-1a/A-1b/RE-0 三片 QA 复核（均 PASS）的 **1 条一般项 G1 + 7 条低项**；按 Chief 指令以 append-only
  新增 `addenda.md` 处置，**三片既有 evidence/README/receipt 字节不改**，各自 manifest 派生重算。
- 词表现版：`np5-01a-vocab-addendum.md` = `46282084cbe884e53446155f0a2ed44336880367651661376b4a1e5d081f9591` / 4867 B（唯一口径）。

## 1. 修正清单（8 项）

| # | 类型 | 内容 | 落点 |
| --- | --- | --- | --- |
| G1 | 必须改判 | 含 tile 分支的 `identical` 站点改判 `not-decidable`：A-1a `resident_mass_factor_派生:accumulation:0`、A-1b `resident_mass_substitute:accumulation:0`（reasons `no-lineinfo`/`frozen-artifact-only`；note「tile 分支设备级不可判，A1/RE-1 覆盖」） | `np5-01a-a1a/addenda.md §5`、`np5-01a-a1b/addenda.md` |
| ① | 低（数字） | A-1a `missing_evidence`「2603 entries」→ **303**（6×48+15） | `np5-01a-a1a/addenda.md §5` |
| ② | 低（一致） | A-1a README §3 第 4 条（`accumulation:2` identical 前提）与 `requires_decision` 计数对齐 | `np5-01a-a1a/addenda.md §5` |
| ③ | 低（词表） | 词表引用统一为现版 `46282084cbe884e5…`/4867 | A-1a/A-1b addenda |
| ④ | 低（计数） | RE-0 `residual_diff_types` 6→5（合并同一项两种措辞） | `re0/addenda.md` |
| ⑤ | 低（路径） | RE-0 §5 补 codegen 脚本含本机绝对路径（开发用、移植需改） | `re0/addenda.md` |
| ⑥ | 低（说明） | RE-0 说明 `a1_*` 脚本用途（A1 codegen 路线） | `re0/addenda.md` |
| ⑦ | 措辞 | 统一「归一化指令流等价」 | 三处 addenda |

## 2. 不变式

- 改判只作用于**本文件/各 addenda 的口径**；三片 landing 字节（evidence.json/README/receipt）不改。
- 三片 manifest 重算（派生索引）：A-1a 增 `addenda.md` 条目、A-1b 增 `addenda.md` 条目、RE-0 增 `addenda.md` 条目。
- 零 GPU、零窗；不改判据/参考/容差/断言语义。

## 3. 非声明

不宣称 NP01 通过/精度完成/上游等价/双平台/位级等价；不写覆盖率/通过率。
