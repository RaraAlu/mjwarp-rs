# NP04 Windows 单平台性能与峰值显存证据

任务：`2026-10-23-np04-windows-performance`（P11-精度门禁，owner QA）。日期：2026-10-23。
范围：Windows 单平台（RTX 4070 Ti SUPER）上既有 GPU 批量测试套件的耗时与峰值显存采样；**不含 Linux 对比**。
本任务只跑既有测试并采样，**未改产品源码/测试/容差/fixtures/脚本/排除清单**。

## 1. 结论（先给）

1. **180 次运行全部通过且计数校验一致**：9 个代表套件 × 每项 5 样本 × 冷/热两种驱动缓存 × debug/release 两轮 = 180 次进程运行，每次均校验 `test result: ok` 且通过数等于冻结期望（见 `bench.ps1` 的 `expected` 表），无失败、无重试剔除。
2. **稳态（driver-warm，5 样本中位数）耗时**：debug 4.90–17.56s、release 4.00–16.36s（按套件）；release 相对 debug 快 **3.5%–27.1%**（mass-matrix/mass-solve/mass_solve_probe 降幅最大，与减少调试开销一致）。
3. **冷启动（driver-cold，每次全新空 CUDA 缓存）相对稳态慢 0.8%–6.9%**（中位数）；NVRTC 编译在进程内每次发生，冷/热差异主要来自驱动 PTX→SASS 缓存，量级 <7%。
4. **峰值显存**：`nvidia-smi` 总占用采样（约 100ms）相对样本前空闲值的增量稳定在 **216–219 MiB**，进程 CUDA 上下文（约 218 MiB）占主导；**采样分辨率不足以区分套件间批量激活差异**（如实标注，不伪造更细的数字）。
5. **宿主峰值工作集**：按套件 151.6–202.2 MiB；mass-solve 系（`resident_mass_solve`、`mass_solve_probe`）最高（约 190–202 MiB）。
6. **边界**：仅 Windows 单平台；无 Linux 对比；不宣称双平台性能/显存达标、不宣称 NP04 整体通过；冷启动/上传耗时未拆分、GPU 非独占（桌面图形进程常驻）。全程 GPU 只跑本任务，未与其他 GPU 满载任务重叠。

## 2. 环境与固定条件

| 项 | 值 |
| --- | --- |
| 系统 | Windows 11 专业版 10.0.26200 build 26200 |
| CPU | AMD Ryzen 9 9950X3D 16-Core |
| 内存 | 32,000 MiB 级（`environment.json` 记录实际值） |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER（16,376 MiB） |
| 驱动 | 596.36 |
| CUDA / NVRTC | 12.8.1 工具链 / NVRTC 12.8.93 |
| Rust | 1.99.0（`b940084d7 2026-09-28`），MSVC，`--all-features` |
| 代码身份 | `0c4db70efd8e01cb6607b06aac21587b7a03996d`（debug 与 release 同 HEAD） |
| 固定输入 | `git rev-parse HEAD:fixtures`（fixtures 树哈希，见 environment.json）；`src`/`tests` 树哈希同记录 |
| 精度 | 产品 f32；既有容差/断言未改动（G01 `2e-5`、G03 `2e-4`、RNE/G26 `2e-5` 等） |
| 构建 | debug 增量 0.45s（复用已构建产物）；release 33.6s（复用上一轮 release 依赖） |

> 说明：采样时工作区未提交内容只有本任务自己的证据目录与任务 JSON；产品 `src`、`tests`、`fixtures` 树哈希与 HEAD 一致，`environment.json` 的 `workingTreePorcelain` 已如实记录。

## 3. 方法

- **套件（9 个代表）**：`resident_kinematics`（6 项）、`resident_mass_matrix`（2）、`resident_mass_solve`（2）、`resident_rne_bias`（7）、`resident_com_velocity`（6）、`resident_rne_acceleration`（2）、`mass_solve_probe`（6）、`g01`（7）、`g26_energy_chain`（1）。每个套件是一个既有 ignored GPU 测试二进制，输入/世界数/精度由仓库 fixtures 固定。
- **每项 ≥5 样本**：每套件每模式 5 次独立进程运行，记录全部样本、中位数与 min/max。
- **冷/热分开**：`driver-cold` 每次样本使用全新空 `CUDA_CACHE_PATH`（含驱动 JIT 冷启动）；`driver-warm` 复用预热的共享缓存（预热运行不记录，但计数校验）。预热在 5 样本前完成。
- **计时边界**：进程墙钟 = 进程启动 + PTX 编译/JIT/加载 + 测试内全部 GPU 批量；构建时间不计入。同步由测试内既有同步点保证。
- **显存边界**：每 100ms 采样 `nvidia-smi --query-gpu=memory.used`（MiB），取样本前空闲值与运行期峰值的差；宿主峰值取进程 `WorkingSet64` 采样最大值。
- **判定**：只采时间与显存，不据此反推精度通过；`test result` 通过数必须等于期望，否则脚本抛错终止（本次无触发）。
- **总时长**：debug 10:36–10:54（约 17 分钟）、release 10:54–11:09（约 15 分钟），合计约 **32 分钟**，在既定 90 分钟有界要求内，未缩样板。

复现：`bench.ps1 -Samples 5`（debug）与 `bench.ps1 -Samples 5 -Release`；原始数据在 `raw/debug/<stamp>/`、`raw/release/<stamp>/`。

## 4. 结果

### 4.1 稳态（driver-warm）耗时：中位数（min–max），秒

| 套件 | debug | release |
| --- | ---: | ---: |
| resident_kinematics | 7.575（7.487–7.676） | 7.206（7.171–7.375） |
| resident_mass_matrix | 4.898（4.808–5.010） | 4.001（3.919–4.045） |
| resident_mass_solve | 6.818（6.766–6.968） | 4.967（4.873–4.973） |
| resident_rne_bias | 14.172（14.154–14.201） | 13.650（13.580–13.750） |
| resident_com_velocity | 5.941（5.886–5.980） | 5.733（5.691–5.908） |
| resident_rne_acceleration | 14.531（14.378–14.783） | 13.562（13.457–13.813） |
| mass_solve_probe | 9.111（8.891–9.146） | 7.627（7.515–7.639） |
| g01 | 17.560（17.264–17.701） | 16.358（16.150–16.661） |
| g26_energy_chain | 7.358（7.329–7.592） | 7.096（7.047–7.106） |

release 相对 debug（稳态中位数）：kinematics −4.9%、mass_matrix −18.3%、mass_solve −27.1%、rne_bias −3.7%、com_velocity −3.5%、rne_acc −6.7%、mass_solve_probe −16.3%、g01 −6.8%、g26 −3.6%。

### 4.2 冷启动（driver-cold）与稳态差（中位数）

| 套件 | debug 冷 vs 温 | release 冷 vs 温 |
| --- | ---: | ---: |
| resident_kinematics | +0.8% | +2.2% |
| resident_mass_matrix | +4.6% | +3.9% |
| resident_mass_solve | +4.1% | +3.3% |
| resident_rne_bias | +1.7% | +0.8% |
| resident_com_velocity | +3.9% | +4.6% |
| resident_rne_acceleration | +2.3% | +1.7% |
| mass_solve_probe | +1.9% | +2.4% |
| g01 | +1.9% | +3.6% |
| g26_energy_chain | +6.9% | +6.2% |

冷启动中位数：debug 5.122–17.890s、release 4.158–16.954s（完整 min/max/全部样本见 `raw/*/summary.json` 与 `runs.json`）。

### 4.3 峰值显存（进程增量，MiB）

| 口径 | debug | release |
| --- | ---: | ---: |
| driver-warm 套件中位数 | 217–219 | 216–218 |
| driver-cold 套件中位数 | 216–219 | 216–218 |

**限制（如实标注）**：采样约 100ms、GPU 由桌面图形进程共享，WDDM 下 `nvidia-smi --query-compute-apps` 的每进程显存为 `N/A`；因此只能给出「总占用相对空闲值的增量」，且该增量被每进程 CUDA 上下文（约 218 MiB）主导。套件间差异 ≤3 MiB，小于采样噪声/上下文波动，**不足以区分批量激活显存差异**，本证据不作套件级显存排序。

### 4.4 宿主峰值工作集（中位数，MiB）

| 套件 | debug 冷 | debug 温 | release 冷 | release 温 |
| --- | ---: | ---: | ---: | ---: |
| resident_kinematics | 162.5 | 154.6 | 160.5 | 152.2 |
| resident_mass_matrix | 161.5 | 153.6 | 159.5 | 151.6 |
| resident_mass_solve | 201.8 | 193.3 | 199.6 | 190.6 |
| resident_rne_bias | 164.7 | 157.4 | 162.8 | 155.0 |
| resident_com_velocity | 162.3 | 154.4 | 160.2 | 152.1 |
| resident_rne_acceleration | 164.6 | 156.0 | 161.9 | 153.8 |
| mass_solve_probe | 202.2 | 194.1 | 200.2 | 191.6 |
| g01 | 181.1 | 165.0 | 178.8 | 161.4 |
| g26_energy_chain | 166.8 | 156.2 | 163.8 | 153.8 |

## 5. 抽样口径与未覆盖清单

**已覆盖**：9 个代表套件（运动学、质量矩阵、质量求解、RNE bias、质心速度、RNE 加速度、G03 质量求解探针、G01 主链、G26 能量链），每套件 debug/release × 冷/热 × 5 样本，共 180 次运行；每项均有中位数与 min/max。

**未覆盖（如实标注，不当成通过）**：

1. **非全路径**：只覆盖 9 个代表套件，未对 NP01 的 30 条产品路径逐一测性能；结论不外推到完整产品。
2. **无 Linux 对比**：用户 2026-10-23 决定 Linux deferred；本证据不含跨平台加速比或对照。
3. **无单内核微基准**：套件耗时含进程启动、PTX 编译/JIT/加载与批量准备，不是纯内核吞吐；不用于推断单内核优化空间。
4. **上传/冷启动耗时未拆分**：进程墙钟包含 host→device 上传与加载，未单独计时（需要产品侧插桩，超出「不改测试」范围）。
5. **显存分辨率限制**：见 §4.3，只能给出总占用增量与上下文主导结论。
6. **GPU 非独占**：桌面图形进程常驻（占用约 3.2 GiB 图形内存），无其他 CUDA 计算任务；采样期间未与 GPU 满载任务重叠。
7. **未做性能目标判定**：本任务只记录耗时与显存，不设加速比目标、不据此判定精度通过。

## 6. 平台边界

- 仅 Windows x86_64 MSVC + RTX 4070 Ti SUPER（本机 DarkSea）；**不含 Linux 对比**。
- 工程值只代表本机本驱动本 HEAD 的实测；不作为双平台性能/显存达标依据。
- 不宣称双平台性能验收、不宣称 NP04 整体通过；性能数字不反推精度验收结论（精度证据见 `docs/development-evidence/np03-windows-evidence/`）。
- 未改产品源码/测试/容差/fixtures/脚本与 G01 排除清单。

## 7. 复现命令

```powershell
# debug：9 套件 × 5 样本 × 冷/热（约 17 分钟）
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File docs\development-evidence\np04-windows-performance\bench.ps1 -Samples 5

# release：同口径（构建约 34s + 约 15 分钟）
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File docs\development-evidence\np04-windows-performance\bench.ps1 -Samples 5 -Release

# 单套件冒烟
... -File docs\development-evidence\np04-windows-performance\bench.ps1 -Samples 1 -WarmOnly -Suites g26_energy_chain
```

脚本行为：先 `cargo test --no-run` 解析测试二进制（构建不计入测量），预热共享缓存，然后逐样本启动新进程（`--ignored`/`g01` 用 `--include-ignored`）、100ms 采样显存与宿主工作集、校验通过数等于期望；任何计数不符即抛错终止，不静默跳过。

## 8. 证据索引

- `np04-windows.json`：机器可判定摘要（环境、代码身份、方法、逐套件中位数/min/max、冷热差、限制、未覆盖清单）。
- `bench.ps1`：采样脚本（只读运行既有测试；不改产品/测试）。
- `raw/debug/20261023T023645Z/`、`raw/release/20261023T025312Z/`：`environment.json`、`runs.json`（180 条原始样本）、`summary.json`（逐模式统计）。
- 本机日志（gitignored）：`target/np04-bench.log`（总日志）、`target/np04-logs/<stamp>-<profile>/`（每次运行 stdout/stderr）。
