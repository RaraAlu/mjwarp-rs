# R09-S3a-capture-exact 阴影参考同配方捕获窗

任务：`2026-10-28-render-bvh-r09-s3a-capture-exact`（P8 渲染与 BVH；p59；owner Backend）
判据树 HEAD（签名时）：`61b82f9a`（R09-S3b 作者阶段提交后）
性质：**authoring-only 捕获**（pinned upstream 环境，不进产品路径）；本片只产出**同配方**（`use_fast_math=False`）参考，**不产任何产品数值结论**（产品数值验收在 R09-S3b 验证窗）
目标：补采与产品内核（精确 f32）同配方的 shadow 参考；原 `capture/`（fast-math 组）**一字不动**

## 1. 签名集（签名后零编辑）

| 项 | 路径 | 字节 | sha256 |
| --- | --- | --- | --- |
| 捕获脚本 | `scripts/capture_render_shadow.py` | 19 648 | `7c04809c6558eb8207d35c03c7b8777985b36d733eb711020910154bc6c18415`（477 行） |
| 场景 | `fixtures/render-bvh/scene-shadow/scene.xml` | 4 757 | `42f34ae4496042a0d64576bee18e1fc3cf30fdadac3ad374989b06d1af12060c` |
| guard | `scripts/gpu-window-guard.ps1` | 19 370 | `17b2a14e582542719eb52f562b388d34e68afbe4faef7795ddae9b29d60a1c4a` |
| sampler | `scripts/gpu-guard-sampler-loop.ps1` | 1 191 | `53194ce7d609d1cb2ecb9e1cb96804513ae4e6a04673d90a81bed48619633f1d` |
| pinned python | `target/mjwarp-parallel/venv/Scripts/python.exe` | 47 104 | `feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77` |

脚本 sha 变更历史：本卡新脚本 `7c04809c…` / 19 648 B / 477 行（加 `--use-fast-math`，签名后零编辑）；S3a 证据
（`docs/development-evidence/render-bvh/r09-s3a-shadow-capture/`）绑定 `b6cc2c27` 树的旧脚本 sha
`8453318d…` / 18 828 B / 461 行，**保留为历史**，不随本卡修改。

两条命令（同窗、同一 `MJWARP_CAPTURE_AT=2026-10-28T19:44:02+08:00`）：
```
python.exe -B scripts/capture_render_shadow.py --use-shadows true  --use-fast-math false --out-dir fixtures/render-bvh/scene-shadow/capture-exact/shadow-on
python.exe -B scripts/capture_render_shadow.py --use-shadows false --use-fast-math false --out-dir fixtures/render-bvh/scene-shadow/capture-exact/shadow-off
```

## 2. 轮次与采样（attempt=1 = 唯一有效轮）

- 监测：`2026-10-28T19:39:01.849+08:00 → 19:44:01.967`，**60 样本**（5 s 间隔，`logs/monitor.csv`），**连续 60=5 分钟**满足 `util<40 且 free≥10240`（required=60、clean_ready=True）后弹窗；监测期 `util 2–4`、`free_min=13479 MiB`
- 窗口：`window_open 19:44:02.872+08:00` → `window_close 19:44:07.037+08:00`，**body ≈4.165 s**
- baseline（3 点）：`2555,13507,2,32 / 2555,13507,4,32 / 2555,13507,4,32`（used/free/util/temp）
- `baseline_free_mib = 13507` → **floor = min(10240, baseline_free−3072) = 10240 MiB**
- 捕获 wall：shadow-on **2 273 ms**、shadow-off **1 755 ms**（两组均 exit 0；warp 内核缓存已热，远快于 S3a 首采）
- post（3 点）：`2555,13507,1,33 / 2555,13507,1,33 / 2555,13507,1,33` → used/free 零漂移、**delta 0 MiB**
- guard：`logs/guard.log` —— baseline `2555/13507/util 1/contexts 32`、preconditions 通过、`aborted=False`
- **零 Rust 编译**（窗内只跑 pinned python ×2 + nvidia-smi + guard）
- `residual_processes=4`：StartTime 为 `10-27 10:33:08`（2 个：Supermimic `.venv` python + uv cpython3.12）与 `10-28 17:20:17`（2 个：freecad-mcp python + uv cpython3.14）→ **外部 IDE/工具进程，非本次捕获**（如实记录，未处置）

## 3. 产出（8 文件 / 5 780 687 B）

| 组 | 文件 | bytes | sha256 |
| --- | --- | --- | --- |
| shadow-on | `capture.json` | 8 898 | `00f07b862de50944d833cb5ab21d0b9187e0f67d501ca1a71f95896a863606ad` |
| shadow-on | `depth.npz` | 480 486 | `28c1ea4bdd4533a3d4811f390051a9e0dfa75ccab2549d7b32b8b426ccc9ba14` |
| shadow-on | `rgb.npz` | 1 440 478 | `1ae5b54b808371e1df2bf7db2da1ba28c7bd78a802dac2d89d3e625155012c1b` |
| shadow-on | `seg.npz` | 960 478 | `2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97` |
| shadow-off | `capture.json` | 8 905 | `6970a49a28f0565c8a03fa00a6e540c5aa4f7b648b486b077ae7a8568ccfd598` |
| shadow-off | `depth.npz` | 480 486 | `28c1ea4bdd4533a3d4811f390051a9e0dfa75ccab2549d7b32b8b426ccc9ba14` |
| shadow-off | `rgb.npz` | 1 440 478 | `1090378b3c11bf87358adb838f4d7cce982acb40675cbadccf4ae3b06f2d7148` |
| shadow-off | `seg.npz` | 960 478 | `2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97` |

- **两组各自 A/B 两次独立捕获逐字节一致**：`two_captures_bitwise_identical = true`（两组 capture.json 均记录）
- 两组 `capture.json` 均记录：`parameters.use_fast_math=false`、`context.resolved.use_fast_math=false`、`use_shadows` true/false、五项 pinned 版本（python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4）、`MJWARP_CAPTURE_AT=2026-10-28T19:44:02+08:00`、`legacy_reference_untouched=true`、`legacy_reference_files=38`
- 两组 depth/seg 逐字节相同（阴影只影响光照）；rgb 两组不同（阴影可见）

## 4. 交叉校验（窗口闭合后 CPU 只读；三条硬条件全部通过）

`logs/cross-check.json`（pinned python，只读 npz + 既有 JSON）：

| 校验 | cam0 | cam1 |
| --- | --- | --- |
| `capture-exact/shadow-off/seg` vs 既有参考 seg **逐像素全等** | 0 mismatch | 0 mismatch |
| `capture-exact/shadow-off/depth` vs `s7b/depth_seg_fast_math_false.json` **f32 位型逐位相同** | 0 mismatch / max_ulp 0 | 0 mismatch / max_ulp 0 |
| `capture-exact/shadow-off/rgb` vs `s7b/rgb_fast_math_false.json` **packed u32 位型逐位相同** | 0 mismatch | 0 mismatch |

**跨配方诊断**（与 `capture/`（fast-math 组）对比；**只作记录、不作判据**）：

| 组/指标 | cam0 | cam1 |
| --- | --- | --- |
| depth 差异像素 / max ULP / 超 4 ULP | 19 074 / 2 / **0** | 4 064 / 4 / **0** |
| rgb max byte diff / 超 1 LSB 像素 / 差异像素 | 1 / **0** / 2 | 0 / **0** / 0 |

结论（仅供记录）：同配方采集与既有 S7b1 同配方参考**位级一致**（这使 S3b 的同配方判据可用），与 fast-math 组的差异全部落在既有容差内（depth ≤4 ULP、rgb ≤1 LSB）。

## 5. 旧参考未动

- 范围：`fixtures/render-bvh/` 除 `scene-shadow/` 外 **38 文件 / 25 320 515 B**；跑前跑后 `aggregate=983617f16b534f9cda743ff2b84bfed08e0154e88c369419b885bacb841cc260` **相同**、count/bytes 不变、`legacy_untouched=True`（逐文件清单由窗口脚本采集；口径同 S3a：`sha256 bytes path` 行拼接）
- `capture/`（fast-math 组）8 文件一字未动（sha 仍为 `3867f6e1…`/`0fa55821…`/`65596761…`/`2153fa72…` 与 `6c1e9d74…`/`0fa55821…`/`9379f5d3…`/`2153fa72…`）
- S3a 证据（`docs/development-evidence/render-bvh/r09-s3a-shadow-capture/`，绑 `b6cc2c27` 树）记录的旧脚本 sha `8453318d…` / 18 828 B **保留为历史**，不改

## 6. 环境与配方

- pinned python：`target/mjwarp-parallel/venv/Scripts/python.exe`（47 104 B / `feeec361…`）
- 五版本：python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4
- GPU：RTX 4070 Ti SUPER / arch 89 / driver 596.36；warp 自带 NVRTC CUDA Toolkit 12.9
- upstream：`71da24d956378a87a703b6e1442b13aec0c4ac29` clean
- 配方：**`use_fast_math=false`（产品同配方）**；场景/相机/seed/布局与 S3a 完全一致（唯一参数变化）

## 7. 限制与口径（必须随证据保留）

- 本片只产出 authoring 参考；**不产产品侧数值结论**，数值验收在 R09-S3b 验证窗
- 跨配方诊断数字只作记录，**不作判据、不用于放宽/收紧任何容差**
- 不写任何窗口结论、覆盖声明、精度结论或等价结论（含上游与双平台）
- 旧参考与 `capture/` 只读；判据/容差/夹具零改动
- S3b 测试切换参考路径后：`capture-exact/` 承担同配方硬判据，`capture/`（fast-math）保留为跨配方诊断比较、不参与断言
