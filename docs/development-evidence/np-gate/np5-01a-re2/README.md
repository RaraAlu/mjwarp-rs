# NP5-01a RE-2：A-2 设备级对照 + policy_scan 漏洞修复入仓（零 GPU）

- 任务：`2026-11-11-np5-01a-re2-a2-device-level-policy-fix`（owner Backend；p95；P8-渲染与BVH）。
- 材料：A-2 source 级证据（`np-gate/np5-01a-a2/`）、冻结 `smooth.ptx`、产品离线编译 PTX（target 工作材料）。

## 1. policy_scan 漏洞修复（QA G1）与 A-2 归因更正

- **漏洞**：旧 `policy_scan` 为无分隔符精确串匹配，「位级」与「等价/一致」之间加 中点/空格/连字符/下划线
  的变体 0 命中（实测绕过）。
- **修复**：`scripts/np-gate/policy_scan.py`（**入仓可复核版本**，`59765f71…`/4,559 B；target 副本同源）：
  匹配前**归一化分隔符**（空白/中点/连字符/下划线/斜杠/句点全去掉后查核心词，核心词同样归一化）；
  否定豁免保留（不得/未/不/无/非/禁止/not/no/never），修正「不同/不仅/不但」被误当否定的 bug；
  附 **12 条构造自测**（分隔符与 bit-level 变体必须命中；「不写位级…」「不提供位级保证」仍豁免）；
  docstring 写明 **18 字符否定窗口局限**（护栏而非证明，须人工复核）。头注：NOT in the 93 guards inputs。
- **A-2 归因更正**（append-only，`np-gate/np5-01a-a2/addenda.md` `da24375b3207c9fc0bb82b7170ad673a85fe0008ff37775b9c546d4afb9e876c`/4179 B）：
  历史「本文件扫描 0 命中（否定形式豁免）」为**错误归因**（实为分隔符漏检）；已更正并重扫。
- **重扫（修复后实跑）**：A-2 `evidence.json` 命中 **5**（= 5 处历史正面表述，已由 addenda 撤回）；
  A-2 `addenda.md` 命中 **0**；`policy_scan --self-test` **12/12 PASS**；工具自身含黑名单/自测
  定义，不纳入扫描口径。清单口径以 `FORBIDDEN_CORES` 为准（A2-POL receipt 历史字段 supersede）。
- A-2 `evidence.json`/`README`/`receipt` 字节不改；A-2 manifest 重算 `be9ffd46a53ea554…`。

## 2. A-2 设备级映射（kinematics/crb + mass_multiply）

- 产品 PTX：NVRTC 12.8.93 / compute_70 / lineinfo 离线编译 `src/physics/mod.rs` 与
  `src/physics/mass_matrix.rs` 的 CUDA raw string；**PTX 实体不入仓**（sha/bytes/命令见 `evidence.json`）。
- 冻结参考：`fixtures/gravity-upstream-abi/independent-inputs/smooth.ptx`（48 entries、0 `.loc`）。

| 行 | 站点 | 映射 | 分类 |
| --- | --- | --- | --- |
| kinematics | 33 | 参考 `_kinematics_branch`/`_compute_body_matrices`/`_subtree_com_*`/`_cinert` 存在；产品单 entry 内联 helpers，entry 非一一对应 | `not-decidable`（frozen-artifact-only,no-lineinfo） |
| crb | 30 | 参考 `_crb_accumulate`+`_M`；产品单 entry 合并树序累加+矩阵组装、resident 变体 atomic 汇聚 | `engine-semantic`（实现路线/数据流不同；bit_level_guarantee:false） |
| mass_multiply | 18 | 冻结 smooth.ptx/module.ptx 均无 entry | `not-decidable`（no-reference-entry） |

- 上述 device 级为**部分/语义级**结论；逐站点 source 级对照见 `np-gate/np5-01a-a2/`（149… 见 A-2 receipt）；
  **不写覆盖率**、不写位级结论。

## 3. 非声明

不宣称 NP01 通过/精度完成/上游等价/双平台/位级保证；不改判据/参考/容差；零 GPU、零窗；PTX 实体不入仓。

## 4. 2026-11-11 B-1 更正（版本化 addenda + 工具引用块豁免）

- **G1（addenda 版本化）**：本节此前把 A-2 `addenda.md` 原地改写（非 append）。已按裁定版本化：A-2
  `addenda.md` 恢复 v1 字节（`0ba40a0b…`/2,030），修正版为 A-2 `addenda-v2.md`（`supersedes:
  addenda.md@0ba40a0b…`）。本目录 `evidence.json` 的 A-2 addenda 指针随之更新；A-2 目录重扫：
  evidence **5**（历史已撤回）、addenda-v2 **0**、addenda v1 历史字节 **7**（不改历史）、README **1**
  （否定式非声明句被 18 字符窗口漏豁免，人工复核确认非正面结论）、receipt **0**。
- **G2（引用块豁免）**：`policy_scan` 新增显式引用块豁免（标记块 + 撤回/supersede/作废声明才豁免；
  未标记/无声明/未闭合/块外照常命中；引用须逐字原文，禁止拆写规避），自测 **20/20 PASS**；工具
  `e28ddb89…`/8,013 B。本文 §1 的历史「addenda 0 命中」仅指当时的改写版，以本 §4 与 A-2
  `addenda-v2.md` §3 为准。
- **L1（时点）**：`policy_tool.landed:true` 在 `949bf18b` 时点不成立，工具实际由 `e5315d95` 落仓；
  `evidence.json`/`receipt.json` 已加 `landed_at` 与更正说明。
- **观察项**：`mass_multiply` 18 站点属 A-3；设备行级与 source 逐站点粒度不同（见 A-2
  `addenda-v2.md` §5）。
- A-2 landing 字节（`evidence.json`/`README.md`/`receipt.json`）仍不改；A-2 manifest 已重算
  `e5d4e35b60dab133cb03105968d610555fc3aaa3d3f5400bd538c2ae4c4fe849`。
