# G10@P5 NVRTC 产品 launcher 接线（产品 GPU 入口可用；不改数值语义）

- 任务：`2026-10-25-g10-p5-nvrtc-product-launcher`（P5 设备缺口片，补 G6 map UNC-2）
- 状态：**已完成**——产品入口经 NVRTC 运行时编译在 GPU 窗口内实际可调用；三个对照 case 与 G5 探针输出**逐字节位级一致**；错误路径有 CPU 单测；未改判据/已 verified 数字。
- 边界：不改任何数值语义、不引入 f16/TF32、不改 `Cargo.*`/fixtures/S2–S8；本片只接线产品入口，不改变设备内核表达式。

## 1. 产品入口与调用方式

```text
CollisionGpuProgram::new(0)                     // NVRTC 编译 → PTX → load_module/load_function
program.run(CollisionKernel::Hfield, &inputs)   // Contact / Hfield / Sdf 三入口
```

- `CollisionGpuProgram::new(device)`：把 `src/physics/collision/cuda/*.cu` 的 14 个设备源按依赖拓扑序拼成**一个**翻译单元（删除本地 `#include "*.cu"`，保留其它预处理），末尾追加产品入口 `product.cu`；经 cudarc NVRTC 直接 API 编译为 PTX，再 `load_module` 并取三个 `extern "C"` 入口。
- `run(kind, inputs)`：上传 23 个 host 输入数组 → 组 `#[repr(C)] CollisionArgs` → `grid=(nworld,1,1)`/`block=(1,1,1)` 发射 → 同步 → 回读；数组字段截到前 `nacon` 行（与 G5 探针 `out1/` 落盘口径一致），`nacon/nccd/overflow` 全量返回。
- **编译选项**与 G5 探针 nvcc 配方对齐：`--std=c++17`、`--gpu-architecture=sm_89`、`--fmad=true`、fast-math 关（NVRTC 默认优化级别即 -O3）。设备源拼装指纹：`len=121566`、`sha256=dc862ec884680349c6f38b40de0379082ec1b6dbf47d4dba9f22adfd8adcad43`；PTX 实测 `1,863,043` B、含 3 个入口（CPU 侧 NVRTC 冒烟，`src/runtime/collision_gpu/nvrtc_smoke.rs`）。
- **运行前置（必须写清）**：NVRTC 运行库需在 `PATH`（本仓库 `target/toolchains/cuda-12.8.1/bin` 下 `nvrtc64_120_0.dll` 等）。窗口/证据均记录 `path_prepend`、`nvcc=12.8.93`、三个 NVRTC DLL 的 sha256；**不带 PATH 时 cudarc 找不到 `nvrtc64_*.dll`，NVRTC 编译会失败/panic——这是环境依赖，不是代码缺陷**。

## 2. 三 case 对照结果（GPU 窗口，2026-10-25 21:49:22–21:49:36）

产品入口输出 vs `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/<case>/out1/`（G5 探针输出）：

| case | kind | nacon | nccd | 位级相等 | max dist ULP | max pos ULP | max frame ULP | 失败 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `hfield-sphere` | Hfield | 8 | 0 | **314/314** | 0 | 0 | 0 | 0 |
| `sdf-box` | Sdf | 80 | 0 | **3122/3122** | 0 | 0 | 0 | 0 |
| `ellipsoid-box` | Contact | 2 | 2 | **80/80** | 0 | 0 | 0 | 0 |

- 比较口径（与 G5 相同）：数组按 `out1` 口径取前 `nacon` 行，先逐字节位级比较并打印 `bit_equal`；非位级才按判据（`dist` ≤4 ULP、`pos` 按向量尺度 `4·ULP(scale)`、`frame` scale=1.0 的 4 ULP、整数/参数/`nacon`/`nccd`/`overflow` 位级）。
- **三 case 全部位级一致（0 ULP、failures=0）**，比判据要求更强；任一 case 超判据会先打印明细再 panic（本片未触发）。
- 由同一套设备函数（14 个 `.cu` 原文件）经 NVRTC 编译，说明产品入口与 G5 nvcc 探针在本次三个 case 上输出等价。

## 3. 窗口事实与资源

| 项 | 值 |
| --- | --- |
| 窗口起点 HEAD | `49e2fab5f59cf35590a14106281044006d540068`（`49e2fab5`） |
| 基线（21:49:22） | RTX 4070 Ti SUPER 16376 MiB，used **3713** / free 12349，util 0%，29°C，driver 596.36；相对申请基线 3713 **漂移 0**；本任务相关计算进程 **0** |
| PATH | `D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin` 前置；nvcc `release 12.8, V12.8.93` |
| 桌面负载 | 34 条图形上下文（WDDM 桌面）、进程 top5 `vmware-vmx:17653MiB, DeepSeek Harness:1670MiB, Code:1043MiB, MsMpEng:746MiB, explorer:668MiB`；CPU 9950X3D 16C/32T |
| 峰值 | used **3898 MiB（+185）**；守卫 3072 MiB/50% **未触发**；0.5 s 采样 25 点，util max 3%，temp max 30°C |
| 结束基线（21:49:36） | used **3713**（回落基线）；跑前跑后本任务相关计算进程 **0**（无 test/probe/python 残留） |
| elapsed | 15.2 s（其中 NVRTC 编译为 CPU 侧 ~11 s；三个 case 的 GPU 发射/回读为毫秒级） |

## 4. 错误路径（CPU 单测，无需 GPU）

`src/runtime/collision_gpu/tests.rs`（12 个纯 CPU 测试，用 seam 注入，不创建真实 `CudaContext`）：

- NVRTC 编译失败：`compile_source_with` 注入返回 `Err` 的编译闭包 → `CollisionGpuError::Compile`（带 `source_sha256`）；编译期 panic 被 `catch_unwind` 兜底 → `Compile("NVRTC编译异常")`，**不 panic**；
- 无可用 GPU：`program_from_parts(device Err, ..)` → `CollisionGpuError::NoDevice`；且 PTX 错误优先，编译失败时根本不碰设备；
- 参数非法：`InvalidArgument`（`nworld/ngeom/naconmax/efc_width` 非正、`g1/g2` 越界、基础数组长度不匹配）；
- ABI 锁死：`#[repr(C)] CollisionArgs` size=496、`CollisionContactOut` size=136，55/17 字段偏移与 probe.cu/contact.cu 的 C 结构逐项一致（64 位）；
- 源拼装：14 源依赖序、只删本地 `.cu` include、确定性 sha256；
- 生产编译冒烟（ignored，仅需 NVRTC、不需 GPU）：`nvrtc_smoke` → PTX 含三入口。

## 5. 证据边界（防误读，Arch 2026-10-25 要求）

1. **特性门控**：`src/runtime/collision_gpu` 整个模块在 `#[cfg(feature = "cuda-probe")]` 下（Cargo 当前只有这一个 CUDA 特性）。因此 **默认构建不含 CUDA 产品入口**；产品入口经 `cuda-probe` 特性启用。**未覆盖：尚未把该入口接入默认产品管线**（默认构建/无 CUDA 环境不可调用），列为后续项。
2. **入口胶水复制范围**：`product.cu` 只从 G5 探针复制**入口胶水**——`mjwarp_collision_args` 结构、`LocalPolytope`/`init_polytope`/`build_geom`/`build_params`/`sdf_type_of` 辅助函数与三个 `__global__` 调度壳；**不复制任何数值内核**（GJK/EPA/CCD/SDF/HFIELD/接触写入等数值函数全部来自 `src/physics/collision/cuda/*.cu` 原文件，由 NVRTC 直接编译）。入口胶水等价性由 §2 的三 case 位级对照证明。

## 6. 未覆盖 / 限制

- 默认产品管线接入（见 §5.1）；只有 `cuda-probe` 特性下可用。
- 对照仅 3 个 case（任务要求下限）：`hfield-sphere`/`sdf-box`/`ellipsoid-box`；multicontact、primitive narrowphase 设备版仍未实装（G6 UNC-1/UNC-3），不在本片范围。
- 输入校验只覆盖基础数组长度；mesh/hfield/octree 长数组长度依赖 `nmesh/noct`（meta 无该字段）未校验。
- NVRTC 默认优化级别即 -O3，未显式传 `--opt-level`；本片三 case 位级一致，未做跨架构/跨 NVRTC 版本等价声明。
- 仅 Windows + RTX 4070 Ti SUPER + NVRTC 12.8 实测；不跨硬件外推。
- 不改判据/已 verified 数字；本片不提供精度结论、等价声明或放行信号。

## 7. 复现命令

```text
# 0) 窗口外：预构建 + CPU 侧验证（NVRTC 需 PATH）
$env:PATH = "D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;" + $env:PATH
cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored
cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher --no-run
# 1) 获批窗口内（脚本内含基线/守卫/0.5 s 采样/PATH/无残留检查）
powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\g10-p5-nvrtc-product-launcher\run_window.ps1
```

## 8. 文件

| 文件 | 内容 |
| --- | --- |
| `README.md` / `receipt.json` | 本证据（task_id、方法与结论） |
| `window-log.txt` / `window-output.txt` / `window-memory-samples.txt` | 窗口基线/守卫/采样/测试原始输出（含 HEAD、PATH、NVRTC DLL sha256、三 case 摘要） |
| `run_window.ps1` | 窗口执行脚本（authoring-only） |
| `artifacts-sha256.txt` | 源码/证据/测试 10 项 sha256 |
- 产品代码：`src/runtime/collision_gpu.rs` + `src/runtime/collision_gpu/{source,error,abi,tests,nvrtc_smoke}.rs`、`src/physics/collision/cuda/product.cu`、`tests/g10_p5_nvrtc_launcher.rs`；`src/runtime/mod.rs`（+3）、`src/physics/mod.rs`（+7，设备源 `pub(crate) use`）仅接线。
