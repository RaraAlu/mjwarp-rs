# NP5-03 复核（QA）：NP02 缺口 32/32 处置 + 2 个真实渲染 `__global__` 行补登（CPU-only；零 GPU）

- 复核对象：卡 `2026-11-11-np5-03-np02-gap-disposition`；内容提交 **`d46f435f`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 1 低 + 2 观察）**。

## 1. 32 缺口处置账本（逐条去向）

- **32/32 每项都有 disposition/reason/closed**，`unclassified=0`；8 类计数
  `9 archive + 2 archive-derived + 3 closed-implemented + 4 blocked-with-card + 6 n/a-with-reason +
  2 recorded-undetermined + 2 recorded-not-restored + 4 deferred-w7 = 32`，与 evidence/receipt 一致。
- **4 个真未实现（flex collision/constraint/solver/island）只记 blocked-with-card（卡 H），
  `counts_as_coverage=false`**，未写成通过；receipt 同时给 `unimplemented_remaining=4`。
- **3 个 closed-implemented 是「按证据关闭」而非放行**：渲染/BVH、hfield/SDF、GJK/EPA——后三者 2026-11-04
  补采 verified 我在 NP02 回执索引可查，渲染 2 行本片补入并 verified。
- `recorded-not-restored` 2 条（T4 历史捕获实体、4/713 PTX 实体不在仓）遵守「不补造」；`deferred-w7` 4 条
  与用户 2026-10-24 暂停 Linux 的决定一致；`gap_items` 保持 32（原清单不删、处置以账本为准）。

## 2. 2 个新行：源身份 / 回执 / 棘轮

- inventory `100→102` 行、kernel_rows `96→98`、evidence_verified `89→91`、receipts `713→715`、
  PTX entities `709→711`、源身份 `89→89.2`、gap_items `32 不变`。
- 我逐个核产物（不是只看 receipt 自述）：`render_megakernel` `.cu` 110,879 B `cf356b72…` /
  `.ptx` 348,459 B `b63a51dd…`；`render_depth_seg` `.cu` 53,552 B `1e3f273c…` /
  `.ptx` 138,857 B `31912377…`；与 NP02 回执索引 27 条（25+2）内的身份一致，没有产物替换。
- capture-report：`nvrtcVersion 12.8`、`cpuOnly=true`、`gpu=false`、`gateOk=true`、recipe `render`——
  **零 GPU 属实**；PTX 仍不入仓（产物在 `np02-evidence/**` 证据目录，属既有回执口径）。
- **棘轮同步**：`tests/precision_evidence_gate.rs` 只同步快照数字（100→102/96→98/89→91/713→715/709→711/
  89→89.2）、`NEW_ROWS` 12→14（加 2 个渲染内核）、`f32-render` 白名单扩 2 行；逐 hunk 核对**无断言削弱**。
  我亲自重跑 `cargo test --test precision_evidence_gate` = **4 passed / 0 failed**。

## 3. 发现

### 低
- **L1 `inventory.counts.unimplemented=7` 与 `unimplemented_remaining=4` 并存易误读**：7 是历史快照
  （3 条已被本片按 evidence 关闭为 closed-implemented），剩余 4 见 receipt/NP5-02 状态。建议在 inventory
  旁加注「历史快照，剩余以 unimplemented_remaining 为准」，或 append-only addendum 说明（不改历史字节、
  不计覆盖）。两处数字各自都有出处，未见矛盾宣称，故不阻断。

### 观察
- **O1 测试快照更新属设计内同步（正向）**：漏登 2 行是清单缺口，补采后同步快照并扩白名单，不是放宽门禁；
  我重跑 4/4 通过，`NEW_ROWS` 14 行守护仍有效。
- **O2 计数派生关系可复算**：`inventory 102/98/102/91`、`receipts 715/711`、`27 条回执索引`、
  `np01_internal_fields 20` 我逐项在 evidence 对上；32 缺口条目不删、只加处置，账本可回溯。

## 4. manifest / 门禁 / 边界 / 非声明

- manifest 4/4（561 B，sha/bytes 全对）；selfcheck 全 PASS；门禁 fmt/clippy/clippy-cuda exit 0、
  unit **1,406/0/402**；policy_scan 0 命中；编码 UTF-8/LF 无 BOM、CR、NUL。
- 写域与卡内声明一致：`np02-evidence/**`、`np-gate/np5-03/**`、`tests/precision_evidence_gate.rs`（快照同步）、
  `scripts/np02-evidence/np02-capture-new-kernels.py`（补采单元）；未改判据/参考/容差。
- 哈希：README `ace95b15…`/4,427、evidence `8c513f54…`/19,150、receipt `8afbb1b2…`/5,103、
  manifest `dddb1d53…`/561、selfcheck `a22b38f4…`/2,118。
- **不宣称** NP02 通过、覆盖 100% 已验收、双平台等价；4 个未实现子系统仍 blocked。
