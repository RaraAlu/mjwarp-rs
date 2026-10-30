# R08-RGB 逐相机掩码验证窗（attempt=1，常规 util<40 阈值，未使用例外）

任务：`2026-10-29-render-r08-rgb-hetero-validate`（窗口执行片）
上游离线片：`2026-10-29-render-r08-rgb-hetero`；窗口材料：`docs/development-evidence/render-bvh/r08-rgb-hetero-window-material/`
窗口时段：**2026-10-30 12:05:31–12:06:27（+08:00）**，已签常规窗口（`util<40`，无例外、无 `-OpenUtilLimit`、判据零改动）。

## 0. 一句话结论

9 实体（1 new + 8 回归）全部 exit 0；`sha_guards=77/77`；窗后 exe 复查 9/9、mismatches=0；`zero_rust_compile=True`（deps 1839 前后一致）；窗内 free_min 13,179 MiB 未破 floor、util_max 7%、`util_streak_ge_80=0`；新实体逐相机判据 `rgb beyond_1_lsb=0 / depth mismatch=0 max_ulp=0 / seg mismatch=0`（120,000 像素×4 个矩阵用例全 0），两跑 sha 相等；8 回归按既有位级判据零漂移。**本片只记录本窗数字，不宣称超出本窗范围的结论。**

## 1. monitor（常规阈值；本轮首次通过）

第五轮（本轮）：`monitor/monitor-pre-window.csv`，60 samples / **longest_clean=60** / **ready=True** / exit 0 / `12:00:05.796–12:05:05.753`；`util 2–8`、`free_min=13452`、`ge40=0`、`ge60=0`、`compute_filtered_max=0`、`build_max=0`。独立重算见 `monitor/recompute.json`（脚本 `target/recompute-exception-monitor.py`，sha256 `088d4358…`）。
**未使用任何例外**：`not_a_normal_threshold_pass=false`、`authoring_exception_used=false`、命令里没有 `-OpenUtilLimit`。

历史四轮（全部失败、原样保留，**不作为本轮证据**）：

| 轮 | 文件 | samples / longest_clean | 原因 |
| --- | --- | --- | --- |
| 1 | `monitor-history/monitor-pre-window-round1.csv` | 354 / **0** | 外部 viz python（pid 69096）全程持 CUDA context（filtered=1） |
| 2 | `monitor-history/monitor-pre-window-round2.csv` | 354 / **37** | 01:5x–02:1x 外部重负载（used→15.6 GiB、util 99–100%） |
| 3 | `monitor-history/monitor-pre-window-round3.csv` | 354 / **7** | 外部重负载持续（util 99%、47 °C） |
| 4 | `monitor-history/monitor-pre-window-round4.csv` | 354 / **37** | 桌面 3D 引擎常驻尖峰（dwm/DSH GUI/msedgewebview2），util 35–44、ge40=40、filtered=0、build=0；见 `monitor-history/monitor-round4-failure.txt` |

## 2. guards 冻结（attempt=1，新 suite `render-r08-rgb-hetero`）

| 项 | 值 |
| --- | --- |
| 冻结器 | `scripts/validate/render-bvh/freeze-guards-r08-rgb-hetero.ps1` sha256 `0c66fdbb3b5a8101e6e3fd1618920526aed1231552aafc13c92259011278c3ac`、8,761 B |
| guards 文件 | `scripts/validate/render-bvh/guards-r08-rgb-hetero.json` sha256 **`e81d69a4daa0268777eff698ecf7621dee333076915f805c2e447cc9d0c34f3a`**、**28,970 B** |
| 条目 | **77** = 9 exe + 68 只读输入（8 hfield capture + 2 hfield xml + 42 legacy(38+4 skybox) + 16 shadow） |
| aggregate | `3ad8b3522314222629f1cf8e5a9703a9c53ad75433373d0c06f402803a517f1c` |
| stash | `target/render-bvh/r08-rgb-hetero-window/frozen-exes/` 9/9，逐 exe 字节与 sha 与冻结记录一致 |
| `-GuardsOnly` | `GUARDS_OK=77/77` + `GUARDS_ONLY_DONE`（exit 0） |
| R05/R02 guards | 未改动 |

## 3. 9 实体结果（`entity-results.csv`）

| 实体 | 类型 | exit | wall_ms | 用例 |
| --- | --- | --- | --- | --- |
| `render_r08_cameras` | **new** | 0 | 11,644 | 8 passed / 0 failed |
| `render_camera_rays` | 回归 | 0 | 1,312 | 2 passed |
| `render_context_lifecycle` | 回归 | 0 | 7,817 | 5 passed |
| `render_e2e` | 回归 | 0 | 8,822 | 5 passed |
| `render_kernel_depth_seg` | 回归 | 0 | 4,497 | 6 passed |
| `render_kernel_flags` | 回归 | 0 | 9,378 | 6 passed |
| `render_kernel_rgb` | 回归 | 0 | 3,639 | 3 passed |
| `render_r10_multi_world` | 回归 | 0 | 2,714 | 2 passed |
| `render_skybox_reference` | 回归 | 0 | 1,605 | 1 passed |

窗后 exe 复查 **9/9**、`mismatches=0`；`residual_processes=0`；`attempt_valid_rounds=1`。

## 4. 新实体 `render_r08_cameras` 逐相机判据（`logs/render_r08_cameras.log`）

**异构 RGB（`gpu_heterogeneous_rgb_channels`）**
- cam0_on：`rgb_adr=[0, -1]`，sha256 `f62f4d733bf5dfc6ac0b3eb8b582193aa50341611e73b3cc2154dedf7ab61b22`
- cam1_on：`rgb_adr=[-1, 0]`，sha256 `1325755cbcdb89578231fa95e673052ff8283acc33db258cb43913f04d6c68c1`
- all_off：`rgb_adr=[-1, -1]`，sha256 `b09e2eeccd48c880ad44e257ef8ecc2da84973af09cd7866fe63d4a3975a0c31`

**异构 depth/seg（`gpu_heterogeneous_depth_seg_channels`）**
- `depth_adr=[0, -1]`、`seg_adr=[-1, 0]`：逐相机对参考成立；未启用相机拒绝成立（含 `MegakernelRenderer` 负偏移回归）

**矩阵用例（`gpu_matrix_render_matches_frozen_reference`）**

| 用例 | 相机 | rgb 像素 / beyond_1_lsb | depth 像素 / mismatch / max_ulp | seg 对 / mismatch | sha256 |
| --- | --- | --- | --- | --- | --- |
| all | 2 | 120000 / 0 | 120000 / 0 / 0 | 120000 / 0 | `f9fe1b9e…84be688` |
| mask_first | 1 | 76800 / 0 | 76800 / 0 / 0 | 76800 / 0 | `cfc1a4b6…5789d66` |
| names_all | 2 | 120000 / 0 | 120000 / 0 / 0 | 120000 / 0 | `f9fe1b9e…84be688` |
| indices_reversed | 2 | 120000 / 0 | 120000 / 0 / 0 | 120000 / 0 | `73891435…8c998b3` |

**cam1 三通道全关（`gpu_cam1_all_channels_off`）**：cam0 三通道对参考；cam1 rgb/depth/seg 全部拒绝（all-off early-return）— pass。
**隔离与重排（`gpu_isolation_subset_and_reorder_bitwise`）**：subset_cam0 = 76800/0、subset_cam1 = 0/43200、reorder = 76800/43200，均与组合渲染逐位相同 — pass。
**动态 vs 预计算 depth/seg（`gpu_dynamic_rays_depth_seg_bitwise_matches_precomputed`）**：逐位相同，sha256 `b09e2eec…975a0c31`。
**两跑确定性（`gpu_two_runs_sha256_identical`）**：run0 = run1 = `f9fe1b9e…84be688`。
**空选择 no-op（`gpu_empty_selection_render_is_noop`）**：mega/depth-seg 均 Ok no-op、缓冲未分配、显存无增量、无可读输出 — pass。

## 5. 8 回归零漂移

8 个回归实体各自沿用既有位级/ULP 判据，全部 `test result: ok`（0 failed），未改判据、未更新参考：
`render_camera_rays`（2）、`render_context_lifecycle`（5）、`render_e2e`（5）、`render_kernel_depth_seg`（6）、`render_kernel_flags`（6）、`render_kernel_rgb`（3）、`render_r10_multi_world`（2）、`render_skybox_reference`（1）。

## 6. 窗口资源事实（`facts.txt`、`samples.csv`、`baseline.txt`、`machine-load.txt`）

- 窗口 `12:05:31.632–12:06:27.460`，`body_wall_s=52.7`，`samples=177`
- baseline（run-window precheck）：`2607,13455,4,31 / 2607,13455,2,31 / 2607,13455,4,31`；`floor_mib=10240`（`OpenFreeMiB` 默认）
- 窗内：`used_peak=2883 MiB`、`used_min=2605 MiB`、**`free_min=13179 MiB`**、`util_max=7%`、`util_streak_ge_80=0`
- `delta_vs_guard_baseline=276 MiB ≤ increment_limit 3072`：`increment_exceeded=False`、`floor_breached=False`、`guard_aborted=False`
- `zero_rust_compile=True`：deps `1839 → 1839`、max_write 同为 `10/30/2026 02:53:25`
- 窗后 compute-apps：raw 31 / filtered 0；窗后 exe 复查 9/9

**开窗前快照与 floor 的两个口径（都未破）**：
- `monitor/pre-window-snapshot-attempt5.txt`：pre-body 三点 min free **13,446** ⇒ `floor = 13,446 − 3,072 = 10,374`
- `facts.txt`：run-window 自带 `floor_mib=10240`（`OpenFreeMiB` 默认）
- 窗内 `free_min=13,179` ⇒ 两个 floor 均未破

## 7. 留痕与工具事实（如实记录）

- 第一次 pre-window 快照（12:00:03）用 .NET 相对路径 `[IO.File]::WriteAllText` 写盘，解析到 harness 进程 cwd 而失败；三个采样只在内存中，**仅最小值 13,453 留在 stdout**，逐条数值未持久化。快照已改用**绝对路径**重写，并补三点 pre-body 采样（min 13,446）与 floor 计算。**未重试 monitor、未改门槛、未影响门禁**（monitor 自身 5 s 采样 CSV 是权威记录）。
- 本轮 monitor 之前已把第四轮 CSV 归档到 `history/monitor-pre-window-attempt4.csv`（本证据再复制为 `monitor-history/monitor-pre-window-round4.csv`），避免覆盖。
- 冻结后未再跑额外构建（窗内 zero compile 由 deps 计数与 max_write 证明）。

## 8. 文件清单

`manifest.txt` 给出全部文件的 path / bytes / sha256。目录约定：
`facts.txt`、`baseline.txt`、`machine-load.txt`、`guard.log`、`sha-guards.txt`、`entity-results.csv`、`samples.csv`、`guard_aggregate.json`、`window-command.txt`、`logs/*.log`（9）、`monitor/*`（本轮 CSV + 快照 + 重算）、`monitor-history/*`（四轮失败 CSV + 第 4 轮快照/失败判定/重算）、`materials/entity-table.json`（窗口材料实体表副本）。

## 9. 不宣称

- 不改判据 / 容差 / 参考 / 实体表；本片只记录本窗数字。
- 不把本轮写成对其它窗口或其它片的结论：SDF cull 解锁、per-world 渲染、mesh 容量、性能均不在本片范围。
- 不宣称上游等价性、产品级验收或跨平台结论。
