# 运动视频逐帧原生复核

日期：2026-10-07。
范围：G01球柱绕行子集。
结论：T4修复后全部通过。
本报告不关闭完整G01。
本报告保留初轮失配证据。
末节记录精度修复与复验。

## 初轮复核方法

我重新导出全部GPU运动帧。
导出器同时保存实际f32输入。
我核对旧视频的GPU数据。
新旧480帧的全部字段一致。
新旧十六个静态检查点一致。
因此复核覆盖原视频的状态。

原生工具加载冻结MJB。
它不编译XML。
它不运行Python。
它逐状态执行以下阶段。

```text
mj_resetData
mj_kinematics
mj_comPos
mj_tendon
```

原生工具逐帧读取相同输入。
它使用相同qpos与mocap。
它也使用相同geom_size。
它回显输入，检查器精确核对。
原生计算保持f64。
原生重复执行结果字节一致。
GPU公开结果保持f32。
两端均不执行动力学积分。

每帧覆盖四个审查世界。
世界编号为`0、1、2、512`。
我比较全部十条肌腱。
我比较全部包裹容量与尾部。
我另外比较几何与site位姿。
本轮不比较其余509个世界。

容限保持原值。
检查器采用以下判据。
整数要求精确一致。

```text
abs(gpu - native) <= 2e-5 + 2e-5 * abs(native)
```

## 初轮结果

| 检查 | 结果 |
| --- | ---: |
| GPU源帧 | 480 |
| 每帧世界数 | 4 |
| 原生执行状态 | 1920 |
| 比较标量 | 852480 |
| 全部通过帧 | 477 |
| 超限帧 | 3 |
| 超限状态 | 6 |
| 浮点超限标量 | 16 |
| 整数失配标量 | 0 |

所有失配均来自T4内侧圆柱。
失配世界均为2与512。
两者采用相同尺寸与状态。
两者产生相同超限结果。

以下帧号从1开始。
时间采用源帧的名义24fps。
时间不等于播放器精确时间。

| 帧号 | 名义秒数 | 世界 | 超限字段 | 每世界超限数 |
| --- | ---: | --- | --- | ---: |
| 310 | 12.875 | 2、512 | 力臂、包裹接点 | 5 |
| 344 | 14.292 | 2、512 | 力臂 | 1 |
| 393 | 16.333 | 2、512 | 力臂 | 2 |

| 字段 | 最大绝对差 | 超限标量 |
| --- | ---: | ---: |
| `ten_length` | 8.845451e-6 | 0 |
| `ten_J` | 3.275069e-4 | 12 |
| `wrap_xpos` | 4.673028e-5 | 4 |
| `geom_xpos` | 1.360927e-7 | 0 |
| `geom_xmat` | 2.384186e-7 | 0 |
| `site_xpos` | 3.371477e-7 | 0 |
| `ten_wrapadr` | 0 | 0 |
| `ten_wrapnum` | 0 | 0 |
| `wrap_obj` | 0 | 0 |

最大力臂差来自第310帧。
CSR标量索引为3。
它对应T4的DOF编号2。

```text
GPU:         -0.6751084327697754
MuJoCo:      -0.674780925932362
absolute:     0.0003275068374133383
allowed:      0.000033495618518647246
ratio:        9.777602322256355
```

接点最大绝对差不等于最大超限。
容限还取决于原生分量大小。
最大接点容限比为1.483810。
它来自第310帧的Z分量。
两处接点均出现该超限。

本轮未确认数值失配根因。
静态十二组通过不能替代逐帧。
后续修复须加入这些回归状态。
后续验收须保持现有容限。

## 初轮审查产物

产物目录：

```text
target/geom-tendon-review/
```

| 文件 | 内容 |
| --- | --- |
| `frame-gpu.json` | 480帧GPU结果与实际输入 |
| `native-frame-inputs.txt` | 1920组原生输入 |
| `native-frame-results.jsonl` | 重新执行的原生结果 |
| `native-frame-build.log` | MSVC构建证据 |
| `native-frame-run.log` | DLL路径、版本与执行计数 |
| `frame-audit-summary.json` | 汇总、误差、判据与哈希 |
| `frame-audit-fields.jsonl` | 每帧每世界每字段差值 |
| `frame-audit-frames.csv` | 每帧通过与超限状态 |
| `frame-audit-failures.json` | 全部16项超限详情 |
| `failing-states.json` | 三组失配回归候选状态 |
| `03-frame-by-frame-review.mp4` | GPU与逐帧原生叠图 |
| `frame-review-310.png` | 最严重超限帧的审查图 |
| `frame-audit-bundle.zip` | 原始数据与复验脚本归档 |

新视频暂停三处超限状态。
每处额外暂停两秒。
实际视频时长约25.94秒。
实时编码器保留619帧。
它省略五帧显示记录。
它不减少数值复核状态。
新视频仍不包含动力学积分。
显示端仍重建接点间弧线。
弧线不参与数值比较。
原视频编码保留476帧。
数值复核覆盖全部480源帧。
Git忽略临时审查产物。
清理target会删除原始证据。
请另存需要保留的产物。

## 初轮实际命令

```powershell
target/geom-tendon-review/frame-export.exe fixtures/geom-tendon/wrap-tree.json target/geom-tendon-review/frame-gpu.json
node target/geom-tendon-review/frame-audit.cjs prepare
target/geom-tendon-review/run-native-frame.ps1
node target/geom-tendon-review/frame-audit.cjs check
node target/geom-tendon-review/frame-render.cjs --preview
node target/geom-tendon-review/frame-render.cjs
node target/geom-tendon-review/frame-video-verify.cjs
node target/geom-tendon-review/frame-finalize.cjs
```

原生工具通过MSVC警告检查。
构建采用`/W4 /WX`。
逐帧比较命令退出码为1。
它如实表示三帧数值超限。
我没有放宽容限或删除失败帧。
我已核对新视频的五处抽帧。
抽帧包含三处超限停留。
文档核对仅覆盖本次三份文档。
本轮未运行Python文档工具。
本轮未重跑完整产品测试。
本轮未验证Linux。

## 环境与锁定

| 项目 | 值 |
| --- | --- |
| 平台 | Windows x86_64 |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER |
| 驱动 | 596.36 |
| 原生版本 | MuJoCo 3.12.0 |
| GPU数据提交 | `8bf37d568f97b399dcc9da03691b7d8faed08c23` |
| 冻结上游 | `71da24d956378a87a703b6e1442b13aec0c4ac29` |

工具核对全部四项参考哈希。
工具核对DLL哈希与加载路径。
工具核对两段原视频哈希。
DLL哈希如下。

```text
79b61d22b4d230a00bd31930fc6943f8ea8c6174a1bf4a6bf90f2459dbf8df1b
```

## T4精度修复与复验

本轮修改两项内部CUDA计算。
内侧投影与迭代改用GPU f64。
迭代容差仍为`1e-6`。
迭代上限仍为二十轮。
方向计算先于世界接点舍入。
局部差分避开矩阵往返误差。
内部缓存六项f32方向分量。
公开物理字段继续采用f32。
外侧圆弧保留冻结f32分支。
产品链不运行Python或CPU物理。

单独提高迭代精度先修复310帧。
393帧仍因短段舍入而超限。
其短连接段约长`0.001029`米。
方向缓存随后消除剩余失配。
回归参考固定三组原生状态。
哈希清单覆盖参考与作者工具。
原四份参考文件保持原哈希。
产品测试只读取固定JSON。

我重新计算全部480个GPU源帧。
新旧实际输入精确一致。
四个审查世界沿用原编号。
本轮仍不比较其余509个世界。
原生工具重新执行1920组状态。
新旧原生结果保持字节一致。
比较器继续检查全部尾部槽位。
浮点判据沿用原`2e-5`公式。
整数判据继续要求精确一致。
本轮没有放宽容限或删除帧。

| 检查 | 修复前 | 修复后 |
| --- | ---: | ---: |
| GPU源帧 | 480 | 480 |
| 原生状态 | 1920 | 1920 |
| 比较标量 | 852480 | 852480 |
| 超限帧 | 3 | 0 |
| 超限状态 | 6 | 0 |
| 浮点超限标量 | 16 | 0 |
| 整数失配标量 | 0 | 0 |
| 全字段最大容限比 | 9.777603 | 0.147321 |

| 字段 | 修复后最大绝对差 | 超限标量 |
| --- | ---: | ---: |
| `ten_length` | 8.845451e-6 | 0 |
| `ten_J` | 2.190997e-6 | 0 |
| `wrap_xpos` | 1.583790e-6 | 0 |
| `geom_xpos` | 1.360927e-7 | 0 |
| `geom_xmat` | 2.384186e-7 | 0 |
| `site_xpos` | 3.371477e-7 | 0 |

上述最大差覆盖全部十条肌腱。
力臂最大差不再来自T4。
三帧固定回归最大差另见[球柱报告](windows-resident-geom-tendon.md)。
调试与发布各通过119项探针。
两种常驻脚本各通过49项。
宿主、Clippy与文档构建均通过。
初轮全量回归发现旧硬编码索引。
我改用布局推导索引后复验。
该测试仍检查溢出拒绝与恢复。
原生初验另缺测试DLL路径。
我运行准备脚本后复验。
全量命令见[球柱报告](windows-resident-geom-tendon.md)。

新数值证据与视频位于：

```text
target/geom-tendon-t4-review/
target/geom-tendon-t4-verification/
```

| 文件 | 内容 |
| --- | --- |
| `frame-gpu.json` | 修复后480帧结果与实际输入 |
| `native-frame-results.jsonl` | 本轮重新执行的1920组原生结果 |
| `frame-audit-summary.json` | 零失配、原容限及源码哈希 |
| `frame-audit-fields.jsonl` | 每帧每字段误差 |
| `frame-audit-failures.json` | 空失配列表 |
| `04-t4-fixed-review.mp4` | 修复后GPU与原生叠图 |
| `frame-review-310.png` | 原超限源帧310 |
| `frame-review-344.png` | 原超限源帧344 |
| `frame-review-393.png` | 原超限源帧393 |
| `frame-video-verification.json` | 编码帧数、播放尺寸与抽帧 |
| `frame-audit-bundle.zip` | 本轮数据、视频与复验脚本 |

```powershell
target/geom-tendon-t4-review/frame-export.exe fixtures/geom-tendon/wrap-tree.json target/geom-tendon-t4-review/frame-gpu.json
node target/geom-tendon-t4-review/frame-audit.cjs prepare
target/geom-tendon-t4-review/run-native-frame.ps1
node target/geom-tendon-t4-review/frame-audit.cjs check
node target/geom-tendon-t4-review/frame-render.cjs --preview
node target/geom-tendon-t4-review/frame-render.cjs
node target/geom-tendon-t4-review/frame-video-verify.cjs
node target/geom-tendon-t4-review/frame-finalize.cjs
```

新比较命令退出码为0。
视频显示端请求480帧。
实时编码器实际保留474帧。
它省略六帧显示记录。
数值复核仍覆盖480源帧。
视频时长约19.92秒。
视频不包含动力学积分。
显示弧线仍不参与数值比较。
我核对五处视频抽帧。
我另核对三张原超限源帧图。
这不代表逐帧视觉验收。

本轮未运行Python文档审计。
我人工核对本轮文档与源码。
核对范围含六份相关源码。
它另含四份修改文档。
63处本地链接均有效。
汇总脚本另修复空日志读取。
汇总复用完整GPU成功日志。
其余快速检查重新执行。
Normify保持完整G01计划态。
架构收尾返回零错误。
工具提示一项旧流程警告。
我保留其余历史变更。
本轮未验证Linux与清洁部署。
Git继续忽略临时审查产物。
复验脚本仍引用初轮目录。
请另存需要保留的证据。
