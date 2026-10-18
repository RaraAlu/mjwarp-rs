# M1发射边界补证

日期：2026-10-18。
任务：`2026-10-18-m1-bounds-evidence`。
关联：已关闭的`2026-10-18-layers-m1`。
本项不需要GPU。
本项不覆盖NP01至NP04。

## 结论

1维发射边界已有静态证据。
大小24字节，对齐8字节。
与`LaunchBounds1`一致。
4维发射边界已删除。
签名检查拒绝4维发射边界。
数组仍支持1至4维。

## 两类维度

| 参数 | 含义 | 支持维度 | 本项处理 |
| --- | --- | --- | --- |
| `SlotKind::Bounds { dims }` | `launch_bounds_t<N>`发射网格 | 1至3 | 收紧，删除`LaunchBounds4` |
| `SlotKind::WarpArrayData { ndim }` | `array_t`数组形状 | 1至4 | 保留 |

`WarpArray`保留`shape[4]`与`strides[4]`。
56字节布局与字段偏移不变。

冻结上游`_src`共有454处`wp.launch`。
其中没有4元`dim`。
因此删除4维边界不减少上游功能。

## 静态样本

样本取自一份冻结生成产物：

| 项目 | 值 |
| --- | --- |
| `.cu` | `fixtures/gravity-upstream-abi/generated/module.cu` |
| `.ptx` | `fixtures/gravity-upstream-abi/generated/module.ptx` |
| `.cu`摘要 | `dd9338e9f38feb767273c3e1a445aea54bee3dee7d899076363202a4d26e865d` |
| `.ptx`摘要 | `d06154da63f64118e8fdb19c0b646a59c5b485d580f4716e5edea97a57c1dd70` |
| 上游提交 | `71da24d956378a87a703b6e1442b13aec0c4ac29` |
| Warp | 1.15.0 |

两项摘要与同目录`receipt.json`一致。
它们也与`src/runtime/warp_gravity.rs`常量一致。

样本清单固定，不依赖全局扫描：

| 入口 | 发射维度 | PTX首参数 | Rust类型 |
| --- | ---: | --- | --- |
| `_qfrc_adhesion_8d86bfc5` | 1 | 对齐8，24字节 | `LaunchBounds1` |
| `_eff_contact_build_f1a6f426` | 1 | 对齐8，24字节 | `LaunchBounds1` |
| `_eff_contact_force_3892154f` | 1 | 对齐8，24字节 | `LaunchBounds1` |
| `_spring_damper_flexedge_passive_ade4181f` | 2 | 对齐8，24字节 | `LaunchBounds2` |
| `_gravity_force_7c63ac28` | 3 | 对齐8，32字节 | `LaunchBounds3` |

入口名省略`_cuda_kernel_forward`后缀。
1维样本为该产物全部1维内核。
2维与3维作对照。
3维入口即L2实际运行的重力入口。

全部样本的其余结构参数均为56字节。
重力入口共有11个数组参数。
与回执登记的11项一致。

机器可读清单见`development-evidence/m1-bounds-evidence/samples.json`。
测试副本见`tests/support/launcher_bounds_evidence.rs`。

## 证据边界

静态结果只证明首参数大小与对齐。
它不证明字段顺序。
它也不证明运行语义。
真实冻结内核运行验证留给M4。

## 验证

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | 通过 |
| `cargo test --locked --lib` | 232通过，0失败 |
| `cargo test --locked --test launcher_abi` | 18通过，0失败 |
| `cuda-probe`下`launcher_abi` | 19通过，0失败；另含摘要核对 |
| `cuda-probe`下发射器单测 | 10通过，0失败 |
| clippy（两种配置） | 本项范围0项诊断 |

工具链为stable-x86_64-pc-windows-gnu与msvc，rustc 1.99.0。
完整输出位于`development-evidence/m1-bounds-evidence/`。

新增测试如下：

- `frozen_launch_bounds_match_ptx_first_parameter`
- `frozen_bounds_samples_keep_recorded_digests`（仅`cuda-probe`）
- `rejects_four_dimensional_launch_bounds`
- `accepts_four_dimensional_warp_arrays`

摘要核对需要`sha2`。
它只在`cuda-probe`下可用。
默认配置只核对摘要格式。

## 既有诊断变化

默认配置clippy诊断由18项变为24项。
增量不在本项范围。
上一项修复后，默认库测试才能编译。
clippy因此首次检查这些测试代码。

其中`fluid_force.rs`有4项`excessive_precision`。
它们是逐位冻结的上游常量。
治理时只能豁免，不能缩短字面量。
该事项转交`2026-10-18-toolchain-clippy-governance`。

## 未覆盖项

- 字段顺序与运行语义未验证。
- 未运行GPU测试。
- Linux未验证。
- 物理调用未迁移。
