# MJWarp组合与边界清单

整理日期：2026-10-06。
版本：0.1分支草案。
状态：源码核对，尚未实现。

本表展开已核对的入口分支。
本表不宣称覆盖全部内部内核。
本表不关闭P0-03与RV01。
功能族仍沿用原矩阵ID。
SC编号只标识本表的分支组。
后续追加编号，不复用编号。

相关文档：
[模块契约](module-contracts.md)、
[阶段契约](physics-stage-contracts.md)、
[声明盘点](upstream-contract-inventory.md)、
[功能矩阵](compatibility-matrix.md)。

## 1. 迁移规则

拒绝、警告与截断分别保留。
警告不能统一升级为拒绝。
不支持项不能扩大为整族排除。
Rust错误类型可以改变名称。
Rust必须保留错误的触发阶段。
资源安全检查仍保持强制。
物理新鲜度检查仍属于可选门面。

本表记录源码中的判定条件。
注释与报错文本不能替代判定。
调用方更新选项后也需重查组合。
检查不隐式改写积分器或求解器。
检查不静默回退CPU。

## 2. 模型转换分支

以下分支首先作用于put_model。
“警告”表示上游继续转换。
Rust通过诊断报告保留该结果。

| 编号 | 判定条件 | 上游行为 | 矩阵 |
| --- | --- | --- | --- |
| SC01 | batch_sizes键不属于Model首维星号数组；或长度小于1 | 拒绝；不接受Option路径键 | M02、M05、M11 |
| SC02 | 执行器传动、动力、增益、偏置，以及eq、geom、sensor、wrap、sleep枚举不属于对应类型 | 拒绝该枚举值；按冻结类型成员检查 | M02、G05、G22、G23 |
| SC03 | integrator、cone、solver不属于类型；或disable/enable存在未知位 | 拒绝；位掩码按全部已知位的并集检查 | M02、G13～G18 |
| SC04 | SLEEP开启且存在FLEX等式 | 拒绝；不能扩大为全部flex禁用 | G22、G25 |
| SC05 | noslip_iterations大于0 | 拒绝；不排除普通摩擦约束 | G13、G14、G15 |
| SC06 | body_plugin、actuator_plugin或sensor_plugin存在非-1值 | 拒绝对应插件；不排除geom插件和用户SDF | M10、G10、G23 |
| SC07 | 任一flex_interp的绝对值为2 | 拒绝二次插值 | G22 |
| SC08 | flex_interp小于0，bendingadr有效，边数大于0，且flex_damping大于0 | 警告插值壳弯曲阻尼尚不支持；继续转换 | G22、M11 |
| SC09 | 原生mjNPOLY不等于2 | 警告多项式布局可能不兼容；不直接拒绝 | M01、M02、M11 |
| SC10 | nv大于60且原生jacobian显式指定DENSE | 拒绝；不扩大为所有大模型禁用 | G14、G15、M02 |
| SC11 | SLEEP开启且solver不是NEWTON | 拒绝；休眠活动集容量另行检查 | G25、M06 |
| SC12 | 当前condim启用的摩擦分量小于MJ_MINMU | 警告可能产生NaN；geom与pair使用不同索引 | G13、M11 |
| SC13 | nflex大于0且任意geom为SDF或HFIELD | 拒绝模型组合；即使过滤后不碰撞也触发 | G10、G11、G22 |
| SC14 | nflex大于0且任一flex_internal非0或flex_rigid非0 | 拒绝内部碰撞或刚性flex | G22 |
| SC15 | 推导has_flex_snh为true且积分器不是DISCRETE | 拒绝；该标志还依赖维数、插值、刚度地址和元素数 | G18、G22 |
| SC16 | DISCRETE、NEWTON与has_non_simple_flex同时成立 | 拒绝；对应一般附着路径需CG | G15、G18、G22 |

依据：[批量与类型检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L305-L389)、[摩擦检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L406-L420)、[柔性组合与推导](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L498-L550)。

摩擦检查按condim逐层启用。
condim至少3检查滑动摩擦。
condim至少4还检查扭转摩擦。
condim至少6还检查滚动摩擦。
geom索引分别为0、1、2。
pair索引分别为0、1和2、3和4。

### CCD与几何扩展

| 编号 | 判定条件 | 上游行为 | 矩阵 |
| --- | --- | --- | --- |
| SC17 | MULTICCD开启且实际候选对包含不支持多接触的凸几何对 | 警告；这些对最多产生一个接触，不拒绝整个场景 | G09、M11 |
| SC18 | 源码判定has_multiccd_pairs；实际包含的BOX/MESH对有非零margin | MULTICCD开启时拒绝；BOX-BOX且NATIVECCD未禁用时也拒绝；不能推广到所有margin | G09、M02 |
| SC19 | geom插件的零终止数值属性个数大于128 | 拒绝；其余属性转换为float并补零至128；保留geom_plugin_index映射 | M03、M10、G10 |

SC18分别检查几何与显式pair。
几何检查只取实际包含的候选对。
显式pair读取自身margin。
Rust迁移需保留外层门控。
全量几何对清单仍需另外展开。
属性解析失败也需明确报告。

依据：[CCD分支与margin](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L836-L900)、[几何插件属性](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L902-L929)。

## 3. 物理执行分支

设备模型执行组合检查。
forward调用该检查。
step1与inverse也调用它。
step2不自动调用该检查。
低层独立入口不补跑完整forward。
Rust不能暗中改变阶段组合。

| 编号 | 判定条件 | 上游行为 | 矩阵 |
| --- | --- | --- | --- |
| SC20 | 非DISCRETE且has_flex_snh或has_flex_passive为true | 拒绝；passive flex接触依赖有效度量路径 | G18、G22 |
| SC21 | IMPLICIT或IMPLICITFAST，且nefmK大于0或has_non_simple_flex | 拒绝这组柔性弹性组合；不排除普通IMPLICIT动力学 | G17、G22 |
| SC22 | DISCRETE且has_unsupported_flex_interp为true | 拒绝；保留模型构造所得标志，不只看报错文字 | G18、G22 |
| SC23 | DISCRETE、NEWTON，且has_non_simple_flex或非flex_interp_assemblable | 拒绝对应一般附着或不可装配路径；不能默认改用CG | G15、G18、G22 |
| SC24 | DISCRETE与SLEEP开启，同时ISLAND禁用 | 拒绝；不把此条件扩大到所有非DISCRETE组合 | G18、G25 |
| SC25 | DISCRETE与SLEEP开启，且nefmK大于0、non_simple、passive或不可装配任一成立 | 拒绝这组flex组合；不把“有flex”当作唯一判断 | G18、G22、G25 |
| SC26 | inverse需要离散加速度转换，积分器为RK4或IMPLICIT | 转换入口拒绝；Euler、IMPLICITFAST与DISCRETE有各自路径 | G20、G16、G17 |

INVDISCRETE决定逆向转换需求。
DISCRETE也会触发该转换。
普通inverse不默认启用此标志。
RK4完整step仍支持正向积分。
RK4的step2采用Euler路径。
阶段默认值另见阶段契约。

依据：[运行组合检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L56-L83)、[分步入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2101-L2156)、[逆向转换](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/inverse.py#L80-L121)、[逆向调度](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/inverse.py#L178-L234)。

## 4. Data容量与输入导入

共享容量不构成逐世界接触上限。
njmax仍限制逐世界约束。
容量覆盖顺序保持源码语义。
乘法与字节数还需检查Rust溢出。

| 编号 | 判定条件 | 上游行为 | 矩阵 |
| --- | --- | --- | --- |
| SC27 | W小于1；nconmax、njmax或naconmax小于0；nvmax不在`[0, nv]` | make_data与put_data拒绝；允许合法零容量 | M06、G25 |
| SC28 | naccdmax不在`[0, naconmax]`；显式nccdmax不在`[0, nconmax]` | 拒绝；共享CCD容量先按显式总量、逐世界量、缺省量解析 | M06、G09 |
| SC29 | put_data未显式给naconmax且mjd.ncon大于nconmax；或解析后总容量不足mjd.ncon乘W；或njmax小于mjd.nefc | 拒绝导入；显式足够总容量可以绕过逐世界nconmax导入检查 | M06、M07、M11 |
| SC30 | njmax_nnz缺省 | 稀疏路径使用估算函数；稠密路径使用njmax乘nv；显式值不重复套用默认 | M06、G14、G15 |

默认接触容量还使用模型估算。
它不采用固定的全局常数。
nvmax也影响压缩缓冲分配。
qLD包含稠密块与稀疏区域。
不能只用nv平方猜测qLD容量。
具体工作区仍需逐构造核对。

依据：[容量解析顺序](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L1689-L1694)、[Data构造检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L1884-L1968)、[Data导入检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2124-L2162)。

## 5. 渲染上下文

声明rank不代表实际输出rank。
输出容量按启用的相机分别计算。
禁用通道不贡献该通道像素数。
空相机选择仍属于合法输入。
相机选择保留输入顺序。
输出标志按源码条件筛选。
活动数等于原生数时不筛选标志。

| 编号 | 判定条件 | 上游行为 | 矩阵 |
| --- | --- | --- | --- |
| SC31 | cam_active为布尔掩码时长度不等于原生相机数；名称未知；元素类型不支持 | 检查失败；整数ID仍需Rust强制索引安全检查 | R08、M11 |
| SC32 | 分辨率数量不等于活动相机数；或通道标志数量无法匹配 | 检查失败；分辨率支持tuple及单项list广播；通道标志另支持原生长度筛选和标量广播 | R08、R10 |
| SC33 | 缺少splat_position但提供其他splat参数；或位置存在但对应旋转、尺寸、颜色形状不匹配 | 拒绝；位置维度3，旋转4，尺寸3，颜色4；adr为一维，group_ids为`[W]` | R07、R10 |
| SC34 | samples_per_pixel小于1；或大于1但未预计算射线；或大于1且RGB像素数为0 | 拒绝；实际子采样数为samples_per_pixel平方 | R02、R08 |
| SC35 | 输出构造 | RGB为`[W, ri]`的uint32；深度为`[W, di]`的float32；分割为`[W, max(si, 1)]`的vec2i；不能从星号注解推出一维 | R02、R03、R04、R10 |

ri、di、si分别表示通道像素总数。
每相机输出起点使用对应adr。
分割的最小占位不能变成有效图像。
AA工作区按实际采样路径分配。
use_fast_math默认true。
深度缩放不强制有限正数。
严格深度检查另设可选入口。

依据：[splat检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L408-L441)、[相机与通道筛选](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L541-L615)、[采样条件](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L617-L685)、[输出分配](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L751-L772)、[全部默认值](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L312-L342)、[深度公式](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L192-L243)。

## 6. 历史、查询与状态

| 编号 | 判定条件 | 上游行为 | 矩阵 |
| --- | --- | --- | --- |
| SC36 | 历史初始化对象ID越界；或对象没有分配历史样本 | 拒绝初始化；无历史读取仍返回即时值 | G24、M08 |
| SC37 | 控制values不是`[W, S*D]`或`[W, S, D]`；传感values不是`[W, S*D]` | 拒绝形状；控制三维入口不自动推广到传感入口 | G24、M11 |
| SC38 | 显式times不是`[S]`；或任意相邻时间差小于MJ_MINVAL | 拒绝；None保留时间槽；不是简单正差检查 | G24、M11 |
| SC39 | 传感phase数组不是`[W]` | 拒绝；None保留用户槽；标量广播 | G24、M08 |
| SC40 | 射线pnt首维不是1或W、pnt与vec形状不同；rays的bodyexclude或输出形状错误 | 检查失败；rc=None仍保留非BVH路径；无命中dist/geomid为-1 | R01、R06、R10 |
| SC41 | reset世界掩码形状或类型错误；keyframe标量越界 | 拒绝；逐世界keyframe数组中的负值或越界值跳过对应世界 | M08、M11 |
| SC42 | get_data_into遇到共享接触或逐世界约束计数超容量 | 截断到现有容量并继续导出；接触仍按worldid和type筛选；Rust报告截断 | M07、M11 |
| SC43 | run_collision_detection为false | 跳过阶段调度中的碰撞；仍构建已有接触的约束；显式collision仍检查禁用位 | G07、G12、M09 |

历史参数不能只看Optional注解。
times仍属于必传参数。
interp也属于必传参数。
阶段回调仍需保留Stage参数。
局部重置不能清空全部世界。
世界0重置影响共享接触计数。
仅重置其他世界不清零该计数。

依据：[控制初始化](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L970-L1068)、[传感初始化](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L1071-L1184)、[射线形状](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/ray.py#L1179-L1264)、[重置掩码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2920-L2929)、[关键帧检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L3041-L3097)、[计数截断](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2396-L2419)、[局部共享计数](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2751-L2757)、[阶段碰撞调度](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1334-L1365)。

## 7. 剩余关闭条件

每个SC组需配套Rust断言。
测试覆盖触发与不触发两侧。
警告测试检查继续执行的输出。
截断测试检查计数与已导出字段。
未知枚举测试记录原生版本。
变更选项测试核对执行时检查。

以下工作仍未完成：

- 展开全部几何对与接触路径。
- 展开全部资产注册失败边界。
- 展开所有内部工作区构造。
- 展开全部状态签名组合。
- 逐用例迁移上游断言。
- 固定数值证据与误差容限。
- 实测Windows与Linux GPU。

本轮不把清单当作测试结果。
首版验收仍要求完整支持集。
声明清单不替代语义与布局契约。
