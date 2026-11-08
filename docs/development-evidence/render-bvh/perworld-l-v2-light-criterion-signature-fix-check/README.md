# 签名留证 fix 轮最小复核（QA 只读）

任务：`2026-11-08-render-perworld-l-v2-light-criterion-signature-fix-check`（owner QA）。日期：2026-11-08。
复核目标：Arch 的 fix 提交 **`ba420eb6`**（`Normify-Fixes: 2026-11-08-render-perworld-l-v2-light-criterion-spec-v3-signature`）。
复核基线：我在 `2026-11-08-...-signature-check` 卡的观察 O1/O2/O3（另 O4 为信息项、不修）。
边界：只读核验 + 只写本卡证据目录；零 GPU；**不改签名记录**；spec 字节零改动。

**结论：PASS（O1/O2/O3 全部闭环；0 阻断、0 新观察）。**

## 1. 不变项（先说）

- **spec.md 三处同 sha**：`ba420eb6^:spec.md` blob、`ba420eb6:spec.md` blob、工作树，sha256 均 **`bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d`**、39,379 B、344 行；`ba420eb6^..ba420eb6` 对 `spec.md` **零 diff** ✅。
- **receipt 新 sha/bytes** = **`1fb51eca56b4fbf2aa0e96b25b1999ca4805249d36309ca475dd6740b447487b` / 22,778 B**（fix 前 `7f79bdf1…`/21,935 B，独立重算）✅。
- **manifest 3/3**：spec `bb064fe0…`/39,379、receipt `1fb51eca…`/22,778、README `31c9fe8d…`/4,561；`# base dir:` 在位、不自列 ✅。
- **编码卫生**：四文件 UTF-8/LF、CR=0、TAB=0、无 BOM ✅。
- **改动面**：`ba420eb6` 仅卡 JSON、4 个 runs JSON、README/manifest/receipt；`src/`、`tests/`、`fixtures/`、`scripts/` **零 diff** ✅。
- **receipt 其余块未动**：`signature`（signer/signed_at/scope/before==after/spec_frozen/note）、`s2_preapproval.conditions`、`not_extended`、`signature_status`、`spec`、`criteria`、`go_list` 等与 fix 前一致（逐项比对）✅。

## 2. O1 闭环（生效口径标签）

| 项 | 实测 | 判定 |
| --- | --- | --- |
| `kind` | `criterion-spec`（原 `criterion-spec-draft`） | ✅ |
| `title` | “perworld L v2 light criterion spec (v3, signed 2026-11-08: …)” | ✅ |
| `pre_signature_label` | 保留旧 `kind`/`title`，注 “pre-signature labels kept for history only; the effective labels are the kind/title above” | ✅ |
| `non_claim[0]` | “S-1 signed and effective 2026-11-08; S-2 and S-3 not signed; the spec header's 'pending user signature' wording is frozen as written and the signature fact is receipt.signature” | ✅ |

即：生效态与签名前留史标签分离，`status=effective` 不再有自相矛盾的 draft 表述；spec 头注冻结的歧义在 `non_claim` 与 `signature.note` 两处都指向 `receipt.signature`。

## 3. O2 闭环（删未获批准的「连续」）

- `s2_preapproval.text` 现为：「预批「采集窗 authoring 例外」——**常规 util<40 不达标时**允许 authoring-only util<60；条件：① 仅采集窗生效，G-L0 pilot 仍按常规阈值、不适用例外；② 仍须 S-2 双签（计划 + 基线 + signed floor = baseline_free − 3072 MiB）后才开 G-L1；③ 失败即停、不重试、不放宽；④ 开窗前 5 分钟内三点基线 + monitor 60/60 + 零编译仍全适用。」——**无「连续」** ✅。
- 四条件逐字保留、`not_extended` 未动 ✅。
- 新增 `wording_note`：「team must not add any unapproved condition; this text is the approved wording (the earlier draft wrongly added '连续', removed in the 2026-11-08 fix round)」✅。
- README §4 引用同步为批准原文，并写「以上为用户批准原文；团队不得追加任何未获批准的条件（签名后 fix 轮已删除此前误加的「连续」，其余文字逐字保留）」✅。
- 全文再扫「连续」：仅剩三处**留史/说明**性出现（README 版本链条目、README §4 括号说明、receipt `wording_note`），预批口径本体无该词；spec 内「连续动画」为无关上下文 ✅。

## 4. O3 闭环（signatures_required 已签状态）

| 项 | 实测 | 判定 |
| --- | --- | --- |
| S-1 | “SIGNED 2026-11-08 and effective; spec frozen at sha256 bb064fe0… / 39,379 B” | ✅ |
| S-2 | “NOT SIGNED (dual sign: plan + baseline + signed floor); authoring exception pre-approved 2026-11-08 only, scope in s2_preapproval; six preconditions still apply” | ✅ |
| S-3 | “NOT SIGNED” | ✅ |
| pilot token | 仍 “Chief-level (G-L0), not a user signature” | ✅ |

与 `signature_status` 口径一致，S-2 预批范围/前置未被顺带放宽。

## 5. 门禁与结论

- 四门禁独立复跑：fmt / clippy / clippy-cuda 通过，**unit 1359 过 / 0 败**（记录见本卡 `.normify/runs/`）✅。
- fix 卡 `2026-11-08-...-signature` 状态 verified、addenda 1 条（记录 O1/O2/O3 修正理由）✅。
- **结论：PASS**，无阻断项、无新观察；O4（spec 头注「待签名」）按冻结与既有说明保持不动。

## 6. 非宣称

- 本复核只判 fix 轮是否闭环我的 O1/O2/O3 且未引入新偏差；不重判判据内容，不构成 G-L0/G-L1/G-L2 启动或通过。
- 未改签名记录、spec 字节、被复核证据与任何卡状态。
