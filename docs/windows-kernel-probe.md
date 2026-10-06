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
| `block-reduce` | 共享内存树形归约 | 每个128线程块输出一个u32部分和 |
| `block-scan` | 共享内存包含式扫描 | 每块单独计算前缀；不合并块间偏移 |
| `control-flow` | 数据相关循环与分支 | 循环1至7轮；奇偶分支；闭式整数参考 |
| `small-solve` | 固定2x2正定矩阵消元 | 批量求解两项未知数；检查已知解与残差 |

归约与扫描使用128项共享区。
尾块线程以零填充共享区。
所有线程统一参加屏障。
扫描先完成读取，再统一写入。
块扫描不等于全局扫描。
固定矩阵不等于通用求解器。
整数原子不证明浮点原子能力。

实现位置：

- `src/runtime/cubecl.rs`：六种Rust内核与两路编译。
- `src/runtime/cuda.rs`：类型化缓冲、事件与图重放。
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
它不表示正式引擎接口。

| 项目 | 约束 |
| --- | --- |
| 线程块 | 128 × 1 × 1 |
| 输入索引 | u32；保持检查模式 |
| 内核参数 | 输入指针、输出指针、元数据指针 |
| 元数据 | 两个连续u32长度，共8字节 |
| `--elements` | 1至1048576；求解探针按系统个数计数 |
| `--replays` | 1至1000；默认3轮 |
| 原子输出长度 | 1个u32及16项守卫 |
| 归约输出长度 | `ceil(elements / 128)`个u32及守卫 |
| 扫描、分支输出长度 | `elements`个u32及守卫 |
| 求解输入长度 | `2 * elements`个f32 |
| 求解输出长度 | `2 * elements`个f32及守卫 |
| 整数守卫 | `0xdead_beef` |
| 浮点守卫 | `-12345.0` |

每种内核先编译并预热。
随后捕获单个内核节点。
捕获期间不编译或分配。
图保留缓冲与元数据地址。
每轮更新输入并重置输出。
跨流事件串行连接传输与执行。
程序下载结果并检查守卫。
程序先完成执行，再释放图。
图先于缓冲释放。

本轮未更新图节点参数。
本轮未导出外部缓冲接口。
本轮未实现产品资源租约。

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
```

## Windows执行证据

本机采用RTX 4070 Ti SUPER。
设备报告SM 8.9。
驱动发行号为596.36。
驱动API版本为13020。
GNU与MSVC Rust均为1.99.0。
MSVC与SDK版本见[安装复验](cubecl-probe.md#msvc与llvm复验)。

每路覆盖六种内核。
每种覆盖以下七种规模：

```text
1, 127, 128, 129, 257, 4097, 1048576
```

每次双路回归覆盖84组配置。
每组完成预热与3轮图重放。
两路共完成252轮图重放。
所有配置均检查输出与守卫。

| 已执行检查 | 最终结果 |
| --- | --- |
| `cargo test --locked` | 24通过；默认不执行GPU |
| GNU C++特性宿主测试 | 26通过；3项GPU默认忽略 |
| MSVC LLVM特性宿主测试 | 26通过；3项GPU默认忽略 |
| MSVC全特性宿主测试 | 27通过；4项GPU默认忽略 |
| C++脚本GPU测试 | 1通过；42组配置 |
| LLVM脚本GPU测试 | 1通过；42组配置 |
| MSVC全特性双路GPU测试 | 2通过；84组配置 |
| MSVC原生GPU回归 | 2通过；六种长度与错误设备 |
| C++源码生成检查 | 六种内核通过；核对原子、共享区与屏障 |
| LLVM PTX生成检查 | 六种内核通过；不访问GPU驱动 |
| MSVC全特性Rust文档构建 | 通过 |
| 默认、CUDA、C++、LLVM与全特性Clippy | 通过；拒绝警告 |
| 原生新内核与缺少LLVM特性 | 分别返回退出码2与1；不回退 |
| 格式与PowerShell启动脚本解析 | 通过 |

完整Cargo命令沿用[复验命令](cubecl-probe.md#安装后复验)。
全特性检查显式调用MSVC目标。
GPU检查显式执行忽略测试。
本轮不启动Python。

## 下一批Windows工作

- 补浮点原子与全局扫描。
- 验证图节点参数更新。
- 验证外部缓冲及上下文。
- 明确在途资源租约。
- 核对生成产物与编译缓存。
- 验证无完整SDK的部署。
- 补最小无窗口图像输出。

Linux工作暂不开展。
P0模型与数值契约仍待收口。
物理与批量渲染仍未实现。
所有Normify模块保持计划态。
