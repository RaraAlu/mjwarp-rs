# 第三方源码说明

## 纯数学公式

`src/math/mod.rs`移植八项公式。
本项目冻结以下上游版本：

- 项目：Google DeepMind MuJoCo Warp。
- 提交：`71da24d956378a87a703b6e1442b13aec0c4ac29`。
- 文件：`mujoco_warp/_src/math.py`。
- 版权：Copyright 2025 The Newton Developers。
- 许可：Apache License 2.0。
- 本地许可：`LICENSES/Apache-2.0.txt`。

本项目改用Rust数组与f32。
本项目添加宿主单元测试。
这些公式不执行GPU调度。
这份说明不选择项目总许可。

[上游源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py)

## 刚体运动学CUDA探针

`src/physics/mod.rs`移植刚体子集。
CUDA部分复用上述许可。
上游版权归Newton开发者。
算法依据`smooth.py`与`math.py`。
冻结提交号保持不变。
本项目改用CUDA C++与f32。
本项目按世界顺序遍历体。
本项目不移植mocap等分支。
本项目添加同步与范围检查。
本项目另移植质心与惯量子集。
零质量分支沿用冻结Warp。
它不代表完整物理阶段。

[上游刚体源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)

## 原生模型输入候选

探针采用MuJoCo 3.12.0。
它只读取可信已编译MJB。
本项目不提交官方DLL或头文件。
下载脚本保留官方许可。
原包包含第三方许可说明。
MuJoCo采用Apache-2.0。
本版本仍属于探针候选。
本轮不冻结生产输入版本。

[官方发行包](https://github.com/google-deepmind/mujoco/releases/tag/3.12.0)
[官方许可](https://github.com/google-deepmind/mujoco/blob/3.12.0/LICENSE)

`native-model-probe`可选接入cc。
本项目固定cc版本1.6.0。
cc采用MIT或Apache-2.0。
它只负责调用原生编译器。
默认构建不启用cc。

[cc包元数据](https://crates.io/crates/cc/1.6.0)
