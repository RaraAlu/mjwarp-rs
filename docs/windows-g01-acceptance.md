# Windows G01全阶段验收

日期：2026-10-09。
状态：Windows全阶段验收通过。
矩阵：G01。

## 1. 验收范围

本轮实现完整运动学调度。
本轮保留等价低层入口。
本轮只验收Windows。
本轮不关闭双平台总门禁。

冻结上游身份如下：

```text
google-deepmind/mujoco_warp
71da24d956378a87a703b6e1442b13aec0c4ac29
```

G01使用以下设备调度：

```text
kinematics -> com_pos -> camlight -> flex -> tendon
  -> 条件肌腱唤醒与轻量树刷新
```

`flex`包含位置、边与壳体面。
它先清除Hessian有效标志。
G01不计算Hessian矩阵。
G01不执行碰撞与质量阶段。
G01不建立约束与岛。
这些阶段不属于本轮门禁。

调度依据：[冻结位置源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1302-L1318)。

## 2. 原生模型转换

`NativeModelProbe::g01_snapshot`复制输入。
`NativeG01Snapshot::into_model`完成转换。
快照独占78项原生字段。
它复用21项刚体惯性字段。
它复用12项柔体位置字段。
它补齐45项G01读字段。
字段覆盖附着、相机与光源。
字段也覆盖肌腱与边稀疏行。
开关只提取G01使用的位。
测试复用`serde_json`。
纯原生测试也启用该解析库。
产品默认构建不启用该解析库。
版本锁定为1.0.151。
许可采用MIT或Apache-2.0。
本轮不冻结完整模型ABI。

桥接先查询全部新增容量。
Rust再分配独占暂存。
桥接逐项复核类型与容量。
零计数允许空指针。
非空字段拒绝缺失指针。
浮点转换检查有限性与溢出。
布尔转换只接受0与1。
模型释放不影响快照。

实现见[原生转换](../src/io/g01.rs)。
ABI见[字段桥接](../native/g01_fields.inc)。

## 3. 常驻入口与数据契约

`KinematicsPlan::for_g01`建立等价计划。
模型字段与拓扑保持只读。
计划复用既有GPU内核。
每次更新不重传模型。
每次更新不重新编译内核。
阶段间不回读宿主中间量。

`import_g01_state`先预检全部输入。
输入错误保留既有结果。
设备写入失败废弃就绪票据。
该导入不隐式计算物理。
缺省缓存保留既有设备值。
世界位姿覆盖刚体缓存首项。

| 状态字段 | 每世界布局 |
| --- | --- |
| `qpos/qvel` | nq与nv项f32 |
| `mocap_pos/mocap_quat` | xyz与wxyz |
| `rigid_cache` | xpos、xquat、xmat、xipos、ximat、xanchor、xaxis |
| `com_cache` | subtree_mass、subtree_com、cinert、cdof |
| `attached_cache` | geom_xpos、geom_xmat、site_xpos、site_xmat |
| `world_pose` | xyz与wxyz，共七项 |
| `flex_hessian_valid` | 每柔体一个布尔值 |
| `sleep` | 独立asleep、awake与三个计数 |

矩阵统一按行展开。
缓存不包含私有哨兵。
`subtree_mass`属于内部暂存段。
低层读者不使用此段。
COM阶段重算此段。
调用方负责物理一致性。
库仍检查模型身份与容量。
有限非单位姿态保持合法。
零状态四元数保持上游语义。
旧严格计划保留原有检查。
严格相机计划检查共享姿态。
参数覆盖只检查实际使用行。
等价构造仍拒绝非有限字段。

`fwd_kinematics`执行完整G01。
五个低层入口不要求票据。
低层入口不隐式计算依赖。
低层`tendon`不执行唤醒。
`readback`继续要求阶段就绪。
`readback_g01`读取当前设备字段。
它不承诺未计算字段新鲜。
两类回读均检查哨兵与有限性。
这些入口不等同完整M02与M07。

## 4. 分支覆盖

| 分支 | 实际检查 |
| --- | --- |
| 刚体 | 四类关节、偏心旋转、多世界 |
| 附着 | 世界位姿、静态缓存、动态site |
| mocap | 独立状态与有限非单位姿态 |
| COM | 子树质心、cinert与cdof |
| 相机光源 | 五类模式与负目标回退 |
| 固定肌腱 | 非零与零系数、全局CSR |
| 空间肌腱 | site、pulley、球柱与内侧 |
| 柔体位置 | 直接、正负线性、居中与非居中 |
| 柔体边 | 长度、稀疏雅可比与速度 |
| 壳体面 | 非均匀线性网格与极分解 |
| Hessian标志 | 先清除全部世界有效标志 |
| 树唤醒 | 限位、循环、活动标记与开关 |
| 原生缺字段 | 缺指针、负容量与字节溢出 |
| ABI错误 | 45项ID、类型、容量与空参数 |
| 状态预检 | 19类无写入失败与模型隔离 |
| 低层调用 | 已准备字段、逆序与无隐式依赖 |
| 空模型 | 零自由度与空可选集合 |

完全唤醒值采用-11。
轻量刷新重算树活动计数。
它清零身体与自由度活动计数。
它不生成完整活动表。
禁用休眠时保留导入树状态。
禁用岛时也保留导入树状态。
完整G25仍待实现。
二次插值仍遵循冻结拒绝。
完整G22仍待实现。

## 5. 原生参考与误差

本轮离线调用原生MuJoCo。
产品测试只读取冻结输出。
产品测试不调用CPU物理。
它们仍调用原生模型读取。
原生版本采用3.12.0。
样本来源见[参考说明](../fixtures/g01/README.md)。

完整链检查2084组世界状态。
批量规模采用1、2、5、513。
每个规模执行四轮导入与更新。
输入覆盖16组确定性状态。
完整链比较7704548个浮点量。
低层检查另比较50512个量。
测试逐项检查包裹整数。
测试检查未用包裹位置槽清零。
测试逐项检查树状态与计数。
测试检查78项转换字段。

```text
abs(actual - expected) <= 2e-5 + 2e-5 * abs(expected)
```

本轮保留原有数值容限。
面姿态允许四元数整体变号。
原生库不输出Warp壳体面。
作者使用独立特征分解参考。
原生库跳过插值边长度。
作者使用原生顶点测量补齐。
原生库不提供Warp树副作用。
测试使用独立树状态公式。
这些差异不隐藏在容限中。
CPU对齐不证明逐位GPU等价。
冻结公式与副作用另行核对。

## 6. Windows命令与证据

平台采用Windows x64 MSVC。
系统版本采用10.0.26200。
GPU采用RTX 4070 Ti SUPER。
驱动采用596.36。
Rust采用1.99.0。
NVRTC采用私有CUDA 12.8.1。

完整验收执行以下命令：

```powershell
scripts/test-windows-g01.ps1 -AllFeatures -FullRegression
scripts/test-windows-g01.ps1 -AllFeatures -FullRegression -Release
```

脚本核对参考与DLL摘要。
脚本拒绝零用例验收。
脚本调用既有原生与常驻门禁。
脚本另跑完整显式探针回归。
它也检查纯原生G01转换。
脚本保留日志与结构化报告。
产物保存至以下忽略目录：

```text
target/g01-acceptance/<run-id>/report.json
target/g01-acceptance/<run-id>/checks.json
```

双模式门禁全部通过。

| 门禁 | debug | release |
| --- | ---: | ---: |
| 原生桥接 | 27 | 27 |
| 受限原生运行 | 27 | 27 |
| MSVC宿主与文档测试 | 259 | 259 |
| G01门禁 | 7 | 7 |
| 原生G01组合 | 2 | 2 |
| 纯原生G01转换 | 1 | 1 |
| 常驻运动学回归 | 98 | 98 |
| 显式忽略项回归 | 169 | 169 |
| 默认GNU宿主测试 | 201 | 201 |
| 格式、三组Clippy与Rustdoc | 通过 | 通过 |
| 差异空白检查 | 通过 | 通过 |

G01门禁含一个宿主测试。
它另含六个GPU测试。
显式回归也包含原生输入测试。
表中各门禁存在重叠。
本轮不把重复项相加。
GNU宿主测试均采用默认模式。

两种模式取得相同误差。
完整链最大绝对误差如下：

```text
1.3855896387748867e-6
```

低层最大绝对误差如下：

```text
4.977585623677783e-7
```

本机原始证据位于以下路径：

```text
debug: target/g01-acceptance/6edc62c9ef674321a44e3bbaf69c43f1/report.json
release: target/g01-acceptance/1ad133e35b6a437c95c325f7d0f375a9/report.json
native-only: target/native-probe/c00cd9723b014299be0ad2b21fe96340/report.json
native-only-host: target/native-probe/f7d6e0a604ed4c26bd2ea1b983248d1e/native-only-host.log
native-only-clippy: target/native-probe/f7d6e0a604ed4c26bd2ea1b983248d1e/native-only-clippy.log
```

这些日志不进入Git。
验收时HEAD仍属编码起点。
工作区包含本轮完整变更。
智能提交随后固化这些变更。

本轮另跑纯原生配置门禁。
常规与受限运行各通过22项。
纯原生宿主测试通过213项。
纯原生Clippy也通过。
默认构建也通过。
默认产品依赖树仍不含解析库。

```powershell
scripts/test-windows-native.ps1 -RestrictedRuntime
cargo +stable-x86_64-pc-windows-msvc test --locked --features native-model-probe
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --features native-model-probe -- -D warnings
cargo build --locked
cargo tree --locked --edges normal --depth 1
```

### 门禁修复记录

首轮脚本错误统计计数。
有序字典不支持该属性统计。
脚本改用显式数值累加。
首轮文档命令拆开单项参数。
脚本改用固定字符串数组。
修复后重新执行完整门禁。

复核发现共享相机校验缺口。
默认参数未复查共享姿态。
实现补齐严格共享检查。
单测检查等价与覆盖边界。
本轮随后复跑最终源码门禁。

纯原生测试初稿缺少解析依赖。
测试曾误借CUDA特性的解析库。
本轮增加测试专用依赖声明。
本轮保留78项字段逐项比较。
脚本另设纯原生转换门禁。
本轮不启用额外产品依赖。

暂存预检发现末尾空行。
本轮清理作者与XML空行。
本轮同步更新两项文件摘要。
本轮不更改原生参考数值。

低层测试初稿遗漏质量暂存段。
测试补齐明确的COM缓存布局。
原生参考初稿误用插值边零值。
作者改用独立顶点长度测量。
原生溢出测试初稿错判错误层。
测试改查桥接预检错误码。
这些修复不放宽数值容限。

### 文档与架构复核

本轮同步当前G01状态文档。
自动文档审计扫描191项源码。
它扫描65项文档。
它保留九条既有路径提示。
本轮G01文档没有新增提示。
历史路径提示不阻塞物理门禁。
本轮不宣称全库手工审计完成。
架构校验取得零错误。
架构仍保留两条已有警告。
它们涉及叶子粒度与开放变更。
本轮只激活G01转换叶子。
其余完整产品容器保持计划态。

## 7. 尚未验收项

Linux验收仍待执行。
T4硬件复验不属于本轮证据。
本轮不验收完整put_model。
本轮不验收完整put_data。
G02与G03仍待完整实现。
G06速度与传动仍待实现。
G22材料与矩阵乘法仍待实现。
G25完整休眠状态机仍待实现。
本轮不验收性能与执行图。
本轮不关闭清洁部署门禁。
本轮不宣称引擎整体完成。
