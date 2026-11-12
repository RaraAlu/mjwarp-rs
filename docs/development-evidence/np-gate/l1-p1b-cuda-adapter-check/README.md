# L1 复核：P1-b CUDA 宿主面适配层（S1 `e8ef68e89` + S2 `65c662107`）

- 复核对象：卡 `2026-11-12-g10-p5-kernel-exec-p1b`；证据 `docs/development-evidence/plan/kernel-exec-layer/p1b-cuda-adapter/`。
- 复核基线提交 **`65c662107`**（S2 收口）；只读复核开始时 HEAD 为 `fdd93cf9b`——其后两提交 `ed42f4013`（A0 同步）、`fdd93cf9b`（program v0.2）**仅改 `docs/architecture/` 两文件（L0）**，对 `src/` 零改动。
- 口径：新 L1（一张卡一次 + 风险抽查 30%；必查范围/零行为/判据数字/断言内容/边界/冻结件身份；L1 不要求 manifest）。
- 复核期间工作树被 P1-c 在途改动占用（`fallback/` 未跟踪文件 + `mock_backend.rs`/`mock_tests.rs`/`mod.rs` 未提交；其后 P1-c 继续改到 `cuda/buffer.rs` 等）。**本卡全部结论按提交 `65c662107` 取证**：取证时（P1-c 改动出现前）cuda 7 文件与核心 6 文件与提交逐字节一致；其后 P1-c 对 `cuda/**` 的改动**不属本卡范围**。行数用 `git show 65c662107:`、grep 用 `git grep 65c662107`。**未碰在途文件、未写仓。**
- **结论：PASS（0 阻断 / 0 严重 / 0 一般 + 2 低 + 4 观察）**；低项与观察进 errata 台账批量收口，不阻塞本片。

## 1. 必查项（不抽样）

### 1.1 范围边界 ✅

- 提交范围：S1 `e8ef68e89`（父 `e45b68e7d`）+ S2 `65c662107`。
- **src 改动只在 `src/runtime/kernel_exec/`**：`git diff --name-only 65c662107~2 65c662107 -- src ':(exclude)src/runtime/kernel_exec'` = **空** ✅。
- 零 `.cu`/`*.ptx`/`*.cubin`/`*.mjb`/`abi/`/`fixtures/`/`scripts/`/`Cargo.toml`/`Cargo.lock` 改动 ✅。
- 非 src 改动仅三类：`.normify`（卡/模块索引/runs）、P1-b 证据 5 文件、**P1-a 证据两文件（E-011 行数更正，Chief 已授权「P1-b 首个提交顺手更正」）** ✅。
- S2 新增 cuda 适配 7 文件（`mod/buffer/queue/module/diagnostics/errors/tests`）+ 改 `error.rs`；符合「先包不改」。

### 1.2 零行为变化 ✅

- `kernel_exec` 在 `src/` 内仍**只有模块声明** 1 处：`src/runtime/mod.rs:7 pub mod kernel_exec;`；`git grep` 确认 `src/` 内无其他引用、无调用点 ✅。
- `cuda` 子模块只在 `#[cfg(feature = "cuda-probe")]` 下编译（`kernel_exec/mod.rs` @commit）✅。
- **29 个冻结 `.cu` 逐文件 sha256 = 29/29 全等**：对 `frozen-cu-digest.log` 独立复算，0 失配、0 缺失（树内另有 13 个 render/rne `.cu`，不在冻结集）✅。
- 全量 unit：**1447 passed / 0 failed / 405 ignored**（`unit.json` 与 `unit-summary.log` 146 个 target 求和一致）✅。
  - 更正口头口径：ignored 是 **405**（= 旧 402 + 新增 3 个 `#[ignore]`），非 402；passed 1447 无误。
  - **本卡门禁在登记 HEAD 实测 1461 过 / 0 败**（登记 HEAD 已含 P1-c 的 14 个新测试、ignored 406）；1447/0/405 是复核基线 `65c662107` 的值，两者不冲突。
- **独立复跑** `cargo test --locked --lib kernel_exec`（在 Backend 取锁前、工作树=P1-b 提交态执行）→ **37 passed / 0 failed / 3 ignored / 934 filtered**，与 `adapter-tests.log` 逐字一致 ✅。

### 1.3 行数（≤500）✅

用 `git show 65c662107:<path>` + LF 切分复算（未用 `Get-Content`）：

| 文件 | 行数 |
| --- | ---: |
| `cuda/buffer.rs` | **500**（恰边界） |
| `cuda/mod.rs` | 251 |
| `cuda/queue.rs` | 210 |
| `cuda/module.rs` | 121 |
| `cuda/diagnostics.rs` | 82 |
| `cuda/errors.rs` | 91 |
| `cuda/tests.rs` | 288 |

全部 ≤500，与 README §1 / `artifacts.json::files[]` 逐值一致 ✅。

### 1.4 接口（12 flag / 四签名 / BufferError）✅

- **12 flag 名称 + 能力映射 + 极性**：与 A0（`docs/architecture/device-layer-contract.md` L37–40）逐字一致；极性 **`true`=原生可用 / `false`=走退路**（`backend.rs` 类型文档写死「与直觉相反」）。
- `cuda_fallbacks()` 取值：**6 true**（`transient_alloc`/`generic_device_fn`/`native_source_injection`/`occupancy_hint`/`int64`/`f32_atomic`）、**6 false**（`zero_copy_external`/`graph_capture`/`device_loop`/`device_branch`/`capture_alloc`/`kernel_printf`）；与 README §3 表、`artifacts.json::fallback_flags`、单测 `frozen_fallback_flags_match_p1b_table` 四方一致 ✅。
- 四签名齐备且 A0 已同步：`Module::suggest_block_width(&Kernel, u64) -> Result<BlockSuggestion, DeviceError>`、`ThreadScope::shared(u64) -> Result<SharedBuf, DeviceError>`、`CompileOptions.entry: Option<&'static str>`、`Diagnostics::record_event() -> Result<Event, DeviceError>` ✅。
- `BufferError` 含 `Unsupported` / `ForeignBuffer { backend }` / `DeviceLost(DeviceLost)`；**`DeviceLost` 在 `kernel_exec` 外 0 引用**，仅 `cuda/errors.rs:50` 一处构造、`:88` 一处匹配、`error.rs` 单测一处——**只被 kernel_exec 使用、零外溢** ✅。

### 1.5 CUDA 适配层（unsafe / cudarc / Unsupported）✅

- 真实 `unsafe` **仅 2 处，都在 `cuda/queue.rs`**：`:33 unsafe impl DeviceRepr for DevicePtrArg`、`:180 unsafe { builder.launch(config) }`；**两处均有 SAFETY 前提注释**（repr(transparent) over u64；句柄来自本设备 compile、实参 ABI 由调用方按 CompileOptions 保证、缓冲保活到 launch 返回）✅。`kernel_exec` 内其余 `unsafe` 字样全为注释/文档。
- `cudarc` 只在 `cuda/**`（`kernel_exec/mod.rs:7` 仅文档提及）✅。
- `kernel_exec` 内 `#[allow(...)]` = **0**；无 lint/Cargo 改动 ✅。
- 未接线面不静默降级（逐条核到实现）：B.11 `borrow_external` → `Unsupported("B.11 external-handle")`；K.1 `Queue::replay` → `Unsupported("K.1 replay")`；K.5 `capture_alloc` → `Unsupported("K.5 capture-alloc")`；C.4 复合 DType 7 种 → `Unsupported("C.4 composite-dtype")`；`IndexSpace::D2/D3` → `Unsupported("G.1 multidim-index-space")`；`Arg::Buffer.offset≠0` → `Unsupported("G.3 buffer-offset")`；别名 → `Unsupported("G.5 alias-declaration")`；重复缓冲地址 → `Unsupported("G.5 duplicate-buffer")`；`KernelSource::Named` → `Unsupported("F.1 named-source")`；`CompileOptions.defines` 非空 → `Unsupported("F.5 defines")` ✅。
- 错误映射：驱动/上下文/拷贝/同步失败 → `DeviceLost`；NVRTC/装载/F.6 PTX 闸 → `Compile`；发射配置/参数 → `Launch`；能力缺失 → `Unsupported`；缓冲面驱动失败 → `BufferError::DeviceLost`——三分类不合并 ✅。
- 语义：`submit` 只入队不等待、`wait_idle` 唯一排空点；`Init::Uninit` 用清零（不引入 `unsafe alloc`，不隐式同步）；`copy_range` 同设备同 dtype、自拷贝等范围 no-op、重叠显式失败；两缓冲加锁按地址升序固定顺序（避免互等）✅。

### 1.6 证据真实性 ✅

逐个独立复算（非采信自述）：

| 输入件 | 期望 | 实测 |
| --- | --- | --- |
| A0 @ `d9c863d45` | `412743…` / 26,274 B | 一致 |
| A1 盘点 | `554267971fe2dc69…` / 37,413 B | 一致 |
| P1-c 退项草案 | `7b3cf603…` / 12,030 B | 一致 |
| P1-b prep | `4e06960a…` / 24,277 B | 一致 |
| `refreeze-10-exe.json` | `36195c24…` / 8,955 B | 一致 |

- `unit-summary.log` 求和 = `unit.json::tests` = 1447/0/405；`adapter-tests.log` 37/3 与独立复跑一致；`frozen-cu-digest.log` 29/29 与复算一致 ✅。
- **L1 不建 manifest、不逐文件 sha 不算缺项** ✅。
- 3 个 `#[ignore]`（`cuda/tests.rs:203/217/278`）**只登记、未运行**；README/artifacts 明写「本片未运行、不声明通过、不开窗」✅。

### 1.7 卡状态 / E-011

- P1-b 卡 `verified`，5 checks（4 gate + `unchanged` 覆盖 `tests/`、`fixtures/`、`src/physics/`、`src/render/`、`src/io/`、`src/model/`、`src/runtime/transfer/`、`src/runtime/launcher/`、`src/runtime/cuda/`——**旧 cudarc/launcher/transfer 路径机检覆盖**）；`revision.before = e45b68e7d`（父提交）✅。
- **E-011（P1-a 证据行数 +1）已在 S1 更正**：P1-a README 与 `artifacts.json` 的 10 个值现为 200/159/168/266/456/212/36/86/151/125、`max_file_lines=456`，与我 P1-a 复核实测一致 → 台账可标 **DONE**（提交 `e8ef68e89`）。

## 2. 抽样清单

| # | 对象 | 范围 | 方法 | 结果 |
| --- | --- | --- | --- | --- |
| 1 | `cuda/**` 7 文件 | 全部逐行（超出 30% 要求） | 提交内容通读 | ✅ |
| 2 | trait 实现方法 | 7 个 impl（Backend/Device/Allocator/Buffer/Queue/Module/Diagnostics） | 逐方法对照 A0 | ✅ |
| 3 | Unsupported 分支 | 10 条 | 逐条核到调用点 | ✅ |
| 4 | 错误三分类路径 | DeviceError/BufferError 映射 | 读映射函数 + 单测 | ✅ |
| 5 | 12 flag 取值理由 | 12 条 | 对 A0/P1-c 草案/代码 | ✅ |
| 6 | 测试清单 | 40 个（37 pass + 3 ignore） | 独立复跑核对 | ✅ |
| 7 | mock 双侧失败路径 | shared 2 条路径 | 读实现 + 断言 | ✅ |

## 3. 发现

### 低（在 `65c662107` 发现；已由 Backend 在 P1-c 首提交 `e8ec1f2f1` 修正，本卡复核到位）

- **L1 门禁耗时与 `.normify/runs` 现值不一致**（原状）：README §6 与 `artifacts.json::gates` 写 fmt 2,013 / clippy 5,328 / clippy-cuda 455 / unit 101,440 ms，而 `runs/*.json` 的 `wall_ms` 为 1,979 / 456 / 455 / 16,845（`at` 15:57:5x–15:58:13），未标注轮次。
  **修正核过**：README §6 已加「两轮独立测量」引用块（cold=首次全量编译 unit 101,440 ms；hot=收口前热跑 1,979/456/455/16,845 ms，两轮分开记录不互相覆盖）；`artifacts.json::gates` 已改成 `rounds.cold` / `rounds.hot` + `note`，「`.normify/runs` 为 hot、README §6 为 cold」写清 ✅。
- **L2 「显式 Unsupported / unsafe 一处」口径不准**（原状）：README §7 与 `artifacts.json::unsupported_surface` 把 L.3 `printf` 归入「显式 Unsupported」（实为签名无错误通道的显式 no-op）、把 K.1 `QueueCapture::capture` 写成 Unsupported（实为未实现该 trait，只有 `replay` 返回 Unsupported）、并称 unsafe「集中在 launch 一处」（实为 2 处）。
  **修正核过**：README §7 已拆成「① 显式 `Unsupported` / ② 签名无错误通道 → 显式 no-op（L.3）/ ③ 未实现该 trait（K.1 capture）」三类，并单列「unsafe 实为 2 处（`queue.rs:33`、`:180`，各带安全前提注释）」；`artifacts.json::unsupported_surface` 同步改三分列表 ✅。

### 观察（进台账）

- **O1**：A0（HEAD）仍写「`BufferError`（P1-b 冻结 **5 类**）」，实现含第 6 项 `BufferError::DeviceLost`（D1 扩展）。**证据侧已补注**（`e8ec1f2f1`：README §7/artifacts `d1_extension.status` 均写「已获 Chief 追认，A0 由 Arch 补一句」）；**A0 本体仍待 Arch 同步**（非 P1-b 缺陷）。
- **O2**：`unit.json::log_sha256=4007ad54…` 与 `unit-summary.log` sha256 `9e752c8a…` 不同。**已在 `e8ec1f2f1` 补注关系**（README §6 引用块 + `artifacts.json::gates.log_relation`：gate 原生日志 vs 过滤摘要，非矛盾/非改写）✅。
- **O3**：`unknown_kernel_handle_is_launch_error` 为手工构造 `DeviceError::Launch(InvalidArgument)` 再断言 `class()`，**未走真实路径**；`Module::suggest_block_width` 的 unknown-handle 与 occupancy 查询失败（→`DeviceLost`）无 0 GPU 覆盖 → 记为 **0 GPU 覆盖缺口**（需驱动，不在本片声明；Open）。
- **O4**：`Diagnostics::elapsed` 除未知句柄外，还把驱动错误/非有限值吞成 `Duration::ZERO`（P1-a 签名无错误通道的既定取舍，README 已披露）→ 建议列为「事件计时错误不可观测」风险项，转 Arch 在 A0/后续片说明（Open）。

## 4. 结论

**PASS（0 阻断 / 0 严重 / 0 一般 + 2 低 + 4 观察）。** 范围、零行为、行数、接口、CUDA 适配层 unsafe/cudarc/Unsupported、证据真实性、冻结身份与 3 个 ignore 登记均通过。
2 个低项（门禁两轮标注、Unsupported/unsafe 口径）**已在 P1-c 首提交 `e8ec1f2f1` 修正并经本卡复核到位**；O1/O2 的证据侧补注亦已核对；O1 的 A0 本体同步与 O4 的计时错误可观测性转 Arch，O3 记为 0 GPU 覆盖缺口。本卡结论对提交 `65c662107` 成立（证据文案修正不影响该提交的代码与判据）。

## 5. 过程事实（不处置）

- 复核期间 Backend 持锁做 P1-c：先见 `fallback/` 7 新文件 + `mock_backend.rs`/`mock_tests.rs`/`mod.rs` 未提交，其后 HEAD 推进到 `e8ec1f2f1` 并继续修改 `cuda/buffer.rs`/`diagnostics.rs`/`mod.rs`/`queue.rs`/`tests.rs` 与新增 `cuda/buffer_codec.rs`、`cuda/fallback_plan.rs`。**这些属 P1-c，不在本卡范围**；本卡结论只对提交 `65c662107` 成立。
- 我未碰在途文件、未写仓；写卡等待锁 FREE 且树 clean 后执行。

## 6. 非声明

本复核只对提交 `65c662107`（S1+S2）承担范围/行为/接口/证据核对责任；**不宣称**任何能力已实现或已验收、不宣称 mesh 容量缺口已清、不作与上游等价（over-32）或整族通过声明、不宣称 NP01/NP02 通过、**不作位级等价声明**、不提供位级等价保证；GPU 等价测试 3 个 `#[ignore]` 未运行，零 GPU、零窗；不改判据/参考/容差/夹具/源码。
