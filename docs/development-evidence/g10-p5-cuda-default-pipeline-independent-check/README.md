# G10@P5 CUDA 默认管线接线 · 阶段 1（决策支撑）· QA 独立复核

- 任务：`2026-10-25-g10-p5-cuda-default-pipeline-independent-check`（p56，QA）。
- 被复核片：`2026-10-25-g10-p5-cuda-default-pipeline`（阶段 1，决策支撑），交付提交 `dea64642`（`234e7255` 为 receipt 去重）。
- 复核 HEAD：**`234e7255`**。`git diff ef16338d..234e7255 -- src tests Cargo.toml Cargo.lock fixtures`
  **为空**（仅 docs/evidence 与 `.normify`），因此本复测与交付测量**同源码树**。
- 交付证据：`docs/development-evidence/g10-p5-cuda-default-pipeline/{DESIGN.md, measurements.json, receipt.json}`。
- 本复核只覆盖**阶段 1**；阶段 2 未发生（交付 `receipt.stage2_status=not_started`、`stage2_task` 记录
  `status=proposed`），本复核未实施/未改任何产品代码与 Cargo 默认特性。
- 无 GPU 窗口（本任务不需要）；未改交付方已 verified 证据。

## 1. 结论

**pass（阻断 0 / 发现 1 / 观察 5）**，`normify gate` + `normify check` 全绿。

- **三选项的构建/测试影响独立复测成立**：14 项度量为 **13/14 在 ±10% 内**，全部体积数字**逐项相同**
  （确定性产物）；唯一超出的是 debug probe build `+16.4%`（19.9 s vs 交付 17.1 s），同配置另一次
  独立 clean build 为 **17.2 s**，属 15% 级 run-to-run 噪声，**量级无差异**。
- **无 CUDA 机器可构建性成立（构建期）**：本 shell 原始环境即无 `nvcc` 可发现、无 CUDA* 变量；
  默认 build / `--features cuda-probe` build / 默认 `cargo test --lib` 均 exit 0、447 passed / 5 ignored。
  **运行期「无 GPU/无驱动」不可在本机实机模拟**（本机有 RTX 4070 Ti SUPER + driver 596.36），见观察 4。
- **阶段 1 口径正确**：未改默认特性/产品代码；`Cargo.toml default = []` 与 `cuda-probe` 依赖集合与
  DESIGN §1 一致；无产品代码调用 `CollisionGpuProgram`（仅 `#[ignore]` 集成测试）；未把「阶段 1 决策
  支撑」写「默认管线已具备 CUDA 入口」（DESIGN 明确「默认构建仍无 CUDA 入口」「候选非默认生效」）。
- **门禁盲区记录成立**：DESIGN §6.4 写明默认 `normify gate` 的 unit 只跑默认特性、`--features
  cuda-probe --lib` 677 项不在门禁内；我在 HEAD 复跑该命令 = **677 passed / 1 failed / 137 ignored**，
  失败为已知既有红（T4 陈旧 fixture，`2026-10-25-t4-source-hash-repin-13` 在修），**未改、未重钉**。
- **发现 1（需阶段 2 设计注意）**：DESIGN §4.1 三态表把「无驱动库」态的**产品 API panic** 判为「否」，
  与代码不符 —— `CollisionGpuProgram::new` 的设备侧调用**没有** catch_unwind，而 cudarc 在
  `libcuda/nvcuda` 缺失时会 `panic!`（详见 §5）。阶段 2 的「探测失败 → Host 回退、不 panic」必须先做
  `is_culib_present()` 预检或包 `catch_unwind`（CLI 探针已经是这么做的）。

## 2. 复核方法（独立测量，不复用交付数字）

- 独立 `CARGO_TARGET_DIR`（`target/qa-default-pipeline/<cfg>`），每次测量**先删目录再 clean build**，
  同 HEAD、同机、串行单次测量；体积口径 = target 目录全部文件字节和（MiB=1048576 B）+ `mjwarp-rs.exe`。
- 选项①/② 的默认构建代理 = `--features cuda-probe`（阶段 1 不改默认特性），数字标注为代理。
- 无 CUDA 模拟：记录 PATH 中 cuda 项/`CUDA*` 环境变量的清除结果（本次 0 项/0 个）、`nvcc` 可发现性，
  再用独立 target 复测默认 build / 默认 `cargo test --lib` / 特性 build / 特性测试 `--no-run`。
- 判据：与交付 `measurements.json` 逐项列差异（秒 / MiB / exit / 测试计数），允许 ±10% 波动，
  量级不得差一个数量级；体积（确定性）要求逐项相同。
- 原始日志：`measure-log-summary.txt`（命令/耗时/exit/结果摘要；完整构建输出在
  `target/qa-default-pipeline/measure.log`，不入库、可由 `measure.ps1` 重建）。

## 3. 独立测量结果 vs 交付

| 指标 | 本复核 | 交付 | 差异 |
| --- | --- | --- | --- |
| debug 默认 build 耗时 | **6.2 s** | 6.0 s | +3.3% |
| debug 默认 target | **195.2 MiB** | 195.2 MiB | 0 |
| debug 默认 exe | **19.62 MiB** | 19.6 MiB | +0.1% |
| debug probe build 耗时 | **19.9 s** | 17.1 s | **+16.4%**（同配置另一次 clean = 17.2 s） |
| debug probe target | **775.5 MiB** | 775.5 MiB | 0 |
| debug probe exe | **77.92 MiB** | 77.9 MiB | +0.03% |
| debug 默认 `test --lib --no-run` | **13.5 s** / 534.4 MiB | 13.5 s / 534.4 MiB | 0 / 0 |
| debug probe `test --lib --no-run` | **21.1 s** / 1295.9 MiB | 20.3 s / 1295.9 MiB | +3.9% / 0 |
| 默认 `cargo test --lib`（运行） | **447 passed / 5 ignored / 0 failed**，0.4 s | 447 / 5 / 0，0.5 s | 计数相同 |
| 特性 `cargo test --lib`（运行） | **677 passed / 1 failed / 137 ignored**，exit 101 | 同 | 已知既有红 |
| release 默认 build 耗时 | **5.0 s** | 5.2 s | −3.8% |
| release 默认 target / exe | **30.1 MiB / 2.46 MiB** | 30.1 / 2.5 MiB | 0 / −1.6% |
| release probe build 耗时 | **17.7 s** | 17.5 s | +1.1% |
| release probe target / exe | **167.8 MiB / 8.88 MiB** | 167.8 / 8.9 MiB | 0 / −0.2% |

**独立增量（默认 → 特性代理）**：debug `+13.7 s`（交付 +11.1 s）、target `+580.3 MiB`（同交付）、
exe `+58.30 MiB`（交付 +58.3）；debug test-norun `+7.6 s`（交付 +6.8）；
release `+12.7 s`（交付 +12.3）、target `+137.7 MiB`（同）、exe `+6.42 MiB`（交付 +6.4）。
**判定**：13/14 在 ±10% 内；唯一越界项属 run-to-run 噪声；结论「①② 默认构建增量约 +11–14 s、
debug target +580 MiB、release exe +6.4 MiB、构建期不需要 CUDA toolkit」**独立成立**。

## 4. 无 CUDA 机器可构建性（模拟）

- 环境事实：`Get-Command nvcc` **失败**（工具链 bin 不在本进程 PATH）、PATH 中匹配 cuda 的项 **0 条**、
  `CUDA*` 环境变量 **0 个**（与交付 `path_stripped_has_cuda_entries=0` 同一事实）。
  → 本复核的「清除」步骤实际未改变环境；它验证的是**「构建期不可发现 CUDA toolkit 的 shell 下可 clean
  build」**，不是「从有到无」的严格对照（观察 4）。
- 独立 target 复测：
  - 默认 `cargo build --locked`：**exit 0，6.0 s，195.2 MiB**；
  - 默认 `cargo test --locked --lib`：**exit 0，13.5 s，447 passed / 5 ignored / 0 failed**（可跑）；
  - `--features cuda-probe` `cargo build --locked`：**exit 0，17.2 s，775.5 MiB**（cudarc 用
    `dynamic-loading`，构建期不需要 toolkit）；
  - `--features cuda-probe --lib --no-run`：**exit 0，20.8 s**（测试二进制可编）。
- **模拟边界**：本机有驱动+设备，运行期「无驱动/无设备/无 NVRTC」未实机模拟；三态表运行期部分只能
  代码核对（§5），阶段 2 需在无 GPU 机器/CI 补实测。

## 5. 三态回退语义核对（DESIGN §4 vs 现有代码）

**一致的（产品 API 错误映射）**：`new()` 先编译后建上下文（`compile_program()?` 在
`CudaContext::new` 之前，`src/runtime/collision_gpu.rs:115-119`），故「NVRTC 编译失败」优先返回
`Compile`、「设备不可用」返回 `NoDevice`；错误文案与 `error.rs` 逐字一致；CPU seam 单测
`constructor_seam_maps_device_and_compile_errors` / `compile_failures_are_mapped_without_panicking`
（`src/runtime/collision_gpu/tests.rs:463-499`，均默认特性内）覆盖映射与「编译期 panic 不冒泡」。
CLI 默认特性路径经实测：默认 build 的 `mjwarp-rs.exe probe`/`resources` → stderr
**「请启用cuda-probe功能」**、exit 1（`ProbeError::FeatureDisabled`，不 panic）。

**发现 1 —— 无驱动库态（A）产品 API 会 panic，DESIGN 判「否」不成立**：
`CudaContext::new` → `result::init()` → `sys::cuInit` → cudarc `culib()`；`culib()` 在
`cuda`/`nvcuda` 都不存在时调用 `panic_no_lib_found` **panic**（cudarc 0.18.2
`driver/sys/mod.rs:14987-14989, 26712-26729`、`lib.rs:180-181`）。`new()` 的设备侧调用**没有**
`catch_unwind`，所以「NVRTC 可用、驱动库缺失」时 panic 会直接冒到调用方，而不是 `NoDevice`。
CLI 探针不受影响（`src/runtime/cuda.rs:110-115` 先 `driver::sys::is_culib_present()` →
`DriverUnavailable`；`src/runtime/run.rs:110-115` 再包 `catch_unwind` → `BackendPanic`）。
**阶段 2 含义**：要实现「探测失败 → Host 回退、不 panic」，必须预检 `is_culib_present()` 或包
`catch_unwind`；不能只匹配 `CollisionGpuError`。

**观察 1 —— 三态表 CLI 细节与代码不符（不改变「三态可回退」的结论）**：
- B（有驱动无设备）：实际 `ProbeError::InvalidDevice{requested:0, available:0}`
  （`src/runtime/cuda.rs:124-138`），表里写 `Cuda`/`UnsupportedDevice`；`UnsupportedDevice` 需先有设备且 SM<7。
- C（有设备无 NVRTC）：默认 `--backend native-ptx` 探针用**编译期内嵌 PTX**（`Ptx::from_src(include_str!("probe.ptx"))`，
  `src/runtime/cuda.rs:155-170`），不加载 NVRTC → C 态不会报 `NvrtcUnavailable`；该错误只在
  `cubecl-cpp` 路径由 `cudarc::nvrtc::sys::is_culib_present()` 产生（`src/runtime/cubecl.rs:250-257`）。
- C 态产品 API 文案：NVRTC DLL 缺失时 cudarc 在加载处 panic，`compile_source_with` 映射为固定
  `Compile("NVRTC编译异常")`（Display：`NVRTC编译失败：NVRTC编译异常`，**不含 source_sha256**）；
  只有非 panic 编译错误才带指纹（`src/runtime/collision_gpu.rs:325-336, 367-372`）。表里 C 行文案
  写的是带 `source_sha256` 的形式。
- 无 `unwrap/expect` 扫描：产品运行时/CLI 非测试代码无 `unwrap/expect`，唯一 `expect` 在
  `#[ignore]` 测试 `nvrtc_smoke.rs:13`（与 DESIGN §1.6 一致）——但如发现 1，cudarc 内部 panic 不在此扫描范围。

## 6. 门禁盲区核对（DESIGN §6.4）

- DESIGN §6.4（L166-170）已写明：默认 `normify gate` 的 unit 只跑默认特性、`--features cuda-probe --lib`
  的 677 项不在门禁内、漂移潜伏约 2 天、建议后续纳入批次门禁但本片不实施。**成立**。
- 我的独立复跑（HEAD `234e7255`）：`cargo test --locked --features cuda-probe --lib` →
  **677 passed / 1 failed / 137 ignored**（0.49 s），失败 =
  `runtime::transfer::kernel::recapture::tests_host::preserves_current_sources_and_all_thirty_variants`
  → `unexpected lineage change`；默认特性同命令 = **447 passed / 5 ignored / 0 failed**。
  该红是已知既有红（T4 陈旧 fixture，`2026-10-25-t4-source-hash-repin-13` 在修），**本复核未改、未重钉**。
- **观察 2**：DESIGN §1.5/§6.1 对门禁/脚本的描述口径不准：`.normify/gates.toml` 的 unit 是
  `cargo test --locked`（不是 `--lib`）；`scripts/test-windows-g01.ps1` 主矩阵默认特性是
  `--features native-model-probe,cuda-probe`（L40-41；只有 `host-gnu` 子检查 L264 是 plain
  `cargo test --locked`），`scripts/g01-fast.ps1` 同（L42）。盲区结论不变。

## 7. 阶段 1 边界与口径

- `git diff ef16338d..234e7255 -- src tests Cargo.toml Cargo.lock fixtures` 为空；`Cargo.toml` 仍
  `default = []`、`cuda-probe` 依赖集合与 DESIGN §1.1 一致；无产品代码调用 `CollisionGpuProgram`
  （仅 3 个 `#[ignore]` 集成测试与 support）；未开 GPU 窗口、未新捕获参考。
- 交付 receipt：`stage2_status=not_started`、`stage2_task=2026-10-25-g10-p5-cuda-default-pipeline-stage2`
  （记录 `status=proposed`）；DESIGN 明确「默认构建仍无 CUDA 入口」「选项 ③ 单独不关闭 UNC-2」
  「候选、非默认生效」；未发现把阶段 1 写成「默认管线已具备 CUDA 入口」或「默认生效」的表述。
- 交付 gate 记录（`.normify/runs/2026-10-25-g10-p5-cuda-default-pipeline/`）fmt/clippy/clippy-cuda 通过、
  unit 595/0；与交付 receipt/gate 段一致。

## 8. 观察项（均非阻断）

1. **三态表 CLI 错误码细节**（§5）：B 实际 `InvalidDevice`、C 默认探针不查 NVRTC、C 产品文案 panic
   分支不含 `source_sha256`。不改变「三态可构建/可回退」结论，但建议按代码修正表内映射。
2. **DESIGN §1.5/§6.1 门禁与脚本描述**（§6）：`gates.toml` unit = `cargo test --locked`；scripts 默认带
   `native-model-probe,cuda-probe` 特性。盲区结论（normify gate 不含特性测试）不变。
3. **无 CUDA 模拟边界**（§4）：PATH cuda 项/`CUDA*` 变量本来就不存在（清除 0 项），`nvcc` 不可用；
   验证的是「构建期不可发现 toolkit」，运行期无驱动/无设备未实机模拟。
4. **交付 `measurements.json` 的 `measured_at_local` 为 `2026-10-26T00:0x+08:00` 占位**（非精确时间）；
   不影响数字，但建议以后写完整时间戳。
5. **阶段 2 的测试前置**：`--features cuda-probe --lib` 当前 1 failed 既有红（T4 fixture）未清零；
   阶段 2 若要求「默认 cargo test 绿」需先等 `repin-13` verified（交付 DESIGN §6.3 已写明）。

## 9. 边界与未覆盖

- 本复核只做**阶段 1 独立复核**：未实施选项①/②/③、未改默认特性、未写选择层、未开 GPU 窗口；
  阶段 2（运行时选择层/Host 回退/9 case host-vs-device）不在本片。
- 体积/编时为单次 clean 测量（确定性体积逐项相同；耗时受同机负载影响，允许 ±10%，跨机器不可外推）。
- 运行期三态（无驱动/无设备/无 NVRTC）无实机模拟；发现 1 与观察 1 为源码 + cudarc 源码推导，
  已标注「不可实机模拟」。
- 未测性能（本片为依赖/CI 决策）；未改交付方已 verified 证据（`dea64642`/`234e7255`）。

## 10. 可复现路径与命令

- 测量脚本：`measure.ps1`（独立 `CARGO_TARGET_DIR=target/qa-default-pipeline/<cfg>`，先删目录、
  记录命令/耗时/exit/target 体积/exe 体积；含 no-CUDA 环境记录；原始日志在
  `target/qa-default-pipeline/measure.log`）。
- 对照脚本：`compare_measurements.py`（解析原始日志 + 交付 `measurements.json` → `measurements-self.json`
  的逐项差异表；体积用目录求和复核）。
- 结果：`measurements-self.json`（逐项对照 + 独立增量 + 判定）、`measure-log-summary.txt`（摘要）。
- 代码核对：`qa-code-check.json`（发现 1 + 观察 1/2 + verified claims + 门禁盲区 + panic 扫描）。
- 环境：cargo/rustc 1.99.0；RTX 4070 Ti SUPER/driver 596.36；HEAD `234e7255`。
