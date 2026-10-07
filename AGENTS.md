- 需要在完成一次合理编码时自动进行智能提交

# Repository Guidelines

## 项目定位与边界

项目只负责MJWarp引擎能力。
当前代码提供无头GPU探针。
基础辅助与原生输入探针已落地。
十二项运动学字段支持GPU上传。
九项惯性字段支持组合上传。
二十一字段仍属辅助子集。
刚体运动学已有GPU探针。
几何与site位姿也有GPU探针。
常驻底座复用三个设备子集。
十三项参数支持独立批量。
拓扑仍只支持共享参数。
质心与自由度映射也有探针。
质量矩阵已有GPU子集探针。
质量求解已有正定辅助探针。
求解内部采用GPU f64计算。
公开物理输入与结果保持f32。
它不替代完整G01阶段。
GPU引擎尚待实现。
首版面向NVIDIA。
验收覆盖Windows与Linux。
对齐文档中的冻结上游。
Rust提供主API与宿主接口。
允许接入C/C++组件与内核。
不要引入Python构建或运行链。
产品测试也不得启动Python。
允许接入原生模型与参考工具。
引擎转换并上传已编译模型。
不要加入CPU play后端。
不要加入窗口或查看器。
不要加入策略与Burn适配。
LRsLab负责应用调用与前端。
LRsLab负责MJCF编译。
LRsLab负责CPU play。
新增依赖仍检查版本与许可。

## 项目结构

| 路径 | 用途 |
| --- | --- |
| `Cargo.toml` | 指定包信息与语言版本 |
| `Cargo.lock` | 锁定依赖版本 |
| `src/main.rs` | 提供探针命令入口 |
| `src/lib.rs` | 导出七模块基础接口与探针 |
| `src/model/` | 检查布局、原生DTO与运动学惯性输入 |
| `src/model/attached.rs` | 检查几何与site六项附着字段 |
| `src/model/parameters.rs` | 检查运动学字段独立批量 |
| `src/io/` | 转换、GPU字段交换与只读字段组 |
| `src/io/fields.rs` | 提供独立连续字段底层 |
| `src/runtime/` | 实现驱动级GPU探针 |
| `src/physics/` | 历史校验与刚体GPU探针 |
| `src/physics/attached.rs` | 复用刚体设备结果计算附着位姿 |
| `src/physics/resident.rs` | 常驻刚体、附着与质心设备链 |
| `src/physics/mass_matrix.rs` | 复合惯量与稠密质量矩阵 |
| `src/physics/mass_solve.rs` | 正定分解与多右端项求解 |
| `src/runtime/transfer/kernel.rs` | 封装内部同步内核ABI |
| `src/diagnostics/` | 定义探针错误 |
| `native/`与`include/` | 提供可选原生桥接 |
| `build.rs` | 按feature编译原生桥接 |
| `tests/` | 存放Rust GPU与原生探针测试 |
| `fixtures/native-probe/` | 保留静态MJB、源XML、结构打印件与哈希 |
| `fixtures/kinematics/` | 保留静态原生运动学参考 |
| `fixtures/attached-kinematics/` | 保留几何与site原生参考 |
| `fixtures/com-position/` | 保留质心参考与零质量边界 |
| `fixtures/mass-matrix/` | 保留矩阵参考与armature链 |
| `fixtures/mass-solve/` | 保留原生因子、右端项与解 |
| `docs/README.md` | 提供文档索引与现状 |
| `docs/project-positioning.md` | 明确产品与依赖边界 |
| `docs/architecture.md` | 描述目标模块与接口 |
| `docs/compatibility-matrix.md` | 管理功能与验收项 |
| `docs/implementation-plan.md` | 划分实施阶段 |
| `target/` | 存放构建产物；Git忽略此目录 |

先读文档索引与项目定位。
区分目标架构与现有代码。
以下目录尚未创建：

```text
benches/    性能测试
examples/   接入示例
```

先扩展内部模块。
复用确有需要时再拆crate。

## 构建与开发命令

在仓库根目录执行命令。
默认构建不需要GPU工具链。
GPU探针需要NVIDIA驱动。
探针不选择正式内核路线。

| 命令 | 作用 |
| --- | --- |
| `cargo build` | 编译调试程序 |
| `cargo run -- --help` | 显示探针命令 |
| `cargo run --locked --features cuda-probe -- probe` | 执行GPU探针 |
| `cargo test` | 运行Rust测试 |
| `cargo fmt --check` | 检查格式 |
| `cargo fmt` | 格式化源码 |
| `cargo clippy --all-targets -- -D warnings` | 检查代码并拒绝警告 |

原生输入探针暂限Windows MSVC。
它通过可选feature编译C++桥接。
默认GNU工具链保持不变。
先运行`prepare-native-probe.ps1`。
再运行`test-windows-native.ps1`。
脚本位于`scripts/`目录。
详见原生模型输入探针报告。

## 编码风格与命名

采用Rust 2024语法。
使用四空格缩进。
沿用rustfmt默认格式。
按职责划分模块。
不要把训练逻辑放入核心。

| 对象 | 命名方式 |
| --- | --- |
| 函数、变量、模块 | `snake_case` |
| 类型、trait | `UpperCamelCase` |
| 常量 | `SCREAMING_SNAKE_CASE` |

将unsafe集中到适配层。
为每处unsafe说明安全前提。
用安全Rust接口封装FFI。
记录缓冲所有权与完成事件。
明确报告越界与容量错误。
不要静默回退到CPU。

## 测试要求

当前仓库含探针测试。
GPU测试默认忽略。
显式执行这些测试：

```text
cargo test --locked --features cuda-probe --test gpu_probe -- --ignored --test-threads=1
```

详细证据见探针报告。
刚体探针显式检查参考哈希。
执行以下Windows脚本：

```text
scripts/test-windows-kinematics.ps1 -AllFeatures
scripts/test-windows-kinematics.ps1 -AllFeatures -Release
```

该脚本需要NVRTC。
附着运动学也需要NVRTC。
执行以下Windows脚本：

```text
scripts/test-windows-attached-kinematics.ps1 -AllFeatures
scripts/test-windows-attached-kinematics.ps1 -AllFeatures -Release
```

质量矩阵脚本也需要NVRTC。
执行以下Windows脚本：

```text
scripts/test-windows-mass-matrix.ps1 -AllFeatures
scripts/test-windows-mass-matrix.ps1 -AllFeatures -Release
scripts/test-windows-mass-solve.ps1 -AllFeatures
scripts/test-windows-mass-solve.ps1 -AllFeatures -Release
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
```

产品测试不生成原生参考。
项目尚未设覆盖率门槛。
新增逻辑须配套Rust测试。
使用内置测试框架。
用`#[test]`标记测试函数。
在模块内添加单元测试。
在`tests/`添加集成测试。
按行为命名测试：

```rust
#[test]
fn rejects_invalid_dimensions() { /* ... */ }
```

覆盖正常输入与失败边界。
物理测试关联矩阵功能ID。
固定种子、样本与误差容限。
不要为优化放宽数值容限。
记录平台、GPU与驱动版本。
缺少GPU时明确记录未执行。
不得将零用例视为物理验收。

## 提交与合并请求

仓库已有中文历史提交。
采用以下中文标题格式：

```text
<中文类型>(<scope>): <简洁中文标题>
文档(指南): 添加贡献指南
```

跨模块时可省略scope。
正文说明变更与真实验证。
多项变更使用Markdown栏目：
`**变更内容**`与`**验证**`。
小范围变更可用紧凑正文。
不要添加署名或sign-off。
只暂存本目标相关内容。
用多个`-m`参数保留段落。

PR说明目的、范围与验证。
关联适用的议题与矩阵ID。
列出未完成项与兼容影响。
渲染变更附图像与误差。
同步更新文档与验收状态。
仅凭真实证据声明完成。

## 代理协作

用简明汉语解释。
每句不超过20字。
只用主动语态。
存在`.codegraph/`时先查图。
使用以下查询命令：

```text
codegraph explore "<符号或问题>"
codegraph node <符号或文件>
```

没有索引时跳过CodeGraph。
不要自行初始化索引。
