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
常驻底座复用六个设备子集。
局部入口使用七个结果缓冲。
混合入口另加GPU合并。
混合入口使用九个结果缓冲。
柔体入口另加位置子集。
纯位置入口使用十一个结果缓冲。
GPU计算节点与顶点位置。
它支持直接与线性插值。
柔体位置字段仍共享。
可选边入口另加两个浮点缓冲。
GPU计算边长、雅可比与速度。
qvel支持独立世界写入。
边拓扑与稀疏行仍共享。
GPU已计算线性壳体面位姿。
原生柔体位置转换已落地。
快照独占十二项位置字段。
转换仍拒绝二次插值。
可选面入口另加一个结果缓冲。
面四元数采用xyzw顺序。
柔体Hessian失效已落地。
GPU已缓存拉伸投影矩阵。
三角形与四面体采用21系数。
调用方显式触发矩阵阶段。
矩阵乘法仍待实现。
可选入口另加肌腱唤醒。
GPU刷新树标记与树计数。
范围与边距各用独立周期。
完整休眠仍待实现。
既有二十三项参数保留独立批量。
GPU支持相机与光源位姿。
GPU支持固定肌腱长度与力臂。
肌腱拓扑与系数仍共享。
空间肌腱支持site与pulley。
GPU支持球柱与内侧绕行。
几何尺寸支持独立周期。
两类肌腱各用局部编号。
混合入口保留原生全局编号。
常驻状态支持mocap写入。
GPU初始化静态geom缓存。
更新仍重算全部site。
拓扑仍只支持共享参数。
质心与自由度映射也有探针。
质量矩阵已有GPU子集探针。
质量求解已有正定辅助探针。
求解内部采用GPU f64计算。
公开物理输入与结果保持f32。
质量求解不进入G01调度。
G02空间速度已有常驻子集。
GPU计算cvel与cdof_dot。
RNE偏置力已有常驻子集。
GPU计算qfrc_bias。
速度与偏置力内部采用f64。
公开物理字段仍采用f32。
完整G02仍待实现。
G01已提供完整常驻入口。
G01也保留等价低层调用。
原生快照覆盖78项读字段。
Windows双模式验收通过。
Linux双模式验收通过。
Linux GPU采用RTX 5080。
T4完整G01仍待复验。
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

## 计算精度硬性指标

产品精度必须对齐冻结MJWarp。
该规则覆盖全部物理路径。
辅助探针也须遵守此规则。
上游提交采用以下基线：

```text
71da24d956378a87a703b6e1442b13aec0c4ac29
```

逐项核对输入与输出类型。
同时核对中间量与累加类型。
核对常量、函数与编译选项。
核对模板生成后的实际内核。
类型匹配率必须达到100%。
产品精度偏离必须为零。
上游采用f32时全链保持f32。
不得私自提升为f64。
不得私自降低为f16或TF32。
上游采用混合精度时逐项跟随。
仅保持公开f32接口不算达标。
不得靠升精度通过原生容限。
不得靠放宽容限隐藏误差。
原生参考通过不证明上游等价。
纯参考与诊断可以采用f64。
它们不得反馈产品物理结果。
原生ABI按上游映射转换。
不因本规则改变外部ABI。

开发预检须列出精度映射。
变更验收须覆盖NP01至NP04。
先冻结逐字段误差容限。
再比较冻结MJWarp静态结果。
测试不得启动Python。
批量测试须记录耗时与显存。
双方采用相同输入与计算精度。
缺少证据时报告精度未验收。
发现偏离时阻止对应等价验收。
修复前不得宣称完整精度达标。
当前f64路径仍属于待修缺口。
旧验收不能豁免此硬指标。
Normify不自动检查浮点类型。
审查者必须核对实际产品路径。
详细指标见[精度契约](docs/numerical-precision-contract.md)。

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
| `src/model/mocap.rs` | 检查mocap映射与静态几何 |
| `src/model/camlight.rs` | 检查相机光源拓扑与独立参数 |
| `src/model/fixed_tendon.rs` | 检查固定肌腱与原生CSR |
| `src/model/flex.rs` | 检查位置、边、稀疏行与面映射 |
| `src/model/flex_hessian.rs` | 检查21系数单元与边映射 |
| `src/model/sleep.rs` | 检查树循环与限位参数 |
| `src/model/spatial_tendon.rs` | 检查路径、球柱尺寸与侧向site |
| `src/model/tendon.rs` | 检查混合字段与全局肌腱编号 |
| `src/io/` | 转换、GPU字段交换与只读字段组 |
| `src/io/fields.rs` | 提供独立连续字段底层 |
| `src/io/g01.rs` | 转换完整G01模型读字段 |
| `src/io/flex.rs` | 复制并转换原生柔体位置字段 |
| `src/runtime/` | 实现驱动级GPU探针 |
| `src/physics/` | 历史校验与刚体GPU探针 |
| `src/physics/attached.rs` | 复用刚体设备结果计算附着位姿 |
| `src/physics/resident.rs` | 常驻设备子集与全局合并 |
| `src/physics/com_velocity.rs` | 常驻GPU空间速度与导数 |
| `src/physics/rne_bias.rs` | 常驻GPU重力与广义偏置力 |
| `src/physics/sleep.rs` | 肌腱唤醒与轻量树刷新 |
| `src/physics/camlight.rs` | 相机光源两段GPU位姿计算 |
| `src/physics/fixed_tendon.rs` | 固定肌腱长度与稀疏力臂 |
| `src/physics/flex.rs` | GPU计算柔体节点与顶点位置 |
| `src/physics/flex_edge.rs` | GPU计算边长、雅可比与速度 |
| `src/physics/flex_face.rs` | GPU计算线性壳体面位姿 |
| `src/physics/flex_hessian.rs` | GPU缓存拉伸投影矩阵 |
| `src/physics/spatial_tendon.rs` | 路径长度、力臂与动态包裹字段 |
| `src/physics/tendon.rs` | GPU合并原生顺序的肌腱结果 |
| `src/physics/tendon_wrap.rs` | 冻结球柱与内侧绕行公式 |
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
| `fixtures/mocap-kinematics/` | 保留mocap与静态几何参考 |
| `fixtures/camlight/` | 保留相机光源原生参考 |
| `fixtures/fixed-tendon/` | 保留固定肌腱原生参考 |
| `fixtures/flex-position/` | 保留柔体位置原生参考 |
| `fixtures/flex-edge/` | 保留柔体边与原生稀疏行 |
| `fixtures/flex-face/` | 保留原生节点与独立极分解参考 |
| `fixtures/flex-stretch/` | 保留原生几何与稠密导数参考 |
| `fixtures/spatial-tendon/` | 保留site与滑轮原生参考 |
| `fixtures/geom-tendon/` | 保留球柱与内侧原生参考 |
| `fixtures/mixed-tendon/` | 保留混合编号与全局原生参考 |
| `fixtures/com-position/` | 保留质心参考与零质量边界 |
| `fixtures/mass-matrix/` | 保留矩阵参考与armature链 |
| `fixtures/mass-solve/` | 保留原生因子、右端项与解 |
| `fixtures/g01/` | 保留完整链原生参考与摘要 |
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

原生输入探针支持两种工具链。
Windows采用x64 MSVC。
Linux采用x86_64 GNU。
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
G01验收需要可信原生动态库。
它也需要NVIDIA驱动与NVRTC。
完整Windows验收执行以下命令：

```text
scripts/test-windows-g01.ps1 -AllFeatures -FullRegression
scripts/test-windows-g01.ps1 -AllFeatures -FullRegression -Release
```

证据见G01全阶段报告。
Linux完整验收执行以下命令：

```text
bash scripts/prepare-linux-native.sh
bash scripts/prepare-linux-cuda.sh
bash scripts/test-linux-g01.sh --all-features --full-regression
bash scripts/test-linux-g01.sh --all-features --full-regression --release
```

Linux证据见对应验收报告。
T4复验另加`--require-t4`。
该选项拒绝其他GPU。
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

## Normify伴随开发

本项目采用Normify伴随开发。
编码必须使用`$normify-dev`。
先确认MCP连接与现有架构。
沿用以下项目定位：

```text
project: mjwarp-rs
dir: /home/zkbot/.codex/normify/normify-mjwarp-rs
repoRoot: /home/zkbot/work/Rust/mjwarp-rs
```

保留原树与模块UID。
不要另建替代架构。
不要关闭无关历史变更。
先读取相关模块与规则。
写代码前完成设计预检。
登记变更与验收条件。
开发中同步模块与源码证据。
只激活实际落地的模块。
文档变更保留模块状态。
先运行目标相关测试。
再校验并关闭本次变更。
如实报告错误、警告与未测项。
架构校验不能替代产品验收。

| 阶段 | 工具 |
| --- | --- |
| 指引与预检 | `normify_brief`、`normify_check` |
| 登记变更 | `normify_change_open` |
| 同步与刷新 | `normify_sync`、`normify_module_refresh` |
| 校验与收尾 | `normify_validate`、`normify_change_close` |

## 项目进展汇报

进展汇报先取Normify简报。
必要时读取模块与变更状态。
仅补查Git与最新验收证据。
先给结论，再列进展与缺口。
区分架构记录与本次实测。
注明历史证据与未核验项。
简报不触发开发变更。
简报不触发全库文档扫描。
简报不触发全量测试。
用户要求审计时再深入。
发现矛盾时仅查相关范围。
工具不可用时说明原因。
再做最小范围只读核查。

## 代理协作

用简明汉语解释。
每句不超过20字。
只用主动语态。
源码分析先查现有图。
使用以下查询命令：

```text
codegraph explore "<符号或问题>"
codegraph node <符号或文件>
```
