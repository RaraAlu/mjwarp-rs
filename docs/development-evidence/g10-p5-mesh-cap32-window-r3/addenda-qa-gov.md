# W2/U7U8 r3 证据 addenda（2026-11-12，QA 复核后治理补注；append-only）

- 任务：`2026-11-12-w2-u7u8-full-condition-assert`；主链 QA 复核 PASS（0 阻断/0 严重/0 一般 + 1 低 + 5 观察）、配套链 PASS（0 低 + 2 观察）。
- 本文件只追加说明：不改本目录既有任何字节（README/receipt/logs/facts/guards 等），manifest 随本文件重算。

## 1. 主卡 acceptance 3 处陈旧（QA 低项；acceptance 运行期冻结，故在此补注）

1. **item 7 证据目录**：acceptance 写“新证据目录 `docs/development-evidence/g10-p5-mesh-cap32-window-r2/`”，
   实际窗后 PASS 证据在 **`docs/development-evidence/g10-p5-mesh-cap32-window-r3/`**（attempt 2，r3 冻结轮）；
   `...-window-r2/` 保留为窗口前材料目录 + attempt 2 收据镜像（见 §4）。
2. **item 9 CPU 数**：acceptance 写“CPU 测试 3 passed / 2 ignored”，终态为 **4 passed / 2 ignored**
   （新增 `pair_normalization_matches_upstream_reference` 后；实测于修正片与本次复核）。
3. **item 11 receipt 指向**：主卡检查路径指向 `...-window-r2/receipt.json`，该文件为**收据镜像**
   （`kind=window-receipt-mirror`），权威窗口 receipt 是本目录 `receipt.json`（attempt 2 PASS）。

## 2. `window-readiness.md` §6 四前提勾表补填（实测值与来源）

| # | 前提 | 实测值 | 来源 |
| --- | --- | --- | --- |
| 1 | fresh freeze | `git diff 2ee0540d..HEAD -- src/`=0、`git diff HEAD -- src/`=0；29 `.cu` digest `266bbf94…`；r3 branch exe `ac823b58…`/41,782,272 B 对盘一致；9 旧 exe sha 与 r2 清单 9/9 全等；旧 r1 `9fe3192b…`、r2 `6d2b39fc…` 未动；guards 18/18；runner selftest PASS | 本目录 `refreeze-10-exe-r3.json`/`guards-r3.tsv`/`logs/runner-selftest.log` + Chief GO 报文 |
| 2 | monitor 60/60 | 03:12:50–03:17:51，60 样本全 clean，`util_max=4`、`free_min=13,440`、compute_filtered 0、build 0 | `logs/monitor-pre-window-r3.csv`（官方脚本，Chief 开窗前执行并群留痕） |
| 3 | 三点基线 | 03:18：util 2/4/2 %、used 2,614/2,614/2,614 MiB、free 13,448/13,448/13,448 MiB、25 °C、driver 596.36 | `logs/three-point-baseline-r3.txt`（Chief 生成 raw） |
| 4 | floor 取严 | 硬门 **10,240**；公式 `baseline_free−3072 = 13,448−3,072 = 10,376`；两者取更严者（max）= **10,376**；窗内观测 pre/post free **13,440 MiB > 10,376 > 10,240** → 通过 | `facts.txt`（FLOOR/POST 行）+ `three-point-baseline-r3.txt` |
| 附 | 零 Rust 编译 | precheck 无 cargo/rustc/cl/nvcc；窗后 POST PASS；无残留 compute | `facts.txt` |
| 附 | 单窗互斥 | 全队同时间仅本窗；窗内无仓写、无群消息 | 执行记录 |

## 3. 两处口径补注（QA 观察）

1. **窗内采样统计排除首行**：`window-samples.csv` 共 **138 个样本**（不含表头）；相邻样本形成 **137 个间隔**
   （首行不构成间隔）。间隔统计口径 = **137/137**（min 290.6 ms / max 361.3 ms，目标 250 ms；
   util_max=4、free_min=11,793 取自 138 样本）。receipt `window_sampling` 的 `samples=138` 与
   `interval_min/max` 并列使用时不冲突，特此注明统计基数。
2. **floor 公式与硬门**：receipt `floor_effective_mib=10240` 记录的是**硬门**；按“取更严者”的严格口径，
   本窗生效门槛应为 `max(10,240, formula 10,376) = 10,376`。因观测 pre/post free 13,440 同时高于两者，
   **窗口有效性不受影响**；此处补正口径，不改 receipt 历史字节。

## 4. r2 目录收据镜像的加固（关联主卡检查冻结问题）

- `docs/development-evidence/g10-p5-mesh-cap32-window-r2/receipt.json`（`kind=window-receipt-mirror`）追加：
  `mirrors_sha256`（指向本目录权威 receipt 的 sha256）+ `verdict_scope: "attempt2-window-only"`；
  逐字段镜像 attempt 2 PASS（11 字段与权威 receipt 相等），并保留 attempt 1 FAILED 指针（`docs/development-evidence/g10-p5-mesh-cap32-window-r2-fail/`）。
- 该目录新增 `manifest.txt`（逐文件 sha256+bytes 自核）。镜像**不是对 r2 exe 的 PASS 声明**。

## 5. 过程观察：未来窗口使用轮次无关目录名（列入后续计划）

- 本片因 r2→r3 重冻导致“注册时证据目录名（`...-window-r2/`）与实际 PASS 轮次（r3）不一致”，
  进而触发主卡检查路径与 acceptance 陈旧、需镜像收据 + addenda 补注的额外治理成本。
- 建议：**后续窗口的证据目录用轮次无关命名**（如 `...-window-u7u8/` 或 `...-window-final/`），
  轮次（attempt/冻结轮）写进 receipt 字段与 README，而不是目录名；登记卡时按任务语义而非轮次取目录名。
- 该建议写入本 addenda 作为过程观察，供 W2/A2 后续窗口计划采用。

## 6. 非声明

- 本 addenda 不改变 r3 窗结论（attempt 2 PASS：U7/U8 comparator failures=0）与限制/边界；
  不宣称 mesh 容量缺口已清、与上游等价（over-32）、G10@P5 整族通过、NP01/NP02 门禁通过；
  不作位级等价声明、不提供位级等价保证。
- 不改判据/容差/参考/comparator；不改 r2-fail 与 A2-3 历史字节。

## 7. floor 口径消歧（2026-11-12，Chief 复核后 append-only 补正；QA 独立核）

- 本节更正 §3.2 中「公式 10,376、严格生效 max=10,376」的表述（把**候选值**误当夹取后 floor）。
  以本节四值为准，**§3.2 原文保留（历史字节不改），冲突时以本节为准**：
  1. **候选**：`candidate = baseline_free − 3072 = 13,448 − 3,072 = 10,376`；
  2. **计划公式夹取**：`floor = min(10,240, candidate) = min(10,240, 10,376) = 10,240`；
  3. **硬门**：`hard_gate = 10,240`；
  4. **生效门槛**：`effective = max(hard_gate, floor) = max(10,240, 10,240) = 10,240`。
- 结论：按计划公式的生效门槛为 **10,240 MiB**；窗内观测 pre/post free **13,440 MiB > 10,240**
  （同时 > 候选 10,376），**窗口有效性不变**；receipt `floor_effective_mib=10240` 与该生效值一致。
- **固定表述（后续窗口 readiness 一律使用此句）**：
  「floor：candidate = baseline_free − 3072；candidate=10,376；min(10,240, candidate)=**10,240**（夹取后 floor）；
   hard_gate=10,240；effective=max(hard_gate, floor)=max(10,240,10,240)=**10,240**。」
- 说明：计划原文 `floor = min(10240, baseline_free − 3072)` 本身无误；消歧点仅在“公式候选值”与“夹取后 floor /
  生效门槛”的分层。此前 §3.2 的文字为笔误，不改任何数值结论或历史字节。

