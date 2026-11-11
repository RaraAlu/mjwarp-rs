# NP02 addenda（NP5-03，2026-11-11；append-only，不改历史字节）

## 1. 补 2 个真实 `__global__` 行（inventory 100→102）

- 发现：NP02 清单（2026-11-04）漏掉 2 个产品真实渲染内核行：
  `render_megakernel`（`src/render/kernel/mod.rs::MEGAKERNEL_SOURCE`，7 文件 `concat!`）
  与 `render_depth_seg`（`DEPTH_SEG_SOURCE`，3 文件 `concat!`）。
- 补采（CPU-only NVRTC 12.8，render recipe：`--fmad=true --include-path=include/warp-headers
  --std=c++17 --gpu-architecture=compute_70 --device-as-default-execution-space`，f32-render gate）：
  | entry | source sha256 | ptx sha256 | ptx bytes | gate |
  | --- | --- | --- | --- | --- |
  | render_megakernel | `cf356b72…` | `b63a51dd…` | 348,459 | pass |
  | render_depth_seg | `1e3f273c…` | `31912377…` | 138,857 | pass |
  回执/实体：`new-kernels-2026-11b/receipts/<entry>-<ptxSha>.json` + 同名 `.cu`/`.ptx`；
  补采索引 `receipt.json` 追加 2 条（25→27）。
- 计数快照更新（`inventory.counts` 与 `tests/precision_evidence_gate.rs` 同步）：
  `windows_implemented_paths` 100→102、`kernel_rows` 96→98、`evidence_present` 100→102、
  `evidence_verified` 89→91、`receipts_total` 713→715、`receipts_file_hash_ok` 713→715、
  `receipt_ptx_entities_verified` 709→711、`coverage_pct_source_identity` 89→89.2；
  `gap_items` 32 **不变**（缺口处置见 `np-gate/np5-03/evidence.json`）。
- 本 addendum 不改 `README.md`/`gap.json`/`receipt.json` 历史内容；inventory 由本片显式更新并留
  `np5_03_update` 字段记录 before/after。

## 2. 32 缺口处置（摘要）

- 9 已知漂移 → archive（裁决 B）；2 派生 → archive-derived；
- 3 条「未实现」缺口（渲染/BVH、hfield/SDF、GJK/EPA）→ **closed-implemented**（后续片已交付，
  其中渲染 2 行由本片补入；均已核回执，不是「写成通过」而是证据到位）；
- 4 条真未实现（flex collision/constraint/solver/island）→ blocked-with-card（卡 H 已登记，不计覆盖）；
- 3 部分覆盖 + 3 不适用 → n/a-with-reason；2 未判定 → recorded-undetermined；
- 2 部分缺失 → recorded-not-restored（owner 指令不补造历史实体）；
- 4 Linux → deferred-w7（用户解冻后集中回测）。
- 逐条处置（32/32，含理由与证据指针）见 `docs/development-evidence/np-gate/np5-03/evidence.json`。

## 3. 非声明

本 addendum 不宣称 NP02 通过、不把未实现子系统写成通过、不改判据/参考/容差；零 GPU、零窗。
