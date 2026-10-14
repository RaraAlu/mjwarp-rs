# 柔体边被动力切片

## 范围

本切片实现边弹簧力。
本切片也实现边阻尼力。
它不代表完整被动力。

冻结来源采用以下函数：

```text
passive.py::_spring_damper_flexedge_passive
commit 71da24d956378a87a703b6e1442b13aec0c4ac29
```

## 字段与精度

| 字段 | 维度 | 产品类型 |
| --- | --- | --- |
| `flexedge_length0` | `nflexedge` | `f32` |
| `flex_edgestiffness` | `nflex` | `f32` |
| `flex_edgedamping` | `nflex` | `f32` |
| `flexedge_rigid` | `nflexedge` | `bool`转`i32` |
| `flex_edgeflexid` | `nflexedge` | 派生`i32` |
| `flexedge_length` | `world,nflexedge` | `f32` |
| `flexedge_velocity` | `world,nflexedge` | `f32` |
| `flexedge_J` | `world,nJfe` | `f32` |
| 两类广义力 | `world,nv` | `f32` |

内部系数使用`f32`。
内部力值使用`f32`。
乘积使用普通表达式。
累加使用`atomicAdd`。
索引与稀疏拓扑使用`i32`。
代码不使用`f64`或TF32。

柔体编号来自已校验范围。
刚性标志来自显式输入。
静止长度来自显式输入。
代码不从当前长度反推。

## 分支顺序

内核先跳过刚性边。
内核再读取启用系数。
双方系数为零时返回。
空稀疏行随后返回。
内核再计算实际力值。
双方力值为零时返回。
内核最后读取列与雅可比。
各类零力不触发原子加。

该顺序保留正负零语义。
它也保留条件读取语义。

## 设备所有权

执行计划借用常驻计划。
它核对模型指针身份。
它要求边阶段已就绪。
它直接读取设备结果。
它不回读中间边结果。
宿主快照只用于验收。

## 定向验证契约

新GPU用途此前零调用。
旧七次产品账本不清零。
旧成功选择器不再执行。
失败后只修真实原因。
源码未变时停止重复。

固定验证选择如下：

```text
physics::resident::tests::flex_edge_passive_uses_resident_values_and_preserves_conditional_reads
```

固定误差条件如下：

```text
abs(actual - expected) <= 2e-6 + 2e-6 * abs(expected)
```

测试覆盖十一世界。
测试覆盖共享自由度。
测试覆盖刚性边跳过。
测试覆盖空稀疏行。
测试覆盖零力后跳过。
测试覆盖双方禁用。
测试覆盖冷源拒绝。
测试覆盖失效源拒绝。
测试覆盖世界数错配。
测试覆盖活动坏值拒绝。

GPU执行前设置以下目录：

```text
target/precision-evidence/u038-flex-edge-passive/gpu-1/ptx
```

编译器将保存CUDA源码。
编译器也保存PTX与回执。

仓库也保存本轮原始产物。
固定存档位于以下目录：

```text
docs/evidence/flex-edge-passive
```

原始契约按字节复制。
契约仍指向捕获目录。
源码身份另记LF摘要。
两种摘要模式没有混用。

## 调用账本

宿主选择首次调用失败。
默认特性缺少门控类型。
第二次编译已经通过。
短名称精确选择为零项。
该次不算功能通过。
后续使用完整测试名称。

完整宿主选择第三次通过。
结果为一项通过。
它过滤二百一十五项。
GPU选择首次通过。
结果为一项通过。
它过滤三百零九项。
GPU用例执行一秒。
Cargo墙钟约七点九六秒。
源码未变，故停止复跑。

原始GPU日志位于：

```text
target/precision-evidence/u038-flex-edge-passive/gpu-1/stdout.log
```

日志SHA256如下：

```text
809a3cfcf8da0cabeb15cc14af450f3adc5563efb7656bfa1ece16260d45a798
```

捕获目录含两种编译变体。
每种变体含三个入口。
共保存六份CUDA源码。
共保存六份PTX产物。
共保存六份JSON回执。
依赖阶段另含九个入口。
仓库存档共含十五入口。
它们形成八份唯一PTX。
全部回执通过f32门禁。
NVRTC版本为12.8。
目标仍为compute_70。
快速数学保持关闭。

活动变体PTX摘要如下：

```text
d9358bad62df56e55df51c490f35f07a985d224c7e291e2ef8a30050c6ccbc2b
```

双方禁用PTX摘要如下：

```text
a4154486a8c80ed4ebe23b8c46ad82d45df5b454f2152f0e04a92bc620957994
```

## 资源与边界

两计划自有静态缓冲568字节。
两数据自有缓冲2392字节。
合计自有缓冲2960字节。
该值不含错配临时缓冲。
该值不含生产者与模块。
该值也不是峰值显存。

本地参考读取同次边快照。
它只验证消费公式与连接。
它不复验边生产者数值。
它不证明一般原子位等价。

NP01覆盖本切片类型。
NP02需要实际PTX回执。
NP03完整等价仍未验收。
NP04公平性能仍未验收。
