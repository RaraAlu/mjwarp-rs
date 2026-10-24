# 渲染/BVH S0b：3.15 冻结参考捕获（GPU 独占）

任务：`2026-10-24-render-bvh-s0b`。日期：2026-10-24。执行：Backend。
依据：`docs/development-evidence/render-bvh-precheck/plan.json` 的 S0b 切片、用户 2026-10-24
裁决 A 后放行的 GPU 窗口；S0a 已 `verified`（含 2026-10-24 cam-persp 内参缺陷修正，提交
`a8a608c`）。**离线 authoring，GPU 独占，产品测试不启动 Python。**

## 1. 结论（先给）

1. **捕获成功并通过全部验收项**：连续两次捕获的 `rgb/depth/seg/rays.npz` 与 `context.json`
   的 sha256 全部一致；另做两次独立进程运行，四份 npz 与 `context.json` sha256 仍一致。
2. **四类射线齐全**：相机像素 128（命中 74）、种子随机 128（命中 20）、掠射 64（命中 64，
   命中 geom 全部为 `floor`，`max|n·d|=0.005`）、无命中 64（命中 0）。
3. **28/28 个 `create_render_context` 参数全解析**并写入 `context.json`
   （`parameter_count=28`，对照 `docs/module-contracts.md` L653–672），输出偏移
   `rgb_adr=[0,76800]`、`depth_adr=[0,76800]`、`seg_adr=[0,76800]`、`total_rays=120000`。
4. **两条相机路径同时覆盖**（用户 2026-10-24 追加验收）：
   `cam-persp` 走 intrinsic 分支（`sensorsize`+`focalpixel`，intrinsic=(0.018,0.016,0,0)，
   fovy=73.73979461915735°）；`cam-track` 走 fovy 分支（fovy=40°，无 sensorsize）。
   `context.json.camera_paths` 记录两者解析值，射线集每条相机 64 条相机像素射线
   （`camera_pixel_rays_per_camera=[64,64]`）。
5. **provenance 完整且不读产品树**：版本/上游 revision 与 clean 状态/MJB 与 XML 哈希/
   脚本哈希/命令/种子/重复哈希/GPU 显存，脚本只读 `fixtures/render-bvh/` 与 pinned 环境。
6. **环境不符大声失败**：版本、venv 前缀、上游 revision/dirty、MJB/XML 哈希、GPU 名称/arch/
   驱动任一不符即 `SystemExit`，无静默降级（本次运行前已实测旧哈希会失败退出）。
7. **GPU 已释放**：运行前后无其他计算任务；进程退出后 `nvidia-smi` = 3276 MiB，
   与桌面基线（3264–3346 MiB 波动）一致。

## 2. 验收对照

| plan/用户验收项 | 结果 | 证据位置 |
| --- | --- | --- |
| ① 连续两次捕获每个输出 sha256 一致 | 通过 | §3 表；`provenance.json.capture.identical_sha256=true`；跨进程再复核一致 |
| ② 射线集覆盖相机像素/种子随机/掠射/无命中四类 | 通过 | §4；`context.json.outputs.ray_stats`；`rays.npz` |
| ③ context receipt 含全部 28 参数解析值与输出偏移 | 通过 | `context.json.parameter_count=28`、`create_render_context_parameters`、`resolved` |
| ④ provenance 完整且脚本不读产品树 | 通过 | `scene-a/provenance.json`；脚本仅读 fixtures + pinned venv |
| ⑤ 环境不符显式失败不静默降级 | 通过 | `check_environment()` 硬校验；失败时 `SystemExit` |
| ⑥（追加）两条相机路径覆盖 + 各自 intrinsic/fovy 解析值 | 通过 | `context.json.camera_paths`；`camera_pixel_rays_per_camera` |
| ⑦（追加）证据绑定新 scene.xml sha256，S0a 与 S0b 同哈希 | 通过 | 两边均 `a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289`（LF 归一化算法） |

## 3. 输出与哈希

| 文件 | 字节 | sha256 | 形状/说明 |
| --- | ---: | --- | --- |
| `reference/rgb.npz` | 1,440,478 | `9379f5d3b39e2d5d0ce8ef21a28b365e3608d20f6a992c39dff2d5df3ef90c0c` | (240,320,3)+(180,240,3) f32 |
| `reference/depth.npz` | 480,486 | `0fa5582170a2b2a117e46868930fa2b4270dd993b0f5d86dc603981e0a2fd7d3` | (240,320)+(180,240) f32，原始米制平面深度，0=无命中 |
| `reference/seg.npz` | 960,478 | `2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97` | (240,320,2)+(180,240,2) int32 `(id,type)` |
| `reference/rays.npz` | 18,700 | `83146ae50946537401be817a44bfe3af58320df475dabc49206aa47eec9d50ab` | 384 条射线 + 命中/dist/normal |
| `reference/context.json` | 6,026 | `e08cce0ad1783b2cadd533115ee0ea0bf1defa8db7ad8ed5a5e8ed1c2c62f2df` | 28 参数 + resolved + camera_paths + ray_stats（含 task_id） |
| `scene-a/provenance.json` | 5,922 | `c13715a07953300aaf85233f2bd361a94a4863d97779fef6910d2ba25c30a90b` | 环境/哈希/命令/种子/显存（含 task_id；显存字段随运行变化） |

相机输出统计（同一份 npz 实测）：

| cam | 路径 | 分辨率 | rgb mean/std | depth 命中 | depth 范围 | seg 背景 |
| --- | --- | --- | --- | ---: | --- | ---: |
| 0 `cam-persp` | intrinsic | 320×240 | 0.758664 / 0.405097 | 59,805/76,800 | 0.599852–1.682580 m | 16,995/76,800（geom id 含 0） |
| 1 `cam-track` | fovy | 240×180 | 0.241771 / 0.375847 | 15,702/43,200 | 1.234752–3.703163 m | 27,498/43,200（geom id 0–5） |

## 4. 射线集

`rays.npz` 共 384 条（`SEED=20261024`，脚本 `capture_reference_rays.py`）：

| 类别 | code | 条数 | 命中 | 校验 |
| --- | ---: | ---: | ---: | --- |
| 相机像素 | 0 | 128 | 74（cam-persp 50 / cam-track 24） | 命中须过半（脚本断言） |
| 种子随机 | 1 | 128 | 20 | 无硬性命中要求 |
| 掠射 | 2 | 64 | 64 | 全部命中、geom 必须为 `floor`、`max|n·d|≤0.05`（实测 0.00499994） |
| 无命中 | 3 | 64 | 0 | 全部不命中（脚本断言） |

## 5. 环境自检与 GPU

| 项 | 本次值 |
| --- | --- |
| MuJoCo / MJWarp | 3.15.0 / 3.15.0（`target/mjwarp-parallel/venv`，Python 3.12.13，NumPy 2.5.4） |
| Warp | 1.15.0（CUDA toolkit 12.9 / driver 13.2 报告值） |
| 冻结上游 | `71da24d956378a87a703b6e1442b13aec0c4ac29`，clean |
| GPU / 驱动 / arch | RTX 4070 Ti SUPER / 596.36 / sm_89 |
| 输入夹具 | `scene.xml` `a9f1f0a3…`（LF 归一化）；`scene-315.mjb` `8953988f…`（101,320 B） |
| GPU 显存 | 脚本内采样 baseline=3,282 MiB → after=3,523 MiB（进程内 mempool 尚未释放）；进程退出后 `nvidia-smi`=3,276 MiB，回桌面基线 |
| 独占性 | 运行前后 `nvidia-smi --query-compute-apps` 无本任务以外的 CUDA 计算进程 |

## 6. 确定性

- 同一进程两次捕获：`provenance.capture.identical_sha256=true`，四份 npz 与 `context.json`
  sha256 一致；`context` 的两次构建也逐字节比较一致（脚本内断言）。
- 跨进程：本次先跑一次、换新进程再跑一次，四份 npz 与 `context.json` sha256 仍一致
  （`render_seconds` 仅在 provenance，不参与比较）。
- NPZ 固定 zip 时间戳与 key 顺序；`context.json`/`provenance.json` 以 UTF-8 + LF 写入。

## 7. 未满足项与边界

- 只覆盖 `scene-a` + 固定 `PARAMS_28`（`enabled_geom_groups=[0,1,2,3]` 以包含 mesh geom 的
  group 3；`use_shadows=False`、无 skybox/splat/flex）；不覆盖 shadow/skybox/splat/flex 路径。
- 只 Windows + RTX 4070 Ti SUPER + 驱动 596.36；未覆盖 Linux/其它 GPU。
- 射线集是抽样覆盖（相机像素 8×8 网格/相机），不是全像素射线；随机/掠射/无命中为固定 128/64/64 条。
- `depth` 为原始 `rc.depth_data`（未 clamp、未归一化），与 MJWarp `get_depth` 的 clamp 语义不同，
  产品对照时必须按同一原始语义比较。
- 本切片只证明参考可用与可复现，**不构成产品渲染验收**（S1+）；不宣称 MuJoCo 原生渲染与
  MJWarp 3.15 一致，不宣称 3.12 产品已对齐。

## 8. 复现

```powershell
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\authoring\render-bvh\run_s0b.ps1
```

单命令：nvidia-smi 前后采样 → pinned Python 运行 `capture_reference.py`（环境自检 → 加载
`scene-315.mjb` → 建 RenderContext → 渲染 rgb/depth/seg → 构造并投射四类射线 → 两次捕获比较 →
写 `reference/` 与 `scene-a/provenance.json`）→ 写 UTF-8/LF `run.log`。退出码 0 表示全部验收
通过；任何环境/哈希/确定性不符都会非 0 退出并在日志里说明。

## 9. 证据索引

- `run.log`（sha256 `b376543037b40ffe708d9a8eae6405f1e558b4ea9a8dbb029a54abcf0f0ce798`）：
  单次运行的 GPU 前后状态、命令、环境日志与捕获汇总。
- `receipt.json`：回执（task_id、验收、哈希、环境、未满足项）。
- `fixtures/render-bvh/scene-a/reference/`：`README.md` + 四份 npz + `context.json`。
- `fixtures/render-bvh/scene-a/provenance.json`：机器可读 provenance。
- `scripts/authoring/render-bvh/capture_reference.py`（484 行）、`capture_reference_rays.py`（167 行）、
  `run_s0b.ps1`（39 行）。
