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
独立刚体探针不处理mocap。
常驻入口另处理mocap分支。
本项目添加同步与范围检查。
本项目另移植质心与惯量子集。
零质量分支沿用冻结Warp。
它不代表完整物理阶段。

[上游刚体源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)

## 几何与site运动学CUDA探针

`src/physics/attached.rs`移植附着变换。
公式来自冻结`smooth.py`。
版权与Apache-2.0许可沿用上述条款。
本项目复用刚体CUDA四元数公式。
GPU输出位置与按行展开矩阵。
独立探针每次重算静态geom。
常驻入口另保留GPU静态缓存。
它不代表完整G01实现。

## 柔体位置CUDA辅助

`src/physics/flex.rs`移植位置子集。
公式来自冻结`smooth.py`。
权重依据冻结`support.py`。
版权归Newton开发者。
许可采用Apache-2.0。
本地许可保留上述文件。
本项目改用同步CUDA C++。
每个线程独占一个世界。
线程先更新节点再更新顶点。
公开物理输入与输出采用f32。
本项目支持线性壳体位置。
本项目仍拒绝高阶插值。
冻结节点辅助不执行内部TFI。
它不提供完整柔体阶段。

[冻结位置源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L211-L317)。
[冻结线性权重](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L1004-L1013)。

## 相机与光源CUDA辅助

`src/physics/camlight.rs`移植位姿分支。
公式来自冻结`smooth.py`。
版权与Apache-2.0沿用上述条款。
本项目改用两段同步CUDA调度。
第一段复用刚体设备结果。
第二段复用子树质心结果。
本项目添加独立字段周期。
本项目不实现渲染与休眠。
它不代表完整G01实现。
方向归一化参考Warp 1.15.0。
该版本将kEps设为零。
上游版权归NVIDIA。
其公式采用Apache-2.0许可。

[冻结相机光源源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L773-L938)。
[Warp归一化](https://github.com/NVIDIA/warp/blob/v1.15.0/warp/native/vec.h#L1033-L1040)。
[Warp零阈值](https://github.com/NVIDIA/warp/blob/v1.15.0/warp/native/builtin.h#L527)。

## 固定肌腱CUDA辅助

`src/physics/fixed_tendon.rs`移植关节项。
公式来自冻结`_joint_tendon`。
版权与Apache-2.0沿用上述条款。
本项目每线程独占一个世界。
本项目顺序累加固定长度。
本项目保留原生CSR列编号。
本项目拒绝同腱重复关节。
固定子集不计算空间绕行。
它不代表完整G01实现。

[冻结肌腱源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。

## 空间肌腱CUDA辅助

`src/physics/spatial_tendon.rs`移植site及geom段。
它复用冻结祖先链力臂公式。
它复用归一化与包裹写入语义。
上游文件为`smooth.py`与`math.py`。
版权与Apache-2.0沿用上述条款。
本项目顺序累加每个世界。
本项目将滑轮除数转为共享比例。
本项目分三段调度CUDA内核。
整数段保留原生双点容量。
本项目另复用球柱绕行辅助。
它不代表完整G01实现。

[冻结空间肌腱源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。
[冻结归一化源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py)。

## 球柱绕行CUDA辅助

`src/physics/tendon_wrap.rs`移植绕行公式。
公式来自冻结`util_misc.py`。
版权归Newton开发者。
许可采用Apache-2.0。
本地许可路径沿用上述条款。
本项目改用CUDA C++与f32。
本项目保留球面退化平面分支。
本项目保留圆柱螺旋与内侧牛顿迭代。
反三角夹紧参考Warp 1.15.0。
版权：Copyright (c) 2022 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
该源码采用Apache-2.0。
本项目没有新增运行时依赖。

[冻结绕行源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/util_misc.py)。
[Warp反三角源码](https://github.com/NVIDIA/warp/blob/v1.15.0/warp/native/builtin.h)。

## 柔体边CUDA子集

`src/physics/flex_edge.rs`移植边公式。
它复用冻结`smooth.py`的边内核。
它复用`math.py`的零长度分支。
版权：Copyright 2025 The Newton Developers。
上游采用Apache-2.0。
本项目保留[Apache-2.0许可](LICENSES/Apache-2.0.txt)。
本项目改用每世界串行计算。
本项目添加CSR检查与守卫。
本项目每轮清零预留稀疏槽。
本项目不移植完整柔体阶段。
本批没有新增运行时依赖。

[冻结柔体边源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L347-L414)。

## 柔体面CUDA子集

`src/physics/flex_face.rs`移植面公式。
`src/model/flex.rs`移植面节点映射。
公式来自冻结`smooth.py`。
映射与迭代来自冻结`support.py`。
版权归2025年Newton开发者。
许可采用Apache-2.0。
本地许可保留上述文件。
本项目只支持线性壳体面。
本项目改用同步CUDA C++。
旋转沿用五十轮冻结迭代。
面四元数保留xyzw顺序。
本项目不移植完整柔体阶段。
本项目没有新增依赖。

[冻结面与节点源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py)。
[冻结面映射与极分解](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py)。

## 肌腱唤醒CUDA子集

`src/physics/sleep.rs`移植冻结唤醒公式。
它复用`sleep.py`的树唤醒逻辑。
它复用肌腱限位条件与轻量刷新。
版权：Copyright 2026 The Newton Developers。
上游采用Apache-2.0。
本项目保留[Apache-2.0许可](LICENSES/Apache-2.0.txt)。
本项目改用每世界串行扫描。
本项目增加检查与双缓冲。
本项目不移植完整休眠状态机。
本批没有新增运行时依赖。

[冻结休眠源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sleep.py)。

## 质量矩阵CUDA探针

`src/physics/mass_matrix.rs`移植刚体CRB。
它复用冻结惯量向量乘法。
上游文件为`smooth.py`与`math.py`。
版权与Apache-2.0许可沿用上述条款。
本项目改用顺序CUDA C++调度。
本项目输出对称稠密f32矩阵。
本项目不移植稀疏布局与休眠。
本项目不叠加肌腱与执行器惯量。
本项目添加完成与有限性检查。
它不代表完整G02实现。

## 质量矩阵求解CUDA探针

`src/physics/mass_solve.rs`复用反向LDL代数。
代数来自冻结`smooth.py`稀疏分解。
版权与Apache-2.0许可沿用上述条款。
本项目改用稠密顺序CUDA C++调度。
本项目使用内部GPU f64计算。
本项目保留公开f32输入与结果。
本项目不移植分块或稀疏布局。
本项目添加主元、完成与有限性检查。
它不代表完整G03实现。

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
