# T4状态与设备打包精度追踪

日期：2026-10-12。
状态：状态与存储类型已核对。
完整T4仍待独立证据。

后续已展开23项批量打包。
质心内部量另登记15组。
见[逐字段与内部量](t4-field-internal.md)。
以下保留本轮历史证据。

## 本轮落实

本轮沿用原伴随主任务。
本轮扩展既有宿主测试目标。
本轮没有新增绕开次数的目标。
产品公式、容差与参考不变。
产品GPU验证次数也不变。

| 核对范围 | 本轮结果 |
| --- | --- |
| G01公开状态 | 十项声明 |
| 休眠状态成员 | 五项声明 |
| 上游对应字段 | 25项去重字段 |
| 模型打包函数 | 十个，含可选Hessian |
| 来源绑定 | 七份产品源码 |
| 布尔与门控读取入口 | 四个内核 |
| 双模式既有产物 | 八份CUDA源码及配套产物 |
| 扩展后宿主目标 | 最终13过、0败、0忽略 |
| 新执行GPU内核 | 零次 |

清单见[状态与打包映射](../fixtures/precision/g01-state-packing.json)。
实核回执见[状态审计](../fixtures/precision/g01-state-audit.json)。
检查器见[宿主追踪测试](../tests/support/t4_state_packing.rs)。
入口报告见[78项模型输入](t4-input-precision.md)。

本轮源码基线：

```text
248a28a7242525d5f86871d4b588e50c1a27693b
```

冻结上游基线：

```text
71da24d956378a87a703b6e1442b13aec0c4ac29
```

## 状态到设备的路径

| 状态 | 宿主标量 | 设备存储 | 说明 |
| --- | --- | --- | --- |
| qpos、mocap位置与四元数 | f32 | 合并状态缓冲f32 | 分段写入，保留位模式 |
| qvel | f32 | 柔体边速度缓冲f32 | 四项前守卫后写入 |
| 刚体缓存 | f32 | 刚体结果缓冲f32 | 展开七项字段 |
| 质心缓存 | f32 | 质心结果缓冲f32 | 含派生子树质量 |
| 世界位姿 | 七项f32 | 刚体缓冲中的世界体位置 | 覆盖xpos与xquat |
| 附着缓存 | f32 | 附着结果缓冲f32 | 展开几何与site四项字段 |
| Hessian有效标记 | bool | 私有i32缓冲 | 明确编码零与一 |
| 休眠五项状态 | i32 | 私有i32缓冲 | 两个数组与三个计数 |

检查器从源码提取状态分母。
它拒绝遗漏、重复与类型错报。
它绑定实际目标缓冲声明。
它核对状态写入调用与类型。
底层暂存保持泛型元素类型。
底层复制不执行浮点量化。
这些结论来自静态源码审查。
本轮不执行设备复制验证。
编译器另核对公开状态类型。

四元数仍采用本库既有顺序。
本轮不证明所有分量布局等价。
世界位姿不是新增上游字段。
它只是世界体字段的别名入口。

## 显式适配与派生缓存

肌腱限位先从u8转为bool。
打包器再把bool编码为i32。
设备端按整数读取限位标记。
上游也以整数存储该标记。
该路径不证明全部限位语义。

两个选项位合成唤醒门控。
条件为启用休眠且未禁用孤岛。
门控采用私有i32元数据。
它不冒充完整上游选项字段。

柔体居中标志也编码为i32。
设备端按非零值转回布尔条件。
Hessian有效标志另用i32槽位。
上游该字段采用bool。
清单保留这项表示差异。
它不宣称两者拥有相同ABI。

质心缓存另含子树质量。
上游从模型读取子树质量。
本库在GPU逐体累加质量。
本库随后用该结果归一化质心。
两路存储标量均为f32。
但它们采用不同的形成路径。
所以清单标注派生缓存。
本轮不将它记成直接复制。
数值与顺序等价仍须独立证明。

## 十个打包器

清单覆盖以下函数：

```text
pack_model
pack_camlight
pack_fixed_tendon
pack_spatial_tendon
pack_tendon
pack_flex
pack_flex_edges
pack_flex_faces
pack_sleep
pack_flex_hessian
```

函数分母来自源码函数集。
元数据统一采用i32。
物理参数存储统一采用f32。
柔体边打包器只返回整数。
面与合并入口保留浮点占位。
空间肌腱另预计算滑轮系数。
该预计算按Rust类型采用f32。
检查器拒绝把它误记为纯复制。

本轮只核对打包存储与已审路径。
本轮尚未逐字段展开全部模型。
十个函数不等于十个等价内核。

## 实际产物与类型来源

显式审计读取冻结上游源码。
它核对25项原始字段声明。
它核对Warp的类型源码。
它另核对十维惯量类型。
空间运动向量也保持f32。
测试只读取Python文本。
测试不启动Python解释器。

四个内核读取入口如下：

```text
tendon_wake
flex_positions
flex_invalidate
flex_hessian
```

审计读取双模式各四份产物。
产物名称采用PTX哈希。
CUDA源码另按源码哈希核验。
审计同时核验PTX及回执哈希。
它不重新编译或执行这些内核。
它不替代上游生成内核审查。

## 调用账本与验证

此宿主目标累计执行三次。
该配置不再追加验证。
首次属于上一轮入口映射。
那次八项全部通过。
本轮扩展到十三项。
第一次十二过、一败。
失败来自检查器的文件定位。
检查器误用源码哈希拼文件名。
本轮修正它，未修改产物。
随后十三项全部通过。
最终宿主测试耗时4.74秒。
这个耗时不属于物理性能证据。

失败日志保留在：
`target/t4-state-packing-tests.log`。
修复日志保留在：
`target/t4-state-packing-tests-fixed.log`。

实际验证命令如下。
命令仅记录证据，不授权重跑。

```powershell
$env:MJWARP_FROZEN_UPSTREAM_ROOT = "$PWD/target/mjwarp-parallel/upstream/mujoco_warp"
$env:MJWARP_FROZEN_WARP_ROOT = "$PWD/target/mjwarp-parallel/venv/Lib/site-packages/warp"
$env:MJWARP_T4_EVIDENCE_ROOT = "$PWD/target/t4-g03-retest"
$env:MJWARP_INPUT_AUDIT_REPORT = "$PWD/target/g01-input-audit-current.json"
$env:MJWARP_STATE_AUDIT_REPORT = "$PWD/target/g01-state-audit.json"
cargo test --locked --features cuda-probe --test t4_input_precision -- --include-ignored --test-threads=1 --nocapture
cargo clippy --locked --features cuda-probe --test t4_input_precision -- -D warnings
cargo fmt --check
```

扩展目标共含两项显式审计。
默认运行会忽略这两项。
只跑默认目标不能冒充实核。

## 后续排程

NP01仍缺全模型逐字段打包。
还缺局部量、累加与完整输出。
其他物理状态路径也须补审。
NP02仍缺上游产物全证据。
NP03保留既有数值验证。
NP04仍缺真实公平性能对照。
完整T4保持进行态。
架构校验不替代产品验收。
