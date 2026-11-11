# QA 复核 W2/U7U8 治理片（`64fc0e41` 主卡补注+镜像加固 → `bde94ca8` addenda-qa-gov → `2c027d4c` floor 消歧）

- 复核对象：治理片三提交 **`64fc0e41`**（主卡 QA 低项 append 补注 + r2 镜像收据加固）、**`bde94ca8`**（r3 `addenda-qa-gov.md` + manifest 17→18）、**`2c027d4c`**（floor 口径消歧 append + manifest 重算）；HEAD `2c027d4c`。
- 性质：只读复核（0 窗、零 GPU）；**当前 HEAD 已从你消息里的 `bde94ca8` 前进到 `2c027d4c`**，我按最终 HEAD 核（§7 在我复核期间由 Backend 追加）。
- 结论：**复核通过（0 阻断，0 严重、0 一般、0 低 + 4 观察）**——我此前 1 低 + 5 观察的 remediation 已全部闭环。
- **floor 口径裁断（本文第 4 节，单独写清）**：候选 10,376 / `min(10,240, candidate)` 夹取后 floor **10,240** / 硬门 **10,240** / 生效门槛按计划 `max(硬门, floor)` = **10,240 MiB**；§7 表述与计划原文一致、正确。

## 1. 主卡 1 低闭环（`64fc0e41`）

- **fix 记录 ✅**：主卡 `.normify/...full-condition-assert.json` 仅新增 `addenda`（`at=19:31:21`、reason 明确三处：item7 证据目录实为 `...-window-r3/`、item9 CPU 终态 4/2、item11 指向 r2 镜像）与 `updated_at`；**acceptance/checks/note/title 字节未改**、卡仍 `verified`、`closed_at` 不变。
- **§1 三处补注准确 ✅**：我逐条对事实核——
  - item 7：窗后 PASS 证据确在 `docs/development-evidence/g10-p5-mesh-cap32-window-r3/`（17 条 manifest + 18 文件）；`...-window-r2/` 确为窗口前材料 + 收据镜像（7 文件）。
  - item 9：终态 CPU 测试确为 **4 passed / 2 ignored**（我本轮前已本地重跑，含 `pair_normalization_matches_upstream_reference ... ok`）。
  - item 11：主卡检查路径确指向 `...-window-r2/receipt.json`，其 `kind=window-receipt-mirror`，权威 receipt 在 r3 目录。
- **无新的过度表述 ✅**：§1 只陈述布局/计数/指向，未把「追加补注」写成「acceptance 已改」，也未削弱非声明。

## 2. 镜像加固（`64fc0e41`）

- **`mirrors_sha256` 对盘 ✅**：r3 权威 `receipt.json` 当前 sha256 = **`365f15e9…01c952`**，镜像 `mirrors_sha256` 与之**逐字相等**（且本轮 receipt 未被改，故 pin 有效）；`mirrors_path` 指向 r3 receipt。
- **`verdict_scope` ✅**：`attempt2-window-only`；镜像仍 `kind=window-receipt-mirror`、`attempt=2`、`window="r3-minimal (attempt 2)"`、
  顶部 `attempt1.verdict=FAILED` + `attempt1_fail_record` 指针 + `full_evidence_dir`→r3 保持醒目；`mirror_note` 追加加固说明，
  仍明写「**不是对 r2 exe 的独立 PASS 声明**」。
- **receipt diff 仅追加 ✅**：非声明 3 条、attempt1 块、11 个镜像字段均未改；只新增 `mirrors_sha256`/`mirrors_path`/`verdict_scope` 与 note 后缀。
- **r2 manifest 7/7 ✅**：新增 `manifest.txt` 逐文件重算 **7/7 全对**、不列自身；文件集与目录实际一致（readiness/refreeze/guards/nvrtc/README/receipt）。

## 3. r3 addenda §2 四前提勾表（`bde94ca8`）

- **fresh freeze ✅**：`git diff 2ee0540d..HEAD -- src/` 为空；`src` digest **`266bbf94…`**（我按口径复算）；r3 branch exe
  **`ac823b58…`/41,782,272 B** 对盘；9 个未重建 exe 与 r2 清单 **9/9 全等**；旧 r1 branch `9fe3192b…`（`target/a2-freeze/`）、
  r2 branch `6d2b39fc…`（`target/a2-freeze-r2/`）**均未动**；guards **18/18**；runner selftest `RESULT=PASS`。
- **monitor 60/60 ✅**：CSV 实为 60 行、03:12:50–03:17:51、`util_max=4`、`free_min=13,440`、compute_filtered/build 全 0（与 §2 行一致）。
- **三点基线 ✅**：`three-point-baseline-r3.txt` P1–P3 = util 2/4/2 %、used 2,614 MiB×3、free 13,448 MiB×3、25°C、596.36（03:17:59–03:18:02）。
- **floor 硬门 ✅**：10,240 MiB；生效值见 §4。

## 4. floor 口径裁断（重点）

**计划原文**（`a2-3-window-plan.md` §2.6 / `window-readiness.md` §7）：硬门 `free ≥ 10,240 MiB`；`floor = min(10,240, baseline_free − 3,072)`；取两者更严者。

**按计划公式逐步算（r3：baseline_free = 13,448）**：
1. **候选**：`candidate = baseline_free − 3,072 = 13,448 − 3,072 = 10,376 MiB`；
2. **计划公式夹取**：`floor = min(10,240, candidate) = min(10,240, 10,376) = 10,240 MiB`；
3. **硬门**：`hard_gate = 10,240 MiB`；
4. **生效门槛**：`effective = max(hard_gate, floor) = max(10,240, 10,240) = 10,240 MiB`。

**准确口径一句话**：「候选 10,376 MiB 只是未夹取值；按计划 `min(10,240, candidate)` 夹取后 floor = 10,240 MiB；
硬门 10,240 MiB；生效门槛 = `max(硬门, floor)` = **10,240 MiB**。窗内 pre/post free 13,440 MiB > 10,240（也 > 候选 10,376），窗口有效性不受影响。」
—— 这与 A2-3 窗同式（候选 9,909 → floor 9,909 → 生效 `max(10,240, 9,909)=10,240`，与其 receipt `floor_effective_mib=10240` 一致）。

**§7 核对结论 ✅**：`2c027d4c` 追加的 §7 四值（候选 10,376 / 夹取 10,240 / 硬门 10,240 / 生效 10,240）**与我上述裁断逐字一致**，
与计划原文一致，与 r3 receipt `floor_effective_mib=10240` 一致；§7 明确「§3.2 原文保留、冲突以 §7 为准」，且
「固定表述」已同步进 `target/.../u7-u8-full-conditions-plan.md` **附录 B**（我核原文在）。
§2 表现存第 4 行仍是被 §7 取代的旧表述（append-only 保留历史），我按「冲突以 §7 为准」记录，不视为现行口径。

## 5. 采样统计口径（§3.1）

- **✅**：`window-samples.csv` **138 个样本**（不含表头）、相邻样本 **137 个间隔**；我重算 137 个间隔全部落在
  **290.6–361.3 ms**（首行不构成间隔）；`util_max=4`、`free_min=11,793` 取自 138 样本。receipt 的 `samples=138` 与
  `interval_min/max` 并列不矛盾（§3.1 已注明统计基数）。

## 6. 边界与 append-only

- **三提交边界 ✅**：`64fc0e41` 7 文件（主卡 JSON + 4 run + r2 manifest + r2 receipt）；`bde94ca8` 7 文件（check 卡 JSON + 4 run + r3 addenda + manifest）；`2c027d4c` 7 文件（check 卡 JSON + 4 run + r3 addenda + manifest）；**无其它路径**。
- **`git diff 4e3abe9c..HEAD` ✅**：只含治理片 + 我的两张复核卡文件；**r2-fail 13 文件零改动**、**r3 历史文件（README/receipt/logs/facts/guards/refreeze/readiness）零改动**（仅 addenda 与 manifest 变）、**A2-3 证据（cap32-window/cap32-identity/cap32）零改动**。
- **append-only ✅**：`addenda-qa-gov.md` 新版（6,613 B）以旧版（5,199 B）**字节前缀**开头；`2c027d4c` 只追加 §7 + manifest 重算；r2 receipt 只追加三字段 + note 后缀。
- **manifest ✅**：r3 **18/18 重算 0 错**、r2 **7/7 重算 0 错**、均不列自身、磁盘文件全覆盖。
- **门禁 ✅**：主卡 fix 后 unit **1410/0/402**（19:31:44）；check 卡 fix 后 unit **1410/0/402**（19:33:55）、fmt/clippy/clippy-cuda 全 passed；两卡 `verified`。
- **非声明未被弱化 ✅**：r2 receipt 3 条、addenda §6、r3 README/receipt 历史非声明均保持；我扫 r2/r2-fail/r3 **40 文件** `policy_scan` **0 命中**；正禁语仅出现在否定句。

## 7. 观察

- **O1 §7 的「QA 独立核」措辞时点**：该句在 `2c027d4c`（03:34）写入时 QA 尚未复核（我现在核完，内容正确）。建议以后写「待 QA 独立核」或「QA 复核见 …」，避免事后读者误判核验时点。
- **O2 同一 addenda 内新旧口径并存**：§2 行 4 与 §3.2 的旧表述按 append-only 保留，与 §7 冲突时以 §7 为准；建议在文件头或 manifest 头注加一行「floor 口径以 §7 为准」，降低审计误读成本。
- **O3 r2 README 文件清单未含后加的 `manifest.txt`**：READY 文件列表（历史字节）只列到 receipt；manifest 为后加固新增，README 未补列（历史不改），建议下轮 addenda 点一句或修 README 清单（非必须）。
- **O4（QA 自我更正）披露**：我在 A2-3 复核里提的 O5（「floor 生效应为 max=10,376」）是**误读**——把未夹取候选当成 floor；治理片 §2/§3.2 一度据此表述，§7 已更正为 10,240。本卡以 §4 的窗口径为准，并在此更正我此前的 O5。

## 8. 非声明

本复核只对治理片的记录准确性/append-only/口径消歧负责；**不宣称** mesh 容量缺口已清、与上游等价（over-32）、G10@P5 整族通过、NP01/NP02 门禁通过、性能结论；**不作位级等价声明、不提供位级等价保证**；零 GPU、零窗；不改判据/参考/容差/夹具/源码。

## 9. 证据文件

`README.md`、`receipt.json`、`logs/independent-check.txt`、`manifest.txt`。
