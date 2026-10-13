# T4其余打包器与固定肌腱内部量

日期：2026-10-12。
状态：本轮静态切片通过。
完整T4仍未验收。

后续已补齐刚体相机整数拓扑。
同时映射附着相机内部量及输出。
见[拓扑与位姿](t4-topology-frames.md)。
下文保留本轮历史证据。
10月13日续补柔体内部量。
见[柔体后续切片](#柔体后续切片)。

## 本轮落实

本轮继续沿原排程推进。
本轮不改产品物理源码。
原容差、参考与覆盖不变。
本轮不重跑满额验证。
本轮只执行新增用例。

| 项目 | 本轮结果 |
| --- | --- |
| 其余打包器 | 八个 |
| 字段与派生段路由 | 82条 |
| 去重上游字段 | 56项 |
| 产品来源 | 15份源码 |
| 固定肌腱内部量 | 四组 |
| 双模式既有产物 | 16份源码及对应产物 |
| 新增宿主测试 | 两次均5过、0败、0忽略 |
| 旧宿主用例 | 18项过滤，本轮未运行 |
| 产品内核执行 | 零次 |
| 真实性能实测 | 未执行 |

清单见[打包路由](../fixtures/precision/g01-packing-routes.json)。
回执见[本轮实核](../fixtures/precision/g01-packing-routes-audit.json)。
检查器见[路由测试](../tests/support/t4_packing_routes.rs)。
前轮见[批量参数与质心](t4-field-internal.md)。

产品基线如下：

```text
9db581d509adbaf487cbe0498e0b7c2b9a5caba1
```

冻结上游基线如下：

```text
71da24d956378a87a703b6e1442b13aec0c4ac29
```

## 八个打包器

| 函数 | 路由数 | 主要内容 |
| --- | --- | --- |
| `pack_flex_hessian` | 17 | 单元拓扑、启用标记、原长与刚度 |
| `pack_flex_faces` | 4 | 法向轴、节点与占位 |
| `pack_sleep` | 9 | 树映射、限位、范围与边距 |
| `pack_flex_edges` | 10 | 端点、稀疏行、所属体与根体 |
| `pack_flex` | 13 | 插值元数据、节点与顶点 |
| `pack_tendon` | 7 | 类型、局部编号与全局稀疏行 |
| `pack_fixed_tendon` | 6 | 关节地址、系数与力臂槽 |
| `pack_spatial_tendon` | 16 | 路径、所属体、侧向点与滑轮 |

路由分母不等于物理字段分母。
八条路由各代表一个头部。
头部可能包含多个整数槽。
其余路由包含字段和派生段。
一个字段可供多处路由使用。
检查器独立冻结路由集合。
它也冻结每条上游字段集合。
删除派生字段引用同样失败。

每条路由记录以下证据：

- 宿主写入表达式。
- 设备读取表达式。
- 存储标量类型。
- 上游字段与原始声明。
- 直接复制或派生类别。

整份源码哈希保护上下文。
局部锚点帮助人工追踪。
格式守卫不替代语义分析。
清单不宣称完整布局等价。
本轮也未运行设备读写测试。

## 浮点参数

以下十条物理参数路由采用f32。
柔体边打包器只返回整数。
浮点占位不计入物理参数。

| 路由 | 写入方式 | 设备读取 |
| --- | --- | --- |
| Hessian原始边长 | 直接复制 | `p[e]` |
| Hessian刚度 | 直接复制 | `stiffness=p+ne` |
| 柔体顶点 | 直接复制 | `verts=params` |
| 柔体初始坐标 | 直接复制 | `coords=verts+3*nv` |
| 柔体节点 | 直接复制 | `nodes=coords+3*nv` |
| 肌腱范围 | 独立批量复制 | `range` |
| 肌腱边距 | 独立批量复制 | `margin` |
| 固定肌腱系数 | 直接复制 | `coefficients[i]` |
| 空间肌腱滑轮系数 | f32倒数预计算 | `scales[i]` |
| 几何尺寸 | 独立批量复制 | `scales`中的尺寸区 |

滑轮预计算不会复制原始除数。
侧向点编号也使用包裹参数。
它先执行f32舍入，再转i32。
这两条路径不能混为直接复制。
本轮没有证明全部舍入语义。
后续仍须核对上游预处理链。

范围与边距各保留独立周期。
几何尺寸也保留独立周期。
本轮未改变任何批量参数。

## 私有整数与派生段

| 路由 | 形成方式 | 审查边界 |
| --- | --- | --- |
| 柔体居中 | 布尔编码成i32 | 上游字段采用布尔 |
| 肌腱限位 | 布尔编码成i32 | 上游字段采用整数 |
| Hessian启用 | 插值、刚性、维数和刚度共同判定 | 不是上游单一字段 |
| 唤醒门控 | 合并休眠与孤岛开关 | 不是完整选项字段 |
| 柔体端点 | 叠加局部编号与顶点起点 | 不是原始边数组直接别名 |
| 根体编号 | 逐体遍历父节点 | 不是原模型根体字段直接复制 |
| 自由度地址及数目 | 从关节元数据推导 | 保留i32元数据 |
| 力臂写入槽 | 搜索稀疏列后生成 | 不冒充关节编号 |
| 包裹所属体 | 按几何或站点选择 | 不冒充包裹对象编号 |
| 包裹所属树 | 从所属体查询树编号 | 私有派生数组 |
| 全局肌腱合并 | 固定与空间子集局部映射 | 保留原生全局顺序 |

`flex_elemadr`占一个整数槽。
当前Hessian内核不读取该槽。
清单明确标注未使用状态。
检查器拒绝新增读取而不复审。
它也区分`r[6]`与`upper[6]`。
标识符子串不能冒充槽位读取。

面打包器保留一个浮点占位。
混合肌腱打包器也保留占位。
休眠空参数使用零值填充。
这些值不构成物理输入字段。

## 固定肌腱内部量

本轮映射四组实际表达式。
本库与上游都使用f32标量。
它们没有采用相同累加调度。

| 本库表达式 | 上游对应量 | 边界 |
| --- | --- | --- |
| `float sum=0.0f` | 长度输出的累加量 | 本库使用局部标量 |
| `coefficients[i]*qpos[qadr[i]]` | `L=prm*qpos` | f32乘法 |
| `jac[jslot[i]]=coefficients[i]` | 力臂写入`prm` | f32存储 |
| `length[t]=sum` | 长度原子累加 | 调度与舍入位置仍可能不同 |

本库每世界串行遍历肌腱。
上游按包裹对象执行原子累加。
因此同型不证明逐位一致。
本轮没有修改计算公式。
本轮没有新增数值放行证据。

## 来源与产物

检查器核对15份产品源码。
它也读取冻结上游类型文件。
固定肌腱另核对上游计算源码。
上游哈希沿用已有冻结清单。
测试只读取Python文本。
测试不启动Python解释器。

每个模式各核对八份产物。
每个打包器选一个读取入口。
空间入口的源码含相关辅助核。
混合入口源码也含索引合并核。
检查器匹配完整已审模板。
它不把源码存在当成运行证据。

源码、PTX和回执分别核对哈希。
每份回执必须通过原精度门禁。
融合乘加保持开启。
快速数学保持关闭。
本轮没有新编译或执行内核。
本轮没有核验上游生成产物。

## 验证与调用账本

本轮复用既有宿主测试目标。
只选择新增命名空间。
首次五项通过，耗时0.88秒。
随后补齐上游引用缺项负例。
第二次仍五项通过，耗时0.83秒。
该新增选择累计执行两次。
这不增加旧用例的运行次数。

旧十八项全部保持过滤状态。
其中五项来自前轮新增切片。
本轮不宣称旧用例重新通过。
完整T4调用次数保持不变。
这些耗时不属于物理性能成绩。

以下命令仅记录已执行验证。
它们不授权重跑满额目标。

```powershell
$env:MJWARP_FROZEN_UPSTREAM_ROOT = "$PWD/target/mjwarp-parallel/upstream/mujoco_warp"
$env:MJWARP_FROZEN_WARP_ROOT = "$PWD/target/mjwarp-parallel/venv/Lib/site-packages/warp"
$env:MJWARP_T4_EVIDENCE_ROOT = "$PWD/target/t4-g03-retest"
$env:MJWARP_PACKING_ROUTES_AUDIT_REPORT = "$PWD/target/g01-packing-routes-audit.json"
cargo test --locked --features cuda-probe --test t4_input_precision packing_routes:: -- --include-ignored --test-threads=1 --nocapture
cargo clippy --locked --features cuda-probe --test t4_input_precision -- -D warnings
cargo fmt --check
git diff --cached --check
```

本机日志保留两份：

- `target/t4-packing-routes-tests.log`。
- `target/t4-packing-routes-tests-complete.log`。

默认运行仍忽略资料审计。
缺少本机资料会阻止显式审计。

## 剩余排程

| 门禁 | 本轮落实 | 仍需补齐 |
| --- | --- | --- |
| NP01 | 八个打包器与固定肌腱内部量 | 刚体相机整数拓扑、其他内部量及输出 |
| NP02 | 双模式16份既有产物 | 上游生成产物与完整工具链证据 |
| NP03 | 保留历史数值证据 | 本轮不重跑，原容差及覆盖不变 |
| NP04 | 本轮未实测 | 真实公平稳态耗时及进程分配峰值 |

本轮仅同步本机伴随记录。
主任务继续保持进行态。
原模块标识与计划状态不变。
本轮不关闭无关历史变更。
架构检查不代替物理验收。

## 柔体后续切片

日期：2026-10-13。
本节单独记录新增证据。
旧打包路由分母保持不变。
生产源码与原参考保持不变。
原容差与全部覆盖保持不变。

### 精度映射

本轮映射16组内部量。
15组物理量使用单精度（f32）。
一组标志使用整数（i32）。
检查器另追踪六项输出。
五项输出使用单精度（f32）。
一项输出采用布尔（bool）。

| 内部组 | 产品表达式 | 上游对应 | 核对边界 |
| --- | --- | --- | --- |
| 节点变换 | `flex_transform` | `_flex_nodes` | 居中与局部全零短路 |
| 直接顶点 | `row[0]==0` | `_flex_vertices` | 矩阵乘加顺序待证 |
| 单元查找 | `coord`转整数 | `ci/cj/ck` | 有限值与整数范围 |
| 局部坐标 | `fminf/fmaxf` | `wp.clamp` | 有限输入域 |
| 三线性权重 | 三项权重乘积 | `eval_basis_trilinear` | `_phi`保持f32 |
| 插值累加 | `p[d]+=weight*nx[...]` | `result += w * ...` | 三重循环顺序一致 |
| 缓存失效 | `flags[f]=0` | `zero_()` | 私有i32适配bool |
| 边向量 | 两顶点相减 | `pos2-pos1` | f32分量 |
| 边长 | `sqrtf`与点积 | `wp.length` | 生成运算顺序待证 |
| 边归一化 | 非零才除法 | `normalize_with_norm` | 零长度保留原向量 |
| 质心偏移 | `off` | `offset1/offset2` | 根体质心索引 |
| 点速度 | `p`与叉积 | `jacp1/jacp2` | 六维速度也采用f32 |
| 雅可比累加 | `j`先减后加 | `J_val` | 端点顺序一致 |
| 边速度 | `vel[e]+=j*qv[d]` | `vel` | 稀疏行顺序一致 |
| 稀疏池初始化 | 全池写零 | 仅写活动行 | 未用尾槽副作用不同 |
| 提前返回 | `continue` | `return` | 速度置零后退出 |

索引和单元编号仍采用i32。
模型检查拒绝非法整数转换。
六维速度别名追到f32定义。
宿主读回保持原标量类型。
检查器不把类型相同当成等价。

| 宿主字段 | 上游字段 | 设备与视图 |
| --- | --- | --- |
| `flexnode_xpos` | 同名 | `nx`到前三倍节点段 |
| `flexvert_xpos` | 同名 | `vx`到后续顶点段 |
| `flexedge_length` | 同名 | `len`到首个边段 |
| `flexedge_velocity` | 同名 | `vel`到第二边段 |
| `flexedge_jacobian` | `flexedge_J` | `jac`到稀疏池段 |
| `flex_hessian_valid` | 同名 | i32检查后转bool |

标志解码只接受零与一。
它不复制上游布尔设备布局。
节点阶段先于顶点插值。
缓存失效先于位置更新。
边阶段随后消费顶点位置。
本库按世界串行遍历。
上游按节点、顶点、边并行。
调度差异仍需生成产物证据。

### 尾槽差异

产品先清零整份雅可比池。
上游边核只写活动稀疏行。
模型允许保留未用尾槽。
因此尾槽清零属于额外副作用。
本轮不推断尾槽数值已等价。
后续须核对上游初始化链。
同时须核对真实入口的可达状态。
本轮不凭源码片段改产品行为。

产品另通过父链判断祖先。
上游读取预计算祖先掩码。
模型检查约束端点自由度并集。
这些证据仍不替代全路径验收。

### 证据与调用账本

检查器核对五份产品源码。
它也核对五份冻结外部源码。
映射见[柔体清单](evidence/t4-g03-retest/flex-internal/map.json)。
结果见[柔体实核](evidence/t4-g03-retest/flex-internal/audit.json)。
本轮使用独立子目录。
旧历史清单不涵盖该子目录。
本轮不改写旧历史清单。

三类入口各核对三份集合。
入口为失效、位置与边。
六份来自旧调试与发布捕获。
三份来自当前增量编译集合。
三份当前记录仍沿用旧产物。
检查器核实完整模板与三类哈希。
它还核对既有精度回执选项。
这些读取不算重新编译或运行。

本轮只选择新增命名空间。
首次四过一败，过滤36项。
失败源于缺少资料目录变量。
第二次补齐变量，五项通过。
第二次耗时0.51秒。
随后补查六维速度标量定义。
第三次五过零败，耗时0.52秒。
该新增选择累计已满三次。
后续不要原配置继续重跑。
格式与静态检查均通过。
旧打包测试正文逐字保持不变。
最终日志为`target/t4-flex-internal-final.log`。
旧36项始终不执行。
所有产品物理内核均不执行。
本轮不启动Python解释器。
本轮不创建GPU会话。
本轮不测稳态性能或显存峰值。

```powershell
$env:MJWARP_FROZEN_UPSTREAM_ROOT = "$PWD/target/mjwarp-parallel/upstream/mujoco_warp"
$env:MJWARP_FROZEN_WARP_ROOT = "$PWD/target/mjwarp-parallel/venv/Lib/site-packages/warp"
$env:MJWARP_T4_EVIDENCE_ROOT = "$PWD/target/t4-g03-retest"
$env:MJWARP_T4_CURRENT_CAPTURE_ROOT = "$PWD/target/t4-rigid-recapture/with-dependents/current"
$env:MJWARP_FLEX_INTERNAL_AUDIT_REPORT = "$PWD/docs/evidence/t4-g03-retest/flex-internal/audit.json"
cargo test --locked --features cuda-probe --test t4_input_precision packing_routes::flex_internal:: -- --include-ignored --test-threads=1
cargo clippy --locked --features cuda-probe --test t4_input_precision -- -D warnings
cargo fmt --check
```

### 排程与验收边界

本轮沿用进行态主任务。
工具拒绝扩展运行态占用。
本轮仅修改已登记文件范围。
证据放入已登记目录前缀。
原依赖与独占资源保持不变。
原模块标识与状态保持不变。
本轮不新占GPU或启动并行任务。

| 门禁 | 本轮结论 |
| --- | --- |
| NP01 | 新增16组与六项输出映射；全路径未完 |
| NP02 | 九份既有集合实核；上游生成运算待证 |
| NP03 | 沿用历史；不重跑满额物理验证 |
| NP04 | 未测公平稳态与进程显存峰值 |

下一步核对柔体面与Hessian。
尾槽副作用单列待审。
完整精度与T4验收继续开放。
