# 设备层契约（A0，草案 v0.2）

- 状态：草案 v0.2（2026-11-12 同步 P1-b S1 `e8ef68e89` 的实际签名与当日裁定措辞；L0 文档，未登记卡、未开窗、未改实现）。目标提交：`docs/architecture/device-layer-contract.md`。
- 依据：`docs/implementation-plan.md` §并行计算单元能力定义（`efadf21c`，13 域 85 项，其中设备层 44 项）；映射与口径见 `target/impl-plan-review/`（工作材料，不入仓）。
- 目的：把**设备通用**的内核执行层所需能力收敛成一份可实现的接口契约，使「接新设备只需重做设备层（44）」成为可执行目标，而不是口号。

## 1. 目的、范围与边界

**在范围内**：设备发现/能力查询、缓冲与传输、内核编译与发射、队列与同步、执行控制（捕获/条件执行）、诊断、错误模型；以及承载这些能力的**内核书写面**（语言层 23 项的意图接口）。

**不在范围内**：库层 17 项（块级算子、扫描/排序、BVH/纹理等）的算法实现；业务层物理/渲染语义；数值判据本身（仍由精度契约与 L2 证据管）。

**边界一句话**：语言层 + 库层写成**一份设备无关源**；设备层由每个后端各实现一次。当前实现是「Rust 宿主 + 手写 CUDA C++ + Warp 1.15 头树」，属于**第一个后端**的落地，不是契约本身。

## 2. 设备层 API 契约（Rust 形式）

以下只定方法与语义，不定 ABI；实现可以内部替换（cudarc/NVRTC/预编译 PTX/其他）。

```rust
// ── 与 P1-a/P1-b 实际签名对齐（2026-11-12；trait object 一律 `Box<dyn …>`，缓冲面错误走 `BufferError`）──

// 1) 发现与能力
pub trait Backend {
    fn devices(&self) -> Result<Vec<DeviceId>, DeviceError>;            // A.1
    fn open(&self, id: &DeviceId) -> Result<Box<dyn Device>, DeviceError>;
}

pub struct DeviceCaps {
    pub fallbacks: FallbackSet,        // A.2：退项是否有原生实现（**true=原生 / false=走退路**）
    pub max_index_dims: u8,            // 1..=3（G 域容量）
    pub max_block_threads: u32,        // G 域容量
    pub max_blocks: u32,               // G 域容量
    pub shared_bytes_per_block: u32,   // I 域容量
    pub max_buffer_bytes: u64,         // B 域容量
    pub numeric: NumericPolicy,        // F.6：f32 / FMA / fast-math 关闭
}
// FallbackSet（P1-b 冻结 12 flag；极性 **true=原生可用 / false=走退路**，与直觉相反）：
//   transient_alloc(B.10)  zero_copy_external(B.11)  generic_device_fn(D.4)
//   native_source_injection(D.10)  occupancy_hint(G.9)  graph_capture(K.1)
//   device_loop(K.2)  device_branch(K.3)  capture_alloc(K.5)  kernel_printf(L.3)
//   int64(C.1，必带退)  f32_atomic(H.1，必带退)
// 「必带退」= 能力必级、退法常备（int64=false → 双 32 位；f32_atomic=false → CAS 循环）。

pub trait Device {
    fn caps(&self) -> DeviceCaps;                                       // A.2
    fn queue(&self) -> Result<Box<dyn Queue>, DeviceError>;             // A.3 载体
    fn allocator(&self) -> Box<dyn Allocator>;                          // B 域
    fn module(&self) -> Box<dyn Module>;                                // F 域
}

// 2) 缓冲（B 域）：句柄面对象安全；参数错误用 `BufferError`（不是 `DeviceError`）
pub enum Init { Uninit, Zeroed, Fill(u64) }
pub struct Address64(pub u64);   // B.3/B.9：64 位字节地址
pub enum DType { /* P1-a 定稿 14 种：标量 7 + 宿主缓冲可见复合 7 */ }

pub trait Buffer {
    fn addr(&self) -> Address64;                                        // B.3/B.9
    fn byte_len(&self) -> u64;
    fn copy_range(&self, dst: &dyn Buffer, dst_off: u64,
                  src_off: u64, len: u64) -> Result<(), BufferError>;   // B.5
    fn upload(&self, host: &[u8], off: u64) -> Result<(), BufferError>; // B.6
    fn readback(&self, host: &mut [u8], off: u64) -> Result<(), BufferError>; // B.6
    fn pin(&self) -> Result<Pinned, BufferError>;                       // B.9
}

pub trait Allocator {
    fn alloc(&self, dtype: DType, len: u64, init: Init) -> Result<Box<dyn Buffer>, BufferError>; // B.1
    fn transient(&self, bytes: u64) -> Result<Box<dyn Buffer>, BufferError>;     // B.10 退：预分配
    fn borrow_external(&self, h: ExternalHandle) -> Result<Box<dyn Buffer>, BufferError>; // B.11 退：拷贝
}
// `DType`（P1-a 定稿，14 种）：标量 7 = f32/i32/u32/u64/i64/u8/bool；
// 宿主缓冲可见复合 7 = Vec2/Vec3/Vec4/Vec6/Vec10/Mat33/Quat。
// `BufferError`（P1-b 冻结 5 类）：`CapacityExceeded` / `OutOfRange` / `Unaligned` /
// **`Unsupported(CapabilityId)`** / **`ForeignBuffer { backend }`**。

// 3) 编译与发射（F/G 域）
pub trait Module {
    fn compile(&self, src: KernelSource, opts: CompileOptions) -> Result<Kernel, DeviceError>; // F.1/F.5
    // G.9：原生查询需要已编译函数句柄，故挂在本 trait（不是纯描述符 `Launch`）
    fn suggest_block_width(&self, kernel: &Kernel, work_items: u64)
        -> Result<BlockSuggestion, DeviceError>;
}
pub struct CompileOptions {
    pub numeric: NumericPolicy,
    pub entry: Option<&'static str>,                 // F.5：`Inline` 必填；`Named` 由后端按名解析
    pub defines: Vec<(&'static str, &'static str)>,  // F.5：**集合由后端声明，超集 → `Unsupported`**
}
pub struct BlockSuggestion { pub block_width: u32, pub min_blocks: u32 }
// 退路固定值：`BlockSuggestion::FALLBACK = { block_width: 256, min_blocks: 1 }`

pub struct Launch {
    pub kernel: Kernel,
    pub dims: IndexSpace,          // G.1：D1/D2/D3
    pub worlds: u32,               // G.2：总索引 = worlds × elements
    pub elements: u32,
    pub blocks: u32,               // G.6
    pub block_width: u32,          // G.7
    pub args: Vec<Arg>,            // G.3
    pub aliases: Vec<Alias>,       // G.5：同缓冲 in/out
    pub numeric: NumericPolicy,    // F.6
}

pub trait Queue {
    fn submit(&self, l: Launch) -> Result<Submission, DeviceError>;     // A.3/G.10
    fn wait_idle(&self) -> Result<(), DeviceError>;                     // L.1
    fn replay(&self, g: &Graph) -> Result<(), DeviceError>;             // K.1
    fn capture_alloc(&self) -> Result<Box<dyn Buffer>, DeviceError>;    // K.5 退：预分配
}
// K.1 的 capture 需要闭包，按 P1-a 定稿拆为独立 trait：仅单态化调用、不作 `dyn` 面，
// `Queue` 保持对象安全。A0 定稿采纳该形态，不改回放进 `Queue`。
pub trait QueueCapture {
    fn capture<F>(&self, body: F) -> Result<Graph, DeviceError>
    where
        F: FnOnce(&dyn Queue);
}

// 4) 设备内原语（内核书写面经此落到具体设备）
pub trait DeviceMath {          // E.1/E.2
    fn sqrt(x: f32) -> f32; fn sin(x: f32) -> f32; /* … 19 个 */
}
pub trait Atomics {             // H.1/H.3/H.4/H.5/H.6（P1-a 定稿：用 Address64，不用裸指针）
    fn add_i32(&self, target: Address64, value: i32) -> i32;
    fn add_f32(&self, target: Address64, value: f32) -> f32;      // 退：CAS 循环
    fn sub_i32(&self, target: Address64, value: i32) -> i32;
    fn or_u32(&self, target: Address64, value: u32) -> u32;
    fn min_i32(&self, target: Address64, value: i32) -> i32;
    fn max_i32(&self, target: Address64, value: i32) -> i32;
    fn cas_i32(&self, target: Address64, compare: i32, value: i32) -> i32;
}
pub trait ThreadScope {         // I.1/I.2/I.3
    fn tid(&self) -> Dim3; fn block_dim(&self) -> Dim3;
    // I.2（P1-b 冻结）：超 `shared_bytes_per_block` → `Launch(InvalidConfig)`；
    // 后端无块共享 → `Unsupported("I.2")`
    fn shared(&self, bytes: u64) -> Result<SharedBuf, DeviceError>;
    fn barrier(&self);
}

// 5) 执行控制与诊断
pub trait ExecControl {         // K.2/K.3（K.4 属库层：探测与降级）
    fn loop_while_device(&self, c: &Counter, body: impl FnMut()) -> Result<(), DeviceError>;
    fn branch_on_device(&self, c: &Counter, then: impl FnOnce(), els: impl FnOnce()) -> Result<(), DeviceError>;
}
pub trait Diagnostics {         // L.2/L.3
    fn record_event(&self) -> Result<Event, DeviceError>;  // 冻结：驱动失败显式返回，不用哨兵句柄
    fn elapsed(&self, a: &Event, b: &Event) -> Duration;
    fn printf(&self, fmt: &str, args: &[Arg]);   // 退：状态缓冲 + 宿主打印
}

// 6) 错误模型（A.4）
pub enum DeviceError {
    Compile(CompileError), Launch(LaunchError), DeviceLost(DeviceLost),
    Unsupported(CapabilityId),   // 能力缺失且无退路（不属于三分类）
}
// 后端可声明「不产生 DeviceLost」（如 CPU 后端）；三分类不得合并。
```

设计约束（契约层）：

1. **退路显式**：每个「退」项在 `DeviceCaps.fallbacks` 中有布尔位；缺失且无退路 → `Unsupported`，不得静默降级（K.4 的统一入口）。
2. **不可隐式同步**：`transient()`、`upload/readback` 默认异步；需要同步必须显式 `wait_idle()`。
3. **数值策略随发射传递**：`NumericPolicy` 是 `Launch`/`compile` 的必填项，后端不得自行改 rounding/FMA/fast-math。
4. **错误三分类**：编译/发射/设备丢失不合并（A.4）。
5. **F.1 语义（2026-11-12 裁）**：把内核源变成可执行物——**JIT 或 AOT 均可**；CUDA 仍走 NVRTC（零行为变化），AOT 后端只支持编译期可知的 `defines` 组合（P5 记录）。
6. **F.5 后端受限（2026-11-12 裁）**：`defines` 集合由后端声明，超集 → `Unsupported`；不得静默忽略。
7. **A.4（2026-11-12 裁）**：后端可声明不产生 `DeviceLost`（如 CPU），但三分类仍不得合并。

## 3. 44 项逐项映射

级别与备注取自 `docs/implementation-plan.md`（必/退/射；射=0 项在设备层）。

| 编号 | 能力 | 级别 | 契约位置 | 备注 |
| --- | --- | --- | --- | --- |
| `A.1` | 枚举并选择设备 | 必 | `Backend::devices()` | 枚举并选择设备；返回稳定 id + 名称 |
| `A.2` | 能力查询：返回各“退”项有无与B、G、I域容量 | 必 | `Device::caps() -> DeviceCaps` | 退项有无 + B/G/I 容量；退路必须显式声明 |
| `A.3` | 执行队列：同一队列按提交顺序执行 | 必 | `Queue::submit()` 语义 | 同一队列按提交顺序执行 |
| `A.4` | 错误模型：区分编译失败、发射失败与设备丢失 | 必 | `DeviceError`（三分类 + `Unsupported`） | 编译失败 / 发射失败 / 设备丢失，不得合并；后端可声明不产生 `DeviceLost` |
| `B.1` | 按元素类型与个数分配：未初始化、清零、填值 | 必 | `Buffer::alloc(dtype,len,Init)` | Init = Uninit | Zeroed | Fill(value) |
| `B.2` | 生命期保持到最后一次使用完成 | 必 | `Buffer` 生命期（RAII） | 持有到最后一次使用完成（Drop/lease 维护） |
| `B.3` | 缓冲内字节偏移采用64位 | 必 | `Buffer::addr() -> Address64` | 字节偏移一律 64 位；元素步长可 32 位 |
| `B.5` | 设备内拷贝，带源偏移、目标偏移与长度 | 必 | `Buffer::copy_range(dst,dst_off,src_off,len)` | 设备内拷贝，带源/目标偏移与长度 |
| `B.6` | 宿主上传与范围回读 | 必 | `Buffer::upload(&[u8]) / readback(&mut [u8])` | 宿主上传与范围回读 |
| `B.9` | 地址稳定：生命期内不搬移 | 必 | `Buffer::pin()` / 地址稳定契约 | 生命期内不搬移；pin 后可把地址交给外部 |
| `B.10` | 临时分配不触发同步 | 退：预分配 | `Allocator::transient(bytes)`（flag `transient_alloc`） | 退：预分配；不得在分配时隐式同步 |
| `B.11` | 把缓冲零拷贝借给外部框架 | 退：拷贝 | `Allocator::borrow_external(ExternalHandle)`（flag `zero_copy_external`） | 退：拷贝；未注册外部句柄 → `BufferError::ForeignBuffer` |
| `C.1` | 标量7种：f32、i32、i64、u8、u32、u64、bool | 必；64位整数可退：双32位 | `DType`（P1-a 定稿 14 种） | 标量 7 + 宿主缓冲可见复合 7（Vec2/3/4/6/10、Mat33、Quat）；64 位整数可退双 32 位（flag `int64`） |
| `D.1` | 控制流：动态上界循环、while、break、continue、提前返回 | 必 | `shim::device_flow`（设备侧 shim，P2） | 落点=设备侧 shim（P2），不塞宿主 trait |
| `D.2` | 设备函数，可互相调用 | 必 | `shim::device_fn`（设备侧 shim，P2） | 落点=设备侧 shim（P2）；设备函数可互相调用、单函数参数上限；flag `generic_device_fn` |
| `D.5` | 整数运算：取模、整除、位与、位或、异或、左移、右移 | 必 | `shim::int_ops`（设备侧 shim，P2） | 落点=设备侧 shim（P2），不塞宿主 trait |
| `E.1` | 标量13个：max、min、abs、clamp、sqrt、sign、pow、exp、log、floor、ceil、round、isnan | 必 | `DeviceMath`（标量 13） | max/min/abs/clamp/sqrt/sign/pow/exp/log/floor/ceil/round/isnan |
| `E.2` | 三角6个：sin、cos、tan、asin、acos、atan2 | 必 | `DeviceMath`（三角 6） | sin/cos/tan/asin/acos/atan2；行为随 NumericPolicy |
| `F.1` | 把内核源变成设备可执行物 | 必 | `Module::compile(KernelSource, CompileOptions)` | **源 → 可执行物（JIT 或 AOT 均可）**；CUDA 走 NVRTC，AOT 后端受 defines 限制（2026-11-12 裁） |
| `F.5` | 每内核可设编译选项 | 必 | `CompileOptions { numeric, entry, defines }` | 每内核可设选项；`defines` 集合由后端声明、超集 → `Unsupported`（2026-11-12 裁） |
| `F.6` | 数值选项：全程f32、乘加融合开、不开fast-math | 必 | `NumericPolicy` | 固定 f32、FMA 开、fast-math 关；随证据冻结 |
| `G.1` | 1、2、3维索引空间，线程可取自身多维索引 | 必 | `Launch::dims(IndexSpace1|2|3)` | 1/2/3 维索引空间，线程可取多维索引 |
| `G.2` | 总索引数等于世界数乘元素数 | 必 | `Launch::elements_per_world / worlds` | 总索引数 = 世界数 × 元素数 |
| `G.3` | 单内核参数个数 | 必 | `Launch::args(&[Arg])` | 内核参数数组上限（数组 ≥73、总数 ≥80 实测；产品 ABI 81 槽） |
| `G.5` | 同一缓冲可同时作输入与输出 | 必 | `Launch::alias(inout, out)` | 同缓冲可同时作输入与输出（显式声明） |
| `G.6` | 按块发射：指定块数，每块多线程 | 必 | `Launch::blocks(u32)` | 按块发射：指定块数，每块多线程 |
| `G.7` | 每次发射指定块宽 | 必 | `Launch::block_width(32|64|128|256)` | 每次发射指定块宽 |
| `G.9` | 建议块宽查询与最小驻留块数提示 | 退：固定值 | `Module::suggest_block_width(kernel, work_items) -> Result<BlockSuggestion, DeviceError>`（P1-b 已落） | 退：`BlockSuggestion::FALLBACK = {256,1}`（flag `occupancy_hint`） |
| `G.10` | 非阻塞提交，队列内有序 | 必 | `Queue::submit()`（非阻塞） | 提交不等待完成，队列内有序 |
| `H.1` | 原子加，返回旧值 | 必；f32可退：比较交换循环 | `Atomics::add_i32/add_f32(Address64, val)` | i32/f32；f32 退：比较交换循环（flag `f32_atomic`，必带退） |
| `H.3` | 原子减 | 必 | `Atomics::sub(ptr,val)` | i32/f32 |
| `H.4` | 原子或 | 必 | `Atomics::or(ptr,val)` | i32/u32 |
| `H.5` | 原子极小与极大 | 必 | `Atomics::min/max(ptr,val)` | i32 |
| `H.6` | 比较交换 | 必 | `Atomics::cas(ptr,old,new)` | i32；f32 回退基石 |
| `I.1` | 取块内线程号与块宽 | 必 | `ThreadScope::tid() -> Dim3 / block_dim()` | 块内线程号与块宽 |
| `I.2` | 块共享存储 | 必 | `ThreadScope::shared(bytes) -> Result<SharedBuf, DeviceError>`（P1-b 已落） | 超容量 → `Launch(InvalidConfig)`；后端无块共享 → `Unsupported("I.2")` |
| `I.3` | 块内同步栅栏 | 必 | `ThreadScope::barrier()` | 块内同步栅栏（显式） |
| `K.1` | 录制一段发射序列并重放 | 退：逐次提交 | `QueueCapture::capture(fn)` / `Queue::replay(&Graph)`（flag `graph_capture`） | 录制一段发射序列并重放 |
| `K.2` | 设备端条件循环，条件为设备上一个i32 | 退：固定次数 | `ExecControl::loop_while_device(counter, body)` | 退：固定次数；设备端条件循环（flag `device_loop`） |
| `K.3` | 设备端条件分支 | 退：无条件执行（flag `device_branch`） | `ExecControl::branch_on_device(counter, then, else)` | 退：无条件执行 |
| `K.5` | 录制期间允许分配，缓冲随录制物存活 | 退：录制前预分配 | `Queue::capture_alloc()`（flag `capture_alloc`） | 退：录制前预分配 |
| `L.1` | 显式等待队列排空 | 必 | `Queue::wait_idle()` | 显式等待队列排空 |
| `L.2` | 设备端事件与耗时 | 必 | `Diagnostics::record_event/elapsed` | 事件句柄 opaque；`record_event` 失败显式返回 `DeviceError` |
| `L.3` | 内核内格式化打印 | 退：状态缓冲 | `Diagnostics::printf(fmt, args)`（flag `kernel_printf`） | 退：状态缓冲回写，由宿主打印 |

## 4. 两个实现的适配说明

### 4.1 后端一：CUDA（现状路径）

| 契约面 | 现状落点 |
| --- | --- |
| 发现/能力（A.1/A.2） | cudarc 0.18.2 driver；能力表需补全 `DeviceCaps`（现仅零散探测，`runtime/cuda/resources/probe.rs`） |
| 编译（F.1/F.5/F.6） | `runtime/launcher/artifact/nvrtc.rs` + `recipe.rs`（4 套 recipe，fast-math 关、fmad 开、arch=compute_70） |
| 发射/队列（A.3/G.*/K.1/L.1） | `runtime/launcher/{submit,graph,artifact}`；graph capture/instantiate 已有；`G.9` occupancy 查询缺 |
| 缓冲（B 域） | `runtime/lease.rs`、`runtime/transfer/*`、`runtime/external/view.rs`；`B.10` 临时分配缺池化 |
| 原子（H 域） | 内核内 `atomicAdd/atomicOr` + 内联 PTX；缺 `H.2/H.3/H.5/H.6` 的统一封装 |
| 线程/共享（I 域） | `wp::tid` + `__syncthreads`；`I.2` 无产品内核用例，`abi.rs` 已支持静态 `__shared__` 记账 |
| 诊断（L 域） | cudaEvent（`tooling/perf.rs`）；`L.3` printf 需状态缓冲回退 |
| 冻结影响 | 本契约不改现有 PTX/ABI；改动若触及源形状或 recipe → 升 L2 重冻 |

### 4.2 后端二：CPU 或另一 GPU（取决于决策）

| 契约面 | CPU 后端（若先做） | 第二 GPU（若先做） |
| --- | --- | --- |
| A/G/K | 单线程或线程池模拟队列/块；`K.1` 用函数序列重放；`K.2/K.3` 软件条件执行 | 依赖目标 API（HIP 同构→抽象几乎不动，检验力弱；WebGPU/Metal 异构→立刻暴露退路） |
| B | Vec/HostBuffer；`B.11` 零拷贝天然支持 | 设备内存/统一内存差异决定 `B.11` 退路 |
| F | 无 NVRTC：**AOT**（build 期生成 Rust + 运行期注册表），F.1 已允许 AOT（见 §5）；defines 受编译期枚举限制 | 各自编译器/SPIR-V；数值策略需逐项对齐 |
| 数值 | 同上；标量数学需与冻结公式逐项核对（E 域） | 逐位一致困难，只能先做容差/统计判据 |
| 窗口 | **0 GPU 窗**，可先验证抽象完整性 | 需 GPU 窗口与冻结集重冻，成本高 |

**边界**：换后端只重做设备层 44 项；语言层 23 项 + 库层 17 项复用。**前提**是语言层已改为书写面（§5）且库层无设备专有代码——当前库层/内核仍大量 CUDA C++，这是 A0 之后的主要工程债。

## 5. 内核书写面（语言层 23 项的 intent shim 草图）

目标：同一份内核源，经 shim 翻译到不同设备。做法是「意图层 + 每后端一个 emitter」，不做通用 IR（第一版）。

> **范围（2026-11-12 定稿）**：本节内容 **P1-a 不含**，属 **P2 交付**——`device_kernel!`、`DeviceType`、表达式层、`Math` 库（`E.3–E.7`）、复合类型原子（`H.2`）与索引模式（`G.8`）均以 P2 为准；P1-a 只提供设备层宿主面与内联原语（`DeviceMath` 标量 19 个、`Atomics` 标量 7 个、`ThreadScope`）。

```rust
// 书写面（示意）：设备无关的内核意图
device_kernel! {
    fn sphere_contact(/* 参数用 shim 类型 */) {
        let i = tid();                       // I.1
        let x = load(qpos, i);               // D.8
        let n = normalize(x);                // E.3
        let d = dot(n, axis);                // E.3
        atomic_add(&mut counts[i], 1);       // H.1/H.7 语义
        barrier();                           // I.3
    }
}
// emitter:
//   CUDA  -> 生成 CUDA C++（沿用 include/warp-headers，NVRTC 编译）
//   CPU   -> 生成 Rust + SIMD（或直接解释执行标量循环）
//   其他  -> 生成目标源 / SPIR-V
```

| 语言层能力 | shim 概念 | 说明 |
| --- | --- | --- |
| C.2/C.3/C.4/C.5/C.7 | `DeviceType`（vec/mat/quat/spatial/array/struct/cast） | 类型系统与布局由 shim 定义，emitter 负责映射到目标类型 |
| D.3/D.4/D.6/D.7/D.8/D.9 | 表达式层（多值返回、泛型、矩阵乘、分量、下标、行视图） | 泛型在 CUDA 走模板、CPU 走单态化 |
| E.3–E.7 | `Math` 库（向量/矩阵/四元数/空间/where） | 与 E.1/E.2 同一 NumericPolicy |
| F.2/F.3 | 编译期（常量折叠、参数化内核） | CUDA 走模板/宏，CPU 走 const generics |
| G.4/G.8 | 发射参数类型、索引模式（一线程一索引 / grid-stride） | 两种模式必须都支持，否则排序/归约类内核落不了 |
| H.2/H.7/H.8 | 原子语义（复合类型逐分量、`+=` 自动原子、`*=` 普通读改写） | 语义由 shim 定，emitter 用目标原子实现 |

## 6. 硬规则

1. **产品代码除 backend 层外禁止直接调 CUDA**：`cudarc`/`Cuda*` 只允许出现在 `src/runtime/backend*` 与明确的 launcher 边界内；新代码不得再直接写 `cudaXxx` 调用（可用脚本/dep 规则检查）。
2. **新内核必须走书写面**：不得新增直接手写 `.cu` 文本（与待决策 ① 联动；容量/缺陷修正另行判定）。
3. **冻结与精确数值不受抽象影响**：契约携带 `NumericPolicy`；同一输入/同一 policy 下，任何后端都必须满足同一数值判据。**若 shim 改变内核源形状 → 冻结 PTX/10 exe 身份变化 → 自动升 L2 重冻**，不得按 L0/L1 处理。
4. **能力缺失必须显式失败或按声明退路**：禁止隐式回退、禁止"能跑就行"的静默降级。
5. **错误三分类**（编译/发射/设备丢失）不得合并、不得吞掉。
6. **不宣称跨设备等价**：位级/容差判据只在「同后端 + 同编译形态」内成立；跨设备只做功能/统计对照，措辞遵守禁用表。

## 7. 已裁与暂定决策（状态列；2026-11-12 更新）

| # | 决策 | 状态 | 影响 |
| --- | --- | --- | --- |
| D1 | 是否暂停按旧写法新增手写 CUDA 内核（容量/缺陷修正豁免？） | **已裁（工作口径，用户可改）**：新内核一律走书写面；容量/缺陷修正豁免 | 决定 §6.2 执行强度与并行节奏 |
| D2 | 第二后端先选 **CPU** 还是**第二 GPU** | **已裁（工作口径，仍暂定）**：先 CPU 验证抽象完整性；**用户签待** | CPU：0 GPU 窗；P4 只出可行性稿，真多线程块（`I.3`）归 P4' 另评 |
| D3 | `F.1` 是否允许 AOT | **已裁（2026-11-12）**：允许——「源 → 可执行物（JIT 或 AOT 均可）」，见 §2 设计约束 5 | 第二后端不必假装运行时编译 |
| D4 | `F.5` defines 是否必须运行期任意 | **已裁（2026-11-12）**：不必须——集合由后端声明、超集 → `Unsupported` | 见 §2 设计约束 6 |

## 8. 与现有证据的关系

- 本文件是**契约草案**，不是验收证据；不改变任何已冻结证据、判据、容差、参考。
- 落地顺序建议：先 B0 定义类 5 片（语言/能力语义）→ 书写面最小试点（PU-16 三个代表内核）→ 第二后端骨架（0 窗）→ 再谈冻结集。
- 与流程：A0 属 L0（文档）；一旦进入书写面实现或冻结集重冻，按 L1/L2 升级。

## 9. 消费方接口（物理层 / 交互层）

**依赖方向**：消费方 → 内核执行层接口（traits）。执行层**不得**反向依赖 `physics/*`、`render/*`、`io/*`；跨层只传 opaque 句柄/描述符（`BufferHandle`、`KernelHandle`、`LaunchDesc`），需要回调（printf、host fallback）时由消费方注入 trait object，而不是执行层 `use` 消费方类型。

### 9.1 物理层

| 消费方 | 现有模块 | 最少 API 集（trait::method） | 数据搬运 | 库层算法 | 备注 |
| --- | --- | --- | --- | --- | --- |
| collision（宽/窄相、接触） | `physics/collision/**`、`runtime/collision_gpu/**` | `Allocator::alloc`、`Buffer::{upload,readback}`、`Module::compile`、`Queue::submit`、`Launch::{dims,blocks,block_width,alias,args}`、`Atomics::{add,cas,min,max}`、`DeviceError` | 模型/数据上传 + `nacon/nacc/efc/contact` 回读 | 无（mesh 射线不属碰撞路径） | 已有 host fallback（K.4 语义），契约需保留 `Unsupported` 路径 |
| dynamics（RNEA/积分/平滑力/质量求解） | `physics/rne*`、`physics/integrator*`、`physics/smooth_force*`、`physics/mass_*` | `Buffer`（状态持久借出）、`Queue::{capture,replay}`（整 step 图）、`Atomics::add`（力累加）、`DeviceMath`（quat/spatial）、`Module::compile` | 状态 in/out 借用；每步少量控制回读 | `I.6` tile matmul / cholesky、`J.1` 前缀和、`J.2/J.3` 排序（当前缺失） | 图捕获是 step 级性能关键；`K.1` 必须可用 |
| flex（柔体 hessian/边/面） | `physics/flex*`、`physics/resident*` | `Launch`、`ThreadScope::{tid,shared,barrier}`、`DeviceMath`、`Atomics` | 节点/边/面缓冲借出 + 校验回读 | `I.6` tile matmul（可选） | `I.2`/`I.3` 目前产品内核只用 `__syncthreads` |
| mass matrix / multiply | `physics/mass_matrix/**` | `Buffer` 借出、`Launch`、`Queue::submit` | 矩阵/向量借出，能量/校验小回读 | `I.6` tile matmul / cholesky、`J.1` 前缀和 | 与 dynamics 共用 tile 路径 |

### 9.2 交互层

| 消费方 | 现有模块 | 最少 API 集 | 数据搬运 | 库层算法 | 备注 |
| --- | --- | --- | --- | --- | --- |
| io（模型/状态存取） | `io/**`、`model/**` | `Allocator::{alloc,borrow_external}`、`Buffer::{upload,readback}`、`DType` | 上传为主；状态回读 | 无 | 零拷贝借用（`B.11`）对模型常驻数组价值最大 |
| render（megakernel/depth/seg/AA） | `render/kernel/**`、`render/context/**` | `Module::compile`（源拼接 + recipe）、`Launch`（2 维、块宽、alias 累加缓冲）、`Queue::{capture,replay}`（多帧）、`DeviceMath`、`ThreadScope::tid` | 逐帧相机/光照小上传；图像/深度回读 | `M.2` BVH 遍历、`M.4` 纹理采样、`M.1` 构建/重拟合 | 渲染 recipe 与物理 recipe 分离（fast-math 关闭一致） |
| BVH / query（构建/重拟合/射线） | `render/bvh/**`、`physics/query*` | `Queue::submit`、`Launch`（1 维）、`Buffer` | 图元/节点上传；命中结果回读 | `M.1` 构建/重拟合、`M.2` 遍历、`M.3` 网格射线 | `M.1` 当前为"已实现"但需纳入库层 17 项验收 |

### 9.3 消费方最小公共集

任何消费方都至少需要：`Backend::devices`、`Device::caps`、`Allocator::alloc`、`Buffer::{upload,readback}`、`Module::compile`、`Queue::submit`、`Launch`、`Queue::wait_idle`、`DeviceError`。
增量：碰撞 `Atomics`+alias；动力学 `capture/replay`+`DeviceMath`+库层 tile/scan/sort；渲染 `M.*`；io 仅 `borrow_external`。

### 9.4 最小化原则

- 消费方只拿**描述符**，不拿具体后端类型（`dyn Device` 或泛型参数），保证换后端不改消费方。
- 同步点只允许出现在消费方显式请求处（`wait_idle`）；执行层不得在 `upload/readback/alloc` 内隐式同步。
- 库层算法（BVH/扫描/排序/tile）以"函数 + 契约"形式暴露，不进消费方。

## 10. 85 项「完成」定义

**总口径**：85/85 全部满足四要素 = **契约接口** + **一个后端实现（CUDA 先行）** + **测试或证据** + **映射到 PU/能力编号**；「全部 85 项实现完成」是本阶段验收目标。当前实测见 §10.3。

### 10.1 四要素细则

| # | 要素 | 判据 |
| --- | --- | --- |
| ① | 契约接口 | 该能力在 §2/§3 有 trait 方法或书写面条目；无接口的能力（如 D.1 控制流）须有语言能力声明位 |
| ② | 后端实现 | CUDA 后端有真实实现；「退」项必须实现退路并在 `DeviceCaps.fallbacks` 声明；缺能力返回 `Unsupported` 或走声明退路 |
| ③ | 测试或证据 | 分级：设备层/语言层 `L1` = 单测 + mock backend + 定向测试；涉及判据数字/冻结/窗 = `L2`（窗证据）。每个「退」项至少 1 条退路测试 |
| ④ | 映射 | `capability_id` → PU 编号（PU-01…PU-31 或 backlog 条目）；无 PU 的须显式记 backlog，不得悬空 |

### 10.2 分层 done 判据

| 层 | 项数 | done 判据 | 证据级别 |
| --- | --- | --- | --- |
| 设备层 | 44 | 每个 trait 方法有单测（mock backend）+ CUDA 适配 smoke；10 个「退」项各 1 条退路测试 | L1（不涉判据数字）；退路若影响数值 → L2 |
| 语言层 | 23 | 书写面条目 + shim 单测 + 3 个代表内核试点（PU-16）；改冻结源形状 = L2 | L1 → 试点 L2 |
| 库层 | 17 | 算法单测 + 与 CPU/解析参考对照（容差按数值契约）；BVH/纹理类含结构断言 | L1；判据数字 L2 |

### 10.3 当前实测口径（2026-11-12）

来源：`target/impl-plan-review/capability-mapping.json`（只读评审产物，工作材料）。**85 = 43 已实现 / 25 部分 / 14 缺失 / 2 不适用 / 1 待并入**；按层：设备 23/12/9、语言 12/8/1（+2 不适用）、库 8/5/4。

- 「完成」判定 = 把 **25 部分 + 14 缺失** 逐项转成「已实现（四要素齐）」；**2 不适用**（`H.7`/`H.8` 隐式原子语义）保留但须注明"仅在当前手写 CUDA 架构下成立"，采用书写面后重评。
- 9 个必级缺口（`H.2/H.3/H.5/H.6/J.1–J.3/B.8`）是首批硬骨头，归属见 `pu-registration-draft.md` §2。
- **A1 清单**：以现状冻结集身份为基准（10 个 exe + 13 fixture + 2 参考，按仓内已 verified 冻结证据；A2 重冻后更新）。P1/P2 若**不改内核源形状**，A1 身份应保持不变；一旦变化即 L2 重冻。每个后端实现完成的证据必须包含该后端对应的身份记录。
- 对外措辞：只写「契约 + 实现 + 测试齐备」，不写「缺口已清」「与上游等价」。
