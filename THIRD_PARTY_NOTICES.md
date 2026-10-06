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
