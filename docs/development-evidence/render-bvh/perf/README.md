# 渲染/BVH perf：独占 GPU 窗口的相对性能基准

任务：`2026-10-25-render-bvh-perf`（渲染/BVH perf，依赖 S8）
冻结上游：`71da24d956378a87a703b6e1442b13aec0c4ac29`（MJWarp 3.15）
GPU：NVIDIA GeForce RTX 4070 Ti SUPER / 驱动 596.36 / SM89（Windows 11）

## 1. 结论（只给单场景单机相对基准）

- **单帧（spp=1，120,000 射线）**：frame 口径 min/median/p95 = **11.373 / 14.030 / 16.622 ms**
  （measurement A）；独立重复测量 **11.423 / 14.389 / 17.039 ms**（B）。
- **AA（spp=2 → nsamples=4，480,000 射线）**：**34.304 / 36.174 / 37.243 ms**（A）；
  重复测量 **34.847 / 36.423 / 38.091 ms**（B）。
- **相对比值**：AA/单帧 median = **2.578**（A）/ **2.531**（B）；每 120,000 射线：
  单帧 **14.030 / 14.389 ms**、AA **9.043 / 9.106 ms**。
- **两遍一致性**：单帧 median 差 **2.6%**（14.030 ↔ 14.389 ms）、AA 差 **0.7%**（36.174 ↔ 36.423 ms）、
  比值差 **1.8%**（2.578 ↔ 2.531）；两次独立窗口的原始序列都保留，不择一、不平均成单值。
- **显存**（`device_memory` 分配口径）：渲染峰值相对进程起始 free 差 **56,623,104 B（54 MiB，含 context 2 MiB）**；
  每帧后 free 序列 10 次全部为 `15,754,854,400 B`（**无逐帧增长**）；destroy 后 context 2 MiB 回池。
  外部 `nvidia-smi` 采样（329 点、100 ms 间隔）：桌面基线 4,278 MiB → 窗口峰值 **4,556 MiB**（+278 MiB，
  含 CUDA 上下文/模块缓存，属进程口径），窗口结束回 4,278 MiB。
- **窗口**：A `2026-10-25T14:06:49.13 → 14:07:00.08`；B（重复）`14:07:00.08 → 14:07:03.85`；
  开始/结束 `nvidia-smi --query-compute-apps` 均无其它 CUDA 计算进程。
- **不回归**：S7b1 6/6（depth/seg 120,000/120,000 位等 0 ULP）、S7b2 3/3（RGB 同配方 0 差异）、
  S7b3 6/6（flag/AA 语义、PTX 197,583 B / 27,536 B 不变）。
- **口径声明**：以上是**单场景（scene-a）单机（本机 RTX 4070 Ti SUPER）相对基准**；
  **不宣称固定性能指标**，不与上游或其它硬件比较；不改渲染语义与已验收数值。

## 2. 测量口径

| 项 | 定义 |
| --- | --- |
| warmup | 5 次/配置（不计入统计，序列中不出现） |
| 测量迭代 | 30 次/配置 |
| `launch_ms` | 仅 `MegakernelRenderer::render(...)` 调用耗时（辅助口径，可能不含完整 GPU 执行） |
| `frame_ms` | 发射 + 输出读回同步（`outputs().read(Rgb)` 作为 CUDA 同步点；**主指标**） |
| 统计 | min / median / **p95**（nearest-rank `index = round((n-1)*fraction)`，四舍五入） |
| 归一化 | 单帧 = 120,000 射线；AA = 480,000 射线 → 每 120,000 射线 ms = frame median / 4 |
| 位姿 | 产品 camlight 输出（与 S8 交付配置一致；数值 0 位差已在 S8/QA 验收，性能与位姿来源无关） |
| 射线 | S5 预计算缓冲（`precomputed_rays` / `aa_rays`） |
| context | 默认 28 参数、`nworld=1`、rgb/depth/seg 全开；单帧与 AA 各建一次，测量后显式 destroy |
| 计时环境 | 独占 GPU 窗口（无其它 CUDA 计算进程）；Windows 桌面仍有图形应用波动 |
| 构建 | `cargo test` 默认 debug 测试构建（未测 release） |

**原始毫秒序列**：`raw-timings.json`（7,739 B，sha256 `731704df427a490ce77637062ba5a1d9763cf160b8a44373c609c787916a2498`）。
结构：`measurement_a`（来自 `gpu-run.log`）与 `measurement_b`（来自 `repeat-run.log`）各含
`modes[].launch_ms`/`frame_ms`（每次迭代原始序列，30 条）与 `ratios`；**measurement 级 `official_summary`
（与 `modes` 平级、不嵌在 `modes[]` 内）**取自同一次运行的测试输出 `s8perf[stat]`，由测试内 percentile
派生、与原始序列同源；另有 `memory` 显存 free 序列。原始序列供 QA 独立复算，摘要不另算。

## 3. 数字表（frame 口径，主指标）

| 配置 | 测量 | 射线/帧 | min (ms) | median (ms) | p95 (ms) |
| --- | --- | --- | --- | --- | --- |
| 单帧 spp=1 | A（`gpu-run.log`） | 120,000 | 11.373 | 14.030 | 16.622 |
| 单帧 spp=1 | B（`repeat-run.log`） | 120,000 | 11.423 | 14.389 | 17.039 |
| AA spp=2（nsamples=4） | A | 480,000 | 34.304 | 36.174 | 37.243 |
| AA spp=2（nsamples=4） | B | 480,000 | 34.847 | 36.423 | 38.091 |

`launch_ms`（辅助）两遍：单帧 median 13.604（A）/ 14.065（B）ms；AA median 35.826（A）/ 36.075（B）ms
（原始序列见 `raw-timings.json`）。

相对比值（同源摘要）：

| 测量 | AA/单帧 median | 单帧每 120,000 射线 ms | AA 每 120,000 射线 ms |
| --- | --- | --- | --- |
| A | 2.578 | 14.030 | 9.043 |
| B | 2.531 | 14.389 | 9.106 |

## 4. 显存

| 项 | 值 |
| --- | --- |
| `device_memory` 起始 free | 15,811,477,504 B |
| context（单帧）create 后 free | 15,809,380,352 B（context = 2,097,152 B = 2 MiB） |
| 渲染内核编译后 free | 15,807,283,200 B |
| 单帧渲染期间最小 free | 15,754,854,400 B（峰值差 **56,623,104 B = 54 MiB**，含 context + 帧工作集） |
| 每帧后 free 序列（10 帧） | 10 次全部 15,754,854,400 B（无逐帧增长） |
| AA context create 后 free | 15,756,951,552 B |
| AA 渲染期间最小 free | 15,754,854,400 B（峰值差同 54 MiB） |
| destroy 后 free | 15,756,951,552 B（context 2 MiB 回池）；全部 destroy 后 15,759,048,704 B |
| 相对进程起始差值 | 54,525,952 B = 52 MiB 属驱动代码缓存/池一次性保留（S6b 先例 32 MiB 级），**非泄漏** |
| `nvidia-smi` 采样（329 点） | min 4,246 / median 4,278 / **max 4,556 MiB**（桌面基线 4,278；窗口结束回 4,278） |

## 5. 独占窗口与设备登记

- 设备：`NVIDIA GeForce RTX 4070 Ti SUPER`，驱动 `596.36`，compute cap `8.9`（SM89）。
- 窗口起止：A `2026-10-25T14:06:49.1272506+08:00 → 14:07:00.0832398+08:00`；
  B `2026-10-25T14:07:00.0832398+08:00 → 14:07:03.8523882+08:00`。
- `nvidia-smi --query-compute-apps`：窗口开始/结束均无其它 CUDA 计算进程（列表仅桌面图形应用）。
- 外部采样：`nvidia-smi-samples.txt`（329 点、100 ms 间隔，sha256
  `7b45dc4b7825d97250831fd76cdbb64dcf1ed104a9d0269e4e28d322081808f4`）。
- 日志：`gpu-run.log`（窗口起止 + 设备 + 两个测试完整输出，sha256
  `3be587af67b0c4aaa6ae6a48b9574939e20139d59783119de99569453add30ea`）、
  `repeat-run.log`（独立重复测量，sha256 `e1da947c141cb40bbfe5cf3ee93f15855dba53a47b02bf7b91910466e4bb2e4b`）、
  `regression.log`（S7b1/S7b2/S7b3 回归，sha256 `7b0d7abc2f6e7ee47945281a13c5dceebc039584a9792cca618f8bef14148a5b`）。

## 6. 未满足项（如实记录）

- 只测 scene-a 一个场景、单机（RTX 4070 Ti SUPER）；不可跨硬件/跨平台比较，不给固定性能指标。
- 未与上游 mujoco_warp 做性能比较（口径/同步方式/编译配方不同，不构成可比基准）。
- 未测 release 构建（本证据为默认 debug 测试构建；release 下数字预计不同，未采样）。
- 显存只有 `device_memory` 点采样与 100 ms 外部采样，亚 100 ms 峰值可能漏采。
- 未测 `nworld>1`、更高 spp、阴影/天空盒等未实现路径（这些路径本身写前拒绝）。
- 无固定 CI 基准：本证据是单次窗口的相对数字，后续复测须按同一口径（warmup/迭代/构建/环境）才有可比性。

## 7. 复现

```powershell
$env:CUDA_PATH=(Resolve-Path 'target\toolchains\cuda-12.8.1').Path; $env:PATH="$env:CUDA_PATH\bin;$env:PATH"
cargo test --locked --features cuda-probe --test render_perf -- --ignored --test-threads=1 --nocapture
# 回归
cargo test --locked --features cuda-probe --test render_kernel_depth_seg -- --ignored --test-threads=1 --nocapture
cargo test --locked --features cuda-probe --test render_kernel_rgb -- --ignored --test-threads=1 --nocapture
cargo test --locked --features cuda-probe --test render_kernel_flags -- --ignored --test-threads=1 --nocapture
```

`tests/render_perf.rs`（340 行，sha256 `96078c108ccd6f9333eddfe2aa4add5b5d893e2c255b26154fd9725462182010`）：
`relative_frame_benchmark_with_aa`（计时 + 原始序列）与 `device_memory_working_set`（显存工作集）。
