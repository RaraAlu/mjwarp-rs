# QA 最小复核：S-2 签名留证卡（`2f361d91`；零 GPU）

- 任务：`2026-11-08-render-perworld-l-v2-capture-window-s2-signature-check`（owner QA、p82、P8-渲染与BVH）
- 复核对象：S-2 签名留证卡 `2026-11-08-render-perworld-l-v2-capture-window-s2-signature`，提交 **`2f361d91`**（parent `beeca456`，即我 r2 复核通过的材料版本）
- 依据：已签 spec v3 `bb064fe0…`/39,379 B；S-2 材料 r2（`0b2b4cc4`，QA `beeca456`）
- 基线：HEAD `2f361d91`、树 clean、锁 FREE；**只读 + 只写本卡证据；零 GPU**

## 0. 结论

**PASS（0 阻断）→ 可发 G-L1 token。**

- **S-2 receipt 签名块齐全**：`status=signed_pending_window`、`signed_object=true`（顶层与 boundary 两处）；`signature` 含 signer=user / signed_at=2026-11-08 / timezone / method=聊天卡双签 / scope=S-2a 计划 + S-2b（13,302 / 10,230 / `max(signed,现场)`）+ authoring 例外（仅本采集窗）+ 失败条款；`material_sha256=c8bb71ef…`/18,299 B；`does_not_open_window` 五项条件齐全；note 解释 README sha 差异。
- **两处 `signature_status` 一致**：S-1 signed 2026-11-08 / **S-2 signed 2026-11-08** / S-3 not yet signed；S-1 `signatures_required` 的 S-2 行 = SIGNED 且带留证卡锚点与「不构成开窗」。
- **S-1 未被顺手改动**：receipt diff 恰 **2 个逻辑行**（`signature_status.S-2`、`signatures_required` S-2 行），判据/事实/其它块零改动；S-1 manifest 仅 receipt sha 更新、spec.md 行 sha 不变。
- **spec 字节三处逐位相同**：工作树 / `HEAD` blob / `HEAD^` blob 均 `bb064fe0…`/39,379 B/344 行；本提交不含 `spec.md`。
- **README/manifest/门禁/边界**：README 状态行/§5/§7/§8 已记录「已双签、但不构成开窗」（签名对象 `c8bb71ef…`/18,299 B，= `beeca456` 材料版本）；两 manifest **12/12**、**3/3**；本提交恰 10 文件、`src/tests/fixtures/scripts` 零 diff；四门禁通过、unit **1387/0/402**。

## 1. 身份（独立重算，与声称逐位一致）

| 文件 | sha256 | bytes |
| --- | --- | --- |
| S-2 `README.md` | `2ca9f1dd8ff7dca554d3532a80fde13273bdb0b5a7357e7b73430f90196bd84c` | 19,502 |
| S-2 `receipt.json` | `1bb459c34bd0183e874b63afb4d836298814240d63b1071c12eb2f19c79c062f` | 13,669 |
| S-2 `manifest.txt` | `c648316c78ea3713e40d1de623a130db8489a214aea9028ad28fd23f8b096fdd` | 1,342 |
| S-1 `receipt.json` | `8cf4d0f9ae266cdc9812c9e24c9a07edab5993d54f3e91b2d83638e2d678a6a7` | 22,822 |
| S-1 `manifest.txt` | `0adc5be5836fc32f9ea9a99ca8a7c44137203e3f04b9917bbeabf61bcf526a7e` | 456 |
| `spec.md` | `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` | 39,379（344 行） |

## 2. S-2 receipt 签名块（逐项）

| 项 | 值 |
| --- | --- |
| `status` | `signed_pending_window` |
| `signed_object` | `true`（顶层 + `boundary.signed_object` 两处） |
| signer / signed_at / method | `user` / `2026-11-08`（Asia/Shanghai）/ `chat card selection (dual sign S-2: plan + floor 10,230)` |
| scope | S-2a G-L1 计划 + S-2b（`baseline_free 13,302`、signed floor `10,230`、`floor_effective = max(signed floor, on-the-spot floor)`）+ authoring 例外（2026-11-08 预批、**capture window only**）+ 失败条款 |
| material | README `c8bb71ef…` / 18,299 B（**= `beeca456` 的 README，即我 r2 复核版本**；可追溯） |
| spec | `bb064fe0…` / 39,379 B / `spec_unchanged_by_this_card=true` |
| `does_not_open_window` | QA PASS ✓ / Chief G-L1 token ✓ / 三点基线 `max(signed 10,230, on-the-spot)`（higher=stricter）✓ / monitor 60/60 ✓ / zero compile ✓ |
| note | 明确「`material_sha256` 是用户签的 README 版本；留证卡随后追加版本记录，故 manifest 的 README sha 与之不同（预期，签名绑定上面的 sha）」 |

**旧措辞已改且不矛盾**：`material_fixed_r2_pending_qa_recheck_and_user_dual_sign` → `signed_pending_window`；`signed_object` false→true（两处）；`awaiting.user_s2_dual_sign` pending→SIGNED；`non_claim[0]` 与 README §7 改为「已签但仍不构成开窗，仍需 QA PASS + Chief token + 现场条件」；`awaiting.chief_g_l1_token` 仍 pending —— 全文一致。

## 3. `signature_status` 跨文档一致

- S-2 receipt 与 S-1 receipt 均：`S-1` signed 2026-11-08（spec v3 sha/bytes frozen）/ `S-2` **signed 2026-11-08（plan + baseline 13,302 / signed floor 10,230 + authoring exception + failure clauses）** / `S-3` not yet signed。
- S-1 `signatures_required` 的 S-2 行 = `SIGNED 2026-11-08 (recorded in perworld-l-v2-capture-window-s2-signature)`，并写明「signing does not open G-L1」。
- S-1 receipt diff **仅 4 个 +/− 行 = 2 个逻辑行**（上述两处），其余判据/事实/`criteria`/`go_list`/`s2_preapproval` 等块零改动（逐行核对）。

## 4. spec 字节未动

- 工作树 = `HEAD` blob = `HEAD^` blob：`bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / 39,379 B / 344 行。
- `2f361d91 --name-only` 中不含 `docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md`；该提交只含证据/卡/runs。

## 5. README / manifest / 边界 / 门禁

- S-2 README：状态行改为「S-2 已由用户双签（2026-11-08），待 Chief token + 现场条件——签名不构成开窗」；§5 标题标注已签；§7 更新「已签但仍需 QA PASS + token + 现场条件」；§8 新增「用户签名 S-2」条目（签名对象 `c8bb71ef…`/18,299 B、留证卡锚点、sha 变化属预期、非声明）。
- 两 manifest 独立重算：S-2 **12/12**、S-1 **3/3**；两目录编码卫生（UTF-8/LF、无 BOM、裸 CR·TAB 0）。
- `2f361d91`（parent `beeca456`）恰 **10 文件**：卡 1 + runs 4 + S-2 README/manifest/receipt 3 + S-1 receipt/manifest 2；**新增 5**（卡+runs）、**修改 5**；`src/`、`tests/`、`fixtures/`、`scripts/` **零 diff**。
- 四门禁：fmt/clippy/clippy-cuda exit 0；unit **1387 passed / 0 failed / 402 ignored**。

## 6. 观察项（均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 信息 | `signature.material_sha256`（`c8bb71ef…`）与当前 manifest 的 README sha（`2ca9f1dd…`）不同——属设计（签名绑定签名时版本，留证卡随后追加版本记录），`signature.note` 与 README §8 均已解释，且 `c8bb71ef` 可追溯到 `beeca456` blob | 无需改；G-L1 材料引用时以 `signature.material_sha256` 为签名对象 |
| O2 | 信息 | `does_not_open_window` 位于 `signature` 对象内部（非顶层独立键），五项条件齐全 | 无需改 |
| O3 | 信息 | `awaiting.qa_prereview` 已写明「signature recorded, minimal QA re-check requested」——本卡即该复核（PASS） | 无需改 |

## 7. 非声明

本卡只核对签名留证记录与跨文档一致性：**不宣称** G-L1 已可开（仍需 QA PASS + Chief token + 现场三点基线取更严 + monitor 60/60 + 零编译）、不宣称采集有效/判别力恢复/任何渲染数值结论；未改被复核内容、spec、S-1/S-2 签名记录。
