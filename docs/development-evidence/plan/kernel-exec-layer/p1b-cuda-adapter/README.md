# P1-b CUDA 适配层（L1 证据）

- 任务卡：`2026-11-12-g10-p5-kernel-exec-p1b`（L1；S1 登记+接口冻结、S2 本证据）。
- 提交：S1 `e8ef68e89`（父 `e45b68e7d`）；S2 见本目录收口提交（卡 `/revision/after`）。
- 依据：A0 定稿 `d9c863d45`（§2/§3）；Arch P1-c 退项草案 `7b3cf603…`/12,030 B；A1 盘点 `55426797…`/37,413 B；P1-b prep `4e06960a…`/24,277 B。
- 边界：**先包不改**——不接任何现有调用点、不改既有 `runtime` 逻辑、不碰 `.cu`/`*_CUDA`/recipe/PTX/ABI/冻结集；旧 cudarc/NVRTC/launcher 路径保持原样（双轨，迁移属 P2）。本片零 GPU、未跑 GPU 测试。

## 1. 文件与行数（每文件 ≤500 行，LF 计数）

| 文件 | 行数 | 内容 |
| --- | ---: | --- |
| `src/runtime/kernel_exec/cuda/mod.rs` | 251 | `CudaBackend`/`CudaDevice`、caps 查询、注册表、冻结退项表、F.6 策略检查 |
| `src/runtime/kernel_exec/cuda/buffer.rs` | 500 | `CudaAllocator`/`CudaBuffer`、`CudaStorage`、缓冲注册表、B.3/B.5/B.6/B.9 |
| `src/runtime/kernel_exec/cuda/queue.rs` | 210 | `CudaQueue`：`submit`/`wait_idle`/`replay·capture_alloc→Unsupported`、发射校验 |
| `src/runtime/kernel_exec/cuda/module.rs` | 121 | `CudaModule`：NVRTC 编译/载入、F.6 PTX 闸、G.9 occupancy |
| `src/runtime/kernel_exec/cuda/diagnostics.rs` | 82 | `CudaDiagnostics`：L.2 计时事件、L.3 未接线 no-op |
| `src/runtime/kernel_exec/cuda/errors.rs` | 91 | 驱动/编译/发射/缓冲错误映射（A.4 不合并） |
| `src/runtime/kernel_exec/cuda/tests.rs` | 288 | 7 个 0 GPU 单测 + 3 个 `#[ignore]` GPU 等价测试 |

S1 已在 P1-a 文件上追加：`FallbackSet` 12 flag 改名与极性注释、`ThreadScope::shared`+`SharedBuf`、`Module::suggest_block_width`+`BlockSuggestion`、`CompileOptions.entry`、`BufferError::{Unsupported, ForeignBuffer}`、`Diagnostics::record_event -> Result`、`cuda/mod.rs` 空模块与 `cuda-probe` 门。
S2 追加（**D1 扩展，见 §7**）：`BufferError::DeviceLost`——缓冲面在驱动失败时不能吞掉设备错误。

## 2. 5 个优先面 → 契约实现（A1 口径）

| # | 面（A1 调用点） | 契约方法 | P1-b 实现 | 边界 |
| ---: | --- | --- | --- | --- |
| 1 | buffer 865 + upload/readback 611 | `Buffer::{addr,byte_len,upload,readback,pin}`、`Allocator::{alloc,transient}` | `cuda/buffer.rs`（自持 `CudaSlice<T>`，C.1 标量 7 种） | 复合 DType 7 种 → `BufferError::Unsupported("C.4")` |
| 2 | copy_range 10 + session 516 | `Buffer::copy_range`；session 拆为 `Device::{allocator,queue}` | `cuda/buffer.rs`（同设备/同 dtype/重叠拒斥/零长 no-op） | 跨设备 → `ForeignBuffer`；同缓冲重叠 → `Unsupported("B.5 self-copy")` |
| 3 | launch 597 | `Module::compile` + `Queue::submit` | `cuda/module.rs` + `cuda/queue.rs`（NVRTC + `launch_builder`） | 仅 D1、缓冲 offset=0、无别名/无重复缓冲；D2/D3、纹理/数组描述符 → `Unsupported` |
| 4 | wait_idle 55 | `Queue::wait_idle` | `cuda/queue.rs`（`synchronize + check_err`） | 唯一显式排空点；`submit` 不等待 |
| 5 | event 11 | `Diagnostics::{record_event,elapsed}` | `cuda/diagnostics.rs`（`CU_EVENT_DEFAULT` 计时事件） | 未知句柄 `elapsed` 返 `Duration::ZERO`（P1-a 签名无错误通道，文档写死） |
| + | G.9 | `Module::suggest_block_width` | `CudaFunction::occupancy_max_potential_block_size` 原生查询 | 查询失败 → `DeviceLost`；`occupancy_hint=false` 时才退 `{256,1}` |
| + | I.2 | `ThreadScope::shared` | 仅契约 + mock（mock 两条失败路径） | CUDA 适配层不实现（产品内核 0 用例）；属 P2 书写面 |

## 3. 12 个 fallback flag 冻结（P1-b 值）

极性：`true` = 该能力有原生实现（走原生）；`false` = 走退路（与直觉相反，`backend.rs` 注释写死）。

| flag | 能力 | 级别 | P1-b 值 | 理由 |
| --- | --- | --- | :---: | --- |
| `transient_alloc` | B.10 临时分配不触发同步 | 退 | true | cudarc 分配不触发宿主等待；池化 P1-c |
| `zero_copy_external` | B.11 零拷贝借用 | 退 | false | 外部句柄导入未接线（P3）；退路拷贝 P1-c |
| `generic_device_fn` | D.4 泛型设备函数 | 退 | true | CUDA C++ 模板原生（书写面 P2 使用） |
| `native_source_injection` | D.10 注入原生片段 | 退 | true | CUDA 原生内核即原生片段路径 |
| `occupancy_hint` | G.9 建议块宽/驻留 | 退 | true | 原生 occupancy 查询已接线 |
| `graph_capture` | K.1 录制重放 | 退 | false | P1-b 未接 `QueueCapture`/`replay`（P1-c） |
| `device_loop` | K.2 设备端条件循环 | 退 | false | 未接线（P1-c；退路=固定次数） |
| `device_branch` | K.3 设备端条件分支 | 退 | false | 未接线（P1-c；退路=无条件） |
| `capture_alloc` | K.5 录制期分配 | 退 | false | 未接线（P1-c；退路=预分配） |
| `kernel_printf` | L.3 内核打印 | 退 | false | `printf` 显式 no-op（P1-c 状态缓冲） |
| `int64` | C.1 64 位整数 | 必带退 | true | CUDA i64 原生（不启用双 i32 退法） |
| `f32_atomic` | H.1 f32 原子加 | 必带退 | true | `atomicAdd(float*)` 原生（不启用 CAS 退法） |

「必带退」语义：能力必级、退法常备；`false` 时分别走双 32 位表示 / CAS 循环（P1-c 落实现与双路测试）。

## 4. 零行为变化证据

- `kernel_exec` 在 `src/` 内**仍无外部引用**：除 `src/runtime/mod.rs:7` 的 `pub mod kernel_exec;`（P1-a 既有）外 0 处；本片未接任何调用点。
- `git diff --name-only -- src ':(exclude)src/runtime/kernel_exec'` = **空**（S2 提交前实测）。
- **29 个 `.cu` 逐文件 sha256 = 29/29 全等**（对 `refreeze-10-exe.json`，见 `frozen-cu-digest.log`；combined 摘要拼接口径未文档化，只声明逐文件 sha）。
- 旧 `runtime::transfer`/`runtime::launcher`/`runtime::cuda` 路径未改（`unchanged` 卡检查覆盖）。
- 现有测试不回归：全量 `cargo test --locked` **1447 passed / 0 failed / 405 ignored**（原 1435/0/402 + 新增 12 个单测 + 3 个新的 `#[ignore]` GPU 测试）。

## 5. 测试

- `cargo test --locked --lib kernel_exec`：**37 passed / 0 failed / 3 ignored**（`adapter-tests.log`）。
  - 0 GPU 单测 7 个：12 flag 极性冻结、caps 构造、DType 标量/复合映射、字节→元素范围与未对齐/越界、F.6 策略、发射校验 7 类拒绝、错误分类映射（另含 P1-b 接口单测与 mock 双路）。
  - `#[ignore]` GPU 等价测试 3 个：真实 caps、缓冲上传/回读/拷贝 + NVRTC 编译发射回读、（事件计时）。**本片未运行，不声明通过、不开窗**。
- 全量 `cargo test --locked`：1447/0/405（`unit-summary.log`，146 个测试目标）。

## 6. 门禁（`normify gate 2026-11-12-g10-p5-kernel-exec-p1b`）

| 门禁 | 耗时 | 结果 |
| --- | ---: | --- |
| fmt | 2,013 ms | PASS |
| clippy | 5,328 ms | PASS（0 warning） |
| clippy-cuda | 455 ms | PASS |
| unit | 101,440 ms | PASS（1447 过 / 0 败） |

补充自证：`cargo clippy --locked --no-default-features --lib -- -D warnings` 通过（`cuda-probe` cfg 门正确；注：`--all-targets` 的 no-default 构建在仓库既有 test target 上本就失败，与本片无关，未作为卡门禁）。结果 JSON 在 `.normify/runs/2026-11-12-g10-p5-kernel-exec-p1b/`。

> **两轮独立测量**：上表是**冷跑**一轮（首次全量编译，unit 101,440 ms）；提交的 `.normify/runs/*.json` 是收口前**热跑**复跑（缓存命中：fmt 1,979 ms / clippy 456 ms / clippy-cuda 455 ms / unit 16,845 ms）。两轮分开记录，不互相覆盖。
> 另：`unit.json::log_sha256` 与 `unit-summary.log` 的 sha 不同是**关系而非矛盾**——前者是 gate 的原生产出日志，后者是本片过滤后的 `Running`/`test result:` 摘要。

## 7. 覆盖/未覆盖与披露

- **未接线不静默降级（三条口径）**：
  1. **返回 `DeviceError::Unsupported` / `BufferError::Unsupported`**：B.11 `borrow_external`（外部句柄导入属 P3）；K.1 `Queue::replay`、K.5 `Queue::capture_alloc`；`DType` 复合 7 种（P2/P3）；`IndexSpace::D2/D3`、`Arg::Buffer.offset≠0`、别名声明、重复缓冲地址、纹理/数组描述符（P2/P3）；`KernelSource::Named`（P1-c/P3）；`CompileOptions.defines` 非空（P1-c）。
  2. **签名无错误通道 → 显式 no-op**：L.3 `Diagnostics::printf`（`fn printf(&self, fmt, args)` 无返回值；P1-b 不分配、不打印、不等待，P1-c 落状态缓冲退路）。
  3. **未实现该 trait**：K.1 的 `QueueCapture` 在 CUDA 适配层**未实现**（不是返回 Unsupported）；`Queue::replay` 单独返回 `Unsupported`。
- **D1 扩展（已获 Chief 追认）**：`BufferError::DeviceLost` 为 S2 实现时新增——P1-a 的 `BufferError` 只有参数类变体，缓冲操作的驱动失败无处上报会违反"A.4 三分类不得吞掉"；只被 `kernel_exec` 使用、零外溢，A0 由 Arch 补一句（本片不改 A0）。
- **语义备注**：`Init::Uninit` 用清零实现（内容"未定义"的合法取值，避免 `unsafe alloc`，且不引入隐式同步）；`upload/readback` 按小端位模式解释字节（宿主机侧允许一次暂存拷贝）；`submit` 的实参 ABI 匹配由调用方按 `CompileOptions`/入口约定保证（A0 §2 G.3 opaque 描述符取舍）。
- **unsafe 实为 2 处**（均在 `src/runtime/kernel_exec/cuda/queue.rs`，各带安全前提注释）：`:33` `unsafe impl DeviceRepr for DevicePtrArg`（transparent u64 实参 ABI）；`:180` `unsafe { builder.launch(config) }`（内核发射）。

## 8. 复跑

```powershell
cd D:\Rust\mjwarp-rs
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked --lib kernel_exec
cargo test --locked
# GPU 等价（不新开窗；有窗时）
cargo test --locked --lib kernel_exec::cuda -- --ignored --test-threads=1
```

## 9. 本目录文件（L1：不建 manifest、不逐文件 sha）

| 文件 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `adapter-tests.log` | `cargo test --locked --lib kernel_exec` 输出（37 passed / 3 ignored） |
| `unit-summary.log` | 全量 `cargo test --locked` 的 `Running`/`test result:` 摘要（146 目标，1447 过） |
| `frozen-cu-digest.log` | 29 个 `.cu` 逐文件 sha256 复算（29/29 match） |
| `artifacts.json` | 文件/行数、12 flag、trait 实现、未覆盖面、测试与门禁、冻结身份、边界 |

## 10. 边界

- 本片只交付**契约冻结 + CUDA 宿主面适配层 + 零行为变化证据**；不代表任何能力已实现/已验收。
- 不宣称跨设备等价、不宣称位级等价；GPU 等价测试未运行。
- 不触碰判据/容差/参考/ABI/冻结集；`A0` 措辞由 Arch 在 S1 落地后同步（本片不改 A0）。
