# 内侧绕行乘加顺序修复

日期：2026-10-11。
状态：局部修复落地。
完整T4仍待验收。

## 实现边界

本轮只固定两处初始长度。
`inside_norm`先舍入x平方。
随后融合y平方与已舍入项。
最后调用原有`sqrtf`。

```cpp
sqrtf(__fmaf_rn(a.y, a.y, __fmul_rn(a.x, a.x)))
```

`inside`用它计算初始长度。
两处变量为`l0`与`l1`。
通用`n2`保持原样。
内核仍采用GPU f32计算。
本轮不修改迭代阈值。
本轮不修改任何验收容差。
本轮不添加模型或帧号分支。

源码见[绕行内核](../src/physics/tendon_wrap.rs)。
因果证据见[历史首差](frame393-first-divergence.md)。

新增回归使用常驻产品入口。
它检查第393帧六个接触坐标。
它要求冻结参考逐位一致。
它同时检查三类离散拓扑。
测试覆盖3与513世界。
3世界检查世界2。
513世界检查世界2与512。
所有编号均从零起算。

回归见[并列测试](../tests/mjwarp_geom_tendon.rs)。
这项回归不替代两级门禁。
旧诊断仅更新源码替换锚点。
本轮没有重新运行旧诊断。

## 第393帧数值结果

下表记录字段最大绝对差。
三组目标世界给出相同结果。
debug与release结果也相同。

| 参考与字段 | 修复前 | 修复后 |
| --- | --- | --- |
| MJWarp `wrap_xpos` | `2.682209014892578e-7` | `0` |
| MJWarp `ten_length` | `1.1920928955078125e-7` | `0` |
| MJWarp `ten_J` | `2.531707286834717e-4` | `8.940696716308594e-8` |
| 原生MuJoCo `ten_J` | `8.519473962564916e-5` | `2.859303998969387e-4` |

修复后接触字段逐位一致。
力臂字段仍有1个分量失配。
该分量索引为3。
原生力臂误差反而增大。
原生超差分量从2个增至3个。
因此不能宣称原生误差修复。

两路参考给出不同数值结果。
本轮落实冻结上游的算术顺序。
它修复已定位的接触点偏差。
它没有完成全部数值验收。
原生仍为第一参考。
仅原生超差后启用MJWarp。
缺批准容差仍记待验收。
本轮不会自动采用通用容差。

## 其他样本与边界

并列诊断比较37组状态。
它覆盖16浮点字段。
它也覆盖3类整数字段。
两种模式均无整数失配。
32组状态的统计保持原样。
另外5组状态产生数值变化。
其中3组属于第393帧。
另外2组属于基础样本1。

基础样本1位于世界0。
它覆盖3与513世界。
其接触坐标新增2个失配。
每个失配均为1 ULP。
最大差为`2.9802322e-8`。
其力臂最大绝对差保持原样。
力臂失配分量从2个降至1个。
对应原生基础测试仍通过。
因此本轮不宣称全样本逐位一致。

第310帧保持原有结果。
其MJWarp三类肌腱字段逐位一致。
第344帧也保持原有结果。
力臂差为`2.9802322e-8`。
长度差为`2.3841858e-7`。
其接触字段逐位一致。

## 实际验证

本轮按配置各调用一次。
两种模式均无忽略用例。

| 测试目标 | debug | release |
| --- | --- | --- |
| `resident_geom_tendon` | 7过、1败 | 7过、1败 |
| `mjwarp_geom_tendon` | 6过、0败 | 6过、0败 |
| `g01` | 7过、0败 | 7过、0败 |
| `native_g01` | 2过、0败 | 2过、0败 |
| `resident_spatial_tendon` | 8过、0败 | 8过、0败 |
| `resident_tendon` | 5过、0败 | 5过、0败 |
| 合计 | 35过、1败 | 35过、1败 |

失败测试仍报告9次待验收。
它报告18次原生字段块通过。
它没有报告双参考均超差。
待验收原因仍为缺二级容差。
第310帧涉及力臂与接触字段。
第393帧只涉及力臂字段。
计数包含三组目标世界。
两种模式的判定摘要一致。

其他几何测试均通过。
它们覆盖球柱及退化路径。
它们也覆盖有限差分与缩扩容。
G01主链与低层门禁均通过。
这不表示完整T4通过。
本轮没有重跑完整T4。
旧诊断此前已累计调用3次。
本轮没有重跑它来绕过上限。
本轮未运行Linux或性能验收。
产品测试没有启动Python。
本轮没有重新生成上游参考。

实际测试命令如下。
发行模式另加以下参数：

```text
--release
```

```powershell
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --test resident_geom_tendon -- --include-ignored --test-threads=1 --nocapture
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --test mjwarp_geom_tendon -- --include-ignored --test-threads=1 --nocapture
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --test g01 --test native_g01 --test resident_tendon --test resident_spatial_tendon -- --include-ignored --test-threads=1 --nocapture
```

静态检查均通过。
以下配置各调用一次。

```powershell
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
cargo fmt --check
```

## 环境与证据

源码起点为`2793741`。
本机GPU型号如下：

```text
NVIDIA GeForce RTX 4070 Ti SUPER
```

驱动版本为596.36。
Rust MSVC版本为1.99.0。
NVRTC版本为12.8.93。
原生参考采用MuJoCo 3.12.0。
MJWarp冻结版本如下。

```text
71da24d956378a87a703b6e1442b13aec0c4ac29
```

本轮核对原生DLL哈希。

```text
79b61d22b4d230a00bd31930fc6943f8ea8c6174a1bf4a6bf90f2459dbf8df1b
```

当前日志目录如下。

```text
target/inside-norm-order-fix/
```

`calls.json`保留命令与次数。
各模式日志保留测试结果。
以下文件分别保留三类数据：

| 文件 | 数据 |
| --- | --- |
| `*-native-np03.json` | 待验收项 |
| `*-parallel.json` | 37组统计 |
| `parallel-comparison.json` | 前后差异 |

`*-ptx/`保留产品源码与PTX。
Git不跟踪上述本地产物。

文档工具检查230个源码文件。
它也检查77个文档文件。
工具报告0处不匹配。
人工核对本轮三份文档。
核对范围包含源码与实际日志。
工具检查不替代完整人工审计。

修复前数据取自以下目录。

```text
target/two-reference-acceptance/20261011-154844/
```

本轮只收尾局部算术修复。
Normify保留完整T4开放状态。
后续仍需独立冻结二级容差。
质量求解也仍缺同状态参考。
历史范围见[两级补测](windows-two-reference-acceptance.md)。
