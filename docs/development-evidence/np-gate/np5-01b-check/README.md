# QA 复核：NP5-01b 已实现子系统证据锚点 + 23 未登记单元（`316b28dd`，卡 J）

- 任务：`2026-11-11-np5-01b-implemented-subsystem-anchors-check`（owner QA）。复核对象：提交 **`316b28dd`** + 卡 `2026-11-11-np5-01b-implemented-subsystem-anchors`。零 GPU、只读复核。
- 基线：`cead8bac`（卡 J 登记/start）。

## 结论

**复核通过（0 阻断，1 低 + 2 观察）。** 3 个已实现子系统的锚点五段齐全且**存在性真实**（我独立核 53 条锚点路径全部在盘）；23 个未登记单元与 NP5-00 账本**逐 id/kind 一致**（23 文件全部存在）；**未新建阻塞卡**、无未加否定的门禁通过表述；W4 计划 §9 为**纯追加**并正确把「24 片」更正为 **25 片 = B0 4 + 实体 21（7 卡×3）**、GPU 窗数不变；写域与门禁干净。低项：receipt 缺 `tree.head` 快照字段（卡 J acceptance 明确要求）。

## 一、3 子系统锚点（✅ 存在性我独立核）

| 子系统 | 锚点五段 | 我的核验 |
| --- | --- | --- |
| 渲染/BVH | 实现面 `.cu`/host 链 / 8 内核入口 / NP02 6 行 verified + 2 缺行 / verified 卡与身份基线 / 仍缺 | 路径全部在盘；NP02 6 行均为 `verified`；缺 2 行 = `render_megakernel`、`render_depth_seg`（与账本 `missing_kernel_rows=2` 一致，→NP5-03）；身份 `cf356b72…`/`b63a51dd…`/348,459 与我 z2vec 复核的当前内核身份一致 |
| 地形/SDF 碰撞 | `.cu` 清单 + host 链 / hfield+sdf 入口 / NP02 2 行 verified / `sdf-release-v2.2` 等发布证据 / product.cu 身份 / 仍缺 | 路径在盘；2 行 verified；身份 `cad08d3e…`/`54ab59e3…`（3,883,769 B）为既有基线引用 |
| 凸体/网格 GJK·EPA·convex | 8 个 `.cu` + gjk host / boxbox·contact·contact_multi·primitive 入口 / NP02 4 行 verified / family-closeout 等 / 身份 / 仍缺 | 路径在盘；4 行 verified；`2026-11-04-g10-p5-family-closeout` 我核 board 状态 **verified** |

- 我一共核 **53 条锚点路径/证据路径**（实现面 + host_chain + 入口 + NP02 行 + verified 卡 + evidence_anchors）：**missing=0**。
- 3 张 verified 卡（`2026-11-04-g10-p5-family-closeout`、`2026-11-04-render-r06-flex-closeout`、`…attempt4`）我核 board 状态均 `verified`。
- JSON 结构：`subsystem_count=3`，每子系统含 `implementation_surface / kernel_entries / np02_evidence_rows / np02_missing_rows / verified_cards|verified_release_evidence / identity_baseline / remaining_gaps` 等段；`boundary` 明写「登记锚点，不建阻塞卡；不宣称门禁通过/NP01 通过/上游等价/双平台」。

## 二、23 未登记单元（✅ 与账本一致）

- `unregistered-units.json`：`unit_count=23`、`source_ledger_sha256=6d6ff7d9…`；id 集合与账本 `np01.unregistered_product_units` **完全一致**（我逐 id 比较），kind 直方图 = 8 render-kernel + 3 render-host-chain + 1 render-host-diagnostic + 6 collision-kernel + 1 collision-device-lib + 2 np01-host-site-uncovered + 2 np01-product-helper = **23**；每单元含 id/kind/product_files/present/np01_status/owner/next_action/registration。
- 我核 23 个单元的 `product_files`：**23/23 文件在盘**（missing=0）。
- 与 NP5-00 账本交叉：账本 `np01.unregistered_product_units` 23 条、`totals` 190 条中对应分项 23 ✅（账本本身见卡 F 复核）。

## 三、非声明 / 边界 / 证据 / 门禁（✅）

- **不建阻塞卡**：本提交 `.normify/changes` 仅改卡 J 自身，无新增卡；4 个真未实现子系统仍由卡 H 的 4 张 backlog 阻塞卡承载（不重复建卡）。README 三处「不宣称门禁通过/NP01 通过/上游等价/双平台」，我逐行 grep：「NP01 通过」「门禁通过」均出现在**否定句**中，「已通过」0 处；checker 第 13 项亦断言无未加否定门禁通过声明。
- 边界：提交 12 files = `docs/development-evidence/np-gate/np5-01b/`（6 文件）+ `plan/W4-dependency-governance.md` + `.normify`（卡 J + 4 门禁记录）；**无** `src/`/`tests/`/`fixtures/`/`scripts/` 改动、guards/freeze script 未动、不改判据/参考/容差；零 GPU、零窗。
- 哈希与 Chief 派工值逐位一致：`implemented-subsystems.json` **`c4a50ccf…`/40,864 B**、`unregistered-units.json` **`35683aaa…`/28,364 B**、W4 文档 **`ef78ba82…`/13,791 B**；README `af416071…`/6,108 B；manifest **5/5** 独立重算全对（不自列）；文件全 UTF-8/LF 无 BOM/CR/NUL。
- 门禁 `.normify/runs/…np5-01b…/`：fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**。

## 四、W4 计划 §9 更正（✅ append-only）

- `W4-dependency-governance.md` **13,457 → 13,791 B，共同前缀 13,457 = 旧长度**（纯追加，§1–§8 原文未改）；§9 内容正确：「原 §3 标题『24 片 = B0 4 片 + 实体 20 片』为算术笔误；实体片为 7 卡 × 3 片 = 21，总数应为 **25 片 = B0 4 + 实体 21**；GPU 窗数不变（仍 3–4 窗）」——与我卡 G 复核的 L1 完全对应（已由本卡 §9 更正）。

## 五、低项 / 观察

- **低 L1（记录完整性）**：卡 J acceptance 要求 receipt 含 `tree.head`，但 receipt 的实际键为 `schema/task_id/verdict/kind/status/inputs/counts/consistency/files/plan_doc_correction/gates/boundaries/non_claims/manifest`——**无 `tree`/`head` 字段**（我搜全文 'tree'/'head' 均无）。落仓时点 HEAD 目前只能由门禁记录时间与 `existence-check.txt` 的 run at 间接推断；建议按 acceptance 补 `tree.head`（不必改已提交 receipt，可在 errata/下一卡补，或直接接受为 minor）。
- **观察 O1**：地形/SDF 段引用 `sdf-release-v2.2`（2026-11-02 发布结论）等作为「verified 发布证据」——属**引用既有证据**，README §5 非声明已覆盖；建议后续引用时加「既有结论引用，非本次判定」字样，避免被读成新结论。
- **观察 O2**：子系统身份基线（如 collision product.cu `cad08d3e…`/`54ab59e3…`）为既有证据引用值；本卡无 `.cu` 改动，我未重算 PTX（无源变、无信息增益），只核了文件存在与卡/行状态。

## 六、口径

不宣称门禁通过/NP01/NP02 通过、不宣称上游等价/双平台/发布；不改判据/参考/容差；不建阻塞卡；零 GPU、零窗；本复核只核锚点真实性/清单一致性与边界。
