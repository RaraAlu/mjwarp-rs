# R09-S1 独立复核（QA）：光照与阴影静态参数（宿主侧）

- 任务：`2026-10-28-render-bvh-r09-s1-check`（复核对象 `2026-10-28-render-bvh-r09-s1-lighting-params`）
- 提交：`ef557f6e`（代码 + 证据，15 文件）→ `0a72c653`（verified 收口）；复核时 HEAD=`0a72c653`
- 结论：**pass / 阻断 0 / 观察 3（3 建议）**；只读复核，未重渲染、未开 GPU、未改代码/判据/参考/容差

## 1. 总体判定

per-light 8 组数组与 `nlight`/`3×nlight` 的长度与类型校验矩阵正确、错误可诊断且不 panic；
`use_shadows=true` 且无 castshadow 灯合法、`shadow_light_fraction` 原值透传（含越界值）、无灯合法、
skybox 与光照解耦——四条「不新增上游未拒绝输入」全部成立；灯数组保持**单份**、不随 `nworld` 复制，
并在 README/receipt 引用 backlog `2026-10-28-render-per-world-geom-light-gap`；红线文件零改动、行数符合；
四门禁我的独立复跑 = **1040 / 0 / 384**、定向 **9 passed**。三条观察均为记录/结构层面，不阻断。

## 2. per-light 校验矩阵（我逐条核）

- `light_type`：`Index`，长度 `nlight`（基准）；`light_castshadow`：`Flag`（u8→bool `!=0`），长度 `nlight`；
  `light_cutoff`/`light_exponent`：`Float`，长度 `nlight`；
  `light_attenuation`/`light_ambient`/`light_diffuse`/`light_specular`：`Float`，长度 **3×nlight**
  （`checked_mul(3)` 防溢出后按 `[f32;3]` 分组）。
- 类型错配 → `ContextError::FieldKind{field}`（8 个字段各有用例）；长度错配 → `LengthMismatch{field,expected,actual}`
  （少 1/多 1、3×nlight 少 1、逐灯标量字段）；缺字段 → `MissingField{field:"light_type"}`。全部不 panic。
- `LightParams.len()/is_empty()` 与 `nlight`/`light_type` 一致；`shadow_light_count()` = `use_shadows ? castshadow 灯数 : 0`。

## 3. 静态参数与「不新增拒绝」

- `use_shadows`（上游 L850）默认 false；`lightcastshadow`（L866）单灯是否投影；`shadow_light_fraction`（L880，
  签名默认 0.3）在做校验时**原值透传**：我核 `0.0/1.0/0.3/2.5/-1.0` 全部接受且等值回传（上游不做范围校验，
  本片不新增拒绝）；`enable_per_light_ambient`（`MK_FLAG_PER_LIGHT_AMBIENT`，L281）与 `use_ambient_lighting`
  默认 true。
- `use_shadows=true` 但所有 `light_castshadow=0`：合法（上游 L919 `if use_shadows and lightcastshadow` 才发阴影射线，
  命中时 L961 `visible = shadow_light_fraction`），产品返回 `any_cast_shadow=false`、`shadow_light_count()=0`。
- 无灯（`nlight=0`）：全部灯字段为空即合法；`any_cast_shadow=false`、`shadow_light_count()=0`、`lights.len()=0`。
- skybox 解耦：`render_skybox` 开/关对 `resolve_lighting` 结果完全相等（代码路径不读该开关）。

## 4. 灯数组单份（不随 nworld 复制）

- `LightParams` 无 world 维；`resolve_lighting` 不读 `params.nworld`；测试以 `nworld=3` 断言 `lights` 与
  `nworld=1` 完全相等。文档明确「按单份上传（内核按单 world 索引灯）」，per-world 灯数据缺口引用 backlog
  `2026-10-28-render-per-world-geom-light-gap`（README 与 receipt 均有）。
- 这不改变 R10 结论：per-world 相机位姿已接（R10-S2），灯仍是单份。

## 5. 接线（特有点）与模块登记

- `src/render/context/params.rs` **+6**：`#[path = "lighting.rs"] pub mod lighting;` + `shadow_light_fraction` 文档；
  `src/render/mod.rs` **+2**：doc + `pub use context::params::lighting::{LightParams, LightingParams, resolve_lighting};`；
  `src/render/context.rs` **未动**（426 行）。
- 观察 O1：`src/render/context/lighting.rs` **未登记**为 `mjwarp.render` 模块源（`.normify/modules/mjwarp/render.md`
  在收口提交里只有 fingerprint 变化，sources=32 不含该文件）；`normify validate --fingerprints` 仍报 0 错误
  （指纹只覆盖已登记源），属治理漂移，与 `camera_output.rs`/`world/*` 先例不一致。

## 6. 行数与红线

| 项 | 值 |
| --- | --- |
| 新增/改动 | `lighting.rs` **184**、`params.rs` **189**（+6）、`render/mod.rs` **380**（+2）、`tests/render_lighting_params.rs` **319** |
| 红线 | `camera_output.rs` 499、`model/render.rs` 500、`kernel/launch.rs` 500、`kernel/megakernel.rs` 491、`kernel/cast.rs` 494、`DEVICE_SOURCES`=28、入口=6、`CollisionArgs` 504 B |
| 提交 diff | `ef557f6e^..0a72c653` 对 `kernel/*`、`kernel/cuda/*.cu`、`fixtures/*`、`Cargo*`、`camera_output.rs`、`model/render.rs` **为空** |

新文件均 ≤500 行；`params.rs`/`mod.rs` 的 +6/+2 只是声明与再导出，**未把渲染逻辑塞进 params/mod**。

## 7. 门禁独立复跑（复核时 HEAD `0a72c653`）

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0 |
| `cargo test --locked` | exit 0：**1040 passed / 0 failed / 384 ignored**（128 result 行） |
| `cargo test --locked --test render_lighting_params` | **9 passed / 0 failed** |

与后端归档 `gate.log` 的 1040/0 一致（ignored 384 一并记录）。门禁日志见 `logs/`（08:28 采集，树=HEAD `0a72c653`）。

## 8. 观察（3 条，均建议级，给修/接受建议）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议 | `lighting.rs` 未登记为 `mjwarp.render` 模块源（sources=32 不含它；`render.md` 只变 fingerprint），与 `camera_output.rs`/`world/*` 先例不一致，`normify validate --fingerprints` 因只覆盖已登记源而仍 0 错误 | **修**：`normify module source mjwarp.render --add src/render/context/lighting.rs`（或在小片里补登记）；不影响代码 |
| O2 | 建议 | `#[path = "lighting.rs"] pub mod lighting;` 放在 `params.rs`，模块路径为 `context::params::lighting`，与文件位置 `context/lighting.rs` 不一致（同 S1 O5 类结构债） | **接受**本轮（scope 只含 `context/` 与 `render/mod.rs`）；后续 scope 放宽时把声明移回 `context.rs` 或把文件移入 `params/` |
| O3 | 建议 | `lighting.rs` L99–100 文档句「`Float` 或 `Index`/`Num` 之外的 `Float`」表述自相矛盾（笔误） | **修**（下次触碰该文件时改措辞）；纯文档，不影响行为 |

## 9. 方法与边界

- 方法：读 pinned `render.py`（`71da24d`）L850/L866/L880/L919-961 语义对照；逐条核 9 个单测并独立重跑；
  代码层核「不读 nworld」「`checked_mul(3)`」「Flag 按 `!=0`」；红线用 `git diff` 逐路径核；门禁独立重跑并重数。
- 边界：只读复核，未重渲染、未开 GPU、未开窗口、未改代码/判据/参考/容差；不做 GPU 端到端（R09-S2 另片）宣称。
