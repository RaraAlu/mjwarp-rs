# 逐 world 采集窗口计划（已签；attempt2 修正后）

任务：`2026-10-30-render-perworld-capture-window-material`（材料片，离线）
被执行的任务：`2026-10-30-render-perworld-capture`（采集跑片，GPU 满载，需用户双签）
上游缺口：`2026-10-28-render-per-world-geom-light-gap`

## 0. 一句话

用 pinned 上游环境把 **nworld=3、每 world 不同 hinge 角度** 的场景渲染一遍，采出逐 world 的 `rgb/depth/seg + 状态数组` 参考，供后续 per-world 实现片（方案 L）逐 world 位级比对。**这是采集，不是验收**：本窗口不产生任何渲染数值判定。
本材料在 attempt1 采集失败后按窗口期修复流程更正了 4 处事实（见 §10），用户已签的**窗口范围与例外未变**。

## 1. 脚本身份（机读）

| 项 | 值 |
| --- | --- |
| 路径 | `scripts/capture_render_perworld.py` |
| sha256 | `67a760fa400f2cf89c19a700e49d08c619a78008664f03cd69fab21bd2077815`（attempt2；旧 `971cb8b1…`/23,531 B/497 行见 §10） |
| 字节 | **24,008** |
| 行数 | **499**（LF，≤500） |
| 非 ASCII 字节 | 0 |
| 派生 | `scripts/capture_render_hfield.py`（R05-S3a 骨架），既有采集脚本与参考零改动 |

## 2. pinned 环境（`--dry-run` 已实测，未建 CUDA device）

| 项 | 值 |
| --- | --- |
| python | `target/mjwarp-parallel/venv/Scripts/python.exe`（sha256 `feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77`，3.12.13） |
| 版本 | warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4 |
| upstream | `target/mjwarp-parallel/upstream` @ `71da24d956378a87a703b6e1442b13aec0c4ac29`（clean） |
| 场景 | `fixtures/render-bvh/scene-a/scene.xml`，sha256 `a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289`，4,639 B |
| 相机 | id 0 `cam-persp`（fixed，320×240，启用）+ id 1 `cam-track`（targetbody，**不启用**）⇒ `cam_active_fixed_only=[true,false]` |
| 场景规模 | ngeom=7、nlight=3、**njnt=2**（FREE 基座 + 1 个 HINGE）、**nq=8**、`jnt_qposadr=[0,7]` |
| 默认位形 | `qpos0=[0.1,-0.2,0.3,0.961636,0.10017,-0.050085,0.250426,0.0]` |
| 每 world qpos | 以 **`qpos0` 为基座**（不是全零），只覆盖 hinge 地址 7 ⇒ 形状 **(3,8)** float32，hinge 弧度 `[0, 0.349066, 0.698132]` |
| legacy | 运行时 **79** 条（69 既有渲染参考 + 10 个 SDF 采集参考文件），采集前后对拍 |

dry-run 静态模型自检（`perworld-capture-authoring/materials/dryrun-plan.json`）：`njnt/jnt_type/jnt_qposadr/nq/qpos0`、每 world hinge 角度与 qpos、相机列表（id/mode/name）、legacy 计数全部打印，材料假设与真实场景不符会在**开窗前**失败。

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
- **标量 joint 只取 SLIDE/HINGE，要求恰好 1 个 HINGE**（FREE/BALL 允许且不计入）；基座 qpos 用模型默认 `qpos0`
- 两跑（`--repeat 2`）逐数组 `np.array_equal` **且** npz 字节相等，否则不落盘、不宣称

## 4. 开窗前基线（5 分钟三点快照 + floor 公式）

窗口前 5 分钟内在仓库根执行并写入窗口事实（`target/render-bvh/perworld-capture-window/attempt2/pre-window-snapshot.txt`）：

```text
nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader   # ×3，间隔约 1 s
nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader                    # raw + filtered
Get-Process cargo,rustc,nvcc,msbuild                                                                  # 构建进程须为 0
git log --oneline -1 && git status --porcelain                                                       # 树干净
```

- `baseline_free_mib` = 三点中 free 的最小值（保守）
- **`floor_mib = baseline_free_mib − 3072`**（R02 先例：12858−3072=9786）
- 例外窗口门槛：`util < 60`（用户签的 authoring-only 例外，`-MaxUtil 60`，**不是常规通过**）；ge60 必须为 0
- 窗口内任一采样 `free < floor_mib` 或 `used` 增量 > 3072 MiB → 停手

## 5. 窗口时限与执行序

| 步 | 内容 | 预计 |
| --- | --- | --- |
| 0 | 确认构建进程 0、树干净；写快照 | 2 min |
| 1 | 例外 monitor（powershell 5.1、`-MaxUtil 60`、新 CSV `attempt2`） | ≤30 min |
| 2 | 执行 (B) 真采集（两跑 + 落盘 + capture.json） | 秒级–1 min |
| 3 | 立即复查 `floor`、核对产物清单与 sha、记录窗口事实 | 3 min |
| — | **整窗上限 30 min**；超时或任一步失败即停手 | — |

**失败即停手条款**（不重试、不改参数、不放宽判据）：
1. 任一步非零退出；2. 两跑不相等；3. legacy manifest **79** 条前后不一致；4. 写盘越界（out-dir 不在子树/已存在非空）；5. free 破 floor；6. 出现外部计算负载或 util ≥60。
出现上述任一 → 停手，保留 `target/` 窗口事实与原始输出，报 Chief 与用户。

## 6. 预期产物（路径 / 结构 / 近似大小）

输出目录：`fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/`

| 文件 | 数组键 | 形状 / 类型 | 近似大小 |
| --- | --- | --- | --- |
| `rgb.npz` | `rgb_w{0,1,2}_cam0` | `[240,320,3]` float32 | 3×76,800×3×4 = **2,764,800 B（≈2.64 MiB）** |
| `depth.npz` | `depth_w{0,1,2}_cam0` | `[240,320]` float32（raw planar，0=无命中） | 3×76,800×4 = **921,600 B（≈0.88 MiB）** |
| `seg.npz` | `seg_w{0,1,2}_cam0` | `[240,320,2]` int32（object_id,type；背景 (-1,-1)） | 3×76,800×2×4 = **1,843,200 B（≈1.76 MiB）** |
| `state.npz` | `qpos` / `geom_xpos` / `geom_xmat` / `light_xpos` / `light_xdir` | **`(3,8)`** / `(3,7,3)` / `(3,7,9)` / `(3,3,3)` / `(3,3,3)` float32 | ≈ **1.3 KB** |
| `capture.json` | — | recipe + environment + 两跑 sha + `two_captures_bitwise_identical` + `legacy_reference_untouched` | ≈ 8–20 KB（JSON） |

- npz 用 deterministic ZIP_STORED（固定时间戳），故字节可复现；每数组另有 ~百字节 `.npy` 头，总量以脚本输出 sha 为准。
- 预期 `capture.json` 关键字段：`nworld=3`、`hinge_deg=[0,20,40]`、`use_fast_math=false`、`use_shadows=true`、`two_captures_bitwise_identical=true`、`legacy_reference_untouched=true`、`legacy_reference_files=79`、`cameras=[cam-persp(320x240, mode=0)]`、`per_world_scope.covers="per-world geom poses"`、`per_world_scope.does_not_cover="per-world light poses"`。
- **口径说明**：`light_xpos/light_xdir` 一并落盘仅为完整性；本参考的覆盖声明只到 **per-world geom 位姿**（设备路径目前单 world 取灯）。

## 7. legacy 参考保护

脚本在采集前后各取一次 `fixtures/render-bvh` 的 sha256 清单（**排除整棵 `scene-a/perworld/`**，即本次输出目录），要求完全一致（运行时实测清单 **79 条** = 69 既有 + 10 个 SDF 采集参考文件）；任何变动 → 非零退出、不落盘。
已知口径：排除范围是整棵 `perworld/` 子树（QA B 复核 O3）；更细粒度的对拍留到实现片，本窗不宣称。

## 8. 用户双签栏（已签）

| 项 | 内容 |
| --- | --- |
| 窗口计划版本 | user via Chief **2026-10-30T23:10:52+08:00** approve；被批准的 plan 文本 sha256 `42791918e7b6aa2977752b5fe065590616f56d57f711479a1062ec719865935d`（当前文件因追加签字/例外/修正块已不同，见 receipt） |
| 用户决定 | ☑ 批准（scope 四项见 `materials/window-plan.txt`） |
| Chief 整理时间 | 2026-10-30T23:11+08:00 |
| 备注 | 采集只产参考、不做数值判定；窗口内只跑 (B) 一条 GPU 命令 |

## 9. authoring-only util<60 例外（已签）

- **例外**：user via Chief **2026-10-31T00:00+08:00**，`mode=authoring_only_exception_util_lt_60`，同一签字覆盖本窗与 SDF 采集窗；五要素与例外 monitor 命令（`-MaxUtil 60`、新 CSV `target/render-bvh/perworld-capture-window/monitor-pre-window-attempt2.csv`、PowerShell 5.1）见 `materials/window-plan.txt` 的 `EXCEPTION BLOCK`。
- **原失败留档**：本窗未跑常规 `util<40` monitor（Chief 指令：共享 SDF 窗口失败 + 用户签字后直接走例外路径）；常规计划文本原样保留，共享失败记录在 `docs/development-evidence/render-bvh/sdf-capture/monitor/monitor-pre-window-regular-failure.csv`。
- **例外生效证据口径**：从例外 CSV 独立重算 util≥40 仍 clean 的样本与连续段、`ge60=0`、`util_max<60`；全 <40 则记 `exception_effectively_not_load_bearing=true`，不得声称 60 门槛生效。

## 10. 窗口期修正（attempt2，2026-10-31）

attempt1 在**例外 monitor 60/60 通过后**、采集命令的**脚本场景自检阶段**非零退出，未到渲染/写盘：
`expected exactly one scalar hinge joint, found scalar=[0,1] hinge=[1]`。attempt1 记录原样保留（monitor CSV + 快照 + `capture-failure.txt` + 例外重算 JSON，见 `target/render-bvh/perworld-capture-window/attempt1/`），本条只更正错误假设，不改写历史结论。

| # | 缺陷（错误假设） | 修复 |
| --- | --- | --- |
| 1 | `scalar` 把 FREE(0) 也计入并要求「恰好 1 个 scalar」 | `scalar` 只取 SLIDE(2)/HINGE(3)，要求恰好 1 个 HINGE；FREE/BALL 允许且不计入 |
| 2 | 每 world `qpos` 以全零为基座 → free 四元数变 (0,0,0,0)、基座位姿丢失 | 基座改用模型默认 `qpos0`，再按 world 覆盖 hinge 地址 7；输出仍 float32 `(3,8)` |
| 3 | 材料假设 `nq=1`、`qpos (3,1)`、legacy 69 | 实测 `nq=8`、`qpos (3,8)`、legacy **79**（69 + 10 个 SDF 参考文件） |
| 4 | dry-run 只查环境与计划，挡不住材料假设与真实场景不符 | dry-run 增加静态模型自检（见 §2 末段），开窗前即暴露不一致 |

- 先例：R02-S2 窗口期修复 `b3a47aca`（走 authoring 卡 addendum + `Normify-Fixes`）。
- **未变**：配方、场景、相机选择、采集判据、两跑位级规则、legacy 对拍、产出目标、用户已签窗口范围。
- attempt2 流程：新基线 → 新 floor → 例外 monitor（新 CSV、`-MaxUtil 60`、PowerShell 5.1）→ 同一条 GPU 命令 → 产物/sha 校验 → 证据落 `perworld-capture-run/`。

## 11. 证据清单

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文 |
| `receipt.json` | 材料 id、脚本身份、场景/位形、文件哈希、双签与例外状态、修正记录 |
| `manifest.txt` | 本目录除自身外全部文件 sha256/bytes（QA 观察：补全材料完整性，2026-10-31） |
| `materials/window-plan.txt` | 窗口计划一页（ASCII）：命令逐字、基线/floor、时限、失败策略、双签与例外块、CORRECTION BLOCK |
| `materials/expected-products.json` | 预期产物/判据、dry-run 自检、例外命令、修正记录 |
