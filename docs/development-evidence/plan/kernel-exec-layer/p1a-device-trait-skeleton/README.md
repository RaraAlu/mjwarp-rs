# P1-a 设备层 trait 骨架（L1 证据）

- 任务卡：`2026-11-12-g10-p5-kernel-exec-p1a`（L1；S1 登记+骨架、S2 本证据）。
- 依据：A0 `docs/architecture/device-layer-contract.md` §2 六组接口；A1 盘点（`target/impl-plan-review/A1-cuda-coupling-inventory.md`，只读输入）。
- 范围：`src/runtime/kernel_exec/**` 新建 + `src/runtime/mod.rs` **加一行** `pub mod kernel_exec;`。
- 边界：**先包不改**——不接任何现有调用点、不改既有 runtime 逻辑、不碰 `.cu`/`*_CUDA`/recipe/PTX/ABI/冻结集；本片零 GPU、未跑 GPU 测试。

## 1. 文件与行数（每文件 ≤500 行）

> 行数口径：**LF 计数**（与 `git show --stat` 一致），下表为 P1-a 收口提交 `522f8661` 时点。
> 初版表格误用 `split('\n')` 物理行计数（含尾部空段，每项 +1）；P1-b 首个提交（`2026-11-12-g10-p5-kernel-exec-p1b`）
> 更正此表与 `artifacts.json`，不改原提交。

| 文件 | 行数 | 内容 |
| --- | ---: | --- |
| `src/runtime/kernel_exec/backend.rs` | 200 | 设备层接口/值与 mock（见模块文档） |
| `src/runtime/kernel_exec/buffer.rs` | 159 | 设备层接口/值与 mock（见模块文档） |
| `src/runtime/kernel_exec/device.rs` | 168 | 设备层接口/值与 mock（见模块文档） |
| `src/runtime/kernel_exec/error.rs` | 266 | 设备层接口/值与 mock（见模块文档） |
| `src/runtime/kernel_exec/mock_backend.rs` | 456 | 设备层接口/值与 mock（见模块文档） |
| `src/runtime/kernel_exec/mock_tests.rs` | 212 | 设备层接口/值与 mock（见模块文档） |
| `src/runtime/kernel_exec/mod.rs` | 36 | 设备层接口/值与 mock（见模块文档） |
| `src/runtime/kernel_exec/module.rs` | 86 | 设备层接口/值与 mock（见模块文档） |
| `src/runtime/kernel_exec/queue.rs` | 151 | 设备层接口/值与 mock（见模块文档） |
| `src/runtime/mod.rs` | 125 | 仅新增一行 `pub mod kernel_exec;` |

`mock.rs` 初版 669 行超 500 棘轮，已拆为 `mock_backend.rs`（456 行）+ `mock_tests.rs`（212 行）；语义与测试不变。

## 2. 设计决策（P1-a）

- **六组 trait 名与分组按 A0 §2**：`Backend`/`Device`/`DeviceCaps`、`Buffer`+`Allocator`、`Queue`、`Module`+`Launch`、`DeviceMath`+`Atomics`+`ThreadScope`+`ExecControl`+`Diagnostics`、`DeviceError` 三分类。
- **对象安全取舍**：宿主调度面（`Backend`/`Device`/`Allocator`/`Buffer`/`Queue`/`Module`/`Diagnostics`/`ThreadScope`/`Atomics`）保持 `dyn` 兼容；`DeviceMath`（静态方法，设备端单态化）、`ExecControl`（闭包参数）、`QueueCapture`（泛型 `capture`）不追求 `dyn`，单测用泛型边界验证可用性。
- **错误模型**：`DeviceError::Compile/Launch/DeviceLost` 为硬性三分类（`class()` 返回 `ErrorClass`）；`Unsupported(CapabilityId)` 是能力缺失（不并入三分类）；缓冲容量/越界用独立 `BufferError` 显式表达。
- **数值策略**：`NumericPolicy` 同批定义，`FROZEN_F32` = f32-only + FMA 开 + fast-math 关（F.6），为编译/发射必填语义。
- **零 unsafe / 零 cudarc**：本层不使用 `unsafe`；签名只出现本层类型；原子用 opaque `Address64`，指针语义留给 P1-b 适配层。
- **dead_code**：模块从 crate 根 pub 可见，未使用 `#[allow(dead_code)]`，未改 lint 配置。

## 3. 测试

- `kernel_exec` 单测 **25 个全过**（`cargo test --locked --lib kernel_exec`），覆盖：
  错误三分类/`Unsupported`/`BufferError`、容量与越界显式报错、`queue.submit` 不隐式同步而 `wait_idle` 显式排空、launch/capture/replay、mock 分配/上传/回读/范围拷贝、对象安全 `dyn` 断言、设备侧 trait 泛型可用性、`NumericPolicy` 冻结默认。
- 全量 `cargo test --locked`：**1435 passed / 0 failed / 402 ignored**（146 个测试目标；为原 1410 + 新增 25）。
- 未运行 GPU 测试。

## 4. 门禁（手动 + `normify gate` 各一次）

| 门禁 | 耗时 | 结果 |
| --- | ---: | --- |
| fmt | 1.77 s | PASS（exit 0） |
| clippy | 0.45 s | PASS（0 warning） |
| clippy-cuda | 0.4 s | PASS |
| unit | 88.34 s | PASS（1435/0/402） |

`normify gate` 结果存于 `.normify/runs/2026-11-12-g10-p5-kernel-exec-p1a/`（fmt/clippy/clippy-cuda/unit 四份 JSON）。
上表为**手动首跑**耗时；`.normify/runs` 是 `normify gate` 的**另一轮独立测量**，两者不是同一轮数据（缓存命中/失效状态不同）。

## 5. 复跑

```powershell
cd D:\Rust\mjwarp-rs
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked --lib kernel_exec
cargo test --locked
```

## 6. 本目录文件（L1：不建 manifest、不逐文件 sha）

| 文件 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `test.log` | `cargo test --locked --lib kernel_exec` 输出（25 passed） |
| `unit-summary.log` | 全量 `cargo test --locked` 的 `test result:` 摘要 |
| `artifacts.json` | 文件/行数、trait 分组、对象安全清单、测试与门禁、边界 |

## 7. 边界

- 本片只交付**接口声明 + mock + 单测**；不代表任何真实后端实现，不构成能力完成/验收结论。
- 行为零变化：未接调用点、未改既有模块逻辑、未改内核源/recipe/PTX/ABI/冻结集；现有测试全过。
- 下一步 P1-b 才把现有 cudarc/NVRTC/launcher 路径平移到 `CudaBackend` 实现。
