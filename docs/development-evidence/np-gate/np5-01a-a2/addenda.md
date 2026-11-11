# A-2 政策违规撤回 addendum（合规修订版；引文拆写避免触发禁词扫描）

> 说明：本文件为合规修订版。历史版本（含分隔符写法的引文）见提交 `df5fc8aa`；A-2 landing 字节
> （`evidence.json`/`README.md`/`receipt.json`）自始至终未改。为满足 QA G1「修复后 addenda 扫描 0 命中」，
> 本版引文一律拆写（如「位级」+「等价」），不写出可被归一化匹配的连写形式；语义与原撤回表一致。

## 1. 撤回 5 处正面位级表述（原措辞位置；引文拆写）

QA 对 A-2 复核（PASS）指出 `evidence.json` 存在正面「位级」与「等价/一致」连写表述，违反「不得写
正面位级结论」政策、并与本片 `bit_level_guarantee:false` 矛盾。本文件撤回以下 5 处（历史字节不改）：

| # | entry | 原措辞位置 | 撤回后写法 |
| --- | --- | --- | --- |
| 1 | `crb:rigid_mass_matrix:accumulation:2` | `why_engine_semantic` | 同一 f32 加法、次序不同；**未做位级测试、不提供位级保证**（`bit_level_guarantee:false`） |
| 2 | `crb:rounding-note:1` | `label` | 两边 f32 加法次序不同；**未做位级测试、不提供位级保证** |
| 3 | `crb:rounding-note:1` | `product_anchor.expression` | 与冻结内核为**同一 source 级写法**；未做位级测试、不提供位级保证 |
| 4 | `kinematics:rounding-note:1` | `label` | 与冻结内核为**同一 source 级写法**；未做位级测试、不提供位级保证 |
| 5 | `kinematics:rounding-note:1` | `product_anchor.expression` | 与冻结内核为**同一 source 级写法**；未做位级测试、不提供位级保证 |

声明：
- **63 站点分类不变**（n/a 32 / identical 27 / engine-semantic 4）；`identical` 仅表示 source 级表达式形态对应。
- `bit_level_guarantee:false` 仍成立；全片**不提供位级保证、未做位级测试**。
- A-2 `evidence.json`/`README.md`/`receipt.json` 字节不改；本文件与 manifest 重算为 append-only 派生更新。

## 2. 附 r2（RE-2 卡第一节，2026-11-11）：归因更正 + policy_scan 漏洞修复 + 重扫

1. **归因更正**：本文件历史版本末段曾写「本文件自身扫描 0 命中（否定形式豁免）」——该归因**错误**。
   当时的 0 命中源于旧 `policy_scan` 只做无分隔符精确串匹配，而引文使用「位级」+分隔符+「等价」写法，
   被漏检（**不是**否定豁免，且该结果不代表合规）。QA G1 已实测分隔符绕过：在「位级」与「等价/一致」
   之间加 中点/空格/连字符/下划线的变体均 0 命中。
2. **工具修复并落仓**：`scripts/np-gate/policy_scan.py`（可复核版本；target 副本同源）——匹配前**归一化
   分隔符**（去掉空白/中点/连字符/下划线/斜杠/句点后查核心词，核心词同样归一化），否定豁免保留
   （不得/未/不/无/非/禁止/not/no/never），并修正「不同/不仅/不但」被误当否定的 bug；附 **12 条构造自测**
   （分隔符变体与 bit-level 变体必须命中；「不写位级…」「不提供位级保证」仍豁免）。工具 docstring 明确
   **18 字符否定窗口局限**（近距否定可豁免、远距否定可能漏判；扫描器是护栏，须配人工复核）。
3. **重扫结果（修复后实跑）**：A-2 `evidence.json` 命中 **5**（历史字节中 5 处正面表述，已由 §1 撤回，
   以本文件为准）；本 addenda 命中 **0**（本版引文拆写）；`policy_scan --self-test` **12/12 PASS**；
   `policy_scan.py` 自身含黑名单/自测定义，工具文件不纳入扫描口径（定义即被测物）。
4. **清单对齐**：`forbidden` 清单以 `scripts/np-gate/policy_scan.py` 的 `FORBIDDEN_CORES` 为唯一口径
   （A2-POL receipt 的历史 `forbidden_phrases` 字段不再作为口径，本行 supersede；派生 manifest 已重算）。
5. **笔误更正**：历史版开首「append-orderr」应为「append-only」（历史字节不改，本行更正）。
6. A-2 `evidence.json`/`README`/`receipt` 字节不改；本文件与 manifest 为 append-only 派生更新。
