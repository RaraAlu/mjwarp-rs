# Windows完整T4补测

日期：2026-10-11。
状态：完成补测；T4尚未通过。

T4指本机Windows验收阶段。
它不指NVIDIA T4显卡。
本轮覆盖完整G01验收范围。
本轮不以肌腱子集代替全量。

## 1. 路径提示与结构修复

文档工具此前报告12处提示。
它没有发现12个源文件缺失。
PTX扩展名未进入工具索引。
工具也误读外部库路径。
本轮保留路径的真实归属。
本轮不修改工具或屏蔽规则。

| 范围 | 修复方式 |
| --- | --- |
| GPU、缓存与架构文档 | 用本地链接标明探针PTX |
| 第三方声明 | 链接冻结上游的原始文件 |
| Linux验收与G02样本 | 显式设置外部库目录 |
| Windows柔体与唤醒样本 | 先解析导入库路径并检查存在 |

本轮逐项审查8处结构漂移。
审查先比较旧指纹对应提交。
随后核对当前源码与调用链。
最后由Normify重新计算指纹。
本轮保留模块状态与公开路径。
本轮未激活计划模块。

| 模块 | 核对内容 |
| --- | --- |
| `mjwarp` | 原生平台桥接、静态参考与验收入口 |
| `mjwarp.io` | Linux原生库加载与所有权 |
| `mjwarp.io.core` | 路径编码、空字节拒绝与快照 |
| `mjwarp.physics.core` | 移除内部升精度路径 |
| `mjwarp.physics.attached` | 固定f32设备模型与输出 |
| `mjwarp.physics.mass-matrix` | f32惯量、矩阵与缓冲 |
| `mjwarp.physics.mass-solve` | f32分解、求解与非有限值检查 |
| `mjwarp.physics.resident` | 固定f32模型与速度前置链 |

结构校验达到0错误。
工具仍报告2项警告。
柔体叶子粒度仍偏粗。
多个变更仍处于进行态。
本轮不靠删除变更消除警告。
本轮只同步本机结构树。
本轮未同步Linux结构树。

## 2. Windows参考换行修复

首轮双模式脚本停在宿主测试。
G02参考哈希检查发现CRLF。
Git原始资料与清单采用LF。
Windows检出改动了参考字节。

本轮补齐两组Git换行规则。
规则覆盖JSON与参考C++。
本轮恢复12个文件的LF字节。
恢复前先计算预期LF哈希。
12项均匹配原始清单。
本轮不修改清单或参考数值。
Git差异不包含这些资料内容。
原始失败日志仍保留。
本轮另做全新检出验证。
强制启用Git自动换行转换。
12个文件仍匹配清单哈希。

规则见[Git属性](../.gitattributes)。
资料见[速度参考](../fixtures/com-velocity/README.md)。
资料见[偏置力参考](../fixtures/rne-bias/README.md)。

本轮还修正验收回执描述。
绕行内部精度应记录f32。
方向计算发生在接点舍入后。
脚本不再报告旧f64实现。
本轮不修改公式或数值容差。

## 3. 实测环境

| 项目 | 条件 |
| --- | --- |
| 源码基线 | `9f2c13a13e21263025403733d1ed8aa290c77660`加本轮工作区修复 |
| 平台 | Windows x86_64 MSVC |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER |
| 驱动 | 596.36 |
| rustc | 1.99.0；`b940084d7` |
| NVRTC | CUDA 12.8.1工具目录 |
| 原生参考 | MuJoCo 3.12.0 |
| MJWarp基线 | `71da24d956378a87a703b6e1442b13aec0c4ac29` |
| DLL SHA256 | `79b61d22b4d230a00bd31930fc6943f8ea8c6174a1bf4a6bf90f2459dbf8df1b` |

本轮测试仅读取冻结参考。
产品测试没有启动Python。
本轮没有重新生成物理参考。
文档审计使用独立宿主工具。
审计不属于产品测试链。
本轮没有执行Linux验收。
本轮没有测量NP04性能。

## 4. 实际命令与覆盖

先运行原有完整脚本：

```powershell
scripts/test-windows-g01.ps1 -AllFeatures -FullRegression
scripts/test-windows-g01.ps1 -AllFeatures -FullRegression -Release
```

修复换行后再次运行两条命令。
两种模式都停在几何肌腱门禁。
原脚本遇错即停。
所以补测另行执行全部测试。
补测保留失败且继续后续目标。
补测不修改原脚本通过条件。
失败用例仍可能提前退出。
本轮不宣称所有运动帧通过。

先准备MSVC与原生测试环境。
完整脚本已经编译原生测试DLL。
补测复用已通过验证的DLL。
随后在同类环境运行：

```powershell
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast -- --include-ignored --test-threads=1 --nocapture
cargo +stable-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast --release -- --include-ignored --test-threads=1 --nocapture
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-features --all-targets --release -- -D warnings
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
$env:RUSTDOCFLAGS = '-D warnings'
cargo +stable-x86_64-pc-windows-msvc doc --locked --all-features --no-deps
```

## 5. 补测结果

debug与release结果相同。
完整回归各执行455项测试。
各通过451项，失败4项。
两种模式均无忽略用例。
这些计数包含宿主与文档测试。
不能将455项全算作物理测试。

| 检查 | debug | release |
| --- | --- | --- |
| 原完整T4脚本 | 几何肌腱门禁失败 | 几何肌腱门禁失败 |
| MSVC非忽略测试 | 269过0败 | 269过0败 |
| G01专项 | 7过0败 | 7过0败 |
| 原生G01专项 | 2过0败 | 2过0败 |
| 仅原生feature的G01 | 1过0败 | 1过0败 |
| 原生输入与受限环境 | 各27过0败 | 各27过0败 |
| 全特性全部测试 | 451过4败 | 451过4败 |
| G02速度子集 | 7过0败 | 7过0败 |
| G02 RNE子集 | 8过0败 | 8过0败 |
| MJWarp并列诊断 | 4过0败 | 4过0败 |
| MSVC全特性Clippy | 通过 | 通过 |

表中专项属于全套测试子集。
不要将各行相加计算总数。
默认GNU宿主另有205项通过。
GNU默认与CUDA Clippy通过。
格式、Rustdoc与差异检查通过。

G01全链覆盖2084组状态。
全链对照7704548个标量。
最大绝对误差为`1.2534923e-6`。
低层入口另对照50512个标量。
最大绝对误差为`4.5833146e-7`。
两种模式记录相同误差。

失败分布如下：

| 目标 | 失败数 | 原生对照日志 |
| --- | --- | --- |
| `mass_solve_probe` | 3 | 对角逆量：125.0687对124.99999999984723 |
| `resident_geom_tendon` | 1 | 力臂绝对差：`3.273876281237875e-4` |

三项质量求解失败如下：

```text
gpu_factors_and_multiple_rhs_match_independent_native_solves
native_snapshots_feed_gpu_solves_without_native_physics_at_runtime
rhs_counts_one_two_three_and_five_reuse_each_world_factor
```

运动肌腱失败如下：

```text
inside_cylinder_motion_frames_match_native_without_relaxing_tolerance
```

质量求解不属于G01调度。
它仍属于完整回归范围。
所以G01专项通过不代表T4通过。

MJWarp诊断覆盖37组状态。
每种模式比较27121个浮点量。
同时比较3478个整数量。
整数失配数为0。
两种模式给出相同诊断文件。
最大力臂差仍为`2.5317073e-4`。
诊断仍标记`parity=not_evaluated`。
诊断通过不代表二级门禁通过。

文档工具扫描224个源码文件。
工具扫描74个文档文件。
原12处路径提示已清零。
工具当前报告0处不匹配。
人工另核对150处本地链接。
14段脚本均无语法错误。
验收脚本也通过语法检查。
以上检查不证明所有文档完备。
Linux示例本轮只审阅命令。
本轮没有执行这些Linux命令。

## 6. 判定边界

第一参考标准始终为MuJoCo。
原生通过即通过该项数值门禁。
仅原生超差后启用MJWarp。
第二参考通过也可通过门禁。
规则不要求两路同时通过。

现有测试尚未接入两级判定。
上游诊断不提供通过门槛。
所以本轮不自动放行原生失败。
本轮也不据此断言实现必错。
缺少完整二级证据仍算待验收。
参见[精度契约](numerical-precision-contract.md)。

## 7. 本地原始证据

证据根目录如下：

```text
target/t4-recheck/20261011-151150/
```

| 文件 | 内容 |
| --- | --- |
| `script-results.json` | 首轮双模式脚本失败 |
| `repaired-script-results.json` | 修复换行后的脚本结果 |
| `lf-normalized-files.json` | 12个换行恢复文件 |
| `lf-checkout-verification.json` | 自动换行模式下的新检出验证 |
| `run-supplement.ps1` | 完整补测环境与实际命令 |
| `supplement-results.json` | 各命令退出码与真实计数 |
| `debug-full.log`、`release-full.log` | 全目标日志与失败详情 |
| `debug-parallel.json`、`release-parallel.json` | 冻结MJWarp诊断 |
| `environment.json` | 工具链、设备与原生库身份 |
| `docs-audit.md` | 文档工具最终报告 |
| `docs-static-checks.json` | 本地链接与PowerShell语法检查 |

原脚本证据目录如下：

```text
debug: target/g01-acceptance/ffecaf8792d94938a4ce364aeb715530/
release: target/g01-acceptance/2a729c52a90b4830a18de41899df784e/
```

这两处保存阶段日志与计数。
完整脚本失败，未生成通过回执。
不要把缺失回执视为通过。
