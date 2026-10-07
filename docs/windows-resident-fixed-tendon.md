# Windows固定肌腱子集

日期：2026-10-07。
范围：G01常驻辅助增量。
本批不关闭完整G01或G06。
本批不新增第三方依赖。
产品链不引入Python。

本报告保留该批历史计数。
常驻链现含六个设备子集。
设备结果现用七个缓冲。
空间site增量见[历史报告](windows-resident-spatial-tendon.md)。
球柱增量见[最新报告](windows-resident-geom-tendon.md)。

## 实际接口

```text
model::FixedTendonFields
model::FixedTendonModelInput
model::FixedTendonRows
physics::KinematicsPlan::with_fixed_tendons
physics::KinematicsPlan::update_fixed_tendons
physics::KinematicsPlan::fixed_tendon_rows
physics::KinematicsData::fixed_tendon_rows
physics::KinematicsSnapshot::fixed_tendon
physics::FixedTendonOutput
physics::FixedTendonWorld
```

调用方提供已编译模型字段。
构造器组合相机光源输入。
旧构造器默认空肌腱集合。
常驻链仍支持23项独立参数。
肌腱系数暂时共享一份。
模型与稀疏布局保持只读。
接口不编译MJCF。
接口不提供CPU后端。

## 输入与稀疏布局

| 字段 | 长度与含义 |
| --- | --- |
| `tendon_adr`、`tendon_num` | 各含`ntendon`项；连续分割关节项 |
| `wrap_type` | 含`nwrap`项；仅接受原生JOINT值1 |
| `wrap_objid` | 含`nwrap`项；引用hinge或slide |
| `wrap_prm` | 含`nwrap`项；共享有限系数 |
| `ten_j_rowadr`、`ten_j_rownnz` | 各含`ntendon`项；连续分割CSR |
| `ten_j_colind` | 含`nJten`项；保存原生DOF编号 |

Rust字段将原生J改为小写。
每行稀疏列严格升序且唯一。
每行必须覆盖引用关节的DOF。
额外合法列保持零力臂。
地址与计数必须覆盖全部元素。
构造器拒绝空的非零肌腱。
空肌腱集合允许零元素。
模型可以含free或ball关节。
肌腱不能引用这些关节。
同腱不能重复引用同一关节。
不同肌腱可以引用同一关节。
系数允许正数、负数与零。
空间项与滑轮项仍不支持。
输入检查仍属严格辅助限制。

构造器先检查全部宿主字段。
它再派生qpos地址与CSR槽位。
它不改变原生稀疏列编号。
它先检查整数包与输出容量。
GPU申请发生在这些检查之后。

每个世界输出两段f32数据。
第一段包含`ntendon`个长度。
第二段包含`nJten`个力臂。
世界视图不重建稠密矩阵。
输出首尾各保留四个守卫。

## 冻结计算语义

```text
ten_length[t] = sum(wrap_prm[i] * qpos[jnt_qposadr[joint[i]]])
ten_J[CSR_slot(t, jnt_dofadr[joint[i]])] = wrap_prm[i]
```

长度直接使用当前qpos。
内核不减去qpos0或关节ref。
内核每次重写全部长度。
内核先清零全部稀疏力臂。
它再写入每个关节系数。
零系数仍占据原生稀疏列。
qpos地址不等于DOF编号。
混合关节参考明确检查此差异。

冻结上游并行累加长度。
本子集每线程独占一个世界。
本子集按关节项顺序累加。
它不承诺逐位相同的加法顺序。
它不放宽已有数值容限。
同腱重复项会引发上游写竞争。
本严格子集直接拒绝重复项。

依据：[冻结肌腱源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。

本入口只更新长度与稀疏力臂。
它不输出包裹点或包裹编号。
它不伪造包裹字段的清零。
肌腱速度、限位与传动仍待实现。

## 常驻链与生命周期

辅助更新采用以下顺序：

```text
rigid -> attached -> com -> camlight -> fixed_tendon
```

固定肌腱只读取常驻qpos。
单独更新无需刚体或质心就绪。
完整回读仍要求其他阶段就绪。
刚体更新废弃全部派生就绪。
qpos与mocap写入也废弃就绪。
其他阶段不废弃肌腱就绪。
非法宿主输入保留已有结果。
跨计划状态返回身份错误。

计划只上传与编译一次。
空肌腱不上传或编译第五组。
状态创建申请五组输出。
空肌腱仍保留八个守卫。
更新不申请或上传设备缓冲。
更新不编译或回读中间结果。
同步内核沿用八参数ABI。
回读检查全部五组输出。
回读失败不发布部分快照。
非有限计算结果返回明确错误。
有限输入也可能引发数值溢出。
后续合法写入允许恢复计算。
计划释放不破坏已完成回读。
历史快照独立拥有宿主数据。

## 静态参考与边界

样本位于`fixtures/fixed-tendon/`。
样本包含6个体与5个关节。
它保留无关的free与ball。
肌腱只引用两个hinge与一个slide。
模型采用`nq=14`与`nv=12`。
四条肌腱共含八个关节项。
原生CSR也包含八个元素。
样本覆盖正负与零系数。
样本覆盖打乱的关节项顺序。
非零ref检查原始qpos语义。
固定种子为1789。
八组输入保留f32精度。
原生参考保留f64结果。
开发期C++作者工具另行执行。
产品测试只读取冻结参考。
测试逐项核对四个文件哈希。

世界数覆盖1、2、5、513。
原生比较包含默认状态与三次写入。
独立公式检查qpos0周期3。
有限差分交叉检查稀疏力臂。
其导数误差阈值为`0.001`。
此阈值只处理差分消减误差。
直接结果仍采用原有`2e-5`容限。

边界测试覆盖以下行为：

- 非法长度、分区与地址。
- 非法wrap种类与关节引用。
- 同腱重复引用与非有限系数。
- 缺失、重复与越界稀疏列。
- 额外稀疏列每次保持零值。
- 五组非空输出的组合更新。
- 首尾世界与跨块世界写入。
- 多状态、跨计划与历史快照。
- 就绪、失效与计划释放。
- 空肌腱与零自由度模型。
- 两项输出的非有限值拒绝。
- 首尾四标量守卫的拒绝。
- 真实f32乘法溢出与恢复。

## 验证记录

平台：Windows x86_64 MSVC。
系统：10.0.26200.0。
GPU：RTX 4070 Ti SUPER。
驱动：596.36。
Rust：1.99.0。
工具链：CUDA 12.8.1。
原生参考候选：MuJoCo 3.12.0。
正式内核路线仍待冻结。

实际执行以下命令：

```powershell
cargo test --locked
cargo fmt --check
git diff --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --features cuda-probe --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets --release -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
```

原生回归先预备DLL与环境。
本批复用已有原生mock产物。
产品测试不生成原生参考。
常驻脚本检查参考与DLL哈希。
常驻脚本不调用Python。

| 验证路线 | 通过 | 失败 | 默认忽略 |
| --- | --- | --- | --- |
| GNU默认宿主 | 166 | 0 | 0 |
| GNU文档测试 | 9 | 0 | 0 |
| MSVC调试宿主 | 201 | 0 | 105 |
| MSVC发布宿主 | 201 | 0 | 105 |
| MSVC调试文档 | 9 | 0 | 0 |
| MSVC发布文档 | 9 | 0 | 0 |
| MSVC调试显式探针 | 105 | 0 | 0 |
| MSVC发布显式探针 | 105 | 0 | 0 |
| 常驻调试脚本 | 33 | 0 | 0 |
| 常驻发布脚本 | 33 | 0 | 0 |

105项包含GPU与原生探针。
它们不全是GPU物理用例。
常驻脚本包含4项宿主检查。
它还包含29项GPU测试。
固定肌腱新增6项GPU测试。
其中4项检查集成行为。
另2项检查守卫与溢出。
后续组合补测也通过两路脚本。
四条Clippy路线均通过。
MSVC文档构建也通过。
格式与差异检查均通过。
本批改动文档的本地链接有效。
此链接检查不替代全仓审计。

两路各比较2084个世界。
最大绝对误差均为：

```text
5.066394805908203e-7
```

绝对与相对容限均为`2e-5`。
本批没有放宽直接比较容限。

常驻完整证据位于：

```text
target/resident-kinematics/77744edb455f48cd9d1cdfa1d46cfbba/
target/resident-kinematics/a1dfbccbc7ab4044af15160b5fa90ee4/
```

第一份记录调试测试。
第二份记录发布测试。
两份均记录完整旧子集回归。
两份均记录五组非空组合。
汇总与全量日志位于：

```text
target/fixed-tendon-verification/report.json
target/fixed-tendon-verification/*.log
```

证据记录基线`0bea987d`。
测试当时保留本批未提交改动。
记录不冒充清洁提交验收。
Normify只激活两个辅助叶子。
正式物理继续保持计划态。
架构校验返回零错误。
收尾保留旧历史变更。

## 未完成项

该批尚未实现空间肌腱。
后续子集已支持site与pulley。
后续子集已支持球柱绕行。
该批尚未实现包裹字段。
后续子集已更新包裹字段。
flex位置与休眠副作用仍待实现。
完整Model、Data与导入仍待实现。
完整G01仍需双平台验收。
Linux与清洁部署仍未执行。
