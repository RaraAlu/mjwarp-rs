# T4源码身份重钉（第二轮）

任务：`2026-10-22-t4-source-hash-repin-2`。
前置：`2026-10-22-t4-source-hash-repin`、`2026-10-21-t4-resident-source-anchors`、拆分程序 61 片。
先例：`2026-10-22-t4-source-hash-repin`、`2026-10-19-t4-input-precision-source-drift`。

## 结论

拆分程序改动了被 T4 证据钉住的产品源码后，旧身份哈希与锚点组合过期。
本轮只重钉源码身份 sha256、锚点组合与读取路径，并移除切片引入的 UTF-8 BOM。
映射条目、断言、容差、fixtures 其余字段与产品逻辑未改动。
重钉后 `cargo test --locked --features cuda-probe --no-fail-fast` 全套无失败目标。

## 重钉口径

沿用 repin-1 口径：拆分模块取「模块根加产品子模块」组合，模块与文件之间以单个换行连接，摘要按 CRLF→LF 归一；未拆分文件取当前全文同口径摘要。
组合顺序按模块根 `mod` 声明顺序，不含 `#[cfg(test)]` 的 tests 子模块。

## 重钉表

| 源码路径 | 旧摘要 | 新摘要 | 口径 | 涉及证据 |
| --- | --- | --- | --- | --- |
| `src/model/camlight.rs` | `e9299df1…` | `81ac86b6…` | 当前全文 | g01-inputs、g01-field-internal、camlight map |
| `src/model/flex.rs` | `d8df34bc…` | `8355b0f1…` | 当前全文（input_precision） | g01-inputs、g01-packing-routes |
| `src/model/flex.rs` | `d8df34bc…` | `b3603225…` | 模块根加 edge/face/input/position | flex-internal、flex-matrices map |
| `src/runtime/transfer.rs` | `de678af8…` | `d8df351e…` | 模块根加 buffer/session | g01-rigid-internal、g01-state-packing、flex-internal、flex-matrices map |
| `src/physics/gravity_compensation.rs` | `245cf69b…` | `1e572036…` | 当前全文（重力包按文件读取） | 重力包与 `t4_gravity_nonzero_product_chain` 内联常量 |
| 重力包整包 | `fec9a9fa…` | `35c6fd84…` | 当前全文 | `PACKAGE_SHA256` |

`src/physics/mod.rs`（`09da2b79…`）与 `src/physics/resident.rs`（`919335fb…`）组合哈希未变，沿用 repin-1。
`src/physics/flex_face.rs` 在 packing-routes 的 hist 校验仍按 `face_lineage::historical` 计算，保持 `47a17fc2…`。

## 锚点组合修订

- `tests/support/t4_rigid_internal.rs`、`t4_state_packing.rs`、`t4_packing_routes/flex_internal.rs`、`t4_field_internal/flex_matrices.rs` 的 TRANSFER 锚点改为模块根加 buffer/session 组合。
- `t4_packing_routes/flex_internal.rs`、`t4_field_internal/flex_matrices.rs` 的 model/flex 锚点改为模块根加 edge/face/input/position 组合。
- `tests/support/t4_rigid_capture.rs` 的 COM_VELOCITY 锚点改为模块根加产品子模块组合，保留路由与模板解析语义。

## BOM 清理

拆分切片中以下文件由脚本写入时带 UTF-8 BOM，本轮只删除首三字节，无其他内容改动：
`src/runtime/cuda/resources.rs`、`src/runtime/cuda/resources/{affine,fence,foreign}.rs`、`src/runtime/transfer/session.rs`、`tests/t4_input_precision.rs`、`tests/t4_input_precision/{external,fields}.rs`。

## 验证

| 编号 | 内容 | 结果 |
| --- | --- | --- |
| run-01 | lib recapture 两项 | 通过 |
| run-02 | t4_input_precision | 36 过 0 败 12 ignored |
| run-03 | t4_camlight_precision | 3 过 0 败 |
| run-04 | t4_gravity_nonzero_product_chain | 1 过 0 败 1 ignored |
| run-05 | `cargo test --locked --features cuda-probe --no-fail-fast` 全套 | 无失败目标 |
| check-06 | `normify gate` fmt、clippy-cuda、unit | 见门禁记录 |

## 未验收项

- 未宣称 NP01 至 NP04、精度与 GPU 性能。
- 未跑 Linux 与 release。
- G01 全量门禁与脚本过期问题由收口与既有债务清单承接。