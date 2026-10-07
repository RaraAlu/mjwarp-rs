# Windows空间肌腱子集

日期：2026-10-07。
范围：G01常驻辅助增量。
本批不关闭完整G01或G06。
本批不新增第三方依赖。
产品链不引入Python。

## 实际接口

```text
model::SpatialTendonFields
model::SpatialTendonModelInput
model::SpatialTendonRows
physics::KinematicsPlan::with_spatial_tendons
physics::KinematicsPlan::update_spatial_tendons
physics::KinematicsPlan::spatial_tendon_rows
physics::KinematicsData::spatial_tendon_rows
physics::KinematicsSnapshot::spatial_tendon
physics::SpatialTendonOutput
physics::SpatialTendonWorld
```

调用方提供已编译模型字段。
新构造器组合固定肌腱输入。
旧构造器采用空空间集合。
常驻链仍支持23项独立参数。
肌腱拓扑与滑轮除数仍共享。
模型与稀疏布局保持只读。
接口不编译MJCF。
接口不提供CPU后端。

## 输入检查

| 字段 | 本子集规则 |
| --- | --- |
| `tendon_adr`、`tendon_num` | 连续分割全部路径记录 |
| `wrap_type` | 只接受SITE=3与PULLEY=2 |
| `wrap_objid` | SITE引用有效site；PULLEY忽略该项 |
| `wrap_prm` | 全部有限；PULLEY使用正除数 |
| `ten_j_rowadr`、`ten_j_rownnz` | 连续分割全部CSR槽位 |
| `ten_j_colind` | 行内严格升序，引用有效DOF |

构造器先检查完整打包容量。
它再检查长度、分区与引用。
除数倒数必须保持有限。
每个分支至少含两个site。
路径允许前置滑轮。
路径拒绝相邻或末尾滑轮。
路径允许同体与重合site。
构造器检查两端祖先DOF。
CSR必须包含非共享祖先列。
CSR可以省略共享祖先列。
额外合法列保留零或实际累加。
几何包裹类型触发明确错误。

两类肌腱各用局部编号。
两类集合各从零开始。
各自包裹与CSR也从零开始。
接口不保留混排全局编号。
调用方不得直接混排原生字段。
完整导入仍需统一映射。

## GPU计算与布局

```text
rigid -> attached -> com -> camlight -> fixed_tendon -> spatial_tendon
spatial_site -> spatial_moment -> spatial_wrap
```

常驻链包含六个设备子集。
设备结果使用七个缓冲。
空间子集独占两种结果缓冲。
前两段计算f32结果。
最后一段直接写入i32结果。
整数段沿用内部同步内核ABI。
接口不把整数编码为浮点。

site段读取附着设备结果。
它复制包裹点并累加长度。
moment段读取质心设备结果。
它沿体祖先链累加稀疏力臂。
wrap段更新索引与对象类型。
阶段间不回读或重传中间结果。
更新不分配、上传模型或编译。
内核逐世界串行累加。
调度等待每段真实完成。

```text
scale = 1 / latest_pulley_divisor
segment_length = norm(site1 - site0) * scale
offset = site_position - subtree_com[body_root]
motion = cdof_linear + cross(cdof_angular, offset)
moment = dot(motion, segment_direction) * signed_scale
float_stride = ntendon + nnz + 6 * nwrap
integer_stride = 2 * ntendon + 2 * nwrap
```

滑轮分隔互不相连的分支。
后续滑轮覆盖当前比例。
同体段只计算长度。
短于`1e-15`的段采用X方向。
每次更新清空全部力臂槽位。
每次更新清空包裹尾部容量。
SITE对象值为`-1`。
PULLEY对象值为`-2`。
滑轮位置保持零。
包裹数组保留原生双点容量。
有效点数等于本子集记录数。

## 状态与失败边界

新计划检查两种输出容量。
新Data创建时检查全部布局。
空空间集合跳过内核编译。
空集合仍保留非空哨兵缓冲。
附着或质心更新使空间结果失效。
qpos或mocap写入使整链失效。
相机及固定肌腱不影响空间结果。
跨计划状态触发身份错误。
未就绪结果拒绝发布快照。
回读检查七个结果缓冲。
它检查两种空间缓冲哨兵。
它拒绝非有限浮点载荷。
失败不发布部分宿主快照。
既有快照保留原有值。
无效写入不破坏既有就绪状态。
接口不静默回退CPU。

## 参考与测试

样本位于`fixtures/spatial-tendon/`。
开发者用原生C++工具生成参考。
产品测试只读取固定JSON。
清单锁定XML、MJB、JSON与源码。
原生版本固定为3.12.0。
种子固定为1789。
八组输入均先转为f32。
模型含五条空间肌腱。
CSR包含45个槽位。
路径包含19个包裹记录。

原生比较覆盖2084个世界。
参数比较覆盖1042个世界。
世界数分别为1、2、5与513。
参数测试覆盖周期3、2与5。
测试复用GPU site与COM结果。
独立f64公式检查参数组合。
原生静态参考另作独立比较。
零质量、同体与祖先共享均有测试。
有限差分另检查三个标量关节。
直接结果保持绝对与相对容限。
两种容限均固定为`2e-5`。
差分检查使用`0.001`扰动。
差分容限采用`0.003`。
该容限只覆盖差分截断误差。
它不放宽直接物理结果容限。
整数结果要求精确一致。

GPU边界测试覆盖两种哨兵。
它注入NaN与正负无穷。
它检查重复更新清空尾部。
它检查重合点的方向回退。
它注入有限大site坐标。
该坐标只隔离长度运算溢出。
它不代表合法状态端到端参考。
恢复测试重新更新全部阶段。
组合测试同时启用六个子集。
它检查两类肌腱编号互不别名。

## 验收记录

| 环境 | 实际值 |
| --- | --- |
| 平台 | Windows x86_64 MSVC |
| Windows | 10.0.26200.0 |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER |
| 驱动 | 596.36 |
| Rust | 1.99.0 |
| CUDA工具链 | 12.8.1 |

以下命令全部实际执行。
宿主命令不启动忽略项。
显式命令启用全部忽略项。

```powershell
cargo test --locked --all-targets
cargo test --locked --doc
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --all-targets
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --all-targets --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --doc
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --doc --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --all-targets -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --all-targets --release -- --ignored --test-threads=1
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --features cuda-probe --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets --release -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
cargo fmt --check
git diff --check
```

| 检查 | 通过 | 失败 | 忽略 |
| --- | ---: | ---: | ---: |
| GNU默认宿主 | 169 | 0 | 0 |
| GNU默认文档测试 | 10 | 0 | 0 |
| MSVC调试宿主 | 207 | 0 | 112 |
| MSVC发布宿主 | 207 | 0 | 112 |
| MSVC调试文档测试 | 10 | 0 | 0 |
| MSVC发布文档测试 | 10 | 0 | 0 |
| MSVC调试显式探针 | 112 | 0 | 0 |
| MSVC发布显式探针 | 112 | 0 | 0 |
| 常驻调试脚本 | 41 | 0 | 0 |
| 常驻发布脚本 | 41 | 0 | 0 |

112项包含GPU与原生探针。
常驻每轮含5项宿主测试。
常驻每轮含36项GPU测试。
本批新增6项宿主测试。
本批新增7项GPU测试。
本批新增1项文档测试。
新文档测试检查只读借用。
四路Clippy均通过。
文档构建与格式检查均通过。
初轮Clippy发现三处风格问题。
我改用定长分块与错误断言。
完整复验随后全部通过。

两种构建的误差保持一致。
以下数字只统计空间肌腱比较。

| 比较 | 每轮世界数 | 最大绝对误差 |
| --- | ---: | ---: |
| 静态原生参考 | 2084 | `6.667048912945006e-7` |
| 独立参数公式 | 1042 | `7.262504873040143e-7` |

绝对与相对容限均为`2e-5`。
本批没有放宽直接比较容限。
整数输出全部精确匹配。

完整常驻证据位于：

```text
target/resident-kinematics/c59fd881af354cb88f897ed396848b61/
target/resident-kinematics/81a17d23fa884a8da9027c5c00ab2bd8/
```

第一份记录调试测试。
第二份记录发布测试。
两份均记录全部旧子集回归。
两份均记录六子集非空组合。
汇总与全量日志位于：

```text
target/spatial-tendon-verification/report.json
target/spatial-tendon-verification/*.log
```

证据记录基线`e76191d`。
测试当时保留本批未提交改动。
记录不冒充清洁提交验收。
Normify只激活两个辅助叶子。
完整产品阶段继续保持计划态。
架构校验返回零错误。
收尾保留旧历史变更。

## 当前限制与后续

本批仍只覆盖严格辅助子集。
球柱绕行与侧向site仍待实现。
完整肌腱编号与导入仍待统一。
肌腱速度与限位仍待实现。
flex位置与休眠副作用仍待补齐。
完整G01仍需双平台验收。
本轮不执行Linux或清洁部署。

下一步先补齐球柱绕行。
随后统一肌腱全局编号。
随后补齐flex与休眠分支。

公式对齐冻结上游。
[冻结肌腱源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。
[冻结归一化源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py)。
