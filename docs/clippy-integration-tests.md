# 集成测试诊断收尾

## 变更范围

本轮只改两项测试。
代码未触及产品路径。
内核与夹具保持不变。
种子与容差保持不变。

修复四项诊断：

- 一项整除判断。
- 三项索引迭代。

整除改用标准方法。
切片迭代保持原顺序。
偏移仍恢复原槽位。
断言与位型不变。

## 历史基线

历史运行各失败一项。
两次都缺少报告路径。
计算完成后才触发失败。

历史日志位于：

- `docs/development-evidence/toolchain-evidence-revalidation/gpu-candidate-logs/test_mixed_mass_factor.log`
- `docs/development-evidence/toolchain-evidence-revalidation/gpu-candidate-logs/test_mixed_mass_solve.log`

本轮显式设置路径。
两项程序均通过两项。

## 当前验证

| 检查 | 结果 |
| --- | --- |
| 默认库测试 | 232项通过 |
| M1发射测试 | 5项通过 |
| 混合质量分解 | 2项通过 |
| 混合质量求解 | 2项通过 |
| 默认全目标检查 | 通过 |
| 显卡全目标检查 | 通过 |

显卡(GPU)采用RTX 4070 Ti SUPER。
驱动版本为596.36。
本轮未使用T4。

## 数值边界

分解覆盖六批数据。
它检查1236040槽。
最大误差未改变容差。
最大重构误差为5.594318963552604e-7。

求解覆盖十四批数据。
它检查141592槽。
零值槽共47656项。
每树残差保持原界限。

这些结果不是G03验收。
它们也不授予精度门禁。
发行模式尚未执行。
Linux验证尚未执行。

完整记录位于：

`docs/development-evidence/clippy-integration-tests/validation.json`
