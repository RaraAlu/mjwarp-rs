# NP5-03：NP02 缺口处置账本（32/32）+ 补 2 个真实渲染 `__global__` 行（CPU-only；零 GPU）

- 任务：`2026-11-11-np5-03-np02-gap-disposition`（owner Backend）。
- 依据：W5 计划 §3；NP02 判据 = **生成内核及精度证据覆盖 100% + 缺失 0 项**（d05:30-31）。
- 本片结论：**32 缺口 32/32 已处置**（每条有去向与理由）；inventory 补入 2 个真实漏登渲染内核行
  （100→102 行）并同步棘轮；**NP02 未验收、4 个未实现子系统仍 blocked（卡 H），不写成通过**。

## 1. 补 2 个真实 `__global__` 行（CPU-only 回执）

- 缺口：NP02 清单（2026-11-04）漏登产品真实渲染内核
  `render_megakernel`（`src/render/kernel/mod.rs::MEGAKERNEL_SOURCE`，7 文件 `concat!`）与
  `render_depth_seg`（`DEPTH_SEG_SOURCE`，3 文件 `concat!`）。
- 补采（**CPU-only NVRTC 12.8**，render recipe：`--fmad=true`、`--include-path=include/warp-headers`、
  `--std=c++17`、`--gpu-architecture=compute_70`、`--device-as-default-execution-space`、
  `f32-render` gate；零 GPU、零窗）：

| entry | source sha256 | ptx sha256 | ptx bytes | gate |
| --- | --- | --- | --- | --- |
| `render_megakernel` | `cf356b72…` | `b63a51dd…` | 348,459 | pass |
| `render_depth_seg` | `1e3f273c…` | `31912377…` | 138,857 | pass |

- 回执/实体：`docs/development-evidence/np02-evidence/new-kernels-2026-11b/receipts/`
  （`<entry>-<ptxSha>.json` + 同名 `.cu`/`.ptx`）；补采索引 `np02-evidence/receipt.json` 追加 2 条
  （25→27）；采集工具沿用入仓 `scripts/np02-evidence/np02-capture-new-kernels.py`（本片为其新增
  2 个多文件拼接单元与 list-relpath 支持）。
- **计数快照更新**（`inventory.counts` 与 `tests/precision_evidence_gate.rs` 同步）：

| 计数 | 旧 | 新 |
| --- | --- | --- |
| windows_implemented_paths | 100 | **102** |
| kernel_rows | 96 | **98** |
| evidence_present | 100 | **102** |
| evidence_verified | 89 | **91** |
| receipts_total / file_hash_ok | 713 / 713 | **715 / 715** |
| receipt_ptx_entities_verified | 709 | **711** |
| coverage_pct_source_identity | 89 | **89.2** |
| gap_items | 32 | **32（不变）** |

- `tests/precision_evidence_gate` 4/4 PASS（计数快照、指针可解析、状态词表、4 个渲染内核
  `f32-render` 门禁白名单 + 14 行 verified 守护）。

## 2. 32 缺口处置账本（逐条见 `evidence.json.gap_disposition.items`）

| 去向 | 条数 | 内容 |
| --- | --- | --- |
| `archive`（裁决 B） | 9 | 已知漂移：历史版本证据，不重采不重基线 |
| `archive-derived` | 2 | resident_mass_factor / resident_mass_substitute：证据由 rigid_mass_solve 回执承担 |
| `closed-implemented` | 3 | 渲染/BVH、hfield/SDF、GJK/EPA 三条「未实现」缺口**已在后续片实现**（回执到位：渲染 2 行本片补入；collision hfield/sdf/GJK 2026-11-04 补采 verified）——按证据关闭，不是「写成通过」 |
| `blocked-with-card` | 4 | flex collision / constraint / solver / island：**真未实现**，实现阻塞卡（卡 H）已登记，不计覆盖 |
| `n/a-with-reason` | 6 | 3 部分覆盖（IO/CLI、block_cholesky 其余、完整语义）+ 3 不适用（tendon_wrap 辅助源、rne_acc 组合、FIELD_BATCH 打包） |
| `recorded-undetermined` | 2 | 上游 codegen 符号级映射、Warp 编译器库内部精度边界：非产品产物，保持未判定记录 |
| `recorded-not-restored` | 2 | T4 历史捕获实体、4/713 PTX 实体不在仓：owner 指令不补造，已入库实体哈希复核 |
| `deferred-w7` | 4 | Linux 三层集成 / 工具链 / NP03 Linux / NP04 Linux：用户解冻后集中回测 |

- **NP01 内部字段 20 个**：由 NP5-01 逐字段类型表关闭（30 路径 + 3 host 站点 = 33 表）。
- 处置纪律：**未实现子系统只记 blocked-with-card，不计覆盖、不写成通过**；`gap_items` 仍 32
  （原清单条目不删除，处置以本账本为准）。

## 3. 非声明与边界

- 不宣称 NP02 通过/覆盖 100% 已验收；4 个未实现子系统仍阻塞（卡 H）；不宣称双平台。
- 写域：`np02-evidence/**`（inventory 更新 + addenda + 新回执/索引）、`np-gate/np5-03/**`、
  `tests/precision_evidence_gate.rs`（快照同步）、`scripts/np02-evidence/np02-capture-new-kernels.py`
  （补采单元）；不改判据/参考/容差；零 GPU、零窗。
