# M3稠密聚集证据

## 范围

- 任务：`2026-10-18-layers-m3-dense-gather`
- 父任务：`2026-10-18-layers-m3`
- 本切片迁移聚集入口。
- 本切片依赖校验切片。
- 本切片依赖治理纠正。
- 其他物理入口不在范围。

## 真实接口

冻结入口如下：

```text
sparse_dense_gather(
  const int* sources,
  const float* matrix,
  const float* unused,
  float* output,
  unsigned nc,
  unsigned prefix,
  unsigned reserved,
  unsigned work
)
```

内核读取`sources[slot]`。
内核读取选中矩阵值。
内核不读取`unused`。
内核写入独占输出。
`reserved`必须保持为零。
`work`必须整除`prefix`。

## 包装约束

- 构造器固定CUDA源码。
- 构造器固定聚集入口。
- 构造器绑定`nc`。
- 构造器绑定`prefix`。
- 计划固定256线程。
- 计划核对真实跨度。
- 计划核对精确容量。
- 计划核对所属会话。
- 计划拒绝写读重叠。
- 计划不设别名豁免。
- 证明绑定设备地址。
- 证明绑定宿主内容。
- 证明绑定写入代际。
- 证明拒绝负索引。
- 证明拒绝越界索引。
- 证明允许`nc`哨兵。
- 零前缀仍跳过聚集。
- 未用输入只需非空。

矩阵容量公式如下：

```text
nc * worlds + 8
```

输出容量公式如下：

```text
prefix * worlds + 8
```

元数据容量公式如下：

```text
prefix
```

零前缀保留单项占位。
包装不证明未用内容。

## 执行顺序

更新路径保持以下顺序：

1. 校验矩阵与守卫。
2. 检查设备状态。
3. 执行稠密聚集。
4. 检查输出守卫。
5. 发布就绪状态。

失败路径清除就绪状态。
同步完成语义保持不变。

## 精度映射

| 对象 | 类型 | 结论 |
| --- | --- | --- |
| 矩阵 | `f32` | 保持产品类型 |
| 输出 | `f32` | 保持产品类型 |
| 元数据 | `i32` | 保持设备布局 |
| 发射标量 | `u32` | 保持参数顺序 |
| 矩阵偏移 | `u64` | 保持CUDA表达式 |
| 编译器 | CUDA 12.8 | 禁用快速数学 |

CUDA源码保持不变。
PTX字节保持不变。
数学与容差保持不变。
夹具也保持不变。

## 验证

| 证据 | 结果 |
| --- | --- |
| `01-host-plan-observation.log` | 一次失败后通过 |
| `01-host-plan-attempt-1.log` | 官方失败输出已恢复 |
| `01-host-plan-attempt-2.log` | 官方成功输出已恢复 |
| `02-gpu-tests.log` | 两项GPU测试通过 |
| `03-gpu-retest.log` | 构造顺序修正后通过 |
| `04-cargo-fmt-check.log` | 首次格式检查通过 |
| `04-cargo-fmt-check-attempt-2.log` | 修正后格式检查通过 |
| `05-cargo-check-default.log` | 默认检查通过 |
| `06-cargo-clippy.log` | 首次发现一个警告 |
| `06-cargo-clippy-attempt-2.log` | 修正后零警告 |
| `07-gpu-environment.log` | 记录设备环境 |
| `kernel-comparison.json` | 历史字节完全一致 |
| `capture-manifest.json` | 映射六项原始文件 |
| `doc-audit.md` | 文档审计零失配 |
| `08-delivery-preflight.log` | 交付预检通过 |

宿主首轮别名用例失败。
容量检查先于别名检查。
修正跨度后测试通过。
官方工具输出已经恢复。
两份日志保留完整文本。
日志不是字节原始流。
监督已经核对转录。
监督记录保留调用标识。
本轮没有补造输出。
本轮没有为日志重跑。

首次GPU执行覆盖两项。
两项测试全部通过。
产品恢复保持逐位一致。
跨块尾部覆盖514项。
第二次执行验证顺序修正。
第二次关闭产物捕获。
首次原始字节保持不变。

静态分析先发现写法警告。
实现随后改用`is_empty`。
第二次静态分析通过。
Normify校验零错误。
既有结构警告共十六项。
交付预检已经通过。

## 捕获保留

原始目录如下：

```text
D:\Rust\mjwarp-rs\target\development-evidence\layers-m3-dense-gather\capture-raw
```

目录保留六个原始文件。
目录保留CU原始字节。
目录保留PTX原始字节。
目录保留JSON原始字节。
本轮没有删除文件。
本轮没有清理文件。
本轮没有恢复旧文件。

两项入口共享源码字节。
两项入口共享PTX字节。
清单逐项映射六文件。
两份元数据分别归档。
源码和PTX采用gzip。
压缩前没有规范化。

源码散列如下：

```text
2ee26abc0db209887b27e0c9b530e51e3b1af74daebaf0c518c3e6098a288042
```

PTX散列如下：

```text
eba8479127c4adb44f9e864d952ad9dd6205a6612ac0690e9951cdb1f39cf8a5
```

当前源码匹配历史归档。
当前PTX匹配历史归档。
比较采用原始字节。
比较没有裁剪尾空白。

## 设备边界

设备为RTX 4070 Ti SUPER。
驱动版本为596.36。
显存总量为16376MiB。
整卡采样约4370MiB。
它不代表任务峰值。
它不代表任务独占值。

## 未验收项

本机不是T4显卡。
本轮不构成T4验收。
本轮未执行Linux验收。
本轮未执行发布构建。
NP01至NP04仍未验收。
机器码等价仍未验证。
父任务M3仍未完成。
M4与A1继续阻塞。
