# R06 多帧采集窗 G-2（refit 修复后重采）：采集有效、参考已落库

- 任务：`2026-11-05-render-r06-flex-multiframe-capture-g2`（owner Backend、p82、P8-渲染与BVH；
  登记 `6d51e10f`、start `9a4a2f6e`）
- 执行者/唯一写者：**Backend**（Arch 只读不执行）；**Chief G-2 token 2026-11-05 21:43**；
  窗口 21:43–23:50（空闲段），实际执行 21:45–21:52
- 签名沿用：**S-1** `docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md`
  sha256 `1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6` / 10,506 B（**未改、不重签**）；
  **S-2** authoring 例外已预批但**本窗未启用**（常规 util<40 全程达标）
- **结论一句话**：G-2 采集**有效**（`pass-margin`），两段 cam2 变化计数均 ≥576、两次采集逐位一致、
  三条硬自查（base 幂等 / 位移 k×0.100 m / 两段变化 ≥576）与 provenance（spec sha + 三脚本 sha +
  `refit_calls=6`、`actual_sequence=[0,1,2,0,1,2]`）全过；三帧参考已落库到
  `fixtures/render-bvh/scene-flex/multiframe/`，并**supersede G-1 负结果的「帧集不足」归因**（根因是缺每帧 refit，
  已由诊断卡 `77aeb6da` + 修复卡 r2 `ff8135e5`/`43295e49` 修复）。本窗只判「采集有效性」，**不产验收结论**。

## 1. 执行序列与结果

| 步骤 | 事实 |
| --- | --- |
| 基线（5 分钟内三点，间隔 2.5 s） | 21:45:08 / 21:45:11 / 21:45:14：used 1,724/1,688/1,620 MiB，**free 14,338 / 14,374 / 14,442 MiB**，util 17/17/17%，temp 29°C；compute filtered=0、build=0 |
| **floor** | `max(min_free − 3,072 = 11,266, 11,688) = `**`11,688 MiB`**（取严）；三点 free 均 ≥ floor、util<40 ✅ |
| monitor | `monitor-until-clean.ps1 -Required 60 -MaxMinutes 30 -OutCsv target/r06-multiframe-capture-g2/monitor-g2.csv` → **samples=60 / longest_clean=60 / ready=true**（21:45:19–21:50:19，util 2–4%，**free_min 14,432 ≥ floor**，0 行低于 floor；CSV 见 `logs/monitor-g2.csv`） |
| dry-run | exit **0**；`out_dir_existed_before=false → after=false`（**未创建产物**）；`refit_policy=every-frame`、`refit_calls=6`；spec sha/bytes 校验通过（`logs/dry-run.log`） |
| 采集（一次，不重试） | 21:50:2x 启动，**exit 0**；`--frames 3 --frame-dz 0.1 --repeat 2 --out-dir fixtures/render-bvh/scene-flex/multiframe`；两次采集 `two_captures_bitwise_identical=true`；`legacy_reference_untouched=true`（既有 render-bvh 参考 84 个文件 sha 前后一致）（`logs/capture.log`） |
| 窗后 | 21:50:41：GPU used 1,541 / free 14,521 MiB / util 5% / 29°C、build=0；随后释放写锁与 GPU |

## 2. 判据与硬自查（全部 PASS）

| 项 | 结果 |
| --- | --- |
| 自检 verdict | **`pass-margin`**（`capture.json.delta_selfcheck.overall`） |
| cam2 `000→001` | depth **1,167** / seg **1,074**（≥576 ✅） |
| cam2 `001→002` | depth **1,262** / seg **1,126**（≥576 ✅） |
| ① base 幂等 | `frame-000` 的 rgb/depth/seg/state 共 **15 个数组** vs `reference/base/` **数组级位级全等**，`mismatches=[]` |
| ② 位姿位移 | 附着顶点（5 个，world 顶点 1 个）相对 base 帧平均位移 = **0.0 / 0.100000 / 0.200000 m**（容差 ±1e-6，源为采集侧 `state.npz.flexvert_xpos`） |
| ③ 两段变化像素 | 同 `000→001`/`001→002` 两项，均 ≥576 ✅ |
| refit 计次 | `refit_calls=6`、`capture_checks.refit.actual_calls=6`、**`actual_sequence=[0,1,2,0,1,2]`**、`ok=true` |
| 两遍一致性 | 逐帧逐文件 sha256 一致（`first == second`），`two_captures_bitwise_identical=true` |
| provenance | `spec_sha256=1d9fa5ce…`、`spec_bytes=10,506`；三脚本 `authoring_script_sha256` 与窗口时点文件一致；`refit_policy=every-frame`、`refit_bvh=true` |

> Arch 提醒的口径已按卡面执行：`check_frame000_vs_base` **只比对第一遍**；两遍一致性由既有逐帧逐文件 sha 比对
> 承担——两项**分别记录、不互相替代**（见 `capture.json.capture_checks` 与 `two_captures_bitwise_identical`）。

## 3. 落库产物（`fixtures/render-bvh/scene-flex/multiframe/`，15 个文件）

`frame-{000,001,002}/{rgb.npz,depth.npz,seg.npz,state.npz}`（12）+ `capture.json` + `cam-poses.json` + `README.md`。
**每文件 sha256/bytes 清单**见 `artifacts/files-sha256.json`（15/15），并逐条进入 `manifest.txt`（首行 `#` 表头）。
`capture.json` / `cam-poses.json` 副本另存 `artifacts/` 供审计（与落库文件同 sha）。

## 4. G-1 负结果的 supersede 关系

- G-1（`2026-11-05-render-r06-flex-multiframe-capture`，提交 `dbe8d2b7`）在**旧脚本（缺每帧 refit）**下得到
  `non-discriminating`（697/616、244/183），其「第二步位姿变化不足 / 疑似帧集设计」归因**不成立**；
- 根因由诊断卡 `2026-11-05-render-r06-flex-multiframe-pose-diag`（`77aeb6da`）A/B 定论：渲染消费链缺
  `mjw.refit_bvh`（A 组 flex 像素 654→246→69 vs B 组 654→718→798）；修复卡 r2（`ff8135e5` + `43295e49`）已修；
- 本窗**同一签名 spec / 同一帧集**重采得到 `pass-margin`（1,167/1,074、1,262/1,126），采集有效；
  **G-1 负结果证据保留不删**（卡面 `unchanged` 检查），本 README 为该 supersede 的书面记录。

## 5. 边界与非声明

- 写范围仅 `fixtures/render-bvh/scene-flex/multiframe/` + 本证据目录 + 卡/runs；未改
  `fixtures/render-bvh/scene-flex/reference/`（`reference/base/` 与单帧参考；脚本 `legacy_manifest` 前后比对通过）、
  `src/`、`tests/`、`scripts/`、冻结 spec（sha/bytes 未变）；零编译；
- 未启用 S-2 例外（常规 util<40 全程达标）；
- **非声明**：本窗只声明「采集有效 → 可进入验证窗（G-2 判据门）」；**不声明**多帧判据整体通过、
  动画一致性覆盖、与上游实现的等价性、任何发布依据。判据执行/验证另窗另卡。
- 已知观察/errata（记已知，不返工）：QA 复核 r2 的 O1（自检日志 U+FFFD 解码）、O2（完整实现 diff 以
  `ff8135e5` 引用）、O3（主模块 `:160` 注释缺 `mujoco_warp/` 前缀）、O4（exit3 残留流程）——其中 O4 本窗未触发
  （采集 exit 0，无残留）；诊断卡 files 含 `scripts/authoring/` 仅用于加诊断开关、不改采集语义。

## 6. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `logs/baseline-three-points.txt` | 开窗前三点基线原始输出 + floor 计算 |
| `logs/monitor-g2.csv` | monitor 60 行原始 CSV（util/free/temp/clean/streak；显式 OutCsv，不落仓原件另在 `target/`） |
| `logs/dry-run.log` / `.stderr.log` | dry-run 原始输出（command/exit_code/out_dir 未创建/refit 计划/spec 校验） |
| `logs/capture.log` / `.stderr.log` | 采集原始输出（command/exit_code/两种一致标记/自检与硬自查 JSON） |
| `logs/post-window-gpu.txt` | 窗后 GPU/构建进程/树/锁状态 |
| `artifacts/files-sha256.json` | 落库 15 个文件的 sha256/bytes 清单 |
| `artifacts/capture.json` / `artifacts/cam-poses.json` | 落库 provenance 副本（与 fixtures 同 sha） |
| `scripts/run_g2.py` | 本窗 wrapper（dry-run/capture/post-gpu/files-sha，UTF-8/LF 日志） |
| `manifest.txt` | 本目录 sha256 + bytes + 证据目录内相对路径（首行 `#` 表头，不自列） |

## 7. 复现

```powershell
# 窗口内（需 GPU 窗口 + 三点基线/monitor 前置；本窗已执行，勿在非窗口时重跑）
target/mjwarp-parallel/venv/Scripts/python.exe -B target/r06-multiframe-capture-g2/run_g2.py dry-run
target/mjwarp-parallel/venv/Scripts/python.exe -B target/r06-multiframe-capture-g2/run_g2.py capture
target/mjwarp-parallel/venv/Scripts/python.exe -B target/r06-multiframe-capture-g2/run_g2.py post-gpu
target/mjwarp-parallel/venv/Scripts/python.exe -B target/r06-multiframe-capture-g2/run_g2.py files-sha
```
