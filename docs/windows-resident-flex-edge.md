# Windows常驻柔体边子集

日期：2026-10-08。
状态：G01辅助增量。
本报告不关闭完整G01。
本报告不关闭G22。

## 本轮范围

GPU计算全部柔体边长。
GPU计算稀疏边雅可比。
GPU计算雅可比与qvel点积。
常驻链直接复用设备顶点。
常驻链直接复用设备质心。
阶段之间不回读宿主结果。
本轮没有新增运行时依赖。

本子集支持直接柔体边。
原生插值空行提供零速度。
负体编号要求空稀疏行。
本轮不计算弹性与被动力。
本轮不计算柔体面姿态。
本轮不实现碰撞与积分。

## 输入与接口

`FlexEdgeFields`保存六项字段：

```text
flex_edgeadr
flex_edgenum
flex_edge
flexedge_j_rowadr
flexedge_j_rownnz
flexedge_j_colind
```

边端点使用柔体局部顶点号。
稀疏列使用全局自由度号。
输入检查连续边分区。
输入拒绝越界与负计数。
活跃稀疏行要求连续且不重叠。
每行要求严格递增的列。
非空行必须匹配端点祖先并集。
空行保留上游跳过语义。
原生稀疏池允许预留尾槽。
GPU每轮将预留槽清零。

启用方式如下：

```rust,ignore
let model = FlexPositionModelInput::new(tendons, positions)?
    .with_edges(edges)?;
let plan = KinematicsPlan::with_flex_positions(
    &session, model, parameters, camlight_parameters,
)?;
let mut data = plan.create_data(worlds)?;
data.write_world_qvel(world, &qvel)?;
plan.update(&mut data)?;
let snapshot = data.readback()?;
let output = snapshot.flex_edges().unwrap();
let fields = output.fields();
let world_output = output.world(world)?;
```

旧位置入口仍返回`None`。
可选空边入口返回空视图。
计划与状态提供`flex_edge_fields()`。
快照独立拥有结果与CSR字段。
后续写入不会修改既有快照。
状态保留设备会话所有者。
计划释放后仍允许显式回读。

## 状态与执行顺序

边入口另加两个f32缓冲。
一个缓冲保存多世界qvel。
另一个缓冲保存三类结果。
输入和结果各保留首尾守卫。
qvel初始值全部为零。
qvel采用世界连续布局。
它不采用参数批量取模。
边拓扑与CSR仍共享一份。

| 写入或更新 | 失效结果 |
| --- | --- |
| qpos、mocap或刚体 | 全部派生结果 |
| qvel | 仅柔体边 |
| 质心 | 柔体边及原有依赖项 |
| 柔体位置 | 柔体边 |
| 肌腱唤醒状态 | 仅唤醒副作用 |

合法qvel写入先检查全部输入。
错误写入保留原有就绪状态。
边阶段要求质心与位置就绪。
空边入口不增加物理依赖。
跨计划更新先拒绝模型身份。

完整辅助更新顺序如下：

```text
刚体 → 附着 → 质心 → 相机光源
→ 柔体位置 → 柔体边 → 肌腱
→ 可选肌腱唤醒
```

新内核采用私有三输入ABI。
运行时检查五个缓冲的会话。
同步完成事件保护设备借用。
单线程独占一个世界。
它使用公开f32输入与结果。
它也使用GPU f32中间量。
它禁止快速数学与融合乘加。
零长度方向沿用冻结零分支。

## 原生参考

样本目录：`fixtures/flex-edge/`。
来源与命令见[样本说明](../fixtures/flex-edge/README.md)。
清单固定五个参考文件哈希。
其中三项来自基础柔体样本。
作者程序使用MuJoCo 3.12.0。
它使用原生边雅可比。
它使用原生顶点测量插值边长。
它保留原生跳过边长的原值。
它保留原生未启用边力的CSR。
产品测试只读取静态文件。
产品测试不启动参考作者程序。

参考含284条边与十个稀疏槽。
六条直接边提供活跃稀疏行。
278条插值边提供长度。
十二组状态覆盖关节与mocap。
GPU测试组合四种世界数量。
世界数量为1、2、5、513。
每种数量执行四轮输入更新。
每轮再验证重复更新的位模式。

误差容限保持以下公式：

```text
abs(actual - native) <= 2e-5 + 2e-5 * abs(native)
```

本轮不放宽已有物理容限。
本轮核对2084组世界状态。
本轮核对1204552项边结果。
最大绝对误差约2.88e-7。
自由关节另通过解析测试。
零自由度静态边也通过测试。
退化测试覆盖重合与同体端点。
反向端点测试保持长度梯度。
守卫测试覆盖两类缓冲。
失败测试覆盖外部会话与维度。
生命周期测试覆盖独立状态。
组合测试覆盖可选肌腱唤醒。

## 验证环境与命令

平台：Windows x86_64。
主验证工具链：Rust 1.99.0 MSVC。
默认工具链：Rust 1.99.0 GNU。
GPU：RTX 4070 Ti SUPER。
驱动：596.36。
NVRTC：CUDA 12.8.1。
本轮没有运行Linux验收。

```powershell
cargo test --locked
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1 --nocapture
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1 --nocapture
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
```

本轮记录以下最终结果：

| 验证路线 | 结果 |
| --- | --- |
| GNU默认测试 | 180项宿主与12项文档 |
| MSVC全特性调试测试 | 227项宿主与12项文档 |
| MSVC全特性发布测试 | 227项宿主与12项文档 |
| 全特性忽略探针，双模式 | 各140项GPU与原生探针 |
| 常驻验收脚本，双模式 | 各76项，含12项宿主与64项GPU |
| GNU发布边测试 | 4项GPU与2项守卫解析测试 |
| GNU与MSVC四路Clippy | 零警告 |
| rustfmt与文档构建 | 通过 |
| 范围内文档复核 | 12项源码与11项文档 |
| 本地链接与参考哈希 | 135条链接与5项哈希 |

首轮默认Clippy发现未读元数据。
我增加状态的只读CSR入口。
最终四路Clippy均通过。
范围内文档复核没有错误。
审计钩子提示七处路径。
人工核对确认七处均为误报。
上游路径不属于本仓库。
三处PTX引用实际存在。
三处链接库来自配置目录。
本轮不修改无关历史文档。

```powershell
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features --release -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
cargo fmt --check
git diff --check
```

证据目录：`target/flex-edge-verification/`。
最终汇总保存18条成功命令。
`final-results.json`记录这些命令。
早期失败日志保留诊断记录。
常驻脚本另保留独立报告目录。

## 剩余缺口

面位置与四元数仍待实现。
柔体Hessian缓存失效仍待实现。
高阶与壳插值仍待范围核对。
完整原生字段转换仍待实现。
完整阶段ABI与副作用仍待冻结。
完整G01仍需双平台验收。
G22动力学与G25休眠仍待实现。
下一步推进柔体面运动学。

[冻结柔体边源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L347-L414)。
[冻结归一化源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py#L260-L265)。
