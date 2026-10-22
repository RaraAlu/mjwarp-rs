# 重力包 T4 源码身份重钉（repin-11）

任务：`2026-10-23-t4-source-hash-repin-11`（登记提交 `a368d29`）。
来源：G26 能量全链（`2026-10-16-next-g26-complete`，`f594d4f`）改变产品源导出后，重力包内嵌的物理核心组合摘要漂移；repin-10（`08e7983`）已重钉其余 T4/G01 夹具，但 files 漏登记 `fixtures/gravity-upstream-abi/`，提交被范围钩子拒绝，按规范把该文件与对应测试期望拆到本任务单独提交（语义相同，不放松）。

## 变更（只改期望字面量）

| 文件 | 字段 | 旧值 | 新值 |
| --- | --- | --- | --- |
| `fixtures/gravity-upstream-abi/nonzero-product-chain-package.json` | `product_sources.physics_core.sha256`（物理核心组合摘要，锚点 `concat!` 口径） | `9d10f056…` | `7eb6e7a3c49ea2812f9b7b2ac579d9ced635d7c790613723297ba33d2718079d` |
| `fixtures/gravity-upstream-abi/nonzero-product-chain-package.json` | 包文件整体（由上项变化触发） | `39426708…` | `3afb835f40a13b9816838615713f64d092f82873acd90cbc97b7589b407f0415` |
| `tests/t4_gravity_nonzero_product_chain.rs` | `PACKAGE_SHA256` | `39426708…` | `3afb835f…` |
| `tests/t4_gravity_nonzero_product_chain.rs` | 内联 `product_sources` 期望（mod.rs 组合摘要） | `9d10f056…` | `7eb6e7a3…` |

未改：引力/运动学 receipt 哈希、输入哈希、容差、`expected` 数值、审计断言逻辑、产品 src、内核/PTX、排除清单。`git diff` 仅上述 2 个文件。

## 验证

| 项 | 结果 |
| --- | --- |
| `cargo +msvc test --locked --all-features --no-fail-fast` | **712 过 0 败 347 ignored**（79 个结果行，EXIT=0；日志 `all-features.log`） |
| 门禁 fmt / clippy / clippy-cuda / unit | 全过（unit 406 过 0 败） |
| unchanged 保护路径 | 未改 |
| `t4_gravity_nonzero_product_chain` 合约与 live | 通过（含包 SHA 与内联期望校验） |

## 边界

- 只做身份重钉，不宣称完整 G01/T4 矩阵通过超出「身份重钉 + all-features 0 失败」。
- release 与 Linux 未重跑；历史捕获产物未重生成；6 项历史捕获身份排除清单保持不变。
- 组合摘要算法与 repin-10 一致：`sha256(LF 规范化(concat(src/physics/mod.rs, com_position.rs, device_checks.rs, history.rs, kinematics.rs，以换行分隔)))`。
