# A-2 政策违规撤回 addendum（append-orderr 追加；不改 evidence.json 字节）

QA 对 A-2 复核（PASS）指出：`evidence.json` 存在正面「位级·等价/位级·一致」表述，违反「不得写位级·等价」
政策、并与本片 `bit_level_guarantee:false` 矛盾。本文件**撤回**以下 5 处正面表述（历史字节不改；以本文件为准）：

| # | entry | 原措辞位置 | 原措辞（节选） | 撤回后写法 |
| --- | --- | --- | --- | --- |
| 1 | `crb:rigid_mass_matrix:accumulation:2` | `why_engine_semantic` | 「同一 f32 加法、次序不同（表 rounding-note:1 已登记『两边加法次序不同但结果位级·等价』）」 | 同一 f32 加法、次序不同；**未做位级测试、不提供位级保证**（`bit_level_guarantee:false`） |
| 2 | `crb:rounding-note:1` | `label` | 「两边 f32 加法次序不同但结果位级·等价」 | 两边 f32 加法次序不同；**未做位级测试、不提供位级保证** |
| 3 | `crb:rounding-note:1` | `product_anchor.expression` | 「…与冻结内核位级·一致」 | 与冻结内核为**同一 source 级写法**；未做位级测试、不提供位级保证 |
| 4 | `kinematics:rounding-note:1` | `label` | 「…与冻结内核位级·一致」 | 与冻结内核为**同一 source 级写法**；未做位级测试、不提供位级保证 |
| 5 | `kinematics:rounding-note:1` | `product_anchor.expression` | 「…与冻结内核位级·一致」 | 与冻结内核为**同一 source 级写法**；未做位级测试、不提供位级保证 |

声明：
- **63 站点分类不变**（n/a 32 / identical 27 / engine-semantic 4）；`identical` 仅表示 source 级表达式形态对应。
- `bit_level_guarantee:false` 仍成立，全片**不提供任何位级保证、未做位级测试**。
- `evidence.json` / `README.md` / `receipt.json` 字节不改；本文件与 manifest 重算为 A-2 的 append-only 派生更新。
- 本文件自身经 `policy_scan.py` 扫描 0 命中（否定形式豁免）。
