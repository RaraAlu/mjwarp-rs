# M3速度位置FK变体G01纠正

日期：2026-10-19。
任务：`2026-10-19-layers-m3-velocity-position-g01-correction`。
基线提交：`f40a78a`。

## 问题

`update_velocity_position`自行编译运动学内核。
它总用严格的mocap变体。
G01等价计划也走这条路。
零范数四元数因此得到NaN。
同输入的`update_rigid`不产生NaN。
冻结上游按等价变体处理零范数。

## 修复

删除`resident.rs`中的严格编译点。
速度位置路径改用计划内核。
等价计划先整段复制`data.rigid`。
G01变体由此读到正确的body0。
内核随后覆盖其余FK槽位。
非等价路径不做复制。
CUDA源码保持不变。

## 复现与验证

全部运行先登记再执行。
原始日志位于`runs/`目录。

| 运行 | 结果 |
| --- | --- |
| `repro-gpu`（修复前） | 504个槽位中280个不一致 |
| `gpu-fix`（修复后） | 0个不一致，导入位姿一致 |
| `format-check` | 通过 |
| `clippy-default`、`clippy-cuda` | 通过 |
| `host-resident` | 14项通过 |
| `gpu-integration` | 16项通过 |
| `g01` | 7项通过 |
| `gpu-lib-regression` | 27项通过，1项失败 |

复现测试覆盖以下输入：

- 正零与负零四元数。
- `1e-30`下溢四元数。
- 自由关节、球关节与mocap体。
- 非单位的导入世界位姿。
- 非等价计划的单位四元数对照。

修复前补丁见`repro-test.patch`。
它只改测试模块。

## PTX

三种变体摘要修复前后一致。

| 变体 | PTX摘要 |
| --- | --- |
| plain | `788c870d…` |
| mocap | `93318e85…` |
| mocap+G01 | `58cb7893…` |

plain摘要等于上一任务捕获。

## 既有失败

单元回归有一项失败：
`flex_face_readback_rejects_guards_nonfinite_and_arithmetic_overflow`。
它与本次改动无关。
基线源码以同一断言失败。
基线复验经过四次修正：

1. 工作树检出因路径过长失败。
2. 导出源码缺少`tests`目录。
3. 导出源码缺少`docs`证据文件。
4. 相对CUDA路径导致NVRTC未加载。

第五次复现了同一失败。
该失败需要另立任务处理。

## 清单

扫描器先逐行复现基线行。
随后重建`resident.rs`全部行。
编译点由90处降为89处。
`rigid_kinematics`剩两处编译点。
未迁移入口仍为56种。
未迁移底层调用仍为71处。

## 未验收项

- 未经`PassiveForcePlan`端到端运行G01流体路径。
- 未与冻结MJWarp比较退化输入结果。
- NP01至NP04仍未验收。
- 性能与显存没有测量。
- release与Linux没有运行。
- 1042项历史超差仍然保留。
- 父任务M3仍未完成。
