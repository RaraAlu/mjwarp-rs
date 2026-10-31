# 球面 a-order refix 独立复核（QA，只读）

任务：`2026-10-31-render-sdf-sphere-dot-a-order-refix-check`
被核：`2026-10-31-render-sdf-sphere-dot-a-order-refix`（pin **`fa604008`**）
复核 start 时 `revision.before` = **`2006544f`**
**结论：pass / 阻断 0 / 需修 1（证据措辞，轻微，不重跑）/ 观察 8。**
只读：零 GPU、零窗口、零编译、零冻结；未改判据/参考/代码/历史证据。

## 0. 一句话

本片把球面求解的 `a` 从 `dot3_ch0` 换回 `dot3_ba`（`(1,0,2)`）、删 `dot3_ch0`、按 RCA 重写注释，并同笔清掉 `render_shade.cu` L10/L14 的 `dot3_ch0` 注释残债。我按 committed blob 独立重算 6 段拼接：**88,865 B / `778216b2…` → 89,041 B / `10070226…`**（与证据一致）；`render_math.cu` post-fix blob **`7a43e575…` 与历史修复 `8126868c` 逐字节相同**（「同一 diff」按字节成立，比对照文字更强）；PTX 两条独立记录互证 `7782aeb1…`/268,589、`aa_resolve` 不变；`013d8d4a + fa604008` 叠加后 attempt4 两项前置同时在场。唯一需修是证据/提交消息里的「`render_math.cu`（290 行）」不实，实测 **308 行**（309→308）。

## 1. 复核方法（全部只读命令，可复跑）

- 范围：`git show --stat fa604008`、`git diff --name-only 8bc0f2b3..fa604008 -- src tests fixtures ...`
- 形态/行数：`git show <rev>:src/render/kernel/cuda/render_math.cu`（总行/空行/纯注释/代码分类计数）、`git grep -n dot3_ch0 / 'float a = dot3_ba' / det`
- 与历史修复比对：`git rev-parse 8126868c:…render_math.cu` 与 `fa604008:…render_math.cu` 的 blob 相等性；两段 patch 文本逐字比较
- 内核身份（独立重算）：`MEGAKERNEL_SOURCE` 的 6 个 `include_str!` 片段按 `mod.rs` 顺序从 committed blob 拼接（before `8bc0f2b3` / after `fa604008`），sha256 + 总字节；`AA_RESOLVE_SOURCE` 同法单独重算
- PTX：不独立复算（零编译），改为两条独立证据互证（原 fix `sphere-dot-a-order-fix/logs/kernel-identity.txt` 与本片 `materials/kernel-identity.txt`）
- 证据卫生：上游 manifest 逐行对 committed blob 重算；`src-diff.patch` 与真实 `git diff` 规范化比较
- 门禁：`.normify/runs/2026-10-31-render-sdf-sphere-dot-a-order-refix/*.json` 与证据 `logs/gate-*.txt` 对比（provenance）；unit 数取 runs 记录；`validate --fingerprints` 只读复跑
- 叠加态：`git diff --name-only fa604008..HEAD -- src`、`git grep reject_sdf_geometry HEAD -- '*.rs'`、`git grep 'float a = dot3_ba' HEAD -- src/render/kernel/cuda`
- 原始输出：`materials/selfcheck.txt`（生成脚本 `materials/selfcheck.ps1`，UTF-8 BOM —— Windows PowerShell 5.1 读取中文脚本所需；其余文本证据 LF 无 BOM）。

## 2. 结果（9 项 check）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `scope.pin_commit` | ✓ | `fa604008` 共 **19** 文件 = 2 个 `.cu` + 卡/模块 md/runs 6 + 证据 11；`tests/fixtures/scripts/Cargo*/criterion-v2` 0 改动 |
| 2 | `render_math.shape` | ✓（另 P1） | **309 → 308** 行；`float a = dot3_ba(...)` 在 275 行；`dot3_ch0` 可执行引用 0，仅 1 处解释性注释（255 行）；`b`/`dd`/融合 `det` 在 276/277/279 行未动 |
| 3 | `render_shade.comments` | ✓ | 仅 2 行注释改动（L10/L14 各 1 行，`dot3_ch0` 残债清零），457 行、无代码行变更 |
| 4 | `same_diff_as_8126868c` | ✓ | post-fix blob `7a43e575b7b73b1abd69a7f70cda733099255a04` = `8126868c` 的同文件 blob；patch 文本逐字相同；before blob `33efcb27…` |
| 5 | `kernel.identity` | ✓ | 独立重算：**88,865 B / `778216b2…` → 89,041 B / `10070226…`**；`AA_RESOLVE_SOURCE` **2,364 B / `0bf8af5a…`** 不变；PTX `7782aeb1…`/268,589 两条独立记录互证（未独立复算） |
| 6 | `evidence.manifest_and_patch` | ✓ | 上游 manifest **10/10** 对 committed blob 逐字节相符；`src-diff.patch` 与真实 diff 逐字相同 |
| 7 | `gates` | ✓（另 O4） | unit runs **1111 / 0 / 397**、exit 0；fmt/clippy/clippy-cuda passed；`no-run` 含 Finished + EXIT=0；复跑 `validate --fingerprints` = 0 错 / 5 警告 / 无 drift |
| 8 | `claims.wording` | ✓（另 P1） | README/receipt 有「`render_math.cu`（290 行）」→ 实测 308（P1）；口径「本形态只保证 scene-sdf 内用于 attempt4 的位级比较，是否位级待 attempt4 验证」「不声明 SDF 通过」✓；criterion v2 §2 受控豁免对象为 scene-a 球轮廓像素，与 README 口径一致 |
| 9 | `stacked_state` | ✓ | `fa604008..HEAD -- src` 空；HEAD host `.rs` 守卫 0、megakernel 405 行；`a = dot3_ba` 在 6 段拼接内 ⇒ attempt4 两项前置同时在场 |

## 3. 需修 1（证据措辞，轻微；不改代码、不重跑）

- **P1「290 行」不实**：`render_math.cu` 实测 **308 行**（本片前 309；其中空行 30、纯注释 49、代码 229）。该数字出现在上游 README 首条、`receipt.json` 的 `change.render_math_cu.lines`、以及 `fa604008` 提交消息；290 与任何行数口径（总/非空/注释/代码）都不符。
  修法：改为 **308**（或写「309 → 308」）；实现与证据其余部分不受影响，不改代码、不重跑门禁。

## 4. 观察 8（记录级接受）

- **O1 PTX 未独立复算，但有两路互证**：原 fix `8126868c` 的 `logs/kernel-identity.txt` 与本片 `materials/kernel-identity.txt` 都给出 `7782aeb1…`/268,589；source 差异（`eb0e717f…` → `10070226…`）由 `render_shade.cu` 注释清理解释（注释计入拼装源码哈希），与本片声明一致。
- **O2「与 8126868c 同一 diff」按字节成立**：post-fix `render_math.cu` blob 与 `8126868c` 完全相同、patch 文本逐字相同 —— 这比文字对照更硬。
- **O3 `dot3_ch0` 残留仅注释**：唯一命中是 `render_math.cu:255` 的解释性注释（说明「只用该顺序复现参考 0/44」）；可执行引用 0。
- **O4 门禁快照与 `runs/` 非同一次**：证据 `logs/gate-*.txt` 时间 05:13:15–05:14:52，`runs/` 为 05:15:20–05:15:34；clippy/unit 的 `log_sha256` 不同；判定数值一致（unit 1111/0/397）。
- **O5 validate 快照含 drift**：0 错 / 6 警告（含 `evidence/fingerprint-drift mjwarp.render.kernel`），同笔提交已刷新该指纹；HEAD 复跑 0 错 / 5 警告 / 无 drift。
- **O6 `no-run.txt` 为裁剪转写**（Finished + EXECUTABLE + EXIT=0）。
- **O7 host 侧 `.cu` 注释残留**（`render_common.cu:391/397` 仍称 host 拒绝 SDF）在同一 HEAD 上存在（已由 U1b redo 复核卡 P1 记录）；本片与 attempt4 材料都**不得**把这两行注释当事实引用（设备能力由窗口验证）。
- **O8 attempt4 材料 v2 的身份引用一致**：`9f12002d` 的 prepare checklist 写 `10070226…/7782aeb1…/268,589`，与本片实测/证据一致（材料本身不在本卡范围）。

## 5. 边界与不宣称

- 只读：零 GPU、零窗口、零编译、零冻结；不改 `src/`、不改他人证据、不改判据/参考/fixtures。
- 不宣称 SDF 通过、不宣称渲染数值验收通过、不宣称上游等价、不宣称精度达标、不宣称 attempt4 会通过。
- 本卡只核「源改动形态 + 身份换代 + 证据一致性 + 叠加态」；scene-sdf 是否位级只能由 attempt4 窗口给出。

## 6. 限制

- PTX 与 GPU 用例未重跑（零构建纪律）；PTX 仅两路记录一致，source 身份为独立重算。
- `scene-a` 球轮廓豁免的正确性由已签 criterion v2 承接，本卡只核 README 引用口径一致，不重审判据。
