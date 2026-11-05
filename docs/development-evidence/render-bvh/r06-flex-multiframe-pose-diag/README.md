# R06 多帧诊断：位移注入链 vs 渲染消费链 —— refit A/B 定根因

- 任务：`2026-11-05-render-r06-flex-multiframe-pose-diag`（owner Backend、p60、P8-渲染与BVH）
- 日期/窗口：2026-11-05 21:02–21:12（空闲段）；执行会话 `backend`；写锁 owner=Backend（21:02:44 取）
- Chief 授权：GO 一次性（仅诊断窗口，失败即停不重试）；**当夜追加授权同卡同 GO：refit 开关 A/B 对照**
- 边界：**诊断专用**——不写 `fixtures/`、不改冻结 spec/判据/阈值/帧数/Δz/相机、不产验收材料；采集输出只落
  `target/r06-multiframe-diag/`；本目录只放诊断日志/分析/证据副本

## 1. 结论（H1 定论）

**根因 = 采集脚本帧循环缺 `mjw.refit_bvh(m, d, rc)`（H1 成立）。位移注入链完整，衰减发生在「状态 → 渲染」消费端：
场景 BVH（含 1D flex 的 capsule bounds）停在 base 帧，帧偏移越大被旧 bounds 错误剔除越多。**

| 证据链段 | A 组（不 refit，现状） | B 组（每帧 refit） | 判定 |
| --- | --- | --- | --- |
| `flexvert_xpos` 相对 frame-000 位移（附着顶点均值） | **0.100000 / 0.200000 m** | **0.100000 / 0.200000 m** | 注入链**完整**（名义 k×0.100 m 精确） |
| frame-000 vs base（四类产物数组级位级） | 全等 | 全等 | 无偏移帧与单帧参考同源 |
| cam2 flex 像素数（seg 第二分量==9；base 654） | **654 → 246 → 69**（崩塌） | **654 → 718 → 798**（正常） | 消费链在 A 组随偏移递增地丢 flex |
| cam2 变化计数 000→001（depth/seg） | **697 / 616**（= 窗口观测） | **1,167 / 1,074** | — |
| cam2 变化计数 001→002（depth/seg） | **244 / 183**（= 窗口观测，且 <384 → non-discriminating） | **1,262 / 1,126** | B 组两段均 ≥576（margin 门槛） |
| 脚本自检 verdict / exit | `non-discriminating` / **exit 3**（重现窗口） | `pass-margin` / **exit 0** | A/B 仅差 refit 一处 |
| 两组逐帧 state 六项是否逐位相同 | **逐元素位级全等**（refit 只影响渲染，不影响状态） | | 对照组有效 |

**对「0.058→0.020 m 位移衰减」的更正**：状态层不存在衰减——`flexvert_xpos` 相对 base 的附着顶点均值精确为
0.100/0.200 m（world 附着顶点恒 0，全顶点均值 0.0833/0.1667）。表面上的「位移变小」是**渲染消费端**现象：
A 组 cam2 上 flex 命中像素随偏移崩塌（654→246→69），等效于「渲染只消费了一部分位移」；B 组 refit 后命中像素
恢复并随几何上移增长（654→718→798）。因此该数字不应再作为「注入量被衰减」的依据。

## 2. 方法（A/B 仅差一个开关）

诊断开关（本卡唯一脚本改动，`scripts/authoring/capture_render_flex_multiframe.py`，**默认关**）：
`--refit-bvh` → 帧循环中 `mjw.forward(m, d)` 之后、`capture_once` 渲染之前调用 `mjw.refit_bvh(m, d, rc)`。
diff 见 `artifacts/diagnose-refit-flag.diff`；HEAD blob 的 refit 命中数为 **0**（改动前采集脚本从不 refit）。

| 组 | 命令（pinned venv；`CUDA_PATH=target/toolchains/cuda-12.8.1`） | 输出 | 时刻 / exit |
| --- | --- | --- | --- |
| A | `capture_render_flex.py --frames 3 --frame-dz 0.1 --out-dir target/r06-multiframe-diag/norefit --repeat 2` | `norefit/` | 21:11:39 / **3**（预期） |
| B | 同上 + `--refit-bvh` | `refit/` | 21:11:46 / **0** |

两组均由 wrapper 记录完整命令行/时刻/exit（`logs/*-capture.log`）；`--repeat 2` 两次采集逐位一致（两组均
`two_captures_bitwise_identical=true`）。**A 组 exit 3 属预期**（与上一窗口同判据），不重试。

## 3. 窗口纪律（S-2 authoring 例外）

- **三点基线**（21:03:52 / 21:03:55 / 21:03:57，间隔 2.5 s）：used 1,544 MiB、**free 14,518 MiB ×3**、
  util 4/2/2%、temp 29°C；compute_apps 命中本任务模式 0、构建进程 0、树 clean。
- **floor = max(min_free−3072 = 11,446, 11,688) = 11,688 MiB**（取严）；三点 free 均 ≥ floor、util < 40 ✓
- **monitor 60/60**（21:05:00–21:10:00，`-MaxUtil 60` = S-2 authoring 例外；`logs/monitor-diag.csv`）：
  util 2–4%、**free_min 14,463 MiB ≥ 11,688**、streak 1→60 全真 → ready。另人工逐点核 60 行 free 均 ≥ floor。
- **post-window**（21:12:46）：used 1,595 / free 14,467 MiB / util 2% / 29°C、构建进程 0、锁仍 HOLD（随后释放）。
- 外部 SuperMimic 负载窗口内未出现（三点/60 点均干净）；未启用任何放宽。

## 4. 根因链（上游源码事实，pinned checkout `71da24d9…`）

引文全文见 `artifacts/upstream-refit-quotes.txt`，要点：

1. `create_render_context` 内部自建 `mjd = MjData(mjm); mj_forward(...)`（`render_util.py:403-404`），
   随后 `build_scene_bvh(mjm, mjd, rc, nworld)`（`render_util.py` 末段）——**bounds 取创建时刻**
   `mjd.geom_xpos` / `mjd.flexvert_xpos`（`bvh.py:314-322`、`348-350`；1D flex 走 scene BVH 的 capsule primitives）。
2. `refit_scene_bvh` 才用**当前** `d.geom_xpos` / `d.flexvert_xpos` 重算 bounds 并 `rc.bvh.refit()`
   （`bvh.py:382-425`）；`refit_bvh` 是它的唯一入口（`bvh.py:39-44`）。
3. **`render(m, d, rc)` 全文件 0 处 refit**（`render.py` 命中 0）——BVH 不会随 `d` 自动更新。
4. 上游自己的渲染循环都**每帧先 refit 再 render**：`mujoco_warp/record.py:110-111`、`contrib/render.py:452-453`。
5. 本仓库采集脚本：HEAD blob（改动前）refit 命中 **0**；`rc` 在帧循环外**只创建一次**（基于无偏移的 base 模型），
   其后每帧 `body_pos.z += k*0.1` 只改模型/数据，不重建或 refit BVH → 渲染遍历始终按 base bounds 剔除。

修复建议（**另卡**，本卡不实施、生产默认不变）：每帧 `mjw.forward` 后、`capture_once` 前调用
`mjw.refit_bvh(m, d, rc)`（B 组已验证 pass-margin）；或每帧重新 `create_render_context`。是否转正为生产行为由
Chief 在诊断结论后决定。

## 5. 数据明细

- **位移**（`analysis/analysis.json` → `B_flexvert_displacement`）：flex-line 4 顶点 + flex-line-fat 2 顶点；
  顶点 0 附着 world（恒 0），其余 5 个附着 `flex-anchor0/1/2`（每帧 +0.100 m）。
- **cam2 flex 像素**：base 参考 654；A [654, 246, 69]；B [654, 718, 798]（`seg` 第二分量==9）。
- **cam0/cam1**：flex 不可见（flex 像素 0）、刚性场景静态 → 两组变化计数均 0，符合预期。
- **变化像素上的 |Δdepth| 均值**：A 1.70 / 1.54 m，B 1.81 / 1.81 m（含 flex 出现/消失像素，仅作参考）。
- **cam2 flex 顶点投影 bbox（uv；u=列, v=行）**：A/B 相同（同一 state）：frame-000 `[135.8,195.7,101.3,139.0]`、
  frame-001 `[135.5,196.1,86.6,129.5]`、frame-002 `[135.2,196.5,71.6,129.5]`——几何按注入上移（v 减小）。
  该投影只作几何一致性佐证，不参与判定。

## 6. 处置与非声明

- 上一窗口的三帧参考**仍未落库**（`fixtures/render-bvh/scene-flex/multiframe/` 不存在；本卡 `fixtures/` 零 diff）；
- 本卡不产验收材料、不改冻结 spec（sha `1d9fa5ce…`/10,506 B 未变）、不改判据/阈值/帧数/Δz/相机；
- 诊断开关 `--refit-bvh` 默认关（生产默认行为不变）；A 组计数与上一窗口 697/616、244/183 **逐值一致**，
  证明默认关路径行为未变；
- 不声称多帧判据可用、不声称与上游一致、不作发布依据；本结论只服务下一轮「refit 是否转正 + 新帧集」决策。

## 7. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `analysis/analysis.json` | A/B 全量分析（位移逐顶点、逐相机变化计数、投影、组间 state 对照） |
| `analysis/analysis-summary.txt` | 上述分析的控制台摘要 |
| `analysis/verify-diag.json` | 本目录自检 16/16 PASS（`scripts/verify_diag.py` 输出） |
| `groups/{norefit,refit}/capture.json` / `cam-poses.json` | 两组采集 provenance 与相机位姿（帧数据本身在 `target/`，不入库） |
| `groups/{norefit,refit}/frame-{000,001,002}/state.npz` | 六项状态小样本（可据此独立重算位移与组间位级同一性） |
| `logs/baseline-three-points.txt` | 开窗前三点基线原始输出 |
| `logs/monitor-diag.csv` | monitor 60 行原始 CSV（util/free/temp/clean/streak） |
| `logs/{norefit,refit}-dry-run.log` | 两组 CPU dry-run（spec sha 校验、refit_bvh 标记） |
| `logs/{norefit,refit}-capture.log` / `.stderr.log` | 两组采集 stdout/stderr（含命令行/时刻/exit_code） |
| `logs/post-window-gpu.txt` | 窗后 GPU/构建进程/树/锁现场 |
| `artifacts/diagnose-refit-flag.diff` | 本卡唯一脚本改动（诊断开关，默认关） |
| `artifacts/upstream-refit-quotes.txt` | 上游 refit 语义引文（行号 + HEAD blob refit=0 对照） |
| `scripts/analyze_ab.py` | A/B 分析脚本（CPU-only，可从 target/ 复跑） |
| `scripts/run_capture.py` | 采集 wrapper（记录命令行/时刻/exit，UTF-8/LF 日志） |
| `scripts/verify_diag.py` | 本目录自检（A–F 组 16 项） |
| `manifest.txt` | 本目录 sha256 + bytes + 证据目录内相对路径（base dir 见表头，不自列） |

## 8. 复现

```powershell
# 1) 证据自检（只读仓库 + 本目录，零 GPU）
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/r06-flex-multiframe-pose-diag/scripts/verify_diag.py

# 2) 重跑 A/B（需 GPU 窗口 + 本目录 logs 前置记录；输出只写 target/）
target/mjwarp-parallel/venv/Scripts/python.exe -B target/r06-multiframe-diag/run_capture.py --label norefit --out-dir target/r06-multiframe-diag/norefit
target/mjwarp-parallel/venv/Scripts/python.exe -B target/r06-multiframe-diag/run_capture.py --label refit --out-dir target/r06-multiframe-diag/refit --refit-bvh
target/mjwarp-parallel/venv/Scripts/python.exe -B target/r06-multiframe-diag/analyze_ab.py
```
