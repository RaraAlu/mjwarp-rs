# Windows运动学mocap子集

日期：2026-10-07。
范围：G01常驻辅助增量。
本批不关闭完整G01。
本批不增加第三方依赖。
产品链不引入Python。

## 实际改动

模型辅助检查mocap映射。
映射长度等于体数量。
-1表示普通体。
非负编号必须唯一且连续。
编号顺序无需匹配体顺序。
mocap体必须直接连接世界。
mocap体自身不允许关节。
固定后代与关节后代均可用。
模型拓扑仍共享一份。

公开接口如下：

```text
model::MocapModelInput
physics::KinematicsPlan::with_mocap
physics::KinematicsPlan::nmocap
physics::KinematicsPlan::body_mocapid
physics::KinematicsData::nmocap
physics::KinematicsData::write_mocap
physics::KinematicsData::write_world_mocap
```

调用方继续提供附着模型。
调用方另外提供mocap映射。
旧计划构造器默认无mocap。
它们现在也初始化静态缓存。
原生DTO导入仍未扩展此映射。
这些接口不替代完整Model。

```rust
use mjwarp_rs::{
    diagnostics::TransferError,
    model::{AttachedModelInput, KinematicsParameters, MocapModelInput},
    physics::KinematicsPlan,
    runtime::TransferSession,
};

fn plan_with_mocap(
    session: &TransferSession,
    attached: AttachedModelInput,
    body_mocapid: Vec<i32>,
) -> Result<KinematicsPlan, TransferError> {
    let model = MocapModelInput::new(attached, body_mocapid)?;
    KinematicsPlan::with_mocap(session, model, KinematicsParameters::default())
}
```

## 状态与默认值

状态按字段保留连续世界行。
设备缓冲连续排列三个字段。
qpos写入不会覆盖mocap。
mocap写入不会覆盖qpos。
位置与四元数按编号排列。
位置单位为米。
四元数顺序为wxyz。

```text
qpos:       [W, nq]
mocap_pos:  [W, nmocap, 3]
mocap_quat: [W, nmocap, 4]
```

全部输入必须保持有限。
四元数平方范数范围如下：

```text
1e-12 <= norm_squared <= 1e12
```

GPU更新归一化四元数。
写入保留原始状态值。
全部输入检查先于设备写入。
输入错误保留已完成结果。
合法写入废弃整组派生结果。
驱动失败不发布宿主快照。
运行时沿用会话隔离规则。
零mocap只接受空输入。
零自由度仍支持mocap状态。

qpos默认值仍取qpos0。
mocap默认值取对应体参数。
位置与姿态分别按世界取模。
它们沿用各自字段周期。
创建状态时复制这些默认行。
后续更新直接读取世界状态。
它不再重新读取默认行。
参数与拓扑仍保持只读。

此规则属于辅助入口约定。
冻结上游使用原生模型默认值。
上游make_data初始化不等价。
本辅助不调用原生CPU物理。
它也不等价于reset_data。
依据：[冻结初始化源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py)。

## 静态geom语义

模型辅助推导静态geom掩码。
普通世界焊接体属于静态体。
关节后代不属于静态体。
mocap后代也不属于静态体。
此判定复现以下冻结条件：

```text
body_weldid[body] == 0
&& body_mocapid[body_rootid[body]] == -1
```

创建状态先运行GPU刚体计算。
GPU随后初始化附着位姿。
初始化也计算动态geom与site。
初始化不标记公开阶段就绪。
调用方仍须显式执行更新。
后续附着更新跳过静态geom。
后续附着更新重算全部site。
十三项参数仍独立取模。
静态缓存保留对应世界初始行。
模型参数刷新接口仍待实现。
完整初始化与等价导入仍待实现。
依据：[冻结运动学源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。

计划仍只上传三份模型子集。
计划一次编译四个内核入口。
第四个入口负责初始化附着。
设备ABI继续使用f32。
普通更新不上传模型或编译。
阶段间仍不回读宿主。
旧探针保持共享参数行为。
旧f64求解也不改变设备ABI。

源码与测试位置如下：

- `src/model/mocap.rs`
- `src/physics/resident.rs`
- `src/physics/mod.rs`
- `src/physics/attached.rs`
- `tests/resident_mocap.rs`
- `fixtures/mocap-kinematics/`

## 独立数值与边界证据

作者使用C++制作一次参考。
生成器链接MuJoCo 3.12.0。
它只调用运动学与质心阶段。
样本保留八组固定输入。
样本包含两个反序mocap编号。
样本覆盖嵌套静态子体。
样本覆盖mocap固定与关节后代。
样本也包含普通滑动关节。
输入先固定为f32数值。
原生参考保留f64输出。
哈希清单记录XML、MJB与JSON。
清单也记录生成器与DLL哈希。
产品测试不运行参考生成器。
产品测试不编译模型。
Git强制保留样本LF与二进制。

GPU比较十五项派生字段。
它们包含刚体、附着与质心。
世界数覆盖1、2、5、513。
每组验证默认状态与三轮更新。
测试共记录2084组世界比较。
固定种子为1789。
绝对与相对容限均为2e-5。
测试不放宽既有容限。

边界测试检查以下行为：

- 首尾世界与跨块世界写入。
- 多组状态与旧快照隔离。
- 相同内容的跨计划拒绝。
- 输入错误保留已完成结果。
- 合法写入废弃阶段就绪状态。
- 非有限值与不可用四元数。
- 零mocap与零自由度。
- 仅geom、仅site与空附着。
- 默认参数周期3、5、7。
- 计划释放后保留回读能力。

缓存测试刻意改写设备体位姿。
它确认静态geom保持原始值。
它确认site使用新体位姿。
测试覆盖513个世界与四种集合。
测试只在私有模块改写设备值。
产品API不开放这种改写。

## Windows实测

平台：Windows。
目标：x86_64 MSVC。
系统版本：10.0.26200.0。
GPU：4070 Ti SUPER。
驱动：596.36。
Rust：1.99.0。
工具链：CUDA 12.8.1。
NVRTC仍属候选工具链。
参考原生库：3.12.0候选。
正式依赖版本仍需单独决议。

实际执行以下脚本：

```powershell
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
```

两路各通过19项显式测试。
两路均无失败或忽略项。
其中两项只检查静态清单。
其余17项执行GPU测试。
共享参考各比较1245组世界。
独立批量各比较3102组世界。
mocap参考各比较2084组世界。

| 比较范围 | 最大绝对误差 |
| --- | --- |
| 共享参考 | 4.158170539447781e-6 |
| 独立批量 | 1.1444091796875e-5 |
| mocap参考 | 2.868815873746655e-7 |

两路报告目录如下：

```text
target/resident-kinematics/811f855fee574a138168787dee46ea7a/
target/resident-kinematics/a384436c5b084bda8d3e9108232e0508/
```

报告记录工作树修改状态。
它们记录修改前的HEAD。
它们不冒充提交后清洁验收。
脚本不调用Python或原生物理。

默认GNU测试通过158项。
默认文档测试通过7项。
格式与GNU两路Clippy通过。
MSVC两路各通过189项宿主测试。
两路各通过7项文档测试。
默认列表各忽略93项。
显式执行后各通过93项。
它们覆盖GPU与原生探针。
两路均无失败与残留忽略项。
MSVC两路Clippy与文档构建通过。
完整回归使用以下命令：

```powershell
cargo test --locked
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features cuda-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features --release -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
```

原生回归需要预备DLL与环境。
本批复用原生脚本预备结果。
命令不能替代环境预备步骤。
日志保留以下目录：

```text
target/mocap-verification/
```

本批仅人工核对相关文档。
本批不运行Python审计工具。

## 剩余G01范围

下一批补齐相机与光源。
模型参数刷新仍待实现。
完整Model与Data仍待实现。
等价阶段与导入仍待实现。
休眠、柔性体与腱分支仍待实现。
Linux与清洁部署仍待验收。
完整G01继续保持计划状态。
