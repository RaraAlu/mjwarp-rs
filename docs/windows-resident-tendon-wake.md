# Windows肌腱唤醒子集

日期：2026-10-07。
范围：G01严格辅助增量。
完整G01仍待实现。
完整G25也仍待实现。

## 实现范围

GPU复用常驻全局肌腱长度。
限位同时支持上下界。
限位条件采用严格小于。
范围与边距各用独立周期。
拓扑与限位开关仍共享。
路径保留原生全局肌腱编号。
关节、site与球柱关联动态树。
静态体与滑轮不关联动态树。
源树活动标记必须为一。
限位激活后唤醒路径睡眠树。
唤醒同时遍历其睡眠循环。
完全唤醒值固定为负十一。
更小源计数继续向循环传播。
原活动标记贯穿整个唤醒扫描。
更新末尾才刷新树活动标记。
新唤醒树不立即级联。
轻量刷新重算树活动计数。
它清零身体与自由度计数。
它不更新身体或自由度活动表。

| 接口 | 职责 |
| --- | --- |
| `TendonWakeFields` | 共享树与独立限位参数 |
| `TendonWakeModelInput` | 检查肌腱唤醒模型 |
| `SleepTreeState` | 写入树循环与初始计数 |
| `with_tendon_wake()` | 创建可选常驻计划 |
| `write_world_sleep()` | 写入单世界树状态 |
| `update_tendon_wake()` | 只更新唤醒副作用 |
| `snapshot.sleep_trees()` | 获取只读树快照 |

新模型可以包含柔体位置。
旧入口仍返回空树快照。
新入口增加两个整数缓冲。
两缓冲分别保存状态与暂存。
每次更新先在GPU复制状态。
随后GPU执行唤醒与树刷新。
完成后交换缓冲所有权。
驱动报错时不交换缓冲。
每个线程独占一个世界。
线程按全局肌腱顺序扫描。
阶段间不回读宿主长度。
更新不上传模型或编译内核。
数据继续保留设备所有者。

```text
payload = 2*ntree + 3
tree_asleep: [world, tree]
tree_awake: [world, tree]
counters: [ntree_awake, nbody_awake, nv_awake]
range: [B_range, tendon, 2]
margin: [B_margin, tendon]
```

## 开关与检查

启用SLEEP且启用ISLAND时更新。
关闭SLEEP时保持原树状态。
禁用ISLAND时也保持原树状态。
零肌腱也保持原状态与计数。
非空肌腱执行轻量刷新。
即使本次没有唤醒也会刷新。

检查器核对原生动态树编号。
身体必须继承动态祖先树。
动态根按体顺序分配编号。
检查器拒绝参数宽度不符。
检查器拒绝倒置范围与负边距。
参数批量要求有限浮点数。
状态写入核对循环与计数容量。
负值代表唤醒倒计数。
非负值代表下一个睡眠树。
每个睡眠树必须属于闭合循环。
检查器拒绝悬链与重复入边。
检查器在写设备前完成检查。
错误写入保留旧结果。

树状态写入只废弃唤醒就绪。
qpos与mocap写入废弃全部就绪。
肌腱重算也废弃唤醒结果。
唤醒更新要求全局肌腱就绪。
计划身份检查拒绝跨模型状态。
回读核对双端整数守卫。
快照独占宿主结果。
后续更新不改变旧快照。

## 原生与冻结语义

参考见[样本说明](../fixtures/tendon-wake/README.md)。
开发者运行原生C++工具。
原生版本固定为3.12.0。
产品测试不生成原生参考。
原生完整刷新更新身体计数。
冻结Warp轻量刷新清零该计数。
本子集采用冻结Warp语义。
测试逐项比较原生树状态。
测试另查冻结轻量计数。
原生睡眠过滤会清零部分长度。
本子集沿用全部肌腱计算。
测试另查无过滤原生长度。
测试不宣称两种完整入口等价。

## 验证

平台采用Windows x86_64。
系统版本为10.0.26200.0。
Rust版本为1.99.0。
GPU采用RTX 4070 Ti SUPER。
驱动版本为596.36。
NVRTC版本为CUDA 12.8.1。

| 验证 | 结果 |
| --- | --- |
| GNU默认宿主与文档测试 | 191通过 |
| MSVC全特性，调试 | 236通过 |
| MSVC全特性，发布 | 236通过 |
| 忽略探针，调试 | 134通过 |
| 忽略探针，发布 | 134通过 |
| 常驻验收脚本，调试 | 68通过 |
| 常驻验收脚本，发布 | 68通过 |
| 四路Clippy | 通过，拒绝全部警告 |
| rustfmt与文档构建 | 通过 |
| 原生树状态对照 | 每模式2084组，逐项一致 |
| 重复更新持久性 | 每模式另查2084组 |

默认宿主包含12项文档测试。
全特性宿主也含12项文档测试。
忽略探针含GPU与原生测试。
本批新增五项GPU集成测试。
本批新增一项GPU守卫测试。
常驻脚本含十项宿主测试。
它另含58项GPU测试。
测试另查三项宿主单元边界。
集成文件另含两项宿主检查。
测试覆盖一、二、五与513世界。
测试核对严格边界、循环与开关。
测试核对错误写入与计划身份。
测试核对快照与设备所有权。
测试覆盖零肌腱与零动态树。
测试保留旧柔体与混合入口。

```powershell
cargo test --locked
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features -- --ignored --test-threads=1 --nocapture
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release -- --ignored --test-threads=1 --nocapture
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features --release -- -D warnings
scripts/test-windows-resident-kinematics.ps1 -AllFeatures
scripts/test-windows-resident-kinematics.ps1 -AllFeatures -Release
```

首次全量探针需准备原生模拟DLL。
准备命令沿用原生验收脚本。
本批没有新增依赖。
开发者另做范围内文档检查。
该检查读取十一个源码文件。
它读取十个文档文件。
它核对127条本地链接。
它核对五个参考哈希。
审计钩子提示六处路径。
人工核对确认六处均为误报。
上游路径不属于本仓库。
三处PTX引用实际存在。
两处链接库来自配置目录。
本批不修改无关历史文档。

完整命令与日志保存在下列目录。

```text
target/tendon-wake-verification/
```

## 剩余边界

本轮不推进自动休眠计数。
本轮不生成岛或身体活动表。
本轮不实现接触与等式唤醒。
本轮不实现扰动与外力唤醒。
本轮不冻结完整选项校验。
世界内扫描采用确定性串行。
它不对标多肌腱并发写入顺序。
完整flex边与弹性仍待实现。
完整原生字段转换仍待实现。
正式阶段入口与ABI仍待冻结。
Linux真实验收仍待执行。
完整G01与G25保持计划状态。

## 后续增量

柔体边现已落地辅助子集。
本报告保留当时的验证数量。
当前范围见[边报告](windows-resident-flex-edge.md)。

[冻结阶段入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py)。
[冻结休眠源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sleep.py)。
