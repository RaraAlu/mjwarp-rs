# A10@P11 Windows 单平台切片：CUDA 事件计时与显存峰值记录

任务：`2026-10-11-roadmap-a10-p11`。日期：2026-10-24。执行：Backend。
矩阵：A10 记录与性能工具（`Rust无头测速与状态导出`，P11）。
范围：**Windows 单平台**（本机 RTX 4070 Ti SUPER）。按用户 2026-10-24 决定重划：
在既有 P10 宿主侧工具（提交 `3757567`）之上补真实 GPU 事件计时与显存峰值统计的
工具与可复现记录；Linux/双平台、清洁部署与 `main.rs` CLI 接线不在本切片。

## 1. 结论（先给）

1. **CUDA 事件计时可用**：`GpuTimingSession::time_phase` 在真实设备流上用
   `cuEventRecord`/`cuEventElapsedTime` 夹住调用方阶段闭包，固定温机 10 次、
   重复 50 次；两档两阶段各 50 个纳秒样本全部为正，中位数均为 **7168 ns**
   （分区间 min 6144–6560 ns、max 7200–8192 ns）。
2. **设备显存采样可用**：`cuMemGetInfo` 采样给出设备总量 16376.5 MiB；
   空闲基线 1264.5 MiB，分配固定工作集（2×4 MiB f32）后为 1296.5 MiB
   （+32.0 MiB，驱动分配粒度/开销未拆解），释放后回到 1264.5 MiB；
   阶段内峰值 = 基线（阶段内无分配），采样 53 次/阶段。
3. **debug 与 release 分开记录**：两档各自写出记录，档位由 `cfg!(debug_assertions)`
   绑定；同一设备负载两档事件中位数一致（7168 ns），因为事件只测设备阶段，
   宿主构建差异不进入计时。
4. **记录可复现**：固定负载（仓库内置 `affine_probe` PTX，797 B，SHA-256
   `520b9ac7…` + 固定 `1<<20` f32 元素）、固定温机/重复次数写入 JSON；同一
   负载两阶段中位数一致；确定性 JSON 记录经 `serde_json` 回读校验。
5. **边界**：本片只做 Windows 单平台工具与记录，**不做物理阶段插桩、性能门槛、
   清洁部署与跨平台对照**；显存是采样口径，不含内核执行内部瞬时分配。

## 2. 条款语义与细化口径

A10 验收为「Rust无头测速与状态导出」；P11 阶段在本仓库长期缺「真实 GPU 事件
计时/显存统计」能力（P10 证据 `docs/development-evidence/backlog-batch2/2026-10-11-roadmap-a10-p10/`
第 5 节明确列为未实现）。本切片补齐这两项工具能力并产出本机记录：

- **事件计时**：不测进程墙钟、不测 PTX 编译；只测设备流上已提交工作的起止。
- **显存峰值**：`cuMemGetInfo` 的「已用 = 总量 - 空闲」采样，采样点固定为
  温机前、温机后、每次重复后、阶段结束；工作负载级检查点另记空闲/加载/分配/释放。
- **固定输入**：内置探针 PTX（摘要锁定）+ 固定元素数；输入按二进制分数生成，
  输出 `2x+1` 逐位可核对，确保被计时的是真实设备工作。
- **不重做 P10**：`BenchmarkRecorder`/`measure`/`StateExport` 与既有报告 JSON
  未改语义；P10 测试保持通过。

## 3. 实现

| 文件 | 行数 | 内容 |
| --- | ---: | --- |
| `src/runtime/tooling/perf.rs` | 462 | `BenchmarkPlan`（温机/重复必须 >0）、`TimingStats`/`PhaseTiming`（聚合与一致性校验）、`MemoryStats`/`MemoryCheckpoint`、`EnvironmentFacts`、`PerformanceReport`（校验 + 确定性 JSON 导出 + 转义） |
| `src/runtime/tooling/gpu.rs` | 197 | `GpuTimingSession`（驱动/设备探测、设备事实、PTX 内核加载、`cuMemGetInfo` 采样、事件计时 `time_phase`）；`cuda-probe` 门控 |
| `src/runtime/tooling/mod.rs` | 256 | 模块挂载与导出；`ToolingError` 增加 `ZeroWarmup`/`InvalidRecord`/`DeviceUnavailable`/`Cuda` |
| `src/runtime/tooling/tests.rs` | 429 | 新增 5 项宿主侧单测：计划拒绝面、统计与 P10 报告口径一致、记录 JSON 往返/转义/确定性、报告一致性拒绝面、档位绑定 |
| `tests/a10_p11_gpu_perf_probe.rs` | 183 | ignored GPU 探针：固定负载计时两阶段 + 输出正确性 + 显存检查点 + 记录落盘 |
| `scripts/test-windows-a10-p11.ps1` | 79 | debug/release 各跑一次探针并校验记录，落盘 `debug.json`/`release.json`/日志 |

所有新增/改动文件均 ≤500 行；`perf.rs`、`gpu.rs` 与探针测试已用
`normify module source mjwarp.runtime.tooling --add …` 挂到 `mjwarp.runtime.tooling`。

## 4. 方法（固定条件）

| 项 | 值 |
| --- | --- |
| 系统 | Windows（本机，`os=windows`、`arch=x86_64`） |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER，计算能力 8.9 |
| 驱动 | 596.36；驱动 API 版本 13020 |
| 设备显存 | 17,170,956,288 B（16376.5 MiB） |
| 工具链 | rustc 1.99.0（`+1.99.0-x86_64-pc-windows-msvc`），`--features cuda-probe` |
| 固定负载 | 内置 `src/runtime/probe.ptx` 的 `affine_probe`：双指针 + u32 元素数；797 B，SHA-256 `520b9ac7ee24cd1f284ec12f55102124871b17e5e5b0fc2a5dd1b00f19c833d1` |
| 输入规模 | 1,048,576 个 f32（4 MiB 输入 + 4 MiB 输出）；网格 8192 块 × 128 线程 |
| 温机 / 重复 | 10 / 50（`BenchmarkPlan` 固定，零值拒绝） |
| 阶段数 | 2（`affine_probe#1`、`affine_probe#2`，用于同负载重复性核对） |
| 计时口径 | 设备流事件（`cuEventRecord` → 发射 → `cuEventRecord` → `cuEventElapsedTime`） |
| 显存口径 | `cuMemGetInfo` 已用 = 总量 - 空闲；采样 53 次/阶段（温机前、温机后、50 次重复后、结束） |
| 构建 / 运行 | 记录由 `scripts/test-windows-a10-p11.ps1` 分别以 debug、release 运行 |

计时边界：事件只覆盖会话流上已提交的设备工作；主机侧提交开销、PTX JIT/加载与
进程启动不计入。`affine_probe` 每元素读 4 B + 写 4 B，1 Mi 元素工作集为 8 MiB，
输入可被 L2 缓存，**本数字不是 DRAM 带宽口径**。

显存边界：`cuMemGetInfo` 为驱动报告的全局占用（含桌面图形进程与其他 CUDA
上下文），WDDM 下其他进程波动会进入读数；采样点之间内核内部瞬时分配不可见。

## 5. 实测结果（本次，HEAD `ccc8658` + 本任务改动）

### 5.1 事件计时（纳秒/次发射）

| 档位 | 阶段 | count | min | median | mean | max | total |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| debug | `affine_probe#1` | 50 | 6144 | 7168 | 7059.84 | 8192 | 352992 |
| debug | `affine_probe#2` | 50 | 6560 | 7168 | 7114.88 | 7200 | 355744 |
| release | `affine_probe#1` | 50 | 6144 | 7168 | 7106.56 | 7776 | 355328 |
| release | `affine_probe#2` | 50 | 6144 | 7168 | 7077.76 | 7488 | 353888 |

- 全部 200 个样本 > 0；事件时间分辨率为数微秒档，样本集中在 6144–8192 ns。
- 同负载两阶段中位数一致（7168 ns）；debug/release 中位数一致，符合「事件只测
  设备阶段」的口径。
- 8 MiB/发射按中位数折合约 1.17 TB/s 等效吞吐——**L2 常驻**所致，不作带宽结论。

### 5.2 设备显存（字节，采样口径）

| 检查点 | debug | release |
| --- | ---: | ---: |
| 空闲（会话创建、加载前） | 1,325,924,352 | 1,325,924,352 |
| 内核加载后 | 1,325,924,352 | 1,325,924,352 |
| 固定工作集分配后 | 1,359,478,784 | 1,359,478,784 |
| 释放后 | 1,325,924,352 | 1,325,924,352 |
| 阶段内峰值（每阶段） | 1,359,478,784 | 1,359,478,784 |

- 固定工作集申请 2×4 MiB = 8 MiB，读数增量 33,554,432 B（32.0 MiB）：
  含驱动分配粒度/额外开销，本片未拆解到单次分配，如实记录。
- 空闲读数含桌面图形进程等其它进程占用（1296.5 MiB 级），非本进程独占口径。

### 5.3 墙钟（脚本口径，含 cargo 构建）

| 档位 | 脚本墙钟 | 探针测试二进制 |
| --- | ---: | --- |
| debug | 0.342 s（复用已构建产物） | 0.15 s |
| release | 11.459 s（含 release 构建 11.21 s） | 0.14 s |

### 5.4 记录内容

`debug.json`/`release.json` 各含：`schema`、`task`、`workload`、`environment`
（档位/系统/架构/GPU/计算能力/驱动 API）、`plan`（10+50）、两条 `records`
（逐次 50 个纳秒样本 + 聚合统计 + 显存统计）、4 个 `checkpoints`、5 条 `notes`
（固定负载非物理阶段、计时/显存口径、单平台、GPU 非独占）。JSON 经
`serde_json` 回读校验后才落盘。

## 6. 测试与门禁

- 宿主侧单测（默认特性、无 GPU）：`cargo test --locked --lib runtime::tooling`
  → **10 过 0 败**（新增 5 项：计划拒绝面、统计与 P10 报告口径一致、记录 JSON
  往返/转义/确定性、报告一致性拒绝面、档位绑定）。
- `cargo test --locked`（unit 门禁口径）：**485 过 0 败**（87 个测试目标；
  较上基线 480 新增 5 项）。
- GPU 探针：debug/release 各 **1 过 0 败**（`probe-debug.log`、`probe-release.log`）。
- 门禁 `fmt`/`clippy`/`clippy-cuda`/`unit` 全部通过，真实数字见
  `.normify/runs/2026-10-11-roadmap-a10-p11/`。

## 7. 未验收边界（如实登记）

1. **不是物理阶段性能**：固定负载是仓库既有 `affine_probe` 微基准，不是
   物理阶段（运动学/质量矩阵/求解等）；工具对任意阶段闭包可复用，但本片
   未把产品物理阶段接进计时（不做 `main.rs` CLI 接线）。
2. **无性能门槛**：只记录，不设加速比/延迟目标，也不据此反推精度验收。
3. **Linux 与双平台 deferred**：仅 Windows 单平台、单设备；不含 Linux 对比、
   不含跨设备对照，Linux 部分并入 `2026-10-24-linux-batch-retest`。
4. **清洁部署未做**：运行依赖本机已有 NVIDIA 驱动与构建产物。
5. **显存为采样口径**：不含内核内部瞬时分配；WDDM 下其他进程占用进入读数；
   固定工作集 8 MiB 对应的读数为 +32 MiB，未拆解分配粒度/开销。
6. **cuda-graph/多流未覆盖**：只覆盖单流顺序发射；CUDA 图耗时、多流并发与
   kernel 级拆分未实现。
7. 不宣称本阶段之外的工具能力（CSV/二进制导出、跨平台报告合并）已具备。

## 8. 验收指标映射（NP01–NP04）

- **NP01**：本片不改产品物理字段与精度；记录中的 mean/median 为诊断统计
  （f64），不反馈产品物理结果。
- **NP02**：新增的是测量工具与记录格式（JSON schema + 校验 + 拒绝面），
  不改既有精度契约、容差、断言与 fixtures。
- **NP03**：不适用——本片不做数值等价对照。
- **NP04**：本片补 P11 所需的「真实 GPU 事件计时 + 显存峰值」工具与单平台
  记录；不做性能门槛判定、不做双平台对照。采样本机独占 GPU 窗口完成，
  未与其他 GPU 满载任务重叠。

## 9. 复现

```powershell
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\test-windows-a10-p11.ps1
```

脚本行为：debug、release 各运行一次 ignored 探针（`--features cuda-probe
--test a10_p11_gpu_perf_probe -- --ignored --test-threads=1 --nocapture`），
校验记录任务/档位/样本数/计时为正后写回 `debug.json`、`release.json` 与日志；
最后跑一次宿主侧 tooling 单测落 `unit-tests.log`。任何校验不符即抛错终止。

## 10. 证据索引

- `debug.json`、`release.json`：两档机器可判定记录（环境、计划、逐样本计时、
  显存统计、检查点、未测项）。
- `probe-debug.log`、`probe-release.log`：探针运行日志与摘要行。
- `unit-tests.log`：宿主侧 tooling 单测（10 过 0 败）。
- `receipt.json`：任务回执（task_id、平台、计划、实测数字、文件清单、边界）。
- 实现：`src/runtime/tooling/perf.rs`、`src/runtime/tooling/gpu.rs`、
  `tests/a10_p11_gpu_perf_probe.rs`、`scripts/test-windows-a10-p11.ps1`。
