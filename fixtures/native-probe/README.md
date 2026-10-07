# 原生输入探针样本

这些样本只供输入探针。
它们不表示物理参考轨迹。
本项目自行编写三个XML。
测试仅读取固定MJB。
产品构建不调用模型编译器。
产品测试不调用模型编译器。
引擎不提供MJCF编译接口。
LRsLab继续负责模型编译。

## 独立样本制备

本轮独立运行官方工具。
该步骤不属于产品工具链。
日期：2026-10-07。
平台：Windows x86_64。
工具版本：MuJoCo 3.12.0。
来源：[官方发行包](https://github.com/google-deepmind/mujoco/releases/tag/3.12.0)。
本项目不提交官方DLL或头文件。
下载脚本保留原包许可。
原包还包含第三方说明。

实际制备命令如下：

```powershell
& target/toolchains/mujoco-3.12.0/package/bin/compile.exe fixtures/native-probe/two-joints.xml fixtures/native-probe/two-joints.mjb
& target/toolchains/mujoco-3.12.0/package/bin/compile.exe fixtures/native-probe/zero-dof.xml fixtures/native-probe/zero-dof.mjb
& target/toolchains/mujoco-3.12.0/package/bin/compile.exe fixtures/native-probe/mixed-joints.xml fixtures/native-probe/mixed-joints.mjb
& target/toolchains/mujoco-3.12.0/package/bin/compile.exe fixtures/native-probe/mixed-joints.xml target/mixed-joints-reference.txt
Copy-Item -LiteralPath target/mixed-joints-reference.txt -Destination fixtures/native-probe/mixed-joints-reference.txt
```

manifest记录工具提交与SHA256。
测试脚本先校验全部样本。
它不重新生成MJB。

## 人工确定的预期值

| 样本 | 字段 | 预期值 |
| --- | --- | --- |
| 双关节 | `nq,nv,nu,na` | `2,2,0,0` |
| 双关节 | `nbody,njnt,ngeom` | `3,2,2` |
| 双关节 | `qpos0` | `[0.25,-0.5]` |
| 双关节 | `body_mass` | `[0,2,3]` kg |
| 双关节 | `body_parentid` | `[0,0,1]` |
| 双关节 | `jnt_type` | `[2,3]`；slide与hinge |
| 零自由度 | `nq,nv,njnt` | `0,0,0` |
| 零自由度 | `body_mass` | `[0]` |
| 零自由度 | `body_parentid` | `[0]` |
| 混合关节 | `nq,nv,nbody,njnt` | `13,11,6,4` |
| 混合关节 | `qpos0` | `[0.5,0.25,0.125,1,0,0,0,1,0,0,0,0.25,-0.5]` |
| 混合关节 | `body_parentid` | `[0,0,1,2,3,0]` |
| 混合关节 | `body_jntadr` | `[-1,0,1,2,3,-1]` |
| 混合关节 | `body_jntnum` | `[0,1,1,1,1,0]` |
| 混合关节 | `jnt_type` | `[0,1,2,3]`；free/ball/slide/hinge |
| 混合关节 | `jnt_bodyid` | `[1,2,3,4]` |
| 混合关节 | `jnt_qposadr` | `[0,7,11,12]` |
| 混合关节 | `jnt_dofadr` | `[0,6,9,10]` |
| 混合关节 | `body_quat` | 六组`[1,0,0,0]` |
| 混合关节 | `jnt_axis` | `[0,0,1],[0,0,1],[1,0,0],[0,1,0]` |

混合样本含一个静态子体。
测试逐项固定十二字段。
XML确定精确二进制分数。
官方打印件确认编译后结构。
打印件仅保留两位小数。
测试不以其小数充当精确值。
manifest固定打印件哈希。
Git保留官方打印件原始字节。
它不归一化换行或行尾空格。
打印件按固定产物禁用文本差异。
旧四项文件哈希保持不变。

qpos0采用关节原生单位。
slide使用米，hinge使用弧度。
世界体使用索引零。
本轮固定精确二进制值。
测试不放宽误差容限。
