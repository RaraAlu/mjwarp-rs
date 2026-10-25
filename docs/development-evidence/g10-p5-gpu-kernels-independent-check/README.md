# G10@P5 G5 GPU 内核独立复核（QA 自算，设备 vs 同配方上游 f32 GPU）

- 任务：`2026-10-25-g10-p5-gpu-kernels-independent-check`
- 复核对象：G5 收口 `e00de497`（修复片 `01690e53`，父 `1f882361`）
- 结论：**G5 在交付声明面通过（阻断 0）**；附 2 项发现（CCD 溢出位、深穿透 EPA 姿态采样）+ 1 项观察（native 材料不能作容差依据）+ 未覆盖项（`multicontact` 设备版未实装等）。
- 独立性：本目录所有上游参考由 QA 自写脚本重采、设备侧由 QA 自写 nvcc launcher 驱动；**结论数字不引用交付 `result.json`**，只作事后对照。

## 1. 方法（独立实现，不复用交付 harness）

| 环节 | 自写件 | 说明 |
| --- | --- | --- |
| 上游参考 | `qa_prepare_capture.py` | 直接调 `mujoco_warp` 3.15.0 / warp 1.15.0（`device=cuda`，`cuda:0`）按同配方输入跑 `ccd`/`ccd_hfield`/`sdf` 与 `ccd()` witness，导出 f32 原始数组 + 本 case 输入 bins；每例**两次独立捕获** |
| 设备输出 | `probe/qa_probe.cu` + `qa_build_probe.ps1` | QA 自写 CUDA launcher，`#include` 产品 14 个 `.cu`（**不复制**交付 probe），按输入 bins 组 lane 状态并导出 `nacon/nccd/overflow/efc_address/dist/pos/frame/worldid/geom...`；nvcc 12.8.93，`-O3 --fmad=true -arch=sm_89`，构建 recipe 与源 sha256 见 `qa-gpu/build-recipe.txt` |
| 判据 | `qa_compare_gpu.py` | 自算 ULP/绝对差；接触行按 `(worldid, dist, pos, frame)` 排序后逐字段比较（raw 行序因 atomic nacon 不稳定，不作判据） |
| G6 材料 | `qa_backend_precision.py` | 同码跨后端（warp CPU vs warp CUDA f32）逐字段误差分布 + native f64 材料核对 |
| 窗口 | `qa_window_run.ps1`、`qa_gpu_rerun.ps1` | 前置基线 → phase1/2/3 → 收尾基线 + `query-compute-apps` + 残留进程；rerun 脚本带自动守卫（≥3072 MiB 或 >50% 设备即停） |

判据口径（Arch 裁决 2，QA 独立重算）：`count` 相等且 `worldid` 分组后相等；`dist` 按自身量级 ≤4 ULP（`ref==0` 用 4 ULP@1.0）；`pos`/`x1`/`x2` 用位置向量尺度 `max_i|ref_i|` 的 `4·ULP(scale)`；`frame`/`normal` 用 scale=1.0 的 4 ULP 绝对差（4.768e-7）；参数/整数位级相等。

环境：rustc/cargo 1.99.0；python 3.12.13 / mujoco 3.15.0 / warp 1.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4；GPU RTX 4070 Ti SUPER 16 GiB（sm_89，Driver 13.2）。上游侧 warp 自报 CUDA Toolkit 12.9，与本探针 nvcc 12.8.93 不同（两侧独立编译，不共享二进制）。

## 2. GPU 窗口记录（两轮，均独占）

前置说明：开工早期（19:05:45）曾落盘三份 **CPU dry-run** 的 `qa-upstream-gpu.json` / `qa-witness-gpu.json` / `qa-native-f64.json`（诊断用，未开 GPU 窗口就生成，已由 Arch 的任务 note 更正为「CPU dry-run，作诊断参考」）。本轮**已用真实 cuda 捕获整份覆盖**这三份文件：`qa-upstream-gpu.json` 现为 `device=cuda`、`warp_device=cuda:0`，`qa-witness-gpu.json` 同源；`qa-native-f64.json` 是 MuJoCo C（native f64）读数，其 CPU 属性是 native 语义本身、不是 dry-run 遗留。**dry-run = 诊断参考，正式数据 = 本窗口**。

### 2.1 主窗口 2026-10-25 19:08:49.351 → 19:09:17.785（wall 28.43 s）

- 开窗前：`used=4021 MiB / 16376`（19:08:49.335），`query-compute-apps` 34 条**全部为桌面/常驻应用**（explorer、Chrome、EdgeWebView2、NVIDIA Overlay、VS Code、Feishu、WeChat、QQ、DeepSeek Harness 等），无 `qa_probe`、无其它 CUDA 计算任务；其中 3 条为 `[Insufficient Permissions]` 无法命名（限制，见 §9）。
- 窗口内容（脚本不构建）：phase1 上游 f32 GPU 捕获 + 输入 bins（19:08:49.979→19:09:16.550，26.6 s，wall 主要是 **CPU 侧 warp JIT 编译**：`smooth` 3685.70 ms、`witness_kernel` 14731.47 ms 等，kernel 本身 ms 级）；phase2 设备探针 run1（19:09:16.568→19:09:17.177）；phase3 设备探针 run2（19:09:17.177→19:09:17.709）。10 个 case 各 2 次上游捕获 + witness 3×(2+24) + 强制 horizon 3 + 自造 2 例 + 21 个 case 目录探针两遍。
- 采样（52 点，0.5 s 间隔，`qa-gpu/window-memory.txt`）：
  - 起点 19:08:50.157 `used=4017 MiB`（相对开窗前基线 −4）
  - **峰值 19:09:15.716 `used=6318 MiB`，Δ+2297 MiB（+14.0% 设备，6318/16376=38.6%）**
  - 收尾 19:09:17.316 `used=5306 MiB`
- 关窗后：`used=4310 MiB`、util 1%、`residual_qa_probe_processes=0`、`query-compute-apps` 仍是同一批 34 条桌面应用（无新增计算进程）。
- 守卫与偏差（如实记录）：本轮按当时规则「峰值增量 <1 GiB」执行；峰值 2297 MiB 已超旧阈值，但峰值出现在 19:09:15.7、距窗口结束 2 s，且属 phase1 warp JIT/模块加载的工具链暂态（窗口跑后回落到 4039–4310 MiB、无 OOM、无残留）。已报 Arch，裁决：**接受本轮数据**；口径修正为「**核函数相位增量 <1 GiB；总增量暂态允许，但 ≥3072 MiB 或 >50% 设备即停**」。本证据写明：**本轮 +2297 MiB 触发旧阈值，按修正口径接受**。限制：主窗口脚本只采样、不自动中止，phase2/3 的核相位增量未单独采样（采样粒度 0.5 s，phase2/3 各约 0.6/0.5 s）。

### 2.2 确定性重跑窗口 19:15:40.839 → 19:15:42.075（wall 1.19 s）

- 开窗前基线 `used=3982 MiB`；`query-compute-apps` 同上 34 条桌面应用、无本任务进程。
- 内容：只重跑设备探针 run1/run2（21 个 case 目录各一遍），**不重采上游**；按修正守卫（`qa_gpu_rerun.ps1` 第 47/94 行传入 3072/50%）自动判定。
- 采样（4 点，`qa-gpu/window-probe-rerun-memory.txt`）：峰值 19:15:41.378 `used=5463 MiB`，Δ+1481 MiB = 33.4% 设备（<3072 MiB 且 ≤50%）→ **守卫未触发**；收尾 19:15:42.045 `used=3982 MiB`，`residual_qa_probe=0`。
- 两次运行结果：`qa-gpu/window-probe-rerun1.log` 与 `...rerun2.log` **文本逐字相同**；`qa-compare-gpu.json` 中所有 case `two_runs_bitwise_identical=true`。
- 记录瑕疵：`qa-gpu/window-probe-rerun-log.txt` 是**旧标签输出**（`guard_threshold_mib=1024`、`peak_delta_mib=58` 为旧解析缺陷），权威值按原始 `window-probe-rerun-memory.txt` 重算，结论写进 `qa-gpu/window-probe-rerun-summary.txt`；脚本已改为 3072/50% 口径。另 19:11 的首次重跑尝试在进程启动阶段 Δ1542 MiB 即触发旧守卫（1 GiB）而中止、未产出数据，故按修正守卫重跑（该次中止未留独立日志文件）。

## 3. 接触数组对照（设备 vs 同配方上游 f32 GPU，2 world，QA 自算）

| case | nacon 设备/上游 | dist 位等 | pos 位等 | frame 位等 | 判定 |
| --- | --- | --- | --- | --- | --- |
| `ellipsoid-box` | 2 / 2 | 1/2（max 1.8626e-9 = 2 ULP） | 1/2（max 3.0195e-9，0.81 ULP@scale） | 0/2（max 3.798e-7 ≤ 4.768e-7） | pass |
| `hfield-sphere` | 8 / 8 | 8/8（0 ULP） | 8/8（0.0） | 8/8（0.0） | pass |
| `hfield-capsule` | 7 / 7 | 7/7（0 ULP） | 7/7（0.0） | 7/7（0.0） | pass |
| `sdf-box` | 80 / 80 | 80/80（0 ULP） | 46/80（max 7.4506e-9 = 1.0 ULP@scale） | 74/80（max 2.384e-7） | pass |

补充：四例 `worldid` 分组逐 world 计数相等；`geom/geomcollisionid/type/friction/adhesion/solref/solimp/solreffriction/dim/includemargin` 参数位级相等；`efc_address` 全 -1（本相位不建约束，写入行数= nacon）；`overflow` 全 0 且相等；设备两次运行位级相同。上游两次捕获：`ellipsoid-box`/`hfield-sphere`/`hfield-capsule` raw 行序也相同，`sdf-box` raw 行序不同（atomic 顺序），按排序后比位型仍 80/80 dist 位等——**raw 行序不作判据**。

## 4. 容量 / 截断（不越界）

| case | 结果 |
| --- | --- |
| `sdf-box-naconmax40` | `naconmax=40` 但设备与上游 `nacon` 均 80/80、实际写入行 40/40、`truncated=true`、**无越界写**、`overflow=[0,0]` 两侧相等 → pass |
| `ellipsoid-box-naccdmax1` | `naccdmax=1`：`nacon` 1/1、写行 1/1、`nccd` 设备 2、无越界；`overflow` 设备 `[16,0]` vs 上游 `[0,16]`，**位多重集相等**（CCD 溢出按 world 归属是 atomic 竞态，world 归属不作判据）→ pass；该例接触字段**不比较**（同上竞态） |

## 5. witness（`ccd()` 直调，上游 `ccd()` vs 设备 `ccd`）

基础 2 world：

| case | dist | x1 | x2 | ncontact | 判定 |
| --- | --- | --- | --- | --- | --- |
| `cylinder-box` | 2/2 位等 | max 3.725e-9，0 越界 | 0.0 | 相等 | pass |
| `box-box-margin` | 2/2 位等 | 0.0 | 0.0 | 相等 | pass |
| `mesh-mesh` | 2/2 位等 | 0.0 | 0.0 | 相等 | pass |

姿态扫描（每例 24 world，`*-sweep`）：`box-box-margin-sweep` 24/24 位等、x2 max 1.164e-9 → pass；`mesh-mesh-sweep` dist 22/24 位等（max 1 ULP = 7.45e-9）、x1 3.725e-9 → pass；`cylinder-box-sweep` dist 17/24 位等（max 4 ULP，0 越界），但 **x1 1/24 world 越界（max 8.926e-7）、x2 3/24 world 越界（max 1.527e-7）**，`ncontact` 24/24 相等 → **FAIL（发现 2，属额外采样面）**。

强制 `horizon=2`（EPA 退化，`dist = FLOAT_MAX`）：`cylinder-box-h2`/`box-box-margin-h2`/`mesh-mesh-h2` 三例 `dist` 位等（=1e30 量级 `FLOAT_MAX`）、`x1/x2` 位等、`ncontact=0` 两侧相等 → pass；但 `overflow` 设备 `272` vs 上游 `256`（`mesh-mesh-h2` 仅 world 1 出现）→ **发现 1**。

自造两例：`is_discrete` 零 margin box-box（`ccd()` 直调）base 2/2 位等、sweep 23/24 位等（1 ULP）→ pass；13×13 hfield 超容量场景 `nacon` 4/4、`prism_count=50`、`overflow=32`（= `OverflowType.HFIELD` 1<<5）两侧相等、接触 dist 4/4 位等 / pos 3/4 位等（2.0 ULP@scale，max 2.384e-7 ≤ 4.768e-7）/ frame 1.788e-7 → pass。

## 6. 发现

### 发现 1（一般）：设备在 EPA `FLOAT_MAX` 时额外置 `CCD` 溢出位

强制 `horizon=2` 三例：设备 `overflow=272`、上游 `256`，差 16 = `OverflowType.CCD`（1<<4）；上游只置 `EPA_HORIZON`（1<<8）。`ncontact` 两侧均为 0、`dist` 位等、三例中交付 3 例不触发此差异。判定：**真实语义偏差（多报一个溢出位）**，不影响本片接触数值；已由 Arch 登记修正片 `g10-p5-ccd-overflow-bit-fix`。

### 发现 2（一般，超出交付声明面）：`cylinder-box` 深穿透随机姿态采样 4/24 world 单字段超判据

24 个随机姿态 world 中：`x1` 1 个 world 超 4·ULP@scale（max 8.926e-7），`x2` 3 个 world 超（max 1.527e-7）；`dist` 无越界（max 4 ULP）、`ncontact` 24/24 相等、`overflow` 全 0。属**深穿透 EPA 在额外姿态采样上的数值差异**，不在交付声明的 3 个 witness case 内，不改变本片声明面结论；已由 Arch 登记后续片 `g10-p5-deep-epa-witness-sampling`（先定性）。

### 观察 3：native f64（MuJoCo C）材料不能作为接触容差依据（已核清）

`qa-compare-gpu.json` 曾出现 `ellipsoid-box upstream_f32_vs_native.pos_abs.max = 4.546e-4`、`frame 1.171e-3`，与 G0b 的 `7.45e-9 / 8.19e-7` 差 4–5 个数量级。核对结论：

- 不是配对错误：native `geom_xpos` 与 warp **逐位相同**（max_abs 0.000e+00）；每 world 接触数相等（`ellipsoid-box` `[1,1]` vs `[1,1]`、`sdf-box` `[40,40]` vs `[40,40]`）；`ellipsoid-box` 每 world 仅 1 接触（1:1，无配错可能）仍有 pos 4.546e-4 / frame 1.171e-3 / dist 3.961e-7。
- 量级判别：`sdf-box` 最近邻位置间距 p50 = **1.3717e-2**，若配错误差应达该量级；`sdf-box` 实测 pos 7.343e-2 / frame 1.576 / dist 8.925e-2，也在该量级——但 1:1 的 `ellipsoid-box` 同样有 4.5e-4/1.2e-3，说明**根因是 MuJoCo C 与 warp 是不同引擎语义/算法**，不是 f32 舍入。
- `hfield-sphere`/`hfield-capsule` native 每 world 接触数（`[6,6]`/`[6,3]`）与设备（`[4,4]`/`[4,3]`）不同 → 不可配对。
- 与 G0b `7.45e-9 / 8.19e-7` 的差异已澄清：那是**同码 warp CPU vs warp CUDA**，不是 native。结论：**native 材料不可用作 atol 依据**；`qa-compare-gpu.json` 的 `g6_tolerance_candidates.status` 与 `native_f64_material[*].usable_as_tolerance_basis=false` 已按此改正（不再写 1e-3 / 2.5e-3 建议值）。

## 7. G6 容差候选材料（同码跨后端基线，非 native）

来源：warp CPU vs warp CUDA f32 同一份代码、同一输入、逐接触配对（`qa-backend-precision.json`，97 个接触样本；`ellipsoid-box` 2、`hfield-sphere` 8、`hfield-capsule` 7、`sdf-box` 80）：

| 字段 | max | p99 | p95 | p50 | 建议 atol（G6 可 ×2 余量） |
| --- | --- | --- | --- | --- | --- |
| `dist` | 6.8778e-7 | 4.0908e-7 | 2.0117e-7 | 7.45e-9 | 6.8778e-7 |
| `dist_rel` | 1.3077e-4 | 1.2064e-4 | 2.9875e-6 | 1.996e-7 | rtol 1.3077e-4 |
| `pos` | 4.6194e-7 | 2.0117e-7 | 1.3225e-7 | 7.45e-9 | 4.6194e-7 |
| `frame` | 2.7781e-5 | 7.6890e-6 | 8.1929e-7 | 1.1921e-7 | 2.7781e-5 |
| `normal` | 1.62795e-5 | 4.9397e-6 | 8.1929e-7 | 1.1921e-7 | 1.62795e-5 |

说明：这是**精度基线材料**（同码不同后端的实现差异），只作 G6 容差候选与影响面输入；G5 的正式判据仍是 §1 的 4·ULP 口径。分 case 读数（含 `sdf-box` 是误差主贡献：dist max 6.8778e-7、pos 4.6194e-7、frame 2.7781e-5；`hfield-capsule` 全 0）见 `qa-backend-precision.json`。

## 8. 不回归（QA 自跑）

- `cargo test --locked --lib`：**438 passed / 0 failed / 2 ignored**（G4 时为 428/0/1，G5 新增 lib 测试 + 1 ignored 探索项）。
- `cargo test --locked --lib physics::collision`：**47 passed / 0 failed / 2 ignored**；关键项全 ok：`case_evidence::upstream_f32_within_4ulp`、`case_evidence::our_contacts_match_upstream_f32_within_4ulp`、`sdf::tests::sdf_box_host_matches_upstream_f32_within_4ulp`、`hfield::tests::hfield_host_matches_upstream_f32_within_4ulp`。
- 范围（`git diff --name-only 1f882361..e00de497`）：产品侧 21 个文件 = `src/physics/collision.rs`、`collision/cuda/*.cu`(14) + `cuda/mod.rs`、`gjk/mod.rs`、`gjk/window_debug.rs`、`relmat.rs`、`src/runtime/collision_upload.rs`、`src/runtime/mod.rs`、`tests/g10_p5_gpu_kernels.rs`；根 `Cargo.toml` / `Cargo.lock` / `rust-toolchain.toml` **0 改动**；`fixtures/mjwarp-complex-physics` **0 改动**；S2–S8 证据目录 **0 改动**（范围内非 G5 的 `docs/development-evidence` 变更均为 G4 复核证据 `0eeb3766`）。

## 9. 未覆盖 / 未过（如实记，不算通过）

1. **`multicontact` 设备版未实装 → 未过**：接触数组的位级全等只覆盖**非 multiccd** 组合；上游对本片数据明确 warning：`MULTICCD is enabled, but ... [('HFIELD','CAPSULE')] At most 1 contact will be generated`。该缺口若 G6 前补不上，作为 G5 已知未覆盖项带进 G6 汇总。
2. `capsule-box`、`sphere-box-shallow`、`sphere-box-deep`：上游 `ccd` 的 dispatch 走 `primitive_narrowphase`（不是 ccd 产品路径），设备 primitive 未实装 → **未过/未覆盖**（不列入接触数组对照，也不冒充 ccd 对照）。
3. `cylinder-box` / `box-box-margin` / `mesh-mesh`：只做 `ccd()` **witness** 对照，未做接触数组对照（上游这些组合的接触不由 ccd 内核产生）。
4. 性能/criterion：本片不在声明面，未测。
5. 产品代码：本复核未修改任何产品文件（只写本证据目录）。
6. 环境限制：`query-compute-apps` 有 3 条进程为 `[Insufficient Permissions]` 无法命名，无法 100% 排除其为 CUDA 计算进程；但基线/收尾显存一致、无 OOM、`residual_qa_probe=0`，且无新增进程。峰值采样 0.5 s 间隔，可能错过更短的瞬态峰值。

## 10. 证据目录与提交清单

入库（显式路径，全部在 `docs/development-evidence/g10-p5-gpu-kernels-independent-check/` 下）：

```
README.md  receipt.json  artifacts-sha256.json  qa_artifacts_sha256.py
qa_prepare_capture.py  qa_compare_gpu.py  qa_backend_precision.py
qa_build_probe.ps1  qa_window_run.ps1  qa_gpu_rerun.ps1
probe/qa_probe.cu
qa-upstream-gpu.json  qa-witness-gpu.json  qa-native-f64.json
qa-compare-gpu.json  qa-backend-precision.json
qa-gpu/build-recipe.txt  qa-gpu/cases-all.txt
qa-gpu/window-log.txt  qa-gpu/window-memory.txt  qa-gpu/window-capture.log
qa-gpu/window-probe-run1.log  qa-gpu/window-probe-run2.log
qa-gpu/window-probe-rerun-log.txt  qa-gpu/window-probe-rerun-memory.txt
qa-gpu/window-probe-rerun1.log  qa-gpu/window-probe-rerun2.log
qa-gpu/window-probe-rerun-summary.txt
```

**不入库（显式列出实际存在的不入库物）**：

- `qa-gpu/cases/**`：21 个 case 目录、504 个文件 = **483 个 `.bin`** + 21 个 `meta.txt`（上游输入 bins，可由 `qa_prepare_capture.py` 从同配方输入重建；`qa-upstream-gpu.json` 每个数组带 `sha256`）。
- `qa-gpu/out/**`：21 个目录、**512 个 `.bin`**（设备探针输出，由 `probe/qa_probe.cu` 对同一输入重跑重建；结论在 `qa-compare-gpu.json`）。
- `__pycache__/qa_compare_gpu.cpython-312.pyc`：**已删除**。
- 证据目录内 `.o`/`.obj`：**0 个**；证据目录内 `warp-cache/`：**0 个**（上游 warp JIT 缓存在仓库 `target/qa-g5/warp-cache/1.15.0`，在 `target/` 下，不进仓库）。
- `target/` 下构建产物（`target/qa-g5/qa_probe.exe`、test-target、warp-cache）不在仓库跟踪范围。

## 11. 复现

1. `python qa_prepare_capture.py`（需 CUDA；上游 f32 GPU 捕获 + 输入 bins，两次捕获）
2. `pwsh qa_build_probe.ps1`（recipe 与源 sha256 见 `qa-gpu/build-recipe.txt`；生成 `target/qa-g5/qa_probe.exe`）
3. `pwsh qa_window_run.ps1`（开窗前基线 → phase1/2/3 → 收尾基线/进程残留/显存采样）
4. `python qa_compare_gpu.py` → `qa-compare-gpu.json`
5. `python qa_backend_precision.py` → `qa-backend-precision.json`
6. `pwsh qa_gpu_rerun.ps1`（确定性重跑 + 自动守卫 ≥3072 MiB 或 >50% 设备即停）
7. `python qa_artifacts_sha256.py` → `artifacts-sha256.json`

## 12. 结论

- **阻断项 0**：交付声明面（4 个接触数组 case、2 个容量/截断例、witness 基础 3 例 + 强制退化 3 例、自造 2 例）在 QA 自算判据下通过；设备两次运行位级一致；不回归 438/0/2。
- **发现 2 项**（CCD 溢出位、深穿透姿态采样，均一般级，已登记后续片）+ **观察 1 项**（native 材料不可作容差依据，G6 候选已改为同码跨后端基线）。
- **未覆盖**：`multicontact` 设备版、primitive 三例、性能；均如实带入 G6 汇总，不据此判 G5 通过。
