# 统一内核发射器设计

日期：2026-10-18。
状态：设计草案第五版，可作实施基线；尚未实现。
范围：内核执行层及其与计算物理层的接口。
本文不改产品物理路径。
本文不新增数值证据。

## 结论

现有内核发射分散在五处。
统一发射函数本身不难。
主要成本在资源管理。
本文先定安全契约，再定接口。
首阶段保留公开同步语义。
异步与图捕获放到后段。
每段都有独立验收规则。

## 修订记录

第一版于同日提交评审。
评审指出六项缺口与两处前提。
本版逐项吸收，处置如下。

| 评审项 | 处置 | 位置 |
| --- | --- | --- |
| 图更新接口过宽 | 改为节点加参数槽 | [图更新契约](#c4-图更新契约) |
| 异步缺少生命周期 | 新增完成凭证契约 | [资源生命周期契约](#c2-资源生命周期契约) |
| 捕获缺少上下文隔离 | 新增捕获隔离契约 | [捕获隔离契约](#c3-捕获隔离契约) |
| 缓存混合两类身份 | 拆成两层缓存 | [两层缓存](#两层缓存) |
| 漏掉动能入口 | 补为十一个方法 | [逐项差异](#逐项差异) |
| 验收规则未闭合 | 按路径分类判定 | [验收规则](#验收规则) |
| 配方差异被写成冲突 | 改为禁止误用 | [编译配方](#编译配方) |
| 调用点数被当成频率 | 改为待测量项 | [已确认问题](#已确认问题) |

第二版经二轮复核。
复核指出三项安全缺口与两处偏差。
本版处置如下。

| 复核项 | 处置 | 位置 |
| --- | --- | --- |
| 资源持有不等于访问安全 | 新增访问依赖账本 | [访问依赖账本](#访问依赖账本) |
| 槽位更新缺少别名检查 | 新增别名与区间校验 | [别名与区间](#别名与区间) |
| 捕获缺少专用入队语义 | 新增捕获作用域入队 | [捕获入队](#捕获入队) |
| 安全分层与接口矛盾 | 新增安全内核包装 | [安全内核包装](#安全内核包装) |
| 现状把目标写成能力 | 方法表拆出现有检查 | [逐项差异](#逐项差异) |

第三版经三轮复核。
复核认可总体分层。
复核另指出两处问题。

| 复核项 | 处置 | 位置 |
| --- | --- | --- |
| 重绑可能绕过元数据证明 | 首版禁止地址类槽位重绑 | [地址类槽位](#地址类槽位) |
| 变更范围自相矛盾 | 改为可调宿主校验层，并列文件范围 | [变更范围](#变更范围) |

第四版经四轮复核。
复核认可总体设计。
复核指出一处证据表述错误。

| 复核项 | 处置 | 位置 |
| --- | --- | --- |
| PTX摘要不能证明源码不变 | 分列源码与PTX两类摘要 | [变更范围](#变更范围)、[验收规则](#验收规则) |

## 现状盘点

以下事实来自当前源码。
核查对象为`D:/Rust/mjwarp-rs`。

### 五条路径

| 编号 | 路径 | 文件 | 内核来源 | 用途 |
| --- | --- | --- | --- | --- |
| L1 | 同步固定ABI | `src/runtime/transfer/kernel.rs`及子模块 | Rust字符串CUDA，经NVRTC | 产品物理子集 |
| L2 | 冻结Warp产物 | `src/runtime/warp_kinematics.rs`等三文件 | 上游生成PTX | 上游对照与诊断 |
| L3 | 驱动探针 | `src/runtime/cuda.rs`、`cuda/artifacts.rs`、`cache.rs` | 内置PTX或CubeCL | P1路线探针 |
| L4 | CubeCL | `src/runtime/cubecl.rs` | Rust宏内核 | P1路线探针 |
| L5 | MathDx链接 | `scripts/native-mathdx7-*.{cpp,cu}` | LTO中间产物 | 仓外工具验证 |

L2包括以下三文件：

```text
src/runtime/warp_kinematics.rs
src/runtime/warp_com.rs
src/runtime/warp_gravity.rs
```

### 逐项差异

| 项目 | L1 | L2 | L3 |
| --- | --- | --- | --- |
| 编译 | NVRTC现场编译 | 加载冻结PTX | NVRTC或内置PTX |
| fmad | 开启 | 随上游产物 | 缓存配方为关闭 |
| fast-math | 关闭 | 随上游产物 | 关闭 |
| 目标 | `compute_70` | 产物自带 | `sm70`起 |
| 参数ABI | 十一种私有签名 | Warp边界加数组描述 | 三参数探针签名 |
| 块大小 | 256，另有128、64、32 | 256 | 128 |
| 同步 | 每次发射后等待 | 调用方自理 | 调用方自理 |
| 图捕获 | 不支持 | 不支持 | 支持，可改前两参数 |
| 缓存 | 无 | 摘要核验 | 磁盘缓存，绑定探针布局 |

L1的十一个发射方法如下：

| 方法 | 位置 | 线程规则 | 现有检查 |
| --- | --- | --- | --- |
| `launch` | `kernel.rs` | 256线程 | 基础检查 |
| `launch_mixed_solve` | `kernel.rs` | 128线程，每块一世界一右端 | 基础检查加容量检查 |
| `launch_small_factor` | `kernel.rs` | 64线程，每线程一世界块 | 基础检查加容量检查 |
| `launch_sparse_solve` | `kernel.rs` | 128线程，块内屏障 | 基础检查加容量检查 |
| `launch_rne_cacc` | `kernel.rs` | 256线程 | 基础检查 |
| `launch_tendon_passive` | `kernel.rs` | 256线程 | 基础检查 |
| `launch_gravity_compensation` | `kernel.rs` | 256线程 | 基础检查 |
| `launch_fluid_force` | `kernel.rs` | 256线程 | 基础检查 |
| `launch_flex_hessian` | `kernel.rs` | 256线程 | 基础检查 |
| `launch_flex_edges` | `kernel.rs` | 256线程 | 基础检查 |
| `launch_kinetic_energy32` | `kernel/kinetic_energy.rs` | 每世界一个warp，32线程 | 基础检查加容量检查，`n≤1024` |

基础检查只含以下三项：

- 全部缓冲属于同一会话。
- 工作量非零且不溢出。
- 全部缓冲非空。

容量检查另核缓冲长度与维度。
七个方法没有容量检查。
它们的访问跨度由调用方证明。
证明责任写在各方法的`# Safety`中。
输入只读与输出独占也由调用方证明。
例见`launch_gravity_compensation`的说明。

迁移须逐项保留线程规则。
线程数属于ABI的一部分。
屏障与ballot要求整块参与。
容量检查在安全包装中补齐。
补齐属于新增能力，不属现状。

### 已确认问题

1. L1每次发射都同步等待。
   它阻断图捕获与流水执行。
   但它也提供完成保证。
   见[资源生命周期契约](#c2-资源生命周期契约)。
2. L1排除测试后有98处编译调用。
   调用点数不等于编译频率。
   冷启动与稳态开销须实测。
   实测前不据此排定优先级。
3. L1按调用形状新增方法。
   现有十一个私有发射方法。
4. L2各文件自行定义Warp结构。
   `WarpArray`重复两份。
   `warp_com.rs`不定义数组结构。
5. 探针与物理使用不同配方。
   两者用途不同，差异合理。
   风险在于误用，不在于不同。
6. L3图能力只覆盖探针ABI。
   它读取三个参数槽。
   它只替换前两个指针。
   它不处理嵌套数组描述。
   因此不能直接泛化。
7. L3缓存校验探针入口与128线程。
   它不能原样承载产品内核。
8. L5依赖Warp动态库编译。
   它不能进入产品路径。

## cudarc能力边界

版本锁定为`=0.18.2`。
当前feature如下：

```text
std, driver, nvrtc, dynamic-loading, cuda-12000
```

以下结论来自0.18.2源码。
核查位置为Rara的cargo缓存。

### 直接可用

| 能力 | cudarc接口 |
| --- | --- |
| 上下文与线程绑定 | `CudaContext`、`bind_to_thread` |
| 流与依赖 | `new_stream`、`fork`、`join`、`record_event`、`wait` |
| 显存分配 | `alloc`、`alloc_zeros`；内部用`malloc_async` |
| 模块加载 | `load_module`、`load_function` |
| 内核发射 | `launch_builder`、`launch`、`launch_cooperative` |
| 函数属性 | `set_attribute`、占用率查询 |
| 基础图捕获 | `begin_capture`、`end_capture`、`CudaGraph::launch` |
| 事件跟踪开关 | `enable_event_tracking`、`disable_event_tracking` |
| 运行时编译 | `compile_ptx_with_opts`、`CompileOptions` |
| 原始驱动API | `driver::sys`、`driver::result` |

### 只能经sys使用

| 能力 | 条件 |
| --- | --- |
| 图节点参数更新 | `cuGraphExecKernelNodeSetParams_v2` |
| 图节点枚举 | `cuGraphGetNodes`、`cuGraphNodeGetType` |
| 条件图节点 | `cuGraphConditionalHandleCreate`需`cuda-12030`起 |
| 通用加节点 | `cuGraphAddNode`需`cuda-12020`起 |

当前`cuda-12000`看不到条件节点。
上游求解器使用`wp.capture_while`。
位置为冻结`solver.py:3994`。
`derivative.py`与`collision_flex.py`也使用。
因此G14与G15需要条件节点。
升级feature须单独评审。
驱动最低版本仍待核实。

### 完全缺失

| 能力 | 影响 |
| --- | --- |
| nvJitLink绑定 | MathDx的LTO无法链接 |
| Warp参数类型 | 需要自建`repr(C)`结构 |
| 模块缓存 | 需要自建两层缓存 |
| Warp头文件 | 编译上游源码需另备 |
| 预分配显存池 | 需要自建`Data`分配器 |

### 已知行为约束

`alloc`走流序异步分配。
捕获期间分配会生成内存节点。
所以捕获前必须完成分配。

事件跟踪开关属于上下文。
它影响该上下文的全部流。
不能只对单个流生效。

`CudaGraph`不具备线程安全。
同一图只能串行访问。

图节点更新只影响后续发射。
在途任务仍用旧参数。
所以旧缓冲不能立即释放。

## 目标分层

本仓库分为三层。
口径见[架构三层划分](architecture.md#三层划分)。
本文只细化内核执行层。

三层调用方向如下：

```text
数据交互层 → 计算物理层
     │           │
     └─────┬─────┘
           ↓
      内核执行层
```

数据交互层只用设备、分配、上传、回读与视图。
计算物理层只用安全内核包装与捕获作用域。
内核执行层内部结构如下：

```text
发射器  launcher
          ├── recipe     编译配方与用途
          ├── source     内核来源
          ├── artifact   编译产物缓存
          ├── module     上下文内模块缓存
          ├── abi        参数打包与槽位描述
          ├── launch     发射与完成凭证
          └── graph      捕获、槽位更新、条件节点
          │
          ▼
底座    cudarc driver / nvrtc / sys
        自建 nvJitLink 动态绑定
```

计算物理层不得直接`use cudarc`。
数据交互层不得暴露cudarc类型。
数据交互层不发射内核。
unsafe只出现在发射器与底座。
计算物理层只调用安全内核包装。
见[安全内核包装](#安全内核包装)。

## 安全契约

接口实现前先冻结以下契约。
契约未冻结时不进入对应阶段。

### C1 上下文与流

一个设备对应一个`GpuDevice`。
一个`GpuDevice`持有一个上下文。
发射器记录上下文身份。
跨上下文使用缓冲直接拒绝。
每次驱动调用前绑定线程。

### C2 资源生命周期契约

本契约约束异步发射。
首阶段仍保留同步语义。

**完成凭证**

- 每次提交返回一个`Completion`。
- 凭证内含驱动事件。
- 凭证持有本次参与的全部资源。
- 资源包括缓冲、模块与图实例。
- 凭证存活期间资源不得释放。

**释放与复用**

- 缓冲析构前须确认完成。
- 确认方式为事件已到达。
- 模块卸载前须等待全部引用任务。
- 图实例销毁前须等待全部重放。

持有资源只保证资源存活。
它不保证访问顺序。
访问顺序由下述账本保证。

#### 访问依赖账本

每个设备维护一本访问账本。
账本以缓冲区间为单位记录。
每条记录含访问类型与完成事件。

| 访问 | 等待条件 |
| --- | --- |
| 读取 | 等待该区间前序全部写入 |
| 写入 | 等待该区间前序全部读写 |

以下提交都须登记：

- 普通发射。
- 图重放。
- 宿主上传与回读。
- 设备内复制。

图重放按捕获时的读写集登记。
读写集来自各节点ABI槽位。
槽位重绑后读写集同步更新。

跨流依赖用事件等待实现。
同流依赖由流序保证，仍须登记。
登记失败则拒绝提交。

例：流甲重放图并写缓冲B。
流乙随后提交读取B。
账本令流乙先等待流甲事件。
双方仅持有资源时，无法保证此序。

**延迟错误**

- 异步错误可能晚于提交出现。
- 错误归属到所在执行流。
- 该流进入隔离状态。
- 隔离流拒绝后续全部提交。
- 同上下文其他流须重新检查状态。
- 粘滞错误使整个上下文隔离。
- 首个错误保留，后续错误追加。

**析构失败**

- 无法证明完成时保留资源。
- 保留方式沿用现有做法。
- 现有做法见`SessionStatus::execute`。
- 它在完成不明时`forget`结果。
- 泄漏优先于释放在途显存。
- 泄漏须进入诊断记录。

### C3 捕获隔离契约

**互斥**

- 捕获以上下文为互斥单位。
- 同一上下文同时只允许一个捕获。
- 捕获期间拒绝其他流提交。
- 嵌套捕获直接拒绝。

**前提**

- 捕获前全部预热完成。
- 捕获前收敛全部流依赖。
- 捕获前完成全部分配与编译。
- 捕获期间拒绝分配与编译。

#### 捕获入队

- 捕获期间只能经捕获作用域入队。
- 作用域入队只记录节点。
- 它返回节点标识，不返回完成凭证。
- 捕获阶段内核尚未执行。
- 所以捕获阶段不能确认完成。
- 只有图重放才返回完成凭证。
- 捕获期间拒绝`launch_sync`。
- 捕获期间拒绝`submit`。
- 捕获流上的无关提交一律拒绝。
- 捕获期间不查询事件状态。
- 捕获期间不做流同步。
- 驱动限制捕获内的事件查询。
- 作用域同时收集节点读写集。
- 读写集供账本登记重放。

**事件跟踪**

- 捕获前读取原开关状态。
- 捕获期间关闭事件跟踪。
- 结束后恢复原状态，而非强制开启。
- 恢复动作放在守卫析构中。

**失败清理**

- 捕获出错须结束捕获。
- 恐慌时守卫同样结束捕获。
- 残缺图直接销毁。
- 上下文状态恢复后才解除互斥。
- 清理失败则隔离上下文。

### C4 图更新契约

**更新单位**

- 更新以节点加参数槽为单位。
- 不按裸地址全局查找替换。
- 槽位来自ABI描述，而非运行时猜测。
- 嵌套数组描述按字段定位。

**类型与容量**

- 新缓冲须为类型化缓冲。
- 校验上下文身份。
- 校验元素类型。
- 校验容量不小于原缓冲。
- 校验读写权限一致。

#### 别名与区间

逐槽检查不足以保证安全。
例如把输出重绑到输入缓冲。
它可能通过上述四项检查。
但内核要求输入输出互斥。

- 每个ABI声明槽位间别名约束。
- 默认规则为可写槽独占。
- 可写槽不得与任何槽重叠。
- 只读槽之间允许重叠。
- 例外须在ABI中显式声明。
- 缓冲视图按字节区间比较。
- 同一分配的不同视图也须比较。
- 整批更新后做一次联合校验。
- 联合校验覆盖全部槽位，含未更新槽。
- 联合校验失败则整批回滚。

同一规则也用于普通发射。
安全包装在发射前执行同样校验。

#### 地址类槽位

长度与类型无法证明索引安全。
例如混合求解依赖元数据证明。
见`kernel.rs`中`launch_mixed_solve`的安全说明。
同型同容量的新缓冲可含越界索引。
重放会绕过原校验证明。

槽位按内容用途分为两类：

| 类别 | 判定 | 例子 |
| --- | --- | --- |
| 数值槽 | 内容只参与算术 | 状态、参数、输出 |
| 地址类槽 | 内容参与寻址、分支或循环界 | 元数据、CSR行列、路由表 |

分类按内容用途，不按元素类型。
i32数值可能属于数值槽。
f32内容若用作界，也属地址类。
分类写入ABI的槽位描述。
不能确定时一律视为地址类。

**首版规则**

- 首版禁止重绑地址类槽位。
- 首版只允许重绑数值槽位。
- 地址类内容变化须重新捕获。

**后续规则**

后续只接受受检元数据重绑。
受检元数据须满足以下条件：

- 证明绑定内容摘要。
- 证明绑定适用维度。
- 证明绑定上下文与ABI。
- 设备内容对外只读。
- 任何写入都使证明失效。
- 失效后不能再用于发射或重绑。
- 捕获时图实例保留证明。
- 重绑时新证明替换旧证明。
- 旧证明随旧缓冲进入待回收列表。

重绑前核对新证明的维度。
维度须与图内节点一致。
不一致则整批回滚。

**所有权**

- 图实例持有当前全部缓冲。
- 更新后图持有新缓冲。
- 旧缓冲移入待回收列表。
- 待回收列表等待在途重放完成。
- 完成后才真正释放。

**连续与部分失败**

- 连续更新按提交顺序生效。
- 一次更新可含多个槽位。
- 任一槽位失败则整批回滚。
- 回滚失败则作废图实例。
- 作废后须重新捕获。

## 需要提供的接口

以下签名为草案。
名称可在实现时调整。
语义须满足上述契约。

### 设备与流

```rust
pub(crate) struct GpuDevice {
    context: Arc<CudaContext>,
    context_id: u64,
    arch: (u32, u32),
    driver_api: i32,
    capture_lock: Mutex<()>,
}

pub(crate) struct ExecStream {
    device: Arc<GpuDevice>,
    stream: Arc<CudaStream>,
    status: SessionStatus,
}

impl ExecStream {
    pub(crate) fn synchronize(&self) -> Result<(), TransferError>;
    pub(crate) fn is_capturing(&self) -> Result<bool, TransferError>;
}
```

`ExecStream`承接`TransferSession`的发射职责。
隔离状态语义原样保留。

### 编译配方

```rust
pub(crate) enum RecipePurpose {
    Physics,
    Probe,
}

pub(crate) struct CompileRecipe {
    purpose: RecipePurpose,
    fmad: bool,
    fast_math: bool,
    std: CppStd,
    arch: ArchTarget,
    defines: Vec<(String, String)>,
}

impl CompileRecipe {
    pub(crate) const WARP_1_15_PHYSICS: Self;
    pub(crate) const P1_PROBE: Self;
    pub(crate) fn fingerprint(&self) -> String;
}
```

探针与物理可以使用不同配方。
配方带用途标记。
物理内核只接受`Physics`配方。
用途不符时直接拒绝。
物理配方只有一个常量来源。
当前取值为`fmad=true`、`fast_math=false`。

### 内核来源

```rust
pub(crate) enum KernelSource {
    Nvrtc {
        name: &'static str,
        source: Cow<'static, str>,
        headers: Vec<HeaderFile>,
    },
    Ptx { name: &'static str, ptx: Cow<'static, str>, sha256: &'static str },
    Lto { name: &'static str, parts: Vec<LtoPart> },
}
```

`Nvrtc`承接L1手写源码。
`Ptx`承接L2冻结产物。
`Lto`预留给MathDx链接。
已加载的PTX一律核验摘要。

### 两层缓存

| 层 | 键 | 值 | 作用域 |
| --- | --- | --- | --- |
| 编译产物缓存 | 源码摘要、头文件内容摘要、配方指纹、目标架构、NVRTC版本 | PTX或cubin字节 | 进程或磁盘 |
| 模块缓存 | 产物摘要、上下文身份、入口名 | `CudaModule`与`CudaFunction` | 单个上下文 |

```rust
pub(crate) struct ArtifactCache { /* 私有 */ }
pub(crate) struct ModuleCache { /* 每上下文一份 */ }

impl ArtifactCache {
    pub(crate) fn compile(
        &self,
        source: &KernelSource,
        recipe: &CompileRecipe,
        target: ArchTarget,
    ) -> Result<Arc<Artifact>, TransferError>;
}

impl ModuleCache {
    pub(crate) fn function<A: KernelAbi>(
        &self,
        artifact: &Arc<Artifact>,
        entry: &'static str,
    ) -> Result<KernelHandle<A>, TransferError>;
}
```

产物缓存不绑定上下文。
同架构设备可共享产物。
模块缓存绑定真实上下文。
头文件内容参与产物摘要。
捕获期间两层都禁止未命中。

L3磁盘缓存格式不能直接复用。
它校验探针入口与128线程。
产品缓存须另定载荷格式。
可沿用其原子写入与损坏处理。

### 参数ABI

```rust
pub(crate) unsafe trait KernelAbi {
    type Args<'a>;
    const BLOCK: BlockRule;
    fn slots() -> &'static [SlotDesc];
    fn push<'a>(builder: &mut LaunchArgs<'a>, args: &'a Self::Args<'a>);
}

pub(crate) struct SlotDesc {
    index: usize,
    kind: SlotKind,
    access: Access,
    role: SlotRole,
}

pub(crate) enum SlotRole {
    /// 内容只参与算术。
    Value,
    /// 内容参与寻址、分支或循环界。
    AddressBearing { proof: ProofKind },
}

pub(crate) enum SlotKind {
    DevicePtr { elem: ElemType },
    WarpArrayData { elem: ElemType },
    Scalar { bytes: u8 },
    Bounds,
}

#[repr(C)]
pub(crate) struct WarpArray<T> {
    data: u64,
    grad: u64,
    shape: [i32; 4],
    strides: [i32; 4],
    ndim: u16,
    flags: u16,
    padding: u32,
    marker: PhantomData<T>,
}

#[repr(C)]
pub(crate) struct LaunchBounds<const N: usize> {
    shape: [i32; N],
    size: u64,
    coord_mult: u64,
}
```

ABI分为两族。

| ABI族 | 形状 | 承接 |
| --- | --- | --- |
| 私有指针ABI | 指针加`u32`维度 | L1现有十一种签名 |
| Warp ABI | 边界加数组描述 | L2及后续上游移植 |

`slots()`供图更新定位参数槽。
`BLOCK`固定线程数规则。
动能入口须声明32线程规则。
`WarpArray`只保留一份定义。
布局以Warp 1.15头文件为准。
新增维度须先补布局测试。

发射边界与数组维度是两类参数：

| 参数 | 含义 | 支持维度 |
| --- | --- | --- |
| `Bounds { dims }` | `launch_bounds_t<N>`发射网格 | 1至3 |
| `WarpArrayData { ndim }` | `array_t`数组形状 | 1至4 |

冻结上游的`wp.launch`未用4维网格。
因此不提供4维发射边界。
1维静态证据见[边界补证](m1-bounds-evidence.md)。
静态证据不证明字段顺序与运行语义。

### 发射

```rust
pub(crate) struct LaunchDims {
    grid: (u32, u32, u32),
    block: (u32, u32, u32),
    shared_bytes: u32,
}

impl<A: KernelAbi> KernelHandle<A> {
    /// 首阶段：入队后等待完成，语义同现状。
    /// # Safety
    /// 调用方证明ABI与全部访问范围。
    pub(in crate::runtime::launcher) unsafe fn launch_sync(
        &self,
        stream: &ExecStream,
        dims: LaunchDims,
        args: &A::Args<'_>,
    ) -> Result<(), TransferError>;

    /// 后续阶段：满足C2后才开放。
    /// # Safety
    /// 同上，另须由凭证持有全部资源。
    pub(in crate::runtime::launcher) unsafe fn submit(
        &self,
        stream: &ExecStream,
        dims: LaunchDims,
        args: &A::Args<'_>,
    ) -> Result<Completion, TransferError>;
}
```

两个入口仅在发射器内部可见。
可见范围为`pub(in crate::runtime::launcher)`。
计算物理层不直接调用它们。
`launch_sync`先承接L1的十一个方法。
`submit`在C2冻结后才实现。
两个入口在捕获期间都拒绝执行。

### 安全内核包装

计算物理层禁止不安全代码。
所以计算物理层不能证明安全前提。
安全前提改由包装校验。

```rust
pub(crate) trait KernelSpec: KernelAbi {
    type Dims;
    /// 由维度推出每个槽位的最小跨度。
    fn spans(dims: &Self::Dims) -> Result<SlotSpans, InputError>;
    /// 由维度推出网格与块。
    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, InputError>;
}

pub(crate) struct Kernel<K: KernelSpec> { /* 私有 */ }

impl<K: KernelSpec> Kernel<K> {
    pub(crate) fn run(
        &self,
        stream: &ExecStream,
        dims: &K::Dims,
        args: &K::Args<'_>,
    ) -> Result<(), TransferError>;
}
```

`run`是安全函数。
它依次执行以下校验：

1. 上下文与会话一致。
2. 元素类型与ABI一致。
3. 各缓冲长度覆盖`spans`。
4. 槽位别名满足ABI约束。
5. 维度推出的网格不溢出。
6. 当前不处于捕获期间。

全部通过后才调用不安全入口。
校验失败时不发射任何任务。

部分前提无法由长度推出。
例如元数据内的索引范围。
这类前提由受检元数据类型承担。
受检元数据只能由校验函数构造。
校验函数位于宿主校验层。
宿主校验层含模型、输入与物理宿主代码。
计算物理层只传递受检类型。

```rust
pub(crate) struct Checked<M: MetadataProof> { /* 私有 */ }

impl<M: MetadataProof> Checked<M> {
    /// 只读设备缓冲；无写入接口。
    pub(crate) fn buffer(&self) -> &TypedBuffer<i32>;
    pub(crate) fn digest(&self) -> &Digest;
    pub(crate) fn dims(&self) -> &M::Dims;
}
```

地址类槽位只接受`Checked`。
`run`核对证明维度与本次维度。
二者不一致则拒绝发射。

七个缺少容量检查的方法须补`spans`。
补齐后拒绝面会变宽。
数值路径不受影响。

### 图捕获

```rust
pub(crate) struct ExecGraph { /* 持有全部缓冲与模块 */ }
pub(crate) struct NodeSlot { node: NodeId, slot: usize }

impl GpuDevice {
    pub(crate) fn capture<F>(&self, stream: &ExecStream, body: F)
        -> Result<ExecGraph, TransferError>
    where
        F: FnOnce(&CaptureScope<'_>) -> Result<(), TransferError>;
}

impl CaptureScope<'_> {
    /// 只记录节点，不执行，不返回完成凭证。
    pub(crate) fn enqueue<K: KernelSpec>(
        &self,
        kernel: &Kernel<K>,
        dims: &K::Dims,
        args: &K::Args<'_>,
    ) -> Result<NodeId, TransferError>;
}

impl ExecGraph {
    pub(crate) fn launch(&self, stream: &ExecStream) -> Result<Completion, TransferError>;
    /// 首版只接受数值槽位。
    pub(crate) fn rebind_values<T>(
        &mut self,
        updates: &[(NodeSlot, &TypedBuffer<T>)],
    ) -> Result<(), TransferError>;
}
```

`capture`在`GpuDevice`上执行。
这样可施加上下文级互斥。
`rebind_values`遵循C4契约。
目标为地址类槽位时直接拒绝。
首版不提供元数据重绑接口。
裸地址替换接口不再提供。

### 诊断记录

每次发射可选写一条记录。
记录入口、维度与配方指纹。
`check_physics_ptx`继续执行。
`record_physics_ptx`继续执行。
泄漏与隔离事件也写入记录。

## 需要额外编写的部分

| 编号 | 工作 | 依赖 | 原因 |
| --- | --- | --- | --- |
| W01 | 统一`WarpArray`与`LaunchBounds` | 无 | 两份定义需合并 |
| W02 | 配方用途标记与指纹 | 无 | 防止探针配方误用 |
| W03 | 两层缓存 | W02 | 区分产物与上下文 |
| W04 | `KernelAbi`与槽位描述 | W01 | 取代十一个私有方法 |
| W05 | `launch_sync`等价迁移 | W04 | 保留现有完成保证 |
| W06 | 完成凭证与资源持有 | C2冻结 | 异步化前提 |
| W07 | 延迟错误与隔离传播 | W06 | 异步错误归属 |
| W08 | 待回收列表 | W06 | 旧缓冲延迟释放 |
| W09 | 上下文级捕获互斥与守卫 | C3冻结 | 共享上下文安全 |
| W10 | 节点槽位更新 | W04、W08、C4冻结 | 替代裸地址替换 |
| W11 | 预分配`Data`显存 | W06 | 捕获期间禁止分配 |
| W12 | 条件图节点 | W09、feature升级 | G14、G15迭代 |
| W13 | nvJitLink动态绑定 | 无 | cudarc不含此库 |
| W14 | 共享内存与tile发射 | W04 | 上游tile内核 |
| W15 | Warp头文件管理 | 待决 | 仅编译上游源码时需要 |
| W16 | 编译开销实测 | 无 | 确定缓存优先级 |
| W17 | 安全内核包装与`spans` | W04 | 计算物理层无需unsafe |
| W18 | 受检元数据类型 | W17 | 承担索引类前提 |
| W19 | 槽位别名约束与联合校验 | W04 | 防止输入输出重叠 |
| W20 | 访问依赖账本 | W06 | 保证读写顺序 |
| W21 | 捕获作用域入队 | W09、W19 | 捕获期间专用语义 |
| W22 | 槽位用途分类 | W04 | 区分数值与地址类 |
| W23 | 受检元数据证明绑定与失效 | W18、W22 | 后续开放元数据重绑 |

W12与W13须先做独立评审。
W15取决于内核来源决议。
若只加载冻结PTX，则不需要W15。

## 迁移步骤

| 阶段 | 内容 | 前提 |
| --- | --- | --- |
| M0 | 冻结C1至C4；W16实测 | 无 |
| M1 | W01、W02、W04、W19、W22 | M0 |
| M2 | W03 | M1 |
| M3 | W05、W17、W18；十一个方法改用安全包装 | M2 |
| M4 | L2三文件改用发射器 | M3 |
| M5 | W06、W07、W08、W20；开放`submit` | C2冻结 |
| M6 | W09、W10、W11、W21；只开放数值槽重绑 | C3、C4冻结 |
| M7 | W12至W15、W23 | 各自另立任务 |

M3另核新增拒绝面。
既有合法输入不得被拒绝。
故障路径须覆盖越界与重叠。

### 变更范围

迁移可调整宿主校验层。
物理内核与容差保持不变。

物理内核指CUDA源码字符串与冻结PTX。
源码字符串内嵌在`src/physics/`的Rust文件中。
冻结PTX由L2按摘要常量加载。
同一文件的宿主代码允许修改。
内核字符串须逐字节不变。

两类摘要分别证明不同事实。

| 摘要 | 证明对象 | 不能证明 |
| --- | --- | --- |
| 源码摘要`sourceSha256` | 内核字符串逐字节不变 | 产物不变 |
| PTX摘要`ptxSha256` | 编译产物不变 | 源码不变 |

不同源码可能生成相同PTX。
例如编译器会忽略注释。
所以PTX摘要不能证明源码不变。
相同源码在配方变化后也可能产出不同PTX。
所以源码摘要也不能证明产物不变。
迁移须两类摘要同时不变。

`record_physics_ptx`已分别记录两者。
记录另含NVRTC版本与编译选项。
迁移前后各取一份记录对照。

各阶段的文件范围如下。

| 阶段 | 允许修改 | 禁止修改 |
| --- | --- | --- |
| M1、M2 | `src/runtime/`新增发射器模块 | 物理宿主代码 |
| M3 | 发射器；`src/runtime/transfer/kernel.rs`及子模块；十一个方法的调用方；构造元数据的宿主校验代码 | 内核字符串、容差、fixtures |
| M4 | 发射器；L2三文件 | 冻结PTX与摘要常量 |
| M5、M6 | 发射器；`TransferSession`；常驻底座的同步点 | 内核字符串、容差 |

M3调用方当前位于`src/physics/`。
静态检索得到约31个候选文件。
登记任务前用`codegraph`确认清单。
清单写入交付任务与回执。

每阶段单独建立交付任务。
M3之前不改变同步语义。

## 验收规则

源码与PTX摘要相同只证明内核不变。
它们不证明调度相同。
参数、依赖与维度仍可能改变。
所以两类摘要只是必要条件。

各阶段按路径分类判定。

| 路径类别 | 判定方式 | 说明 |
| --- | --- | --- |
| 确定性路径 | 逐位比较 | 无原子累加的内核 |
| 原子路径 | 沿用冻结门禁 | 原容差与原参考不变 |
| 分布记录 | 仅作补充证据 | 不得豁免任何超差 |
| 故障路径 | 检查隔离与泄漏 | 注入错误后核对状态 |
| 性能路径 | 记录耗时与显存 | 区分冷启动与稳态 |

每阶段另核以下调度要素：

- 入口名、源码摘要与PTX摘要。
- NVRTC版本与编译选项。
- 参数槽内容与顺序。
- 网格、块与共享区大小。
- 流依赖与同步点位置。

分布记录不能替代原门禁。
超差仍按原规则阻断。

## 精度约束

发射器不得修改内核源码。
发射器不得改动编译选项。
物理配方只有一个来源。
配方变化视为物理变更。
配方变化须重新走NP01至NP04。
迁移不得放宽任何容差。

## 退场与保留

| 路径 | 处置 |
| --- | --- |
| L1 | 并入发射器后删除私有方法 |
| L2 | 并入发射器；保留摘要核验 |
| L3 | 探针命令保留；图能力按C4重写 |
| L4 | 待决：保留探针或移除 |
| L5 | 保持仓外；W13完成后再评估 |

## 待决与待核实

- 条件图节点的驱动最低版本。
- feature升至`cuda-12030`以上的影响。
- CubeCL探针是否继续保留。
- 产品内核来源：手写移植或冻结PTX。
- nvJitLink的许可与部署方式。
- 多线程宿主调用的上下文策略。
- 粘滞错误的判定清单。

## 依据

- `src/runtime/transfer/kernel.rs`
- `src/runtime/transfer/kernel/kinetic_energy.rs`
- `src/runtime/transfer.rs`中的`SessionStatus::execute`
- `src/runtime/warp_kinematics.rs`
- `src/runtime/warp_gravity.rs`
- `src/runtime/cuda.rs`中的探针图实现
- `src/runtime/cache.rs`中的载荷校验
- cudarc 0.18.2源码`driver/safe`与`driver/sys`
- 冻结上游`solver.py`、`derivative.py`、`collision_flex.py`
- [CUDA驱动图接口](https://docs.nvidia.com/cuda/cuda-driver-api/cuda_driver_api/group__CUDA__GRAPH.html)
- [架构与接口](architecture.md)的GPU运行时契约
