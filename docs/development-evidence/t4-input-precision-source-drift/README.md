# T4输入精度映射源码身份复核

任务：`2026-10-19-t4-input-precision-source-drift`。
基线提交：`129d14dae66db600e4c45a129a13e9e6bb7cb956`。
平台：Rara，Windows x64 MSVC。
本次只跑宿主测试。
本次不占用GPU。

## 结论

七项失败来自过期源码身份。
被映射的内容没有变化。
本次只更新11行sha256。
产品源码保持不变。
测试代码保持不变。
映射条目保持不变。

## 失败项

以下七项均报源码漂移：

```text
field_internal::maps_all_batched_fields_and_com_steps
field_internal::flex_matrices::covers_twenty_one_groups_four_float_outputs_and_cache_publication
packing_routes::covers_eight_packers_and_all_82_routes
packing_routes::flex_internal::covers_sixteen_groups_and_six_output_routes
rigid_internal::covers_seventeen_internal_groups_and_seven_outputs
state_packing::covers_state_declarations_and_all_ten_packing_functions
topology_frames::covers_topology_internal_groups_and_outputs
```

见[修复前日志](runs/run-01-before.log)。

## 漂移来源

`4c123c3`统一刷新过源码身份。
此后两个文件继续变化。
逐提交哈希见[源码谱系](diagnosis/source-lineage.txt)。

| 文件 | 提交 | 改动 |
| --- | --- | --- |
| `resident.rs` | `40aac80` | 休眠复制改用类型化发射 |
| `resident.rs` | `5a1593b` | 速度位置FK复用计划内核 |
| `resident.rs` | `79d4494` | 只改柔体面测试断言 |
| `transfer.rs` | `4c32408` | 新增设备跨度查询 |

`flex_face.rs`也有漂移。
`face_lineage`已做历史逆映射。
该文件无需更新身份。

## 语义核对

`40aac80`保留同一`sleep_copy`内核。
它只替换发射包装。
`4c32408`缓存分配地址。
它不改字段交换数据。
`79d4494`只改测试模块。

`5a1593b`改变一条路由。
G01等价计划原用严格变体。
现在改用G01变体。
这次修改对齐冻结上游。
三种运动学PTX摘要不变。
七份清单都不映射该路由。
见[路由范围检查](runs/check-07-route-scope.log)。

测试仍逐项核对锚点。
锚点全部仍然匹配。
因此映射语义未变。

## 精度映射

本次不改任何内核。
输入、输出与中间量不变。
常量与编译选项不变。
不放宽任何容差。
不新增Clippy豁免。

## 验证

汇总见[运行摘要](runs/summary.json)。

| 编号 | 内容 | 结果 |
| --- | --- | --- |
| run-01 | 修复前五个测试文件 | 17项失败 |
| run-02 | 修复后`t4_input_precision` | 36过，0败，12忽略 |
| check-03 | `cargo fmt --check` | 通过 |
| check-04 | 默认clippy全目标 | 通过 |
| check-05 | cuda-probe clippy全目标 | 通过 |
| check-06 | 产品与测试源码无差异 | 通过 |
| check-07 | 清单不含变更路由 | 通过 |

## 未验收项

- 另有10项同根因失败。
- 它们位于四个测试文件。
- 涉及camlight与spatial-tendon。
- 也涉及tendon-wake与重力链。
- tendon-wake映射涉及`sleep_copy`。
- 它们需另建任务复核。
- 未跑GPU、Linux与release。
- 未验收NP01至NP04。
- 未宣称精度达标。
