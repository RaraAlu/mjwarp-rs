# mjwarp-rs 首版模块契约

整理日期：2026-10-06。
契约版本：0.3细化草案。
状态：源码核对与设计细化。

本文件细化目标架构。
本仓库尚未实现这些接口。
Rust签名仍需原型验证。
本文件不关闭P0或P1。
本文件不证明物理等价。

本版增加三份细化清单。
[声明盘点](upstream-contract-inventory.md)保留原始声明。
[阶段契约](physics-stage-contracts.md)细化调用边界。
[组合清单](supported-combinations.md)登记条件分支。
清单不代替Rust实现与GPU验收。

## 1. 基线与证据

本轮读取冻结版源码。
本轮不运行上游Python。
本轮重新核对公开入口。
入口仍显式导出99个符号。
代码树仍含30个测试模块。
这些数量不代表验收用例数。

```text
repository: google-deepmind/mujoco_warp
revision: 71da24d956378a87a703b6e1442b13aec0c4ac29
contract_version: 0.3-detail-draft
implementation_status: not-implemented
```

| 证据 | 本轮核对内容 |
| --- | --- |
| [公开入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/__init__.py) | 核对公开符号与原始归属 |
| [类型定义](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py) | 核对尺寸、状态与上下文 |
| [模型与状态IO](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py) | 核对转换、容量与重置 |
| [物理调度](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py) | 核对阶段、回调与积分 |
| [常量刷新](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/set_const.py) | 核对模型修改与动力学依赖 |
| [历史状态](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py) | 核对延迟、采样与局部重置 |
| [状态打包](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py) | 核对签名与字段拼接 |
| [公共数学](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py) | 核对四元数与空间向量约定 |
| [渲染BVH](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/bvh.py) | 核对动态加速结构归属 |
| [渲染调度](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render.py) | 核对输出与深度语义 |
| [渲染转换](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py) | 核对相机与像素布局 |

本轮区分两种陈述。
“上游事实”描述源码行为。
“Rust契约”描述本项目设计。
新增安全规则不冒充上游规则。

### 等价入口与可选门面

首版完整对齐GPU引擎语义。
等价入口保留有效调用范围。
它也保留默认值与阶段副作用。
拒绝、警告与截断分别登记。
接口改名不能改变这些行为。

Rust强制资源安全检查。
检查覆盖越界、寿命与竞争。
物理新鲜度检查属于可选门面。
严格票据也属于可选门面。
门面不能替代等价阶段入口。
门面不能暗改默认物理行为。
额外快照也不能替代状态打包。

完整发行包存在范围差异。
边界复审见[复审报告](initial-release-review.md)。

## 2. 模块归属与依赖

以下关系属于首版设计。
箭头表示出向依赖。
它不表示物理执行顺序。

```text
公共库入口 -> io, physics, render
io         -> model, physics, runtime, math, diagnostics
physics    -> model, runtime, math, diagnostics
render     -> model, runtime, math, diagnostics
model      -> runtime, math, diagnostics
runtime    -> diagnostics
math       -> diagnostics
diagnostics -> 无内部模块
```

上层模块组织业务调度。
下层模块不反向调用上层。
公共库入口协调跨模块调用。
所有模块均不依赖LRsLab。

| 模块 | 持有内容 | 禁止承担 |
| --- | --- | --- |
| `model` | 输入规范、设备模型与状态布局 | 不执行动力学调度 |
| `io` | 原生转换、状态交换与恢复流程 | 不编译MJCF或组织场景 |
| `math` | 空间代数与纯数学函数 | 不持有渲染或设备上下文 |
| `runtime` | 设备、缓冲、队列与执行图 | 不解释关节、接触与历史 |
| `physics` | 物理阶段、回调与常量刷新 | 不依赖IO或渲染模块 |
| `render` | 相机、图像、纹理与渲染BVH | 不修改物理动态状态 |
| `diagnostics` | 错误类型、诊断记录与有效性 | 不执行设备或业务操作 |

### 首轮归属修正

本轮保留原有七个顶层模块。
本轮只调整内部职责。

| 原草案 | 首版归属 | 源码依据 |
| --- | --- | --- |
| `runtime::state` | `model::data` | Data解释物理字段与尺寸 |
| `math::bvh`的公开refit | `render::bvh` | refit写入RenderContext |
| `io::history` | `physics::history` | 步进与传感阶段消费历史 |
| `model::constants` | `physics::constants` | 刷新调用smooth动力学 |
| `runtime::callback` | `physics::callback` | 回调具有物理阶段语义 |
| 物理模块中的数据容器 | `model::data` | Data聚合Contact与Constraint |

数据容器只保存布局与缓冲。
碰撞算法仍归物理模块。
约束算法仍归物理模块。
IO可以调用物理历史重置。
物理阶段不能回调IO。
这样避免模块依赖成环。

公开refit只证明渲染归属。
碰撞加速结构仍归碰撞模块。
纯包围盒公式可放数学层。
不要因此合并全部BVH资源。

## 3. model：输入与数据布局

### 3.1 类型与所有权

以下类型属于目标概念。

| 类型 | 持有规则 | 使用规则 |
| --- | --- | --- |
| `HostModelView<'a>` | 借用已编译模型与资产 | 转换结束前保持借用有效 |
| `ModelInput` | 拥有检查后的字段与资产 | 不保留无所有者的原生指针 |
| `Model` | 拥有设备模型与上下文句柄 | 固定拓扑与布局身份 |
| `Data` | 拥有世界状态与工作区 | 记录模型身份与设备身份 |
| `Contact` | 拥有跨世界接触数组 | 每条接触记录worldid |
| `Constraint` | 拥有按世界约束数组 | 区分稠密与稀疏布局 |

Model与Data分别管理资源。
Data不依赖原生MjData寿命。
Data记录模型与布局版本。
物理提交核对双方身份。
同尺寸不代表同拓扑。
拓扑改变要求调用方重编译。
随后引擎重新转换并上传。

### 3.2 首批字段契约

下表核对上游字段声明。
它不是完整字段清单。
`W`表示世界数。
`B_f`表示字段批量长度。
`vec3`与`vec6`表示元素形状。
标量展开还需记录步长。

| 字段 | 上游逻辑形状 | 分类与权限 |
| --- | --- | --- |
| `body_parentid` | `[nbody]` | 拓扑；上传后只读 |
| `jnt_type` | `[njnt]` | 拓扑；上传后只读 |
| `actuator_ctrladr/ctrlnum` | `[nactuator]` | 控制索引；上传后只读 |
| `body_mass` | `[B_f, nbody]` | 参数；修改需刷新常量 |
| `body_inertia` | `[B_f, nbody]`的vec3 | 参数；修改需刷新常量 |
| `qpos0` | `[B_f, nq]` | 初态；影响重置与常量 |
| `actuator_gainprm` | `[B_f, nactuator]`的vec10 | 参数；按类型检查刷新 |
| `time` | `[W]` | 动态状态；积分器写入 |
| `qpos/qvel` | `[W, nq]`与`[W, nv]` | 动态状态；状态写入使派生量失效 |
| `ctrl` | `[W, nu]` | 物理输入；调用方写入 |
| `act/act_dot` | `[W, na]` | 执行器状态与导数 |
| `history` | `[W, nhistory]` | 延迟状态；历史模块更新 |
| `xfrc_applied` | `[W, nbody]`的vec6 | 外力输入；保留分量顺序 |
| `sensordata` | `[W, nsensordata]` | 派生输出；按阶段更新 |
| `contact.worldid` | `[naconmax]` | 接触归属；碰撞阶段写入 |
| `efc.force` | `[W, njmax]` | 约束输出；求解阶段写入 |
| `overflow` | `[W]` | 诊断状态；设备内核写入 |

`nu`不等于执行器数量。
多输入执行器可以占多个控制。
接口使用ctrladr与ctrlnum。
不得把控制长度写死为nactuator。

上游允许任意正数B_f。
未指定字段默认B_f为1。
相关内核按字段长度取模。
Rust保留字段级批量布局。
接口显式区分以下模式：

```text
Shared:   B_f = 1
PerWorld: B_f = W
Periodic: B_f > 0, field_index = world_index % B_f
```

Periodic仍需逐字段迁移验证。
首版不能只验收共享参数。
参数写入声明字段与批量模式。
只读共享参数不提供可写裸指针。

依据：[字段声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L1610-L1967)、[批量字段测试](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io_test.py#L2089-L2162)。

### 3.3 容量契约

上游区分共享池与世界容量。
接口不得混用两种含义。

| 上游参数 | 范围 | Rust契约 |
| --- | --- | --- |
| `nworld` | 全批次世界数 | 必须至少为1 |
| `nconmax` | 每世界分配估算 | 不构成单世界接触硬上限 |
| `naconmax` | 全批次接触池 | 显式值覆盖nconmax估算 |
| `nccdmax/naccdmax` | CCD估算与共享池 | 分别不得超过对应接触容量 |
| `njmax` | 单世界约束容量 | 每个世界独立检查nefc |
| `njmax_nnz` | 单世界稀疏雅可比容量 | 单独检查非零元计数 |
| `nvmax` | 单世界活动自由度容量 | 必须位于`[0, nv]` |
| `*_pad` | 内核对齐后的容量 | 不等于有效物理尺寸 |

尺寸乘法使用溢出检查。
零接触容量属于有效边界。
空物理维度不等于零世界数。
默认估算必须对齐上游算法。
估算迁移仍需Rust测试。
调用方可显式覆盖容量。

| 默认项 | 上游规则 |
| --- | --- |
| `nconmax` | 按自由度、地形、flex、SDF与已有接触估算 |
| `njmax` | 按自由度、地形、flex、SDF与已有约束估算 |
| 估算档位 | 使用`16, 24, 32, 48, ... 8192`；超出后取整数 |
| `naconmax` | 显式总量优先；否则使用`nconmax * W` |
| `naccdmax` | 显式总量优先；其次使用`nccdmax * W`；否则使用naconmax |
| `njmax_nnz` | 稀疏采用拓扑估算；稠密采用`njmax * nv` |
| `nvmax` | 默认采用nv；显式值影响活动自由度分配 |

make_data不提供已有状态。
put_data估算考虑已有状态。
后端可以另选物理内存对齐。
对齐不能暗改逻辑容量。

依据：[数据创建](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L1855-L1940)、[数据布局](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L2399-L2574)。

估算依据：[接触与约束估算](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L1407-L1434)、[稀疏容量估算](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L1591-L1694)。

## 4. io：转换、交换与恢复

### 4.1 模型转换

```text
已编译模型与资产
  -> 版本、字段与能力检查
  -> 精度、索引与布局转换
  -> 拥有数据的ModelInput
  -> GPU上传与Model构造
```

转换不解释XML。
转换不调用模型编译器。
转换保留资产内容与来源。
GPU资源不借用原生地址。
异步上传保留宿主暂存数据。
上传完成前不公开可用Model。

上游转换并非简单内存复制。
上游还调用原生常量计算。
柔性预处理调用原生位置计算。
部分状态转换调用原生前向计算。
Rust允许对应原生预处理。
它不构成CPU play后端。
结构化输入也需等价常量。
缺少常量时明确报告错误。

上游将tolerance下限设为1e-6。
Rust记录输入值与有效值。
测试容限不等于求解器容限。
该转换不授权放宽测试容限。

依据：[原生预处理](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L83-L224)、[求解器配置转换](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L422-L431)。

### 4.2 状态交换与快照

状态交换使用明确字段签名。
签名决定打包顺序与长度。
接口检查世界数与输出长度。
接口保留上游签名检查。
组合签名保留原生位值。
不支持的插件状态不添加载荷。
组合签名需核对零长度字段。
状态打包保留世界掩码。
未选世界保持原有缓冲内容。
等价入口不强制快照身份检查。

上游状态打包不是完整快照。
派生量不属于默认状态载荷。
Rust快照额外记录以下元数据：

| 记录 | 恢复检查 |
| --- | --- |
| 模型与资产摘要 | 检查模型内容身份 |
| 拓扑与布局版本 | 拒绝不兼容状态布局 |
| 参数身份与精度 | 拒绝未声明的语义变化 |
| 世界数与字段签名 | 检查载荷长度与选择规则 |
| 历史与休眠附加载荷 | 保留影响后续演化的状态 |

休眠附加载荷仍需完整盘点。
本版不宣称精确续跑能力。
扩展快照记录派生量新鲜度。
调用方决定是否刷新相关阶段。
等价set_state只写签名字段。
它不隐式刷新派生量或休眠。
严格门面可以检查新鲜度。
恢复不隐式推进物理时间。
快照不保存裸设备地址。
快照不包含策略权重。

导出接触需按worldid筛选。
原生导出还重排约束地址。
Rust不保证原始接触数组顺序。
容量溢出不能冒充完整导出。
等价导出保留上游截断行为。
报告同时列出计数与截断标志。
截断结果仍可导出已有字段。
严格完整导出可以另设入口。
它不能替代get_data_into语义。

依据：[状态签名](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L772-L815)、[状态打包](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L674-L948)、[原生导出](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2396-L2470)。

### 4.3 重置与关键帧

掩码长度必须等于W。
空选择不修改任何世界。
默认重置恢复qpos0。
它清理控制、外力与热启动。
它恢复mocap与等式开关。
它重置历史、时钟与诊断。
休眠模块刷新所选世界状态。

标量关键帧越界必须报错。
逐世界关键帧保留跳过语义。
负数或越界项不重置该世界。
接口明确区分这两种输入。
不要把跳过语义改成全批失败。

共享接触池不属于单个世界。
未选世界保留积分状态。
上游仅在选中世界0时清零nacon。
等价重置保留这一写入规则。
它不自动重建接触或推进物理。
报告另行标注接触新鲜度。
严格门面可以拒绝旧接触查询。
这类检查不改变原始计数。

依据：[重置入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2670-L3038)、[关键帧选择](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L3041-L3086)。

## 5. math：数学与表示约定

数学函数不持有设备资源。
它们不提交物理或渲染任务。
GPU代码生成由工具链负责。
运行时负责内核资源与调度。

| 对象 | 首版约定 | 验证要求 |
| --- | --- | --- |
| 四元数 | 使用`[w, x, y, z]`顺序 | 检查乘法、旋转与积分 |
| 物理空间向量 | 先旋转分量，再平移分量 | 检查惯量乘法与叉乘 |
| 外部力字段 | 单独声明力与力矩顺序 | 不套用运动向量的分量顺序 |
| 矩阵视图 | 声明行列、步长与精度 | 检查转置与非连续布局 |
| 退化输入 | 逐函数保留明确处理规则 | 覆盖零长度与平行几何 |

GPU数值基线先采用f32。
宿主参考也声明实际精度。
四元数比较处理等价符号。
外部数学库需要显式转换。
不能因类型同名就直接复用。
纯公式测试不提供CPU play。
数学模块不拥有渲染BVH。

依据：[四元数公式](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py#L24-L84)、[空间代数](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py#L120-L160)、[退化几何测试](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math_test.py#L96-L152)。

## 6. runtime：设备与提交

runtime只解释通用资源。
它不持有物理字段定义。
Model与Data组合通用缓冲。
物理与渲染提交内核计划。
运行时执行计划与事件依赖。

| 操作 | 前置条件 | 完成与失效规则 |
| --- | --- | --- |
| 分配缓冲 | 检查字节数与设备容量 | 返回所有者与布局信息 |
| 上传与下载 | 检查上下文、长度与类型 | 保留暂存数据至复制完成 |
| 内核提交 | 检查资源租约与能力 | 返回提交令牌，不冒充完成 |
| 外部视图接入 | 检查所有者与上下文 | 保留读写权限及完成事件 |
| 图捕获 | 完成预热与工作区分配 | 禁止捕获期JIT与扩容 |
| 图重放 | 核对布局与计划版本 | 拒绝旧地址与旧内核配置 |

设备缺失必须返回错误。
运行时不得切换到CPU。
运行时不绑定学习库张量。
同GPU不能替代上下文检查。
底层适配集中封装unsafe。

### 异步所有权

以下规则属于Rust安全契约。
上游Python接口不提供此保证。

在途队列保留资源所有者。
提交令牌保留独占或只读租约。
安全写入必须检查在途租约。
令牌丢弃不能提前释放资源。
令牌泄漏不能造成悬空指针。
完成查询不能假定设备成功。
设备错误需隔离受影响资源。

首版签名采用保守借用。
借用令牌限制冲突操作。
队列所有权提供第二层保护。
Drop、泄漏与错误路径均需测试。
跨流并发在原型后再细化。

### 版本与缓存

模型保存拓扑与参数身份。
Data保存状态与派生量版本。
缓冲保存地址与布局版本。
执行计划保存内核配置版本。
视图记录布局版本与完成事件。

内容修改不总是使图失效。
地址与容量修改必须使图失效。
静态选项修改必须重建计划。
参数刷新检查受影响的图。
扩容只能发生在同步边界。

## 7. physics：阶段与常量刷新

### 7.1 完整步与分阶段步

下图只摘录主要阶段。
具体条件仍按冻结源码迁移。

```text
step
  forward
    检查积分器组合
    可选休眠唤醒与状态更新
    fwd_position
      运动学与质量矩阵
      碰撞、唤醒与约束构建
      可选岛构建与有效度量
    清理传感输出
    位置传感与势能
    fwd_velocity
    速度传感与动能
    control回调
    fwd_actuation
    fwd_acceleration
    solve
    可选后约束计算
    加速度传感
  指定积分器
```

forward不执行最后积分。
它仍会修改派生量与休眠状态。
step积分后可能留下旧派生量。
状态版本与派生版本分别记录。
等价查询读取当前派生缓冲。
它不强制同步到最新积分状态。
严格门面另行检查派生版本。

step1计算位置与速度阶段。
它还执行control回调。
调用方随后可以修改物理输入。
step2继续执行求解与积分。
RK4模式下step2改用Euler。
完整step仍采用RK4。
两种调用不能无条件等价。

等价step2不要求分步票据。
它允许导入已准备的中间量。
它也允许调用方组合阶段。
调用方负责物理阶段一致性。
资源租约仍检查真实访问冲突。

严格分步门面可以使用票据。
票据绑定模型与阶段版本。
门面检查重复或过期step2。
门面允许修改控制与外力。
这些规则不约束等价入口。

依据：[完整步与分步](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2063-L2156)、[位置阶段](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1322-L1365)。

导入依据：[独立step2测试](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward_test.py#L689-L740)。

### 7.2 回调与历史

上游Model持有回调集合。
Rust将实现放入物理计划。
Model不保存回调实现。
这样避免模型层依赖物理层。

回调保留调用阶段与参数。
回调可以提交自定义GPU内核。
回调可以组合上游允许的查询。
资源检查只拒绝真实访问冲突。
回调不能释放在途资源。
图捕获检查工具链兼容性。
上游可捕获回调必须等价支持。
实现缺口不能冒充上游限制。
具体回调表示仍待P1验证。

| 回调 | 上游主要写入目标 |
| --- | --- |
| `passive` | `qfrc_passive` |
| `control` | `ctrl` |
| `act_dyn` | `act_dot` |
| `act_gain/act_bias` | `actuator_force` |
| `sensor` | `sensordata` |
| `contactfilter` | `contact` |

历史模块解释采样与延迟。
IO只组织初始化与状态交换。
历史初始化检查对象范围。
接口检查样本形状与采样时间。
显式时间必须严格递增。
未提供时间时保留上游行为。
未提供phase时不暗改用户值。
多输入历史保留控制通道维度。
局部重置同步清理历史。
快照签名保留HISTORY字段。

依据：[回调定义](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L1047-L1067)、[多输入历史测试](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history_test.py#L272-L312)。

### 7.3 模型参数刷新

参数修改不是任意字段写入。
接口先检查可修改字段集合。
常量刷新归物理模块。
model只定义字段与资源布局。

| 修改类别 | 必须处理 |
| --- | --- |
| 质量、惯量与参考姿态 | 刷新相关动力学常量 |
| 执行器增益与偏置 | 检查dampratio等派生关系 |
| 静态几何或资产 | 核对碰撞与渲染加速结构 |
| 拓扑、尺寸与稀疏布局 | 拒绝复用旧模型与状态 |

刷新必须取得模型写入租约。
所有共享读者先到同步边界。
等价刷新使用调用方Data。
restore默认采用true。
false仍恢复qpos本身。
它保留参考姿态的相关派生量。
true重新计算当前qpos派生量。
该选项不表示完整Data回滚。
独立工作区只供可选门面使用。
门面不能替代等价刷新入口。
刷新后更新参数与派生版本。
静态BVH变化另行触发重建。
set_const不证明BVH仍然有效。

依据：[常量刷新调用](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/set_const.py#L616-L650)、[qpos恢复与派生量](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/set_const.py#L804-L849)、[完整刷新](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/set_const.py#L852-L920)。

## 8. render：相机、BVH与像素

RenderContext拥有渲染资源。
它保存相机映射与输出偏移。
它持有动态BVH与纹理所有者。
它只借用物理派生状态。
它不调用物理积分。

调用方决定何时更新运动学。
调用方决定何时执行BVH refit。
refit_bvh更新场景与柔性BVH。
泼溅refit更新自身BVH。
render不替调用方刷新物理状态。
等价render读取现有派生缓冲。
它不要求派生量等于最新qpos。
它不添加新鲜度拒绝条件。
严格门面可以核对位姿版本。
严格门面可以检查BVH新鲜度。
布局与寿命检查始终保持强制。

### 输出契约

| 输出 | 上游行为 | Rust契约 |
| --- | --- | --- |
| RGB | 内部使用u32打包像素 | 公开格式显式声明位序 |
| RGB提取 | 输出`[W,H,width]`的vec3 | 标量视图声明末尾三通道 |
| 原始深度 | 沿相机负Z轴投影距离 | 保留模型长度单位与f32 |
| 无几何及泼溅命中 | 原始深度写入0 | 明确区别零值与有效命中 |
| get_depth | `clamp(depth / scale, 0, 1)` | 使用独立缩放接口 |
| 分割 | 每像素保存对象ID与类型 | 不简化成单一geom_id |
| 分割背景 | 保存`(-1, -1)` | 保留背景哨兵值 |
| 异构相机 | 使用各输出独立偏移表 | 记录尺寸、步长与启用标志 |

上游提取RGB归一化到0至1。
Rust不能根据端序猜测通道。
原始深度不是射线欧氏距离。
缩放接口不覆盖原始深度。
等价缩放不新增正数限制。
它保留上游浮点运算规则。
边界样本仍需GPU数值验证。
严格缩放另行检查有限正数。
相机索引区分模型ID与输出序号。
禁用输出不得读取负偏移。
空相机列表属于有效配置。

上游每轴采样数至少为1。
大于1时执行平方次数采样。
该配置要求预计算射线。
它还要求至少一路RGB输出。
Rust保留这些组合限制。
渲染默认开启快速数学。
这是上游use_fast_math默认值。
关闭配置仍需独立验收。
数值基线与性能配置分别记录。

### 创建参数与默认值

下表覆盖创建入口的参数。
None保留上游推导语义。
默认值不能跟随调优配置改变。

| 参数 | 上游默认值 |
| --- | --- |
| `nworld` | `1` |
| `cam_res`、`cam_active` | `None`；读取模型分辨率与全部相机 |
| `render_rgb/depth/seg` | `None`；读取模型相机输出位 |
| `use_textures`、`use_fast_math` | `true`、`true` |
| `use_shadows`、`use_ambient_lighting` | `false`、`true` |
| `enabled_geom_groups` | `[0, 1, 2]` |
| `background_color` | `(0, 0, 0, 1)` |
| `flex_render_smooth`、`use_precomputed_rays` | `true`、`true` |
| `render_skybox`、`enable_backface_culling` | `false`、`true` |
| `shadow_light_fraction`、`samples_per_pixel` | `0.3`、`1` |
| `enable_vertex_normals`、`enable_specular` | `true`、`true` |
| `enable_emission`、`enable_per_light_ambient` | `true`、`true` |
| `splat_position/rotation/scale/rgba` | `None` |
| `splat_adr`、`splat_group_id` | `None`；提供泼溅后默认共用首组 |

创建上下文保留原生预处理。
它不提供MJCF编译入口。
内参随机化保留动态射线选项。
关闭预计算射线不缩减为固定相机。

依据：[BVH入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/bvh.py#L39-L43)、[原始深度](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render.py#L1251-L1294)、[输出转换](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L172-L281)、[相机与采样](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L541-L647)。

默认依据：[创建入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L312-L404)。

## 9. diagnostics：错误与有效性

诊断不只输出日志文字。
诊断返回阶段与资源定位。
设备诊断在完成后读取。
它不能把提交成功当执行成功。

| 类别 | 首版处理 |
| --- | --- |
| 输入与版本错误 | 提交前拒绝，不修改状态 |
| 上游不支持组合 | 返回字段、选项与矩阵ID |
| 容量不足 | 报告不完整；保留现有输出与显式导出 |
| 求解迭代上限 | 报告终止原因与数值指标 |
| 非有限数值 | 报告字段与世界；不隐式修复或重置 |
| 设备或异步执行错误 | 隔离相关队列与资源 |
| 上游警告 | 保留警告，不升级为支持声明 |

输入错误可保证不修改状态。
异步失败不保证状态回滚。
调用方从快照或重置恢复。
受影响视图不得冒充有效结果。
渲染失败不得改写物理缓冲。
共享池溢出需报告批次影响。
归属不明时不能只标记世界0。

上游OverflowType包含迭代上限。
Rust将容量与求解状态分开。
迭代上限不等于接触池溢出。
关闭打印不能关闭状态记录。
诊断保留原始上游类别。
等价入口不把迭代上限改成异常。
严格门面可以增加结果检查。
检查不能成为唯一调用路径。

### 首批组合检查

下表只摘录已经核对的条件。
它不是完整拒绝清单。

| 条件 | 上游行为 | 关联矩阵 |
| --- | --- | --- |
| 未知枚举或能力位 | 上传时报错 | M01、M02、M11 |
| noslip或body/actuator/sensor插件 | 上传时拒绝 | G05、G23、M01 |
| geom插件与自定义SDF | 上传属性并调用用户距离及梯度 | G10、M03、M10 |
| 休眠与柔性等式 | 上传时拒绝 | G12、G22、G25 |
| 休眠与非Newton求解器 | 上传时拒绝 | G14、G15、G25 |
| 显式稠密Jacobian且nv大于60 | 上传时拒绝 | G03、G15、M02 |
| 稳定Neo-Hookean但非DISCRETE | forward时报错 | G18、G22 |
| DISCRETE与特定柔性或休眠组合 | check_discrete按条件拒绝 | G18、G22、G25 |
| 插值柔性壳弯曲阻尼 | 上传时发出警告 | G22、M11 |
| flex与SDF或HField同时出现 | 上传时拒绝 | G10、G11、G22 |
| flex内部碰撞或rigid flex | 上传时拒绝 | G11、G22 |
| 特定CCD几何对与非零margin | 按MULTICCD与NATIVECCD条件拒绝 | G09、M11 |
| MULTICCD不支持的多接触几何对 | 发出警告并保留单接触路径 | G09、M11 |

Rust尽量在提交前检查组合。
参数修改后再次检查条件。
提前报错不改变限制含义。
警告项保留明确的未支持语义。
发布前仍需展开全部条件分支。

插件排除不涵盖全部geom插件。
首版保留SDF距离与梯度扩展。
扩展通过Rust或原生GPU接入。
调用方提供插件槽位映射。
单个实例最多保留128个属性。
首版不强制复制Python补丁方式。
没有距离实现时报告能力缺失。
不能用零值占位宣称SDF等价。

依据：[上传检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L313-L389)、[积分器检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L56-L83)、[诊断位](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L157-L201)。

补充依据：[flex条件](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L498-L550)、[CCD与geom插件](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L836-L929)、[用户SDF钩子](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/collision_sdf.py#L298-L315)。

## 10. 目标Rust调用形状

以下签名只表达设计方向。
相关类型均尚未实现。
代码块不是可编译示例。
完整公开接口仍查映射表。
这里不缩减99项等价范围。

```rust,ignore
// io::model
fn convert_model(input: HostModelView<'_>) -> Result<ModelInput>;
fn upload_model<'a>(
    device: &'a Device,
    input: &'a ModelInput,
    config: &UploadConfig,
) -> Result<Submission<'a, Model>>;

// model::data
fn make_data(model: &Model, config: &DataConfig) -> Result<Data>;

// physics
fn forward<'a>(
    model: &'a Model,
    data: &'a mut Data,
    plan: &'a PhysicsPlan,
) -> Result<Submission<'a, PhysicsReport>>;

// physics::forward
fn step1<'a>(
    model: &'a Model,
    data: &'a mut Data,
    plan: &'a PhysicsPlan,
) -> Result<Submission<'a, PhysicsReport>>;
fn step2<'a>(
    model: &'a Model,
    data: &'a mut Data,
    plan: &'a PhysicsPlan,
) -> Result<Submission<'a, PhysicsReport>>;

// render
fn render<'a>(
    model: &'a Model,
    data: &'a Data,
    context: &'a mut RenderContext,
) -> Result<Submission<'a, RenderReport>>;

// io::state
fn reset<'a>(
    model: &'a Model,
    data: &'a mut Data,
    selection: WorldSelection<'a>,
) -> Result<Submission<'a, ResetReport>>;
```

Simulator作为安全调用门面。
它持有Model与物理执行计划。
Data由调用方显式传入。
渲染上下文独立持有资源。
低层阶段接口仍保留等价语义。
票据接口只供可选严格门面使用。
门面不要求调用方依赖LRsLab。

Submission提供等待与完成查询。
完成返回报告及输出有效性。
报告与设备视图分开交付。
视图必须关联相应完成事件。
只读视图阻止冲突的可写提交。
可写视图必须独占对应资源。
最终生命周期由Rust测试验证。

## 11. 首批契约测试任务

以下任务尚未实现或运行。
测试使用Rust入口。
上游测试只提供审查依据。
上游测试里的XML编译不搬入产品。
测试改用已编译输入或原生视图。

| 契约ID | 首批断言 | 关联矩阵 | 上游审查入口 |
| --- | --- | --- | --- |
| MC01 | 检查原生版本与模型身份 | M01、M02 | `io_test.py` |
| MC02 | 保留字段级批量长度 | M05、M09 | `test_put_model_batch_sizes` |
| MC03 | 拒绝非法批量字段与零长度 | M02、M11 | `test_put_model_batch_sizes_errors` |
| MC04 | 区分共享接触池与世界约束容量 | G07、G12、M06 | `test_put_data_contact_batching` |
| MC05 | 检查CCD容量与活动自由度 | G09、G25、M06 | `io_test.py` |
| MC06 | 正确映射多输入控制通道 | G05、G24 | `test_init_ctrl_history_mimo` |
| MC07 | 局部重置保留未选世界 | M08、G24 | `test_reset_data_world`、`test_selective_reset_data` |
| MC08 | 拒绝错误掩码形状与类型 | M08、M11 | `test_reset_data_reset_invalid` |
| MC09 | 区分标量错误与逐世界跳过 | M08 | `test_reset_data_keyframe_per_world` |
| MC10 | 保留状态签名与HISTORY载荷 | M07、G24 | `test_get_state`、`test_set_state` |
| MC11 | 分离状态打包与严格快照检查 | M07、A09 | Rust新增门面测试 |
| MC12 | 保留独立step2与RK4分步语义 | G16、G19 | `test_step2`与forward源码 |
| MC13 | 保留DISCRETE分步等价条件 | G18、G19 | `test_discrete_pendulum_step1_step2_equivalence` |
| MC14 | 保留restore默认值与派生副作用 | G27、M09 | `test_set_const_restore` |
| MC15 | 分离显式refit与可选版本检查 | R01、R06、R07 | `test_refit_scene_bvh`与BVH源码 |
| MC16 | 保留深度缩放公式与边界值 | R03 | `render_util_test.py`与render源码 |
| MC17 | 保留分割对象类型与背景值 | R04、R06 | `test_get_segmentation_preserves_flex_ids` |
| MC18 | 隔离异构相机与禁用输出 | R08、R10 | `render_util_test.py` |
| MC19 | 检查采样与预计算射线组合 | R02、R08 | `render_util.py` |
| MC20 | 区分容量、迭代与异步错误 | M11、A04 | OverflowType与Rust安全测试 |
| MC21 | 覆盖令牌Drop、泄漏与在途释放 | A03、A04 | Rust新增安全测试 |
| MC22 | 拒绝冲突视图与错误上下文 | A07、A08 | Rust新增安全测试 |
| MC23 | 拒绝失效图与捕获期扩容 | A05、A06 | Rust新增安全测试 |
| MC24 | 保留拒绝分支与警告分支 | G18、G22、G25、M11 | io与check_discrete源码 |

复审补充测试任务：

| 契约ID | 补充断言 | 关联矩阵 | 上游审查入口 |
| --- | --- | --- | --- |
| MC25 | 保留渲染全部默认参数 | R01～R10 | `create_render_context` |
| MC26 | 保留默认容量与覆盖优先级 | M06 | io估算函数与`make_data/put_data` |
| MC27 | 保留溢出截断导出并报告计数 | M07、M11 | `get_data_into` |
| MC28 | 保留体积SDF与用户距离梯度钩子 | G10、M03、M10 | `test_sdf_collision`、`test_sdf_volume_collision` |
| MC29 | 保留局部重置的共享计数规则 | M08、M11 | `reset_data`内核 |
| MC30 | 严格门面不缩减等价阶段入口 | G19、R10、A09 | Rust新增门面测试 |

细化补充测试任务：

| 契约ID | 补充断言 | 关联矩阵 | 上游审查入口 |
| --- | --- | --- | --- |
| MC31 | 对齐12类记录的895项字段声明 | M02、M06、R10 | types声明与各构造入口 |
| MC32 | 对齐31类枚举及218项成员声明 | M01、M02、M11 | types枚举与匹配的原生版本 |
| MC33 | 保留选项转换默认、numeric及告警映射 | M02、G07、G23、M11 | put_model与Option属性 |
| MC34 | 区分位置和加速度factorize默认 | G03、G19、G26 | fwd_position、fwd_acceleration、forward |
| MC35 | 保留人工接触及增量追加路径 | G07、G12、M09 | run_collision_detection与collision |
| MC36 | 保留七类回调的参数、顺序和跳过条件 | G05、G23、M10 | forward、passive、sensor与collision |
| MC37 | 拒绝离散逆向的RK4及IMPLICIT转换 | G16、G17、G20 | inverse与离散加速度转换 |
| MC38 | 按MJ_MINVAL检查历史相邻时间差 | G24、M11 | init_ctrl_history、init_sensor_history |
| MC39 | 区分控制三维与传感二维初始化 | G24、M08 | 历史初始化形状分支 |
| MC40 | 保留times与phase的None副作用 | G24、M08 | test_init_history_none_times_preserves_buffer、test_init_sensor_history_phase_none_preserves_user_slot |
| MC41 | 保留射线形状、可选BVH与无命中哨兵 | R01、R06、R10 | ray、rays与形状断言 |
| MC42 | 区分渲染声明与实际输出分配 | R02～R04、R10 | create_render_context输出及AA工作区 |

数学测试另按G01至G03展开。
碰撞组合另按G07至G11展开。
本表不替代全量物理验收。
容限仍需独立证据与实测。
Windows与Linux分别执行测试。
缺少GPU时明确记录未执行。

## 12. 尚未冻结的决议

| 决议 | 当前边界 | 下一步证据 |
| --- | --- | --- |
| GPU工具链 | 不选择具体后端库 | 完成P1双平台原型 |
| 原生输入ABI | 3.12.0候选支持二十一字段子集；旧四字段与十二字段接口保持兼容；不冻结生产版本 | 展开资产、完整字段及版本条件 |
| 全部字段规范 | 盘点895项直接声明；未冻结语义布局 | 补齐单位、索引、分配及变长布局 |
| 完整组合限制 | 核对43组入口分支；未穷尽内核 | 逐条审查测试、几何对与资产路径 |
| 休眠快照载荷 | 不承诺精确续跑 | 盘点影响演化的全部状态 |
| 默认容量估算 | 算法对齐上游，迁移待验证 | 对照全部估算函数与测试 |
| 安全异步表示 | 采用借用与队列所有权方向 | 验证Drop、泄漏与设备失败 |
| 回调与图兼容 | 不假定所有回调可捕获 | 验证候选工具链与钩子机制 |
| 摘要与序列化 | 不冻结二进制格式 | 固定参数、资产与状态身份 |

十二字段详见[增量报告](windows-kinematic-fields.md)。
九惯性字段详见[惯性报告](windows-inertial-fields.md)。
本轮校验自由度最近祖先。
质量矩阵仅有GPU子集。
完整G01与G02仍待实现。
矩阵证据见[矩阵报告](windows-mass-matrix-probe.md)。
刚体子集已有七项GPU结果。
质心子集新增四项GPU结果。
零质量语义对齐冻结Warp。
原生CPU阈值回退不适用。
质心证据见[质心报告](windows-com-position-probe.md)。
数值证据见[刚体报告](windows-kinematics-probe.md)。
该子集只支持一组共享参数。
结构校验不证明G01物理等价。
严格转换不替代等价put_model。
MC01、MC02与MC31仍待完成。

下一步先审查本版契约。
再补齐字段语义与全量组合。
随后进入无头工具链探针。
实现后再固定可调用Rust签名。
