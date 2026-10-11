# 原生基准与5%误差红线验收

日期：2026-10-11。
状态：机制与专项验证完成。
完整T4仍待收尾。

本报告记录机制专项验证。
后续已完成完整产品范围复测。
质量求解仍阻止完整T4放行。
最新记录见[T4复测](windows-t4-redline-retest.md)。
下文未测项只描述本报告当时。

## 1. 冻结判定

用户在本轮测试前批准5%。
第一参考仍为原生MuJoCo。
原生达标直接放行该项NP03。
仅原生超差后启用MJWarp。
两路误差都采用同一原生基准。
每个字段块采用相同分母。

```text
t_i = atol_native + rtol_native * abs(native_i)
E_candidate = max_i(abs(candidate_i - native_i) / t_i)
E_mjwarp    = max_i(abs(mjwarp_i    - native_i) / t_i)
redline     = 1.05 * E_mjwarp
```

| 条件 | 标注 | NP03放行 |
| --- | --- | --- |
| 原生容差达标 | 原生放行 | 是 |
| 原生超差，误差不劣于MJWarp | 对照放行 | 是 |
| 原生超差，误差略高但未越红线 | 红线内放行 | 是 |
| 原生超差，误差越过红线 | 失败不放行 | 否 |
| 缺少必要参考、容差或红线 | 待验收不放行 | 否 |
| 身份、条件或有限性检查失败 | 失败不放行 | 否 |
| 未运行或没有数值样本 | 未验收不放行 | 否 |

等于边界时放行。
5%约束最大归一化误差。
它不是输出值的5%偏移量。
它也不改变原生容差。
本轮不改变任何物理内核。
本轮保留此前FMA顺序修复。
新规不代表物理精度改善。

字段块提前固定比较范围。
程序不跨字段或跨世界取最大值。
最大值不保证逐分量优于上游。
上游误差很大时红线也会增大。
因此独立不变量仍须通过。
NP01、NP02与NP04仍独立检查。
完整规则见[精度契约](numerical-precision-contract.md)。

## 2. 接入与标注

判定实现见[辅助模块](../tests/support/two_reference.rs)。
边界测试见[宿主用例](../tests/two_reference_policy.rs)。
运动入口见[肌腱测试](../tests/resident_geom_tendon.rs)。

辅助模块提供统一误差算法。
原生通过时不加载第二参考。
运动入口先校验冻结资料哈希。
它核对模型、状态与批量身份。
它也检查帧、世界与拓扑。
校验后才提供第二参考。
G01主链与低层沿用该判定。
质量求解也使用该辅助模块。
但它仍缺同条件第二参考。
本轮没有重测质量求解。

规则名为`native_error_v2`。
汇总采用schema 2。
报告明确写入准入布尔值。
报告也记录依据与放行标注。
两路误差都指向原生基准。
详细报告记录归一化最大值。
它也记录5%余量与实际红线。
`mjwarp_passed`合计两类二级放行。
详细原因继续区分两类放行。
旧解析器不能沿用值差语义。

汇总仅声明已记录字段块。
空字段只能检查形状。
零数值样本不能通过汇总。
缺失或异常证据阻止准入。
独立诊断明确标注不作放行。
本轮只编译该诊断标注变更。
本轮没有执行并列诊断目标。

## 3. 实际验证

各配置本轮只调用一次。
测试没有忽略用例。
运动测试改名不重置调用计数。
旧393诊断已累计调用三次。
本轮没有再次调用它。
本轮没有运行完整T4脚本。
本轮没有运行Linux或性能验收。
产品测试没有启动Python。
本轮没有重新生成参考资料。

| 配置 | 通过 | 失败 | 忽略 |
| --- | --- | --- | --- |
| GNU宿主判定 | 20 | 0 | 0 |
| MSVC debug专项 | 35 | 0 | 0 |
| MSVC release专项 | 35 | 0 | 0 |

双模式各含三组测试目标。
G01各通过7项。
运动几何肌腱各通过8项。
宿主规则各通过20项。
宿主合成测试不算物理证据。
Clippy与格式检查均通过。

实际命令如下：

```powershell
cargo test --locked --test two_reference_policy -- --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --test resident_geom_tendon --test g01 --test two_reference_policy -- --include-ignored --test-threads=1 --nocapture
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --release --test resident_geom_tendon --test g01 --test two_reference_policy -- --include-ignored --test-threads=1 --nocapture
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
cargo fmt --check
```

宿主用例覆盖三类放行。
它们也覆盖以下拒绝边界：

- 严格越线与缺失红线。
- 无效参考、条件与形状。
- 非有限值、溢出与零分母。
- 缺原生证据却请求回退。
- 汇总没有真实数值样本。

## 4. 字段块实测

下表分别适用于两种模式。
统计只保留三个物理scope。
它不包含宿主合成审计。

| 范围 | 原生放行 | 二级放行 | 失败 | 待验收 | 标量数 |
| --- | --- | --- | --- | --- | --- |
| `G01-complete` | 58352 | 0 | 0 | 0 | 7704548 |
| `G01-raw` | 336 | 0 | 0 | 0 | 50512 |
| `G01-geom-motion` | 18 | 9 | 0 | 0 | 2178 |

运动肌腱共27个字段块。
其中9块均属于对照放行。
没有字段使用5%余量放行。
宿主边界用例覆盖这项余量。

三组目标状态如下：

```text
(worlds=3, world=2)
(worlds=513, world=2)
(worlds=513, world=512)
```

三组状态给出相同判定。
以下分数采用归一化误差。
表中数值保留六位小数。
程序使用完整f64分数。

| 帧与字段 | 本库分数 | MJWarp分数 | 5%红线 | 标注 |
| --- | --- | --- | --- | --- |
| 310 `ten_J` | 9.774043 | 9.774043 | 10.262746 | 对照放行 |
| 310 `wrap_xpos` | 1.481762 | 1.481762 | 1.555850 | 对照放行 |
| 393 `ten_J` | 12.236763 | 12.240589 | 12.852618 | 对照放行 |

第344帧继续原生放行。
第393帧仍超出原生容差。
它的误差略低于MJWarp。
因此新规允许该字段块通过。
这不代表原生误差已经达标。
本轮改变规则，不改变物理结果。
旧失败日志保留原始结论。

## 5. 环境与证据

| 项目 | 记录 |
| --- | --- |
| 源码基线 | `acb58601c5f36284049157608d01e007c6c0fc85`加本轮判定变更 |
| 平台 | Windows x86_64；GPU专项采用MSVC |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER |
| 驱动 | 596.36 |
| NVRTC | 12.8.93；目录`target/toolchains/cuda-12.8.1` |
| 原生参考 | MuJoCo 3.12.0；f64 |
| 本库与MJWarp输出 | f32 |
| 冻结上游 | `71da24d956378a87a703b6e1442b13aec0c4ac29` |
| 上游作者环境 | Warp 1.15.0；MuJoCo 3.15.0；CUDA 12.9.86 |
| 原生DLL SHA256 | `79b61d22b4d230a00bd31930fc6943f8ea8c6174a1bf4a6bf90f2459dbf8df1b` |

同条件检查针对两路GPU输入。
它涵盖模型量化、状态与拓扑。
原生f64参考提供共同基准。
原生输入与GPU仍有量化差异。
作者环境与本库工具链也不同。
本轮不宣称工具链完全相同。
加载细节见[参考辅助](../tests/support/mjwarp_geom_reference.rs)。

本地证据目录如下：

```text
target/native-error-redline/
```

| 文件 | 内容 |
| --- | --- |
| `run.ps1` | 命令与调用计数控制 |
| `calls.json` | 每项调用次数、退出码与计数 |
| `host-policy-1.log` | GNU宿主测试 |
| `debug-gates-1.log`、`release-gates-1.log` | 双模式专项日志 |
| `debug-gates-np03.json`、`release-gates-np03.json` | 物理与宿主合成摘要 |
| `clippy-msvc-1.log` | 全特性全目标静态检查 |
| `debug-ptx/`、`release-ptx/` | 实测CUDA与PTX |

Git不跟踪这些本地产物。
消费摘要时须筛选物理scope。
不能把宿主模拟失败算入GPU。
也不能把宿主通过算作物理验收。

## 6. 收尾边界

本轮只关闭判定机制工作。
Normify继续保留完整T4开放。
运动肌腱不再缺批准红线。
质量求解仍缺同条件参考。
其余辅助仍需核对门禁覆盖。
完整T4仍需另行安排补测。
补测不能违反三次调用上限。
Linux与NP04仍需独立证据。

历史证据见[旧两级补测](windows-two-reference-acceptance.md)。
算术变化见[初始长度修复](inside-norm-order-fix.md)。
