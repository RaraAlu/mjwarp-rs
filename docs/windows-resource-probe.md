# Windows资源租约探针

日期：2026-10-06。
状态：P1内部资源原型。

本轮只推进Windows。
本轮不推送GitHub。
本轮不冻结编译主路线。
本轮不激活Normify模块。
物理与渲染仍未实现。

## 实现文件

```text
src/
  diagnostics/mod.rs      资源错误类别
  main.rs                 resources命令
  runtime/
    mod.rs                同步探针接口
    lease.rs              字节范围与所有权
    completion.rs         完成队列与令牌
    cuda.rs               设备准备与图工具
    cuda/resources.rs     真实CUDA资源探针
tests/
  runtime_resources.rs    三路GPU矩阵
```

新增文件承担真实行为。
本轮没有铺设空壳模块。
本轮没有新增Cargo依赖。
本轮保留全部冻结版本。
许可记录见[双路报告](cubecl-probe.md#依赖与许可)。

## 所有权与范围

适配层移动真实CUDA缓冲。
内部包装独占原始所有者。
包装从所有者读取设备属性。
它读取真实容量与上下文。
它不信任调用方填写容量。
租约持有所有者的强引用。
弱引用只供探针检查寿命。

| 检查 | 当前行为 |
| --- | --- |
| 设备身份 | 匹配实际CUDA设备序号 |
| 上下文身份 | 匹配实际`CUcontext`句柄 |
| 字节范围 | 拒绝零长度、溢出与越界 |
| 对齐 | 偏移与长度均按4字节对齐 |
| 布局版本 | 初始1；在途拒绝变更；过期请求报错 |
| 读权限 | 允许重叠读租约 |
| 写权限 | 拒绝只读缓冲写入 |
| 写别名 | 拒绝范围重叠的读写或写写租约 |
| 不相交范围 | 允许不相交写租约；宿主测试覆盖 |
| 编号与版本 | 拒绝u64耗尽；不回绕 |
| 锁异常 | 拒绝中毒注册表 |
| 隔离状态 | 拒绝失败资源的新租约 |

原型只处理连续f32视图。
GPU内核执行`y = 2x + 1`。
输入与输出各留两侧守卫。
每侧均包含16个f32。
视图偏移固定为64字节。
视图只包含有效元素。
CubeCL元数据排除两侧守卫。
程序精确核对结果与守卫。
直接等待路径也核对输入。

| 字段 | 含义 |
| --- | --- |
| `view_bytes` | `elements * 4` |
| `output_owner_bytes` | `(elements + 32) * 4`；不是总显存 |
| `completed_submissions` | 6个成功队列任务；不含图预热 |
| `failed_submissions` | 1个模拟宿主失败；不代表设备故障 |
| `rejected_requests` | 16次资源边界拒绝 |
| `graph_replays` | 1次真实仿射图重放 |

GPU探针创建异域上下文。
两个上下文使用同一GPU。
探针验证租约拒绝跨域请求。
它不向异域提交物理任务。
它不伪造异域句柄。
上下文语义见[CUDA上下文管理](https://docs.nvidia.com/cuda/cuda-driver-api/cuda_driver_api/group__CUDA__CTX.html)。

## 完成队列

队列先登记资源再提交内核。
队列持有租约与内核模块。
它同时持有元数据与图。
完成令牌只持有状态回执。
令牌不控制缓冲寿命。
丢弃令牌不释放在途资源。
遗忘令牌只泄漏该状态回执。
销毁队列会等待已提交任务。
图先于其缓冲与模块释放。

查询完成不会立即退还租约。
调用方须等待或销毁队列。
队列拒绝重新提交完成任务。
队列拒绝另一队列的令牌。
宿主测试也覆盖乱序退还。

未记录栅栏保持待定状态。
未记录路径以流同步清理。
仅`CUDA_ERROR_NOT_READY`代表待定。
其他CUDA错误保留阶段与码。
队列不将设备错误视为待定。
CUDA事件语义见[官方事件文档](https://docs.nvidia.com/cuda/cuda-driver-api/cuda_driver_api/group__CUDA__EVENT.html)。

生产、执行与下载各用一条流。
生产流记录上传完成事件。
执行流等待上传事件。
下载流等待内核完成事件。
调用方在提交前释放输入句柄。
直接等待、查询与图路径中，
调用方也提前释放输出句柄。
队列仍保留真实所有者。
图路径也提前释放内核句柄。
队列继续保留编译模块。

图预热写入独立缓冲。
实际重放输出保持守卫初值。
这避免预热结果掩盖图失败。
捕获期间不编译或分配。
捕获只记录一个内核节点。

## 错误路径与限制

提交错误会隔离资源。
成功清理不抹去失败状态。
失败状态保留首次错误。
清理失败也不会冒充完成。

GPU测试模拟宿主提交失败。
模拟发生在内核提交之后。
此时尚未记录完成事件。
队列等待执行流再释放资源。
原有缓冲拒绝新的租约。
探针不向GPU注入非法访问。

宿主模拟覆盖等待与查询失败。
它覆盖析构期间栅栏panic。
若无法证明设备完成，
队列保守保留全部资源。
此路径会泄漏资源与上下文。
产品恢复策略仍需另定。
成功路径检查所有者正常释放。

本原型不接收外部裸指针。
它不冻结外部tensor ABI。
它不支持步长或二维视图。
它不定义物理字段设备布局。
它不验收CubeCL运行时调度。
它不覆盖跨进程CUDA共享。
它不覆盖在途图参数更新。
设备故障隔离仍待真实验收。
清洁部署与缓存仍待验证。
Linux验收本轮暂缓。

## 执行命令

在既有Windows开发环境执行。
工具准备见[双路说明](cubecl-probe.md#windows私有工具)。

```powershell
cargo run --locked --features cuda-probe -- resources
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend cpp -ResourceTests
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend llvm -ResourceTests
```

CLI仅接收以下配置：

```text
resources --backend native-ptx|cubecl-cpp|cubecl-llvm --device N --elements N
```

默认路线为`native-ptx`。
默认设备为0，元素数为257。
元素数限制为1至1048576。
命令拒绝`--kernel`与`--replays`。
脚本拒绝两种矩阵开关同用。
缺少特性或设备均明确报错。
程序不回退CPU或更换路线。

## 实际验证

设备：RTX 4070 Ti SUPER。
SM：8.9；驱动：596.36。
驱动API：13020。
Rust：1.99.0。
GNU保持默认工具链。
LLVM与全特性使用MSVC。
工具版本沿用[安装后复验](cubecl-probe.md#安装后复验)。

资源矩阵使用五种规模：

```text
1, 129, 257, 4097, 1048576
```

每路各执行5组配置。
三路合计15组配置。
每次全矩阵验证90个成功任务。
它验证15个宿主失败任务。
它验证240次边界拒绝。
它验证15次真实图重放。
这些数量不代表物理验收。

| 检查 | 实际结果 |
| --- | --- |
| `cargo test --locked` | 46项宿主测试通过；默认不执行GPU |
| GNU C++特性宿主测试 | 50项通过；6项GPU测试默认忽略 |
| MSVC全特性宿主测试 | 调试与发行各51项通过；8项GPU测试默认忽略 |
| C++脚本`-ResourceTests` | 3项通过；原生及C++共10组配置，另测错误设备 |
| LLVM脚本`-ResourceTests` | 3项通过；原生及LLVM共10组配置，另测错误设备 |
| MSVC全特性`runtime_resources` | 调试与发行各4项通过；三路15组配置及错误设备 |
| MSVC全特性`cubecl_probe` | 调试与发行各2项通过；两路各74组既有配置 |
| MSVC全特性`gpu_probe` | 调试与发行各2项通过；原生事件与图回归 |
| GNU C++既有GPU矩阵 | 1项通过；74组配置回归 |
| 三路`resources` CLI | 原生与C++为257元素；LLVM发行模式为1048576元素 |
| Clippy与Rust文档 | 默认及路线检查通过；不启动Python |
| 格式、脚本解析与差异检查 | 通过 |

Normify校验取得零错误。
它保留6项架构警告。
五项目标模块尚无源码。
运行时叶子映射15个文件。
本轮保持全部模块计划态。

全特性GPU复验采用以下命令。
先按脚本导入MSVC与CUDA环境。
发行模式再加`--release`。

```powershell
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --test runtime_resources --test cubecl_probe --test gpu_probe -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
```

文档核对CLI、脚本与测试源码。
本轮不运行Python审计脚本。
PowerShell检查新增路径与命令。
此检查不替代全仓文档审计。

首次编译发现可见性与移动错误。
本轮修正后重新执行全部检查。
CLI重编译曾遇到文件锁冲突。
本轮采用串行重试。
串行重试随后通过。
发行测试不等于清洁部署。

## 下一批Windows工作

本轮后续已验证PTX缓存。
后续也通过受限进程烟测。
详情见[缓存报告](windows-artifact-probe.md)。
清洁部署仍需独立机器证据。

- 核对JIT产物与缓存失效。
- 验证缺少开发SDK的运行。
- 收口外部缓冲ABI与租约。
- 明确设备故障恢复边界。
- 补最小无窗口图像输出。

P0模型与数值契约仍待收口。
P1完整准入仍未完成。
