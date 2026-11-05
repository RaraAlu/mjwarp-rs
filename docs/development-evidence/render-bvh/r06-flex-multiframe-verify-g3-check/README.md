# R06 多帧验证窗 G-3 —— QA 只读复核

- 任务：`2026-11-05-render-r06-flex-multiframe-verify-g3-check`（owner QA；**QA 自登记**，Chief 2026-11-05 夜指派；登记提交 `d90585fa`）
- 复核对象：`2026-11-05-render-r06-flex-multiframe-verify-g3`（**verified**；登记 `515de099`、start `f7d32afd`、结果 `b3a05f2b`）；**复核目标 HEAD `b3a05f2b`**
- 前置链路：实体卡 A（`2a27c2fd`/`241d702f`）、小修卡（`a2d7d4c5`/`a175c7f8`）、QA 复核 B PASS（`8c9e7192`）均 verified
- 边界：**只读复核 + 只写本目录**；零 GPU、不采集、不跑窗口；不改 guards/实体/证据/测试/spec/fixtures
- 方法：不采信执行方结论，独立复算冻结 aggregate、逐文件哈希核对 live/guard/stash、解析基线/monitor CSV/实体日志与时间序、独立重算参考 15 文件；QA 自检 **27/27 PASS**（3 条低观察）

## 1. 结论

**PASS-with-observations（0 阻断）。** G-3 验证窗成立：真冻结 28 条（2 exes + 26 inputs）且 aggregate 双复算一致、live==guard==stash 2/2；窗口前冻结+构建、窗口内**零编译**跑冻结 exe，GPU `#[ignore]` 用例单次通过（exit 0、running 1 test、1 passed、wall 2,764 ms）；该 3 帧集逐帧口径 + 跨帧可见性成立（frame-000 cam2 allowlist 1 个像素 ULP=3/beyond4=0、001/002 严格 bit-first、两跑整块 sha 相同、`refits()==0`、跨帧 cam2-only 1167/1074 与 1262/1126 ≥384）；参考 15 文件校验 exit 0（独立重算一致）。3 条低观察均属证据工件字节口径/措辞/锁留痕，不影响判据与数值。

## 2. 逐项核对

### 2.1 真冻结（PASS）
- 28 条 = **2 exes + 26 inputs**（input=25 / guard_script=1）；`attempt=1`、`status=frozen`、exe 非 pending。
- `aggregate_inputs_sha256` 独立复算 = `b8110cee…`；`aggregate_sha256`（28 条按 path 排序、sha 十六进制串拼接后 sha256）独立复算 = `6a6cca42…` —— 与 guards 文件/`freeze.log`/`freeze-verify.json` 一致。
- guards 文件：raw 工作树 sha256 `64bfecdf…`/11,899 B（CRLF 203 处，Windows autocrlf 工作树口径）；HEAD blob `bf4b9302…`/11,696 B（LF）；证据 `guards-frozen-snapshot.json` = blob（LF 规范化），README §8 已说明——见 O1。
- **live==guard==stash 2/2**（逐文件独立哈希）：bin `a75cf7db…`/85,041,741；test `0f76f9e1…`/118,098,954；三者相同。
- 历史 guards 未动：`8c9e7192..b3a05f2b` 中 `scripts/validate/render-bvh/` 仅 `guards-r06-multiframe.json` 变更；`guards.json`/`guards-r06-s3b-flex.json` 零 diff。

### 2.2 窗口纪律（PASS）
- 基线三点 22:39:40/55, 22:40:10：free **13,726/13,727/13,696 ≥ floor 11,688**（`max(min_free−3072, 11688)` 取严）、util 37/36/38<40、compute_filtered=0、build=0。
- monitor：CSV **72 样本**，最长干净 streak **60 = required**（exit 0、ready=true）；69 个干净行全部 free≥floor、util<40、build=0；`free_min=13,442`；util_max=58 的 3 个非干净样本（streak=0）均出现在 60 连击之前，不影响 60/60 判定。
- **窗口内零编译**：窗口前一次性构建（22:37–22:39），窗口前后 live/stash exe sha/bytes/mtime 不变、build 进程 0/0；构建指纹 = guards 冻结 exe sha。
- 时间序：freeze（22:39:25）< 基线 < monitor（22:40:31–22:46:33）< 实体（22:46:50.555–22:46:53.322）< post-GPU（22:46:53.631，2,282/13,780 MiB、5%、29°C）。

### 2.3 实体运行（PASS）
- 冻结 test exe 单次：**exit 0、`running 1 test`、`1 passed; 0 failed`、wall 2,764 ms**、日志含 `r06-multiframe PASS: 3 frames, per-frame + cross-frame + option-A 3-context audit`。
- frame-000 逐相机 depth：cam0/cam1 `(0,0,0)`、cam2 `(1,0,3)` + allowlist 明细 `cam2 (134,128): bits=0x400015ba expected=0x400015b7 ulp=3 beyond4=0 flexid=0`；frame-001/002 严格 bit-first 全过（无失配打印）。
- 两跑：三帧各打印 `refits=0 two-run sha equal`；跨帧 **cam2-only** `000→001 = 1167/1074`、`001→002 = 1262/1126`（均 ≥384），全相机合计同值仅诊断。
- 参考完整性：冻结 exe `reference_integrity_15_files` → `running 1 test`、1 passed、exit 0；QA 独立重算 12 npz 对 `capture.files_sha256` + 3 文本锚定 = **15 文件**一致（并复跑冻结 exe 得 1 passed/0.17 s）。

### 2.4 证据 / 边界 / 非声明（PASS）
- 证据 manifest **14/14** 自核；文本 UTF-8/LF/无 BOM、裸 CR/TAB 0、无 U+FFFD。
- 窗口改动范围 `8c9e7192..b3a05f2b` = `guards-r06-multiframe.json` + 证据目录 + 卡；`tests/`、`src/`、`fixtures/`、`scripts/authoring/`、`criterion/`（spec `1d9fa5ce…`/10,506 B）、实体/修卡证据、G-2 证据**零 diff**；实体文件 499/496/210 窗口期未动。
- 历史原文口径说明在窗口材料；O1–O7 已知项在 README §7/receipt；收口口径只判「G-3 判据门：该 3 帧集逐帧口径 + 跨帧可见性」，不写 R06 整族/动画一致性/上游等价/发布依据。

## 3. 观察清单（按 Chief 定级规则：只影响诊断/文案/证据完整性 → observation，0 阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低 | `guards-r06-multiframe.json` 无 `-text`/eol 规则：raw 工作树 sha/bytes `64bfecdf…`/11,899（CRLF 203）≠ HEAD blob `bf4b9302…`/11,696；证据已用 LF 快照（=blob）兜底并在 README §8 说明。既有 `guards.json` 仅靠本机 `.git/info/attributes`（不随克隆传播） | 补仓库 `.gitattributes` 规则，使 raw sha 全平台/克隆可复现 |
| O2 | 低 | 禁词「动画一致性已覆盖」只在**非声明否定句**出现（README §1「**不写** …」、receipt `not_claimed`）；语义为不得写，字面 forbid_claim 扫描会误报 | 改「动画一致性结论」等避词 |
| O3 | 低 | 证据未留 `repo-write-lock` status/acquire/release 原始输出（仅 README §6「收尾释放写锁」与时间戳） | 后续窗口把 lock status/acquire/release 输出入证据 |

## 4. 复现与证据

- `scripts/verify_g3_review.py`：QA 独立复核脚本（A 冻结/aggregate/live==guard==stash、B 基线/monitor/零编译/时间序、C 实体日志/参考完整性、D 证据/边界/非声明）；只读、零 GPU、`sys.dont_write_bytecode`。
- `logs/verify-g3.log` / `logs/verify-g3.json`：**RESULT: PASS (27/27，3 observations)**。
- `logs/reference-integrity-repro.log`：复跑冻结 test exe 的 15 文件参考完整性（1 passed、0.17 s、exit 0）。
- `facts.json`：机器可读事实（冻结/窗口/实体/证据/观察）。
- `manifest.txt`：本目录 sha256 + bytes + 证据目录内相对路径（首行 `#`，不自列）。

## 5. 非声明

- 本复核只确认 **G-3 判据门 PASS（该 3 帧集逐帧口径 + 跨帧可见性）**；不写 R06 整族通过、动画一致性已覆盖、与上游实现的等价结论、任何发布依据。
- 多帧收口（tracking 卡与 roadmap 状态）由 Chief 裁。
