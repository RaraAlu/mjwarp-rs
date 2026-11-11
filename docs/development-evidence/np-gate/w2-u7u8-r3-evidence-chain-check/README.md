# QA 复核 U7U8 配套片链（r2 FAIL 记录 + r3 重冻/就绪 + abandoned→check 卡 → `normify fix` 追加 r3 证据）

- 复核对象：配套链三卡
  1. `2026-11-12-w2-u7u8-r3-evidence`（登记 `303c5146`，**abandoned** `ee02b40d`）；
  2. 其证据提交 **`63b95170`**（r2-fail 目录 13 文件 + r3 就绪/重冻 5 文件）；
  3. `2026-11-12-w2-u7u8-r3-evidence-check`（登记 `adf52890` → verified `2ccc182b` → `normify fix` 追加 r3 窗证据 `1bd85287`）。
- 性质：只读复核（0 窗）；零 GPU。结论：**复核通过（0 阻断，0 严重、0 一般、0 低 + 2 观察）**。
- 主链结论另卡：`2026-11-12-w2-u7u8-full-condition-assert-check`（PASS，1 低 + 5 观察）。

## 1. abandoned 卡（`...-r3-evidence`）

- **状态/理由 ✅**：卡 `status=abandoned`、`closed_at=2026-11-11T19:11:11.439Z`；note 记「放弃：该卡 unchanged tests/ 检查与主卡 tests 修正冲突（运行中不可改 checks/waive）；证据已由本卡提交 `63b95170` 落地，改由新验证卡复核」——
  与 `ee02b40d` 提交说明一致（提交仅改卡 JSON）。
- **放弃前已落证据 ✅**：该卡 6 条 json 检查（r2-fail `/verdict=FAILED`、`/attempt=1`、`/failures_total=21`；
  r3 refreeze `/freeze_dir`、`/counts/old_freeze_match=9`、`/source_tree_digest/combined=266bbf94…`）我逐条解析实际文件 **6/6 满足**。
- **无假 PASS ✅**：该卡 acceptance 明写「不宣称 U7/U8 通过或任何窗口结论」，其检查只断言 **FAILED** 收据与重冻事实，
  无任何通过类判定。

## 2. 证据提交 `63b95170`（append-only 基线）

- **r2 FAIL 记录 ✅**：13 文件（README/receipt/failure-record/u7-u8-failures/facts/results/sha-guards/window-samples/manifest + 4 logs）；
  receipt `verdict=FAILED`、`attempt=1`、`failures_total=21`（U7 4 + U8 17）、`exit_code_capture_bug`、`summary_count_bug` 均在位；
  manifest **12/12** 重算 0 错。
- **r3 重冻/就绪 ✅**：`refreeze-10-exe-r3.json`/`.tsv`、`guards-r3.tsv`（18 条）、`window-readiness.md`、
  `logs/runner-selftest.log`；branch 重建 `ac823b58…`/41,782,272 B、其余 9 个 sha 与 r2 清单 9/9 全等、src digest 不变。
- **append-only 无改写 ✅**：r2-fail 目录 13 文件在 `63b95170` 与 HEAD 的 blob **逐个相同**；
  r3 目录在该提交已有的 5 文件在 HEAD **逐字节未变**（后续只新增文件，无改写）。

## 3. 验证卡（`...-r3-evidence-check`）与 `normify fix` 追加

- **登记/验证 ✅**：`adf52890` 仅加卡 JSON（in_progress）；`2ccc182b` 提交卡 verified + 4 个 gate run（fmt/clippy/clippy-cuda/unit）。
  7 条 json 检查我逐条解析：r2-fail `/verdict=FAILED`、`/attempt=1`、`/failures_total=21`；
  r3 refreeze `/freeze_dir`、`/counts/old_freeze_match=9`、`/source_tree_digest/combined`、`/supersession/new_branch_sha256=ac823b58…` —— **7/7 满足**。
- **fix 追加可审计 ✅**：`1bd85287` 同时改该卡（新增 `addenda` 记录 `at=19:20:26`、reason「追加 r3 窗 PASS 证据（attempt 2）」、
  `hash=sha256:440572af…`）与其 4 个 run，并新增 r3 窗证据 13 文件（README/facts/receipt/results/sha-guards/u7-u8-full/manifest + 6 logs 等）；
  卡维持 verified。
- **无历史改写/无假 PASS ✅**：本次 fix 只**新增** r3 窗文件并更新 manifest/卡记录；r2-fail 与本卡已提交文件字节未变；
  该卡 checks 仍只断言 FAILED 收据与重冻字段（不含 PASS 判定），其 acceptance 明写「不宣称窗口结论」；
  r3 窗 PASS 由**主卡**检查（经 r2 目录镜像）与 r3 权威 receipt 联合支撑，不由本卡断言。

## 4. 观察

- **O1 PASS 证据「托管卡」错位（治理可读性）**：r3 窗 PASS 证据实际提交在 `...-r3-evidence-check`（机器复核卡）名下，
  而主卡范围不含 `...-window-r3/`、其检查改指 `...-window-r2/` 镜像。链条可追溯（镜像 `full_evidence_dir`→r3、
  本卡 addenda、r3 权威 receipt），但审计需两跳。建议：镜像 receipt 增 `mirrors_sha256`（r3 receipt 的 sha256）+
  `verdict_scope`；未来窗口证据目录用轮次无关命名，避免「evidence 托管在 check 卡」。
- **O2 r2 attempt 1 的「放行 monitor 60/60」CSV 未入证据**：`...-r2-fail/` 只保留**开窗前失败**的那轮 monitor
  （142 行、util_max 94、streak_last 37，作为留档）；让 attempt 1 获 GO 的第二次 60/60 monitor 原始 CSV 未见入库
  （应在群里留痕）。本次 attempt 1 已 FAILED、未重试，影响为零，仅影响事后完整复算。

## 5. 非声明

本复核只对配套链的证据完整性/append-only/卡状态与 fix 可审计性负责；**不宣称** U7/U8 全条件通过
（由主卡窗口径证据支撑）、U8 已验收、mesh 容量缺口已清、与上游等价（over-32）、G10@P5 整族通过、NP01/NP02 门禁通过；
**不作位级等价声明、不提供位级等价保证**；零 GPU、零窗；不改判据/参考/容差/夹具/源码。

## 6. 证据文件

`README.md`、`receipt.json`、`logs/independent-check.txt`、`manifest.txt`。
