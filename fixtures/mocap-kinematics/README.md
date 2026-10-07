# mocap运动学静态参考

日期：2026-10-07。
范围：G01常驻辅助子集。
参考使用MuJoCo 3.12.0。
候选版本不代表正式冻结。
样本固定种子1789。
绝对与相对容限均为2e-5。

模型包含两项mocap。
生成器反转两个mocap编号。
MJB与JSON保留反转映射。
XML保留初始编号顺序。
样本覆盖世界geom与site。
样本覆盖静态嵌套子体。
样本覆盖mocap固定后代。
样本也覆盖关节后代。
八组输入包含非单位四元数。
输入先取f32，再交原生计算。
原生输出保留f64数值。
生成器调用原生运动学与质心。

`reference.cpp`只供样本制作。
产品测试不运行生成器。
产品测试不编译XML或MJB。
测试先检查清单哈希。
清单也记录生成器与DLL哈希。
生产构建不依赖此生成器。
产品与样本工具都不启动Python。

首次制作使用MSVC编译器。
作者链接候选原生库。
作者执行以下参数顺序：

```text
reference.exe mixed-tree.xml mixed-tree.json mixed-tree.mjb
```

冻结GPU语义见[增量报告](../../docs/windows-resident-mocap.md)。
