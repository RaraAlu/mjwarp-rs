# QA 复验：产品侧扩展像素 dump 修复卡（`8b37670f`）

- 任务：`2026-11-11-render-perworld-l-v2-diag-dump-ext-fix-check`（owner QA；零 GPU、只读复验 + 本卡证据）。
- 对象：修复卡 `2026-11-11-render-perworld-l-v2-diag-dump-ext-fix`，提交 **`8b37670f`**（15 文件 +401/−128，**无任何 `.cu`**；卡 verified）。触发物：我上一张复核卡 `2026-11-11-render-perworld-l-v2-diag-dump-ext-check`（`00bfaef8`）的 B1 + G1–G4 + 低优先项。
- **结论：PASS（0 阻断，2 条低残留）→ 可发诊断轮 #7 token。** B1（schema 1→2）已修且由纯 CPU 单测实测断言；G1–G4 与低优先项均落地；无 `.cu` 改动、内核身份逐字不变（我独立复算 source/PTX）；门禁 unit 1,391/0/402；manifest 10/10。两条低残留均为记录面文字（见 §5），不影响 #7 数据或任何数值，可并入下一张 errata 一并收。

## 1. B1：dump 顶层 schema 1 → 2（实测 ✅）

- `tests/support/render_bvh/light/diag.rs`：`pub const SCHEMA: u32 = 2;`；顶层组装拆为纯函数 `document_from_parts()`，`diagnostics` 由 `attach_diagnostics()` 统一挂载。
- 我独立重跑定向纯 CPU 测试（零 GPU、无需渲染）：
  - lib：`render::kernel::diag::tests::synthetic_record_round_trips_bit_exactly` **ok**、`decode_rejects_wrong_length` **ok**（2 passed / 0 failed）；
  - 集成：`light_diag::tests::schema_constant_and_document_top_level_agree` **ok**（断言 `SCHEMA==2` 且 `document_from_parts` 顶层 `schema==2`）、`rgb_mismatch_entries_always_carry_diagnostics_key` **ok**（未启用 env 时键存在且为 null）；2 passed / 0 failed。
- ⇒ 「实际 dump 顶层 `schema==2`」由被测函数直接断言，不是只改常量。

## 2. G4：解码往返自证 ✅

`src/render/kernel/diag.rs` 新增 `#[cfg(test)]`：按内核布局把合成记录编码成 64 f32 → `DiagRecord::from_slice` 解码 → 再编码，逐字段 f32 位型一致；`face=-3`/`mesh_id=42` i32 位型；`view_dir=-ray_dir` 精确负号位翻转（含 `-0.0→+0.0`）；第 1/第 4 灯槽；保留槽 57–63 恒 0；长度 63/65 拒绝。我重跑 **2 passed**。

## 3. 无 `.cu` 改动 + 内核身份逐字不变 ✅（我独立复算）

- `git diff 8b37670f^..8b37670f -- src/render/kernel/cuda/` = **空**（`.cu` 零改动）。
- 我按 `mod.rs` 的 `include_str!` 顺序把当前 7 个 `.cu` 逐字节拼接：109,384 B / sha256 **`624ab298…`**（与修复前 dump-ext 证据逐位一致）。
- 我用仓库 pinned CUDA 12.8.1 NVRTC + product 选项**重新编译**该拼接源：PTX **347,070 B / `1fd53131…`** —— 与修复前逐字节一致（aa_resolve / depth_seg 源未动，其身份沿用 dump-ext 复核结论：`0bf8af5a…`/`b28e155c…`/27,536、`b546ef08…`/`e8d84963…`/138,540）。

## 4. G1–G4 与低优先项落地 ✅

- **G1**（不存在的 `render_diag.cu`）：README §1/§4、`src/render/kernel/diag.rs` 与 `tests/support/render_bvh/light/diag_ext.rs` 模块文档均改为「DIAG 布局/helper 在 `render_megakernel.cu` 顶部；`MEGAKERNEL_SOURCE` 仍 7 文件拼接；无单独 `render_diag.cu`」；README 仅以「不存在/未创建」语义提及该文件名，并注明原卡 `schedule.files` 曾规划、最终未创建。
- **G2**（记录下标）：README §2 + receipt `record_index` 更正为 `rayid * STRIDE`（单次发射内射线号，无 `worldid` 项），写清 per-world 单发射下 `(world, cam*PIXELS+pixel)` 等价、multi-world 单发射会静默互相覆盖（诊断失效、不影响渲染数值）并登记约束。
- **G3**（计数/行数）：§6 行数改为实测 **478/499/500/496/494**；§1 计数改「本卡 10（8 src + 2 tests）+ 配套卡 5 = 15」；diff 段改按 `git show --numstat`。我核：主卡代码+测试实际 **+360/−12** ✅ 与声明一致。
- **低优先**：ABI 断言文字 7→8 标量 ✅；`megakernel_slots_signature_is_unchanged` → `megakernel_slots_keep_frozen_81_and_append_diagnostic_slot` ✅；README §4 补 PTX 尾 NUL 口径（离线文件 344,182/347,071 vs identity 文本 344,181/347,070）✅；`nlight>=4` 灯槽冲突登记 backlog ✅。
- 边界：`fixtures/`、`scripts/` 零 diff；guards/冻结件未动；判据代码（`tests/support/render_bvh/light.rs` 及 `light/` 下除 diag 模块外）零 diff；不改算术/内核签名/判据/参考/容差；零 GPU。

## 5. 低残留（2 条，记录面；不阻断 #7）

1. **README §2 / receipt 的 `diagnostics` 字段说明未同步新契约**：README「JSON 位置」行与 receipt `dump_schema.json_location` 仍写「未启用 env 时不存在该字段（absent when env unset）」，而 B1 修复后契约是「键始终存在，未采集为 null」（新单测名即 `rgb_mismatch_entries_always_carry_diagnostics_key`）。建议下一张 errata 统一为一句话。
2. **配套卡 numstat 仍写 +86/−8**：实际 `tests/` 5 文件 = **+84/−8**（`diag_ext.rs` +76、4 个 ABI 测试 +8/−8）。该数字只出现在 `8b37670f` 提交信息（历史说明不改写），README/receipt 未复述；建议 errata 记一句。

## 6. 门禁 / manifest / 口径

- 四门禁（本卡 `normify gate` 复跑）：fmt / clippy / clippy-cuda / unit 全过；unit **1,391 passed / 0 failed / 402 ignored**（= 原 1,387 + 新增 4 个纯 CPU 测试）。
- 证据 manifest **10/10** 独立重算（bad=[] / extra=[]）。
- 口径：README §7 errata 与 receipt `errata_fix` 明写「原提交 `bb29e351` 声称 schema 2、实际 1，本次修复；历史说明不改写」；不宣称修复生效、不改判据/参考/容差、G-L2 判定维持未通过（判据 A FAIL）。

## 7. 非声明

本复验只核修复卡实现/测试/身份/边界/记录面，不判定 G-L2 修复效果；诊断轮 #7 由 Chief 另发 token。低残留 1/2 属文档一致性，不影响 dump 数据契约的实现（键存在性已由测试固定）与 #7 数据可用性。
