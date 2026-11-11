# NP5-01a A-2 复核（QA）：kinematics 33 + crb 30 source 级逐站点对照（63 站点）

- 复核对象：卡 `2026-11-11-np5-01a-a2-kinematics-crb-source-level`；内容提交 **`2cede9b6`**。
- 性质：只读复核（零 GPU、零窗、不改判据/参考/容差）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，1 一般 + 1 低 + 2 观察）**——结构/锚点/计数全对；**一般项=证据内出现正面「位级等价」表述，须改**。

## 1. 结构/字段（我独立重算）

- **站点集合**：63 条，与 `np5-01a/sites.json` 的 `kinematics`(33)+`crb`(30) 站点 id **逐 id 相等**（无缺/无多/无重复）；`site_count=63`、`pending_manual=0`、`np5_01a_total_sites=139`。
- **累计 139 ✅**：A-1a(40) ∪ A-1b(36) ∪ A-2(63) = **139，交集为空**，与拆分口径一致。
- **直方图自算**：`{n/a 32, identical 27, engine-semantic 4}` = 63，与 evidence/receipt/README 一致。
- **必填字段**：63/63 齐全；`type_mapping` 63/63 有 product/upstream/type_class，`double_in_product=false` 63/63。
- **4 条 `engine-semantic` ✅**：均带 `why_engine_semantic`（66–119 字）与 `bit_level_guarantee:false`；我逐条读，均为实现路线/调度语义差异（确定性累加 vs `wp.atomic_add` 树序、惯量旋转三重循环 vs `quat_to_mat` 组合、armature 加法位置、常驻原子汇聚），**未作数值等价结论**（com_position acc:0 明写「f32 值可能 1 ULP 级差异、不声明等价」）。
- **`n/a` 32**：均有 `why_not_applicable`（声明/缓冲/视图/纯拷贝/零初始化/prose）。
- **锚点抽样**：`src/physics/mod.rs:398/399/210/223/384/405` 行内容与 expression 相符；上游 `smooth.py` `_kinematics_branch` 48–146、`_compute_body_matrices` 169–176、`_site_local_to_global` 210–227、`_crb_accumulate` 1000–1015、`_M` 1019–1046、`crb` 1050–1068 与 `math.py inert_vec` 121–130 **我逐段打开核对，均为所述函数/表达式，行域正确**。

## 2. 输入/manifest/门禁/边界

- 场表哈希我复算全对：kinematics `dccba32d…`/11,278、crb `184ae567…`/9,752；spec/词表哈希与 receipt 一致；`tree.head=cb13e7b3…` 为该卡父提交。
- `manifest.txt` **4/4** 全对（README/evidence/receipt/logs，不自列；我独立重算 sha+bytes）。
- 门禁：fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**。
- 边界：提交 10 files = 证据 5 + `.normify`（卡+4 gate 日志）；无 `src/tests/fixtures/scripts` 改动；目录内文件 UTF-8/LF 无 BOM/CR/NUL（我逐文件核）。

## 3. 发现

### 一般（须改）
- **G1 正面「位级等价」表述（2 处）**：
  1. `evidence.json` → `crb:rigid_mass_matrix:accumulation:2` 的 `why_engine_semantic`：「…（表 rounding-note:1 已登记「两边加法次序不同但**结果位级等价**」）」；
  2. `evidence.json` → `crb:rounding-note:1` 的 `label`：「…两边 f32 加法次序不同但**结果位级等价**」。
  - 这是**正面位级等价结论**，与本片自己的 `bit_level_guarantee:false`、README §2「这些差异均为实现路线/调度语义差异，**不作数值等价结论**」自相矛盾，也违反「不得写位级等价」的全片口径。
  - 建议改法：写成「同一 f32 加法、结合/次序不同；**未做位级测试、不提供位级保证**」，或用「同值预期（解析）、未位级验证」。若确要做位级结论，须 B 类前提（同编译形态+同 recipe+同 GPU）+ 逐位测试证据，本片没有。

### 低
- **L1 selfcheck 禁词表缺口**：`logs/selfcheck.txt` 报 `forbidden_phrases_hit=[]`，但 evidence 实际含正面「位级等价」；说明 selfcheck 的禁词表未含该词（也未含「字节相等/上游等价」等组合）。建议把「位级等价/字节相等/覆盖率/通过率」纳入自检正则，并在后续片复用。

### 观察
- **O1**：直方图只有 `n/a/identical/engine-semantic` 三类（无 `not-decidable`/`recipe-difference`/`pin-needed`）；source 级-only 片可解释，但建议 README 明写「本片 source 级全部可判，device 级不在本片范围」以免被误读为「无遗留项」。
- **O2**：参考锚点有的用单行 `line`、有的用 `body_lines` 区间，粒度不统一；不影响本次结论，后续片可统一（区间 + 关键行）。

## 4. 非声明

本复核对 A-2 的站点/分类/锚点/卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。G1 与 L1 待 Backend 下一片（或 addenda）处置，处置方式由 Chief/Arch 定。
