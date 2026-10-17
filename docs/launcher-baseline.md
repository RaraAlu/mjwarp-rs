# M0 编译开销基线

日期：2026-10-18。
任务：`2026-10-18-layers-m0-w16`。
对应工作项：[W16](unified-launcher.md#需要额外编写的部分)。
状态：Windows debug实测完成。
本报告只测编译与加载开销。
本报告不证明物理精度达标。
本报告不构成性能验收。

## 结论

现有L1发射路径以重复编译为主。
5个常驻套件共编译512次。
其中只有8份不同源码。
进程内首次编译约占编译耗时5%。
重复编译约占端到端耗时77%至79%。
驱动JIT冷缓存只增约0.9秒加载。
模块加载显存每次不超过2 MiB。

缓存优先级决议如下：

| 优先级 | 工作 | 依据 |
| --- | --- | --- |
| 1 | 进程内产物与模块缓存 | 可消除约32秒重复编译，占77%以上 |
| 2 | 同源多入口共用一个模块 | 8份源码对应11个入口，每入口各编一次 |
| 3 | 磁盘PTX缓存 | 每进程首次编译合计约1.6秒 |
| 4 | 驱动JIT产物缓存 | 驱动已自带；冷热差约0.9秒 |

该决议适用于M2两层缓存的排序。
它不改变M2的契约与门禁。

## 测量对象

### 编译路径

测量对象为`SynchronousKernel::compile`。
它覆盖L1的全部NVRTC编译。
L2冻结PTX与L3探针不在本次范围。

记录器新增于`src/runtime/transfer/kernel.rs`。
它只在设置以下变量时工作：

```text
MJWARP_LAUNCHER_BASELINE_DIR
```

未设置时不计时显存、不写文件。
编译选项与内核字符串保持不变。
同步语义保持不变。

每次编译写一行JSON记录：

| 字段 | 含义 |
| --- | --- |
| `entry` | 入口名 |
| `sourceSha256` | 内核源码摘要 |
| `ptxSha256` | 生成PTX摘要 |
| `nvrtcVersion` | NVRTC主次版本 |
| `options`、`fmad`、`useFastMath` | 实际编译配方 |
| `occurrence` | 本进程内同源码第几次编译 |
| `compileNs` | NVRTC编译耗时 |
| `checkNs` | PTX精度检查耗时 |
| `loadNs` | 模块加载与取函数耗时，含驱动JIT |
| `deviceFreeBeforeLoad`、`deviceFreeAfterLoad` | 加载前后空闲显存 |
| `deviceTotal` | 设备总显存 |

`occurrence`为1表示进程内首次。
大于1表示可被进程内缓存消除。

### 工作负载

| 套件 | 测试数 | 主要阶段 |
| --- | ---: | --- |
| `resident_kinematics` | 6 | 刚体、附着与质心 |
| `resident_mass_matrix` | 2 | CRB与稠密矩阵 |
| `resident_mass_solve` | 2 | 分解与回代 |
| `resident_rne_bias` | 7 | RNE偏置力 |
| `resident_com_velocity` | 6 | 空间速度 |

全部套件以`--ignored --test-threads=1`运行。
每次运行核对通过数，失败数须为零。
测试数漂移时脚本直接停止。

套件属于测试工作负载。
它们反复构建计划，编译次数偏高。
产品长进程通常只建一次计划。
因此重复比例不能外推到产品稳态。

### 两种驱动缓存模式

| 模式 | 设置 | 衡量对象 |
| --- | --- | --- |
| `driver-cold` | 每个进程使用空`CUDA_CACHE_PATH` | 驱动首次JIT |
| `driver-warm` | 共享已预热的`CUDA_CACHE_PATH` | 驱动缓存命中 |

热模式先预热一轮，预热不记录。
每种模式采样3次。
共30次套件运行，另有5次预热。

## 环境

| 项目 | 值 |
| --- | --- |
| 平台 | Windows，x64 MSVC |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER，16376 MiB |
| 驱动 | 596.36 |
| NVRTC | 12.8（项目私有工具链12.8.1） |
| Rust | rustc 1.99.0 |
| 配置 | debug |
| 编译配方 | `--std=c++17 --gpu-architecture=compute_70`，fmad开启，fast-math关闭 |
| 基线提交 | `0f492f75285fd5d05c5b441eede0eef6ab25ee0f`加本任务未提交改动 |

测量时未独占GPU。
桌面图形进程仍在运行。
运行前已用显存约4.46 GB。
进程清单见`environment.json`。

## 结果

以下取3次采样的中位数。
括号内为最小与最大值。

### 逐套件

| 模式 | 套件 | 端到端(s) | 编译次数 | 不同源码 | 首次编译(ms) | 重复编译(ms) | 加载(ms) | 编译加检查加载占比 |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | `resident_kinematics` | 7.42 (7.32–7.46) | 84 | 3 | 255 | 6091 | 189 | 91.5% |
| cold | `resident_mass_matrix` | 4.86 (4.84–4.95) | 56 | 4 | 294 | 3018 | 181 | 74.4% |
| cold | `resident_mass_solve` | 6.97 (6.80–6.97) | 72 | 6 | 416 | 3872 | 291 | 67.9% |
| cold | `resident_rne_bias` | 15.89 (15.58–16.10) | 216 | 5 | 357 | 13934 | 290 | 95.1% |
| cold | `resident_com_velocity` | 6.83 (6.78–7.01) | 84 | 4 | 312 | 5474 | 200 | 91.1% |
| warm | `resident_kinematics` | 7.24 (7.20–7.70) | 84 | 3 | 263 | 6053 | 53 | 91.4% |
| warm | `resident_mass_matrix` | 4.65 (4.61–4.90) | 56 | 4 | 293 | 2988 | 31 | 73.7% |
| warm | `resident_mass_solve` | 6.60 (6.55–6.80) | 72 | 6 | 418 | 3797 | 39 | 66.7% |
| warm | `resident_rne_bias` | 15.50 (15.40–16.59) | 216 | 5 | 353 | 13728 | 120 | 94.9% |
| warm | `resident_com_velocity` | 6.54 (6.49–6.65) | 84 | 4 | 302 | 5406 | 47 | 91.3% |

### 合计

| 模式 | 端到端(s) | 编译(s) | 其中首次(s) | 其中重复(s) | 检查(s) | 加载(s) | 重复编译占比 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| cold | 41.96 | 34.04 | 1.63 | 32.39 | 1.27 | 1.15 | 77.2% |
| warm | 40.53 | 33.60 | 1.63 | 31.97 | 1.26 | 0.29 | 78.9% |

合计为五个套件中位数之和。
编译、检查与加载合计约占87%。
其余约13%为物理内核、传输与测试代码。

### 逐入口

8份源码对应11个入口。
同源多入口目前每入口各编一次。

| 源码摘要前缀 | 入口 | 单次编译(ms) | 冷JIT首次加载(ms) | 热首次加载(ms) | PTX字节 |
| --- | --- | ---: | ---: | ---: | ---: |
| `cd569106005e` | `attached_kinematics`、`initialize_attached` | 约85 | 57.2 | 0.57 | 55155 |
| `f9dcc93e7609` | `rigid_kinematics` | 60–98 | 42.1 | 0.95 | 30916 |
| `5e938552482b` | `rigid_com_position` | 约70 | 38.6 | 0.53 | 30115 |
| `deb8ca44e517` | `resident_mass_factor` | 约60 | 43.0 | 0.48 | 28220 |
| `6f0716ef0a43` | `resident_mass_substitute` | 约57 | 54.8 | 0.53 | 22288 |
| `569282871671` | `com_velocity` | 约47 | 18.2 | 0.44 | 20815 |
| `78239ca47c56` | `rne_bias` | 约51 | 18.7 | 0.47 | 20253 |
| `03ac3e774458` | `resident_crb_init`、`resident_crb_accumulate`、`resident_crb_matrix` | 约35 | 18.7 | 0.44 | 13660 |

单次编译取首次与重复的中位数。
两者差异很小，NVRTC本身没有缓存。
每个入口的源码摘要与PTX摘要在全部运行中各只有一个值。
完整摘要见`entries.json`。

### 显存

| 项目 | 值 |
| --- | --- |
| 单次加载前后空闲显存差最大值 | 2 MiB |
| 单进程加载差值之和最大值 | 52 MiB |
| 驱动报告的最小空闲显存 | 约15.8 GB |

2 MiB应为驱动分配粒度。
差值之和不等于常驻占用。
已卸载模块是否归还未单独测量。
Windows WDDM下空闲显存口径可能与`nvidia-smi`不同。

## 验收条款对照

| 条款 | 结果 |
| --- | --- |
| 复用当前源码与配方，不改物理结果、容差与同步语义 | 只改宿主记录代码；内核字符串、配方与容差未改；7轮共35次套件运行，每轮23项GPU测试按原容差全部通过 |
| 记录NVRTC版本、两类摘要、选项、冷启动、重复编译、加载、显存与样本数 | 全部记录于JSON行文件与汇总 |
| 区分端到端与编译加载；不以一次采样称达标；形成缓存决议 | 两项分列；每模式3次采样；决议见[结论](#结论) |
| 列出NP01至NP04覆盖与缺口 | 见下节 |

## NP01至NP04覆盖与缺口

| 指标 | 本次覆盖 | 缺口 |
| --- | --- | --- |
| NP01类型映射 | 未改任何物理类型与配方；记录实际编译选项 | 无新增类型映射证据；全路径映射仍未完成 |
| NP02精度证据 | 记录8份源码与PTX摘要 | 只覆盖5个套件用到的入口；不是全路径 |
| NP03两级数值判定 | 5个套件按原容差全部通过 | 只覆盖原生参考子集；不构成完整NP03 |
| NP04批量性能 | 记录编译、加载、端到端耗时与显存 | 未与冻结MJWarp对照；无物理步稳态性能；无Linux；无release |

缺双平台与稳态证据。
因此不得据此宣称完整精度验收。
也不得据此宣称性能达标。

## 未覆盖项

- release配置未测量。
- Linux未测量。
- L2冻结PTX加载路径未测量。
- 其余物理套件未纳入工作负载。
- GPU未独占，桌面进程同时运行。
- 模块卸载后的显存归还未测量。
- 产品长进程的真实编译频率未测量。

## 复现

先准备CUDA探针工具链：

```powershell
pwsh -NoProfile -File scripts/prepare-cuda-probe.ps1
```

再执行测量：

```powershell
pwsh -NoProfile -File scripts/measure-launcher-baseline.ps1 -Samples 3
pwsh -NoProfile -File scripts/measure-launcher-baseline.ps1 -Samples 3 -Release
```

输出写入`docs/evidence/launcher-baseline/<UTC时间>-<配置>/`。
本次输出为`20261017T193713Z-debug`。

| 文件 | 内容 |
| --- | --- |
| `environment.json` | 平台、GPU、提交、配方与模式说明 |
| `runs.json` | 30次运行的逐项结果 |
| `summary.json` | 逐模式与套件的中位数和范围 |
| `entries.json` | 逐入口摘要与耗时 |
| `records/` | 原始JSON行记录 |

## 依据

- [统一内核发射器设计](unified-launcher.md)
- [三层增量实施排程](three-layer-schedule.md)
- [精度契约](numerical-precision-contract.md)
