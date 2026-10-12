# T4逐字段打包与质心内部量

日期：2026-10-12。
状态：本轮静态切片通过。
完整精度验收仍未完成。

## 本轮范围

本轮沿原排程串行推进。
本轮不修改产品物理源码。
原参考、容差和覆盖不变。
本轮只运行新增宿主用例。
本轮不重跑满额验证。

| 项目 | 本轮结果 |
| --- | --- |
| 独立批量参数 | 23项 |
| 打包函数 | `pack_model`与`pack_camlight` |
| 质心内部计算 | 15组映射 |
| 产品来源 | 六份源码及换行归一化哈希 |
| 上游计算来源 | 冻结上游模块`smooth.py` |
| 本库既有内核产物 | 双模式合计20份 |
| 其中批量变体 | 14份，单模式各覆盖23项 |
| 新增宿主测试 | 5过、0败、0忽略 |
| 旧宿主测试 | 13项过滤，本轮不运行 |
| 图形处理器(GPU)执行 | 零次 |
| 稳态性能实测 | 未执行 |

清单见[字段与内部量](../fixtures/precision/g01-field-internal.json)。
回执见[静态实核](../fixtures/precision/g01-field-internal-audit.json)。
实现见[新增检查器](../tests/support/t4_field_internal.rs)。

产品源码基线如下：

```text
78601790d2d09d8d08a93ba6876b388f1ca43174
```

冻结上游基线如下：

```text
71da24d956378a87a703b6e1442b13aec0c4ac29
```

## 逐字段打包

字段分母来自公开枚举。
设备顺序来自分组常量。
相机光源采用枚举顺序。
共享行与覆盖行均保持f32。
打包器按原位复制参数。
每字段占两个整数描述槽。
首槽记录参数标量偏移。
次槽记录独立周期。
设备按世界编号取模。
设备指针运算使用宽整数。
宽整数不提升物理浮点精度。

下表序号从零开始。
描述槽偏移等于序号乘二。
宽度采用设备内核符号。

| 字段 | 分组 | 序号 | 读取变量 | 每世界宽度 |
| --- | --- | --- | --- | --- |
| `qpos0` | `rigid` | 0 | `q0` | `nq` |
| `body_pos` | `rigid` | 1 | `bp` | `3*nb` |
| `body_quat` | `rigid` | 2 | `bq` | `4*nb` |
| `body_ipos` | `rigid` | 3 | `ip` | `3*nb` |
| `body_iquat` | `rigid` | 4 | `iq` | `4*nb` |
| `jnt_pos` | `rigid` | 5 | `jp` | `3*nj` |
| `jnt_axis` | `rigid` | 6 | `axis` | `3*nj` |
| `body_mass` | `com` | 0 | `mass` | `nb` |
| `body_inertia` | `com` | 1 | `inertia` | `3*nb` |
| `geom_pos` | `attached` | 0 | `gp` | `3*ng` |
| `geom_quat` | `attached` | 1 | `gq` | `4*ng` |
| `site_pos` | `attached` | 2 | `sp` | `3*ns` |
| `site_quat` | `attached` | 3 | `sq` | `4*ns` |
| `cam_pos` | `camlight` | 0 | `cp` | `3*nc` |
| `cam_quat` | `camlight` | 1 | `cq` | `4*nc` |
| `cam_poscom0` | `camlight` | 2 | `cpc` | `3*nc` |
| `cam_pos0` | `camlight` | 3 | `cp0` | `3*nc` |
| `cam_mat0` | `camlight` | 4 | `cmat` | `9*nc` |
| `light_pos` | `camlight` | 5 | `lp` | `3*nl` |
| `light_dir` | `camlight` | 6 | `ld` | `3*nl` |
| `light_poscom0` | `camlight` | 7 | `lpc` | `3*nl` |
| `light_pos0` | `camlight` | 8 | `lp0` | `3*nl` |
| `light_dir0` | `camlight` | 9 | `ld0` | `3*nl` |

检查器逐项核对以下关系：

- 公开枚举与字段名称。
- 共享参数与实际成员。
- 分组顺序与设备描述槽。
- 输入类型与上游原始声明。
- 复制目标与设备读取类型。
- 世界周期与参数偏移。

相机质心参数另走第二入口。
其他相机参数走刚体入口。
本轮没有改变两个入口顺序。
本轮没有执行设备复制测试。
静态来源不证明所有布局等价。
私有拓扑与派生索引仍待细审。
其余八个打包函数仍待展开。

## 质心内部精度映射

本轮审查`rigid_com_position`。
清单固定十五组计算锚点。
它绑定完整模板的源码来源。
上游类型依据冻结类型文件。
空间向量与矩阵均采用f32。
惯量向量也采用f32。
描述符和循环索引保持整数。

| 计算组 | 本库内部量 | 上游来源 | 类型与边界 |
| --- | --- | --- | --- |
| 质量初始化 | `sm[b]` | `_subtree_com_init.body_mass` | f32派生缓存，无直接对应输出 |
| 加权位置 | `com`、`mass`、`ipos` | `_subtree_com_init` | f32乘法与向量存储 |
| 质量累加 | `sm[up]` | `_subtree_div.body_subtreemass` | f32累加，形成路径不同 |
| 位置累加 | `com[3*up+a]` | `_subtree_com_acc` | f32，串行与原子顺序不同 |
| 质量归一化 | `com/sm` | `_subtree_div` | f32除法，零值分支保持 |
| 惯量偏移 | `x,y,z,mb` | `_cinert.dif,mass` | f32局部量 |
| 惯量旋转 | `r,d,t[9],v` | `_cinert.mat,inert,tmp` | f32矩阵，表达与累加顺序不同 |
| 惯量对角 | `c[0..4]` | `_cinert.res` | f32，含首项非对角结果 |
| 惯量非对角 | `c[4..6]` | `_cinert.res` | f32 |
| 质量一阶矩 | `c[6..10]` | `_cinert.res` | f32 |
| 自由度偏移 | `off[3]` | `_cdof.offset` | f32 |
| 自由平移 | `c[0..6]` | `_cdof.res` | f32常量0与1 |
| 自由度轴 | `ax[3]` | `_cdof.xmat,xaxis` | f32 |
| 滑动自由度 | `c[0..6]` | `_cdof.res` | f32 |
| 转动自由度 | `c[0..6]` | `_cdof.res` | f32叉积 |

这里的同型只表示标量一致。
它不表示逐算子语义一致。
本库重新累加子树质量。
上游读取模型中的子树质量。
本库串行累加加权位置。
上游按树层执行原子累加。
矩阵乘法也采用不同表达。
融合乘加可能影响舍入位置。
本轮只登记差异，不改公式。
后续仍须核验生成算子。
本轮不新增等价放行结论。

## 产物实核与限制

本轮读取既有双模式产物。
每模式各核对十份变体。
入口包括刚体、质心、附着。
入口还包括相机光源两段。
源码、PTX和回执各核对哈希。
精度回执须保留f32门禁。
融合乘加保持开启。
快速数学保持关闭。

检查器识别真实批量宏。
仅条件分支文本不计批量覆盖。
非批量变体仍核对产物哈希。
质心变体另匹配完整模板。
本轮不重新编译这些内核。
本轮只读取上游Python文本。
测试不启动Python解释器。
本轮未核验上游生成内核。
本轮未开展Linux验收。

检查器属于静态回归守卫。
它不充当通用语言类型推导器。
人工审查负责符号语义映射。
哈希漂移要求重新审查。
已有正确清单是审查基线。
存在源码锚点不证明语义等价。

## 本轮验证与调用账本

旧目标已累计执行三次。
本轮不重复运行旧用例。
本轮只选择新增命名空间。
该选择包含四项规则测试。
它也包含一项显式资料审计。
全部五项通过，耗时0.99秒。
另外十三项保持过滤状态。
此耗时不属于物理性能成绩。

本轮新增选择只执行一次。
完整T4调用次数保持不变。
旧独立诊断次数保持不变。

以下命令仅记录已执行验证。
它们不构成追加验证授权。

```powershell
$env:MJWARP_FROZEN_UPSTREAM_ROOT = "$PWD/target/mjwarp-parallel/upstream/mujoco_warp"
$env:MJWARP_FROZEN_WARP_ROOT = "$PWD/target/mjwarp-parallel/venv/Lib/site-packages/warp"
$env:MJWARP_T4_EVIDENCE_ROOT = "$PWD/target/t4-g03-retest"
$env:MJWARP_FIELD_INTERNAL_AUDIT_REPORT = "$PWD/target/g01-field-internal-audit.json"
cargo test --locked --features cuda-probe --test t4_input_precision field_internal:: -- --include-ignored --test-threads=1 --nocapture
cargo clippy --locked --features cuda-probe --test t4_input_precision -- -D warnings
cargo fmt --check
git diff --cached --check
```

本机日志：`target/t4-field-internal-tests.log`。
缺少本机资料会阻止显式审计。
默认忽略该资料审计。
默认运行不代表实核全部通过。

## 排程与剩余门禁

| 门禁 | 本轮状态 | 剩余工作 |
| --- | --- | --- |
| NP01 | 23项打包与15组内部量已映射 | 其余打包、拓扑、局部量及完整输出 |
| NP02 | 双模式20份既有产物已核对 | 上游生成内核与工具链完整证据 |
| NP03 | 保留历史数值证据 | 本轮不重跑，不修改容差及覆盖 |
| NP04 | 本轮未实测 | 公平稳态耗时与进程分配峰值 |

原主任务继续保持进行态。
本轮沿用根模块独占锁。
排程不允许运行中改写占用。
本轮保留原占用声明。
本轮通过备注登记新文件。
本轮仅同步本机伴随记录。
模块标识与计划状态保持不变。
架构校验不替代物理验收。
