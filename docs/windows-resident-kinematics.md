# Windows常驻运动学底座

日期：2026-10-07。
范围：G01严格辅助子集。
本轮不关闭完整G01。
本轮不冻结正式GPU路线。
本报告记录首批共享参数证据。
后续批量见[参数报告](windows-resident-parameters.md)。

## 本批目标

现有探针每次上传模型。
它们每次编译并回读结果。
本批新增常驻设备执行链。
旧探针继续保留原有行为。

本批复用三组CUDA公式。
本批不添加第三方依赖。
参数批量长度仍固定为一。

## 接口与所有权

| 类型 | 实际职责 |
| --- | --- |
| `KinematicsPlan` | 拥有模型快照、设备字段与三组编译内核 |
| `KinematicsData` | 独占qpos、刚体、附着及质心设备输出 |
| `KinematicsSnapshot` | 独立拥有显式回读后的宿主结果 |

计划创建时检查模型与qpos0。
它只上传一次模型参数。
它只编译一次每组内核。
一个计划可以创建多组状态。
每组状态拥有独立输出。
状态默认复制模型qpos0。
初始派生结果保持未就绪。

状态记录具体计划的模型身份。
同尺寸或同内容不代表同模型。
跨计划调用返回ModelMismatch。
这个检查先于输出失效。

计划与缓冲各自保留会话。
会话句柄提前释放仍保持安全。
计划释放后仍能读取完成结果。
本批不公开裸设备指针。

## 实际调用

```rust,no_run
use mjwarp_rs::diagnostics::TransferError;
use mjwarp_rs::model::AttachedModelInput;
use mjwarp_rs::physics::KinematicsPlan;
use mjwarp_rs::runtime::TransferSession;

fn update(model: AttachedModelInput, qpos: &[f32]) -> Result<(), TransferError> {
    let session = TransferSession::new(0)?;
    let plan = KinematicsPlan::new(&session, model)?;
    let mut data = plan.create_data(64)?;
    data.write_qpos(qpos)?;
    plan.update(&mut data)?;
    let result = data.readback()?;
    let _ = result.rigid().world(0)?;
    let _ = result.attached().world(0)?;
    let _ = result.com().world(0)?;
    Ok(())
}
```

完整qpos采用世界优先布局。
局部写入只修改指定世界。
它不会修改其他世界状态。
成功写入废弃全部派生结果。
输入错误保留已有设备结果。
设备写入失败保留失效标记。

独立更新入口如下：

```text
update_rigid
update_attached
update_com
```

刚体更新废弃另外两组结果。
后两组要求刚体已经就绪。
它们直接读取刚体设备缓冲。
组合update依次执行三组。
更新不申请新的设备输出。
更新不上传模型或编译内核。
阶段之间不强制宿主回读。
本批仍同步等待每次内核。

readback要求三组全部就绪。
它检查全部守卫与有限性。
它失败时不发布部分快照。
旧宿主快照不借用设备状态。
后续更新不会修改旧快照。

更新成功只证明驱动执行成功。
数值溢出仍可能产生非有限值。
显式回读随后报告数值错误。
数值错误不隔离健康会话。
后续合法输入可以继续执行。

## 输入与等价边界

本批保留既有严格检查。
模型旋转与轴向要求单位化。
世界体要求规范零质量惯量。
qpos0也须符合状态检查。
自由与球关节检查状态四元数。
GPU负责归一化合法状态姿态。
公开输入与输出保持f32。
旧质量求解继续保留内部f64。

本批没有实现以下能力：

- 字段级逐世界或周期参数。
- mocap及其子树更新。
- 相机与光源位姿。
- 静态geom初始化缓存语义。
- 柔性体、肌腱与休眠唤醒。
- 完整Model与Data字段契约。
- 等价导入、重置与快照入口。
- 生产异步调度与执行图。
- Linux与清洁部署验收。

几何输出仍每次全部重算。
本批不替代等价低层入口。
就绪检查只约束此严格辅助。
它不改变上游调用方责任。
组合update不等于完整运动学阶段。

## 参考与验收入口

测试读取三组静态原生参考。
它们分别覆盖刚体、质心与附着。
测试核对三组清单的全部哈希。
参考采用MuJoCo原生候选。
候选版本为3.12.0。
候选不改变冻结Warp基线。
测试不生成原生参考或编译模型。
测试不启动Python。

误差容限继续固定如下：

```text
atol = 2e-5
rtol = 2e-5
```

零质量边界继续遵循冻结Warp。
原生CPU参考保留其不同结果。
测试不冒称此边界CPU等价。

执行以下脚本：

```powershell
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
```

脚本检查固定用例数量。
零用例不能通过验收。
它保存GPU、驱动及动态库摘要。
它记录运行版本与工作树状态。
结果位于target/resident-kinematics。

## 实际验证

本轮实际执行两次验收脚本。
两次都启用全部feature。

| 配置 | 常驻GPU用例 | 内核ABI用例 | 哈希用例 | 世界比较 | 最大绝对误差 |
| --- | --- | --- | --- | --- | --- |
| 调试 | 6 | 3 | 1 | 1245 | 4.158170539447781e-6 |
| 发布 | 6 | 3 | 1 | 1245 | 4.158170539447781e-6 |

比较次数包含重复更新与回读。
它不代表1245种独立模型。
测试覆盖14份静态参考。
世界规模包含1、257及513。
数值误差没有超过原有容限。

| 环境 | 实际记录 |
| --- | --- |
| 系统 | Windows x86_64 MSVC；10.0.26200.0 |
| Rust | rustc 1.99.0 |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER |
| 驱动 | 596.36 |
| NVRTC | CUDA 12.8.1探针工具链 |
| 参考候选 | MuJoCo 3.12.0；只读静态参考 |

脚本保留提交前工作树记录。
基准提交为5e2e2cf。
两个报告均标记工作树有改动。

```text
target/resident-kinematics/4139966d49e24493a28379b0e62a951a/report.json
target/resident-kinematics/76127fd5bbc34427ba934c95bc03a32d/report.json
```

默认Rust测试通过149项。
文档测试另通过7项。
GNU默认与CUDA检查通过。
两组clippy都拒绝任何警告。
格式检查也通过。

全部feature回归取得以下结果：

| 配置 | 主机测试 | 文档测试 | 默认忽略 | 显式测试 |
| --- | --- | --- | --- | --- |
| 调试 | 178通过 | 7通过 | 85 | 85通过 |
| 发布 | 178通过 | 7通过 | 85 | 85通过 |

两版最终都没有失败用例。
显式用例包含GPU及原生边界。
它们不代表完整G01验收。
旧f64求解与原生生命周期也通过。

实际回归命令如下：

```powershell
scripts/test-windows-native.ps1 -AllFeatures
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --features cuda-probe --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets --release -- -D warnings
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
cargo fmt --check
```

全量回归先初始化原生测试环境。
本轮另行纠正环境路径导入。
脚本自身不负责全量环境初始化。
Linux与清洁部署仍未执行。

本轮还核对三个更新路径。
它们不调用上传、编译或回读。
文档逐项核对新增API与状态。
本轮未运行Python审计工具。

### 已纠正的环境尝试

首次全量运行缺少原生测试环境。
原生测试出现5项失败。
环境导入出现Path大小写冲突。
后续环境缺少NVRTC路径。
内核相关测试出现5项失败。
本轮显式保留MSVC工具链路径。
再运行原生脚本创建测试DLL。
原生22项测试随后通过。
本轮没有修改这些测试断言。
最终全量重跑没有剩余失败。

重跑日志保存于以下目录：

```text
target/resident-kinematics/verification/
```

架构预检与源码校验均无错误。
本轮只激活常驻辅助叶子。
完整物理节点继续保持计划态。
提交前保留一项临时协作警告。
本轮关闭已验证的编码变更。
首次提交遇到索引权限限制。
当时Git无法创建index.lock。
当时本轮未完成智能提交。
当时工作树保留全部改动。
后续提交请查Git历史。

## 后续顺序

先补字段级批量参数布局。
再接入mocap与静态初始化。
随后实现相机与光源。
最后收口等价入口与完整验收。
完整G01继续保持待办。
