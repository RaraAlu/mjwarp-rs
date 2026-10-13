# D05数值与性能门槛决议

日期：2026-10-14。
任务：`2026-10-11-roadmap-d05`。
本决议冻结现行判定边界。
它不宣称完成产品验收。

## 范围与前置

本任务没有登记前置任务。
它独占本决议文件。
本轮不修改D04或主T4占用。
本轮保留原模块UID与状态。
本轮不新增依赖或产品逻辑。
本轮不改变任何既有样本。
本轮不改变容差与精度。
本轮不重跑已满额配置。

依据见[精度契约](../numerical-precision-contract.md)、
[现行红线规则](../native-error-redline-acceptance.md)、
[阶段性能要求](../implementation-plan.md#性能采样)。
[旧两级报告](../windows-two-reference-acceptance.md)
仅记录当时机制与结果。
旧值差规则不定义当前准入。

## 四项独立门禁

| 门禁 | 必须满足 | 本决议不提供 |
| --- | --- | --- |
| NP01 | 全范围类型映射100%，精度偏离0项 | 文档映射不能证明实际全路径 |
| NP02 | 生成内核及精度证据覆盖100%，缺失0项 | 模板源码不能顶替实际产物 |
| NP03 | 必测字段块判定覆盖100%，失败与待验收0项 | 原生放行不证明上游等价 |
| NP04 | 公平批量对照记录缺失0项 | 板卡快照不证明进程峰值 |

四门禁须分别报告。
数值通过不能豁免类型偏离。
输出f32不豁免内部f64。
上游显式混合精度须逐项跟随。
常量、累加与数学函数也须核对。
原生ABI可以保留f64。
纯诊断f64不得反馈产品物理。
CUDA三角慢路径单独映射。
它影响物理输出，不属纯诊断。
精度门禁不能简单禁止所有f64指令。
它也不能豁免任意升精度。

冻结上游提交：
`71da24d956378a87a703b6e1442b13aec0c4ac29`。
原生输入ABI仍采用MuJoCo 3.12.0。
历史上游来源声明3.15.0。
双方版本与语义必须分别登记。
版本不同不自动证明输入同义。
不能把零样本算作物理通过。

## 原生优先与误差红线

实际实现见[判定辅助](../../tests/support/two_reference.rs)
及[宿主边界](../../tests/two_reference_policy.rs)。

```text
t_i = atol_native + rtol_native * abs(native_i)
E_candidate = max_i(abs(candidate_i - native_i) / t_i)
E_mjwarp = max_i(abs(mjwarp_i - native_i) / t_i)
redline = 1.05 * E_mjwarp
```

上述5%沿用用户既有批准。
它不改变原生容差。
它不表示输出允许偏移5%。
原生字段块通过即放行该项NP03。
此时无需读取上游第二参考。
原生通过不证明上游算子等价。
只有原生超差才取第二参考。
两路始终使用同一原生基准。
两路始终使用同一容差分母。
字段块须在比较前固定。
不得逐元素择优拼接通过。
不得跨字段、世界或帧借用极值。

| 情况 | 判定 |
| --- | --- |
| 原生误差落在冻结容差内 | 原生放行 |
| 原生超差；本库分数不高于上游 | 对照放行 |
| 原生超差；本库分数不高于红线 | 红线内放行 |
| 原生超差；本库分数高于红线 | 失败不放行 |
| 缺必要原生参考或容差 | 待验收不放行 |
| 原生超差且缺必要上游参考 | 待验收不放行 |
| 本库较差且缺已批准红线 | 待验收不放行 |
| 有限性、身份、形状或条件错误 | 失败不放行 |
| 汇总没有实际数值标量 | 未验收不放行 |

边界采用实现中的小于等于。
零容差且零误差归一为零。
零分母非零误差不能提供有效分数。
分数或红线溢出不能放行。
大上游误差也会增大红线。
独立物理不变量仍须通过。

判定辅助不验证完整模型。
调用方核模型、状态与拓扑。
调用方核批量、世界与帧。
各自参考须核各自冻结摘要。
共同输入与量化规则须一致。
两路输出摘要不要求相同。
缺同义字段不能强行比较。
报告采用`native_error_v2`。
汇总采用schema 2。
`mjwarp_passed`合计两类二级放行。
细项必须保留各自原因。
汇总仅覆盖实际记录的字段块。

## 既有逐族容差

定义A为：
`2e-5 + 2e-5 * abs(expected)`。
定义S为：
`2e-4 + 2e-4 * abs(expected)`。
两者均沿用现有测试。
下表不创造通用默认容差。
独立断言继续采用原比较符。
表中A不覆盖全部解析断言。

| 测试族 | 既有浮点门槛 | 实际来源与独立边界 |
| --- | --- | --- |
| 刚体运动学 | A | [kinematics_probe](../../tests/kinematics_probe.rs)；保留布局与输入拒绝 |
| 附着位姿 | A | [attached_kinematics_probe](../../tests/attached_kinematics_probe.rs)；保留geom/site覆盖 |
| 质心与自由度 | A | [com_position_probe](../../tests/com_position_probe.rs)；独立解析断言保留 |
| 质量矩阵 | A | [mass_matrix_probe](../../tests/mass_matrix_probe.rs)；对称与能量另检 |
| 质量分解与求解 | S | [mass_solve_probe](../../tests/mass_solve_probe.rs)；残差与重构不可回退 |
| 常驻刚体 | A | [resident_kinematics](../../tests/resident_kinematics.rs)；快照与生命周期另检 |
| 参数周期 | A | [resident_parameters](../../tests/resident_parameters.rs)；独立批量周期保留 |
| mocap与静态几何 | A | [resident_mocap](../../tests/resident_mocap.rs)；原解析阈值保留 |
| 相机与光源 | A | [resident_camlight](../../tests/resident_camlight.rs)；模式与周期保留 |
| 固定肌腱 | A | [resident_fixed_tendon](../../tests/resident_fixed_tendon.rs)；稀疏行与拓扑另检 |
| 空间site与pulley | A | [resident_spatial_tendon](../../tests/resident_spatial_tendon.rs)；局部编号保留 |
| 球柱与内侧绕行 | A；超差走现行红线 | [resident_geom_tendon](../../tests/resident_geom_tendon.rs)；世界与运动帧身份固定 |
| 混合肌腱 | A | [resident_tendon](../../tests/resident_tendon.rs)；原生全局编号另检 |
| 柔体位置 | A | [resident_flex_position](../../tests/resident_flex_position.rs)；节点与插值域保持 |
| 柔体边 | A | [resident_flex_edge](../../tests/resident_flex_edge.rs)；原1e-6解析断言不变 |
| 柔体面位置 | A | [resident_flex_face](../../tests/resident_flex_face.rs)；线性壳体范围不扩大 |
| 柔体面四元数 | 符号对齐后A | 同文件；单位范数误差严格小于2e-6 |
| 柔体21系数矩阵 | A | [stretch辅助](../../tests/support/flex_stretch_reference.rs)及[常驻测试](../../tests/resident_flex_stretch.rs)；缓存位型与转置另检 |
| 肌腱唤醒 | 长度A；树状态精确相等 | [resident_tendon_wake](../../tests/resident_tendon_wake.rs)；轻量身体计数为零 |
| 空间速度 | A | [resident_com_velocity](../../tests/resident_com_velocity.rs)；完整G02仍缺 |
| RNE偏置力子集 | A | [resident_rne_bias](../../tests/resident_rne_bias.rs)；不能替代完整动力学 |
| 完整G01与低层 | A | [g01](../../tests/g01.rs)；四元数按现有符号等价比较 |
| 原生G01输入 | 不另设浮点门槛 | [native_g01](../../tests/native_g01.rs)复用现有入口及字段校验 |
| 原生柔体输入 | A | [native_flex_position](../../tests/native_flex_position.rs)；原生转换与GPU输出分开 |

S只作用于已有求解字段比较。
求解归一残差不超过2e-5。
归一重构误差不超过2e-6。
具体归一方式保留原源码。
零分母分支保留绝对误差。
逆对角关系等独立断言不变。
红线不能替代这些不变量。

四元数只能按既有规则对齐。
不能逐分量选择正负号。
整数、索引、计数不走浮点回退。
缓存、位型与哨兵保持原断言。
快照所有权与失败恢复另检。
共享参数及独立批量覆盖不减。
本清单不把21系数扩成24系数。

通用GPU探针另采用其原规则。
整数与指定浮点样本精确比较。
小型求解采用`1e-6*(1+abs(expected))`。
其独立残差检查仍保留。
依据见[运行时样本](../../src/runtime/samples.rs)
与[双路探针](../../tests/cubecl_probe.rs)。
这些辅助规则不是物理族默认容差。

碰撞、约束、完整柔体及渲染仍有缺口。
本决议不为它们补填A或S。
新字段必须先形成同义参考。
它们须先冻结专属比较范围。
尚无门槛的路径继续标记未验收。

## 历史实测与当前产物

[追加验证](../windows-t4-g03-retest.md)
记录2026-10-12双模式结果。
当时每模式产品范围483项通过。
其中包含宿主与文档测试。
当时判定65733块原生放行。
当时另有29块对照放行。
当时失败及待验收均为零。
这些计数不是当前源码复验。
它们不替代完整NP01与NP02。
它们也不代表Tesla T4证据。

[当前状态](../../fixtures/precision/t4-current-capture-status.json)
绑定当前面源码与PTX。
其当前数值标志仍为false。
其当前机器码标志仍为false。
当前位差为null，不是零。
2078仅属于旧源码严格失败。
[算子隔离](../evidence/t4-g03-retest/face-operators/audit.json)
保留该历史失败与原容差结果。
跨设备逐位相等不是通用精度指标。
但本轮不删除已有严格诊断。
容差内结果不冒充严格诊断通过。

当前清单仍覆盖24入口、30变体。
[编译记录](../evidence/t4-g03-retest/face-current-compile/audit.json)
只证明已记录的编译与PTX门禁。
它不证明修复后GPU数值。
当前面数值准入继续开放。
实际Hessian生成产物也仍缺。
空间肌腱串行与原子等价仍待证。

## 性能准入与未定门槛

性能以同条件对照为前提。
双方须匹配以下对照条件：

- 模型与状态字节及量化规则。
- 同义任务、输出与执行范围。
- 世界数、精度与参数配置。
- 设备、驱动及独占条件。
- 精度策略、缓存与执行模式。
- 容量、拓扑及算法分支。
- 预热、同步与计时边界。

不能强行把不同后端编译器写成相同。
工具链与后端差异须单列。
须记录实际版本及生成策略。
须记录影响公平性的编译选项。
双方配置应遵守同一精度契约。
必要差异须单列并解释。
本轮不执行性能采样。

采样前冻结预热轮数与结束条件。
采样前冻结迭代数与统计方法。
本轮没有证据可填这些数值。
采样前冻结普通与图执行路线。
预热结束后再计稳态窗口。
步耗时必须包含必要同步。
上传、冷启动、JIT分别报告。
不能拿测试墙钟当稳态步耗时。

显存须区分以下来源：

- 本进程设备分配及工作区峰值。
- 模块、运行库与驱动开销。
- 其他进程及全板卡占用。
- 冷启动与稳态的不同峰值。

仅有自有分配字节不算进程峰值。
前后板卡快照也不能替代峰值。
报告必须注明采样工具及边界。
记录吞吐、延迟分位数与容量。
宿主开销与下载成本另列。
渲染另冻结相机、分辨率与输出。

既有建议规模为：
`1,64,1024,4096 worlds`。
这些是采样建议，不是通过阈值。
既有G01性能校验另固定：
世界数为1、2、5、513。
每样本固定四次状态更新。
两组规模不能互换或删减。
正确性测试规模继续独立保留。
显存不足须报告容量与未测项。
不得静默缩规模后声明完整通过。

现有计划没有统一加速倍数。
本轮不杜撰耗时或显存上限。
本轮不把“有数字”当性能达标。
NP04要求完整公平记录。
P10/P11仍须冻结适用性能目标。
若目标尚未批准，性能准入仍开放。
即使本轮有局部数据也不关闭它。

[历史资源记录](../windows-t4-g03-retest.md)
明确缺上游同条件稳态对照。
其板卡快照含其他进程。
其冷启动墙钟也含编译与传输。
本决议不升级这些记录的效力。

### 现有性能校验与合成示例

实际校验见[证据辅助](../../tests/support/t4_evidence.rs)。
其函数为\`verify_performance\`。
它要求预热次数大于零。
它要求重复次数大于一。
具体次数仍须写入冻结计划。
各行必须绑定同一计划摘要。
G01边界固定为：
\`resident-g01-complete-no-transfer\`。
同步固定在每样本前后。
计时行必须标记稳态。
精度要求f32、FMA开启。
快速数学保持关闭。
显存记录采用进程分配事件。
记录须声明采集完整。
校验器检查声明与结构。
它不能证明数据确实来自实测。
实际采集仍须提供独立证据。

[性能宿主示例](../../tests/t4_evidence.rs)
明确使用合成真值表。
其中预热10次、重复3次。
耗时为1.0、1.1、0.9毫秒。
显存值为1024与4096字节。
这些都不是实测或正式阈值。
本轮不运行该宿主选择。
本轮不把合成布尔值当实证。

## 调用账本与执行限制

同测试同配置总上限为三次。
失败、终止与环境错误均计次。
改名、拆分或改语言不重置。
新增代码也不恢复满额配置。
成功后不得无故再次验证。
缺证据不构成绕过额度的理由。
任何新增采集须先核同目的账本。

旧追加验证曾记录特批额外调用。
本决议保留该历史实际次数。
它不把4、5、6次改写成3次。
历史特批不自动授权新追加。

| 目的 | 当前约束与证据 |
| --- | --- |
| 原完整G01/T4及原输入精度 | 已满额，禁止重跑；沿原总账 |
| 原面数值对照 | 3次满额；[面诊断](../evidence/t4-g03-retest/face-polar/audit.json) |
| 算子因子、轨迹与归档 | 各3次满额；[算子账本](../evidence/t4-g03-retest/face-operators/audit.json) |
| 原显式编译选择 | 3次满额；[当前编译账本](../evidence/t4-g03-retest/face-current-compile/audit.json) |
| 面融合宿主选择 | 旧切片记2次；后续累计3次，禁止回退；同上 |
| Hessian、机器捕获、静态图等 | 沿各自既有满额总账；本轮不执行 |
| 静态GPU参考门禁 | 同目的3次满额；[门禁账本](../evidence/t4-g03-retest/d04-reference-inventory.json) |
| G08输入制作 | 累计2次已成功；[输入账本](../evidence/t4-g03-retest/g08-reference-input.json)；不重复 |
| G08原生接触诊断 | 累计1次已成功；[接触账本](../evidence/t4-g03-retest/g08-native-contacts.json)；不重复 |

未列出不代表调用次数为零。
当前面缺证据不允许重跑旧选择。
本轮只读账本与相关产物状态。
本轮不调用上述任何执行入口。

## 本批验收与后续实质工作

本轮只执行静态文档核对。
本轮不运行Rust物理测试。
本轮不运行GPU、Python或编译捕获。
本轮保留全部旧测试源码。
下表固定31份相关源码身份。
归一化只统一CRLF为LF。
它不改源码或浮点语义。

D05完成门槛决议及证据分类。
它不关闭任一产品物理缺口。
NP01至NP04整体仍未完成。
P0至P11也仍需实际实现。

后续回到原D04参考工作。
实质缺口是冻结上游GPU接触输出。
已有MJB与七f32状态可复用。
已有原生接触只能作诊断。
采集前须明确生成产物来源。
不得改名重跑满额参考门禁。
后续仍沿原主任务源码占用。
不得另建替代架构或竞争任务。

| 源码 | SHA-256，LF规范 |
| --- | --- |
| [tests/kinematics_probe.rs](../../tests/kinematics_probe.rs) | `02f0d183c49dd4d70f5a0346c93e21d0979c7d8c36339784e2b6a2d1fa216e15` |
| [tests/attached_kinematics_probe.rs](../../tests/attached_kinematics_probe.rs) | `84cbefecf1f690d5093b6ba0157cab014c78591e76d94a840418e871e582a695` |
| [tests/com_position_probe.rs](../../tests/com_position_probe.rs) | `917fe9cbcda649035cec7d8dce57e9979e51730d80fb71627f6784981ca2a066` |
| [tests/mass_matrix_probe.rs](../../tests/mass_matrix_probe.rs) | `b1d6694f5461aa570e9cdf12e111dafb67f777694e810a30bb5424e181e09db9` |
| [tests/mass_solve_probe.rs](../../tests/mass_solve_probe.rs) | `42426f2c91d10f5d84dff50151966c259ffad64f1d63dcb677ec80d0f0ab8bba` |
| [tests/resident_kinematics.rs](../../tests/resident_kinematics.rs) | `7f7a8750ed3c884421f520ae95da7650eb934092a7ad573a3719a4ba653b25a8` |
| [tests/resident_parameters.rs](../../tests/resident_parameters.rs) | `a083848e1b0ba1e8ec3bfe95404ed2811a60d98a72163fa8b556bd3f7c0ab113` |
| [tests/resident_mocap.rs](../../tests/resident_mocap.rs) | `1a60dd5f15e68f0ae76c4203b62244564edb40b4ae60038f8a97e7ce93458a9a` |
| [tests/resident_camlight.rs](../../tests/resident_camlight.rs) | `0ffa975491727dd55c26d7b931af7ba1ea70b6ee2870f6c6d32f9a4c79dea5c2` |
| [tests/resident_fixed_tendon.rs](../../tests/resident_fixed_tendon.rs) | `6a4644299700a222d1903c738c563ae28232f9569eade71d1bd9fe39d6cf070f` |
| [tests/resident_spatial_tendon.rs](../../tests/resident_spatial_tendon.rs) | `fec0d10eab97eed7f36d7c3afee68987a78e16a3b22b0468efa4adc1ec1f48be` |
| [tests/resident_geom_tendon.rs](../../tests/resident_geom_tendon.rs) | `8b137e652e91f92866542aaabfc0b87f29e7d378b201f10fbe66d12e5fbb3c36` |
| [tests/resident_tendon.rs](../../tests/resident_tendon.rs) | `6e9fb104a5b448aeacf0859f6436492582d9f356201a4c7388d3dc4127bb6701` |
| [tests/resident_flex_position.rs](../../tests/resident_flex_position.rs) | `a9b8ba0579199b18dd6bfcafb258a63f4bfc21e567d147ba2e4715142f072e28` |
| [tests/resident_flex_edge.rs](../../tests/resident_flex_edge.rs) | `444ace07554ebba1d11ed57e46b710f41f891d0a8838501bf02aafc966bb179d` |
| [tests/resident_flex_face.rs](../../tests/resident_flex_face.rs) | `b93285f6db11a9a6376b01e6044b253964b8e835b723bbcfca3c7d0f1c51b73a` |
| [tests/resident_flex_stretch.rs](../../tests/resident_flex_stretch.rs) | `e3e0f7eb4373d40a9f85c1d3e504d2279542667d98d4d9050e888989e2dff8c2` |
| [tests/support/flex_stretch_reference.rs](../../tests/support/flex_stretch_reference.rs) | `331e1425f70e48248ddff0baa0810bf369480f77b22120d5a10b573958d7b08f` |
| [tests/resident_tendon_wake.rs](../../tests/resident_tendon_wake.rs) | `de0711c0b462ef938396900b07b3ecbb65143a15358dfc421636bdda8f1c2bd8` |
| [tests/resident_com_velocity.rs](../../tests/resident_com_velocity.rs) | `a82248ddc610b937e791d824473949ec9b40e8a614b4fd3258351fcee9138f33` |
| [tests/resident_rne_bias.rs](../../tests/resident_rne_bias.rs) | `0358b76ee56beccde231fb03e0d56baca0ae78b86c9474f9e20d9e7b2cd160f0` |
| [tests/g01.rs](../../tests/g01.rs) | `8f25a8299df937e940bc566b45bec739686855f401bb322b577b8746b7740385` |
| [tests/native_g01.rs](../../tests/native_g01.rs) | `ab264d96450d8a059cb9ee66f5cbae63fa28b79cf6c46481a3a57978979fb395` |
| [tests/native_flex_position.rs](../../tests/native_flex_position.rs) | `7fa8d2bcd32d484811a018eb35f442a9ebb862861d33674b250f0803b01895c7` |
| [tests/support/two_reference.rs](../../tests/support/two_reference.rs) | `2277300f68b4d2f00ffc2522c95cc5b790c1c1bb5d43d825b9b39ae516946568` |
| [tests/two_reference_policy.rs](../../tests/two_reference_policy.rs) | `2bf29e2797381522178ff55d833e60aff2ea99b189469c64380a0cf1add827b5` |
| [src/runtime/samples.rs](../../src/runtime/samples.rs) | `1666a23776b084495bd1e6b56f2830d03671c4afa668daf513c8b9ee1de8d20a` |
| [tests/cubecl_probe.rs](../../tests/cubecl_probe.rs) | `3cc5b15dbde308356a4511d0001262ff969dda9058a204adae6eb681c6d4c215` |
| [tests/gpu_probe.rs](../../tests/gpu_probe.rs) | `91a49aaad36ca4fb26a690a9d8228d912171a4f6f0c64812753726b8ed61bee6` |
| [tests/t4_evidence.rs](../../tests/t4_evidence.rs) | `1da9dcbe6cfc7b3427f28243e6d8b5a143d68fa46ec51a0aa550d2ce5b36ef33` |
| [tests/support/t4_evidence.rs](../../tests/support/t4_evidence.rs) | `444458174d8ccd34ba1814ebc3d05ce9f3027e7f777ea8fbe783d154ab29e6bc` |
