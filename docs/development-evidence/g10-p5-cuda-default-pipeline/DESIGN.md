# G10@P5 CUDA 默认管线接线 · 阶段 1 设计稿（决策支撑）

- 任务：`2026-10-25-g10-p5-cuda-default-pipeline`（p56）
- 阶段：**阶段 1（只做决策支撑与实测）**；**不写产品代码、不改 Cargo 默认特性、不开 GPU 窗口、不新捕获参考**
- HEAD：`ef16338dccc5f54adc54102784e28d24e960e569`
- 本文所有结论都是**候选与影响面**，不代表任何选项已默认生效；阶段 2 需用户拍板后才实施。

## 0. 结论速览（三选项）

| 维度 | ① cuda-probe 转默认 | ② 新 `cuda` 特性默认开启 | ③ 保持门控 + 运行时选择层/Host 回退 |
| --- | --- | --- | --- |
| 默认构建含 cudarc | 是 | 是 | **否** |
| 默认构建耗时增量（debug，clean） | +11.1 s（6.0→17.1 s，+185%） | 同① | **0** |
| 默认构建体积增量（debug target） | +580.3 MiB（195.2→775.5，+297%） | 同① | **0** |
| release 增量 | +12.3 s（5.2→17.5 s，+237%）、target +137.7 MiB、exe +6.4 MiB | 同① | **0** |
| `cargo test --lib` 增量 | 编译 +6.8 s；运行结果见 §6.3（HEAD 上特性测试已红，需先修） | 同① | **0** |
| 默认行为是否变化 | 是（依赖/CLI probe/二进制体积） | 是（同①，但可 `--no-default-features` 退回现状） | **否**（仅新增回退层，默认碰撞仍 Host） |
| 无 CUDA 机器可构建 | 可（实测，构建期不需要 CUDA toolkit） | 可 | 可（现状） |
| 是否让默认构建具备 CUDA 入口（UNC-2） | 是（依赖具备；功能仍需阶段 2 接线） | 是（同①） | **否**（保持门控；要具备仍需另开特性） |
| 可选退出 | 无 | `--no-default-features` | 特性默认关闭 |
| 主要代价 | 所有默认构建都编 cudarc，无退出开关 | 同①但保留退出开关 | 不解决 UNC-2；多一层选择/回退代码 |

**一句话**：①②解决“默认构建里有 CUDA 入口”的依赖前提（增量 ~11–12 s 构建、~580 MiB debug target，构建期不需要 CUDA toolkit）；③ 只解决“选择层与 Host 回退语义”，默认构建仍无 CUDA 入口，**单独不关闭 UNC-2**。两者都还需要阶段 2 的运行时接线（见 §8）。

## 1. 现状事实（HEAD `ef16338d`，实测/代码核对）

1. `Cargo.toml`：`default = []`；`cuda-probe = ["dep:cudarc","dep:libloading","dep:serde","dep:serde_json","dep:sha2"]`；
   `serde/serde_json/sha2` 同时是 dev-dependencies（测试构建本就会编译）；`cudarc` 版本 `0.18.2`，
   `default-features = false`，features 为 `std,driver,nvrtc,dynamic-loading,cuda-12000`。
2. 默认构建**不编译**任何 CUDA 依赖；`src/runtime/collision_gpu` 与 `src/physics/collision/cuda` 都在
   `#[cfg(feature = "cuda-probe")]` 下。
3. 默认构建的碰撞路径 = **host 函数**（`src/physics/collision.rs` 的 `ccd/ccd_pair/primitive_narrowphase`
   等，无特性门控）；默认 CLI（`src/main.rs`）的 `probe` 走 `ProbeBackend::NativePtx`，未启用特性时
   返回 `ProbeError::FeatureDisabled("cuda-probe")`（文案：`请启用cuda-probe功能`）。
4. **当前没有任何产品代码路径调用 `CollisionGpuProgram`**（仅 3 个 ignored 集成测试使用）。
   因此“默认产品管线有 CUDA 入口”不是仅改依赖就能达成，三选项都还需要阶段 2 的运行时选择层与接线。
5. 门禁：`normify gate` 的 unit 跑**默认特性** `cargo test --lib`（HEAD 上 595 过/0 败）；
   仓库无 `.github` CI 配置；构建/测试矩阵由 `scripts/test-windows-g01.ps1`（默认 `cargo test --locked`、
   clippy 默认与 `--features cuda-probe` 两遍）与 `scripts/g01-fast.ps1` 等脚本承担。
6. 错误码现状（阶段 2 回退语义的素材）：
   - 产品 API`CollisionGpuError`：`NoDevice("CUDA设备不可用：…")`、`Compile("NVRTC编译失败：…")`、
     `Load("设备模块加载失败：…")`、`Launch("内核执行失败：…")`、`InvalidArgument("输入参数无效：…")`；
   - CLI 探针 `ProbeError`：`FeatureDisabled`（`请启用{feature}功能`）、`NvrtcUnavailable`
     （`找不到NVRTC库；请检查PATH`）、`DriverUnavailable`（`找不到NVIDIA驱动库`）、
     `Cuda{stage,code}`（`CUDA失败：{stage}；错误码{code}`）、`BackendPanic`
     （`GPU适配层异常；未回退CPU`）、`UnsupportedDevice`（`探针需要SM 7.0；当前SM {major}.{minor}`）、
     `UnsupportedDriver`（`探针需要CUDA 12.0；当前API版本{version}`）。
   - CLI 错误退出码：`ExitCode::FAILURE`（Windows 进程退出码 1）；库/CLI 错误路径不 panic
     （`.expect()` 只出现在 ignored 测试里）。

## 2. 度量方法（可复现口径）

- 机器：AMD Ryzen 9 9950X3D 16C/32T；RTX 4070 Ti SUPER；driver 596.36；cargo/rustc **1.99.0**；
  HEAD `ef16338d`；测量时段无其他 cargo/GPU 负载（串行测量）。
- **clean build**：每次用独立 `CARGO_TARGET_DIR=target/measure/<cfg>`，先删除目录再构建；
  同一 HEAD、同一机器、串行执行（避免 CPU/磁盘争用）。
- 体积口径：target 目录全部文件字节和（MiB = 1048576 B）与 `debug|release/mjwarp-rs.exe` 体积。
- **选项①/② 的默认构建代理**：阶段 1 禁止改 `Cargo.toml` 默认特性，故用
  `--features cuda-probe` 的 clean build 作为“默认构建包含 CUDA 依赖”的实测代理
  （特性开启后的默认构建与同参数构建等价）。数字标注为代理。
- 无 CUDA 机器可构建性：从 `PATH` 移除全部含 cuda 的项、删除全部 `CUDA*` 环境变量后，用独立
  `CARGO_TARGET_DIR` 重新 clean build。**模拟边界**：本机磁盘上仍有 CUDA toolkit 文件与驱动，
  无法实机模拟“无 GPU/无驱动运行期”；该部分见 §4 并标注为代码/证据推导。
- 原始数据：`measurements.json`（含每条命令原文、耗时、体积、exit code）。

## 3. 实测结果

### 3.1 debug `cargo build`

| 配置 | clean 耗时 | target 体积 | `mjwarp-rs.exe` |
| --- | --- | --- | --- |
| 默认（现状基线） | **6.0 s** | **195.2 MiB** | **19.6 MiB** |
| `--features cuda-probe`（①② 默认构建代理） | **17.1 s** | **775.5 MiB** | **77.9 MiB** |
| 增量 | **+11.1 s（+185%）** | **+580.3 MiB（+297%）** | **+58.3 MiB** |

### 3.2 `cargo test --lib`

| 配置 | `--no-run` 编译耗时 | target 体积 | 运行结果 |
| --- | --- | --- | --- |
| 默认 | **13.5 s** | 534.4 MiB | **447 passed / 5 ignored**（452 tests，0.5 s） |
| `--features cuda-probe` | **20.3 s** | 1295.9 MiB | **677 passed / 1 failed / 137 ignored**（815 tests，0.5 s） |
| 增量 | **+6.8 s** | +761.5 MiB | 运行失败项见 §6.3（**与选项无关的既有红**） |

### 3.3 release `cargo build --release`

| 配置 | clean 耗时 | target 体积 | release exe |
| --- | --- | --- | --- |
| 默认 | **5.2 s** | **30.1 MiB** | **2.5 MiB** |
| `--features cuda-probe` | **17.5 s** | **167.8 MiB** | **8.9 MiB** |
| 增量 | **+12.3 s（+237%）** | **+137.7 MiB（+457%）** | **+6.4 MiB** |

### 3.4 无 CUDA 机器可构建性（模拟）

| 配置 | 结果 | 耗时 | 说明 |
| --- | --- | --- | --- |
| 默认 build（PATH/CUDA* 清除） | **exit 0** | 6.1 s | 原本就不依赖 CUDA |
| `--features cuda-probe` build（同环境） | **exit 0** | 16.6 s | `cudarc` 用 `dynamic-loading`：driver/NVRTC 只在运行期加载，构建期不需要 toolkit |
| 默认 **`cargo test --lib`**（同环境，独立 target） | **exit 0** | **13.3 s** | **447 passed / 5 ignored / 0 failed**（452 tests，0.25 s）→ 无 CUDA 环境**测试可跑** |
| `--features cuda-probe` 测试编译（同环境，`--no-run`） | **exit 0** | 25.7 s | 测试二进制可编；**运行期无 GPU/无驱动不可在本机模拟**（见边界） |

**边界**：本机无法实机模拟“无驱动/无设备/无 NVRTC”的运行期；§4 的三态表由代码路径与既有证据推导
（NVRTC DLL 缺失时 cudarc 动态加载失败已有证据）。

## 4. 三态回退语义（逐选项）

三态定义：**A** = 无 CUDA 驱动库（`libcuda` 不可用/未装驱动）；**B** = 有驱动但无可用设备；
**C** = 有设备但 NVRTC DLL 缺失。“现状”= 阶段 1 未实现选择层；“阶段 2 目标”= 拍板后要实现的语义。

### 4.1 选项 ①（cuda-probe 转默认特性）

| 环境 | 编译期（默认构建） | 运行期默认碰撞路径（阶段 2 前） | 若走设备入口的错误（code/文案） | panic | 阶段 2 目标回退 |
| --- | --- | --- | --- | --- | --- |
| A 无驱动 | 可构建（实测，cudarc 动态加载） | Host（无产品调用方） | `CollisionGpuError::NoDevice`：`CUDA设备不可用：…`；CLI 探针：`ProbeError::DriverUnavailable`（`找不到NVIDIA驱动库`） | 否（测试 `.expect()` 仅 ignored 测试） | 设备探测失败 → **Host 回退** + 记录可诊断错误（不静默、不 panic） |
| B 有驱动无设备 | 可构建 | Host | `NoDevice`（`CudaContext::new` 失败）；CLI：`ProbeError::Cuda`/`UnsupportedDevice` | 否 | 同上 |
| C 有设备无 NVRTC | 可构建 | Host | `CollisionGpuError::Compile`：`NVRTC编译失败：source_sha256=…：…`（`new()` 先编译再建上下文）；CLI：`ProbeError::NvrtcUnavailable`（`找不到NVRTC库；请检查PATH`） | 否 | 同上，错误码保留可诊断 |

### 4.2 选项 ②（新增 `cuda` 特性默认开启）

- 默认构建行同 4.1（三态与错误码一致）；`--no-default-features` 时退化为**现状**：
  默认构建无 CUDA 代码，三态下运行期都只走 Host，无错误码、无 panic、无回退开销。
- 阶段 2 目标回退同 4.1。

### 4.3 选项 ③（保持门控 + 运行时选择层与 Host 回退）

| 环境 | 编译期（默认构建） | 运行期默认碰撞路径 | 错误码/文案 | panic | 回退 |
| --- | --- | --- | --- | --- | --- |
| A 无驱动 | 可构建（不编 CUDA 依赖） | **Host（始终）** | 无（不尝试设备） | 否 | 不需要（无设备尝试） |
| B 有驱动无设备 | 可构建 | **Host（始终）** | 无 | 否 | 不需要 |
| C 有设备无 NVRTC | 可构建 | **Host（始终）** | 无 | 否 | 不需要 |

- 若用 `--features cuda-probe` 构建：三态行为与 4.1 一致（选择层 + Host 回退）。
- **诚实说明（Arch 要求）**：选项 ③ 相对现状多接的是“运行时后端选择层 + Host 回退 + 统一错误码/
  可诊断状态”，但**默认构建仍没有 CUDA 入口**；要达成 UNC-2 的“默认产品管线有 CUDA 入口”，
  或另把特性默认打开（即回到 ①②），或在发行构建中显式启用特性。③ 单独**不关闭 UNC-2**。

## 5. 每个选项对默认行为的影响

| 维度 | ① | ② | ③ |
| --- | --- | --- | --- |
| 默认构建依赖 | +cudarc/libloading（+serde/sha2/serde_json 从 dev 变普通） | 同① | 无变化 |
| 默认 `cargo run -- probe` | `native-ptx` 后端默认可用 → 会尝试 CUDA（错误码从 `FeatureDisabled` 变 `DriverUnavailable`/`NvrtcUnavailable`/`Cuda`） | 同①；`--no-default-features` 时保持现状 | 现状（`FeatureDisabled`） |
| 库 API | 无特性门控的 cuda 模块可用 | 同① | 仅选择层；GPU API 仍门控 |
| 默认碰撞数值/性能 | 阶段 2 接线前无变化（仍 Host）；接线后 Host 与设备按后端选择，回退时 Host 逐值一致 | 同① | 阶段 2 后默认仍 Host（除非构建启用特性） |
| 二进制体积 | debug +58.3 MiB / release +6.4 MiB | 同① | 无变化 |

## 6. CI / 门禁影响

1. 仓库**无 `.github` CI**；当前门禁 = `normify gate`（默认特性 unit）+ `scripts/test-windows-g01.ps1`
   （默认 `cargo test --locked`、clippy 默认与 `--features cuda-probe` 两遍）+ `scripts/g01-fast.ps1`。
2. 选项 ①/② 会让**所有**默认构建都编 cudarc：构建时间与磁盘上升（§3），但**不需要 CUDA toolkit**
   （§3.4 实测），无 GPU 的 CI 机器可构建。
3. **⚠ 既有红（与本片选项无关，但会阻塞 ①/② 的“默认 cargo test 绿”）**：
   HEAD `ef16338d` 上 `cargo test --locked --features cuda-probe --lib` 已失败（677 passed / 1 failed /
   137 ignored）：`runtime::transfer::kernel::recapture::tests_host::preserves_current_sources_and_all_thirty_variants`
   → `unexpected lineage change`。Arch 独立复算（LF 归一化逐条 sha256）：`fixtures/precision/t4-current-kernels.json`
   的 `product_sources` 20 条中 **9 条**与当前源码不一致（attached / com_velocity / mass_matrix / mass_solve /
   `src/physics/mod.rs` / resident / rne_bias / `src/runtime/transfer.rs` / `src/runtime/transfer/kernel.rs`）；
   其中 `src/physics/mod.rs` 的记录值 `d7f6c7d1…` 与锚点
   `docs/development-evidence/t4-resident-anchors/physics.rs` 的当前摘要 `b4bb7ed9…`、以及 live
   `src/physics/mod.rs` 的 `f840ddfe…` 都不一致。上次重钉 `e4f74816`（2026-10-23）之后多片继续改动这些源码而未重钉；
   默认 gate 只跑默认特性 unit、不编译该特性测试，所以红了约 2 天未被发现。**不是** `source_revision` 字面量问题：
   `REVISION` 常量与 fixture 的 `source_revision` 都是 `8a5bac00…`；`934667a8…` 出现在
   `docs/evidence/t4-g03-retest/rigid-rotate-recapture/previous-kernels.json`（PRE_ROTATE 基线）与
   `t4-current-capture-status.json` 的 `base_revision`，与 fixture 的 `source_revision` 无关。修法：独立小片
   `2026-10-25-t4-source-hash-repin-13`（已登记 `171daba7`，按 repin-12 模式先枚举后重钉）。
4. **门禁盲区（记录，不在本片修）**：默认 `normify gate` 的 unit 只跑默认特性 `cargo test --lib`
   （447 passed / 5 ignored），`--features cuda-probe --lib` 的 677 项（含 T4 谱系测试）**不在门禁内**，
   所以 §6.3 的身份漂移在默认门禁下不可见、潜伏约 2 天。本片把特性配置测试单列进证据（§3.2）；
   建议后续把「至少一次 `--features cuda-probe --lib`」纳入批次门禁，但**不在本片实施**、不改
   `gates.toml`/门禁脚本。
5. 若选 ①/②，CI 需明确“无 GPU 机器跑特性测试”的口径：GPU 依赖测试已 `#[ignore]`（137 个），
   非 ignored 特性测试在本机 677/678 通过（唯一失败即第 3 条既有红）。
6. 若选 ③，CI 无变化；但需要为“选择层 + Host 回退”新增无 GPU 环境测试。

## 7. 建议（候选，非默认生效）

**决策判据**（供用户/Chief 拍板）：

1. 目标是“默认产品管线现在就具备 CUDA 入口（关闭 UNC-2）”吗？
   - 是 → ①/② 是必要前提；其中**② 优于 ①**（默认代价相同，但保留 `--no-default-features`
     回到当前最小构建，对下游/CI 更可控）。
   - 否 / 优先“默认依赖最小、CI 最轻” → **③** 先做（0 构建成本，先落选择层与回退语义），
     UNC-2 保持 open，后续再决定是否翻转特性默认。
2. 下游是否在意默认二进制体积与构建时间（debug +58 MiB / +11 s；release +6.4 MiB / +12 s）？
3. 是否接受“默认 CLI probe 从 FeatureDisabled 变为需要处理 CUDA 错误码”的行为变化？
4. 是否愿意先修 §6.3 的 T4 既有红（已登记独立小片 `2026-10-25-t4-source-hash-repin-13`）并定义
   无 GPU CI 口径？

**候选建议**：
- 若现在要关闭 UNC-2：选 **②**，**前置 = repin-13（§6.3 既有红）+ 阶段 2 选择层/Host 回退**；
  阶段 2 完成后默认构建即可用 GPU，`--no-default-features` 保留现状。
- 若用户更看重默认依赖最小：选 **③** 作为第一步（把回退语义与错误码统一），再评估后续是否 ②。
- ① 与 ② 默认代价相同但无退出开关，**不作为首选**。

## 8. 阶段 2 草案（拍板后实施，本阶段不做）

> 阶段 2 已拆分为独立任务 `2026-10-25-g10-p5-cuda-default-pipeline-stage2`（proposed，依赖阶段 1 +
> repin-13，等用户拍板后 start）；本文件只作设计输入。

1. `Cargo.toml`：按选定选项改默认特性（②：`default = ["cuda"]` + `cuda = ["dep:cudarc",…]`；
   ③：不变，仅新增运行时层）。
2. 运行时后端选择层（新模块，默认构建可用）：lazy 探测顺序 driver → device → NVRTC → PTX 编译；
   缓存选择结果；失败时记录 `CollisionGpuError`/`ProbeError` 错误码与文案（可查询状态），
   **不 panic**，回退 Host；Host 结果与当前路径**逐值一致**（同一函数）。
3. 产品接线：为碰撞入口加“后端选择”参数/状态（默认 Host；有设备且探测成功时用设备），
   不改 ABI、不改精度判据、不改已 verified 数字。
4. 测试：9 个 G5 已冻结 case 上 host vs device 接触数组逐字段对照（复用已入库 `qa-upstream-gpu.json`
   与输入，不新捕获；位级优先、非位级按既有 4·ULP 判据）；无 GPU 环境下 Host 与当前逐值一致；
   CLI 错误码/文案测试；gate fmt/clippy/clippy-cuda/unit。
5. GPU 设备侧复验按既有纪律报窗口（Arch 审 → Chief/用户签字），不新捕获。

## 9. 未覆盖 / 边界

- 阶段 1 **未改任何产品代码/Cargo 默认特性**；未开 GPU 窗口；未新捕获参考。
- “无 GPU/无驱动”运行期未实机模拟（本机有设备+驱动）；三态表运行期部分是代码路径与既有证据推导，
  阶段 2 需在无 GPU 机器/CI 上补实测。
- 未测性能（本片为依赖/CI 决策，不是性能片）。
- 时长/体积为单次 clean 测量（debug 默认两次 6.0/6.1 s，波动约 ±0.1 s）；跨机器不可外推。
- 选项①/② 的默认构建为 `--features cuda-probe` 代理测量；真正改默认特性后的首次构建还需另测。
- 阶段 2 之外的 G10 未覆盖项（mesh multicontact、Linux/T4 等）不在本片范围。
