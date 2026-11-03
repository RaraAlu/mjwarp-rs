# R06-S3b③：flex 设备侧接线与 GPU 验证用例（离线作者；不开窗）

任务：`2026-11-04-render-r06-s3b-flex-device-harness`（p75，owner Backend；depends S3b① capture r2）。
基线 HEAD：`87b8fbb9`。本片**不开窗、不跑 GPU 用例**：只做接线 + `#[ignore]` 用例作者与编译自证。
`fixtures/` 零改动（`unchanged fixtures/`）；`src/render/kernel/cuda/`、全部 `.cu` 零改动。

## 1. 交付面（均 ≤500 行）

| 文件 | 行数 | 内容 |
| --- | --- | --- |
| `src/render/context/flex_host.rs` | 500 | 原 `FlexHostAssembly`（S3pre）+ `mod render;` 注册（+1 行） |
| `src/render/context/flex_host/render.rs` | 32 | **生产调用点** `FlexHostAssembly::render_with`：把 `inputs()` + 可选 `FlexMeshChannel` 交给 `MegakernelRenderer::render_with_flex` |
| `tests/render_flex_validate.rs` | 500 | `#[ignore]` GPU 用例：读 `scene-flex/reference/base/` + `state.npz`（含 `flexvert_xpos`）复现设备渲染，按 S3b② 实体表三相机判据比对 |

## 2. 生产调用点（缺口 ① 闭）

调用链：`FlexHostAssembly::build(...)`（S3pre，无 flex → `Ok(None)` 走既有 `render`）→
`FlexHostAssembly::render_with(renderer, context, fields, model, poses, rays, mesh)` →
`MegakernelRenderer::render_with_flex(..., Some(&inputs), mesh)`。
- 入参来自 `FlexRenderFields`（`flex_dim/radius/rgba/edgeadr/edgenum`）+ `FlexPositionFields`（`flex_vertadr/vertnum/vertbodyid`）+
  `FlexEdgeFields`（`flex_edge`）+ `FlexHostData`（`flexvert_xpos`/`flex_bvh_ids`/`flex_group_root`）；
  2D/3D 设备段经 `Option<&FlexMeshChannel>` 透传（本 1D 场景传 `None`，走 S3a 胶囊路径）。
- 该方法为库级生产 API（非 `#[cfg(test)]`）；仓内调用方 = GPU 用例（窗口执行），外部生产调用方为 LRsLab/前端。

## 3. `flex=None` / `flex_mesh=None` 等价性论证（缺口 ① 的等价性条款）

- **`flex=None` 路径**：`FlexHostAssembly::build` 在 `nflex == 0` 时返回 `Ok(None)`，调用方按既定约定走
  `MegakernelRenderer::render`；本片未改 `render` 与 `render_with_flex` 的实现（`megakernel.rs` 零 diff）。
- **`flex_mesh=None`**：`render_with_flex` 内 `FlexMeshAssembly::prepare(None, …) -> Ok(None)`，
  `mesh_assembly`/`flex_bvh_ids`/`adjusted_flex` 全 `None` → 上传的 `flex` 参数即 S3a 通道原值；
  设备侧 `.cu` 无 flex 分支被数据门控（无负 `leaf_geom`）。
- 与父版本逐字节等价的充分条件：`render` 与 `render_with_flex(…, None, None)` 都构造
  `RenderPass { world: None, flex: None, flex_mesh: None }` 并调用同一 `render_impl`（字面量逐字段相同，
  无任何额外变换/分支）。窗口卡的 `flex=None` 对照用例（GPU）可把该论证升级为实测。

## 4. GPU 用例（缺口 ② 闭；`#[ignore]`，不在本片执行）

`tests/render_flex_validate.rs`：
- 参考：`rgb.npz`/`depth.npz`/`seg.npz`（attempt E，产品同配方）+ `state.npz`（`geom_xpos/geom_xmat/light_*/qpos/flexvert_xpos`）+ `capture.json`；
  host 自证 `two_captures_bitwise_identical=true`、`legacy_reference_untouched=true`；`flexvert_xpos` 形状 `[1,6,3]`。
- 产品侧：`scene-flex/model-312.json` → `RenderFields`/场景叶（世界位姿取 `state.npz`）/网格/纹理 → `RenderContext`
  → `CameraModel`（3 相机世界位姿取 `cam_pos0`/`cam_mat0`）→ `precomputed_rays` → `MegakernelRenderer`
  → `FlexHostAssembly::build` → **生产调用点 `render_with(..., None)`**。
- 判据（零放宽）：rgb bit-first `beyond_1_lsb_pixels=0` 且 `max_byte_diff<=1`；depth 位级 `mismatched=0` 且
  `max_ulp<=4`、`beyond=0`；seg 逐像素全等（cam-flex 参考 654 个 `(flexid,9)`，flexid 0/1 都覆盖）；两跑 sha256 相同。
- 编译自证：`cargo test --locked --features cuda-probe --no-run --test render_flex_validate` → exit 0（`compile.log`）。

## 5. 执行计划（窗口卡 `2026-11-04-render-r06-s3b-flex-validate`）

```
cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --test-threads=1
```

窗口内由 Arch 的冻结器/守卫链执行；本片不宣称 flex 数值可用、不做参考比对、不进验收链。

## 6. 验证

- 四门禁：fmt / clippy / clippy-cuda / unit 全过；unit **1314 passed / 0 failed**（含支持模块用例；只增不减）。
- 零 GPU、零窗口、零参考采集；`fixtures/` 零改动；不改判据/容差/参考。
- `tests/support/render_bvh/outline_tests.rs` 的 depth 白名单扫描零新增裸调用（GPU 用例用本地 ULP 循环，不调 `diff_depth`）。

## 7. 边界与遗留

- 用例未在 GPU 上执行（本片红线）；执行归 S3b④ 窗口卡。
- `render_with` 的 2D/3D `FlexMeshChannel` 分支未在 GPU 上跑（1D 场景传 `None`；机制单测/NVRTC 覆盖在 S3c）。
- 3 相机顺序 = 模型相机顺序（`cam-persp`/`cam-track`/`cam-flex`）；参考中非 cam-flex 相机 flex 像素为 0。
