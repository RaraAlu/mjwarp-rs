# NP5-01a B-1 复核（QA）：RE-2 G1/G2/L1 修复 + B 批肌腱 78 站点 source 级

- 复核对象：卡 `2026-11-11-np5-01a-b1-tendon-source-level`；内容提交 **`bbd7c7cc`**（登记 `4e56fe1e`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 2 低 + 1 观察）**。

## 1. RE-2 G1 版本化（A-2 addenda）

- **v1 字节恢复 ✅**：`np5-01a-a2/addenda.md` = **2,030 B / `0ba40a0b…`**（与我此前实测的 v1 完全一致，字节级恢复，无静默改写）。
- **v2 版本化 ✅**：新增 `addenda-v2.md` = **6,729 B / `e6bde50c…`**，含 `version: 2`、`supersedes: addenda.md@0ba40a0b…`（2,030 B）+ `supersedes_reason`（说明 r2 曾原地改写、按裁定改版本化）、`quote_policy`（引用须逐字）。
- **标记引用块 ✅**：v2 §2 用 `<!-- policy-scan:quote:start/end -->` 包住 5 处逐字原文（保留原始分隔符写法），块内含「撤回/supersede 声明：…作废」；块外全否定式。
- **manifest ✅**：A-2 manifest = **909 B、6 条**（README/addenda/addenda-v2/evidence/receipt/selfcheck），我独立重算 sha+bytes **bad=0、集合相等、v2 已列入**。
- **RE-2 指针更新（additive）✅**：`evidence.json` 顶层 `removed=[]`、新增 `corrections_at_b1`、`policy_fix` 仅按 L1 更新（含 `landed_at: e5315d95` + 保留 `landing_at_949bf18b` supersede 子对象）；`receipt.json` 仅新增 `corrections_at_b1`（含 a2 addenda v1/v2 哈希与重扫数）；README 3322→4890 增加指针/更正说明；RE-2 manifest **645 B、4 条**全对。

## 2. RE-2 G2 扫描器引用块豁免（我构造实测）

- **工具身份 ✅**：`scripts/np-gate/policy_scan.py` = **8,013 B / `e28ddb89…`**（与 addenda/证据记值一致）；头注「NOT in the 93 guards inputs」保留；docstring 写明标记用法、逐字要求、人工复核职责与 18 字符窗口局限。
- **自测 ✅**：`--self-test` **20/20 PASS**（原 12 + 引用块 8 条）。
- **构造用例（我另写 11 例，全部符合规格）**：标记块+声明 → 0；未标记 → 1；标记但无声明 → 1；未闭合 → 1；声明在块外 → 1；end 之后正文 → 1；块内拆写原文 → 0（逐字原文允许）；两个块只有一个带声明 → 1。
- **重扫如实 ✅**：A-2 evidence **5**、`addenda.md`（v1 历史字节）**7**（仅审计）、`addenda-v2` **0**、A-2 README **1**（否定式非声明句被 18 字符窗口漏豁免，人工确认非正面结论）、receipt **0** —— 与声明逐项一致。

## 3. RE-2 L1/观察

- **L1 ✅**：RE-2 `policy_fix.tool` 改为 `sha256=e28ddb89…/8,013`、`landed_at: e5315d95`，并保留 `landing_at_949bf18b`（旧 sha 4,559）作 supersede 说明；receipt 记 `policy_tool_landed_at_949bf18b: false`。
- **观察记录 ✅**：mass_multiply 18 属 A-3、device 行级与 source 逐站点粒度不同，均记入 `corrections_at_b1.observations`。

## 4. B-1 肌腱 78 站点（section 2）

- **站点集合 ✅**：78/78，与 `sites.json` 的 7 行并集**逐 id 相等**（fixed_tendon 7 / spatial_tendon 20 / tendon 10 / tendon_wrap 9 / tendon_velocity 7 / tendon_passive 12 / joint_passive 13），无缺/无多/无重复。
- **直方图 ✅**：`{n/a 50, identical 22, engine-semantic 6}` = 78，与 evidence/receipt/README 一致；**累计 217**（40+36+63+78）且交集为空。
- **6 条 engine-semantic ✅**：`fixed_tendon:fixed_tendon:accumulation:0`、`spatial_tendon:spatial_site:accumulation:0/1`、`spatial_tendon:spatial_moment:accumulation:0`、`tendon_wrap:wrap_circle_inside_intersect_device_函数族:rounding-note:0`、`joint_passive:joint_passive_compute:intermediate:3` —— 均带 `why_engine_semantic`（107–139 字）+ `bit_level_guarantee:false`；理由为「产品单线程确定性累加 vs 上游 `wp.atomic_add` 并发、f32 求和次序可能不同，不声明等价、不做位级结论」等，**未作数值等价结论**。
- **锚点 ✅**：我全量解析 **0 错误**（57 条 file:line + 21 条 `no-reference-expression`，后者均为 `n/a` 且带 reason）；抽样核行内容正确（如 `fixed_tendon.rs:113 length[t]=sum;` ↔ `smooth.py:4013 wp.atomic_add(...)`；`spatial_tendon.rs:159` ↔ `smooth.py:4110`）。
- **必填字段/类型映射 ✅**：78/78 齐全；`policy_scan` 对 B-1 evidence/README **0 命中**；selfcheck 15 项全 PASS；manifest 4/4；门禁 1,406/0/402；UTF-8/LF 无 BOM/CR/NUL；无覆盖率/通过率表述。

## 5. 发现

### 低
- **L1 声明匹配未防否定**：引用块声明按裸子串匹配「撤回/supersede/作废」，因此块内写「**未撤回**」「**不作废**」仍会被豁免（我构造实测 0 命中）。建议拒绝被否定的声明（检查 撤回/作废 前是否紧邻 未/不），或要求正向声明句式。
- **L2 新状态未文档化**：B-1 引入 `reference_anchor.status = "no-reference-expression"`（21 条 n/a），README/词表未说明其定义与与 `no-reference-entry` 的区别；建议补一行。

### 观察
- **O1 豁免是内容盲的**：标记+声明块内任何内容（包括新写的断言而非逐字引用）都会被豁免；「逐字」规则只靠 QA 人工复核（docstring 已声明该职责）。建议后续片对每个标记块做一次人工核对留痕。

## 6. 非声明

本复核对 RE-2 三项修复闭环与 B-1 站点/分类/锚点/卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
