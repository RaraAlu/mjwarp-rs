# 逐 world 采集窗口计划（待用户双签；未执行）

任务：`2026-10-30-render-perworld-capture-window-material`（材料片，离线）
被执行的任务：`2026-10-30-render-perworld-capture`（采集跑片，GPU 满载，需用户双签）
上游缺口：`2026-10-28-render-per-world-geom-light-gap`

## 0. 一句话

用 pinned 上游环境把 **nworld=3、每 world 不同 hinge 角度** 的场景渲染一遍，采出逐 world 的 `rgb/depth/seg + 状态数组` 参考，供后续 per-world 实现片（方案 L）逐 world 位级比对。**这是采集，不是验收**：本窗口不产生任何渲染数值判定。

## 1. 脚本身份（机读）

| 项 | 值 |
| --- | --- |
| 路径 | `scripts/capture_render_perworld.py` |
| sha256 | `971cb8b13db82a9076c81b15936b2c97ffe2df8d3c0e47d1d9d0b10c78552775` |
| 字节 | **23,531** |
| 行数 | **497**（LF，≤500） |
| 非 ASCII 字节 | 0 |
| 派生 | `scripts/capture_render_hfield.py`（R05-S3a 骨架），既有采集脚本与参考零改动 |

## 2. pinned 环境（`--dry-run` 已实测，2026-10-30，未建 CUDA device）

| 项 | 值 |
| --- | --- |
| python | `target/mjwarp-parallel/venv/Scripts/python.exe`（sha256 `feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77`，3.12.13） |
| 版本 | warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4 |
| upstream | `target/mjwarp-parallel/upstream` @ `71da24d956378a87a703b6e1442b13aec0c4ac29`（clean） |
| 场景 | `fixtures/render-bvh/scene-a/scene.xml`，sha256 `a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289`，4,639 B |
| 相机 | `cam-persp`（fixed，320×240，启用）+ `cam-track`（targetbody，**不启用**）⇒ `cam_active_fixed_only=[true,false]` |
| 场景规模 | ngeom=7、nlight=3、nq=1（单 hinge `tri-hinge`） |

## 3. 逐字命令（窗口内按下表执行）

```text
# (A) 开窗前自检（不建 CUDA device、不写盘；可在窗口外先跑）
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_perworld.py --dry-run --out-dir fixtures/render-bvh/scene-a/perworld/nw3-hinge02040

# (B) 真采集（唯一一条 GPU 命令；窗口内只跑一次）
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_perworld.py --worlds 3 --hinge-deg "0,20,40" --use-fast-math false --use-shadows true --repeat 2 --out-dir fixtures/render-bvh/scene-a/perworld/nw3-hinge02040
```

可选环境变量：`MJWARP_CAPTURE_AT`（ISO8601；写进 `capture.json.captured_at`，不设置则记空串，允许）。

硬约束（脚本自检，违反即非零退出）：
- `--out-dir` 必须位于 `fixtures/render-bvh/scene-a/perworld/` 子树内，且**不得已存在非空**
- `--worlds >= 2` 且 `--hinge-deg` 值个数 == `--worlds`
- `--use-fast-math false`（产品同配方）、`--use-shadows true`、`samples_per_pixel=1`、只启用 fixed 相机
- 两跑（`--repeat 2`）逐数组 `np.array_equal` **且** npz 字节相等，否则不落盘、不宣称

## 4. 开窗前基线（5 分钟三点快照 + floor 公式）

窗口前 5 分钟内在仓库根执行并写入窗口事实（`target/render-bvh/perworld-capture-window/attempt1/pre-window-snapshot.txt`）：

```text
nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader   # ×3，间隔约 1 s
nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader                    # raw + filtered
Get-Process cargo,rustc,nvcc,msbuild                                                                  # 构建进程须为 0
git log --oneline -1 && git status --porcelain                                                       # 树干净
```

- `baseline_free_mib` = 三点中 free 的最小值（保守）
- **`floor_mib = baseline_free_mib − 3072`**（R02 先例：12858−3072=9786）
- 窗口内任一采样 `free < floor_mib` 或 `used` 增量 > 3072 MiB → 停手
- 常规门槛：`util < 40`（不预批例外、不加 `-OpenUtilLimit`）；开窗前若 util 持续 ≥40 或存在外部 compute-apps → **停手报 Chief**，不重试

## 5. 窗口时限与执行序

| 步 | 内容 | 预计 |
| --- | --- | --- |
| 0 | 确认构建进程 0、树干净；写快照 | 2 min |
| 1 | 核对 pinnen 环境（(A) 命令或直接看仓库） | 1–2 min |
| 2 | 执行 (B) 真采集（两跑 + 落盘 + capture.json） | 秒级–1 min |
| 3 | 立即复查 `floor`、核对产物清单与 sha、记录窗口事实 | 3 min |
| — | **整窗上限 30 min**；超时或任一步失败即停手 | — |

**失败即停手条款**（不重试、不改参数、不放宽判据）：
1. 任一步非零退出；2. 两跑不相等；3. legacy manifest 69 条前后不一致；4. 写盘越界（out-dir 不在子树/已存在非空）；5. free 破 floor；6. 出现外部 GPU 负载或 util 持续 ≥40。
出现上述任一 → 停手，保留 `target/` 窗口事实与原始输出，报 Chief 与用户。

## 6. 预期产物（路径 / 结构 / 近似大小）

输出目录：`fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/`

| 文件 | 数组键 | 形状 / 类型 | 近似大小 |
| --- | --- | --- | --- |
| `rgb.npz` | `rgb_w{0,1,2}_cam0` | `[240,320,3]` float32 | 3×76,800×3×4 = **2,764,800 B（≈2.64 MiB）** |
| `depth.npz` | `depth_w{0,1,2}_cam0` | `[240,320]` float32（raw planar，0=无命中） | 3×76,800×4 = **921,600 B（≈0.88 MiB）** |
| `seg.npz` | `seg_w{0,1,2}_cam0` | `[240,320,2]` int32（object_id,type；背景 (-1,-1)） | 3×76,800×2×4 = **1,843,200 B（≈1.76 MiB）** |
| `state.npz` | `qpos` / `geom_xpos` / `geom_xmat` / `light_xpos` / `light_xdir` | `(3,1)` / `(3,7,3)` / `(3,7,9)` / `(3,3,3)` / `(3,3,3)` float32 | ≈ **1.2 KB** |
| `capture.json` | — | recipe + environment + 两跑 sha + `two_captures_bitwise_identical` + `legacy_reference_untouched` | ≈ 8–20 KB（JSON） |

- npz 用 deterministic ZIP_STORED（固定时间戳），故字节可复现；每数组另有 ~百字节 `.npy` 头，总量以脚本输出 sha 为准。
- 预期 `capture.json` 关键字段：`nworld=3`、`hinge_deg=[0,20,40]`、`use_fast_math=false`、`use_shadows=true`、`two_captures_bitwise_identical=true`、`legacy_reference_untouched=true`、`legacy_reference_files=69`、`cameras=[cam-persp(320x240, mode=0)]`、`per_world_scope.covers="per-world geom poses"`、`per_world_scope.does_not_cover="per-world light poses"`。
- **口径说明**：`light_xpos/light_xdir` 一并落盘仅为完整性；本参考的覆盖声明只到 **per-world geom 位姿**（设备路径目前单 world 取灯）。

## 7. legacy 参考保护

脚本在采集前后各取一次 `fixtures/render-bvh` 的 sha256 清单（**排除整棵 `scene-a/perworld/`**，即本次输出目录），要求完全一致（本次实测清单 **69 条**）；任何变动 → 非零退出、不落盘。
已知口径：排除范围是整棵 `perworld/` 子树（QA B 复核 O3）；更细粒度的对拍留到实现片，本窗不宣称。

## 8. 用户双签栏（待填）

| 项 | 内容 |
| --- | --- |
| 窗口计划版本 | 本文件（sha256 见 receipt） |
| 用户决定 | ☐ 批准 ☐ 驳回 ☐ 修改（说明：__________） |
| 用户签名 / 日期 | ______________________ |
| Chief 整理时间 | ______________________ |
| 备注 | 采集只产参考、不做数值判定；窗口内只跑 (B) 一条 GPU 命令 |

## 9. 签字与 authoring-only util<60 例外（已签）

- **窗口计划**：user via Chief **2026-10-30T23:10:52+08:00** approve（scope 四项见 `materials/window-plan.txt`）；被批准的 plan 版本 sha256 `42791918e7b6aa2977752b5fe065590616f56d57f711479a1062ec719865935d`。
- **authoring-only 例外**：user via Chief **2026-10-31T00:00+08:00**，`mode=authoring_only_exception_util_lt_60`，同一签字覆盖本窗与 SDF 采集窗；五要素与例外 monitor 命令（`-MaxUtil 60`、新 CSV `target/render-bvh/perworld-capture-window/monitor-pre-window-attempt1.csv`、PowerShell 5.1）见 `materials/window-plan.txt` 的 `EXCEPTION BLOCK`。
- **原失败留档**：本窗未跑常规 `util<40` monitor（Chief 指令：共享 SDF 窗口失败 + 用户签字后直接走例外路径）；常规计划文本原样保留，共享失败记录在 `docs/development-evidence/render-bvh/sdf-capture/monitor/monitor-pre-window-regular-failure.csv`。
- **例外生效证据口径**：从例外 CSV 独立重算 util≥40 仍 clean 的样本与连续段、`ge60=0`、`util_max<60`；全 <40 则记 `exception_effectively_not_load_bearing=true`，不得声称 60 门槛生效。
