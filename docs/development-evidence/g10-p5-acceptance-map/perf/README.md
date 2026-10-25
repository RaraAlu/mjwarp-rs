# G6-2 性能证据（host / 设备 kernel-only / 设备端到端 / 上游 f32 CPU 参考）

- 任务：`2026-10-25-g10-p5-acceptance-perf-native`（G6-2）
- 状态：**已采集**（一次获批独占 GPU 窗口，wall 5.5 s，守卫未触发）；数字只作性能证据，**不进精度判据、不跨硬件外推**。
- 上游 pinned：`71da24d956378a87a703b6e1442b13aec0c4ac29`（MJWarp 3.15）；本机实测上游树 HEAD 与 pinned 一致。
- 聚合结果：`perf-summary.json`（每 case 四列 + 覆盖度 + 环境）；原始样本在 `device/<case>.json`、`host/<case>.json`、`upstream-cpu.json`。

## 1. 窗口事实（HEAD 绑定 + 基线/峰值/桌面负载）

| 项 | 值 |
| --- | --- |
| 窗口起点 HEAD | `9c0a0480cd4a1be9508155261c8286e9eae1b54b`（`9c0a0480`） |
| 相对 `6510317f` 产品代码变更 | `git diff 6510317f..HEAD -- src tests Cargo.toml Cargo.lock` = **空**（只有 docs/治理提交） |
| 窗口时工作区 | 9 项未提交（本片 host harness：`src/physics/collision.rs`、`gjk/case_evidence.rs`、`sdf/tests.rs`、`hfield/tests.rs`、新增 `perf_report.rs`；证据目录与本片脚本），均已记入 `artifacts-sha256.txt` |
| 基线（20:23:05） | RTX 4070 Ti SUPER 16376 MiB，used **3663** / free 12399，util 0%，29°C，driver 596.36 |
| 开跑前复测 | used 3663（20:23:01 漂移 0 MiB）；本任务相关计算进程 **0**；cargo/rustc 空闲 |
| 图形上下文 | 桌面上下文 34 个（explorer/Chrome/EdgeWebView/微信/Feishu/NVIDIA Overlay/VS Code 等，WDDM 图形，不算计算进程） |
| 桌面负载 | 进程数 395；WorkingSet 前五：`vmware-vmx:17548MiB, DeepSeek Harness:1768MiB, Code:1085MiB, MsMpEng:735MiB, explorer:684MiB`（host 计时无 CPU 隔离，见 §5） |
| 采样口径 | 0.5 s 一次 `memory.used,utilization.gpu,temperature.gpu`，落 `window-memory-samples.txt`（本窗口 wall 5.5 s → **7 点**，峰值采样较稀疏，如实标注） |
| 峰值 | used **4834 MiB**，增量 **+1171 MiB**；守卫 3072 MiB / 50%（8188 MiB）**未触发** |
| 窗口内 GPU 负载 | util max 90% / avg 20%；temp max 31°C |
| 结束基线（20:23:09） | used **3663**（回落为基线）；图形上下文仍 34；本任务相关计算进程 **0**（无 probe/python 残留） |
| elapsed | 5.5 s（构建与预编译在窗口外完成，见 §6） |

环境：GPU RTX 4070 Ti SUPER（sm_89，16376 MiB）、cudart 12080、driver 596.36、nvcc `release 12.8, V12.8.93`（`target/toolchains/cuda-12.8.1`）；CPU AMD Ryzen 9 9950X3D 16C/32T max 4300 MHz；Windows 11 26200。上游参考：warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / Python 3.12.13 / threads=1（backend=cpu）。

## 2. 四列口径

| 列 | 口径 |
| --- | --- |
| `host_cpu_single_thread` | Rust host 产品路径，纯 CPU 单线程；warmup 20 + 200 次；一个样本 = 该 case 全部 world 各一次产品调用（G2：`ccd_pair`；G3：`sdf_pair_contacts`；G4：hfield 接触）；中位/p95/contacts/s，装载/建 pair 在计时区外 |
| `device_kernel_only` | 设备内核单次发射，**CUDA events**（`cudaEventElapsedTime`）；warmup 5 + 50；输出清零在同一流、start 事件之前，不计入计时 |
| `device_end_to_end` | 设备一次完整调用：`geom_xpos`/`geom_xmat` H2D + 输出清零 + kernel + **全部输出 D2H** + `cudaDeviceSynchronize`，host `steady_clock`；warmup 5 + 50 |
| `upstream_f32_cpu` | 冻结上游 f32 CPU 同机同版本参考（离线 Python，`mujoco_warp.collision` + `wp.synchronize_device`）；warmup 20 + 200；编译/加载/make_data 不计入 |

## 3. 结果（中位 / p95，ms）

| case | host median | host p95 | 设备 kernel median | kernel p95 | 设备 e2e median | e2e p95 | 上游 CPU median | 上游 p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| box-box-margin | - | - | 0.052 | 0.053 | 0.599 | 0.710 | 0.645 | 0.696 |
| convex-mesh | 0.013 | 0.013 | - | - | - | - | 1.678 | 1.760 |
| cylinder-box | 0.020 | 0.020 | 0.022 | 0.023 | 0.538 | 0.619 | 1.501 | 1.588 |
| degenerate-mesh | 0.014 | 0.015 | - | - | - | - | 1.669 | 1.746 |
| ellipsoid-box | 0.017 | 0.018 | 0.035 | 0.035 | 0.557 | 0.727 | - | - |
| hfield-box | 0.050 | 0.051 | - | - | - | - | 1.729 | 1.798 |
| hfield-capsule | 0.070 | 0.073 | 0.142 | 0.144 | 0.673 | 0.807 | 1.699 | 1.787 |
| hfield-sphere | 0.012 | 0.012 | 0.021 | 0.022 | 0.584 | 0.726 | 1.703 | 1.829 |
| mesh-mesh | 0.010 | 0.010 | 0.022 | 0.023 | 0.557 | 0.696 | 1.604 | 1.682 |
| sdf-box | 2.697 | 2.740 | 4.774 | 4.778 | 5.060 | 5.377 | 11.722 | 12.257 |

吞吐（contacts/s，按中位；样本量 200）：

| case | contacts/次（见脚注） | host | 设备 kernel | 上游 CPU |
| --- | ---: | ---: | ---: | ---: |
| box-box-margin | 2（设备）/ 10（上游） | - | 38284.8 | 15506.3 |
| convex-mesh | 2 | 152671.8 | - | 1191.9 |
| cylinder-box | 8（host/上游）/ 2（设备） | 406091.4 | 90711.2 | 5328.2 |
| degenerate-mesh | 2 | 139860.1 | - | 1198.0 |
| ellipsoid-box | 2 | 115606.9 | 57444.9 | - |
| hfield-box | 8 | 161290.3 | - | 4628.2 |
| hfield-capsule | 7 | 99573.3 | 49179.4 | 4121.2 |
| hfield-sphere | 8 | 677966.1 | 383141.8 | 4696.6 |
| mesh-mesh | 2 | 198019.8 | 93006.0 | 1246.9 |
| sdf-box | 80 | 29665.3 | 16757.8 | 6824.9 |

> **吞吐表脚注**：`contacts/次` 按各列**自身 case 定义**取数，同一 case 跨列可能不同（见 §5 第 9 条）；`contacts/s` 只按自身接触数计算，**跨列不可直接相除比较**。

设备 case 的 `nacon/nccd/overflow`（探针回读，仅作与 G5 已验收对照的 sanity，不作本片判据）：`hfield-sphere` 8/0/[0,0]、`hfield-capsule` 7/0/[0,0]、`sdf-box` 80/0/[0,0]、`ellipsoid-box` 2/2/[0,0]、`cylinder-box` 2/2/[0,0]、`box-box-margin` 2/2/[0,0]、`mesh-mesh` 2/2/[0,0]（与 G5 已验收 nacon 一致）。

## 4. 覆盖度（缺口显式，不补齐、不外推）

- **host 9/9**：G2 实际 harness 五 case = `cylinder-box`/`ellipsoid-box`/`mesh-mesh`/`convex-mesh`/`degenerate-mesh` + G3 `sdf-box` + G4 三例。`box-box-margin` 在本仓库证据里属 PRIMITIVE（G5/设备侧），不在 G2 host harness 中，故 host 列缺它。
- **device 7/7**：冻结 G5 case 目录只有 7 个 case；`convex-mesh`/`degenerate-mesh`/`hfield-box` 没有设备探针输入（G5 未造这些设备 case），故设备两列缺这三项 — 记为缺口，不补造输入。
- **upstream 9/9**：按任务清单的 G2 五 case（含 `box-box-margin`）+ sdf-box + hfield 三例；本地 G2 harness 用的 `ellipsoid-box` 不在上游清单中，故上游列缺 `ellipsoid-box`。
- 结论：四列不存在「同一 case 四列齐全」的子集；跨列比较只在同 case 两列之间做量级参考，**不构成等价或性能达标声明**。

## 5. 解释与限制（防误读）

1. **设备端到端列对许多 case 是传输主导**：探针按产品形状把输出缓冲固定为 `naconmax=8192` 行，e2e 每次 D2H 全部输出字段（dist/pos/frame/includermargin/friction/.../efc），对 20–50 µs 的内核，约 0.5–0.7 ms 的 e2e 主要是固定传输成本；**设备计算量看 kernel-only 列**，e2e 列展示「按产品缓冲形状一次完整调用的固定开销」。
2. **冒烟不计入证据**：`hfield-sphere --perf 0 1` 的 kernel 27.2 ms 含首次 CUDA 上下文/模块初始化（一次性），只用于验证探针可运行；正式 kernel 列 warmup 5 次后测量。
3. **host 计时无 CPU 隔离**：窗口在共享桌面上执行（395 进程、`vmware-vmx` 约 17.5 GiB WorkingSet、Chrome/VS Code 等常驻），host 中位/p95 含桌面 CPU 干扰；同机可比、跨机不可比。
4. **采样稀疏**：窗口 wall 5.5 s，0.5 s 采样只有 7 点；峰值 4834 MiB（+1171）是这 7 点的最大值，不排除采样间隙的更高瞬时值 — 防守卫口径（未触发）与结束回落到基线。
5. **单轮**：host 与上游各单轮 200 样本；上游另跑一轮临时输出中位差约 −4%～+6%，说明中位/p95 只作量级参考。
6. **排除项**：编译/JIT/加载时间不计入任何列（nvcc 构建在窗口外；上游 `load_ms_total=60.69`、`compile_ms_total=48.24`、`make_data_ms_total=19.89` 单列在 `upstream-cpu.json`）；性能不进精度判据、不跨硬件外推、不用于 NP03/NP04 放行声明（NP04 仅表示「已记录耗时与显存」）。
7. 上游 CPU 计时关闭 Python 循环 GC（timeit 惯例；否则 `cylinder-box` 第 17 个计时样本出现一次约 51 ms GC 暂停），JSON 已注明；未剔除离群点。
8. `upstream-cpu.json` 的 p95 用 numpy 线性插值，`perf-summary.json` 的 p95 用 nearest-rank，两者口径已在各自 JSON 注明（差异 <1%，不影响量级结论）。
9. **同一 case 跨列的接触数可能不同（不可直接相除比较）**：四列基于不同 case 定义/参数——设备列用冻结 G5 case 输入（产品形状 `naconmax=8192`），上游列按任务清单 `fixtures/g10-p5/references/*.json`，host 列用本仓库 G2/G3/G4 harness；接触数随 margin/gap/multicontact 策略不同。已发现两处具体差异：`box-box-margin` 设备列 `nacon=2` vs 上游 CPU 接触数 **10**（上游 `contacts_per_s=15506.3` 用 10 算）；`cylinder-box` host 列 8、上游 8 vs 设备列 `nacon=2`（设备 `contacts_per_s=90711.2` 用 2 算，host `406091.4` 用 8 算）。因此 `contacts/s` 只按各列自身接触数计算，**跨列只作量级参考，不可直接比较比值或假定同一基数**。

## 6. 复现命令（authoring-only；窗口需 Arch+Chief 双签）

```text
# 1) 复制冻结 G5 探针源 + 加 --perf 计时分支（split 脚本内含来源 sha256）
powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\g10-p5-acceptance-map\perf\split_perf_probe.ps1
# 2) 构建 perf 探针（CPU，不占 GPU；产物 target/g6-perf/probe_perf.exe）
powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\g10-p5-acceptance-map\perf\build_perf_probe.ps1
# 3) 预构建 host 测量件（窗口外；窗口内不编译）
cargo test --locked --lib perf_ --no-run
# 4) 先跑上游 f32 CPU 参考（离线 CPU，不占 GPU）
target\mjwarp-parallel\venv\Scripts\python.exe -B docs\development-evidence\g10-p5-acceptance-map\perf\perf_upstream_cpu.py
# 5) 获批窗口内执行（基线/守卫/0.5 s 采样/设备 7 case/host 9 case/聚合/artifacts）
powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\g10-p5-acceptance-map\perf\run_window.ps1
# 6) 窗口后重算聚合（可选；窗口内已跑过一次）
target\mjwarp-parallel\venv\Scripts\python.exe -B docs\development-evidence\g10-p5-acceptance-map\perf\aggregate_perf.py
```

## 7. 文件

| 文件 | 内容 |
| --- | --- |
| `perf-summary.json` | 聚合：每 case 四列 + 覆盖度 + 环境 + 口径说明 |
| `window-log.txt` / `window-memory-samples.txt` | 窗口基线/HEAD 绑定/桌面负载/逐步守卫/0.5 s 采样/峰值/结束 |
| `device/<case>.json`（7 个） | 设备原始样本：`kernel_ns[50]`、`e2e_ns[50]`、nacon/nccd/overflow、free/total |
| `host/<case>.json`（9 个） | host 原始样本：`samples_ns[200]`、median/p95、ncontacts、contacts/s |
| `upstream-cpu.json` / `upstream-cpu.log` | 上游 f32 CPU 参考（9 case、raw 200 ns、环境、编译/加载单列） |
| `perf_probe*.cu` / `split_perf_probe.ps1` / `build_perf_probe.ps1` | 性能探针源（冻结 G5 probe.cu 机械拆分 + `--perf` 分支）与构建脚本 |
| `run_window.ps1` / `aggregate_perf.py` | 窗口执行与聚合脚本 |
| `perf_upstream_cpu.py` | 上游 f32 CPU 计时脚本（离线 authoring） |
| `build-report.txt` / `artifacts-sha256.txt` | 构建命令与产物/源 sha256 |

## 8. `src/` 5 处改动均为 test-only（无需 verified 任务加注）

- `src/physics/collision/perf_report.rs`（新文件）是 `#[cfg(test)]` 模块，只在 unit 测试编译；
- `src/physics/collision/gjk/case_evidence.rs`、`src/physics/collision/sdf/tests.rs`、`src/physics/collision/hfield/tests.rs` 是 **helper 抽取 + 各 1 个 `#[ignore]` 性能测试**（`perf_gjk_case_throughput`/`perf_sdf_box_throughput`/`perf_hfield_case_throughput`）；
- **既有断言一条未删**（Arch 2026-10-25 逐条核对：「参考 geom 结构」「hfield 为 g1」「对侧为 g2」「不应溢出」「接触数」等均在原位或随 helper 迁移保留）；**非 ignored 测试数不变**（每文件 +1 test/+1 ignore）；产品语义、`#[test]` 判据与已 verified 数字均未改，`normify gate` unit **588 过 0 败**覆盖。
- 因此 G2/G3/G4 verified 任务**不需要** `normify fix` 加注（Arch 2026-10-25 裁决：verified 语义未变，加 test-only addenda 只会稀释证据链）。
