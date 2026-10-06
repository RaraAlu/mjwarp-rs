# Windows字段交换探针

日期：2026-10-07。
范围：同步连续GPU字段。
本轮不开展Linux验收。
本轮不冻结正式GPU路线。

## 本轮结果

连续字段已有真实GPU交换。
接口支持三类四字节元素。
宿主接口不导出设备地址。
新增依赖与默认工具链均不变。
测试与脚本不调用Python。
GPU物理与渲染仍待实现。

新增源码如下：

```text
C:/Rust/mjwarp-rs/
  src/runtime/transfer.rs
  tests/batch_transfer.rs
  scripts/test-windows-transfers.ps1
  docs/windows-transfer-probe.md
```

已有模块另作实际扩展：

| 文件 | 本轮职责 |
| --- | --- |
| `src/diagnostics/mod.rs` | 登记元素范围与传输错误 |
| `src/runtime/mod.rs` | 导出同步传输门面 |
| `src/runtime/cuda.rs` | 复用驱动准入并创建独立流 |
| `src/io/mod.rs` | 组合布局、字段交换与严格转换 |

## 接口与布局

运行时提供以下辅助：

| 接口 | 实际行为 |
| --- | --- |
| `TransferElement` | 封闭支持`f32`、`i32`与`u32` |
| `TransferSession::new` | 检查真实驱动与设备；不加载编译器 |
| `TransferSession::upload` | 独占分配与全字段上传 |
| `TransferBuffer::read_range_into` | 暂存后回读指定范围 |
| `TransferBuffer::write_range` | 暂存后写入指定范围 |
| `TransferBuffer::copy_range_from` | 复制同会话的独立设备缓冲 |
| `TransferBuffer::copy_range_within` | 复制同缓冲的不相交范围 |

范围参数均按元素计数。
字节容量另作溢出检查。
空范围允许容量端点。
空范围也拒绝越界偏移。
相同自复制范围不执行复制。
部分重叠自复制返回错误。
缓冲不提供clone或外部导入。

IO层提供以下辅助：

| 接口 | 实际行为 |
| --- | --- |
| `DeviceBatch::upload` | 检查布局与元素字节，再上传 |
| `DeviceBatch::read_into` | 回读整个连续字段 |
| `DeviceBatch::read_world_into` | 回读指定世界 |
| `DeviceBatch::write_world` | 写入指定世界 |
| `DeviceBatch::copy_world_from` | 跨字段复制一个世界 |
| `DeviceBatch::copy_world_within` | 同字段复制一个世界 |
| `upload_f64_batch` | 严格转换为f32，再上传 |

布局采用`[world, element]`。
每个世界使用相同字段宽度。
跨字段复制允许世界数不同。
复制仍要求世界字段宽度相同。
元素字节数必须等于四。
本轮不描述共享或稀疏字段。
本轮不解释模型身份与字段单位。

原始复制保留全部位模式。
浮点可包含NaN与无穷。
严格f64辅助拒绝非有限值。
严格辅助也拒绝f32溢出。
严格辅助允许舍入与下溢。
两种入口不能混淆语义。
这些辅助不替代put_data。
这些辅助也不替代put_model。
完整Model与Data仍待实现。

## 所有权与完成边界

会话clone共享流与隔离状态。
独立创建的会话互不兼容。
同一物理GPU也遵守此规则。
该限制不判定真实上下文差异。
缓冲保留会话与实际上下文。
调用方可以提前释放会话句柄。
缓冲也可以移交另一线程。
每次操作重新绑定线程上下文。
共享会话串行执行设备操作。

成功返回前等待复制完成。
上传先复制到独占宿主暂存。
回读先写入独占宿主暂存。
等待成功后才改写调用方切片。
输入错误不改写目标缓冲。
后端错误隔离整个会话。
后续有效请求拒绝执行。
后端失败不承诺设备原值。

操作失败仍执行完成检查。
操作panic也执行完成检查。
两个步骤都失败时保留首错。
完成检查失败时保留资源。
宿主暂存与显存可能泄漏。
该策略避免未知DMA悬空访问。
本轮不提供隔离会话恢复。
故障策略采用宿主模拟测试。
本轮不注入真实GPU非法访问。

零元素字段不分配设备缓冲。
它仍要求真实GPU会话。
缺少feature时明确报错。
接口不创建CPU替代会话。
接口不提供异步完成令牌。
图捕获与外部租约另行收口。

上传暂存增加一次宿主复制。
初始化与传输另有同步成本。
本轮不宣称吞吐与零拷贝收益。
准入沿用驱动探针的检查。
CUDA驱动API要求至少12.0。
当前要求计算能力至少7.0。
这些检查不冻结正式版本底线。

## 调用示例

调用方启用`cuda-probe`。
运行时只要求NVIDIA驱动。

```rust
use mjwarp_rs::{
    diagnostics::TransferError,
    io::DeviceBatch,
    model::BatchLayout,
    runtime::TransferSession,
};

fn exchange() -> Result<(), TransferError> {
    let session = TransferSession::new(0)?;
    let layout = BatchLayout::new(2, 3, 4)?;
    let mut field = DeviceBatch::upload(&session, layout, &[0.0_f32; 6])?;
    field.write_world(1, &[1.0, 2.0, 3.0])?;
    field.copy_world_within(0, 1)?;
    let mut world = [0.0; 3];
    field.read_world_into(0, &mut world)?;
    assert_eq!(world, [1.0, 2.0, 3.0]);
    Ok(())
}
```

## Windows验证

GPU：RTX 4070 Ti SUPER。
驱动：596.36。
Rust：1.99.0。
主机：Windows 10.0.26200.0 x64。
MSVC：VS Build Tools 17.14.41。
Windows SDK：10.0.26100。
开发CUDA：私有12.8.1工具目录。
默认Rust工具链继续采用GNU。
LLVM与全特性检查显式用MSVC。

| 检查 | 本轮结果 |
| --- | --- |
| GNU默认宿主测试 | 99通过，零失败；另过2项文档测试 |
| GNU隔离C++宿主测试 | 115通过，零失败；另过2项文档测试 |
| MSVC隔离LLVM宿主测试 | 115通过，零失败；另过2项文档测试 |
| MSVC全特性宿主测试 | 调试与发行各116通过；各另过2项文档测试 |
| MSVC全特性GPU测试 | 调试与发行各26通过；零失败、零忽略 |
| GNU隔离CUDA字段测试 | 新增10项GPU测试全部通过 |
| 仅驱动受限子进程 | 10项GPU测试通过；NVRTC不可见 |
| fmt与clippy | fmt通过；五种特性clippy通过 |
| rustdoc | 全特性生成通过；拒绝警告 |
| 文档与脚本 | 50个本地链接存在；PowerShell语法零错误；diff检查通过 |
| Normify | 零错误、零警告；仅激活同步辅助叶子 |

宿主新增13项通用测试。
默认构建另测feature缺失。
文档另测布尔类型准入失败。
字段示例也通过编译检查。
新增10项真实GPU测试。
世界数采用1、3与64。
字段宽度覆盖六种规模。
组合共包含18组布局。

```text
elements_per_world: 0, 1, 5, 129, 257, 4097
```

GPU测试覆盖三类位模式。
测试覆盖世界写入与双向复制。
测试核对错误后的原值。
测试核对会话clone与提前释放。
测试核对线程移交与并发串行。
测试核对空字段与错误设备。
原16项GPU探针也各自复验。
这些测试不构成物理验收。

### 实际命令

MSVC命令先加载VS环境。
编译路线另加载私有CUDA环境。
rustdoc设置`-D warnings`。
命令保持锁定依赖。

```text
cargo test --locked
cargo test --locked --features cubecl-cpp-probe
cargo test --locked --features cuda-probe --test batch_transfer -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --features cubecl-llvm-probe
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --release --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --release --all-features -- --ignored --test-threads=1
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo clippy --locked --all-targets --features cubecl-cpp-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --features cubecl-llvm-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
./scripts/test-windows-transfers.ps1
```

全特性默认忽略26项GPU测试。
本轮另行显式执行全部GPU测试。
零用例与忽略不能算作通过。
检查日志位于以下目录：

```text
C:/Rust/mjwarp-rs/target/transfer-check/
```

### 仅驱动烟测

脚本构建隔离CUDA测试目标。
脚本复制执行文件到独立目录。
子进程仅保留系统目录PATH。
脚本清除CUDA与LLVM环境变量。
子进程实际检查NVRTC不可见。
脚本核对完整测试通过记录。
脚本保留烟测失败报告。

本轮实际证据如下：

```text
C:/Rust/mjwarp-rs/target/transfer-probe/1a09cc4d5334410d8dacd1a34b7e2c37/
  report.json
  build.jsonl
  stdout.log
  stderr.log
  batch-transfer.exe
```

报告记录`passed=true`。
报告记录`nvrtcInvisible=true`。
报告记录10项通过与退出码零。
这仍属于开发机受限烟测。
系统内驱动继续参与执行。
独立清洁机器验收仍待开展。

## 架构证据

七顶层职责保持不变。
运行时新增transfer叶子。
运行时目前共八个叶子。
transfer只登记主实现文件。
容器登记测试与脚本证据。
运行时28项映射各登记一次。
原25项映射全部保留。
本轮不新增空壳或规则豁免。

```text
mjwarp.runtime
  driver       驱动与执行图
  buffers      内部缓冲租约
  completion   完成队列
  external     外部资源导入
  transfer     同步字段传输
  compiler     内核编译探针
  cache        可信产物缓存
  tooling      工具与命令入口
```

架构登记170项契约。
原141项契约全部保留。
本轮新增七项辅助与工具契约。
同步辅助叶子标记active。
该叶子只含三项实际接口。
其余15节点继续保持计划态。
生产API与路线仍未冻结。
真实指纹由Normify刷新。
rpc仍只表示工具协议。
项目不设计网络RPC服务。
零警告不证明正式引擎完成。

初次收尾拒绝计划态新叶子。
本轮核对新增叶子的实际范围。
本轮只激活已验证的同步辅助。
本轮不激活生产运行时。
该状态不替代双平台准入。

变更编号如下：

```text
2026-10-07-batched-device-transfers
```

## 后续边界

原生模型版本与ABI仍待收口。
字段单位与设备布局仍待冻结。
完整模型上传仍待实现。
控制与状态等价交换仍待实现。
GPU动力学与渲染仍待实现。
异步队列与图捕获仍需整合。
生产失败恢复与清洁部署仍待验收。
正式发布继续要求双平台证据。
