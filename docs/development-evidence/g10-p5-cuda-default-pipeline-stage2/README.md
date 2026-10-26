# G10@P5 CUDA 默认管线接线 · 阶段 2 证据（`cuda` 默认特性 + 历史 T4 审计 gating + 三态回退 + 选择层）

- 任务：`2026-10-25-g10-p5-cuda-default-pipeline-stage2`（p56，owner Backend；start HEAD `ec44ca0d`）
- 用户拍板：cudarc 走 **②**（新增 `cuda` 特性默认开启、`--no-default-features` 可关）；
  结构项按 `target/stage2-prep/DESIGN-stage2.md` 的变体 **B** 与 Arch 2026-10-26 裁决执行。
- 资源：**GPU 窗口 0**（本证据截至 ③a 全 CPU；GPU 未使用、未开窗）。等价性（host vs device）在 ③b/③c
  单独申请窗口后补；**UNC-2 记为 open（not closed）**。
- 铁律：不改 GPU 路径/host 数值链/ABI/判据/容差/期望值/已 verified 证据语义；不宣称整体通过。

## 0. 结论速览（事实；不作整体结论）

1. 变体 B 落地：`default=["cuda"]`、`cuda=["cuda-probe"]`（单一依赖集合）；`--no-default-features`
   在无 CUDA/驱动的环境构建+测试通过（604→**605**，含本片新增 1 条选择层测试）；默认测试
   **971 / 0 / 364**（lib 698/0/137），`--features cuda` 与默认同集合同数。
2. 历史 T4 审计：`Cargo.toml` 加非默认 `t4-historical-audits = []`；7 个 target 门控为
   `all(cuda, t4-historical-audits)`；默认 0 tests、带特性 **27/0**（3/4/4/4/4/4/4，与改动前集合逐项相同）；
   `--all-features`（MSVC 工具链 + G01 环境）**27/0**（见 §6）。
3. 不 panic 三态回退已修：`device_context()` 驱动预检 + `catch_unwind`→`NoDevice`、`compile_program()`
   的 NVRTC 预检→`Compile`；修前红（scratch 还原守卫：panic 逃逸 exit 101）→ 修后绿（panic 被捕获，
   进程不 abort）。CLI 侧 `DriverUnavailable`/`NvrtcUnavailable`/`InvalidDevice` 同源断言。
4. 选择层（`runtime::backend`）：driver→device→NVRTC→PTX 编译探测、`OnceLock` 缓存、默认 Host、
   显式 `BackendChoice::Device` 且 `CudaReady` 才允许；设备入口 `device_program()` lazy 缓存。
5. 门禁：`gates.toml` 新增 `unit-nocuda`/`clippy-nocuda`、`clippy-cuda` 改 `--features cuda`；
   G01 两个脚本去冗余 flag；基线记录见 §3。**未 commit、未 done**（提交由 Arch 做）。

## 1. 特性与依赖

```toml
[features]
default = ["cuda"]
cuda = ["cuda-probe"]                       # 默认开启入口；transitively 启用内部实现名 cuda-probe
cuda-probe = ["dep:cudarc", "dep:libloading", "dep:serde", "dep:serde_json", "dep:sha2"]
cubecl-cpp-probe = ["cuda-probe", "dep:cubecl-core", "dep:cubecl-cpp"]
cubecl-llvm-probe = ["cuda-probe", "dep:cubecl-core", "dep:cubecl-llvm"]
native-model-probe = ["dep:cc"]
t4-historical-audits = []                   # 非默认 test-only：7 个历史 T4 审计 target
```

- 依赖集合仍只有一套（`cudarc` 只由 `cuda-probe` 引入）；`cudarc` 依赖行未改，**`Cargo.lock` 0 改动**。
- `cargo tree --no-default-features -e normal,build` 外部包 **0**（`cudarc` 命中 0）；默认外部包 **24**
  （去重；`cargo tree` 原始行 33，含 root/重复子树）。
- 迁移注记：`cuda` = 默认启用入口；`cuda-probe` 保留为代码内部门控名/历史命令兼容名；名称统一
  （cfg/文案）留给后续治理片（见 §8 迁移债）。

## 2. 三配置独占 clean 实测（CPU；每配置独立 `CARGO_TARGET_DIR`，先删后建；debug）

| 配置 | build clean | target（含全量测试产物） | exe | 外部依赖包 | 测试（pass/fail/ignored） |
| --- | --- | --- | --- | --- | --- |
| 默认（= `cuda`） | **16.91 s** | **8977.4 MiB** | **77.98 MiB** | **24** | **971 / 0 / 364**（111 binary；lib 698/0/137） |
| `--no-default-features`（PATH 去 CUDA/NVIDIA；CUDA_PATH/CUDA_HOME/CUDA_INCLUDE/CUDA_LIB_PATH/CUDA_BIN_PATH 清空） | **6.19 s** | **1807.5 MiB** | **19.69 MiB** | **0** | **605 / 0 / 7**（lib 455/0/5） |
| `--features cuda`（与默认同 feature 集） | **16.80 s** | 8977.4 MiB | 77.98 MiB | 24 | **971 / 0 / 364** |

- 增量（相对改动前基线 604/0/7、build 8.0 s / target 196 MiB / exe 19.7 MiB）：build **+8.9 s**、
  target **+8781 MiB（全量测试后）**/ 仅 build 后约 +581 MiB、exe **+58.3 MiB**、依赖 **+24**、测试 **+367**。
- 测试计数增量来源（全部为 CPU 测试）：`cuda` 门控库测试 +244（默认可跑 CUDA 探针测试）、
  历史 T4 审计 gating 移出 27 项、本片新增三态/选择层测试 +14（默认 13 + no-default 1）。
- 计时为 2026-10-26 主树独占时段实测（无并行重任务）；正式门禁仍以最终 HEAD 重跑为准。
- 附：`--all-features` 定向（MSVC 工具链 + G01 环境；不属于上面三配置，只跑 7 个历史审计 target）：
  全特性编译 **24.18 s**（复用 `g01-fast` 默认构建缓存），7 target **27/0**；**未跑**全量 all-features 测试
  （按 Arch 指示只跑定向）。日志：`logs/allfeatures-audit27.log`。

## 3. 门禁变更（old / new / 理由 / 回滚）

| 项 | old | new | 理由 | 回滚 |
| --- | --- | --- | --- | --- |
| `gate.unit` | `cargo test --locked`（604/0/7） | 命令不变；基线 **971/0/364** | 默认含 cuda + 历史审计移出 + 新增测试 | Cargo default 回退后自然回旧语义 |
| `gate.clippy-cuda` | `--features cuda-probe` | `--features cuda` | 默认入口名统一（代码 cfg 仍 `cuda-probe`） | 改回旧 flag |
| `gate.unit-nocuda` | （无） | `cargo test --locked --no-default-features`（parse=libtest；基线 605/0/7） | 无卡/快速 CI 路径，可重复门禁 | 删除该节 |
| `gate.clippy-nocuda` | （无） | `cargo clippy --all-targets --locked --no-default-features -- -D warnings` | 同上 | 删除该节 |
| G01 默认矩阵 | `--features native-model-probe,cuda-probe` | `--features native-model-probe` | `cuda` 已默认开启，去掉冗余 flag | 改回旧 flag |
| G01 `clippy-gnu-cuda` | `--features cuda-probe` | `--features cuda` | 同上 | 改回旧 flag |

- `gates.toml` 的 `hash_inputs` 已含 `Cargo.toml`（门禁哈希变化属预期）。
- task checks 由 Arch 收口时用 `normify gate <task> fmt clippy clippy-cuda unit unit-nocuda clippy-nocuda`
  显式跑并把 runs 留证（任务检查项已冻结，不新增）。

## 4. 不 panic 三态回退（产品 API + CLI）

`src/runtime/collision_gpu/context.rs::device_context()`：驱动库缺失 → 预检 `NoDevice`（避免 cudarc
`culib()` panic）；`CudaContext::new` 返回错误（有驱动无设备）→ `NoDevice`；内部 panic →
`catch_unwind` 捕获 → `NoDevice`。`compile_program()` 先 NVRTC 预检 → `Compile`（编译先于设备构造）。

| 路径 | 产品 API | CLI/探针 |
| --- | --- | --- |
| 无驱动 | `NoDevice("CUDA驱动库缺失（libcuda/nvcuda 不可加载）")` | `DriverUnavailable` |
| 有驱动无设备 | `NoDevice("CudaContext::new(0): …")`（seam 注入模拟；实机不可模拟） | `InvalidDevice{requested:0,available:0}`（seam 注入） |
| 有设备无 NVRTC | `Compile("NVRTC库缺失（nvrtc 不可加载）")` | `NvrtcUnavailable` |
| culib 初始化 panic（兜底） | `NoDevice("…panic（cudarc culib）已捕获")` | 既有 `run.rs` 边界 `catch_unwind`→`BackendPanic`（语义未改） |

- **修前红（实测）**：scratch 副本还原守卫（去预检/`catch_unwind`，保留注入）→ 同一 panic 测试
  **exit 101**，日志含 `panicked at context.rs:15 注入的 cudarc culib panic` 与
  `panic 逃出了 device_context（守卫缺失）`。
- **修后绿（实测）**：同测试 exit 0（`catch_unwind` 外层证明 panic 不外泄、进程不 abort）；
  三条路径 API 4 测试 + CLI 3 测试均**独立进程**单跑 exit 0。
- **seam 纪律**：注入 seam 仅 `#[cfg(test)]`（`src/runtime/presence.rs`：原子 + 串行锁 + RAII 复位）；
  产品二进制扫描 `target/debug/mjwarp-rs.exe`：`DRIVER_OVERRIDE`/`NVRTC_OVERRIDE`/`CONTEXT_PANIC`/
  `SeamGuard`/`set_*_for_tests`/`MJWARP_*` 全部 **0 命中**；改动文件无 `std::env::var` 读取。
- 未覆盖：真实无驱动/无卡机器未实机（本机有驱动+设备）；「有驱动无设备」「culib panic」为 seam 模拟；
  普通 shell 下 cudarc NVRTC 探测为 false（NVRTC DLL 不在 PATH），产品 `new()` 全链路「编译成功→设备失败」
  未在 CPU 环境覆盖（G01 环境可覆盖）。

## 5. 选择层（`src/runtime/backend.rs`，新文件；不改 host 数值链与 device kernel）

- 公共类型：`BackendChoice{Host,Device}`、`BackendStatus{HostOnly(BackendUnavailable),CudaReady}`、
  `BackendUnavailable{FeatureDisabled,DriverUnavailable,NoDevice,NvrtcUnavailable,Compile}`。
- 探测顺序（有 `cuda` 特性）：驱动库 → 设备数 → NVRTC 库 → PTX 编译；任一失败 → `HostOnly(原因)`，
  不 panic；`OnceLock` 进程内缓存（不每 pair 重探）。无 `cuda` 特性恒 `HostOnly(FeatureDisabled)`。
- 默认 **Host**：`device_permitted(choice)` 仅在显式 `Device` 且缓存状态 `CudaReady` 时返回 true；
  设备入口 `device_program()` 为 lazy `OnceLock` 缓存（仅在 `CudaReady` 时尝试构造，失败返回 None）。
- CLI/API 一致：两者共享 `runtime::presence` 探测与错误映射；seam 测试断言同状态下列：
  无驱动 → API `HostOnly(DriverUnavailable)` + CLI `DriverUnavailable`；无 NVRTC → API `NvrtcUnavailable`
  + CLI `NvrtcUnavailable`（独立进程测试，日志见 `logs/`）。
- 本层不参与数值计算、不新增判据/容差；host 与 device 数值口径沿用既有已 verified 证据。
- 日志：`logs/selection-backend-tests-default.log`（6 条 cuda 构建）、`logs/selection-backend-tests-nocuda.log`（1 条 no-default 构建）。

## 6. 历史 T4 审计 gating（7 target / 27 测试）

- 改动：`Cargo.toml` 加 `t4-historical-audits = []`；7 个测试文件第 3 行
  `#![cfg(feature="cuda-probe")]` → `#![cfg(all(feature="cuda", feature="t4-historical-audits"))]`。
- 理由：这 7 个 target 依赖本机 `target/` 冻结产物（干净 clone/CI 会 `NotFound` 假红）；默认门禁
  须可移植；`--all-features`（G01）仍运行且须在主树 pass。
- 实测：
  - 干净 clone 语义（无 `target/` 产物）：未门控默认全量 **969/16/364**（16 全 NotFound）→ 门控后 **958/0/364**
    （阶段①数字；阶段③加入新测试后为 971/0/364，历史审计仍 0 tests）。
  - 默认 `cargo test --locked` 下 7 target = **0 tests**；`--features cuda,t4-historical-audits --test <7>`
    = **27/0**（3/4/4/4/4/4/4，与改动前 `--features cuda-probe` 的 27/0 集合逐项相同）。
  - `--all-features`（MSVC 工具链 + G01 环境）7 target = **27/0**（`logs/allfeatures-audit27.log`）。
  - G01 定向：`host-gnu`（GNU `cargo test --locked`）**971/0/364**；`host-msvc` 的 **all-features 口径**
    27/0（同上）；`host-msvc` 的旧 `--features native-model-probe` 口径对这 7 target 为 **0 tests**
    （gating 生效，`logs/g01-host-msvc-audit27-native-model-probe.log`）；`clippy-gnu-cuda`
    （`--features cuda --all-targets -D warnings`）exit 0。
  - `g01-fast.ps1`（改动后）：默认（不带 `-AllFeatures`）这 7 target **0 tests**、exit 0；
    `-AllFeatures` 时 **27 passed / 0 failed**、exit 0（`logs/g01-fast-default-audit27.log`、
    `logs/g01-fast-allfeatures-audit27.log`）——与 §8 债务 5 一致。
- 无 `src/` 改动、无需 T4 repin（`product_sources` 20 条不含 `tests/`）。

## 7. 等价性 / UNC-2（**closed，Chief 2026-10-26 批准；限定语见 §7.1**）

- 窗口与记录 B：`window-attempt4-and-b-classification.md`（attempt4 事实、三条偏差、B 分类、行序修正前后、QA 要点）。
- **不是 B 全绿**：可评估子集全绿（cylinder/ellipsoid/mesh-mesh 26/26 位等、sphere-box-shallow 判据内、sdf-box 1040/1040 位等）；
  `sphere-box-deep`/`capsule-box` = `reference_not_comparable`（G5 §11.2）；`convex-mesh`/`degenerate-mesh` = `no_frozen_reference`。

### 7.1 UNC-2 定稿限定语（Chief 批准，一字不改）
「默认构建含 CUDA 入口（`cuda` 默认特性、`--no-default-features` 可关）；产品入口经 NVRTC launcher 在真实设备执行中被调用（4 个设备测试二进制 exit 0；9 case dump 两次运行逐位一致）；记录 B 可评估子集全绿（cylinder/ellipsoid/mesh-mesh 26/26 位等、sphere-box-shallow 判据内、sdf-box 1040/1040 位等）；2 例按 G5 §11.2 参考不可比（sphere-box-deep/capsule-box）、2 例无冻结参考（convex/degenerate）——**不等于 B 全绿**；**默认选择层仍 Host、Device 需显式选择**——本项关闭不代表默认数值路径走 Device，不代表 host↔device 等价、上游等价、精度达标。」

- UNC-2 关闭条件：默认管线真接上 CUDA 入口 **且** host vs device 在已冻结 case 上逐字段一致已验证。
- 已具备：选择层 + lazy 设备入口 + 三态回退；device 内核/host 数值链未改。
- 已做（原「未做」项）：GPU 窗口 attempt4（E 窗口内 exit=101 已披露）+ ③c CPU 记录 A/B/C → `window-attempt4-and-b-classification.md`、`three-records.json`。
- **未实机验证前不宣称**默认管线已具备可用 CUDA 入口，也不记 UNC-2 closed。

## 8. 迁移债清单（本片不修，移交后续治理片）

1. **代码 cfg 名**：`src/`+`tests/` 仍 4437 行 `feature = "cuda-probe"`（名称统一需重命名 486 文件 +
   T4 `product_sources` 17/20 repin，另立治理片）。
2. **用户可见文案**：`ProbeError::FeatureDisabled("cuda-probe")` 167 行/70 文件；无特性构建提示仍写旧名。
3. **文档手工命令**：6 份文档的 `cargo test --locked --features cuda-probe --test t4_…` 在变体 B 下
   **静默 0 tests**（默认已含 `cuda-probe`，cfg 无法区分显式/伞形）：`docs/t4-camlight-internal.md:112-113`、
   `docs/t4-face-normalization-control.md:89-90`、`docs/t4-face-trig-control.md:110-111`、
   `docs/t4-field-internal.md:112-113,226-227`、`docs/t4-spatial-tendon-internal.md:107-108`、
   `docs/t4-tendon-wake-internal.md:112-113`；需加 `--features t4-historical-audits`（或 `--all-features`）。
4. **G01 外脚本冗余 flag**：`migration-inventory.md` §3 列出的其余 16 处（standalone 探针脚本等）仍带
   `--features cuda-probe`——可继续工作（特性名保留）但冗余；随名称统一治理片清理。
5. `--features cuda-probe` 不再触发 7 个历史审计 target（变体 B 固有），CI 若用该命令跑全量，计数会少 27。
6. **G01 / g01-fast 的非 all-features 口径**：`host-msvc`（`--features native-model-probe`）与 `g01-fast`
   默认模式在这 7 个 target 上为 0 tests；需 `-AllFeatures`/`--all-features` 才运行（已实测 27/0）。
   G01 全量门禁 `g01-full` 本身用 `-AllFeatures`，故不受影响。

## 9. 未覆盖 / 边界 / 红线

- 真实无驱动/无卡机器未实机；三态为 seam 模拟（见 §4）。
- GPU 窗口 1（attempt4，唯一有效窗口；attempt1/2/3 零 GPU 作废）；等价性记录见 `window-attempt4-and-b-classification.md` 与 §7/§7.1。
- 未跑全量 G01（按 Arch 指示只跑定向 `host-gnu`/`host-msvc`/`clippy-gnu-cuda`）。
- 不宣称精度达标/跨环境全绿/上游完全等价/双平台通过/原生放行/G10 已实现/完整物理通过；
  NaN payload 不作位级判据；边界计数 16；host 与 device 两口径并存，不宣称二者数值等价。

## 10. 复现命令

```powershell
# 三配置
cargo build --locked; cargo test --locked
cargo build --locked --no-default-features; cargo test --locked --no-default-features
cargo build --locked --features cuda; cargo test --locked --features cuda

# 历史 T4 审计 gating（默认 0 tests；带特性 27/0）
cargo test --locked --test t4_camlight_precision -- --list
cargo test --locked --features cuda,t4-historical-audits --test t4_camlight_precision `
  --test t4_face_fusion_repair --test t4_face_normalization_control --test t4_hessian_generation `
  --test t4_spatial_tendon_precision --test t4_tendon_wake_precision --test t4_trig_control_domain `
  -- --test-threads=1

# 三态回退（独立进程）
cargo test --locked --lib runtime::collision_gpu::fallback_tests -- --test-threads=1
cargo test --locked --lib runtime::cuda::tests -- --test-threads=1
cargo test --locked --lib runtime::backend -- --test-threads=1

# 门禁（含新增两个）
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo clippy --all-targets --locked --no-default-features -- -D warnings
cargo test --locked
cargo test --locked --no-default-features
```
