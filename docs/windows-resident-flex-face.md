# Windows柔体面运动学子集

日期：2026-10-08。
范围：G01线性壳体面辅助。
完整G01仍需双平台验收。
完整G22与G25仍待实现。
本轮没有运行Linux验收。

## 实现范围

本轮新增线性壳体输入。
插值模式支持零、一、负一。
负一表示线性壳体。
高阶正负二仍返回错误。
GPU继续计算节点与顶点。
面阶段读取常驻节点结果。
它输出九点与四元数。
线性面只使用前四点。
末五点每轮严格写零。
本轮不实现弹性或碰撞。
本轮不实现柔体Hessian缓存。
本轮没有新增依赖。

## 输入校验

`FlexFaceFields`提供两项字段。
`flex_face_map`提供柔体与局部面号。
`flex_face`提供九项全局节点号。
每个线性壳体提供完整表面。
校验要求冻结上游顺序。
面数采用`2*(cy*cz+cx*cz+cx*cy)`。
六侧按x、y、z依次排列。
每轴先低侧，再高侧。
面内序号沿冻结格点顺序。
前四项对应同一格点四角。
末五项必须等于负一。
校验拒绝缺面与重复面。
校验拒绝跨柔体节点引用。
校验拒绝重排与非法哨兵。
尺寸计算先检查整数溢出。
位置入口仍检查正格点数量。
格点数量须匹配节点总数。
参数、拓扑与映射共享一份。

## API与状态

| API | 用途 |
| --- | --- |
| `FlexPositionModelInput::with_faces` | 检查并启用面子集 |
| `FlexPositionModelInput::faces` | 读取共享面字段 |
| `KinematicsPlan::update_flex_faces` | 更新面位姿 |
| `KinematicsPlan::flex_face_fields` | 读取计划面字段 |
| `KinematicsData::flex_face_fields` | 读取状态面字段 |
| `KinematicsSnapshot::flex_faces` | 读取可选面快照 |
| `FlexFaceOutput::world` | 读取单世界面视图 |
| `FlexFaceOutput::fields` | 读取保留的共享映射 |
| `TendonWakeModelInput::flex_faces` | 读取组合输入的面字段 |

旧入口返回`None`。
显式空入口提供空视图。
面入口无需启用边计算。
面入口无需qvel缓冲。
它也无需质心就绪。
非空面只要求位置就绪。
qpos与mocap写入废弃派生结果。
位置更新同时废弃边面结果。
COM更新不废弃面结果。
qvel写入也不废弃面结果。
输入错误保留已有结果。
跨计划操作返回身份错误。
同尺寸不等于同模型。

组合更新顺序如下。

```text
rigid → attached → com → camlight
→ flex_positions → flex_edges → flex_faces
→ tendons → optional tendon_wake
```

边、面与唤醒均采用可选入口。
唤醒包装保留已有边面字段。
旧入口不强制增加面缓冲。

## 布局与所有权

每世界输出包含`31*nflexface`个f32。
前段提供`27*nflexface`项位置。
后段提供`4*nflexface`项旋转。
四元数明确采用xyzw顺序。
其他刚体接口继续采用原顺序。
面子集只新增一个结果缓冲。
纯位置入口保留十个结果缓冲。
边另加qvel与边结果缓冲。
唤醒另加两个整数缓冲。
每个结果缓冲两端各设四项守卫。
空输出也保留八项守卫。
回读检查守卫与全部有限性。
算术溢出明确返回错误。
回读失败不发布部分快照。
快照独立拥有宿主结果。
快照通过Arc保留面映射。
状态也保留模型与设备资源。
删除计划后仍能回读状态。
同步内核等待结束才返回。
阶段之间不回读节点数据。

## 冻结公式与边界

面公式沿用冻结`smooth.py`。
切线使用线性基函数导数。
法线使用两切线的叉积。
三列矩阵顺序依赖法线轴。
面法线不采用低侧向外翻转。
旋转沿用冻结迭代极分解。
迭代从单位四元数开始。
它最多执行五十轮。
旋转更新阈值采用`1e-6`。
分母保护常数采用`1e-10`。
内部与公开输出均采用f32。
本轮不改动数值容限。
零矩阵保持单位四元数。
精确180度可能保持单位旋转。
冻结算法存在这个驻点限制。
本轮解析测试锁定该限制。
本轮不将它改成理想SVD结果。

壳体节点沿用冻结变换语义。
冻结节点阶段不执行内部TFI。
本辅助也不重建内部节点。
原生MuJoCo会执行内部TFI。
因此两者并非全范围等价。
解析测试明确保留独立内部节点。
完整等价入口还须核对此差异。

## 原生参考来源

参考复用已有柔体MJB。
作者工具只改插值符号。
两组线性格点均没有内部节点。
因此本参考不触发TFI差异。
原生版本采用MuJoCo3.12.0。
十二组状态先舍入至f32。
节点采用原生位置变换。
顶点直接采用`mj_flex`结果。
MuJoCo没有公开面位姿字段。
面参考先采集原生节点。
参考切线采用对边差分平均。
旋转参考采用独立对称特征分解。
作者工具调用原生`mju_eig3`。
它随后计算极分解旋转。
它不复制GPU迭代代码。
旋转比较允许四元数正负等价。
原生参考不覆盖奇异矩阵。
解析测试另覆盖零矩阵与驻点。
产品测试只读取静态JSON。
产品测试不调用作者工具。
产品链不引入Python。
详情见[参考说明](../fixtures/flex-face/README.md)。

## 验证记录

最终回归使用修复后的源码。
本机使用Windows x86_64。
GPU采用RTX 4070 Ti SUPER。
驱动版本采用596.36。
CUDA探针采用12.8.1。
Rust工具链采用1.99.0。

比较覆盖1、2、5、513世界。
每组执行四轮状态更新。
总共覆盖2084组状态。
面比较覆盖1033664项标量。
位置与旋转分别记录最大误差。
双模式最大位置误差为`3.453353754068189e-7`。
双模式最大旋转分量误差为`2.0164503808650647e-6`。
比较沿用`2e-5+2e-5*abs(reference)`。
重复更新要求逐位一致。
解析测试覆盖三轴与全部六侧。
解析测试覆盖旋转、剪切与内部点。
守卫测试覆盖首尾与非有限值。
算术测试覆盖有限输入溢出。
生命周期测试覆盖独立状态与快照。
组合测试覆盖边、面与肌腱唤醒。
首轮编译曾发现借用冲突。
我改用独立布局读取世界数。
首轮Clippy曾提示范围索引。
我改用迭代器与固定块接口。
最终验证只记录修复后的源码。

| 检查 | 实际结果 |
| --- | --- |
| GNU默认测试 | 182项宿主与12项文档 |
| MSVC全特性调试测试 | 231项宿主与12项文档 |
| MSVC全特性发布测试 | 231项宿主与12项文档 |
| 全特性忽略探针，双模式 | 各145项GPU与原生探针 |
| 常驻验收脚本，双模式 | 各83项，含14项宿主与69项GPU |
| GNU发布面测试 | 5项集成与2项GPU边界测试 |
| GNU与MSVC四路Clippy | 零警告 |
| rustfmt与文档构建 | 通过 |
| 范围内路径与契约检查 | 14项源码与13项文档 |
| 本地链接与参考哈希 | 144条链接与5项哈希 |

范围内路径与契约检查没有错误。
审计钩子提示八处路径。
人工核对确认八处均为误报。
一处路径来自冻结上游。
三处PTX引用实际存在。
四处链接库来自配置目录。
本轮不修改无关历史文档。
架构收尾检查没有错误或警告。

```powershell
cargo test --locked
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1 --nocapture
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1 --nocapture
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features --release -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
cargo fmt --check
git diff --check
```

证据目录：`target/flex-face-verification/`。
最终汇总保存十八条成功命令。
`final-results.json`记录真实计数。
常驻脚本另保留独立报告。

## 剩余缺口

下一步补齐Hessian失效语义。
随后收口完整阶段入口。
高阶柔体仍需单独实现。
完整原生输入转换仍待实现。
完整阶段ABI与副作用仍待冻结。
完整G01仍需双平台验收。
G22动力学与G25休眠仍待实现。

[冻结面公式](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。
[冻结面映射与极分解](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py)。
