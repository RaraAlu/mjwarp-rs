# Windows惯性字段增量

日期：2026-10-07。
范围：P2惯性与自由度子集。
本轮只验收Windows。
本轮不冻结生产原生版本。
本轮不实现物理或渲染。

## 实现归属

```text
src/model/inertial.rs       九字段与祖先链校验
src/io/inertial.rs          原生转换与GPU字段组
src/io/kinematic.rs         复用内部转换与传输
src/io/fields.rs            独立连续字段底层
src/io/mod.rs               新快照入口与原有所有者
src/diagnostics/mod.rs      新增负值错误
include/mjwarp_native_probe.h  私有目标DTO
native/model_probe.cpp     全量指针与计数预检
tests/native/inertial_fields.rs     原生与GPU测试
tests/native/inertial_preflight.rs  独立桥接ABI测试
tests/native/mock_mujoco.cpp        缺失指针替身
fixtures/native-probe/inertial-tree.xml
fixtures/native-probe/inertial-tree.mjb
fixtures/native-probe/inertial-tree-reference.txt
```

本轮不增加第三方依赖。
默认构建不接入C++或DLL。
原生feature仍要求Windows MSVC。
Rust继续提供主API。
C++仅复制已编译模型字段。
旧四字段及十二字段函数不变。
InputError新增NegativeValue。
穷尽匹配需补充错误分支。

## 九字段规范

本轮组合此前十二字段。
组合字段总数为二十一。
参数批量长度固定为一。
原生浮点快照使用f64。
校验输入与GPU使用f32。
整数索引继续使用i32。
vec3与四元数按标量展开。
每个GPU字段独占连续缓冲。
每个GPU标量占四字节。

| 字段 | 标量长度 | 语义与单位 |
| --- | --- | --- |
| `body_ipos` | `3*nbody` | 体坐标系内质心；米 |
| `body_iquat` | `4*nbody` | 惯性系相对体姿态；wxyz |
| `body_mass` | `nbody` | 体质量；kg |
| `body_inertia` | `3*nbody` | 惯性系对角主惯量；kg*m^2 |
| `dof_bodyid` | `nv` | 所属体索引 |
| `dof_jntid` | `nv` | 所属关节索引 |
| `dof_parentid` | `nv` | 最近祖先自由度；无祖先用`-1` |
| `dof_armature` | `nv` | 平移用kg；转动用kg*m^2 |
| `dof_damping` | `nv` | 原生阻尼；平移kg/s；转动kg*m^2/s |

字段依据[候选原生声明](https://github.com/google-deepmind/mujoco/blob/3.12.0/include/mujoco/mjmodel.h)。
祖先关系依据[原生编译树](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/user/user_model.cc)。
候选原生版本仍为3.12.0。
冻结MJWarp提交保持不变。
本轮只复制原生标量阻尼。
本轮不叠加执行器阻尼。
本轮不接入阻尼多项式。
它不替代上游put_model转换。
十二字段详见[历史报告](windows-kinematic-fields.md)。

## 校验与失败边界

InertialFields保留未检查数组。
InertialModelInput执行校验。
它拥有已校验运动学输入。
它仅导出只读字段借用。

- 全部字段长度精确匹配。
- 乘法与切片容量不溢出。
- 六项浮点数组保持有限。
- 质量、惯量与参数不为负。
- 自由度匹配关节与所属体。
- 自由度宽度沿用四类关节。
- 同关节与同体自由度串联。
- 首自由度查找最近动态祖先。
- 静态祖先不会切断自由度链。
- 兄弟分支不串入全局前项。
- 无动态祖先时保留`-1`。

本辅助接口主动拒绝负参数。
它不替代正式等价准入。
校验保留有限姿态原始位值。
校验不归一化惯性四元数。
校验允许负零与零惯量。
校验不强制世界质量为零。
校验不检查惯量三角关系。
校验不检查动态体最小质量。
结构校验不证明物理有效。

原生转换先检查全量长度。
转换拒绝NaN、无穷与f32溢出。
负原生参数在转换前触发错误。
下溢不能掩盖微小负参数。
其余有限值允许舍入与下溢。
错误保留字段、索引与阶段。
失败不发布部分模型输入。

## 原生ABI与所有权

新增私有目标DTO为96字节。
两侧核对八字节对齐。
两侧核对四处关键偏移。
目标DTO采用schema一。
桥接先检查nbody与nv。
桥接先检查全部必需指针。
桥接先检查展开容量。
全部预检通过后才复制。
零自由度允许空目标指针。
桥接不解引用空自由度数组。
不安全调用要求目标互不重叠。
调用方须提供足够目标容量。
Rust拥有独立对齐数组。

快照独占二十一字段副本。
两次复制借用同一只读模型。
所有者在两次复制间保持存活。
接口不允许修改原生模型。
原生模型与DLL可先释放。
失败不会提前删除模型。
Drop先删除模型，再卸载DLL。
新增桥接不增加原生DLL符号。
本轮不承诺恶意MJB隔离。

DeviceInertialModel复用字段组。
各字段同步等待上传完成。
全部字段成功后才返回对象。
失败释放已构建的健康字段。
不确定完成沿用运行时隔离。
字段持续持有传输会话。
宿主输入和会话句柄可先释放。
回读重新校验全部二十一字段。
接口不暴露设备地址或可写缓冲。
本轮未注入真实GPU故障。
本轮不增加异步上传或图捕获。

## 独立样本

新样本包含八体与六关节。
关节覆盖free、ball、slide和hinge。
样本包含静态桥接与兄弟分支。
同体包含slide与hinge双关节。
另一根体经静态祖先连接。
显式惯量采用精确二进制分数。
惯性姿态覆盖四种轴向四元数。
官方compile.exe另行制备MJB。
官方工具另行打印模型结构。
产品测试只读取固定MJB。
测试不调用模型编译器或Python。
打印件不充当精确浮点参考。
manifest固定十项文件哈希。
此前七项哈希全部保留。
详见[样本来源](../fixtures/native-probe/README.md)。

独立桥接测试覆盖15种失败。
它检查全部九目标不受改写。
它验证空自由度指针不触发访问。
缺失源指针测试保留错误阶段。
该测试同时确认DU释放顺序。

## Windows实际验证

沿用Windows 11 Pro 26200。
GPU：RTX 4070 Ti SUPER。
驱动：596.36。
Rust：1.99.0；默认GNU不变。
MSVC：14.44.35207。
SDK：10.0.26100。
CUDA候选工具链：12.8.1。
原生版本与许可沿用[原生报告](windows-native-model-probe.md)。

| 检查 | 实际结果 |
| --- | --- |
| 默认GNU宿主 | 120通过；文档6通过 |
| MSVC仅原生宿主 | 126通过；文档6通过 |
| MSVC仅CUDA宿主 | 133通过；文档6通过 |
| MSVC全feature宿主 | debug/release各143通过；文档各6通过 |
| 全feature显式集成 | debug/release各48通过；无忽略 |
| 原生单feature显式集成 | debug/release各17通过；受限PATH各17通过 |
| 原生与CUDA显式集成 | debug/release各22通过；受限PATH各22通过 |
| 全feature原生集成 | debug/release各22通过；受限PATH各22通过 |
| fmt | 通过 |
| 默认、原生、CUDA与全feature clippy | 全部拒绝警告并通过 |
| 全feature rustdoc | 拒绝警告并通过 |

48项包含原生与GPU探针。
其中原生宿主17项，原生GPU5项。
另外26项覆盖原有GPU行为。
本轮新增8项纯宿主单元测试。
本轮新增1项原生暂存单元测试。
本轮新增5项原生宿主测试。
本轮新增2项原生GPU测试。
本轮新增2项只读编译失败测试。
以上数量不表示物理覆盖率。

实际命令如下：

```powershell
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --features native-model-probe
cargo +stable-x86_64-pc-windows-msvc test --locked --features cuda-probe
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --features native-model-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
$env:RUSTDOCFLAGS='-D warnings'
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
./scripts/test-windows-native.ps1 -RestrictedRuntime
./scripts/test-windows-native.ps1 -Release -RestrictedRuntime
./scripts/test-windows-native.ps1 -Gpu -RestrictedRuntime
./scripts/test-windows-native.ps1 -Gpu -Release -RestrictedRuntime
./scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime
./scripts/test-windows-native.ps1 -AllFeatures -Release -RestrictedRuntime
```

MSVC检查先导入VsDevCmd环境。
原生检查指定候选头文件目录。
全feature检查指定CUDA候选目录。
默认GNU检查沿用私有MinGW。
脚本复用同一VS的x64环境。
它不反复扩张MSVC的PATH。
同一进程连跑三次均通过17项。
三次PATH长度均为2996。
日志位于target/inertial-checks。

原生证据位于target/native-probe。
各目录含report.json及测试日志。

| 提交前证据 | 目录ID |
| --- | --- |
| 原生单feature debug | `6e0019023d584301a912c7868e6376fd` |
| 原生单feature release | `95e002b5b7db4cd28054e760646d4929` |
| 原生加CUDA debug | `f1f24b1385bc471f96b272b5e0f39bea` |
| 原生加CUDA release | `0d513c536b4541a39e6bb094faefbe0e` |
| 全feature debug | `0e15b822d8954ebe8b5356bab5d9a935` |
| 全feature release | `bd176043abbd4a8eb706a114329e1f70` |

报告如实记录当时HEAD与脏状态。
受限PATH仍使用系统运行库。
它不替代清洁机器部署验收。
本轮不开展Linux验证。

## 架构与剩余边界

本轮新增三个窄职责叶子。
字段底层承接真实转换与传输。
公开路径继续保持不变。
字段组不依赖原生所有者。
原生所有者依赖字段组。
这项真实拆分打断依赖环。
原生暂存另登记内部契约。
节点总数现为23。
契约总数现为183。
依赖总数现为62。
十二条架构规则保持不变。
旧178项API路径全部保留。
四项旧字段契约移入底层。
本轮新增四项公开辅助契约。
本轮新增一项内部暂存契约。
四个窄辅助叶子保持活跃。
其余十九节点保持计划。
提交后执行变更收尾校验。
源码文件各自对应单个叶子。
两个叶子只声明本轮辅助行为。
完整生产模块继续保持计划态。
Model与Data仍待完整实现。
MC01、MC02与MC31仍待完成。
M01、M02及G01仍无完整验收。
旧运动学变更继续保持开放。
其宽核心节点仍有计划契约。
本轮不伪造该变更的激活证据。

下一步建立独立运动学参考。
随后落实最小GPU前向运动学。
再扩展资产、完整模型与状态。
正式准入仍要求双平台。
