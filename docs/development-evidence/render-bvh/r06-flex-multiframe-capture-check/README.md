# R06 多帧采集窗（G-1）负结果 QA 只读复核

任务：`2026-11-05-render-r06-flex-multiframe-capture-check`（owner QA；**QA 自登记**——Arch 已转只读、窗口卡未同批登记 QA 卡，Chief 指派只读复核）。复核对象：窗口执行卡 `2026-11-05-render-r06-flex-multiframe-capture`（**verified**）→ 提交 `484852ca` / `dbe8d2b7` / `27474ac5`；证据 `docs/development-evidence/render-bvh/r06-flex-multiframe-capture/`。日期：2026-11-05（12:00–14:00 空闲段）。
**边界：只读复核 + 只写本目录；零 GPU、不执行采集、不改窗口证据/脚本/spec/fixtures。**

## 1. 结论

**PASS-with-observations（0 阻断）。** 负结果判定成立、处置与窗口合规性均与证据一致：`001→002` depth **244** / seg **183**（<384）→ overall **non-discriminating** → 按 spec 停手、exit 3、**未落库**；两次采集逐位一致；基线三点 ≥ floor 11,688、monitor **60/60**、S-2 未启用；冻结 spec 与现有单帧参考均未变。
4 条低观察（O1–O4，见 §3）：证据 4 个原始文件编码/BOM 与卡面「UTF-8/LF/无 BOM」不符；窗口卡登记/提交时序晚于基线且与证据同笔提交；`capture.log` 无命令行/exit code；`frame-000 vs base` 数组级对照按 spec 停手未执行（如实记录）。均不影响负结果、停手与不落库结论。

## 2. 逐项核对

### 2.1 负结果判定成立（PASS）
- `capture.log`（cp936 解码）内脚本摘要 JSON：`frames=3`、`frame_dz=0.1`、`two_captures_bitwise_identical=true`、`legacy_reference_untouched=true`；
  `delta_report` = 000→001 **697/616 pass-margin**、001→002 **244/183 non-discriminating**、`delta_verdict=non-discriminating`，并打印「不得落库」警告。
- `capture-diagnostic.json.delta_selfcheck` 与 receipt 的 `capture.delta_report` **逐项一致**；两对值均 <384 的第二段成立。
- **exit 3**：日志为脚本 stdout（无 exit code），但 receipt 记录 `exit_code=3`；且按已复核的 O3 代码 `delta_exit_code(non-discriminating)=3`（复核提交 `dccb00ce`）可判定——与停手一致。
- **无重试/无放宽**：日志仅一次采集摘要（无第二次运行）；`--frames 3 --frame-dz 0.1` 冻值（dry-run 与 receipt 一致）；窗口提交未改 `scripts/`、spec、判据/阈值/步进。

### 2.2 未落库（PASS）
- `fixtures/render-bvh/scene-flex/multiframe/` **不存在**；`git ls-files fixtures` 无 multiframe 产物；窗口三笔提交**不含任何 `.npz`**；`git status` 除本复核自身文件外 clean；`target/r06-multiframe-capture/` 无残留帧。
- 证据目录只保留诊断日志与 `capture.json` / `cam-poses.json` 副本；12 个帧文件的 sha256 仅作诊断记录（receipt `not_landed.retained_file_sha256_for_diagnosis`）；**未作为参考/验收材料**。
- 诊断佐证：`frame-000` 四个文件 sha256 与现有 `reference/base/` 参考**完全相同**（无偏移帧与单帧参考同源）。

### 2.3 窗口合规（PASS）
- 基线 3 点（12:00:26/28/31）：used 1,730 MiB、free **14,332 MiB**、util 2–3%；floor = max(min_free−3,072, 11,688) = **11,688**，逐点 free ≥ floor、util<40 ✅。
- monitor：60 行、**60/60 clean**、util 2–4%（max 4）、free_min **14,329** ≥ floor、compute_filtered/build 全 0；**S-2 例外未启用**（常规 util<40 达标）。
- dry-run：exit 0，spec sha `1d9fa5ce…`/10,506 B 校验、锚点 `[1,2,3]`、base z `[0,0,0]`、`repeat=2`、三帧布局，未写产物。
- 采集：12:06:38–12:06:44（约 6 s）、两次采集逐位一致、`legacy_reference_untouched=true`；冻结 spec sha/bytes 未变；`fixtures/`（含 base 参考）、`scripts/`、`src/`、`tests/` 在窗口提交范围均零 diff；无编译、guards 未重冻；窗后 GPU `1,714 used / 14,348 free MiB / 2%` 已释放。

### 2.4 锁时序证据（PASS，事实记录）
- README §10：11:59:25（窗口前）误 `acquire`、**无仓库内容写**（仅锁文件）；12:00:03 后 `release`→`acquire`（12:00:07）→ `status` 后开始写；窗口内均在锁下、12:06:44 结束释放。卡 addenda 与之逐字一致。按 Chief 口径仅作事实记录。

### 2.5 证据完整性与非声明（PASS）
- 窗口证据 manifest **9/9** sha256/bytes 自核通过；`README/receipt/manifest/monitor-csv/两个 diagnostic json` 均 UTF-8/LF/无 BOM。
- 非声明齐全：不声明多帧已覆盖/多帧验收结论/动画一致性/上游等价/对外发布依据；卡 `forbid_claim` 三词（多帧验收通过、上游等价、超窗写）**0 命中**。

## 3. 观察清单（按严重度，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低（证据卫生） | 证据目录 4 个原始文件不符合卡面「UTF-8/LF/无 BOM」：`capture.log`、`dry-run.log` 为 PowerShell ANSI(cp936)+CRLF（grep/`read` 工具会失败）；`baseline-three-points.txt`、`post-window-gpu.txt` 带 UTF-8 BOM+CRLF | 走一次 `normify fix` 把这些转 UTF-8/LF（保持正文不变），或在卡面注明「原始控制台输出按原字节保留」 |
| O2 | 低（记账时序） | 窗口卡 `created_at` 12:01 晚于基线 12:00:26 / monitor 12:00:44；卡与全部窗口证据同在 **12:07:59** 的 `484852ca`（标题「登记 + start」）提交里落库，晚于采集结束 12:06:44 | 窗口类流程建议「材料/卡先落库再执行」；本次不影响数值/停手/不落库结论，§10 已如实记录锁时序 |
| O3 | 低（日志完备性） | `capture.log` 仅含脚本 stdout（警告 + 摘要 JSON），无命令行与退出码；命令与 exit=3 由 receipt/README 记录，exit3 可由代码路径判定 | 采集窗日志建议附 wrapper 的 command/exit 行，减少对回执的依赖 |
| O4 | 低（卡面 literal） | 卡验收第 5 条要求 `frame-000 vs base` **数组级**对照；本窗在自检处按 spec §8.5 停手、未执行该对照（receipt/README 如实记为 NOT RUN） | 与 spec 停手顺序一致、不算违规；建议在卡面把「仅在 pass-margin 后执行」写清，避免后续误读为缺项 |

## 4. 复现与证据

- `scripts/verify_capture_window.py`：提交范围/边界、冻结 spec、负结果数值（cp936 解码 `capture.log`）、dry-run、基线/floor、monitor、receipt vs diagnostic、frame-000 vs base（文件 sha）、未落库、manifest/编码/非声明、§10 与卡状态。
- `logs/verify.log`：`RESULT: PASS`（0 FAIL；4 条 OBS 对应 O1–O4）。
- `logs/qa-check.json`：机器可读结果（编码偏差清单、监视器统计、一致性与时序事实）。
- `manifest.txt` 5 条，LF、无 BOM，路径为仓相对。
