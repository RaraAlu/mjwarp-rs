# T4常驻源码身份复核

任务：`2026-10-19-t4-resident-source-drift`。
前置：`2026-10-19-t4-input-precision-source-drift`。
基线提交：`bc9ff16c064c8d717ca5c212ada805c5af98e846`。
平台：Rara，Windows x64 MSVC。
本次只跑宿主测试。
本次不占用GPU。

## 结论

十项失败来自过期源码身份。
被映射的内容没有变化。
本次只更新哈希。
产品源码保持不变。
映射条目保持不变。

## 失败项

以下十项均报源码漂移：

```text
t4_camlight_precision::checks_two_stage_dependencies_and_keeps_numerical_admission_open
t4_camlight_precision::maps_eighteen_internal_groups_and_four_public_f32_outputs
t4_spatial_tendon_precision::binds_generated_artifacts_and_maps_physical_library_f64
t4_spatial_tendon_precision::checks_stage_order_without_admitting_atomic_or_device_equivalence
t4_spatial_tendon_precision::distinguishes_zero_and_tiny_normalization_branches
t4_spatial_tendon_precision::maps_sixteen_site_chain_groups_and_six_mixed_outputs
t4_tendon_wake_precision::inspects_existing_ptx_float_distances_and_signed_integer_updates
t4_tendon_wake_precision::maps_eighteen_mixed_scalar_groups_and_five_integer_outputs
t4_tendon_wake_precision::maps_transaction_publication_and_diagnostic_only_readback_conversion
t4_gravity_nonzero_product_chain::frozen_nonzero_product_chain_contract_is_self_consistent
```

见[修复前日志](runs/run-01-before.log)。

## 改动

| 文件 | 改动 |
| --- | --- |
| `camlight-internal/map.json` | 一行`resident.rs`哈希 |
| `spatial-tendon-internal/map.json` | 一行`resident.rs`哈希 |
| `tendon-wake-internal/map.json` | 一行`resident.rs`哈希 |
| `nonzero-product-chain-package.json` | 一行`resident.rs`哈希 |
| `t4_gravity_nonzero_product_chain.rs` | 两个哈希常量 |

重力测试有两层固定。
第141行复写包内源码身份。
第27行固定整包摘要。
包摘要随之更新。
新摘要为`c4752d60…d199`。
见[范围检查](runs/check-06-scope.log)。

## 漂移来源

`4c123c3`统一刷新过源码身份。
此后`resident.rs`经三次提交。
逐提交改动见[源码谱系](diagnosis/source-lineage.txt)。

| 提交 | 改动 |
| --- | --- |
| `40aac80` | 休眠复制改用类型化发射 |
| `5a1593b` | 等价计划速度位置FK改用计划内核 |
| `79d4494` | 只改柔体面测试断言 |

三个更新函数均未改动。
它们是相机光源、空间肌腱和主更新。

## 休眠复制核对

肌腱唤醒映射涉及`sleep_copy`。
`40aac80`改了它的发射方式。
本次逐项核对该改动。
见[语义检查](runs/check-07-sleep-copy-semantics.log)。

- 休眠CUDA源码未变。
- 包装仍编译同一入口。
- 维度仍为`[0,0,0,worlds]`。
- 包装仍转交同一ABI。
- 网格仍为256线程一块。
- 复制、唤醒、交换顺序不变。
- 元数据与参数内容不变。
- 新增检查只拦截无效输入。
- 映射捕获摘要不变。
- 它们与`40aac80`证据一致。

因此映射语义未变。
锚点全部仍然匹配。

## 其他路由

`5a1593b`只改等价计划路由。
四份清单都不映射该路由。
重力实测走`update`主路径。
它不调用速度位置入口。

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
| run-01 | 修复前四个测试文件 | 10项失败 |
| run-02 | 修复后四个测试文件 | 12过，0败，1忽略 |
| check-03 | `cargo fmt --check` | 通过 |
| check-04 | 默认clippy全目标 | 通过 |
| check-05 | cuda-probe clippy全目标 | 通过 |
| check-06 | 改动范围 | 通过 |
| check-07 | 休眠复制语义 | 通过 |

## 未验收项

- 重力GPU实测未重跑。
- `gpu-3`证据绑定旧包摘要。
- 旧包摘要为`9154df31…69ac`。
- 新包只改源码身份。
- 下次实测将绑定新包。
- 40aac80证据保留旧映射摘要。
- 那是已关闭任务的历史记录。
- 未跑GPU、Linux与release。
- 未验收NP01至NP04。
- 未宣称精度达标。
