# L1 复核：P1-c 退路实现与 DeviceCaps.fallbacks 双路（S1 `e8ec1f2f1` + S2 `e8f07706c`）

- 复核对象：卡 `2026-11-12-p1c-fallback-impl`；证据 `docs/development-evidence/plan/kernel-exec-layer/p1c-fallback-impl/`。
- 复核基线 **`e8f07706c`**（S2 收口）；只读复核期间后续提交均为文档/其它片，不影响本卡。
- 口径：L1 一次复核 + 风险抽查；必查范围/12 flag 表/4096 组合覆盖/FallbackTrace 留痕/10 项接线/unit/行数/冻结边界；L1 不建 manifest。
- **结论：PASS（0 阻断 / 0 严重 / 0 一般 / 0 低 + 6 观察）**；观察进 errata 台账，不阻塞。

## 1. 必查项

### 1.1 范围边界 ✅

- src 改动只在 `src/runtime/kernel_exec/`（`git diff --name-only e8ec1f2f1~1 e8f07706c -- src ':(exclude)src/runtime/kernel_exec'` = 空）；零 `*.cu`/`*.ptx`/abi/fixtures/scripts/Cargo 改动 ✅。
- 非 src：`.normify`（卡/模块索引/runs）、P1-c 证据 4 文件、**P1-b 证据 README/artifacts（复核低项 L1/L2 文案修正，Chief 已定）** ✅。
- `kernel_exec` 在 src 内仍只有 `pub mod` 声明 1 处（`src/runtime/mod.rs:7`），未接调用点 ✅。

### 1.2 12 flag 表与卡/items.json 一致 ✅

- `items.json`：**12 条**、id 顺序与 A0 一致（B.10/B.11/D.4/D.10/G.9/K.1/K.2/K.3/K.5/L.3/C.1/H.1）；`impl_scope` = **10 now + 2 after_shim**（D.4/D.10）；`level` ∈ {退, 必带退}。
- 卡 acceptance 与 README §1 同步写死「10/12 实现、D.4/D.10 after_shim 未实现、**不是 12/12**」；D.4/D.10 的 `behavior` 明写「未实现」，未出现「已实现」表述。
- 代码侧注册表 `fallback/mod.rs`：`FallbackCapability::ALL` 12 项、`impl_scope()` D.4/D.10 = `AfterShim`；单测 `registry_has_12_flags_with_10_now_and_2_after_shim` 断言。
- `cuda_fallbacks()` 极性与取值未在 P1-c 改动（仍 P1-b 冻结 6 true/6 false）✅。

### 1.3 4096 组合覆盖（代码级核）✅（口径见 O1）

- `fallback/tests.rs::any_combination_of_12_flags_has_no_unsupported_path`：`for mask in 0u16..(1 << 12)` **真枚举 4096**，逐 bit 写满 12 个 flag，再对 **12 个能力** 各调 `select_native`，断言 `is_native == capability.is_native(set)` 且 `trace.is_empty() == is_native`、`trace.len() == !is_native` → 共 4096×12 次断言 ✅。
- 「不返回 Unsupported」在选择层是**结构性成立**（`select_native` 返回 `bool`，无错误通道），不是靠断言某次调用不报错。

### 1.4 FallbackTrace 留痕 ✅

- `select_native` 仅在 `!native` 时 `trace.record`；`fallback/graph.rs` 的两处 record 都在退路实现函数内（`capture_sequential`/`replay_sequential`），生产侧只在 `GraphMode::SequentialReplay` 分支调用。
- 4096×12 断言与各双路测试均断言「原生路径 `trace.is_empty()`」✅。重复留痕见 O2。

### 1.5 10 项 CUDA 侧接线（6 项产品接线 + 4 项选择冻结）

| 能力 | CUDA 侧 | 说明 |
| --- | --- | --- |
| B.10 | 产品接线 | `CudaAllocator::transient` → `transient_mode`；前置 `CudaDevice::preallocate_transient` |
| B.11 | 产品接线 | `borrow_external` → `external_borrow_mode`；`register_external` |
| G.9 | 产品接线 | `CudaModule::suggest_block_width` 按 caps flag 分支（P1-b 既有）+ `occupancy_mode`（测试） |
| K.1 | 产品接线 | `QueueCapture::capture`/`Queue::replay` → `graph_mode`；原生未实现时显式 Unsupported |
| K.5 | 产品接线 | `capture_alloc` → `capture_alloc_mode`；`preallocate_capture` |
| L.3 | 产品接线 | `printf` → `printf_mode` + 状态缓冲 `drain_status_buffer` |
| K.2/K.3/C.1/H.1 | **仅选择冻结** | `loop_mode`/`branch_mode`/`int64_lowering`/`f32_atomic_mode` 为 `#[cfg(test)]`；设备端执行属 P2（README §3/§6 已限定） |

### 1.6 测试与门禁 ✅

- 独立复跑 `cargo test --locked --lib kernel_exec` → **51 passed / 0 failed / 4 ignored / 934 filtered**，与 `fallback-tests.log` 逐字一致（cargo exit 0）✅。
- 4 个 `#[ignore]` = P1-b 3 个 + P1-c 新增 1 个（`gpu_cuda_fallback_paths_are_reachable`），**只登记未运行、不开窗**，证据明写不声明通过 ✅。
- 全量 unit：**1461 passed / 0 failed / 406 ignored**（`unit.json` 与 `unit-summary.log` 146 个 target 求和一致）✅。
- 新增测试 14 个（fallback 13 + cuda 1）与 README §4 一致 ✅。

### 1.7 行数与冻结边界 ✅

- 全量 >改动文件用 `git show e8f07706c:` + LF 复算：**max = mock_backend.rs 468**（其余 fallback/tests 453、cuda/buffer 454、cuda/mod 294、queue 294、tests 389、mod 44…），全部 ≤500 ✅。
- 29 `.cu` 冻结集、recipe/PTX/ABI 零改动（路径 diff 空）✅；`gpu_used=false`。

## 2. 抽样清单

| # | 对象 | 覆盖 |
| --- | --- | --- |
| 1 | fallback 7 文件 + cuda 新增/改动 6 文件 | 逐文件读（超 30%） |
| 2 | 12 项 items 六要素 | 全量对代码/README/A0 |
| 3 | 双路测试 14 个 | 名称+断言抽样（b10/b11/k1/k5/l3/g9/c1/h1 + 组合/无同步） |
| 4 | 退路实现无 panic | `unwrap/expect/panic!` 在 fallback 生产文件 = 0 |
| 5 | policy_scan | 4 证据文件 0 命中 PASS |

## 3. 发现（6 观察，无低/无阻断）

- **O1 组合规则口径分层**：README §4「12 退项任意组合都不返回 `DeviceError::Unsupported`」在选择层成立（`select_native` 无错误通道）；但**适配层**在 flag=true 而原生未实现时仍返回 Unsupported（如 `K.1 native-capture`、`B.11 native-zero-copy`）。默认 `cuda_fallbacks()` 这两项为 false，默认设备不触发；建议证据把「选择层规则」与「适配层原生未实现」两层口径分开写。
- **O2 K.1 退路重复留痕**：`CudaQueue::capture` 先 `graph_mode()`（记 1 条）再 `capture_sequential()`（再记 1 条）→ 一次 capture 记 **2 条** GraphCapture；若后续用 trace 计数做判据会偏大（测试未覆盖 CUDA 侧 trace 计数）。
- **O3 B.11 退路含阻塞拷贝**：CUDA `borrow_external` 的 Copy 路径用 `memcpy_dtod_sync`（阻塞 DtoD），并用 `usize::try_from(bytes).unwrap_or(usize::MAX)` 兜底；证据只写「无 wait_idle」，未提阻塞拷贝，建议在退路语义里注明「拷贝为同步 DtoD」。
- **O4 B.10 退路池不复用 + byte_len 语义**：CUDA transient 池 `swap_remove` 取走不归还（无 release API），且返回缓冲 `byte_len` 用**池槽容量**（≥请求字节）而非请求值，与 `alloc` 的 byte_len 语义不一致；属 P1-c 简化，建议注明 P2 池化。
- **O5 H.1「双路」同实现**：`native_add` 与 `cas_loop_add` 都走同一 CAS 循环（`retry_loop` 仅注释区分），设备侧原生属 P2；建议在证据里把「入口区分」写清。
- **O6 10 项接线强度不一（Chief 已裁口径）**：统一口径为「**10/12 = 选择层实现**；其中 **6 项产品接线**（B.10/B.11/G.9/K.1/K.5/L.3），**4 项（K.2/K.3/C.1/H.1）选择函数仅测试态、设备端执行归 P2**」。不返工；P1-c 证据措辞由 Backend 在 P2-b 首提交顺手改，P2-b 范围扩为「D.4/D.10 + K.2/K.3/C.1/H.1 的设备端执行」。

## 4. 结论

**PASS（0 阻断 / 0 严重 / 0 一般 / 0 低 + 6 观察）。** 范围、12 flag 表（10 now/2 after_shim）、4096 组合代码级枚举、FallbackTrace 仅降级写、**10/12 选择层实现（6 项产品接线 + 4 项仅测试态选择，设备端执行归 P2，Chief 已裁口径）**、unit 1461/0/406、51/4 独立复跑、行数 ≤500、冻结边界与 P1-b 证据文案修正均核过；观察项进 errata 台账。

## 5. 非声明

**不宣称** 12/12 完成、D.4/D.10 已实现、任何能力已实现或已验收、不宣称缺口已清、不作与上游等价（over-32）或整族通过声明、不宣称 NP01/NP02 通过、**不作位级等价声明**、不提供位级等价保证；4 个 `#[ignore]` GPU 测试未运行，零 GPU、零窗；不改判据/参考/容差/夹具/源码。
