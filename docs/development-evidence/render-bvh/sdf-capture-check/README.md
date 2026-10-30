# SDF 渲染参考采集窗（U3）独立复核（QA，只读）

任务：`2026-10-30-render-sdf-capture-check`
被核：`2026-10-30-render-sdf-capture`（U3；主卡 verified）
被核 pin：**`1b185557`**（窗口本体 `e0ab7ce8`，`util_min=5` 更正 `1b185557` 在其后）；核 start 时 `revision.before = e0988285`。
**结论：pass / 阻断 0 / 需修 0 / 观察 8。**
本卡只读：零 GPU、零窗口、零设备执行；未重跑采集（GPU）——产物用 committed blob + 独立解析复核。

## 0. 一句话

作者在**用户签字的 authoring-only `util<60` 例外**下跑完采集：例外确凿 load-bearing（17 个 `util≥40` 的样本仍记为 clean、streak 继续到 60 ⇒ 实际门槛是 60 而非 40）、`ge60=0`、`util_max=42`；常规 `util<40` 失败记录（354 samples / longest_clean=21 / exit 20）原样保留且未被覆盖；两组产物**两跑 npz 位级相等**（capture.json 的 run1/run2 hash 与磁盘三方一致）、legacy 69 未动、窗口内零 Rust 编译。我唯一需要确认的 `util_min` 数字（5）已在更正提交里统一三处。

## 1. 复核方法

- 所有数字由 `git cat-file` 读 committed blob 现算；monitor/产物用本机 pinned python 独立解析（npz 逐数组形状/dtype/有限性、哈希自算）。
- 重算脚本：`recompute.py`（9 项 check → `recompute.json`，problems 0）。它同时核对 target/ 下的窗口留档与证据副本**逐字节一致**（失败记录未被覆盖）。

## 2. 结果（9 项 check，全过）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `exception.five_elements_and_retention` | ✓ | `mode=authoring_only_exception_util_lt_60`、`approved_by=user via Chief 2026-10-31T00:00+08:00`、`not_a_normal_threshold_pass=true`、`authoring_exception_used=true`、`signature_unchanged=true`；`original_normal_threshold_attempts_retained` 3 项，且 target 副本**逐字节相同**（常规失败 CSV 21,785 B、快照、failure txt、例外快照、例外 CSV 3,800 B） |
| 2 | `exception.monitor_load_bearing` | ✓ | 例外 CSV 我重算：**60/60 clean**、streak 到 60、util **30–42**、**ge40=17 且 17 个全 clean**（streak 继续 ⇒ 生效门槛 60）、**ge60=0**、`util_max=42<60`、free_min **13,171**、filtered/build=0；`clean` 列与 `util<60` 规则逐行吻合 |
| 3 | `exception.maxutil_verbatim` | ✓ | 例外命令逐字含 `powershell … monitor-until-clean.ps1 … -Required 60 -MaxMinutes 30 -MaxUtil 60`（不是只改注释） |
| 4 | `exception.regular_failure_reproduced` | ✓ | 常规失败 CSV 我重算：354 samples、longest_clean **21**、util **5–50**、ge40 **84**（全非 clean）、free_min 13,388、exit 20 记录；与 receipt 一致（`util_min=5` 更正后） |
| 5 | `products.two_run_and_arrays` | ✓ | 两组：`capture.json` 的 **run1 == context_repeat.run2 == 磁盘 npz sha**（三方一致）+ `two_captures_bitwise_identical=true`；`legacy_reference_files=69`；脚本 sha `80645968…`、场景 sha `376eb81c…`/`7869cfae…`、`nrender=2`、`use_fast_math=false/use_shadows=true/repeat=2`；数组形状/dtype 全按预期（rgb 240×320×3 / 180×240×3、depth、seg int32）且有限 |
| 6 | `boundary.legacy_and_fixtures` | ✓ | `fixtures/render-bvh` 排除 `scene-sdf/` 在窗前后均 **69** 条、列表相同、**0 字节改动**；两个场景 fixture sha 与 U2 时一致 |
| 7 | `scope.window_and_correction` | ✓ | 窗口提交 `dea7d0f1..e0ab7ce8` = **25** 路径（证据 16 + 产物 8 + 卡 1），不含 `src/tests/scripts`，fixtures 只新增 `scene-sdf/capture-sdf/**`；U3 自己的更正 = **6** 路径（README/receipt/manifest×2/window-plan/卡），其余 11 路径属于夹在区间里的 per-world 切片提交 |
| 8 | `evidence.manifests_and_correction` | ✓ | 证据 manifest 11/11、窗口材料 3/3 全等；`util_min=5` 在 README/receipt/window-plan **三处一致**；2 个 UTF-16 日志（观察 O1） |
| 9 | `card.checks_and_validate` | ✓ | 主卡 `normify check` 通过；我重跑 `validate --fingerprints` = **0 错误 / 5 警告** |

逐项原始数字见 `recompute.json`。

## 3. 观察 8（记录级接受）

- **O1 证据里 2 个 UTF-16LE 日志**：`logs/capture-sdf-mesh.log`、`logs/capture-sdf-occluder.log`（PowerShell 重定向产物，带 CRLF）；仓库历史同类很多，建议今后统一 UTF-8/LF。
- **O2 例外确凿 load-bearing**：17 个 `util≥40` 样本被记为 clean 且 streak 继续到 60 —— 这是「例外门槛 60 真的在起作用」的机器证据；单子仍按 `not_a_normal_threshold_pass=true` 记录，未写成常规通过。
- **O3 窗口内写盘范围**：Rust 构建目录 `target/debug` 在窗口时段 **0 文件改动**（最新 10-30 18:02，早于窗口），只有 warp kernel cache 落 `target/render-bvh/sdf-capture-cache`（pinned python 编译 kernel，预期）；采集命令只有两条 python，`window_commands_executed=2`。
- **O4 `util_min` 更正干净**：35 → 5 只动 6 个文档路径，产物与 monitor CSV **blob 未变**（我逐字节核过），manifest 同步刷新 → 更正没有污染原始数据。
- **O5 `captured_at` 为空串**：未设 `MJWARP_CAPTURE_AT`，脚本允许；`capture.json` 不携带采集时间，时间只能由 receipt/文件 mtime 提供。建议 U4 材料明确沿用或统一设置该 env。
- **O6 U3 pin 区间夹带 per-world 提交**：`1b185557` 之后才出现 per-world 修复，但 U3 证据本身在 `e0ab7ce8` 后只有 6 路径文档更正；我把范围核按此拆分，不把 per-world 改动算到 U3。
- **O7 两跑相等证据链完整（值得沿用）**：SDF 脚本在 `capture.json` 里同时留 run1 与 `context_repeat` 的 files_sha256，复核可做「两跑 + 磁盘」三方对拍 —— 这正是我在本卡能独立确认位级相等的原因。
- **O8 legacy 口径**：本窗 legacy = `fixtures/render-bvh` 排除 `scene-sdf/`（69），新子树不参与 legacy；U3 新增的 10 个文件（2 xml + 8 产物）会进入后续 per-world 的 legacy（79），属预期增长，核 per-world 时已单独确认。

## 4. 边界与不宣称

- 本卡只读：零 GPU、零窗口、零设备执行、零参考写入；不改 `src/tests/fixtures/scripts`、不改他人证据。
- 确认主卡口径：**采集不是验收**；不宣称 SDF 渲染已覆盖、不宣称任何渲染数值结论、不宣称与上游逐位相同；本窗产物只是 U4 的输入参考。
- 确认例外的 scope 边界：本例外是 **authoring-only**、覆盖 U3 + per-world 两个采集窗；**U4 验证窗不得复用它**（U4 走常规 `util<40` 或单独签字），我会在 U4 复核时按这条核。

## 5. 限制

- 未重跑 GPU 采集：两跑相等只能通过脚本自报 + capture.json 两跑 hash + 磁盘 sha 三方对拍确认（不重跑即无法再产生第三份独立运行）。
- 例外 monitor 的 shell 版本（5.1）来自快照/README 记录，CSV 本身不携带 shell 字段；这是既有窗口口径。
- 未对 rgb/depth/seg 的数值语义下任何判定（那属于 U4）。
