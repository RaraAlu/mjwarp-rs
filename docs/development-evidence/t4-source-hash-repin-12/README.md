# 批次 1–4 后 T4 源码身份重钉（repin-12）

任务：`2026-10-23-t4-source-hash-repin-12`。日期：2026-10-23。执行：Backend。
性质：**只重钉冻结身份期望字面量**；未改断言逻辑、容差、排除清单、产品源码/内核 PTX。

## 1. 结论（先给）

批次 1–4 的 20 条路线图交付在 `src/physics/mod.rs` 注册新模块、G24 改动
`history.rs` 等，属合法交付；但 T4/G01 历史审计按**组合源码摘要**记录的身份因此漂移，
不是产品数值回归。按 Chief 指示先枚举后一次重钉：

- 枚举前（`--all-features --no-fail-fast`）：**784 过 / 12 败 / 353 ignored**，
  6 个 target 失败（lib + 5 个 T4 精度/重力 target），全部为身份摘要断言。
- 重钉后同命令：**796 过 / 0 败 / 353 ignored**，82 条 result 行，`CARGO_EXIT=0`。
- 失败 target 定向复跑亦 0 失败；默认特性 `cargo test --locked` 门禁通过。

## 2. 漂移与重钉值

| 项 | 旧值 | 新值 | 位置 |
| --- | --- | --- | --- |
| 物理核心组合摘要 | `7eb6e7a3…` | `d7f6c7d1…` | `fixtures/precision/{g01-field-internal,g01-rigid-internal,g01-state-packing,g01-topology-frames,t4-current-kernels}.json`、gravity 两包 `product_sources.physics_core`、camlight/tendon evidence map、gravity nonzero 测试内联期望 |
| 旋转谱系期望 | `d9c8cfab…` | `a9d1b4d6…` | `tests/support/t4_rigid_internal/tests.rs` |
| gravity nonzero 包 sha | `3afb835f…` | `ba40ffcd…` | 包文件字节变化后 `tests/t4_gravity_nonzero_product_chain.rs` 的 `PACKAGE_SHA256` |
| gravity three-d 包 sha | `476d648c…` | `95b6818d…` | `tests/t4_gravity_3d_product_chain.rs` 的 `PACKAGE_SHA256` |
| t4 current inventory sha | `bafa1f32…` | `8a4b6e1f…` | `fixtures/precision/t4-current-capture-status.json` |

组合摘要算法（与锚点 `docs/development-evidence/t4-resident-anchors/physics.rs` 一致）：
LF-normalized 的 `src/physics/{mod,com_position,device_checks,history,kinematics}.rs`
按顺序以换行连接后 sha256。旧值最后匹配 HEAD `6e8c79a`，首个破坏提交 `70b36b7`
（G27@P2 交付）；新值为当前 HEAD 实测 `d7f6c7d1…`。

## 3. 枚举失败清单（重钉前，12 条）

- `--lib`：`runtime::transfer::kernel::recapture::tests_host::preserves_current_sources_and_all_thirty_variants`
  （`unexpected lineage change`）
- `--test t4_camlight_precision`：`maps_eighteen_internal_groups_and_four_public_f32_outputs`（source drift）
- `--test t4_gravity_3d_product_chain`：`contract::frozen_three_d_package_contract_is_self_consistent`（package identity）
- `--test t4_gravity_nonzero_product_chain`：`contract::frozen_nonzero_product_chain_contract_is_self_consistent`（physics_core digest）
- `--test t4_input_precision`（5）：`field_internal::tests::maps_all_batched_fields_and_com_steps`、
  `rigid_internal::tests::covers_seventeen_internal_groups_and_seven_outputs`、
  `rigid_internal::tests::preserves_zero_quaternion_evidence_and_active_macros`、
  `state_packing::tests::covers_state_declarations_and_all_ten_packing_functions`、
  `topology_frames::tests::covers_topology_internal_groups_and_outputs`
- `--test t4_tendon_wake_precision`（3）：`maps_eighteen_mixed_scalar_groups_and_five_integer_outputs`、
  `maps_transaction_publication_and_diagnostic_only_readback_conversion`、
  `inspects_existing_ptx_float_distances_and_signed_integer_updates`

## 4. 变更文件（14 个，仅字面量）

fixtures/precision：`t4-current-kernels.json`、`t4-current-capture-status.json`、
`g01-field-internal.json`、`g01-rigid-internal.json`、`g01-state-packing.json`、
`g01-topology-frames.json`；fixtures/gravity-upstream-abi：`nonzero-product-chain-package.json`、
`three-d-package.json`；docs/evidence/t4-g03-retest：`camlight-internal/map.json`、
`tendon-wake-internal/map.json`；tests：`support/t4_rigid_internal/tests.rs`、
`t4_gravity_nonzero_product_chain.rs`、`t4_gravity_3d_product_chain.rs`。

未改：断言逻辑、容差、排除清单、产品 src、内核 PTX、G01/原生脚本、历史捕获产物、
`docs/development-evidence/frozen-inventory/inventory.json`（历史 frozen/mid/head 快照）。

## 5. 验证（本次实测）

| 项 | 结果 |
| --- | --- |
| 重钉前全量 `--all-features --no-fail-fast` | 784 过 / 12 败 / 353 ignored（6 target） |
| 重钉后全量同命令 | **796 过 / 0 败 / 353 ignored**，`CARGO_EXIT=0` |
| 失败 target 定向复跑 | lib recapture、t4_camlight_precision、t4_gravity_3d、t4_gravity_nonzero、t4_input_precision、t4_tendon_wake_precision 均 0 失败 |
| 默认特性 unit 门禁 | `cargo test --locked` 通过 |
| 门禁 | fmt / clippy / clippy-cuda / unit 全过 |
| unchanged 保护路径 | native/include/Cargo/rust-toolchain/G01 脚本未改 |

环境：Windows MSVC 1.99.0；`MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/package`
（`MJWARP_MUJOCO_DLL` 同目录 `bin/mujoco.dll`）。

## 6. 复现

```powershell
$env:MJWARP_MUJOCO_ROOT = (Resolve-Path 'target/toolchains/mujoco-3.12.0/package').Path
$env:MJWARP_MUJOCO_DLL = Join-Path $env:MJWARP_MUJOCO_ROOT 'bin/mujoco.dll'
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast
cargo test --locked
```

## 7. 边界

- 只改身份期望字面量；release 与 Linux 未重跑；历史捕获产物未重生成。
- 排除清单（既有 11 项）与 G01 脚本未改；若后续 all-probes 因新模块需新 env/新排除，
  按 Chief 要求停手报告，不自行改排除清单。
- 不宣称完整 G01/T4 矩阵通过超出身份重钉范围。
