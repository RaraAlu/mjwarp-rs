# QA 复核：实体卡 §2.4 字典适配 fix（`bab01507`；零 GPU）

- 任务：`2026-11-08-render-perworld-l-v2-criteria-entity-c1-dict-fix-check`（owner QA、p80、P8-渲染与BVH）
- 复核对象：实体卡 fix 提交 **`bab01507`**（parent `569e636c`；卡 `2026-11-08-render-perworld-l-v2-criteria-entity`，**addenda=3**、自登记 verified）
- 依据：已签 spec v3 `bb064fe0…` §2.4（`c1_rgb_changed_px = {"0->1": {magnitude, bitwise}, ...}`）与 G-L0 `readout_c1_c2.py`（`4a054aba…`）
- 基线：HEAD `bab01507`、树 clean；**只读被复核内容 + 只写本卡证据；零 GPU、不采集、不开窗**

## 0. 结论

**PASS（0 阻断）。可用于 G-L2 冻结。**

- `check_capture_c1` 现返回 `CaptureC1Check::{Compared, Skipped(&'static str)}`：逐对 world 断言实体 C1 **magnitude == capture magnitude**；capture 有 `bitwise` 时逐对断言实体 C1 bitwise == capture bitwise；旧数组结构 / 缺字段 / 缺世界对 → `Skipped(原因)`，GPU 用例以 `{:?}` 显式打印（不静默）。
- bitwise 口径正确：`rgb_bitwise_changed` = 任一通道 `to_bits()` 不同，等价 readout `np.any(x.view(int32) != y.view(int32), axis=-1)`；pilot 独立复算 **43,604 / 41,815**，与 `readout.json` 逐值一致。
- 自证覆盖全部要求：字典一致→Compared、仅 magnitude→Compared、magnitude/bitwise 不一致→必败、缺字段/旧数组/缺 1→2→Skipped、完整结构不得误判跳过；独立复跑 **16 passed / 0 failed / 2 ignored**。
- 边界干净（3 个 tests + 实体卡证据 + 卡 addenda=3 + runs）；行数 478/499/487 ≤500、无新模块源；`fixtures/src/scripts` 与 spec 零 diff；A/B/C 语义与 384/576 未动；manifest 8/8；四门禁 fmt/clippy/clippy-cuda exit 0、unit **1387/0/402**。

## 1. CaptureC1Check 语义（逐条核）

| 输入 | 行为 |
| --- | --- |
| `c1_rgb_changed_px` 缺失 | `Skipped("capture.json 未记录 …（旧参考无此字段）")` |
| `c1_rgb_changed_px` 为旧数组 `[c1_0, c1_1]` | `Skipped("…旧数组结构；spec v3 §2.4 要求 {"0->1":…,"1->2":…} 字典…")`（显式拒绝、不做静默兼容） |
| 缺任一 world 的 `magnitude` | `Skipped("…缺 0->1/1->2 magnitude 计数")` |
| 两对 world `magnitude` 齐全 | 逐对 `assert_eq!(capture magnitude, 实体 c1_rgb_changed[i])` |
| 且 capture 含 `bitwise` | 逐对 `assert_eq!(capture bitwise, 实体 c1_rgb_bitwise_changed[i])` → 不一致即 panic 失败 |
| 全部一致 | `Compared` |

- **旧数组结构显式拒绝的决定合理**：`[c1_0,c1_1]` 从来不是 spec §2.4 形态（那是旧实体期望，已在本轮前被 spec 口径取代）；采集侧 `verify_capture_metadata` 强制字典，G-L1 参考必然是字典；显式 `Skipped(原因)` 不会被误当成通过，且 GPU 用例打印原因。无自证覆盖缺口。
- 完整结构不会被误判跳过：自证用 `assert!(!matches!(..., Skipped(_)))` 显式覆盖。

## 2. bitwise 口径独立复算

- 实体：`rgb_bitwise_changed(left,right) = any_channel(left[c].to_bits() != right[c].to_bits())`，`CCounts.c1_rgb_bitwise_changed` 按 `rgb_f32` 逐像素累计。
- readout：`np.any(x.view(np.int32) != y.view(np.int32), axis=-1)`。f32 的 `view(int32)` 与 `to_bits()` 是同一 32-bit 位型比较（含 NaN 位型），逐像素等价。
- pilot npz 独立复算（`artifacts/c1-bitwise-recompute.json`，rgb npz sha 记录在案）：

| 量 | 0→1 | 1→2 | 与 readout.json |
| --- | --- | --- | --- |
| C1 magnitude | 38,285 | 40,644 | 一致 |
| **C1 bitwise** | **43,604** | **41,815** | 一致 |

## 3. 自证覆盖（独立复跑 16 passed / 2 ignored）

`capture_c1_cross_check_compares_spec_2_4_dict_counts`：字典一致→`Compared`；仅 magnitude→`Compared`；magnitude 不一致→panic；bitwise 不一致→panic。
`capture_c1_cross_check_reports_explicit_skip_reasons`：缺字段→`Skipped`；旧数组→`Skipped`；缺 1→2→`Skipped`；完整结构→**不得**为 `Skipped`。
`pilot_c1_crosscheck_matches_readout`（ignored，真实 pilot 数据）：断言 `C1 magnitude=[38_285,40_644]`、**`C1 bitwise=[43_604,41_815]`**；旧 pilot capture.json 无 §2.4 字段 → 显式 `Skipped("…旧参考无此字段")`；用真实 counts 构造 §2.4 字典 → **`Compared`**。

我独立跑该 ignored 用例：`pilot C1=[38285, 40644] … verdict=PassMargin`，并打印 `旧 pilot capture.json（无 §2.4 字段）C1 交叉核对 = Skipped("capture.json 未记录 c1_rgb_changed_px（spec v3 §2.4；旧参考无此字段）")` —— 跳过原因可见、非静默。

## 4. 判据语义未动

- `criterion_a` / `criterion_b` / `criterion_c` 的 C1 magnitude、C2 depth/seg、诊断项、`ACCEPT=384` / `MARGIN=576`、`PAIRS=[(0,1),(1,2)]`、相机归属、`verdict`、无 allowlist —— diff 中均未改；仅**新增** `CCounts.c1_rgb_bitwise_changed`（不进 verdict）与 `check_capture_c1` 适配。
- `check_capture_provenance` 仅等价重构（合并 assert、同一组期望值），负例自证仍 20 项。

## 5. 边界 / 规模 / 门禁 / 证据

- `bab01507`（parent `569e636c`）只动 3 个 tests 文件 + 实体卡证据 + 卡（addenda=3）+ runs；**无新增文件**（全 M）；`fixtures/`、`src/`、`scripts/`、spec 目录零 diff。
- 行数（git-blob 换行口径）：`render_per_world_light.rs` **478**、`light.rs` **499**、`light/tests.rs` **487**，均 ≤500；未新建模块源。
- 实体证据 manifest **8/8** 独立重算；编码卫生通过。
- 四门禁：fmt/clippy/clippy-cuda exit 0；unit **1387 passed / 0 failed / 402 ignored**（+1 来自本卡新增自证）。

## 6. 观察项（均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 信息 | capture 只记 `magnitude` 时实体仍判 `Compared`（不比对 bitwise），而采集侧 `verify_capture_metadata` 要求两个计数都在 | 分层合理（实体是防御性交叉核对，字段完整性由采集侧强制）；无需改 |
| O2 | 信息 | 旧数组结构走 `Skipped` 而非失败：若 G-L2 误用旧参考会「打印跳过」而不是失败 | G-L1 参考必为字典结构；`Skipped` 原因已打印进 G-L2 日志，可追溯。无需改 |
| O3 | 信息 | `light.rs` 已 499/500 行 | 后续再加逻辑需先拆 |

## 7. 非声明

本卡只做只读复核与独立复算：**不宣称** G-L2 已通过、不宣称渲染数值/上游等价结论；未修改被复核内容、spec、pilot 证据与签名记录。实体在 G-L2 真跑前不得声称通过。
