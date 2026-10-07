# 球柱绕行静态参考

范围：G01严格辅助子集。
原生版本固定为MuJoCo 3.12.0。
冻结上游提交见清单。
清单锁定七份文件哈希。
产品测试只读取固定JSON。
产品测试不启动Python。
产品测试不生成参考。

开发者单独编译`reference.cpp`。
该工具先检查原生版本。
它通过原生C++编译XML。
它输出MJB与JSON参考。
首尾参数依次指定三份路径。
工具不进入产品构建链。

```text
reference.exe wrap-tree.xml wrap-tree.json wrap-tree.mjb
```

模型包含五个体与三类关节。
关节覆盖hinge、slide与ball。
模型含四个geom与二十个site。
肌腱数量为十。
CSR槽位数量为十。
包裹记录数量为三十七。
一个mocap体移动端点。
球柱既包含静态体也包含运动体。
路径覆盖多geom与前置滑轮。
侧向site覆盖外侧、内侧与缺省。
路径覆盖长弧与端点内含分支。

参考包含十二组确定性状态。
`seed=2099`只标识样本组。
工具不调用随机数生成器。
状态与尺寸先舍入为f32。
原生计算随后采用f64。
三组半径乘数为1、0.8与1.2。
四轮状态切换有效包裹数量。
直接结果固定绝对与相对容限。
两种容限均采用`2e-5`。
整数输出要求精确一致。

## 内侧运动回归

新增参考固定三组失配状态。
源帧编号为310、344与393。
源世界编号为2。
输入保留逐帧导出的f32数值。
GPU测试另核对世界512。
测试保留原有直接比较容限。

开发者编译`motion-reference.cpp`。
该工具加载既有MJB。
它不调用XML编译器。
它独立运行原生运动学与肌腱。
产品测试只读取新增JSON。
原四份文件保持原哈希。
新增文本固定采用LF换行。

```text
motion-reference.exe wrap-tree.mjb inside-cylinder-motion-inputs.txt inside-cylinder-motion.json
```

详细证据见[球柱报告](../../docs/windows-resident-geom-tendon.md)。
逐帧证据见[复核报告](../../docs/windows-geom-tendon-frame-review.md)。
