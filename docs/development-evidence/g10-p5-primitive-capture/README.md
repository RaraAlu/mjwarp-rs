# G10@P5 primitive 其余 10 组合：B0 authoring（fixtures + 上游 GPU 参考捕获）

任务：`2026-10-26-g10-p5-primitive-capture`（p48，owner Backend）。
**窗口状态：未开窗（等 Arch/Chief 双签）。** 本片只交付 fixture 与上游 GPU f32 参考，不做 host 交叉、不判产品精度。

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
## 6. 捕获窗口事实（attempt3 有效；2026-10-26 19:40–19:42）

- **开窗前基线**：`4286 MiB used / 11776 MiB free / 31-36% util / 33 °C / driver 596.36`（3 点）；`query-compute-apps` 34 条桌面/系统上下文、无本任务 CUDA 计算进程 → free≥10240 通过。
- **执行**（pinned `target/mjwarp-parallel/venv/Scripts/python.exe` 3.12.13，`device=cuda:0`，窗口内零 Rust 编译）：
  RUN A 独立进程 + cache A（wall **8.4 s**）→ RUN B 独立进程 + cache B（wall **8.2 s**）→
  `compare_primitive_capture.py` **RAW BYTE-IDENTICAL: PASS**，
  两轮 sha256 均 `13b2399d03ae39c5a579b7c20c6855b53e020e288d5c2f89b2e84531067d338c`；新参考已落
  `upstream-primitive-gpu.json`（13516 B，同一 sha）。
- **每 case（GPU）**：plane-sphere 1、plane-capsule 1、plane-ellipsoid 1、plane-cylinder 1、plane-box 3、
  plane-mesh 3、sphere-sphere 1、sphere-capsule 1、sphere-cylinder 1、capsule-capsule 1；每 case
  `two_captures_bitwise_identical=True`。
- **provenance**：interpreter `...mjwarp-parallel/venv/Scripts/python.exe` 3.12.13；mujoco 3.15.0 / warp 1.15.0 /
  mujoco_warp 3.15.0；GPU RTX 4070 Ti SUPER / driver 596.36；rev `71da24d9…`；脚本 sha；`kernel_cache_root`
  `target/g10-p5-primitive-capture/warp-cache`；`captured_at=2026-10-26T19:40:00+08:00`。
- **守卫/采样摘要**：68 样本 @0.25 s；used 峰值 **4532 MiB**（基线增量 +246，<3072 守卫）；free 最低 11530；
  util 最高 37%（本任务 GPU 步期间，无本任务步外的 util 连续越限）；进程退出、**无残留**；
  结束基线 4286/11776；post `query-compute-apps` 无本任务进程。
- **旧参考与既有 verified fixtures**：`git status --short fixtures` 仅 `?? fixtures/g10-p5-primitive/`（新目录），
  旧参考/既有 fixture **0 改动**；旧参考（`qa-upstream-gpu.json` 等）逐字节未动。
- **attempt 记录**（如实）：attempt1 因运行目录未建 → 零 GPU 作废；attempt2 因 capture 脚本 provenance 含每轮
  cache 路径（导致 raw 非逐字节一致）→ 该轮 GPU 工作量作废（不采纳）；修复为 `kernel_cache_root` 相对路径后
  attempt3 为唯一有效轮。窗口会话 1 次。
- **「nacon 预期」口径（必须如实）**：预检 `precheck-cpu.md` 的 nacon 预期由本轮 CPU 后端预检实测确定，
  **只作 fixture 有效性基线，不作参考/判据**；新参考以上游 GPU 捕获为准。
- **观察（交 Arch 判定，不自行解释为通过/失败）**：`capsule-capsule` GPU 捕获 nacon=1，而 CPU 预检为 2；
  其余 9 例 CPU 预检与 GPU 捕获 nacon 一致。该差异不影响「新参考已捕获且有 provenance」的交付，但需 Arch 决定
  B1 对照前是否先排查（本片不判产品精度）。