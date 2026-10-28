# R05-S1 地形 hfield 静态参数校验与宿主解析（纯宿主，无窗口）

任务：`2026-10-28-render-bvh-r05-s1-hfield-params`（P8 渲染与 BVH；p58；owner Backend）
判据树 HEAD：`4723fd4402572d14f03f77bd64ac6331a0867447`
阶段：**实现 + 10 条宿主单测 + 四门禁完成**；本片纯宿主，无 GPU、无窗口、无渲染数值结论。
资源：无需 GPU 窗口（CPU-only）。

## 1. 文件与行数（LF 口径）

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `src/render/context/hfield.rs`（新） | **332** | `HfieldInputs` 只读输入 + `HfieldView`/`HfieldParams` 视图 + `HfieldError` + `resolve_hfield` |
| `src/render/context/params.rs` | **193**（+4） | `#[path = "hfield.rs"] pub mod hfield;` 挂模块 + 文档注 |
| `src/render/mod.rs` | **384**（+2） | `pub use context::params::hfield::{HfieldError, HfieldInputs, HfieldParams, HfieldView, resolve_hfield};` |
| `tests/render_hfield_params.rs`（新） | **284** | 10 条宿主单测 |

全部 ≤500 行；新文件无超限。

## 2. 裁决 A 留痕（与原卡面签名的差异 + 语义等价清单）

**原卡面要求**：`resolve_hfield(&RenderFields)`（`HfieldParams` 读取五个 `hfield_*` 字段）。

**实测冲突（2026-10-28 核查，已报 Arch 并裁决 A）**：
- `src/model/render.rs` 的 `RENDER_FIELDS: [FieldSpec; 116]` **没有任何 hfield 字段**；
- `RenderFields::field(name)` 按 `RENDER_FIELDS` 查名（`index_of`），对 `hfield_*` **恒为 `None`**；
- `RenderFields::new(Vec<FieldData>)` 按 116 项顺序包装，**无法注入额外字段**；
- `src/model/render.rs` 已是 **500 行红线**且不在本卡 files；`src/io/render.rs` 从 native `render_stage` 只读这 116 项（grep 无 hfield）；
- 若按原签名实现，acceptance 中「合法两 hfield / 部分缺失 / 形状错配 / 长度不一致 / adr 越界」等**至少 6 条测试构造不出来**，且 S2 拿不到数据。

**Arch 裁决 A**：签名改为 `resolve_hfield(&HfieldInputs)`，`HfieldInputs` 定义在 `src/render/context/hfield.rs` 内；渲染侧 hfield 数据通道（含 `model/render.rs` 红线拆分与 `RENDER_FIELDS`/`io/render.rs`/native 扩展）由后续卡 `2026-10-28-render-bvh-r05-hfield-channel`（proposed，S2 前置）承担；S2 的数据源适配另开卡。

**语义等价清单（逐条对应）**：

| 卡面要求 | 本片实现 |
| --- | --- |
| 五字段必须同时提供 | `HfieldInputs` 五个 `Option`；`any_present`/`all_present`/`first_missing`；部分提供 → `HfieldError::MissingField { field }`（点名首个缺失字段） |
| 缺任一 → `MissingField` | 同上（`hfield_size`/`hfield_nrow`/`hfield_ncol`/`hfield_adr`/`hfield_data` 依次点名） |
| 类型错配 → `FieldKind` | 强类型 `HfieldInputs`（`&[f32]`/`&[i32]`）在编译期承担；运行期形状错配以 `hfield_size.len() != 4×nhfield` 的 `LengthMismatch` 表达（原卡「类型错配」测试由 `size_shape_mismatch_is_rejected` 承担） |
| `size` 长度 = `nhfield`（×4 分量） | `size.len() == 4 × nhfield`（`checked_mul` 后 `LengthMismatch`） |
| `nrow/ncol/adr` 长度 = `nhfield` | 以 `nrow.len()` 为 `nhfield` 基准，三者相等否则 `LengthMismatch{field,expected,actual}` |
| `data.len() >= Σ(nrow×ncol)` | 逐项 `adr+span <= data.len()` 界值检查 + 显式 `Σspan <= data.len()`（重叠 adr 也覆盖，否则 `LengthMismatch`） |
| `nhfield=0` 合法 → `None` | 全 `None` 或全空切片 → `Ok(None)`（不新增上游未拒绝输入） |
| `nrow<=0 || ncol<=0` 拒绝 | `HfieldError::NonPositiveRowsCols { index, nrow, ncol }` |
| `nrow*ncol` 溢出 | `checked_mul`/`checked_add`；溢出 → `CountOverflow { field }`（64 位 usize 上 i32 乘积不可达，防御性保留） |
| `adr<0` / `adr+span>data.len()` 可读错误 | `HfieldError::DataRange { index, field, start, end, actual }`（含 index/期望/实际） |
| 任何路径不 panic | 无 `unwrap` 于用户数据、无索引越界、`try_reserve_exact` 失败 → `CountOverflow` |
| 单份（不读 nworld、不复制） | `HfieldInputs` 不含 `nworld`；`resolve_hfield` 不接收 world 参数；视图数恒等于 `nhfield` |

## 3. 校验矩阵（`resolve_hfield`）

1. 全 `None` → `Ok(None)`；任一 `Some` 而其它 `None` → `MissingField`；
2. `ncol.len() != nrow.len()` / `adr.len() != nrow.len()` → `LengthMismatch`；
3. `size.len() != 4 × nhfield` → `LengthMismatch`；
4. `nhfield == 0`（全空切片）→ `Ok(None)`；
5. 逐项：`nrow<=0 || ncol<=0` → `NonPositiveRowsCols`；`span = nrow×ncol`（checked）；`adr<0` → `DataRange`；`adr+span > data.len()` → `DataRange`；
6. 总和 `Σspan > data.len()` → `LengthMismatch`（重叠 adr 防护）；
7. 成功 → `HfieldParams { data, hfields: Vec<HfieldView> }`，`data_of(index)` 返回该段高程切片。

## 4. 单测（`tests/render_hfield_params.rs`，10/10 通过）

| 用例 | 断言 |
| --- | --- |
| `accepts_two_valid_hfields` | 5×5 + 3×4 两 hfield：size/nrow/ncol/adr/span 与 `data_of` 分段正确 |
| `no_hfield_is_legal_and_returns_none` | 全 `None` → `Ok(None)` |
| `partial_fields_report_first_missing_field` | 缺 `ncol`/缺 `data` → `MissingField` 点名 |
| `size_shape_mismatch_is_rejected` | `size.len()=4`（需 8）→ `LengthMismatch{field:"hfield_size",expected:8,actual:4}`（原卡「类型错配」替代） |
| `row_col_adr_lengths_must_match` | `ncol` 短、`adr` 长 → 两条 `LengthMismatch` |
| `adr_out_of_range_is_reported_with_index` | `adr=26,span=12,data=37` → `DataRange{index:1,start:26,end:38,actual:37}`；`adr=-3` → `DataRange{index:1,start:-3}` |
| `non_positive_rows_or_cols_are_rejected` | `nrow=0`、`ncol=-2` → `NonPositiveRowsCols` |
| `extreme_span_does_not_panic_and_reports_range` | `i32::MAX×i32::MAX` 与 `adr=i32::MAX`：checked 运算不 panic，报 `DataRange` |
| `empty_field_set_is_legal_and_returns_none` | 全 `Some` 空切片 → `Ok(None)` |
| `resolution_is_single_copy_and_stable` | 两次解析 `HfieldParams` 相等；视图数 = `nhfield`；`Σspan=37`（不随 world 放大） |

## 5. 红线与不变量（零改动）

- `render.cu` **499**、`launch.rs` **500**、`cast.rs` **494**、`model/render.rs` **500**、`camera_output.rs` **499**、`megakernel.rs` **490**：均未改动（行数实测见 `logs/lines.log`）；
- `DEVICE_SOURCES = 28`、入口 = 6、`CollisionArgs = 504 B`、`fixtures/precision`：零改动；本片未触碰任何 `*.cu`；
- 81 槽 ABI 未改：`hfield_bvh_id` 仍为既有占位上传 `[0u32, 0]`（`megakernel.rs` 未改）；`MEGAKERNEL_SLOTS.len()==81` 现有断言保持；
- `render.cu`/`launch.rs`/`cast.rs` 一行未加。

## 6. 未覆盖项（必须随证据保留）

- **渲染侧 hfield 数据通道未接入**：`RENDER_FIELDS`/`io/render.rs`/native `model_probe.cpp` 尚无 hfield；由 `2026-10-28-render-bvh-r05-hfield-channel`（proposed）承担，是 S2 前置；
- **S2 数据源适配未做**：`HfieldInputs` 从既有 collision/volume 输入（`hfield_size/nrow/ncol/adr/data`）构建的适配件另开卡；
- **设备侧 hfield BVH 与内核相交未实现**：S2 负责；`hfield_bvh_id` 槽位语义未变；
- **上游 2D 数值/等价未验证**：本片零渲染数值结论，不做窗口判定。

## 7. 验证与上游锚点

- `cargo test --locked --test render_hfield_params` → **10 passed / 0 failed**（`logs/test.log`）
- `cargo clippy --all-targets -- -D warnings` → 通过（`logs/clippy.log`）；`cargo fmt --check` → 通过（`logs/fmt.log`）
- 四门禁（fmt/clippy/clippy-cuda/unit）绑最终树由 `normify gate` 记录（`.normify/runs/2026-10-28-render-bvh-r05-s1-hfield-params/`）
- 上游锚点（冻结 MJWarp 3.15 `render.py`，`71da24d`）：L332 `_texture_hfield`；L462-464 / L682-685 HFIELD 分支；L623 / L858 `hfield_bvh_id`
- 单份口径引 backlog `2026-10-28-render-per-world-geom-light-gap`（逐 world 几何/灯数据缺口，本片不宣称上游 2D 等价）
