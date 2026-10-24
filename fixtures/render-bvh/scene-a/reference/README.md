# scene-a 冻结渲染参考（S0b，MJWarp 3.15 / MuJoCo 3.15）

本目录是 `fixtures/render-bvh/scene-a/` 的**冻结参考输出**，由离线 authoring 管线
（`scripts/authoring/render-bvh/capture_reference.py`）在 pinned 环境里生成，任务
`2026-10-24-render-bvh-s0b`。用途：S1+ 产品侧（MuJoCo 3.12 / MJWarp 等价实现）的渲染与
射线验收对照。**本目录不是产品代码，产品构建/测试/门禁不会读取或运行这里的任何 Python。**

## 1. 生成环境（不符则捕获脚本直接失败退出）

| 项 | 值 |
| --- | --- |
| MuJoCo / MJWarp | 3.15.0 / 3.15.0（pinned venv `target/mjwarp-parallel/venv`） |
| Python / NumPy / Warp | 3.12.13 / 2.5.4 / 1.15.0 |
| 冻结上游 | `71da24d956378a87a703b6e1442b13aec0c4ac29`（clean） |
| GPU / 驱动 / arch | RTX 4070 Ti SUPER / 596.36 / sm_89（CUDA toolkit 12.9） |
| 场景 | `scene.xml` sha256 `a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289`（LF 归一化，S0a/S0b 同一算法） |
| 编译模型 | `scene-315.mjb` sha256 `8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4`（101,320 B） |
| 捕获脚本 | `capture_reference.py` sha256 `6cb7f016e0be1dcae064925a56f6b4186e1031ea6c57df18430bdc02d04ee952`（484 行；自哈希按 LF 归一化字节计算）；射线构造 `capture_reference_rays.py` sha256 `fafd6d91f680b2394048980cef7b1a2e0eb0395e9b0f7baea6fb8f08fc92c5bd`（167 行） |
| 包装脚本 | `run_s0b.ps1` sha256 `9e72b229bd355d9e1cad744c3677ba571d94f9318acb8f0f9d65053358f66476` |
| 复现 | `target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass -File scripts\authoring\render-bvh\run_s0b.ps1` |

## 2. 文件与语义

| 文件 | 内容 | 形状/类型 | sha256 |
| --- | --- | --- | --- |
| `rgb.npz` | `rgb_cam0`、`rgb_cam1`：`get_rgb` 解包后的线性 RGB | (240,320,3)/(180,240,3) float32，[0,1] | `9379f5d3b39e2d5d0ce8ef21a28b365e3608d20f6a992c39dff2d5df3ef90c0c` |
| `depth.npz` | `depth_cam0/1`：`rc.depth_data` 原始平面深度（米） | (240,320)/(180,240) float32；**0 = 无命中**，未归一化、未 clamp | `0fa5582170a2b2a117e46868930fa2b4270dd993b0f5d86dc603981e0a2fd7d3` |
| `seg.npz` | `seg_cam0/1`：`rc.seg_data` 原始 `(object_id, object_type)` 对 | (240,320,2)/(180,240,2) int32；背景 `(-1,-1)` | `2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97` |
| `rays.npz` | 384 条固定射线：`ray_origin`/`ray_direction`（世界系、方向单位化）、`ray_category`（0..3）、`ray_geomid`、`ray_dist`（米）、`ray_normal`（世界系） | (384,3)/(384,) | `83146ae50946537401be817a44bfe3af58320df475dabc49206aa47eec9d50ab` |
| `context.json` | `create_render_context` 28 参数解析值、resolved 输出偏移、两条相机路径、四类射线统计（含 task_id） | JSON | `e08cce0ad1783b2cadd533115ee0ea0bf1defa8db7ad8ed5a5e8ed1c2c62f2df` |
| `../provenance.json` | 环境/版本/上游 revision/哈希/命令/种子/两次捕获一致性/GPU 显存（含 task_id） | JSON | 每次运行写入（显存字段随运行变化，不作为确定性输入） |

## 3. 两条相机路径（`context.json` 的 `camera_paths`）

| cam | 路径 | 分辨率 | sensorsize | intrinsic (fx,fy,cx,cy) | fovy | 说明 |
| --- | --- | --- | --- | --- | --- | --- |
| 0 `cam-persp` | **intrinsic 分支**（`sensorsize` + `focalpixel="160 160"`） | 320×240 | (0.036, 0.024) | (0.018, 0.016, 0, 0) | 73.73979461915735° | 走 `compute_ray` 内参分支；S0a 缺陷修正后有效 |
| 1 `cam-track` | **fovy 分支**（无 sensorsize） | 240×180 | (0, 0) | (0.01, 0.01, 0, 0)（默认值，此路径不使用） | 40° | 走 `compute_ray` fovy 分支 |

相机像素类射线每条相机取 8×8 网格（共 64 条/相机，`camera_pixel_rays_per_camera=[64,64]`），
覆盖两条路径；参考图像为 `PARAMS_28` 中 `render_rgb/depth/seg=True`、`use_precomputed_rays=True`
（相机像素射线即渲染器预计算射线）。

## 4. 四类射线（`rays.npz` + `context.json` 的 `ray_stats`）

| 类别 | code | 条数 | 命中 | 构造/校验 |
| --- | ---: | ---: | ---: | --- |
| 相机像素 `camera_pixels` | 0 | 128（2×64） | 74（cam-persp 50 / cam-track 24） | 取 `rc.ray` 预计算方向的相机系→世界系；要求多数命中 |
| 种子随机 `seeded_random` | 1 | 128 | 20 | `SEED=20261024`，均匀起点 + 随机方向；无命中期望 |
| 掠射 `grazing` | 2 | 64 | 64 | 地板平面上方 6 mm、水平走向 1.2 m 的近切向入射；要求全部命中且命中 geom 必须是 `floor`、`max|n·d|≤0.05`（实测 0.005） |
| 无命中 `no_hit` | 3 | 64 | 0 | 从场景上方朝外；要求全部不命中 |

## 5. 确定性

- 同一进程连续捕获两次：四份 npz 与 `context.json` 的 sha256 全部一致（脚本内强校验）。
- 本次另做了两次独立进程运行，四份 npz 与 `context.json` sha256 仍一致（跨进程可复现）。
- NPZ 写入器固定 zip 时间戳（1980-01-01）、按 key 排序、不含运行时间等易变字段；
  `context.json` 不含渲染耗时（耗时只写 `provenance.json`）。

## 6. 边界（不宣称）

- 这是 **MJWarp 3.15 参考**，不是 MuJoCo 原生 GL 渲染参考，也不宣称两者一致。
- 只覆盖 `scene-a` 与 `PARAMS_28` 的固定配置：未覆盖 shadow/skybox/splat/flex、
  未覆盖多世界（`nworld=1`）、未覆盖正交相机、未覆盖 Linux。
- 射线集是**抽样**（相机像素 8×8 网格、随机/掠射/无命中各固定条数），不是全像素射线覆盖。
- 产品侧是否与该参考一致属于 S1+ 验收，本目录不构成产品验收结论。
