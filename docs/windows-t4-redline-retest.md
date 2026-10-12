# Windows红线T4复测

日期：2026-10-11。
状态：复测完成；完整T4不放行。

本报告保留10月11日失败证据。
10月12日追加验证各483过0败。
新数值失败与待验收均为零。
独立精度与性能门禁仍待收口。
最新状态见[T4追加验证](windows-t4-g03-retest.md)。

## 1. 执行边界

本轮沿用已批准的5%红线。
规则名为`native_error_v2`。
第一参考仍为原生MuJoCo。
原生超差后才启用MJWarp。
两路均计算相对原生的误差。
契约见[精度规则](numerical-precision-contract.md)。

本轮不改产品源码或测试源码。
本轮不改参考数据与原生容差。
本轮只运行验收并记录证据。
产品测试没有启动Python。
本轮没有重新生成物理参考。
原生DLL只承担加载与字段转换。
数值对照读取冻结原生资料。

本轮拆分原T4包装流程。
它避免重复执行G01专项。
它也避免再次调用满额诊断。
目标清单来自Cargo元数据。
主体执行库、主程序与32个目标。
另用原生脚本执行27项输入测试。
两种模式各执行12项文档测试。
本轮覆盖完整产品测试目标。

旧`frame393_trace`包含两项诊断。
它不承担容差准入。
旧记录已经累计调用三次。
因此本轮不选择这个目标。
历史证据见[首差追踪](frame393-first-divergence.md)。
该证据不能充当本轮新测试。
本轮不宣称重跑全部仓库用例。

原生受限运行另各执行27项。
仅原生feature另各执行1项。
默认GNU宿主也单独验证。
Linux条件目标在Windows运行零项。
零项不构成Linux验收。
本轮没有运行Linux或NP04性能。

## 2. 测试结果

每条测试命令本轮只执行一次。
组合调用不重置既有次数。
G01与运动肌腱此前各调用两次。
本轮各增加一次，达到三次。
旧393诊断本轮增加零次。
本轮没有重试失败测试。

| 范围 | debug | release |
| --- | --- | --- |
| 主体库、程序与32目标 | 435过、3败 | 435过、3败 |
| 原生输入 | 27过、0败 | 27过、0败 |
| 文档测试 | 12过、0败 | 12过、0败 |
| 产品范围合计 | 474过、3败 | 474过、3败 |
| 附加：原生受限运行 | 27过、0败 | 27过、0败 |
| 附加：仅原生feature | 1过、0败 | 1过、0败 |

产品范围合计每种模式477项。
各模式均无忽略用例。
两项诊断排除不计入该合计。
合计包含宿主与文档测试。
不能将其全部视为GPU物理测试。
附加配置不重复计入产品合计。

以下目标属于上述合计子集。

| 目标 | 每种模式结果 |
| --- | --- |
| `g01` | 7过、0败 |
| `native_g01` | 2过、0败 |
| `resident_geom_tendon` | 8过、0败 |
| `resident_com_velocity` | 7过、0败 |
| `resident_rne_bias` | 8过、0败 |
| `mass_matrix_probe` | 7过、0败 |
| `mass_solve_probe` | 4过、3败 |
| `mjwarp_geom_tendon` | 6过、0败；仅诊断 |
| `two_reference_policy` | 20过、0败；宿主规则 |

GNU宿主与文档共225项通过。
GNU默认与CUDA Clippy均通过。
MSVC双模式全特性Clippy通过。
格式与Rustdoc检查通过。
完整产品命令退出码均为101。
测试失败仍阻止整体放行。

## 3. 数值判定摘要

双模式NP03摘要逐字节一致。
统计只包含真实物理scope。
宿主合成审计不参与下表。

| 范围 | 原生放行 | 二级放行 | 失败 | 待验收 |
| --- | --- | --- | --- | --- |
| G01完整链 | 58352 | 0 | 0 | 0 |
| G01低层入口 | 336 | 0 | 0 | 0 |
| G03分解与多右端项 | 233 | 0 | 0 | 7 |
| G03批量求解 | 6410 | 0 | 0 | 0 |
| G03原生快照 | 233 | 0 | 0 | 7 |
| G03右端项数量 | 128 | 0 | 0 | 24 |
| G03错误恢复 | 5 | 0 | 0 | 0 |
| 运动肌腱 | 18 | 9 | 0 | 0 |
| 合计 | 65715 | 9 | 0 | 38 |

9块二级通过均属对照放行。
没有物理字段使用5%余量放行。
宿主规则测试覆盖该余量边界。
第393帧仍超出原生容差。
它略优于同条件MJWarp误差。
因此该字段块继续对照放行。

G01完整链覆盖2084组状态。
它对照7704548个标量。
最大绝对误差为`1.2534923e-6`。
低层入口另对照50512个标量。
最大绝对误差为`4.5833145e-7`。

并列诊断报告也逐字节一致。
它记录零整数失配。
它仍标记`parity=not_evaluated`。
准入标志为`np03_admitted=false`。
该诊断不替代上述数值门禁。

## 4. 三项失败的原因

失败全部来自质量求解目标。
以下三个测试拒绝准入：

```text
gpu_factors_and_multiple_rhs_match_independent_native_solves
native_snapshots_feed_gpu_solves_without_native_physics_at_runtime
rhs_counts_one_two_three_and_five_reuse_each_world_factor
```

它们记录38次待验收。
原因均为`missing_mjwarp_reference`。
它们没有记录红线越界失败。
这不表示原生容差已经达标。
程序先发现原生超差。
随后因缺第二参考而阻断。
源码见[质量求解测试](../tests/mass_solve_probe.rs)。

38次记录对应7种字段参考。
重复入口与右端项扩大计数。
样本均来自`mixed-joints`。
下表按原生参考身份去重。
E表示最大归一化误差。
表中编号均从零起算。

| case | 字段 | rhs | E | 重复次数 |
| --- | --- | --- | --- | --- |
| 2 | `diagonal_inverse` | 无 | 2.726298 | 6 |
| 2 | `solution` | 1 | 2.736048 | 6 |
| 2 | `solution` | 2 | 2.744052 | 4 |
| 5 | `ld` | 无 | 1.050041 | 6 |
| 5 | `solution` | 2 | 1.034242 | 4 |
| 6 | `ld` | 无 | 1.125766 | 6 |
| 6 | `solution` | 1 | 2.052568 | 6 |

最高E来自case 2的rhs 2。
该字段最大绝对差为`0.3730135`。
原生容差仍为以下公式：

```text
2e-4 + 2e-4 * abs(native_i)
```

5%不能直接放宽这个公式。
它只作用于MJWarp误差分数。
没有同条件MJWarp就没有红线值。
因此不能凭原生超差幅度放行。
独立求解残差与重构检查均通过。
这仍不足以补齐第二参考。

下一步应冻结G03同条件参考。
它须匹配模型、状态与右端项。
它也须核对f32输入与字段布局。
随后接入现有5%红线机制。
取得证据前不宣称数值缺陷修复。

## 5. 环境、命令与证据

| 项目 | 本轮记录 |
| --- | --- |
| 源码版本 | `76e9118028ad3418a6aa5573a88e1c2ce8817ded` |
| 测试时工作区 | 干净；本轮报告随后写入 |
| 平台 | Windows x86_64；GPU采用MSVC |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER |
| 驱动 | 596.36 |
| rustc | 1.99.0；`b940084d7` |
| 原生MuJoCo | 3.12.0；冻结f64数值参考 |
| CUDA工具目录 | `target/toolchains/cuda-12.8.1` |
| 冻结MJWarp | `71da24d956378a87a703b6e1442b13aec0c4ac29` |
| 原生DLL SHA256 | `79b61d22b4d230a00bd31930fc6943f8ea8c6174a1bf4a6bf90f2459dbf8df1b` |

本地证据根目录如下：

```text
target/t4-redline-retest/
```

`run.ps1`保留完整实际命令。
主体命令显式选择32个目标。
它没有调用原完整G01包装脚本。
它保留遇错继续的执行方式。
它不重新运行已经记录的配置。

主体命令的结构如下。
占位处代表完整目标清单。
发行模式另加`--release`。

```text
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast --lib --bins <32组--test参数> -- --include-ignored --test-threads=1 --nocapture
scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast --doc -- --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --locked --features native-model-probe --test native_g01 -- --include-ignored --test-threads=1
```

| 文件 | 内容 |
| --- | --- |
| `environment.json` | 版本、目标清单、排除项与既有次数 |
| `calls.json` | 全部命令、调用次数与退出码 |
| `report.json` | 产品计数、待验收明细与不放行结论 |
| `*-products.log` | 全特性主体测试日志 |
| `*-products-np03.json` | 真实物理数值门禁摘要 |
| `*-native-report.json` | 原生常规与受限运行证据 |
| `*-doc-tests.log` | 文档测试结果 |
| `*-parallel.json` | 仅诊断的上游并列结果 |
| `*-ptx/` | 实测CUDA与PTX |

Git不跟踪上述本地产物。
本轮只提交文档记录。
Normify继续保留主T4开放状态。
局部复测工作收尾不等于T4通过。
