# R09-S1 光照与阴影静态参数（纯宿主）

任务：`2026-10-28-render-bvh-r09-s1-lighting-params`（P8 渲染与 BVH，CPU-only，无 GPU、无窗口）。
本片只做宿主侧校验与静态参数结构，不分配设备资源、不发射内核、不改 kernel/设备 `.cu`。

## 1. 上游锚点（冻结 MJWarp 3.15 `render.py`，`71da24d`）

| 上游位置 | 语义 | 本片落点 |
| --- | --- | --- |
| L830 `_make_compute_lighting` | per-light 光照计算入口 | `LightParams` 逐灯静态参数 |
| L850 `use_shadows` | 阴影总开关 | `LightingParams::use_shadows` |
| L866 `lightcastshadow` | 单灯是否投影 | `LightParams::cast_shadow` / `any_cast_shadow` |
| L880 `shadow_light_fraction` | 遮挡像素可见度系数（签名默认 0.3） | `LightingParams::shadow_light_fraction`（原值透传） |
| L919-961 | `if use_shadows and lightcastshadow` 才发阴影射线；命中 `visible = shadow_light_fraction` | `shadow_light_count()`（=0 表示合法但无阴影） |
| L281 `MK_FLAG_PER_LIGHT_AMBIENT` | 按灯 ambient 档位 | `enable_per_light_ambient` / `use_ambient_lighting` |

## 2. 校验矩阵（`resolve_lighting`，读取前可诊断、不 panic）

| 字段 | 期望类型 | 期望长度 | 错误面 |
| --- | --- | --- | --- |
| `light_type` | `FieldData::Index` | `nlight`（基准） | 缺失/类型错 |
| `light_castshadow` | `FieldData::Flag` | `nlight` | 缺失/类型错/长度错 |
| `light_cutoff` | `Float` | `nlight` | 缺失/类型错/长度错 |
| `light_exponent` | `Float` | `nlight` | 缺失/类型错/长度错 |
| `light_attenuation` | `Float` | `3 × nlight` | 缺失/类型错/长度错 |
| `light_ambient` | `Float` | `3 × nlight` | 缺失/类型错/长度错 |
| `light_diffuse` | `Float` | `3 × nlight` | 缺失/类型错/长度错 |
| `light_specular` | `Float` | `3 × nlight` | 缺失/类型错/长度错 |

错误类型：`ContextError::MissingField` / `FieldKind` / `LengthMismatch`（带字段名与 expected/actual）/ `CountOverflow`（`3 × nlight` 溢出）。

## 3. 不新增上游未拒绝的输入（硬约束）

- `use_shadows=true` 但**没有任何 `light_castshadow` 灯**：合法（`any_cast_shadow=false`、`shadow_light_count()==0`），不产生阴影（上游 L919 分支条件）；
- `shadow_light_fraction`：上游无范围校验，**原值透传**（0.0/1.0/2.5/-1.0 均接受），本片不新增拒绝；
- 无灯（`nlight=0`）：合法，空数组；
- `render_skybox` 与光照**解耦**：只影响 skybox 字段，不改变光照校验结果（回归断言）。

## 4. 灯数组保持单份（R10 结论延续）

`LightParams` 逐灯数组长度为 `nlight` / `3 × nlight`，**不随 `nworld` 复制**；内核按单 world 索引灯（`render.py` L294 起 `light_*[light]`）。逐 world 灯数据缺口见 backlog `2026-10-28-render-per-world-geom-light-gap`（backlog 不 start）。

## 5. 改动文件与行数

| 文件 | 变化 |
| --- | --- |
| `src/render/context/lighting.rs` | 新增 184 行（`LightParams`/`LightingParams`/`resolve_lighting`/`shadow_light_count`） |
| `src/render/context/params.rs` | 189 行（+6：`#[path]` 模块声明 + `shadow_light_fraction` 语义注释） |
| `src/render/mod.rs` | 380 行（+2：导出 `LightParams`/`LightingParams`/`resolve_lighting`） |
| `tests/render_lighting_params.rs` | 新增 319 行（9 条测试） |

未改：`src/model/render.rs`(500)、`camera_output.rs`(499)、kernel 三文件、任何 `.cu`、`DEVICE_SOURCES`(28)、入口(6)、`CollisionArgs`(504 B)。

## 6. 测试（9 条，`cargo test --locked --features cuda-probe --test render_lighting_params`）

1. `resolves_per_light_fields_and_derived_flags`（数组长度/派生标记/阴影计数）
2. `per_light_arrays_stay_single_world_regardless_of_nworld`（nworld=1 vs 3 灯数组一致）
3. `shadow_without_castshadow_lights_is_legal`（use_shadows 无投影灯合法）
4. `shadow_light_fraction_passes_through_without_new_rejection`（0.0/1.0/0.3/2.5/-1.0）
5. `rejects_wrong_field_kinds`（8 字段逐一类型错 → FieldKind）
6. `rejects_length_mismatch_with_diagnostics`（castshadow/attenuation/ambient/cutoff/exponent）
7. `zero_lights_is_legal`
8. `skybox_flag_does_not_change_lighting_validation`
9. `rejects_missing_light_field`

## 7. 门禁（绑最终树，`normify gate`）

- fmt 通过 1558 ms
- clippy 通过 455 ms
- clippy-cuda 通过 403 ms
- unit 通过 69319 ms，**1040 通过 / 0 失败**

本片无 GPU、无窗口、无新捕获；无上游 nworld>1 结论。

## 8. 限制与后续

- 本片只做**参数/结构层**，不含内核光照/阴影实现（R09-S2 待 S1 verified 后 start；实现前需先拆贴线文件，any-hit API 需先报 Arch）；
- skybox 独立列片（不进 R09）；
- 逐 world 灯位姿/数据未支持（backlog `2026-10-28-render-per-world-geom-light-gap`）；
- 不写「窗口通过/已覆盖/精度达标/上游等价」类结论。
