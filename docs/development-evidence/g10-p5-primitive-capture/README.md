# G10@P5 primitive 其余 10 组合：B0 authoring（fixtures + 上游 GPU 参考捕获）

任务：`2026-10-26-g10-p5-primitive-capture`（p48，owner Backend）。
**窗口状态：捕获已完成（更正窗口 attempt4 为唯一有效轮，两轮 raw 逐字节一致；captured_at 为窗口开始真实时间）；旧 attempt3 记录见 §6、最终以 §7 为准。** 本片只交付 fixture 与上游 GPU f32 参考，不做 host 交叉、不判产品精度。

## 1. dispatch 审计（已完成，纯 CPU）
见 `dispatch-audit.md`：`nativeccd=disable` 下 10/10 组合路由 `CollisionType.PRIMITIVE`，wrapper 映射：
plane 6（L48-53）→ `plane_*_wrapper`（mesh 走 `plane_convex_wrapper`），sphere-sphere/capsule/cylinder 与
capsule-capsule（L60/61/63/66）→ `sphere_*_wrapper`/`capsule_capsule_wrapper`；(BOX,BOX) L78 CONVEX 由
L910-912 NATIVECCD 覆盖为 PRIMITIVE → `box_box_wrapper`（既有参考，不重捕）。

## 2. fixtures（`fixtures/g10-p5-primitive/`，既有 verified fixtures 0 改动）
10 个 MJCF：plane-sphere/capsule/ellipsoid/cylinder/box/mesh、sphere-sphere（**新文件**，不改
`fixtures/mjwarp-complex-physics/sphere-sphere.xml`）、sphere-capsule、sphere-cylinder、capsule-capsule；
每个含 `<flag nativeccd="disable"/>` 与确定性 `<key name="capture">` qpos；`plane-mesh` 附 `tetra.obj`。
输入 sha 清单与 qpos f32 位型由捕获 JSON 记录（`qpos_bits`/`xml_sha256`）。窗口外已用 pinned python 做 CPU 预检（`precheck-cpu.md`）：10/10 解析成功、ngeom=2、NATIVECCD 置位、nacon 与几何预期一致；**预检只证 fixture 有效，参考仍来自窗口内 GPU 捕获**。

## 3. 捕获脚本（`scripts/`，已入仓，尚未执行）
- `scripts/capture_primitive_gpu.py`：pinned python/mujoco/warp/mujoco_warp，`device=cuda:0`，
  参数化 10 case；每 case 进程内两次捕获并记录 `two_captures_bitwise_identical`；provenance 全项
  （python/mujoco/warp/mujoco_warp/numpy 版本、GPU 型号/驱动、脚本 sha、cache 目录、captured_at）。
- `scripts/compare_primitive_capture.py`：两轮 raw JSON **逐字节**比较；不同 → exit 2，停手报 Arch
  （不接受 canonical-only 相等）。

## 4. 窗口计划（待双签后执行）
- 前置：采样器 dry-run 19/19 PASS（`docs/development-evidence/gpu-window-guard-sampler/`）。
- 步骤：开窗基线重采 → Run A（独立进程 + cache A）→ Run B（独立进程 + cache B）→ raw 逐字节比较 →
  post 快照 + 结束基线 + `query-compute-apps`（无残留）+ 0.25 s 采样摘要。
- 预计 wall <5 min、峰值显存增量 <1 GiB；窗口数 1；窗口内零 Rust 编译。
- 产物：`upstream-primitive-gpu.json`（双轮 sha 相同才落盘）。

## 5. 边界
- 不判产品精度；后续设备片对照口径 = 「device vs 本片上游 GPU 参考」（位级优先/既有 4 ULP 判据，不放宽）。
- 未实现组合照旧记未覆盖；若 plane-mesh 后续设备实现需要 mesh polygon ABI → 停手报 Arch，本片不绕。
- ABI：本片 0 新 ABI 字段。
## 6. 捕获窗口事实（历史 attempt3，2026-10-26 19:23 窗口；其参考已被 §7 的 attempt4 替换）

> 留档说明：本节记录当时 attempt3 的产出与数字；QA 复核发现其 `captured_at` 为写死常量，已由 §7 更正窗口 attempt4 重采替换。**当前有效参考与窗口事实以 §7 为准**，本节数字只作历史留档。

- **开窗前基线**：`4286 MiB used / 11776 MiB free / 31-36% util / 33 °C / driver 596.36`（3 点）；`query-compute-apps` 34 条桌面/系统上下文、无本任务 CUDA 计算进程 → free≥10240 通过。
- **执行**（pinned `target/mjwarp-parallel/venv/Scripts/python.exe` 3.12.13，`device=cuda:0`，窗口内零 Rust 编译）：
  RUN A 独立进程 + cache A（wall **8.4 s**）→ RUN B 独立进程 + cache B（wall **8.2 s**）→
  `compare_primitive_capture.py` **RAW BYTE-IDENTICAL: PASS**，
  两轮 sha256 均 `13b2399d03ae39c5a579b7c20c6855b53e020e288d5c2f89b2e84531067d338c`（该参考已被 §7 的 `3a6475ac…` 替换，仅留档）。
- **每 case（GPU）**：plane-sphere 1、plane-capsule 1、plane-ellipsoid 1、plane-cylinder 1、plane-box 3、
  plane-mesh 3、sphere-sphere 1、sphere-capsule 1、sphere-cylinder 1、capsule-capsule 1；每 case
  `two_captures_bitwise_identical=True`。
- **provenance**：interpreter `...mjwarp-parallel/venv/Scripts/python.exe` 3.12.13；mujoco 3.15.0 / warp 1.15.0 /
  mujoco_warp 3.15.0；GPU RTX 4070 Ti SUPER / driver 596.36；rev `71da24d9…`；脚本 sha；`kernel_cache_root`
  `target/g10-p5-primitive-capture/warp-cache`（attempt3 留档）；`captured_at=2026-10-26T19:40:00+08:00`（写死常量，已由 §7 修正）。
- **守卫/采样摘要**：68 样本 @0.25 s；used 峰值 **4532 MiB**（基线增量 +246，<3072 守卫）；free 最低 11530；
  util 最高 37%（本任务 GPU 步期间，无本任务步外的 util 连续越限）；进程退出、**无残留**；
  结束基线 4286/11776；post `query-compute-apps` 无本任务进程。
- **旧参考与既有 verified fixtures**：`git status --short fixtures` 仅 `?? fixtures/g10-p5-primitive/`（新目录），
  旧参考/既有 fixture **0 改动**；旧参考（`qa-upstream-gpu.json` 等）逐字节未动。
- **attempt 记录**（如实）：attempt1 因 capture 脚本 line 105 残留字面反引号n 的 SyntaxError → 零 GPU 作废；attempt2 因 capture 脚本 provenance 含每轮
  cache 路径（导致 raw 非逐字节一致）→ 该轮 GPU 工作量作废（不采纳）；修复为 `kernel_cache_root` 相对路径后
  attempt3 曾作为有效轮（其后因 captured_at 真实性问题由 §7 的 attempt4 取代）；窗口会话合计 2 次（attempt3 + attempt4）。
- **「nacon 预期」口径（必须如实）**：预检 `precheck-cpu.md` 的 nacon 预期由本轮 CPU 后端预检实测确定，
  **只作 fixture 有效性基线，不作参考/判据**；新参考以上游 GPU 捕获为准。**参考尚不是判据**：本片只交付「新组合上游 GPU 参考已捕获且有 provenance」，不判任何组合通过。
- **窗口轮次口径（O7）**：attempt1/2/3 为三次采样会话（samples 8 / 100 / 68）；attempt3 曾产出后被 superseded；**attempt4 为更正窗口唯一有效轮（RUN A+B，wall 7.9 s / 7.9 s）；GPU 窗口会话合计 2 次（attempt3 + attempt4）**。**runner 位置（O8）**：runner 位于 gitignored `target/`（不入库）；轮次独立性由 `window-attempt3/run-a.log`/`run-b.log` 与独立 cache（`attempt3/warp-cache-a`、`-b`）旁证。**kernel_cache_parent（O4）**：provenance 记录 `target/g10-p5-primitive-capture/attempt4`（窗口内真实存在、两轮相同，保证 raw 逐字节一致），每轮 cache 子目录见 run 日志（§7）。**观察（交 Arch 判定，不自行解释为通过/失败）**：`capsule-capsule` GPU 捕获 nacon=1，而 CPU 预检为 2；
  其余 9 例 CPU 预检与 GPU 捕获 nacon 一致。该差异不影响「新参考已捕获且有 provenance」的交付，但需 Arch 决定
  B1 对照前是否先排查（本片不判产品精度）。

## 7. 更正窗口（attempt4，2026-10-26 19:34:44–19:35:04，captured_at 真实性修正）

- **修正点**：删除 runner 写死 captured_at；窗口开始取真实时间（带时区）并同一值 export 给 A/B。
- **新两轮 sha（逐字节一致）**：A=B=`3a6475acf362795ab98e8ee39309b09102c2dc76f7a57c6c6dbc95a069a98734`；新参考 `upstream-primitive-gpu.json` 同 sha（替换旧修正前参考，属修正提交）。
- **captured_at 与窗口日志交叉核验**：`captured_at=2026-10-26T19:34:44.106+08:00` = `window_start`；`window_end=2026-10-26T19:35:04.312+08:00` → **captured_at 落在 [window_start, window_end] 内**；mtime：baseline 19:34:43.393（窗口前）、run-a 19:34:51.983、reference 19:34:51.673、run-b 19:34:59.894、samples 19:35:01.772、facts 19:35:04.325；samples 时间戳 19:34:44.26–19:35:01.77 → **与窗口同区间**。
- **采样摘要**：64 样本 @0.25 s；used 峰值 **4406 MiB**（基线 4170–4192，增量 <3072 守卫）；free 最低 11656；util 最高 39%（本任务 GPU 步期间）；进程退出、无残留；开窗基线 3 点 `4192/11870, 4180/11882, 4170/11892`，结束基线 `4161/11901`；post compute-apps 无本任务进程。
- **执行**：RUN A wall 7.9 s / RUN B wall 7.9 s，exit 0；窗口内零 Rust 编译；window-attempt4 日志入档。
- **kernel_cache_parent（O4 闭环）**：provenance `target/g10-p5-primitive-capture/attempt4`（窗口内真实存在，两轮相同）；每轮子目录 `attempt4/warp-cache-a`/`-b` 由 run 日志/实际目录旁证。
- **capsule-capsule 观察保留**（Chief 裁定 B0 不排查、如实记录）：GPU 捕获 nacon=1，CPU 预检 2；B1 判据参考为 GPU 参考（nacon=1），不得按 CPU 预检凑 count；本片不解释为通过/失败。
- **旧参考/既有 fixtures**：`git status --short fixtures` 无 tracked 改动；旧参考逐字节未动。
- **轮次口径（O7）**：attempt1/2/3 为三次采样会话（8/100/68 样本，attempt3 曾产出被 superseded 的参考）；**attempt4 为更正窗口唯一有效轮**；GPU 窗口会话合计 2（attempt3 + attempt4）。
