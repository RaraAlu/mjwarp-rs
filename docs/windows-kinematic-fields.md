# Windows运动学字段增量

日期：2026-10-07。
后续增量详见[惯性报告](windows-inertial-fields.md)。
本报告保留十二字段历史证据。
范围：P2只读运动学子集。
本轮继续只做Windows。
本轮不冻结生产原生ABI。
本轮不实现GPU物理或渲染。

## 实现与文件归属

```text
src/
  model/
    mod.rs           批量布局与原生计数DTO
    topology.rs      关节值、字段与只读输入
  io/
    mod.rs           旧字段与原生模型所有者
    kinematic.rs     十二字段转换与GPU组
  diagnostics/
    mod.rs           拓扑错误与原有诊断
include/
  mjwarp_native_probe.h    私有目标DTO声明
native/
  model_probe.cpp         原生复制前校验
tests/
  native_model_probe.rs   宿主与GPU集成测试
  native/
    kinematic_preflight.rs  独立桥接ABI断言
    mock_mujoco.cpp        原生错误测试替身
fixtures/native-probe/
  mixed-joints.xml         独立编写的混合样本
  mixed-joints.mjb         官方工具编译产物
  mixed-joints-reference.txt  官方结构打印件
  manifest.json           来源与全部样本哈希
```

内部文件沿用原有模块。
本轮不铺新增空壳模块。
旧四字段接口保持兼容。
Rust提供所有公开宿主接口。
C++只复制已编译模型字段。
默认构建不接入C++或DLL。
原生feature仍限定Windows MSVC。
本轮不新增第三方依赖。
依赖版本与许可沿用[原生报告](windows-native-model-probe.md)。

## 十二项字段规范

参数批量长度固定为一。
这对应共享参数`B_f=1`。
本轮不支持周期参数批量。
它不表示只模拟一个世界。
当前尚未接入物理世界数量。
整数保留连续i32索引布局。
原生快照使用f64浮点。
校验输入与GPU字段使用f32。
vec3与四元数按标量展开。
GPU各字段独占连续缓冲。
每个标量占四字节。

| 字段 | 标量长度 | 语义与单位 |
| --- | --- | --- |
| `qpos0` | `nq` | 初始广义坐标；free含位置米及四元数，ball含四元数，slide米，hinge弧度 |
| `body_parentid` | `nbody` | 父体索引；世界体索引零，世界父体零 |
| `body_jntadr` | `nbody` | 体的首关节地址；无关节时为`-1` |
| `body_jntnum` | `nbody` | 每体关节数；世界体为零 |
| `body_pos` | `3*nbody` | 父体坐标系内位置；米 |
| `body_quat` | `4*nbody` | 父体坐标系内姿态；wxyz |
| `jnt_type` | `njnt` | 原生四类关节整数值 |
| `jnt_bodyid` | `njnt` | 关节所属体索引；不得指向世界体 |
| `jnt_qposadr` | `njnt` | 广义坐标首地址 |
| `jnt_dofadr` | `njnt` | 自由度首地址 |
| `jnt_pos` | `3*njnt` | 体坐标系内关节锚点；米 |
| `jnt_axis` | `3*njnt` | 体坐标系内轴向量；无量纲 |

字段语义依据[候选原生声明](https://github.com/google-deepmind/mujoco/blob/3.12.0/include/mujoco/mjmodel.h)。
四类关节依据[原生枚举声明](https://github.com/google-deepmind/mujoco/blob/3.12.0/include/mujoco/mjtype.h)。
候选版本仍采用3.12.0。
项目仍对标冻结MJWarp源码。
候选原生版本不代表正式冻结。

| JointType | 原生值 | qpos宽度 | dof宽度 |
| --- | --- | --- | --- |
| Free | 0 | 7 | 6 |
| Ball | 1 | 4 | 3 |
| Slide | 2 | 1 | 1 |
| Hinge | 3 | 1 | 1 |

## 校验与失败边界

`KinematicFields`持有原始数组。
`KinematicModelInput`先校验。
校验成功后只允许只读借用。
校验覆盖以下边界：

- 世界体存在且父体为零。
- 所有父体均先于子体。
- 关节分区连续且完整。
- 空关节地址保持`-1`。
- 关节所属体匹配分区。
- free关节独占根体关节。
- 坐标地址按关节宽度连续。
- 宽度总和匹配nq与nv。
- 所有字段长度精确匹配。
- 乘法与切片容量不溢出。
- 计数不超出i32地址容量。
- 五项浮点字段保持有限。

零自由度与静态体保持合法。
校验不归一化四元数或轴。
校验不修改有限原始位值。
有限非单位值仍能通过。
这不宣称它们物理有效。
完整物理组合仍待展开。
质量、资产与惯性仍待校验。
错误保留字段、索引与原因。

原生快照转换先检查全量长度。
转换拒绝NaN、无穷与f32溢出。
转换允许舍入、下溢与负零。
失败不返回部分校验输入。
严格辅助不替代等价put_model。

## 原生桥接与所有权

原生加载沿用可信入口。
它仍先检查版本与三个符号。
新复制入口不新增DLL符号。
私有目标DTO采用schema一。
Rust与C++校验128字节布局。
两侧校验八字节对齐。
两侧校验关键字段偏移。
桥接先检查三项计数。
桥接先检查全部所需指针。
任一错误均发生在复制前。
空关节数组不触发解引用。

快照独占十二项字段副本。
它不保留任何原生指针。
原生模型与DLL可以先释放。
原生删除仍先于DLL卸载。
恶意DLL或MJB仍不属于安全输入。
本轮不新增进程级解析隔离。

`DeviceKinematicModel`拥有字段组。
上传逐字段执行同步复制。
全部上传完成后才返回对象。
失败不发布部分GPU字段组。
清理沿用运行时完成检查。
未知完成继续隔离并保留资源。
本轮不注入真实GPU故障。
本轮不新增异步模型转换。

设备对象隐藏可写缓冲。
设备对象不导出裸指针。
每项缓冲保留传输会话所有者。
调用者可以先释放会话句柄。
回读产生独占宿主数组。
回读后再次校验结构。
失败不返回部分回读对象。

## 独立样本与预期值

旧两个MJB保持原字节。
本轮新增混合关节样本。
它覆盖free、ball、slide、hinge。
它还覆盖世界体与静态子体。
人工预期采用精确二进制分数。
官方工具独立编译固定MJB。
官方工具另生成结构打印件。
打印件确认顺序、默认轴与地址。
打印件小数精度不作浮点容限。
产品测试只读取固定MJB。
产品链不运行模型编译器。
产品链不运行Python。
样本制备步骤见[样本说明](../fixtures/native-probe/README.md)。
SHA256见[固定清单](../fixtures/native-probe/manifest.json)。

## 验证记录

本机使用Windows x86_64。
系统版本：Windows 11 Pro 26200。
GPU：RTX 4070 Ti SUPER。
驱动版本：596.36。
Rust版本：1.99.0。
MSVC版本：14.44.35207。
SDK版本：10.0.26100。
CUDA候选工具链：12.8.1。
默认Rust GNU工具链保持不变。

| 检查 | 通过 | 失败 | 说明 |
| --- | --- | --- | --- |
| 默认GNU宿主 | 112 | 0 | 105项库测试及7项CLI；另有4项文档测试 |
| MSVC原生单feature宿主 | 117 | 0 | 110项库测试及7项CLI；另有4项文档测试 |
| MSVC CUDA单feature宿主 | 125 | 0 | 118项库测试及7项CLI；另有4项文档测试 |
| 全feature宿主debug | 134 | 0 | 127项库测试及7项CLI；另有4项文档测试 |
| 全feature宿主release | 134 | 0 | 同上 |
| 全feature显式debug | 41 | 0 | 原有26项，加12项原生宿主与3项原生GPU测试 |
| 全feature显式release | 41 | 0 | 同上 |
| 原生单feature显式debug/release | 各12 | 0 | 不要求GPU |
| 原生加CUDA显式debug | 15 | 0 | 包含整组GPU上传、回读与提前释放 |
| 全feature原生受限debug/release | 各15 | 0 | 系统PATH运行实际测试产物 |
| 原生单feature受限debug/release | 各12 | 0 | 系统PATH运行实际测试产物 |

默认GPU测试继续显式忽略。
全量显式检查另行运行。
本轮新增九项默认拓扑测试。
原生feature另增两项单元测试。
文档测试拒绝修改只读资源。
桥接测试逐项注入17类错误。
每类错误都保留全量目标哨兵。
另测原生字段指针缺失与删除顺序。
实际GPU回读覆盖十二项字段。
实际GPU测试覆盖提前释放。
实际GPU测试保留负零与非单位值。
所有浮点样本采用既定精确值。
本轮不放宽数值容限。

以下检查也已执行：

- `cargo fmt --check`。
- 默认GNU全target clippy。
- 原生单feature全target clippy。
- CUDA单feature全target clippy。
- 全feature全target clippy。
- 全feature rustdoc。

clippy全部使用`-D warnings`。
rustdoc检查不包含依赖文档。
旧截断测试仍输出原生警告。
该警告属于预期失败分支。
它不表示Rust或架构警告。
MUJOCO_LOG仍遵循现有忽略规则。

本轮实际执行以下命令：

```powershell
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --features native-model-probe
cargo +stable-x86_64-pc-windows-msvc clippy --locked --features native-model-probe --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --features cuda-probe
cargo +stable-x86_64-pc-windows-msvc clippy --locked --features cuda-probe --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
& scripts/test-windows-native.ps1 -Gpu -RestrictedRuntime
& scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime
& scripts/test-windows-native.ps1 -AllFeatures -Release -RestrictedRuntime
& scripts/test-windows-native.ps1 -RestrictedRuntime
& scripts/test-windows-native.ps1 -Release -RestrictedRuntime
```

MSVC命令先初始化开发环境。
原生命令设置已校验的包路径。
全feature另设置现有CUDA路径。
脚本执行相同的局部初始化。
测试脚本先核对七份样本文件。
本轮不修改Rust默认工具链。

宿主与全量回归日志位于：

```text
target/kinematic-checks/host-native.log
target/kinematic-checks/host-cuda.log
target/kinematic-checks/host-all.log
target/kinematic-checks/host-all-release.log
target/kinematic-checks/gpu-all.log
target/kinematic-checks/gpu-all-release.log
target/kinematic-checks/clippy-native.log
target/kinematic-checks/clippy-cuda.log
target/kinematic-checks/clippy-all.log
target/kinematic-checks/doc-all.log
```

受限证据位于以下运行目录：

| 运行 | `target/native-probe/`下的run-id |
| --- | --- |
| 原生加CUDA debug | `ba757737eb234cb8ae717cbc591c112f` |
| 全feature debug | `f4182b279c8d46cf8c0c7251deb366df` |
| 全feature release | `1ca5285b16324af1bb10aa0fe23e6c01` |
| 原生单feature debug | `286cae39ac0448678e13ec66879b025c` |
| 原生单feature release | `0f27ec3895684f7b97abd909c9b115bd` |

每项报告记录当时HEAD与脏状态。
上述报告验证提交前工作区。
它们不把基线HEAD冒充新提交。
运行目录保留JSON及测试日志。
受限PATH不移除系统运行库。
它不证明独立清洁机器部署。
原生信任与运行库边界不变。

## 架构边界

本轮保留原有十六节点。
model与io各新增两个职责叶子。
架构节点总数现为20。
本轮保留旧174项API路径。
API归属随职责叶子显式迁移。
引用方同步指向新叶子。
本轮新增四项字段辅助API。
API总数现为178。
依赖总数现为42。
图校验取得零错误与零警告。
源码漂移检查不发现失效证据。
本轮不削弱任何架构规则。
四种关节值实现原JointType路径。
该实现不证明完整G01等价。
七个产品模块仍保持计划态。
同步传输叶子保持活跃态。
四个新叶子均保持计划态。
完整Model与ModelInput仍待实现。
MC01、MC02与MC31仍待完成。
生产摘要与模型身份仍待冻结。
Linux与清洁部署均未执行。

## 下一步

- 扩展体惯性与自由度字段。
- 接入运动学所需派生布局。
- 建立独立前向运动学参考。
- 验证多世界GPU运动学。
- 再落实完整模型与状态对象。

以上步骤不缩减首版范围。
