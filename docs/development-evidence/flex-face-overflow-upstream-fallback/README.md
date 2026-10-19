# 柔体面溢出测试对齐上游回退

任务：`2026-10-19-flex-face-overflow-upstream-fallback`。
基线提交：`5a1593ba1e7f45d3275c895df1e79e7ff57a8781`。
平台：Windows，RTX 4070 Ti SUPER，驱动596.36。
资源：`gpu:windows-t4`，即本机该卡。

## 结论

产品内核符合冻结上游。
测试期望已经过时。
本次只改测试与文档措辞。
产品源码保持不变。

## 失败来源

失败测试如下：

```text
physics::resident::tests::flex_face_readback_rejects_guards_nonfinite_and_arithmetic_overflow
```

提交`97a7b54`改写面极分解归一化。
它按Warp改为倒数乘法。
长度非正时返回单位四元数。
父提交`d2d7b46`该测试通过。
见[父提交日志](diagnosis/parent-d2d7b46.log)。
HEAD`5a1593b`该测试失败。
见[HEAD探针日志](diagnosis/head-5a1593b-probe.log)。
探针只加一行打印。
见[探针补丁](diagnosis/readback-probe.patch)。
探针代码位于忽略目录。
它不进入本次提交。

## 数值路径

节点放大到`1e20`。
切向量叉积超出f32范围。
法向量因此含无穷。
omega与长度w变为NaN。
`w<1e-6`对NaN为假。
迭代继续执行。
四元数长度n也为NaN。
`n>0`对NaN为假。
内核返回单位四元数。
旧实现逐分量除以n。
NaN因此进入输出。
旧测试依赖这一行为。

HEAD实测world 512结果：

| 字段 | 读回值 |
| --- | --- |
| `face_xpos` | 原节点值，含`1e20`，全部有限 |
| `face_quat` | 六个面均为`[0,0,0,1]` |

有限输入不会产生非有限输出。
位置只是节点拷贝。
四元数要么归一化，要么回退。
提前退出时保留上轮有限值。

## 上游对照

上游提交`71da24d9`。
`support.py`定义`mat33_to_quat_polar`。
每轮调用`wp.normalize`。
Warp版本为1.15.0。
`quat.h`的`normalize`比较`l > kEps`。
`kEps`等于零。
否则返回`(0,0,0,1)`。
NaN同样走回退分支。
[面归一化对照](../../t4-face-normalization-control.md)已核对PTX谓词。
产品`setp.gt`与上游`setp.leu`互补。

## 精度映射

本次不改任何内核。
输入、输出与中间量仍为f32。
回退常量与阈值不变。
测试只改断言目标。
测试不放宽任何容差。
`97a7b54`记录的2078项面位差保留。

## 验证

汇总见[运行摘要](runs/summary.json)。

| 编号 | 命令要点 | 结果 |
| --- | --- | --- |
| run-01 | 目标测试 | 1过，0.79秒 |
| run-02 | `physics::resident::`单元GPU | 21过，25.31秒 |
| check-03 | `cargo fmt --check` | 通过 |
| check-04 | 默认clippy全目标 | 通过 |
| check-05 | cuda-probe clippy全目标 | 通过 |
| check-06 | `flex_face.rs`对基线无差异 | 通过 |

check-05复用同目录检查缓存。
此前同源码`--lib --tests`检查已通过。
GPU测试未记录显存。

## 未验收项

- 未运行完整GPU回归。
- 未运行Linux与release。
- 未验收NP01至NP04。
- 未宣称柔体面精度达标。
