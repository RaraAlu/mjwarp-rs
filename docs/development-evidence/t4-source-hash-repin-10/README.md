# G26 后的 T4 源码身份重钉（repin-10）

任务：`2026-10-23-t4-source-hash-repin-10`（登记提交 `eceaa0e`）。
触发：G26 能量全链交付（`2026-10-16-next-g26-complete`，提交 `f594d4f`）修改了产品源 `src/physics/mod.rs`、`src/physics/mass_matrix.rs`、`src/physics/mass_matrix/multiply*`、`src/physics/kinetic_energy*`、`src/physics/mass_matrix/resident.rs`、`src/model/data/versions.rs`。T4/G01 历史审计按整文件/组合源码摘要记录身份，`--all-features` 下 11 项身份审计因此失败；本任务只把期望摘要重钉到当前源码文本。

## 边界（只改期望，不改语义）

- 只改期望哈希/摘要字面量与包文件内的 `product_sources` 摘要；**不改审计断言逻辑、不放宽容差、不改产品 src、不改内核/PTX、不伪造历史捕获、不改排除清单**。
- 谱系逆变换函数（`rotate_lineage`、`matrix_lineage`、`historical_rigid_kernel` 等）代码一行未改；只更新它们在当前源码文本上的期望摘要。
- 无产品代码、无容差、无测试选择器变化；`git diff` 仅 11 个 fixtures/evidence/tests 文件、13 行增删。

## 漂移清单（旧 → 新）

物理核心组合摘要 = `sha256(LF 规范化(concat(src/physics/mod.rs, com_position.rs, device_checks.rs, history.rs, kinematics.rs，以换行分隔)))`（与 `docs/development-evidence/t4-resident-anchors/physics.rs` 的 `concat!` 口径一致），由 G26 新增模块导出而变化。

| 文件 | 字段 | 旧值 | 新值 |
| --- | --- | --- | --- |
| `fixtures/precision/g01-field-internal.json` | product source mod.rs 组合摘要 | `9d10f056…` | `7eb6e7a3c49ea2812f9b7b2ac579d9ced635d7c790613723297ba33d2718079d` |
| `fixtures/precision/g01-rigid-internal.json` | 同上 | `9d10f056…` | `7eb6e7a3…` |
| `fixtures/precision/g01-state-packing.json` | 同上 | `9d10f056…` | `7eb6e7a3…` |
| `fixtures/precision/g01-topology-frames.json` | 同上 | `9d10f056…` | `7eb6e7a3…` |
| `fixtures/precision/t4-current-kernels.json` | `product_sources` mod.rs | `9d10f056…` | `7eb6e7a3…` |
| `fixtures/gravity-upstream-abi/nonzero-product-chain-package.json` | `product_sources.physics_core` | `9d10f056…` | `7eb6e7a3…` |
| `docs/evidence/t4-g03-retest/camlight-internal/map.json` | `src/physics/mod.rs`（锚点组合） | `9d10f056…` | `7eb6e7a3…` |
| `docs/evidence/t4-g03-retest/tendon-wake-internal/map.json` | 同上 | `9d10f056…` | `7eb6e7a3…` |
| `tests/support/t4_rigid_internal/tests.rs` | `rotate_lineage::before(PHYSICS)` 期望 | `714839bf…` | `d9c8cfabde82f7364b508742675783a2c535fd83753c7606f2e61c94d9ebe525` |
| `fixtures/gravity-upstream-abi/nonzero-product-chain-package.json` | 包内嵌 `PRODUCT_SOURCES` 同步 | `9d10f056…` | `7eb6e7a3…` |
| `tests/t4_gravity_nonzero_product_chain.rs` | `PACKAGE_SHA256` | `39426708…` | `3afb835f40a13b9816838615713f64d092f82873acd90cbc97b7589b407f0415` |
| `tests/t4_gravity_nonzero_product_chain.rs` | 内联 `product_sources` 期望 | `9d10f056…` | `7eb6e7a3…` |

## 方法

1. 用 `--all-features --no-fail-fast` 逐轮枚举失败，区分「源码身份漂移」与其它失败；11 项均为摘要漂移。
2. 物理核心组合摘要按锚点 `concat!` 口径离线复算（pinned Python 仅作证据计算，不进产品链）：`sha256(concat LF)` = `7eb6e7a3…`。
3. 旋转谱系摘要由 `rotate_lineage::before(组合源码)` 离线复算 = `d9c8cfab…`。
4. 重力包先改内部 `product_sources` 摘要，再按文件字节重算 `PACKAGE_SHA256` 并同步测试常量。
5. 每轮回归 `cargo +msvc test --locked --all-features --no-fail-fast`，直到 0 失败。

## 验证

| 项 | 命令 | 结果 |
| --- | --- | --- |
| 全量 all-features | `cargo +msvc test --locked --all-features --no-fail-fast` | **712 过 0 败 347 ignored**（79 个结果行，EXIT=0；重钉前 701 过 11 败） |
| 门禁 | fmt / clippy / clippy-cuda / unit | 全过（unit 406 过 0 败） |
| 保护路径 | unchanged native/include/Cargo.*/rust-toolchain/两个验收脚本 | 未改 |
| 提交范围 | `git diff` | 仅 11 个 fixtures/evidence/tests 文件，13 行增删；无 src 变更 |

日志：`all-features.log`（本轮 0 失败）。重钉前失败清单见任务登记说明与本目录 `drift.json`。

## 未验收边界

- 不重跑 release 与 Linux；不宣称完整 G01/T4 矩阵通过超出「身份重钉 + all-features 0 失败」。
- 未重生成历史捕获产物（历史审计的 6 项排除清单保持不变，仍由 `2026-10-22-t4-historical-capture-rebaseline` 跟踪）。
- 历史谱系逆变换仍只回溯已实测的固定修复；不改写历史结论。
