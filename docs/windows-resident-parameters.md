# Windows运动学字段批量

日期：2026-10-07。
范围：G01常驻严格子集。
本批不关闭完整G01。
本批不添加第三方依赖。

## 实际接口

`ParameterBatch`拥有连续参数。
它检查正批量、容量与有限性。
它也支持零宽度字段。
`KinematicsParameters`记录覆盖。
缺省字段使用原模型共享行。
`KinematicsPlan::with_parameters`
先检查全部行，再上传设备。
计划创建后不允许改写参数。
默认构建仍不需要GPU工具链。

源码位于：

- `src/model/parameters.rs`
- `src/physics/resident.rs`
- `src/physics/mod.rs`
- `src/physics/attached.rs`

当前支持十三项浮点参数。

以下函数展示两行位置覆盖。
调用方须提供规范世界体行。
计划创建后可以创建多组状态。

```rust
use mjwarp_rs::{
    diagnostics::TransferError,
    model::{AttachedModelInput, KinematicsParameter, KinematicsParameters, ParameterBatch},
    physics::KinematicsPlan,
    runtime::TransferSession,
};

fn plan_with_positions(
    session: &TransferSession,
    model: AttachedModelInput,
    positions: Vec<f32>,
) -> Result<KinematicsPlan, TransferError> {
    let row_elements = model.rigid().kinematics().nbody() * 3;
    let mut parameters = KinematicsParameters::default();
    parameters.set(
        KinematicsParameter::BodyPos,
        ParameterBatch::new(2, row_elements, positions)?,
    );
    KinematicsPlan::with_parameters(session, model, parameters)
}
```

| 阶段 | 字段 |
| --- | --- |
| 刚体 | `qpos0`、`body_pos`、`body_quat`、`body_ipos`、`body_iquat`、`jnt_pos`、`jnt_axis` |
| 质心 | `body_mass`、`body_inertia` |
| 附着 | `geom_pos`、`geom_quat`、`site_pos`、`site_quat` |

各字段独立采用`world % B_f`。
`B_f=1`表示共享参数。
`B_f=W`表示逐世界参数。
容量内的正周期保持合法。
周期无需整除世界数。
周期也可以超过世界数。
默认qpos采用对应的qpos0行。
关节计算也使用该字段周期。

我们核对[冻结上游](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。
当前只对齐字段取模语义。
完整阶段仍需要独立验收。

## 校验与设备所有权

模型拓扑仍只共享一份。
基础模型仍须通过严格校验。
覆盖字段不修复非法基础模型。
各字段行宽须匹配模型尺寸。
每行都须通过现有物理限制。
校验不跳过当前未使用的行。
世界体保留规范位置与姿态。
世界体质量与惯量保持零。
旋转与关节轴保留单位限制。
默认状态保留四元数范围限制。
质量与惯量不能出现负值。
错误索引包含批量行偏移。
结果继续采用固定误差容限。

三个设备包保存字段偏移与周期。
打包容量保持有符号32位上限。
批量上限也采用`i32::MAX`。
指针运算采用64位偏移。
空附着字段保留哑参数缓冲。
打包不按世界数复制参数。
打包也不计算周期最小公倍数。
更新继续复用设备模型与内核。
更新不上传模型或重新编译。
阶段间继续直接读取设备结果。
显式回读检查守卫与有限性。
模型身份隔离与状态寿命不变。
旧探针继续采用共享设备布局。
旧f64质量求解路径保持不变。

## 测试与证据

新增测试使用固定模型与周期。
产品测试不启动Python。
产品测试不生成原生参考。
产品测试不调用模型编译器。

宿主测试覆盖全部十三项尺寸。
它们检查后续行的物理错误。
它们检查零周期与容量溢出。
它们检查非有限值与负参数。
它们检查打包总容量与描述符。

GPU测试采用三类独立证据。

| 测试 | 证据 | 世界比较数 |
| --- | --- | --- |
| 混合周期 | 轴置换解析解，覆盖十五项输出 | 1563 |
| qpos0周期 | 固定原生四类关节参考 | 1026 |
| 空字段长周期 | 固定零自由度原生参考 | 513 |

解析测试使用1、2、5、513世界。
各世界数均执行三次状态更新。
最后一轮只改写末尾世界。
它们跨越两个完整CUDA块。
十三项周期依次采用：

```text
3, W, 3, 5, 4, 7, 2, 3, 5, 2, 4, 7, 3
```

空字段采用`B_f=2147483647`。
测试不按该周期申请空参数。
解析参考只采用已知轴置换。
它不复制GPU四元数公式。
原生参考只读取冻结静态样本。
绝对与相对容限均为`2e-5`。
脚本同时复核三个参考清单。
脚本要求3102次批量比较。
脚本保留1245次旧底座比较。
脚本要求真实非零用例数量。

执行以下命令：

```text
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
```

## 实际验证结果

本轮使用以下环境：

| 项目 | 实际值 |
| --- | --- |
| 平台 | Windows x86_64 MSVC |
| Windows | `10.0.26200.0` |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER |
| 驱动 | `596.36` |
| Rust | `1.99.0` |
| NVRTC | CUDA `12.8.1`探针工具链 |
| 默认工具链 | Windows GNU |

我们执行并通过以下检查。

| 检查 | 真实结果 |
| --- | --- |
| 默认GNU测试 | 154项宿主、7项文档通过 |
| MSVC全feature调试测试 | 184项宿主、7项文档通过；88项GPU与原生测试默认忽略 |
| MSVC全feature发布测试 | 184项宿主、7项文档通过；88项GPU与原生测试默认忽略 |
| 调试显式忽略测试 | 88项通过，0失败，0忽略 |
| 发布显式忽略测试 | 88项通过，0失败，0忽略 |
| 底座脚本调试与发布 | 各通过1项清单、6项底座、3项批量、3项内核ABI测试 |
| Clippy四路检查 | 默认GNU、CUDA GNU、全feature MSVC调试与发布均通过 |
| 格式与差异检查 | `cargo fmt --check`、`git diff --check`通过 |
| 全feature文档构建 | `cargo doc --locked --all-features --no-deps`通过 |

显式全量命令如下：

```text
cargo test --locked
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features --release -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
```

原生准备脚本另通过22项测试。
它只编译参考接口mock。
它不生成物理参考或编译模型。
我们显式保留MSVC的`PATH`。
我们排除小写`Path`重复覆盖。
本批不修改旧原生准备脚本。

调试与发布采用相同容限。
两路各完成3102次批量比较。
批量最大绝对误差为`1.144410e-5`。
两路各完成1245次旧底座比较。
旧底座最大误差为`4.158171e-6`。
我们没有放宽固定容限。

调试证据目录：

```text
target/resident-kinematics/a5108528a15a4dcbac90f95eab0d5a4b/
```

发布证据目录：

```text
target/resident-kinematics/8881d556c1f049a186e4d6525b0b66c8/
```

两目录保留日志与`report.json`。
报告记录驱动、容限与库哈希。
报告也记录Git版本与脏状态。
全量日志目录如下：

```text
target/resident-kinematics/verification-field-batches/
```

本轮只取得Windows本机证据。
我们未执行Linux与清洁部署。

## 架构与提交

我们已收尾字段批量变更。
最终检查确认0错误、0警告。
结构新增批量参数辅助叶子。
完整Model与Data仍保持计划。
完整G01也仍保持计划。
收尾曾报告并行变更警告。
最终复查确认该警告消失。
旧运动学字段变更继续开放。
本批不关闭无关历史变更。

### 首次自动提交尝试

我们当时尝试自动智能提交。
沙箱当时拒绝Git索引写入。
我们当时未创建新提交。
工作树当时保留两批修改。
后续提交请查Git历史。
尝试时HEAD为：

```text
5e2e2cffb0cd811432aedaa780c0b8716c7d7f0b
```

## 剩余范围

当前仍不处理mocap。
当前仍重算全部geom与site。
静态geom初始化缓存仍待补齐。
模型参数变更与刷新仍待补齐。
相机、光源与等价阶段仍待实现。
休眠、柔性体与腱分支仍待实现。
Linux与清洁部署验收仍待执行。
完整G01继续保持计划状态。

下一批先补mocap与静态语义。
随后补相机与光源。
