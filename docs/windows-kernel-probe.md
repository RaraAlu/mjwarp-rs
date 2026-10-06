# Windows内核能力探针

日期：2026-10-06。
状态：P1局部能力验证。

用户本轮只推进Windows。
本轮不处理Linux或GitHub。
首版双平台要求保持不变。
本轮不冻结引擎主路线。
本轮不宣称物理等价。

## 实现范围

两路共用同一组Rust定义。
C++路线采用NVRTC编译。
LLVM路线采用NVPTX目标。
程序不改选编译路线。
程序不回退CPU。
宿主只生成样本与测试参考。
默认内核仍为`affine`。
原生PTX只支持该默认内核。
原生路线拒绝其他内核。

| `--kernel` | 设备操作 | 数值与范围 |
| --- | --- | --- |
| `affine` | f32仿射变换 | `y = 2x + 1`，精确比较 |
| `atomic-sum` | 全局u32原子累加 | 所有线程竞争单个累加器；每轮清零 |
| `float-atomic-sum` | 全局f32原子累加 | 正负四分之一整数；每轮清零 |
| `block-reduce` | 共享内存树形归约 | 每个128线程块输出一个u32部分和 |
| `block-scan` | 共享内存包含式扫描 | 每块单独计算前缀；不合并块间偏移 |
| `global-scan` | 分层全局包含式扫描 | GPU提取块总和；递归扫描及偏移回填 |
| `control-flow` | 数据相关循环与分支 | 循环1至7轮；奇偶分支；闭式整数参考 |
| `small-solve` | 固定2x2正定矩阵消元 | 批量求解两项未知数；检查已知解与残差 |

归约与扫描使用128项共享区。
尾块线程以零填充共享区。
所有线程统一参加屏障。
扫描先完成读取，再统一写入。
块扫描不等于全局扫描。
固定矩阵不等于通用求解器。
整数原子不证明浮点原子能力。
新增探针独立验证f32原子。

实现位置：

- `src/runtime/cubecl.rs`：八项探针与两个扫描辅助内核。
- `src/runtime/cuda.rs`：分层工作区、事件与图节点更新。
- `src/runtime/samples.rs`：固定样本与独立参考。
- `src/runtime/mod.rs`：内核选择、容量与路线约束。
- `src/diagnostics/mod.rs`：精确整数及求解错误。
- `tests/cubecl_probe.rs`：两路GPU配置矩阵。

本轮没有新增Cargo依赖。
本轮保持既有冻结提交。
版本与许可见[双路报告](cubecl-probe.md)。

## 配置与布局

`ProbeConfig`新增`kernel`字段。
显式结构字面量需补该字段。
调用方也可沿用默认构造。
`ProbeReport`记录实际内核。
它记录图内核节点与更新数。
它不表示正式引擎接口。

| 项目 | 约束 |
| --- | --- |
| 线程块 | 128 × 1 × 1 |
| 输入索引 | u32；保持检查模式 |
| 内核参数 | 输入指针、输出指针、元数据指针 |
| 元数据 | 两个连续u32逻辑长度，共8字节；均不计守卫 |
| `--elements` | 1至1048576；求解探针按系统个数计数 |
| `--replays` | 1至1000；默认3轮 |
| 原子输出长度 | 1个u32或f32及16项守卫 |
| 归约输出长度 | `ceil(elements / 128)`个u32及守卫 |
| 扫描、分支输出长度 | `elements`个u32及守卫 |
| 求解输入长度 | `2 * elements`个f32 |
| 求解输出长度 | `2 * elements`个f32及守卫 |
| 整数守卫 | `0xdead_beef` |
| 浮点守卫 | `-12345.0` |

每种内核先编译并预热。
程序再捕获全部内核节点。
捕获期间不编译或分配。
图保留缓冲与元数据所有者。
每轮更新输入并重置输出。
跨流事件串行连接传输与执行。
程序下载结果并检查守卫。
程序先完成执行，再释放图。
图先于缓冲释放。

输出字节数只统计单个输出。
它不包含备用输出与工作区。
本轮未导出外部缓冲接口。
本轮未实现产品资源租约。

## 全局扫描与工作区

第一步计算各块局部前缀。
第二步提取各块末项。
尾块提取最后有效项。
第三步递归扫描块总和。
最后从小层向下回填偏移。
GPU执行全部扫描与回填。
CPU不计算设备执行偏移。
宿主只生成独立串行参考。

每层持有总和与前缀缓冲。
每个缓冲均附16项守卫。
元数据排除守卫长度。
程序逐项核对中间层结果。
每轮先重置全部工作区。

| 元素数 | 工作区逻辑长度 | 捕获节点数 |
| --- | --- | --- |
| 1至128 | 无 | 1 |
| 129 | 2 | 4 |
| 16384 | 128 | 4 |
| 16385 | 129、2 | 7 |
| 1048576 | 8192、64 | 7 |

节点数遵循`1 + 3 * 层数`。
宿主测试拒绝块内扫描结果。
这些规模覆盖递归层边界。

## 图节点参数更新

程序先保存捕获参数副本。
它不写驱动拥有的参数内存。
每轮交替选择两个输出。
奇数轮写备用输出。
偶数轮恢复原输出。
程序调用以下真实驱动接口：

```text
cuGraphKernelNodeGetParams_v2
cuGraphExecKernelNodeSetParams_v2
```

单内核图每轮更新一个节点。
跨块扫描每轮更新三个节点。
它替换根扫描的输出参数。
它替换首层总和的输入参数。
它替换根偏移回填的输出参数。
它保留辅助缓冲与元数据。
它不改变函数或启动维度。
它先等待，再更新节点。
它不测试在途并发图更新。

每轮核对新输出全部结果。
它核对未选输出保持初值。
它还检查两者的尾部守卫。
图先于两个输出及工作区释放。
错误路径也等待执行完成。
这只验证私有同型缓冲切换。
外部资源租约仍待收口。
容量、拓扑及上下文变更仍待测。

接口契约见[CUDA图管理](https://docs.nvidia.com/cuda/cuda-driver-api/cuda_driver_api/group__CUDA__GRAPH.html)。

## 独立参考与容限

整数样本采用固定序列。
序列同时加入重放轮次。
宿主采用串行和与扫描。
分支探针采用闭式参考。
参考不复制设备动态循环。
整数结果与守卫均精确比较。
宿主测试核对最大累加范围。
最大元素与1000轮不会溢出。
GPU矩阵只执行3轮重放。
新增两项长重放各执行10轮。

f32原子采用以下固定样本：

```text
q(i, epoch) = (i % 17) - 8 + (epoch % 9) - 4
x(i, epoch) = q(i, epoch) / 4
```

样本同时覆盖正负值。
每轮变化并每九轮回绕。
单项整数绝对值不超过12。
最大规模满足以下边界：

```text
sum(abs(q)) <= 12 * 1048576 < 2^24
```

任意部分和仍为精确整数。
四分之一缩放保持精确表示。
因此本样本采用精确比较。
宿主独立用f64计算参考和。
测试也核对i64整数参考。
非二进制小数仍需另定容限。
本结果不保证通用f32次序稳定。

求解矩阵固定如下：

```text
A = [[4, 1], [1, 2]]
```

宿主先生成已知解。
宿主再计算右端向量。
GPU独立执行消元。
宿主以f64核对两行残差。
解误差与残差共享以下界限：

```text
abs(actual - expected) <= 1e-6 * (1 + abs(expected))
```

程序拒绝NaN与无穷值。
守卫不采用浮点容限。
宿主测试固定容限边界。
这些容限只约束该固定矩阵。
物理数值容限仍需独立决议。

## 执行命令

在仓库根目录执行。
脚本只修改当前进程环境。
本轮保留GNU默认工具链。
LLVM脚本显式调用MSVC目标。
`-GpuTests`运行整组测试。
该开关不筛选`-Kernel`参数。

```powershell
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend cpp -GpuTests
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend llvm -GpuTests
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend cpp -Kernel block-scan -Elements 129
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend llvm -Kernel small-solve -Elements 257
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend cpp -Kernel global-scan -Elements 16385
pwsh -NoProfile -File scripts/run-cubecl-probe.ps1 -Backend llvm -Kernel float-atomic-sum -Elements 1048576
```

## Windows执行证据

本机采用RTX 4070 Ti SUPER。
设备报告SM 8.9。
驱动发行号为596.36。
驱动API版本为13020。
GNU与MSVC Rust均为1.99.0。
MSVC与SDK版本见[安装复验](cubecl-probe.md#msvc与llvm复验)。

每路覆盖八项探针。
每项覆盖以下九种规模：

```text
1, 127, 128, 129, 257, 4097, 16384, 16385, 1048576
```

每路矩阵覆盖72组配置。
每组先预热，再重放3轮。
两路各增两组10轮配置。
它们均采用16385个元素。
它们验证扫描及浮点原子。
每次双路回归共148组配置。
两路共完成472轮图重放。
两路共执行584次节点更新。
所有配置均检查输出与守卫。

| 已执行检查 | 最终结果 |
| --- | --- |
| `cargo test --locked` | 26通过；默认不执行GPU |
| GNU C++特性宿主测试 | 29通过；3项GPU默认忽略 |
| MSVC LLVM特性宿主测试 | 29通过；3项GPU默认忽略 |
| MSVC全特性宿主测试 | 30通过；4项GPU默认忽略 |
| C++脚本GPU测试 | 1通过；74组配置 |
| LLVM脚本GPU测试 | 1通过；74组配置 |
| MSVC全特性双路GPU测试 | 2通过；148组配置 |
| MSVC release双路GPU测试 | 2通过；同组148配置与472轮重放 |
| MSVC原生GPU回归 | 2通过；六种长度与错误设备 |
| LLVM全局扫描长重放 | 129项、1000轮、3000次节点更新通过 |
| C++源码生成检查 | 十种设备内核通过；含两个辅助内核 |
| LLVM PTX生成检查 | 十种设备内核通过；不访问GPU驱动 |
| MSVC全特性Rust文档构建 | 通过 |
| 默认、CUDA、C++、LLVM与全特性Clippy | 通过；拒绝警告 |
| 原生新内核与缺少LLVM特性 | 分别返回退出码2与1；不回退 |
| 格式与PowerShell启动脚本解析 | 通过 |

完整Cargo命令沿用[复验命令](cubecl-probe.md#安装后复验)。
全特性检查显式调用MSVC目标。
GPU检查显式执行忽略测试。
发行模式另执行以下命令：

```powershell
cargo +stable-x86_64-pc-windows-msvc test --locked --release --all-features --test cubecl_probe -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc run --locked --features cubecl-llvm-probe -- probe --backend cubecl-llvm --kernel global-scan --elements 129 --replays 1000
```

发行测试不等于清洁部署。
本轮不启动Python。

首次编译发现旧绑定名称。
本轮改用CUDA 12的v2接口。
首次Clippy发现测试模块位置。
本轮修正位置并重新检查。
本轮不修改第三方依赖源码。

## 下一批Windows工作

- 验证更多f32数值分布。
- 验证图布局变化与失败边界。
- 验证外部缓冲及上下文。
- 明确在途资源租约。
- 核对生成产物与编译缓存。
- 验证无完整SDK的部署。
- 补最小无窗口图像输出。

Linux工作暂不开展。
P0模型与数值契约仍待收口。
物理与批量渲染仍未实现。
所有Normify模块保持计划态。
