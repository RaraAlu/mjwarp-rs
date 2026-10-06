# Windows外部资源ABI探针

日期：2026-10-06。
状态：P1私有导入原型。

本轮只推进Windows。
本轮不推送GitHub。
本轮不冻结生产张量ABI。
本轮不激活架构模块。
物理与渲染仍待实现。

## 实现文件

```text
include/
  mjwarp_probe.h              C数据声明；不导出函数
src/
  diagnostics/mod.rs          外部资源错误
  main.rs                     external-resources命令
  runtime/
    mod.rs                    同步探针入口
    external.rs               描述符检查与弱注册表
    lease.rs                  复用字节租约
    completion.rs             复用完成队列
    cuda.rs                   复用设备与图工具
    cuda/resources.rs         共用栅栏与仿射启动
    cuda/external.rs          私有导入及客户端模拟
tests/
  external_resources.rs       三路GPU矩阵
  native/external_descriptor.cpp
                              C++布局断言
scripts/
  test-windows-external.ps1   MSVC布局与GPU检查
```

新增文件承担实际行为。
本轮不新增Cargo依赖。
本轮不修改冻结版本。
许可沿用[双路记录](cubecl-probe.md#依赖与许可)。

## 原型数据声明

Rust采用`repr(C)`。
C头使用固定宽度整数。
两侧核对全部字段偏移。
双方采用x64进程内表示。
它不是序列化格式。
它不提供C引擎函数。

| 字段 | 偏移 | 类型 | 单位与规则 |
| --- | ---: | --- | --- |
| `abi_version` | 0 | u32 | 固定1 |
| `struct_size` | 4 | u32 | 字节；固定96 |
| `device_ordinal` | 8 | u32 | 本进程CUDA设备序号 |
| `flags` | 12 | u32 | 0只读；1允许写；拒绝其他位 |
| `context` | 16 | u64 | 本进程`CUcontext`句柄值 |
| `allocation` | 24 | u64 | 传统设备分配的基址 |
| `allocation_bytes` | 32 | u64 | 字节；声明容量不超实际容量 |
| `offset_bytes` | 40 | u64 | 字节；相对分配基址；4字节对齐 |
| `elements` | 48 | u64 | f32元素数；描述符上限u32::MAX |
| `stride_bytes` | 56 | u64 | 字节；固定4 |
| `layout_version` | 64 | u64 | 非零；首次导入要求1 |
| `scalar_type` | 72 | u32 | 1表示f32；拒绝其他值 |
| `rank` | 76 | u32 | 固定1；不支持多维步长 |
| `reserved` | 80 | u64[2] | 两项均须为零 |

结构大小为96字节。
结构对齐为8字节。
整数字段不构造Rust枚举。
纯宿主验证不解引用地址。
验证检查相加溢出与范围。
验证拒绝空视图与非连续布局。
探针元素上限仍为1048576。
生产模型字段布局仍待决议。

## 驱动查询与范围

私有导入器先检查描述符。
它再绑定实际执行上下文。
它采用单项指针属性查询。
它不信任填写的容量与身份。

| 驱动证据 | 行为 |
| --- | --- |
| `MEMORY_TYPE` | 只允许设备内存 |
| `IS_MANAGED` | 拒绝托管分配 |
| `MEMPOOL_HANDLE` | 拒绝异步池分配 |
| `CONTEXT` | 匹配真实执行上下文 |
| `DEVICE_ORDINAL` | 匹配实际设备序号 |
| `BUFFER_ID` | 标识分配代次；参与注册键 |
| `cuMemGetAddressRange` | 查询传统分配基址及实际容量 |

导入器拒绝内部分配地址。
声明容量可以小于实际容量。
设备查询不证明元素类型。
驱动也不证明外部所有权。
调用方必须保证本地传统分配。
原型不接收IPC或虚拟重映射。
原型不接收任意张量框架。

依据：[CUDA指针属性](https://docs.nvidia.com/cuda/cuda-driver-api/cuda_driver_api/group__CUDA__UNIFIED.html)、[CUDA分配范围](https://docs.nvidia.com/cuda/cuda-driver-api/cuda_driver_api/group__CUDA__MEM.html)。

## 所有者与事件契约

导入器仍为私有unsafe函数。
它不导出生产导入入口。
公开接口只运行同步探针。
公开描述符只检查数据声明。

调用方承担以下安全前提：

- 所有者保留分配与上下文。
- 所有者允许引擎延长其寿命。
- 生产事件确实记录上传完成。
- 调用方不得重录生产事件。
- 调用方移交全部外部访问。
- 调用方不得提前释放分配。
- 调用方不得重映射该地址。
- 声明dtype匹配实际内核。

未记录事件不证明上传完成。
原型通过unsafe契约约束它。
它不靠事件查询猜测记录状态。
依据：[CUDA事件语义](https://docs.nvidia.com/cuda/cuda-driver-api/cuda_driver_api/group__CUDA__EVENT.html)。

探针在同进程模拟客户端。
客户端调用传统`cuMemAlloc`。
客户端保留上传源切片。
客户端记录实际生产事件。
它再移交所有者与事件。
执行流显式等待生产事件。
完成队列先保留资源再启动。
下载流等待内核完成事件。
下载返回前等待拷贝完成。

外部所有者负责实际释放。
引擎只释放所有者强引用。
弱引用核对所有者及事件寿命。
原型也核对实际释放计数。
C++测试只核对数据布局。
真实C客户端仍待接入。
C回调及销毁线程ABI仍待决议。

注册表使用进程级互斥锁。
注册键包含上下文与分配代次。
注册表只保存所有者弱引用。
在途租约保留所有者强引用。
根句柄释放不允许重复导入。
所有者释放后才清除旧注册。
此规则不监控外部裸指针写入。
外部竞态仍属于安全前提。

## 失败与图边界

探针复用内部完成队列。
令牌只保留完成状态。
丢弃或遗忘令牌不释放缓冲。
队列析构等待已提交工作。
未记录完成事件时等待执行流。
宿主提交失败隔离相关租约。
清理成功不覆盖原始失败状态。

清理失败保守保留资源。
上传失败等待生产流。
下载失败仍等待已排入拷贝。
无法证明完成时泄漏相关资源。
此策略不提供设备故障恢复。
本轮不注入设备非法访问。

图路径先编译并分配资源。
预热写入独立外部分配。
捕获只记录仿射内核。
图任务独立保留内核模块。
真实输出只由图重放写入。
资源字段先销毁图，再释放租约。
生产图失效与更新策略仍待定。

## 实际验证

| 环境 | 本轮记录 |
| --- | --- |
| Windows | NT 10.0.26200.0；x64 |
| GPU | RTX 4070 Ti SUPER；SM 8.9 |
| 驱动 | 596.36；驱动API 13020 |
| Rust | 1.99.0；保留GNU默认 |
| MSVC | VS 2022 Build Tools；x64 |
| CUDA | 私有12.8.1；沿用现有工具链 |
| CubeCL | 冻结`1f73b9f63de50a17398c1d5278e2a5f11612c7e1` |

三路各覆盖五种规模。
规模为1、129、257与4097。
第五项为1048576。
每组执行以下七种模式。

| 模式 | 检查 |
| --- | --- |
| Wait | 根句柄提前释放；等待后核对输入与输出 |
| Query | 查询完成不释放租约；拒绝重新提交 |
| DropToken | 丢弃令牌；队列仍等待完成 |
| ForgetToken | 遗忘令牌；外部所有者仍实际释放 |
| DropQueue | 析构等待；随后读取外部输出 |
| HostFailure | 启动后模拟宿主失败；隔离且保留错误 |
| Graph | 独立预热输出；释放原模块后重放 |

每组包含6次成功提交。
每组另含1次宿主失败。
每组完成1次真实图重放。
每组保留15项外部生产依赖。
CubeCL元数据另设依赖。
每组导入并释放15个所有者。
每组验证17次预期拒绝。
三路矩阵共90次成功提交。
矩阵另含15次宿主失败。
矩阵共225次所有者释放。
矩阵共255次边界拒绝。
矩阵共15次真实图重放。

17次拒绝分为以下类别：

- 8次身份、容量与内存拒绝。
- 2次重复导入拒绝。
- 3次读写及在途布局拒绝。
- 3次过期布局拒绝。
- 1次失败隔离拒绝。

8次包含真实异域分配。
它也包含页锁定宿主内存。
它还包含托管与池分配。
宿主另外覆盖描述符非法值。
f32结果与两侧守卫精确核对。
每侧固定16个守卫元素。
视图固定偏移64字节。
本轮不放宽既有数值容限。

| 检查 | 结果 |
| --- | --- |
| 默认宿主 | 56项通过；不运行GPU |
| GNU C++宿主 | 73项通过 |
| MSVC隔离LLVM宿主 | 73项通过 |
| MSVC全特性宿主 | 调试及发行各74项通过 |
| C++17 x64布局 | 大小、对齐、14项偏移及常量通过 |
| MSVC外部GPU矩阵 | 调试及发行各4项通过 |
| GNU C++外部GPU矩阵 | 3项通过；包含原生路线 |
| MSVC全量GPU回归 | 调试及发行各16项通过 |
| Clippy | 默认、CUDA、C++、LLVM及全特性通过 |
| rustdoc | 全特性拒绝警告并通过 |

全量回归包含已有缓存检查。
它也包含内核及内部租约检查。
脚本语法检查通过。
本地文档链接42项通过。
新增源码路径6项通过。
本轮不运行Python审计工具。
全仓文档审计仍待开展。
仅驱动部署烟测重新通过。
消费程序执行68组GPU配置。
它完成204轮图重放。
它完成240次节点更新。
107次子进程含5次预期失败。
报告真实记录`passed: true`。
证据目录如下：

```text
C:/Rust/mjwarp-rs/target/deployment-probe/d62ba55b8ee745b99b93a4f94968f03d/
  report.json
  consumer-dependencies.txt
  consumer-imports.txt
  producer/mjwarp-rs.exe
  driver/mjwarp-rs.exe
  cache/
```

此烟测不等于清洁部署验收。
零GPU用例不表示物理验收。
本轮没有执行Python。

本轮实际执行以下命令：

```powershell
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo run --locked --features cuda-probe -- external-resources

# 先设置私有GNU及CUDA工具链。
cargo +stable-x86_64-pc-windows-gnu test --locked --offline --features cubecl-cpp-probe --quiet
cargo +stable-x86_64-pc-windows-gnu clippy --locked --offline --all-targets --features cubecl-cpp-probe -- -D warnings
cargo +stable-x86_64-pc-windows-gnu test --locked --offline --features cubecl-cpp-probe --test external_resources -- --ignored --test-threads=1

# 以下命令先导入MSVC环境。
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --offline --release --all-features --quiet
cargo +stable-x86_64-pc-windows-msvc test --locked --offline --release --all-features --tests -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --offline --features cubecl-llvm-probe --quiet
cargo +stable-x86_64-pc-windows-msvc clippy --locked --offline --all-targets --features cubecl-llvm-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --offline --all-targets --all-features -- -D warnings
$env:RUSTDOCFLAGS = '-D warnings'
cargo +stable-x86_64-pc-windows-msvc doc --locked --offline --all-features --no-deps

.\scripts\test-windows-external.ps1
.\scripts\test-windows-external.ps1 -Release
.\scripts\test-windows-deployment.ps1
cargo fmt --check
git diff --check
```

## Normify证据

变更ID如下：

```text
2026-10-06-windows-external-resources
```

运行时新增六项源码映射。
运行时共映射25个文件。
诊断模块记录四类新增错误。
目标契约继续保持原数量。
工具刷新实际源码指纹。
全部模块继续保持计划态。
校验取得零错误与六项警告。
一项提示运行时叶子过粗。
五项提示计划源码尚未落地。
本轮不伪造产品激活状态。

## 后续边界

- 决议已编译模型的输入ABI。
- 决议原生依赖与字段精度。
- 设计生产所有者与事件租约。
- 决议C回调及销毁线程契约。
- 评估池、IPC与通用张量适配。
- 补独立清洁部署机器证据。
- 收口完整编译工具指纹。
- 明确设备故障恢复边界。

原型不代替生产安全接口。
P0全量语义契约仍待收口。
P1正式准入仍未完成。
Linux工作继续暂缓。
正式准入仍要求双平台。
