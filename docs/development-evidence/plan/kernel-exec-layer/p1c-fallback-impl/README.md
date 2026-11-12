# P1-c 退路实现与 DeviceCaps.fallbacks 双路（L1 证据）

- 任务卡：`2026-11-12-p1c-fallback-impl`（L1；S1 登记+核心、S2 本证据）。
- 提交：S1 `e8ec1f2f1`（退路核心 + mock 双路 + P1-b 复核低项修正）；S2 见本目录收口提交。
- 依据：A0 定稿（`docs/architecture/device-layer-contract.md`，12 flag 定稿名与极性）；P1-c 退项定义草案 `target/impl-plan-review/p1c-fallback-definitions.md`（`7b3cf603…` / 12,030 B）；P1-b 卡 `2026-11-12-g10-p5-kernel-exec-p1b`（verified，`depends_on`）。
- 范围：`src/runtime/kernel_exec/**`（`fallback/` 新模块 + `cuda/` 退路接线 + mock 注入点）+ 本证据目录；不接现有调用点、不碰 `.cu`/recipe/PTX/ABI/冻结集；0 窗 0 GPU。

## 1. 结论（口径写死）

- **12 个 fallback flag 定义齐**（10 个「退」项 + 2 个「必带退」）。
- **10/12 实现**：B.10、B.11、G.9、K.1、K.2、K.3、K.5、L.3、C.1、H.1。
- **D.4 / D.10 = `after_shim`，未实现**：只登记退路语义、触发点与 P2 依赖，不出双路结论。
- 本片结论**不是「12/12 完成」**；D.4/D.10 待 P2 shim 收口后另片补齐（P1-c2 或并入 P2 卡）。

## 2. 12 项逐项（`items.json` 为机器可读版）

| 能力 | flag | 级别 | 范围 | 退路语义（false） | 双路证据 |
| --- | --- | --- | --- | --- | --- |
| B.10 | `transient_alloc` | 退 | now | 预分配池；池空显式 `CapacityExceeded` | `fallback/tests.rs::b10_*`；CUDA `transient_mode` 双路 + 池前置 API |
| B.11 | `zero_copy_external` | 退 | now | 拷贝到独立所有权缓冲（未登记范围→显式 Unsupported） | `b11_*`；CUDA `external_borrow_mode` + 登记拷贝退路 |
| D.4 | `generic_device_fn` | 退 | **after_shim** | 按类型单态化展开（P2 emitter） | 只登记（`ImplScope::AfterShim` 断言） |
| G.9 | `occupancy_hint` | 退 | now | 固定值 `{256,1}` | `g9_*`；mock 双路 `{512,2}`/`{256,1}`；CUDA `occupancy_mode` |
| K.1 | `graph_capture` | 退 | now | 逐次提交（录制-重放） | `k1_*`；CUDA `graph_mode` + capture/replay 退路 |
| K.2 | `device_loop` | 退 | now | 固定次数发射 | `k2_*`；CUDA `loop_mode`（执行属 P2） |
| K.3 | `device_branch` | 退 | now | 两分支都执行（then→else，冻结） | `k3_*`；CUDA `branch_mode`（执行属 P2） |
| K.5 | `capture_alloc` | 退 | now | 录制前预分配；池空显式错误 | `k5_*`；CUDA `capture_alloc_mode` + 预分配池 |
| L.3 | `kernel_printf` | 退 | now | 状态缓冲 + 宿主打印（占位符安全替换） | `l3_*`；CUDA `printf_mode` + 状态缓冲 |
| D.10 | `native_source_injection` | 退 | **after_shim** | 改用 I.3 barrier（P2 书写面） | 只登记 |
| C.1 | `int64` | 必带退 | now | 双 32 位表示（无损往返） | `c1_*`；CUDA `int64_lowering`（执行属 P2） |
| H.1 | `f32_atomic` | 必带退 | now | CAS 循环（并发 32×250 得 8000.0） | `h1_*`；CUDA `f32_atomic_mode`（执行属 P2） |

## 3. 实现落点

- `src/runtime/kernel_exec/fallback/`（7 文件，均 ≤500 行）：`mod.rs`（12 能力编号/`ImplScope`/极性/`FallbackTrace`/`select_native`）、`buffers.rs`（B.10/B.11）、`graph.rs`（K.1/K.5）、`control.rs`（K.2/K.3）、`printf.rs`（L.3）、`numeric.rs`（C.1/H.1）、`tests.rs`（双路/组合/无同步）。
- `src/runtime/kernel_exec/cuda/fallback_plan.rs`：CUDA 侧"原生/退路"选择双路（0 GPU 可测）；适配层接线：
  - `CudaAllocator::{transient, borrow_external}` 走 `transient_mode`/`external_borrow_mode`；
  - `CudaQueue::{replay, capture_alloc}` 与 `impl QueueCapture for CudaQueue` 走 `graph_mode`/`capture_alloc_mode`；
  - `CudaDiagnostics::printf` 走 `printf_mode` + 状态缓冲（`drain_status_buffer`）；
  - 前置 API：`CudaDevice::{preallocate_transient, register_external, preallocate_capture}`。
  - `K.2/K.3/C.1/H.1` 在 CUDA 侧只冻结选择（设备端 lowering/emitter 属 P2），见 §1 边界。
- mock backend：`MockDevice::with_fallbacks`、`MockModule::new(caps)`（G.9 双路）、`QueueState.waits`（无隐式同步断言）。

## 4. 测试

- `cargo test --locked --lib kernel_exec`：**51 passed / 0 failed / 4 ignored**（`fallback-tests.log`；4 个 ignored = P1-b 3 个 + P1-c 新增 1 个 CUDA 退路可达性 GPU 测试，**未运行、不开窗**）。
  新增 14 个测试：12 项双路/登记、4096 种 flag 组合 × 12 项选择、主机退路不碰队列/不隐式同步、CUDA 侧 10 项选择双路。
- 全量 `cargo test --locked`：结果见 `unit-summary.log`（0 failed）。
- 组合规则：**12 个退项任意组合都不返回 `DeviceError::Unsupported`**（退路存在即可走）；降级写入 `FallbackTrace`（能力编号 + 退路名），原生路径不写。

## 5. 门禁

四门禁 `normify gate`（fmt/clippy/clippy-cuda/unit）+ `normify check`；结果 JSON 在 `.normify/runs/2026-11-12-p1c-fallback-impl/`。耗时见提交正文（两轮独立测量：冷跑与收口热跑）。

## 6. 边界与不宣称

- D.4/D.10 **未实现**（`after_shim`），本卡不宣称 12/12 完成；`K.2/K.3/C.1/H.1` 只交付"选哪条路"+宿主侧退路语义，设备端执行属 P2。
- 不接产品调用点；不改 A0 判据/容差；不动 29 `.cu` 冻结身份；0 窗 0 GPU（`gpu_used=false`）。
- mock backend 是测试载体，不是后端实现；CUDA 退路路径的 GPU 执行测试只登记、未运行、不声明通过。
- 不宣称与上游一致、等价或验收通过。

## 7. 复跑

```powershell
cd D:\Rust\mjwarp-rs
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked --lib kernel_exec
cargo test --locked
# GPU 退路可达性（不新开窗；有窗时）
cargo test --locked --lib kernel_exec::cuda -- --ignored --test-threads=1
```

## 8. 本目录文件（L1：不建 manifest、不逐文件 sha）

| 文件 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `items.json` | 12 条逐项六要素（capability_id/behavior/judgement/upstream_anchor/frozen_delta/evidence + impl_scope）与顶层计数 |
| `fallback-tests.log` | `cargo test --locked --lib kernel_exec -- --nocapture` 输出（51 passed / 4 ignored） |
| `unit-summary.log` | 全量 `cargo test --locked` 的 `Running`/`test result:` 摘要 |
