# T4源码身份重钉

任务：`2026-10-22-t4-source-hash-repin`。
前置：`2026-10-21-t4-resident-source-anchors`。
基线提交：`6803311`（任务登记），代码父提交 `77e4acd`。
先例：`2026-10-19-t4-input-precision-source-drift`、`2026-10-19-t4-resident-source-drift`。

## 结论

拆分后过期的源码身份 sha256 已重钉。
受影响六个测试目标的十六项失败全部转绿。
`cargo test --features cuda-probe --no-fail-fast` 全套无失败目标。
映射条目、断言、容差与产品逻辑未改动。
仅恢复拆分时丢失的两行纯注释。

## 重钉口径

拆分模块取「模块根加产品子模块」组合，模块与文件之间以单个换行连接，摘要按 CRLF→LF 归一口径。
未拆分文件取当前全文同口径摘要。

| 源码路径 | 旧摘要 | 新摘要 | 涉及证据 |
| --- | --- | --- | --- |
| `src/physics/resident.rs` | `d0cdb116…a174216a` | `919335fb…d5c5aeaf` | 五个 precision 夹具、五个 t4-g03-retest map、重力包 |
| `src/physics/mod.rs` | `e9ec17db…f6235019e` | `09da2b79…dd6fe09` | 四个 precision 夹具、camlight 与 tendon-wake map、`t4-current-kernels.json` |
| `src/physics/attached.rs` | `9073a967…1de9197` | 不变（归一化口径与旧值一致） | 三个 precision 夹具 |
| `src/physics/tendon_wrap.rs` | `6e796199…b02f5b4e2` | 不变（同上） | spatial-tendon map |
| 重力包整包 | `c1b5bc52…7afe901be` | `fec9a9fa…24b625c5` | `tests/t4_gravity_nonzero_product_chain.rs` 的 `PACKAGE_SHA256` |

`src/physics/resident.rs` 的新摘要口径为模块根加 16 个产品子模块，见
`docs/development-evidence/t4-resident-anchors/README.md`。

## 锚点修订

重钉同时补齐拆分布局造成的锚点差异，均为锚点组合、读取路径或哈希数值：

- 三个 precision 测试（camlight、spatial-tendon、tendon-wake）对拆分路径读取组合文本。
- `tests/support/t4_rigid_capture.rs` 的路由锚点随拆分从 `super::` 改为 `crate::physics::`。
- `tests/support/t4_rigid_internal.rs` 的历史整源码摘要按组合文本重钉。
- `tests/support/t4_state_packing.rs` 允许设备字段带 `pub(super) ` 前缀，仍逐字核对字段名与类型。
- `tests/support/t4_packing_routes.rs` 的 resident 读取优先使用组合锚点。
- `tests/t4_gravity_nonzero_product_chain.rs` 对拆分路径使用组合文本，文件行数净不增。
- `src/physics/resident/pack_batches.rs` 恢复两行拆分丢失的纯注释。

## 验证

| 编号 | 内容 | 结果 |
| --- | --- | --- |
| run-01 | 重钉后受影响六个目标（lib、input_precision、camlight、spatial-tendon、tendon-wake、gravity） | 全部 0 失败，见 `runs/run-01-repin-tests.log` |
| check-02 | `cargo test --locked --features cuda-probe --no-fail-fast` 全套 | 无失败目标 |
| check-03 | `cargo fmt --check` | 通过 |
| check-04 | `normify gate` fmt、clippy-cuda、unit | 见门禁记录 |
| check-05 | `normify gate` g01-full | 见门禁记录 |

## 未验收项

- 未宣称 NP01 至 NP04、精度与 GPU 性能。
- 未跑 Linux 与 release。
- G01 结果以门禁记录为准。
