# 渲染/BVH S0a + S0b 独立复核（快验口径）

- 任务：`2026-10-24-render-bvh-s0ab-independent-check`（owner QA）
- 日期：2026-10-24
- 复核对象：S0a 修正 `a8a608c`（`Normify-Fixes`，scene.xml 加 focalpixel、新增 camera_sanity.py、重钉夹具哈希；改判提交 `edaca66`）；S0b `f28c3a4`（捕获）+ `435fa5a`（行尾/自哈希口径修正）
- 复核 HEAD：`435fa5abca62222e6ed267df00969d25ff7a194d`（工作区干净，未改产品/夹具/原证据）
- 口径：快验独立复核。**本轮未跑全量 G01**（S0a/S0b 与 G01 无关；最近一次全量 G01 仍为 HEAD `959be10` 的 `6494fe8`，见 `docs/three-layer-acceptance.md` §4.14）。GPU 只用于两次秒级捕获复跑与一次环境反例。
- 结论：**7/7 项成立**，未发现阻断问题；4 条观察项/建议项见 §8，均不影响本次结论。

## 1. 哈希自洽（成立）

复核 26 个文件（12 个夹具/参考输出 + 6 个证据文件 + 8 个 authoring 脚本）：
工作树原始 sha256、LF 归一化（CRLF→LF）sha256、HEAD git blob sha256 三者与交付回执/provenance/README
记录逐一对上（`hash-comparison.json`，`all_match=true`，26/26）。本机 `core.autocrlf=true`，但当前检出
工作树与 blob 已是同一字节流（raw = LF = blob）；S0a/S0b 记录统一使用 LF 归一化口径，记录值与
HEAD blob 一致，不随检出配置漂移。

关键值（全部实测命中）：

| 文件 | 记录 sha256（前 12 位） | raw | LF | blob |
| --- | --- | --- | --- | --- |
| `scene.xml`（4,639 B） | `a9f1f0a35464` | ✓ | ✓ | ✓ |
| `scene-312.mjb`（101,328 B） | `c6f5f9f17118` | ✓ | ✓ | ✓ |
| `scene-315.mjb`（101,320 B） | `8953988f832c` | ✓ | ✓ | ✓ |
| `model-312.json` / `model-315.json` | `cdd490f0209d` / `6b055ca2b14e` | ✓ | ✓ | ✓ |
| `rgb.npz` / `depth.npz` / `seg.npz` / `rays.npz` | `9379f5d3…` / `0fa55821…` / `2153fa72…` / `83146ae5…` | ✓ | ✓ | ✓ |
| `reference/context.json` | `e08cce0ad178` | ✓ | ✓ | ✓ |
| `provenance.json` | `ade6178939` | ✓ | ✓ | ✓ |
| `capture_reference.py`（自哈希） | `6cb7f016e0be` | ✓ | ✓ | ✓ |

S0a 与 S0b 引用同一 `scene.xml` 哈希（`a9f1f0a3…`），S0b `provenance.fixtures.scene_xml` 与
`context.json`、receipt 全部一致；`camera_sanity.py`（`d289e09a…`）、`model_parity.py`
（`d19a82f8…`）、`model_parity_judge.py`（`347949e7…`）、`run_s0a.ps1`（`c5e5a2bd…`）等脚本哈希与回执一致。

## 2. 重跑捕获（成立）

用交付脚本原样（未修改 `capture_reference.py`）导入并把输出路径重定向到 `target/qa-render-bvh-s0ab/rerun/`，
pinned venv 跑 **两次独立进程**（不做任何仓库 fixtures 写入，runner 内有「输出路径必须在 temp 下」断言）：

```powershell
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-render-bvh-s0ab\run_capture_rerun.py --label run1
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-render-bvh-s0ab\run_capture_rerun.py --label run2
```

| 项 | 结果 |
| --- | --- |
| run1 / run2 退出码 | 0 / 0 |
| 耗时 | 2.6 s / 2.0 s（Warp 内核缓存已热，日志见 `s0b-rerun1.log`、`s0b-rerun2.log`） |
| 环境自检 | pinned prefix、Python 3.12.13 / Warp 1.15.0 / MuJoCo 3.15.0 / MJWarp 3.15.0 / NumPy 2.5.4、上游 `71da24d` clean、MJB/XML 哈希、GPU 名称/arch 89/驱动 596.36 全部通过 |
| 五份输出 vs 入库 reference | `rgb.npz`/`depth.npz`/`seg.npz`/`rays.npz`/`context.json` **逐文件 sha256 全部相等**（run1 与 run2 也互相相等） |
| 仓库 fixtures | `git status --porcelain -- fixtures/render-bvh` 为空（零改动） |

环境不符反例（成立）：把临时副本 `scene-315.mjb` 追加 1 字节后重跑，脚本在
`check_environment()` 直接 `SystemExit`，退出码 **1**、输出 `S0b environment/reference mismatch:
scene-315.mjb hash mismatch`（`s0b-env-negative.log`），无静默降级、未产生参考输出。

GPU：本次窗口只做上述两次捕获 + 一次反例；显存采样 before 3,410 MiB → after 3,396 MiB
（进程内 provenance 采样 3,517→3,664 / 3,439→3,672 MiB），结束后无本任务 CUDA 计算进程残留，
已回桌面基线（`gpu-session.json`、`gpu-before.log`、`gpu-after.log`）。

## 3. 两次确定性 / 三次独立进程（成立，但原证据表述需精确）

- **同进程两次捕获**：`capture_reference.py` 内做两次 `capture_once`，逐输出 sha256 比较，不一致即
  `fail()`；两次 `context_receipt` 逐字节比较，不一致即 `fail()`。本轮两次复跑均未触发断言，
  `capture.identical_sha256=true`。
- **三次独立进程**：原交付证据里**只有 1 份 `run.log`（一次进程运行）**，「3 次独立进程」是
  receipt/README 的文字声明；本次复核补了 **2 次独立进程**（run1/run2），加上原交付进程（其
  `run.log` 记录的四份 npz + context sha256 与入库文件相同）合计 **3 次独立进程，五份输出 sha256 全一致**。
  即：该声明现在可由证据复核成立，但原交付自身未逐次留日志——建议后续同类交付保留每次运行的
  单独日志/哈希清单（观察项，见 §8）。

## 4. 28/28 参数（成立）

`context.json` 的 `create_render_context_parameters` 共 **28** 项（`parameter_count=28`），与
`docs/module-contracts.md` L653–672 的创建参数表逐名对应（nworld、cam_res、cam_active、
render_rgb/depth/seg、use_textures、use_fast_math、use_shadows、use_ambient_lighting、
enabled_geom_groups、background_color、flex_render_smooth、use_precomputed_rays、render_skybox、
enable_backface_culling、shadow_light_fraction、samples_per_pixel、enable_vertex_normals、
enable_specular、enable_emission、enable_per_light_ambient、splat_position/rotation/scale/rgba/adr/group_id）。

输出偏移与派生值独立核对：`parameter_count=28`、`rgb_adr=[0,76800]`、`depth_adr=[0,76800]`、
`seg_adr=[0,76800]`、`total_rays=120000`（=320×240+240×180）、`nrender=2`、`bvh_ngeom=7`、
`znear=0.024999999441206455`、`zfar=125.0`。

两条相机路径同时在 receipt 与射线集里（成立）：

| cam | 分支 | sensorsize | intrinsic | fovy | 相机像素射线/命中 |
| --- | --- | --- | --- | --- | --- |
| 0 `cam-persp` | **intrinsic** | (0.036, 0.024) | (0.017999999225139618, 0.01600000075995922, 0, 0) | 73.73979461915735° | 64 / 50 |
| 1 `cam-track` | **fovy** | (0, 0) | (0.01, 0.01, 0, 0)（该路径不使用） | 40.0° | 64 / 24 |

与上游默认值有 4 处**显式配置差异**（非缺陷，均已在交付 README 记录）：`render_rgb/depth/seg`
显式 `True`（合同默认 `None` 表示读模型输出位）、`enabled_geom_groups=[0,1,2,3]`（默认 `[0,1,2]`，
为包含 mesh geom 的 group 3）；其余 24 项与合同默认一致。S1+ 产品对照必须用同一组参数。

## 5. 射线四类（成立）

独立加载 `rays.npz` 重算（不经 authoring 代码）并与 `context.json`/receipt 对表：

| 类别 | code | 条数 | 命中 | 说明 |
| --- | ---: | ---: | ---: | --- |
| 相机像素 | 0 | 128（2×64） | 74（cam0 50 / cam1 24） | 两条相机路径各 64 条 8×8 网格射线 |
| 种子随机 | 1 | 128 | 20 | `SEED=20261024` |
| 掠射 | 2 | 64 | 64 | 命中 geom 全部为 `floor`（geom id 0），`max|n·d|=0.004999937489628792 ≤ 0.05` |
| 无命中 | 3 | 64 | 0 | 全部 miss |

自洽性：384 条 = 128+128+64+64；命中合计 158 = `count(geomid≥0)` = 74+20+64+0；
相机像素 74 = 50+24；`camera_pixel_rays_per_camera=[64,64]`；相机射线起点每相机同一点、
方向均为单位向量。`ray_stats.max_abs_normal_dot_direction` 四类与 receipt 逐值一致
（0.9987175464630127 / 0.9860121607780457 / 0.004999937489628792 / 0.0）。
图像统计亦独立复算一致：cam0 rgb mean/std 0.758664/0.405097、depth 命中 59,805/76,800
（0.599852–1.682580 m）、背景 16,995；cam1 0.241771/0.375847、15,702/43,200
（1.234752–3.703163 m）、背景 27,498。

## 6. camera_sanity 不是空壳（成立）

`camera_sanity.py` 对两份模型的每个相机断言：intrinsic 各分量 finite、`fx>0 且 fy>0`、
`0 < cam_fovy < 180`（显式拒绝 180° 哨兵）。用临时改坏副本实测（仓库夹具未动，
`camera-sanity-negative.json`）：

| 用例 | 期望 | 实测 |
| --- | --- | --- |
| 未修改副本 | pass / exit 0 | exit 0，verdict=pass，cam0 intrinsic≈(0.018,0.016,0,0)、fovy 73.74 |
| intrinsic fx=fy=0 | fail / exit 1，点名焦距检查 | exit 1，`focal lengths must be positive` |
| cam_fovy[0]=180.00000500895632 哨兵 | fail / exit 1，点名哨兵 | exit 1，`outside (0, 180) sentinel range` |
| cam_fovy[0]=0 | fail / exit 1 | exit 1 |

`run_s0a.ps1` 串联成立：脚本依次执行 parity → `camera_sanity.py`，非 0 退出即
`throw "相机内参健全性检查失败…"`；本轮把 `-FixtureDir/-EvidenceDir` 重定向到 `target/` 下
**端到端重跑两次**（`s0a-rerun`、`s0a-rerun2`），均 `camera-sanity PASS: 4 camera records across 2
models`、`S0a parity 通过`、退出码 0，且两次产物完全一致（见 §7）。

## 7. 判据无放宽（成立）

独立重算（不经 `model_parity.py`/`model_parity_judge.py`，`parity-recheck.json`）：直接读取两份
`model-*.json` 的数组，按「整型/字节 exact、float32 比 f32 位型、float64 先比 f32 位型」重算：

- **133 字段 / 126 严格相等 / 7 差异**，差异清单与交付完全一致：`geom_size`、`geom_aabb`、
  `geom_rbound`、`geom_quat`、`mesh_quat`、`mesh_polynormal`（6 个 float64，**f32 位差均为 0**）、
  `geom_sameframe`（byte，差值 1→3）。
- `strict_elementwise_parity=fail` 在 `conclusion` 与 `version_difference_classification` 两处原样保留；
  double 层数字未删：`max_abs` 9.42e-14 / 1.27e-13 / 7.93e-14 / 9.55e-13 / 9.55e-13 / 2.56e-12，
  `max_ulp_f64` 6790 / 372448 / 2858 / 34396 / 34396 / 2161481。
- `geom_sameframe` 排除理由为「collision frame-reuse flag 不被渲染读取 + 3.15 枚举语义变化」，
  `render_relevant=false`，且公开列出 312/315 两版取值，未隐藏差异。
- 判据标记：`tolerance_changed=false`、`double_layer_numbers_removed=false`；`not_claimed` 明写不宣称
  两版逐位一致、不宣称 3.12/3.15 等价模型、不宣称严格逐元素 parity 通过。
- 容差路径核对：S0a 修正提交只改 `scene.xml`（focalpixel + 注释）、`model_parity.py` 的哈希算法、
  `run_s0a.ps1` 串联与新增 `camera_sanity.py`，**未改** `model_parity.py` 的逐字段比较/容差逻辑；
  S0b 修正提交只把自哈希改为 LF 归一化并刷新记录，捕获内容哈希未变（484 行脚本只改 1 行自哈希调用）。

S0a 端到端重编译两次（CPU，无 GPU）：`scene-312.mjb`、`scene-315.mjb` 两次重编译与入库文件
**逐字节一致**（`c6f5f9f1…` / `8953988f…`），`model-315.json` 一致；`model-312.json` 仅
`model_file` 路径字符串不同（输出被重定向到 `target/`），**arrays 全等**。parity 结论、7 差异清单、
camera-sanity 均与入库证据一致。

## 8. 观察项 / 建议项（不影响结论）

1. `a8a608c` 在为 `scene.xml` 补 focalpixel 的同时**去掉了文件原有的 UTF-8 BOM**（提交说明未提到）。
   对 MuJoCo XML 解析无影响，且哈希已重钉；建议后续同类修改在提交说明里显式列出字节级变化。
2. S0b 原交付的「3 次独立进程」只有 1 份 `run.log` 可查（其余为文字声明）；本次复核补 2 次独立进程后
  成立。建议以后把每次运行的输出哈希清单单独落盘。
3. `context.json` 的 4 处参数偏离上游默认值（render 输出显式 True、geom group 含 3）已在 README 记录；
   建议在 `context.json` 里也标注 `explicit_overrides` 便于机器校验。
4. `model-312.json` 的记录哈希依赖输出路径（`model_file` 字段）；若未来需要跨目录复算哈希，建议
   把该字段写成相对路径或不参与哈希（S0a 回执已用 LF 归一口径，未影响本次结论）。
5. `depth` 是原始 `rc.depth_data`（0=miss、未 clamp），与 MJWarp `get_depth` clamp 语义不同；
   交付 README 已注明，S1+ 对照必须同口径比较。

## 9. 未满足项（如实）

- 只覆盖 `scene-a` + 固定 `PARAMS_28`；未覆盖 shadow/skybox/splat/flex、多世界、正交相机。
- 射线集为抽样（相机像素 8×8 网格/相机），非全像素覆盖。
- 只 Windows + RTX 4070 Ti SUPER + 驱动 596.36；未覆盖 Linux/其它 GPU。
- 未做产品侧（S1+）对照验收；本复核不构成产品渲染验收。
- 不宣称 MuJoCo 原生 GL 渲染参考、不宣称 MuJoCo 3.12 与 3.15 渲染等价。
- S0a 未满足项沿用交付记录：3.15 mesh 资产 frame 计算的源码位置仅有假设、无 3.15 原生产品影响分析。

## 复现命令

```powershell
$py = 'target\mjwarp-parallel\venv\Scripts\python.exe'
& $py -B target\qa-render-bvh-s0ab\hash_compare.py
& $py -B target\qa-render-bvh-s0ab\verify_npz.py --out target\qa-render-bvh-s0ab\npz-verify.json
& $py -B target\qa-render-bvh-s0ab\parity_recheck.py --out target\qa-render-bvh-s0ab\parity-recheck.json
& $py -B target\qa-render-bvh-s0ab\camera_sanity_negative.py --out target\qa-render-bvh-s0ab\camera-sanity-negative.json
& $py -B target\qa-render-bvh-s0ab\run_capture_rerun.py --label run1
& $py -B target\qa-render-bvh-s0ab\run_capture_rerun.py --label run2
& $py -B target\qa-render-bvh-s0ab\run_capture_rerun.py --label env-negative --corrupt-mjb   # 预期 exit 1
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\authoring\render-bvh\run_s0a.ps1 `
  -FixtureDir target\qa-render-bvh-s0ab\s0a-rerun\fixtures -EvidenceDir target\qa-render-bvh-s0ab\s0a-rerun\evidence
& $py -B target\qa-render-bvh-s0ab\rerun_compare.py
```

QA 校验脚本在 `target/qa-render-bvh-s0ab/`（本机、gitignored）；检查输出已复制到本目录
（`hash-comparison.json`、`npz-verify.json`、`parity-recheck.json`、`camera-sanity-negative.json`、
`rerun-comparison.json`、`s0b-rerun1.log`、`s0b-rerun2.log`、`s0b-env-negative.log`、`gpu-session.json`）。

## 证据索引

- `hash-comparison.json`：26 个文件的 raw/LF/blob 哈希与记录值对照
- `npz-verify.json`：`rays.npz`/`rgb`/`depth`/`seg`/`context` 独立重算（四类射线、图像统计、28 参数、偏移）
- `parity-recheck.json`：133/126/7 独立重算与判据标记
- `camera-sanity-negative.json`：正向 + 3 个反例 + `run_s0a.ps1` 串联检查
- `rerun-comparison.json`：两次捕获复跑与两次 S0a 重编译的逐哈希/逐结论对照
- `gpu-session.json`、`gpu-before.log`、`gpu-after.log`：GPU 采样与释放
- `s0b-rerun1.log`、`s0b-rerun2.log`、`s0b-env-negative.log`：运行日志
