# R05-S1 加固：hfield 模块源登记 + 视图不变量 + 死变体清理

任务：`2026-10-28-render-bvh-r05-s1-hfield-hardening`（P8 渲染与 BVH；p57；owner Backend）
判据树 HEAD：`1d3758ee8335e116e7fffbad243bf2f428f8decd`
来源：QA 复核 `2026-10-28-render-bvh-r05-s1-hfield-params-check`（verdict=pass / 阻断 0 / 观察 6）
处置范围：**O1（一般，必须修）/ O3（必须修）/ O2（收敛死面）/ O4（注释口径）**；O5（`#[path]` 结构债，与 `lighting.rs` 同构）、O6（裁决 A 留痕充分）接受不动。
本片纯宿主，无 GPU、无窗口、无渲染数值结论。

## 1. 观察处置

### O1 模块源未登记 → 已补登记

- 命令：`normify module source mjwarp.render --add src/render/context/hfield.rs`（照 `context/lighting.rs` 先例）
- 结果：`[[source]]` 计数 **34 → 35**（`mjwarp.render：新增源码1 删除0，共35`）；`hfield.rs` 登记于 `.normify/modules/mjwarp/render.md` L122
- `normify validate --fingerprints`：**错误 0**（登记后出现 1 条 `evidence/fingerprint-drift`（mjwarp.render 指纹过期），由本任务 `normify done` 刷新指纹消除；收口后复核错误仍为 0）

### O3 视图不变量可被外部破坏 → 已加固

- `HfieldView` / `HfieldParams` **字段全部私有**，只暴露只读 getter：
  - `HfieldView::{size, nrow, ncol, adr, span}()`；
  - `HfieldParams::{len, is_empty, data, hfields, data_of}()`；
  - 外部在安全 Rust 里构造 `HfieldView { .. }` / `HfieldParams { .. }` 会因私有字段编译失败（E0451），**不变量不可从外部破坏**；
- `HfieldParams::data_of` 改为 `view.adr.checked_add(view.span)?` 再 `data.get(start..end)`：越界/溢出返回 `None`，**不再有 debug panic**（QA 探针 `adr=usize::MAX, span=2` 场景）；
- `is_empty` 文档改为：「恒为 `false`——`resolve_hfield` 对空输入（`nhfield=0`）返回 `Ok(None)`，不构造本结构；保留方法仅为与 `len` 配对」；
- 新增回归测试：`view_invariants_hold_for_every_parsed_hfield`（逐视图断言 `nrow/ncol>0`、`span=nrow×ncol`、`adr+span<=data.len()`、越界 `data_of` 返回 None）、`data_of_out_of_range_returns_none_not_panic`（`data_of(len)` 与 `data_of(usize::MAX)` 均 None）。

### O2 `HfieldError::FieldKind` 无构造点 → 已删除

- 删除 `HfieldError::FieldKind` 变体及其 `Display` 分支（实测构造 0 处，runtime 不可达）；
- 枚举文档注明：「原 `FieldKind` 变体已删除（死面）；真实渲染通道接入（`2026-10-28-render-bvh-r05-hfield-channel`）若出现类型错配路径，按实际构造点重新引入」。

### O4 「单份」测试强度 → 注释写明口径（断言强度不变）

- `resolution_is_single_copy_and_stable` 注释写明：宿主只能做结构性断言（`HfieldInputs` 不含 world 维度，无法观察 per-world 复制）；(a) 断言视图数恒等于 `nhfield`、`Σspan` 不随 world 放大；(b) **用指针相等**（`std::ptr::eq(first.data().as_ptr(), data.as_ptr())`）佐证解析结果直接借用输入缓冲、未复制。

### O5 / O6（接受不动）

- O5：`#[path = "hfield.rs"]` 挂模块与既有 `lighting.rs` 同构（R09-S1 O2 已记录的结构债），本片沿用、无新增债务类型；
- O6：裁决 A 留痕（原卡签名差异、不可执行理由、语义等价清单、后续卡归属）已在 S1 证据 README §2 与 receipt `ruling_a`，本片不重复。

## 2. 文件与行数（LF 口径）

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `src/render/context/hfield.rs` | **372** | O3 加固：私有字段 + getter + `checked_add` + doc 修正；O2 删死变体 |
| `tests/render_hfield_params.rs` | **322** | 12 条单测（原 10 条 getter 化 + 新增 2 条 O3 回归） |
| `.normify/modules/mjwarp/render.md` | — | O1 模块源登记（34→35）+ 指纹刷新（done 时） |

`params.rs`（193）/ `render/mod.rs`（384）本片未改动。全部文件 ≤500 行。

## 3. 测试（12/12 通过）

`cargo test --locked --test render_hfield_params` → **12 passed / 0 failed**（`logs/test.log`）

| 用例 | 说明 |
| --- | --- |
| `accepts_two_valid_hfields` | 合法两 hfield（getter 化） |
| `no_hfield_is_legal_and_returns_none` | 全 `None` → `None` |
| `partial_fields_report_first_missing_field` | 部分缺失点名 |
| `size_shape_mismatch_is_rejected` | `size.len()!=4×nhfield` |
| `row_col_adr_lengths_must_match` | 长度不一致 |
| `adr_out_of_range_is_reported_with_index` | adr 越界/负 adr（含 index） |
| `non_positive_rows_or_cols_are_rejected` | 非正行/列 |
| `extreme_span_does_not_panic_and_reports_range` | 极端乘积不 panic |
| `empty_field_set_is_legal_and_returns_none` | 空切片 → `None` |
| `view_invariants_hold_for_every_parsed_hfield` | **新增**：逐视图不变量 + 越界 None |
| `data_of_out_of_range_returns_none_not_panic` | **新增**：`data_of(len)`/`data_of(usize::MAX)` → None |
| `resolution_is_single_copy_and_stable` | 单份（结构性 + 指针相等，O4 口径注释） |

## 4. 红线与不变量（零改动）

`render.cu` **499** / `launch.rs` **500** / `cast.rs` **494** / `model/render.rs` **500** / `camera_output.rs` **499** / `megakernel.rs` **490**：均未改动（`logs/lines.log`）；`DEVICE_SOURCES=28`、入口 6、`CollisionArgs` 504 B、81 槽 ABI、`fixtures/precision` 零改动。

## 5. 验证

- `cargo test --locked --test render_hfield_params` → 12 passed / 0 failed（`logs/test.log`）
- `cargo clippy --all-targets -- -D warnings` → 通过（`logs/clippy.log`）；`cargo fmt --check` → 通过（`logs/fmt.log`）
- `normify module source` 登记与 `normify validate --fingerprints` 错误 0：见 §1 O1；四门禁绑最终树由 `normify gate` 记录（`.normify/runs/2026-10-28-render-bvh-r05-s1-hfield-hardening/`）

## 6. 未覆盖项（不涉及）

- 渲染侧 hfield 数据通道：`2026-10-28-render-bvh-r05-hfield-channel`（proposed，S2 前置，另行报拆分行数方案给 Arch/Chief）；
- S2 数据源适配与设备侧 hfield BVH/内核相交：另开卡；
- 本片不产生渲染数值结论，不做窗口判定。
