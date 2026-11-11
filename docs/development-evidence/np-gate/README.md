# NP5-00：NP01/NP02 全量清单与判据映射冻结（机读账本 190 条）

- 任务：`2026-11-11-np5-00-gate-ledger-freeze`（owner Backend；p85；P8-渲染与BVH）。依据 Chief 2026-11-11
  裁定 1（NP5-00…NP5-04 独立证据卡先行、不提前标通过）与 Arch 规格（`8a88fa9d…`/10,610 B，**工作材料，非证据**）。
  本片**只冻结清单、判据映射、状态与 owner**，**不宣称 NP01/NP02 通过**、不宣称全产品精度门禁通过；
  不改判据/参考/容差；不重采证据；不重跑窗口；零 GPU。

## 1. 产物与落点

| 文件 | 说明 | sha256 / bytes |
| --- | --- | --- |
| `docs/development-evidence/np-gate/np-gate-ledger.json` | 机读账本（schema 1，**190 条**；与 target 定稿**逐字节相同**） | `6d6ff7d98f82e6429038d6f4c731afcc49cf693951a17e520a5a6fde6568c8ad` / 116,688 B |
| `scripts/np-gate/gen_np5_ledger.py` | 生成器（target 源 `7fe344d1…`/17,807 B + 1 行 guards 头注） | `bc9e9e0e5b1ab13d6975f260969c4b4e64cd224b86536194fe28e60e32c33db8` / 17,892 B |
| `scripts/np-gate/check_np5_ledger.py` | 独立校验器（target 源 `a7dbde52…`/4,817 B + 1 行 guards 头注） | `e7174bce4af7f5801b696e6ca13c0751a505ce12fb08c555b5e4438e54704f05` / 4,902 B |
| `README.md` / `receipt.json` / `manifest.txt` | 本说明 / 机器可读回执 / 逐文件清单（含两脚本） | 见 manifest |
| `logs/np-gate-ledger-check.log` | 独立校验器**原文输出**（19 PASS，RESULT: PASS） | `beae8f16e035e2c035fc7b1504adce8c9be9fe58e241e2ae493ad05e578eb22e` / 862 B |
| `logs/ledger-summary.txt` | 账本摘要（totals/条目/未登记单元/状态直方图/漂移/boundaries） | 见 manifest |
| `logs/tool-diff.txt` | target 源 → 落仓工具 diff（**仅 1 行 guards 头注/文件**，无逻辑改动） | 见 manifest |

来源材料（`target/render-bvh/gl2-ptx-diff/`，**工作材料，非证据**）：

| 材料 | sha256 / bytes |
| --- | --- |
| `np-gate-ledger.json`（账本定稿） | `6d6ff7d98f82e6429038d6f4c731afcc49cf693951a17e520a5a6fde6568c8ad` / 116,688 B |
| `np-gate-ledger-spec.md`（规格，**替代**旧 12,364 B 稿） | `8a88fa9dae8a2224ae3f937ad51b00f358f89305e9ae2f0670fe0b374a3fa96d` / 10,610 B |
| `_check_np5_ledger.py`（校验器源） | `a7dbde5280d70a4fa47928b53ef9dcdae03b11f47d71ca954743b7516f247697` / 4,817 B |
| `_gen_np5_ledger.py`（生成器源） | `7fe344d1fc1eceb5a8aa256995ee185fe8e0993e549fb01a78fd58ed43cd09eb` / 17,807 B |
| `np5-rulings-memo.md`（四点裁定落实备忘） | `3c0d68a4f5548a15d9da6f0706280b30a6e109de1846345088fb71698f0f173d` / 9,667 B |
| `np5-rulings-addendum.md`（12:2x 追加裁定） | `ce49fcf7141561a9eddaad42d3dc62185607f05b1cda5e69dfba6fd8fe4f8d3c` / 6,222 B |

**材料版本说明（观察项，不阻断）**：本卡按 Chief 指定的**更新版**材料收口（规格 10,610 B / 账本 190 条 / 校验器
4,817 B）；旧稿（规格 12,364 B、账本 186 条 `c2032170…`）**作废**。规格 §3 执行单第 2 步仍写旧 sha `c2032170…`
与「186 条」，属规格内部残留笔误（§1 与验收点已为 190 条/`6d6ff7d9…`），本卡以 Chief 指定值与规格 §1/验收点为准。

## 2. 账本结构（schema 1）与条目

顶层键：`schema / task / generated_at / producer / scope / inputs / status_vocabulary / totals / np01 / np02 /
boundaries`。**190 条** = `np01.paths` 30 + `np01.host_sites` 3 + `np01.unregistered_product_units` **23** +
`np02.kernels` 100 + `np02.missing_kernel_rows` 2 + `np02.gaps` 32。

- **23 个未登记产品单元**（NP5-01b 新增逐字段表范围）：8 render-kernel-entry（render_megakernel、render_depth_seg、
  aa_resolve、build_rays、mesh_bvh_ray、scene_bvh_ray、sample_texture2d、s7a_abi_probe）+ 3 render-host-chain
  （per_world、world_light、megakernel_arrays）+ 1 render-host-diagnostic（render_diag opt-in）+ 6 collision-kernel-entry
  + 1 collision-device-lib-aggregate + **2 np01-host-site-uncovered**（`host_site_render_light_dir_normalize`、
  `host_site_relmat_relative_matrix`，siteSemantics 已登记但无表）+ **2 np01-product-helper**（`math_fma_exact_f32_helper`、
  `render_quat_cast_host`）。
- **状态**：np01.paths/host_sites 全 `partial`（30/3）；unregistered 23 条 `absent（未入 30 条清单；2026-10-23 后新增/实现）`；
  gaps 32 条 `ledger_action` = archive 9 / reclassify-or-blocker 7 / deferred-linux 4 / partial 3 / n/a-archived 3 /
  covered-by-base 2 / partial-missing 2 / undetermined 2。`owner` 只用角色，不写人名。
- **无历史时点混用**：`generated_at = 2026-11-11T12:27:08+0800`、`inputs.tree.head = f4628bd7…`（生成时 HEAD，
  即本卡登记提交）；6 份输入 sha 全部记录在 `inputs`。
- `np02.new_source_drift` 1 条：`scene_bvh_ray` 记录 `8dad4d61…` vs 当前 LF `caac8c86…`（由 `d35cab68` 造成，
  晚于清单生成）⇒「已知漂移 9」实为 **10**；按 Chief 裁定 **CPU-only 补采、不归档、不改 2026-11-04 快照**（NP5-03 执行）。
- 30 张 field-table + 3 张 host-table 全部 present 且 sha 与账本一致（checker 覆盖）。

## 3. 落仓与机读校验（复现口径）

- **账本落仓**：逐字节复制 target 定稿 → 落仓副本 sha **仍为 `6d6ff7d9…` / 116,688 B**（未重排 JSON）。
- **机读校验**（`logs/np-gate-ledger-check.log`，`beae8f16…`/862 B）：`python -B scripts/np-gate/check_np5_ledger.py`
  → **19 PASS / RESULT: PASS**。该日志由落仓工具**重跑生成**，与 target 同名日志**逐字节相同**（sha 一致）；
  校验覆盖 schema、np01/np02 与 inventory 一致、30+3 表 present+sha、23 未登记单元存在性、totals 算术（190）、
  owner/action 覆盖、9 archive + 7 reclassify-or-blocker、7 子系统 recheck、无未加否定的门禁通过声明、`scene.cu` 漂移
  与实际一致。
- **工具落仓（Chief 已批 `scripts/np-gate/**`）**：两脚本与 target 源**仅差 1 行文件头注记**（diff 见
  `logs/tool-diff.txt`），**不触碰** freeze script / guards JSON / 既有采集脚本 / guards 输入清单；两脚本文件头已注明
  **「NOT in the 93 guards inputs; adding it to guards requires a user re-sign (NP5-00)」**（`scripts/np-gate/**`
  不在 guards 93 输入内；如需纳入须用户重签）。
- **复现命令**（pinned venv：`target/mjwarp-parallel/venv/Scripts/python.exe`，本机无系统 python）：
  ```text
  python -B scripts/np-gate/gen_np5_ledger.py      # 生成器：输出到 target/ 草稿目录（源逻辑不变）
  python -B scripts/np-gate/check_np5_ledger.py    # 校验器：读 target/ 草稿账本（与落仓副本同字节）
  ```
  两脚本沿用 target 源的 `ROOT` 常量（本机即仓路径 `D:\Rust\mjwarp-rs`）；落仓账本与 target 草稿逐字节相同，
  故 checker 结果直接适用于落仓副本。复现生成时 `generated_at`/`tree.head` 会随运行时刻/提交变化，比对时忽略这两字段。

## 4. 现状事实摘要（账本记录；不构成通过结论）

1. **NP01 逐字段表已 30/30 + host 3/3 完成**（B1–B2e + B3）；顶层 NP01 README 只到 B1 属证据卫生不同步。
   NP01 剩余 = host/device 逐表达式位级对照 + g10-p5 E 指标定稿（2026-10-27 用户裁定）+ Linux 复验。
2. **NP01 未登记产品单元 23 个**（§2 明细）→ **NP5-01b** 新增逐字段表范围（0 窗、后台可做）。
3. **7 个「未实现子系统」重核分流（Chief 裁定 1/2）**：3 个已实现（渲染/BVH、地形/SDF、GJK/EPA）**不建阻塞卡**，
   走 NP5-01b/NP5-03 证据登记；**4 张阻塞卡只登记**：`2026-11-11-np-blocker-flex-collision`、
   `-constraint-solver`、`-solver-pipeline`、`-island-sleep`（不纳入当前排程，是否排 W2/W3/R07 由用户裁决）。
4. **NP02 缺 2 个真实 `__global__` 行**：`render_megakernel`（`render_megakernel.cu`）、`render_depth_seg`（`render.cu`），
   由 `src/render/kernel/mod.rs` 的 `concat!` 装配、扫描口径外；**NP5-03 补行 + 棘轮 +2 + CPU-only 回执**（NVRTC 12.8.93、
   不建设备上下文），属证据登记而非新实现。
5. **`scene_bvh_ray` 第 10 条漂移**：CPU-only 补采回执（不归档、不改历史快照；NP02 计数棘轮同步，NP5-03）。
6. **9 条已知漂移按用户 2026-10-24 裁决 B 归档**（全部 `archive`），不重采不重基线；本片只登记。

## 5. 与依赖的关系

- 本片**不构成 W4 GPU 窗前置**（只是清单冻结）；`p8-entry`/`r10-p8`/`p8-base` 要求的 NP01/NP02 由 NP5-01/02/03 的
  verified 承担。原 NP01/NP02 伞卡 `depends_on:[p10]` **未改动**，完成后由 Chief 按裁定 1 裁决改指/按证据标 verified。
- NP03/04 与 P11 不排窗（等 Linux/双平台用户裁决）；NP01/02 未过前不得 start NP03/04、不得提 P11。

## 6. 非声明

- 本证据**只冻结清单/判据映射/状态与 owner**：**不宣称 NP01/NP02 通过**、不宣称全产品精度门禁通过、不宣称上游等价
  或双平台；账本 `scope`/`boundaries` 与 checker 第 8 项已断言无未加否定的通过声明。
- 不改判据/参考/容差；不重采证据、不重跑窗口、不改 2026-11-04 的 gap 快照语义（用账本 addendum 覆盖，历史文件不改）。
- 零 GPU；不写 `src/`、`tests/`、`fixtures/`；`scripts/np-gate/**` 仅新增两工具并已注明不在 guards 93 输入内。
## 六、复核低项 addendum（卡 K，2026-11-11；append-only，原文不改）

来源：QA 复核卡 `docs/development-evidence/np-gate/np5-00-freeze-check/`（复核对象 `875ae41d`，卡 F r2）。本节只更正记录面；账本仍保持与 target 定稿逐字节相同（`6d6ff7d9…`/116,688 B），不就地改。

- **L1（unregistered 状态注解与词表匹配）**：`np01.unregistered_product_units` 23 条的 `np01_status` 值为 `absent（未入 30 条清单；2026-10-23 后新增/实现）`，是「裸值 + 括号理由」拼接。与 `status_vocabulary.np01 = [verified, partial, missing, absent]` 的匹配口径应为：**括号前裸值 `absent` 入词表，括号内是理由注记**；按裸值全等机读会报 23 个 unknown（QA 复核实测），属匹配口径问题、不是状态错误。因「逐字节相同」要求本卡不改账本；下一版账本应拆 `np01_status` / `reason` 两字段（或把带注记形态写进词表），并给 checker 增该字段断言 —— **留 backlog（本批未做）**。
- **L2（tree.head 归属更正）**：§2 原写 `inputs.tree.head = f4628bd7…（生成时 HEAD，即本卡登记提交）`，对 r2 卡不准确。更正：`f4628bd7` 是**被 abandon 的第一版卡 F（`2026-11-11-np5-00-gate-ledger-freeze`）登记提交**（生成时 HEAD）；r2 卡（`…-r2`）登记提交是 `dec4e7d9`。账本 `inputs.tree.head` 字段本身不改（逐字节要求）。
- **L3（task 草稿字样）**：账本 `task` 字段仍为 `np5-00（草稿；由 target/ 生成，待 Backend 落 docs/development-evidence/np-gate/）`，属生成时草稿态残留；落仓后该字段已过时，但按逐字节复制未改。以本目录 README/receipt 的材料版本记录为准。
- **观察 O1**（`scope` 首句「100%/0」清单完备性措辞易误读为验收结论）：本批不改措辞，留下一步修订 backlog；同句已有「不宣称任何门禁通过」。
- 本节追加后 manifest 已重算（README 之外其余 7 条 sha 不变；含两条 `scripts/np-gate/*.py` repo-relative 条目）。
