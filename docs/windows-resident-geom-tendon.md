# Windows球柱绕行子集

日期：2026-10-07。
范围：G01常驻辅助增量。
本批不关闭完整G01或G06。
初轮运动中间帧出现三帧超限。
本轮修复T4后通过逐帧复验。
详见[逐帧原生复核](windows-geom-tendon-frame-review.md)。
本批不新增第三方依赖。
产品链不引入Python。

## 输入与接口

```text
model::SpatialTendonGeometry
model::SpatialTendonModelInput::with_geometry
model::SpatialTendonModelInput::geometry
physics::KinematicsPlan::with_spatial_tendons
physics::KinematicsPlan::update_spatial_tendons
physics::KinematicsSnapshot::spatial_tendon
```

调用方提供已编译模型字段。
新输入沿用既有空间集合。
旧构造器保留site与pulley行为。
新构造器另接收类型与尺寸。
`geom_size`采用原生三标量布局。
每行宽度等于`3*ngeom`。
尺寸按`world % B`独立取模。
既有二十三项参数保持独立。
尺寸另外提供一个独立周期。
绕行只读取尺寸首项半径。
圆柱沿Z轴采用无限柱面。
柱体半高不限制此绕行公式。

| 检查 | 严格子集规则 |
| --- | --- |
| `wrap_type` | SITE=3、PULLEY=2、SPHERE=4、CYLINDER=5 |
| `geom_type` | 原生类型0至8；球引用类型2，柱引用类型5 |
| `geom_size` | 有限、非负，行数正数，宽度匹配 |
| `wrap_objid` | site或geom引用有效对象 |
| `wrap_prm` | 球柱项四舍五入为侧向site；舍入后负值采用缺省 |
| 路径 | 球柱项前后必须紧邻site |
| CSR | 包含两端与geom非共享祖先DOF |
| 容量 | 检查元数据、参数及两种结果布局 |

侧向site只影响绕行选择。
例如`-0.49`引用site零。
其体DOF不直接写入力臂。
拓扑、侧向编号与滑轮比例仍共享。
零半径与极小半径保留输入。
GPU按冻结阈值跳过绕行。
固定与空间集合各用局部编号。
本批不统一原生混排全局编号。

## GPU机制与布局

```text
rigid -> attached -> com -> camlight -> fixed_tendon -> spatial_tendon
spatial_site -> spatial_moment -> spatial_wrap
float_stride = ntendon + nnz + 19 * nwrap
public_wrap_xpos = 6 * nwrap
private_scratch = 13 * nwrap
integer_stride = 2 * ntendon + 2 * nwrap
```

常驻链仍含六个设备子集。
设备结果仍使用七个缓冲。
浮点结果缓冲另容纳私有暂存。
公开位姿视图不暴露暂存。
每个记录暂存弧长与两处接点。
它另存两条内侧连接方向。
site段读取GPU几何与site位姿。
它计算球面平面与圆柱螺旋。
它保留内侧牛顿迭代规则。
内侧投影与迭代采用GPU f64。
方向计算先于世界接点舍入。
方向缓存与公开结果仍为f32。
外侧绕行保留原f32公式。
它按滑轮比例累加总长度。
moment段读取GPU质心与接点。
它累加两条连接段的稀疏力臂。
无绕行时它改用直接连接段。
wrap段直接读取浮点暂存。
它写入独立i32结果缓冲。
内部unsafe ABI分离三种类型。
调用方证明指针类型与访问范围。
接口不把整数编码为浮点。
调度同步等待每段完成。
更新不回读或重传中间结果。
更新不分配、上传模型或编译。

有效geom写入两处接点与geom编号。
无绕行geom不占有效点槽位。
SITE对象值为`-1`。
PULLEY对象值为`-2`。
滑轮位置保持零。
GPU逐世界计算动态数量与地址。
每轮清空两种载荷与全部尾部。
回读也检查私有暂存有限性。
失败不发布部分宿主快照。
旧快照保持独立所有权。
无效状态写入不破坏已有就绪结果。

## 测试与参考

静态参考位于`fixtures/geom-tendon/`。
开发者独立运行原生C++工具。
产品测试只读取哈希锁定文件。
清单固定MuJoCo 3.12.0与DLL。
十二组状态先舍入为f32。
样本标识为`2099`。
样本不使用随机生成器。
模型含十条肌腱与十个CSR槽位。
路径共含三十七个包裹记录。
参考覆盖内外侧球柱与长弧。
它覆盖运动geom、mocap与滑轮。
它覆盖有绕行与无绕行切换。

原生比较覆盖2084个世界。
圆柱解析比较覆盖1042个世界。
世界数采用1、2、5与513。
半径周期采用3。
解析位置、旋转周期采用2与5。
解析site位置周期采用7。
解析测试采用零DOF模型。
球面退化测试另检查七种分支。
它覆盖平行、重合、内含与极小半径。
切线边界保留冻结f32分支。
接点叉积舍入可选中整圆。
该边界不声明原生f64等价。
直接结果采用绝对与相对`2e-5`。
整数结果要求精确一致。
有限差分另检查hinge与slide。
它采用`0.001`扰动与`0.003`容限。
差分容限不放宽直接物理比较。
生命周期测试检查就绪与身份。
它检查缩减、扩展与快照独立性。
旧子集回归继续检查七缓冲哨兵。

## 首批验收记录

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
| MSVC调试宿主 | 209 | 0 | 118 |
| MSVC发布宿主 | 209 | 0 | 118 |
| MSVC调试文档测试 | 10 | 0 | 0 |
| MSVC发布文档测试 | 10 | 0 | 0 |
| MSVC调试显式探针 | 118 | 0 | 0 |
| MSVC发布显式探针 | 118 | 0 | 0 |
| 常驻调试脚本 | 48 | 0 | 0 |
| 常驻发布脚本 | 48 | 0 | 0 |

118项包含GPU与原生探针。
常驻每轮含6项宿主测试。
常驻每轮含42项GPU测试。
本批新增2项宿主测试。
本批新增6项GPU测试。
其中一项检查混合类型ABI。
四路Clippy全部通过。
文档构建与格式检查均通过。
初轮ABI回归发现宽度推断变化。
我显式固定可信测试的f32类型。
我另核对全部内部调用点。
复验随后全部通过。
文档另作人工核对。
五处本地链接检查通过。
本批不运行Python审计脚本。

两种构建的误差保持一致。
以下数字只统计本批比较。

| 比较 | 每轮世界数 | 最大绝对误差 |
| --- | ---: | ---: |
| 静态原生参考 | 2084 | `7.624788453952647e-7` |
| 圆柱独立解析公式 | 1042 | `2.68923144641775e-7` |

绝对与相对容限均为`2e-5`。
本批没有放宽直接比较容限。
整数输出全部精确匹配。

完整常驻证据位于：

```text
target/resident-kinematics/7193eaa6019a485185e22b8fa8328a56/
target/resident-kinematics/e34ac577b3f040af9ae26a4435e99b21/
```

第一份记录调试测试。
第二份记录发布测试。
两份均记录全部旧子集回归。
汇总与全量日志位于：

```text
target/geom-tendon-verification/report.json
target/geom-tendon-verification/*.log
```

证据记录基线`14a301d`。
测试当时保留本批未提交改动。
记录不冒充清洁提交验收。
Normify只激活一个绕行辅助叶子。
架构保留40个模块与34个叶子。
架构登记226项API与129条依赖。
完整阶段的20个节点仍保持计划。
收尾保留旧历史变更。
架构校验与构建均返回零错误。
收尾复验也返回零警告。

## T4精度修复复验

本轮固定三组运动回归状态。
原生工具加载既有冻结MJB。
产品测试不生成原生参考。
清单另锁定三份文件哈希。
三个源帧为310、344与393。
两种构建各比较九组状态。
九组最大绝对差为`5.561441e-7`。
该数字覆盖长度、力臂与接点。
整数输出仍要求精确一致。

内部暂存从七值扩至十三值。
新增六值缓存内侧连接方向。
载荷测试按布局推导溢出索引。
它继续拒绝非有限结果。
它继续检查后续恢复。
旧哨兵与冻结退化测试均通过。
本轮不新增API或第三方依赖。

| 复验 | 调试 | 发布 | 失败 |
| --- | ---: | ---: | ---: |
| MSVC宿主 | 209 | 209 | 0 |
| MSVC文档测试 | 10 | 10 | 0 |
| 显式GPU与原生探针 | 119 | 119 | 0 |
| 常驻脚本 | 49 | 49 | 0 |

GNU宿主169项通过。
GNU文档测试10项通过。
常驻每轮含6项宿主测试。
常驻每轮含43项GPU测试。
本轮只新增一项GPU测试。
四路Clippy通过。
文档构建与格式检查通过。
执行命令沿用首批验收命令。
原生测试先准备模拟DLL。

```powershell
scripts/test-windows-native.ps1 -AllFeatures
```

| 比较 | 每轮世界数 | 最大绝对误差 |
| --- | ---: | ---: |
| 静态原生参考 | 2084 | `4.350222e-7` |
| 圆柱独立解析公式 | 1042 | `2.689232e-7` |
| 三帧内侧原生回归 | 9 | `5.561441e-7` |

全部480帧另作原生复核。
1920状态均保持原容限通过。
完整结果见[逐帧报告](windows-geom-tendon-frame-review.md)。
本轮保留首批日志与视频。
新日志位于以下目录。

```text
target/geom-tendon-t4-verification/
target/geom-tendon-t4-review/
```

记录基线为`4b13bd0`。
测试当时包含本轮未提交改动。
本轮未验证Linux或清洁部署。

## 当前限制与后续

本批仍只覆盖严格辅助子集。
全局肌腱编号与导入仍待统一。
肌腱速度、限位与惯量仍待实现。
flex位置与休眠副作用仍待补齐。
完整G01仍需双平台验收。
本轮不执行Linux或清洁部署。
Normify不激活完整产品阶段。

下一步统一肌腱全局编号。
随后补齐flex与休眠分支。

[冻结绕行源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/util_misc.py)。
[冻结肌腱源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。
[Warp反三角源码](https://github.com/NVIDIA/warp/blob/v1.15.0/warp/native/builtin.h)。
