# NP5-01a QA-ADD 复核（QA）：跨片复核修正 8 项（1 一般改判 + 7 低项）

- 复核对象：卡 `2026-11-11-np5-01a-qa-cross-slice-addenda`；内容提交 **`e077765a`**（登记 `a1b65580`）。
- 性质：只读复核（零 GPU、零窗、不改判据）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 1 低 + 2 观察）**；8 项处置与 QA 记录逐项对应、append-only 成立。

## 1. G1 改判落实（2 站点）

- `np5-01a-a1a/addenda.md §5`：`mass_solve:resident_mass_factor_派生:accumulation:0` **`identical` → `not-decidable`**，`not_decidable_reasons:["no-lineinfo","frozen-artifact-only"]`，note「tile 分支设备级不可判；A1 已 codegen、由 RE-1 覆盖」✅
- `np5-01a-a1b/addenda.md`：`mass_solve:resident_mass_substitute:accumulation:0` **`identical` → `not-decidable`**，同上理由码与 note ✅
- 两处均明写「`evidence.json` 字节不改，以本文件为准」，与 append-only 口径一致。
- 复核依据与我原卡一致：同一覆盖文本（含 tile 分支）不得在不同站点分别判 `identical` 与 `not-decidable`。

## 2. 7 低项逐项落实

| # | QA 记录 | 落点与内容 | 结论 |
| --- | --- | --- | --- |
| L1 | A-1a `missing_evidence`「2603 entries」 | A-1a addenda §5 改为 **303**（6×48+15），并注明原字节不改 | ✅ |
| L2 | A-1a README §3 4 条 vs `requires_decision` 3 条 | A-1a addenda §5 补第 4 条 = `accumulation:2` identical 前提（收缩许可、未证位级相等） | ✅ |
| L3 | 词表引用旧 `fcbec8f9…`/4,560 | A-1a/A-1b addenda 统一为现版 **`46282084…`/4,867**，并标注旧值为历史记录 | ✅ |
| L4 | RE-0 `residual_diff_types` 6 vs README 5 类 | `re0/addenda.md` ④ 明列 [5]/[6] 同项，统一 5 类并给出五类清单 | ✅ |
| L5 | 三个脚本硬编码本机绝对路径 | `re0/addenda.md` ⑤ 说明开发用、移植需改路径与 pinned 环境 | ✅ |
| L6 | `a1_*` 脚本用途未说明 | `re0/addenda.md` ⑥ 说明为 A1 离线 codegen 路线（对应 A1-final 6 族/12 变体），与 RE-0 判级脚本用途区分 | ✅ |
| L7 | 措辞统一 | 三处 addenda 均写「统一**归一化指令流等价**；不写字节相等/位级等价」 | ✅ |

## 3. append-only 与 manifest（我逐文件核 blob）

- **三片 landing 字节未改**（landing 提交 → `e077765a` 逐 blob 比对完全相同）：A-1a 的 `evidence.json`/`README.md`/`receipt.json`；A-1b 同名三件；RE-0 的 `README.md`/`receipt.json`/`re0-provenance.json`/`g1-compare.json`/`g2-compare.json`。
- **A-1a addenda 自身也是 append-only**：旧 2,347 B 是新 4,270 B 的**精确前缀**（`new[:2347] == old`）。
- **manifest 派生重算**（我独立重算 sha256+bytes + 集合相等、不自列）：A-1a **5/5**（含 addenda）、A-1b **5/5**（含 addenda）、RE-0 **11/11**（含 addenda）、`qa-cross-slice-addenda` **2/2**；全 bad=0。
- `qa-cross-slice-addenda/receipt.json`：`corrections_total=8`（`general_items=1`+`low_items=7`）、`reclassified_sites=2` 且列出两个精确 site id、`vocab_current_sha256=46282084…`/`vocab_current_bytes=4867`（我复算一致）✅

## 4. 边界/门禁/编码

- 提交 14 files = A-1a/A-1b/RE-0 三片 addenda + 各自 manifest + `qa-cross-slice-addenda/`（README/receipt/manifest）+ `.normify`（卡+4 gate 日志）；无 `src/tests/fixtures/scripts` 改动；零 GPU。
- 门禁：fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**。
- 编码：改动文件全部 UTF-8/LF、无 BOM/CR/TAB/NUL；树内无通过类/覆盖率/位级等价正面表述。

## 5. 发现

### 低
- **L1 派生计数仍未给**：G1 改判后 `evidence.json` 直方图仍是 `identical 12 / not-decidable 2`；addenda 只写「以本文件为准」，未给改判后的派生计数（A-1a→`identical 11 / not-decidable 3`，A-1b→`identical 10 / not-decidable 2`）。与我 A-1a O4 同型，建议在下一张 errata/收口卡补一行派生计数，避免下游按旧直方图汇总。

### 观察
- **O1 时序注记**：A-1a addenda §5 仍写「acc:1 `not-decidable` **待裁**」；Chief 随后已裁定 **`engine-semantic`**，由 RE-1 修正稿覆盖——本条不要求改 QA-ADD。
- **O2**：本卡 G1 实际落在 **QA-ADD 卡**（非 RE-1 卡）；我此前三张复核卡写「由 RE-1 修正」系按当时口径，落地路径以本卡为准（不作历史改写，仅在此记录）。

## 6. 非声明

本复核对 G1/7 低项的落实与 append-only 不变式负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
