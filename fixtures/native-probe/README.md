# 原生输入探针样本

这些样本只供输入探针。
它们不表示物理参考轨迹。
本项目自行编写两个XML。
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

qpos0采用关节原生单位。
slide使用米，hinge使用弧度。
世界体使用索引零。
本轮固定精确二进制值。
测试不放宽误差容限。
