# R06-S3b attempt5 证据复核 + allowlist 断言实现复核 ——**PASS-with-observations（0 阻断）**

- 复核卡：`2026-11-04-render-r06-s3b-flex-allowlist-assert-check`（owner QA，p81；**attempt6 GO 前必须 PASS**）
- 对象 A：attempt5 窗口记录 `6e9aa585`（`docs/development-evidence/render-bvh/r06-s3b-flex-attempt5/`）
- 对象 B：allowlist 断言实现 `2026-11-04-render-r06-s3b-flex-allowlist-assert-fix-r2`（提交 `8c15ad75`，**verified**；证据 `docs/development-evidence/render-bvh/r06-s3b-flex-allowlist-assert/`）
- 纪律：只读、零 GPU、未重跑窗口；CPU 侧只跑普通 `cargo test` 自证；写锁仅用于本证据落库。

## A. attempt5 证据复核（`6e9aa585`）——PASS-with-observations（0 阻断）

**结论成立**：attempt5 是 **FAILED_ROUND（判据实现缺口）**，不是数值红、不是回归。

| 复核项 | 独立核对结果 |
| --- | --- |
| 实体跑成立 | `attempt/entity-flex.log`：`running 1 test`、`finished in 1.44s`、`test result: FAILED`；无 `Compiling` 行 |
| 观测落点 | 失败在 `tests/render_flex_validate.rs:347`，`left (1, 0) vs right (0, 0)`；即 cam2 `mismatched=1, beyond4=0`（ULP≤4），正是签名 allowlist 允许的状态，而冻结用例仍要求 `(0,0)` → 断言先于 allowlist 生效 |
| 部分断言 | D5 三相机位姿行在场；cam0/cam1 depth `(0,0)` 与 seg 通过、三相机 rgb 通过（按 `compare()` 断言顺序：rgb→逐相机 depth→seg，失败停在 cam2 depth）；cam2 seg / 两跑 sha **未评估**（断言顺序所致，证据如实记录） |
| 冻结指纹 | `attempt/guards-frozen.json` 7,508 B、sha256 `9b6a3535…`；counts `16/16`（frozen_exes 2 + guard_inputs 14）；`aggregate_all=8091ab5a…`、`aggregate_inputs=ec6fa222…`——我按 json 声明的算法（16 条 sha 十六进制按路径排序无分隔拼接后 sha256）**独立重算 2/2 一致**；guard_inputs 逐条 sha/bytes 与当前文件 **14/14 一致**；frozen_exes：`mjwarp-rs.exe` 85,041,741 B `6dafc24c…`、`render_flex_validate` 107,661,795 B `b92e1bed…`；`stash_sha256 == recorded` **2/2**、`pending_freeze=false` |
| freeze 日志 | `attempt/freeze.log`：`GUARDS_OK=16/16 suite=render-r06-s3b-flex exes=2 inputs=14`，aggregate/guards sha 与 json 相同（`9b6a3535…`/7508） |
| 窗口规则 v2 | canonical rebuild 一次（`14.04s`、1 次 `Compiling mjwarp-rs`）；freeze 后实体跑 `Finished ... in 0.16s`（无编译/relink）；单次跑、无重试/二次微跑记录 |
| monitor | 105 样本（`22:31:08.270`–`22:39:57.405`）；最终 **60/60** clean（最长也是 60）；外部负载插曲 8 样本（`22:31:59.163`–`22:34:52.146`，free_min **4,419** MiB、util_max **98%**，`compute_filtered`/`build` 全程 0）——与 README 描述一致；floor 11,688（= max(14,610−3,072=11,538, 沿用更严 11,688)），三点均 ≥ floor |

**观察（不阻塞）**
1. **O-A1 引用错位**：README §2 把两个 exe 的 sha/bytes 归因到 `attempt/canonical-rebuild.log`，但该日志只有编译/产物路径、**不含 sha256**；这些值实际记录在 `attempt/guards-frozen.json` 的 `frozen_exes`。
2. **O-A2 证据格式不一致**：attempt5 证据目录**无 `manifest.txt`**；`canonical-rebuild.log`/`entity-flex.log`/`freeze.log` 为 **UTF-16LE + BOM**（PowerShell 重定向产物），而仓内证据约定 UTF-8/LF（`check-evidence-bom.ps1` 只查 UTF-8 BOM，故不告警）；grep/审计会受影响，建议归档时统一为 UTF-8/LF 并补 manifest。
3. **O-A3 live==guard 不可独立复核**：json 只持久化 `stash==recorded`（2/2，我核过）与 `guard_inputs` 14/14；README 的「live==guard 2/2」是冻结工具当场断言（`GUARDS_OK` 已通过），但 live 瞬时 sha 未落盘，事后无法从证据复算。
4. **O-A4 floor 用词**：README 写「已签 11688」；11,688 实际是沿用/公式更严值（签署先例 R02=9,795、本窗公式值 11,538），取更严不违规，仅措辞可更精确。
5. **O-A5 allowlist 归属为推断**：attempt5 日志只打印 `(1,0)` 计数、未打印像素 coords/bits；「落在 allowlist 内」是结合 r2 证据（同源同码、唯一失配 `(134,128)/ULP=3`）推断。修复后的断言已补打印 bits/flat/x/y（见 B）。

## B. allowlist 断言实现复核（`8c15ad75`，r2 verified）——PASS（0 阻断）

| 复核项 | 独立核对结果 |
| --- | --- |
| 拆分与 500 行 | `tests/render_flex_validate.rs` **495 行**（≤500）+ `tests/support/render_bvh/allowlist.rs` 156 行（`#[path]` 引入、`mod allowlist;`） |
| 显式常量 | `ALLOWLIST_CAMERA=2`、`ALLOWLIST_PIXEL_X=134`、`ALLOWLIST_PIXEL_Y=128`、`ALLOWLIST_FLEXID=0`、`ALLOWLIST_SEG_TYPE=9`——全部显式常量，非散落魔数 |
| 仅该像素放宽 | `allowlist_flat = if camera == ALLOWLIST_CAMERA { y*width+x } else { usize::MAX }`；命中分支还断言参考 seg **确实** `(flexid0, 9)`（防 allowlist 误挂其它图元） |
| 无隐式减法 / 相机级放宽 | 严格区计数 `strict_mismatched/beyond/max_ulp` **只在非 allowlist 像素累加**；allowlist 像素单独判 `beyond==0 && ULP≤4`；非 cam2 时 `allowlist_flat=usize::MAX` → 全严格 |
| 失败打印 | 严格区 panic 打印 `mismatched/beyond/max_ulp/bits + flat/x/y`；allowlist 区额外打印 `bits/expected/ulp`——满足「打印 bits」要求 |
| CPU 自证（4 条，卡面 3 条之外多 1 条） | ① `allowlist_pixel_within_4_ulp_passes_cpu`（`(1,0)`、ULP=3 → PASS）；② `allowlist_pixel_beyond_4_ulp_fails_cpu`（ULP=5 → 必须失败且信息含「超 4 ULP」+`mismatched=1`）；③ `non_allowlisted_pixel_mismatch_fails_cpu`（其它像素失配 → 必须失败且含 `bits=0x`、`flat=100`、严格区只统计 1）；④ `allowlist_is_camera_scoped_cpu`（cam0/cam1 同 flat 像素照旧严格） |
| 独立复跑 | 我用**普通 `cargo test`（不碰 CUDA）**独立跑 `render_flex_validate`：`running 18 tests` → **17 passed / 0 failed / 1 ignored**（4 条新自证全过）；日志 `logs/cpu-selftest-qa.log` |
| 门禁记录 | r2 提交内 fmt/clippy/clippy-cuda/unit **全 pass、exit 0**；unit **1320/0**（较 1316 +4，与新增 4 条自证一致） |
| 边界 | 只改 `tests/`（用例 + 测试支持模块），未改产品/内核/判据/参考/`fixtures/`；零 GPU、未重冻；证据 manifest 4/4 自核通过、`artifacts/allowlist.rs` 与仓内源码逐字节相同、diff 覆盖两份文件 |

**观察（不阻塞）**
1. **O-B1 卡号引用**：support 模块 doc 注释仍引用旧卡 `…-allowlist-assert-fix`（已 abandoned）；r2 收口时可改引 r2 卡（纯文档，不影响逻辑）。

## 结论与 attempt6 前置

- **两项均 PASS-with-observations（0 阻断）**：attempt5 失败定性成立（判据实现缺口）；allowlist 断言实现符合签名口径（仅该像素、其余严格、失败打印 bits、CPU 自证走同一实现路径）。
- **attempt6 前置**：本卡 verified 后，仍需按 r2 卡**重新 canonical rebuild + 重新 freeze 16/16（exes=2、inputs=14） + Chief GO**；attempt5 的冻结指纹（`b92e1bed…`/`9b6a3535…`）**不可复用**（源码已变）。
- 不宣称 attempt5/6 通过、不宣称 flex 渲染通过；判据文本（签名 allowlist）未改，只改实现。

## 复现

```powershell
$ev = "docs\development-evidence\render-bvh\r06-s3b-flex-allowlist-assert-check"
# A) attempt5 证据核（只读）
target\mjwarp-parallel\venv\Scripts\python.exe -B "$ev\scripts\qa_attempt5_check.py" --out "$ev\logs\attempt5-check.log"
# B) 断言实现 blob 核（只读）
target\mjwarp-parallel\venv\Scripts\python.exe -B "$ev\scripts\qa_assert_check.py" --out "$ev\logs\assert-check.log"
# C) CPU 自证独立复跑（不碰 CUDA）
cargo test --locked --features cuda-probe --test render_flex_validate
```

## 产物清单

- `README.md`（本文）、`facts.json`、`manifest.txt`
- `scripts/qa_attempt5_check.py`、`scripts/qa_assert_check.py`
- `logs/attempt5-check.log`、`logs/assert-check.log`、`logs/cpu-selftest-qa.log`
