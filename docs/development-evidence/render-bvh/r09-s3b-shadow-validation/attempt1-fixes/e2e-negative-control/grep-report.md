# R09-S2b-e2e-negative-control 全仓旧断言清查报告

## 1. 结论摘要
- 旧契约「`use_shadows=true` 必须写前拒绝」全仓共 **3 处**：`tests/render_kernel_flags.rs`（S2b 已改）、`tests/render_kernel_rgb.rs`（S2b 已改）、`tests/render_e2e.rs:475`（本卡修复）。
- 修复后复查：**无第四处**。其余 `use_shadows` 命中均为「接受断言 / ResolvedContext 接线 / 参数解析 / 参考读取 / 诊断打印」，不是拒绝契约。
- 产品源码（`src/`）无任何 `use_shadows=true` 拒绝逻辑（S2b 已删除 `megakernel.rs` 前置拒绝，L97 处仅剩解锁说明注释）。

## 2. 全仓 `use_shadows` 命中（src/ + tests/ + scripts/，共 94 处）

| 文件 | 命中数 | 类别与结论 |
| --- | --- | --- |
| `scripts/authoring/render-bvh/capture_reference.py` | 2 | authoring 捕获脚本参数 |
| `scripts/authoring/render-bvh/capture_s2_bvh.py` | 1 | authoring 捕获脚本参数 |
| `scripts/authoring/render-bvh/capture_s2a_bounds.py` | 1 | authoring 捕获脚本参数 |
| `scripts/authoring/render-bvh/capture_s2b_query.py` | 1 | authoring 捕获脚本参数 |
| `scripts/authoring/render-bvh/capture_s3_mesh.py` | 1 | authoring 捕获脚本参数 |
| `scripts/authoring/render-bvh-s5/capture_s5_camera_rays.py` | 1 | authoring 捕获脚本参数 |
| `scripts/authoring/render-bvh-s7b/capture_fast_math_false.py` | 1 | authoring 捕获脚本参数 |
| `scripts/authoring/render-bvh-s7b/capture_flags_aa.py` | 1 | authoring 捕获脚本参数 |
| `scripts/authoring/render-bvh-s7b/capture_rgb_fast_math_false.py` | 1 | authoring 捕获脚本参数 |
| `scripts/authoring/render-bvh-s7b/capture_synthetic_normals.py` | 1 | authoring 捕获脚本参数 |
| `scripts/capture_render_shadow.py` | 8 | authoring 捕获脚本参数 |
| `src/render/context.rs` | 2 | ResolvedContext 接线 / 参数解析 / 解锁注释 |
| `src/render/context/lighting.rs` | 10 | ResolvedContext 接线 / 参数解析 / 解锁注释 |
| `src/render/context/params.rs` | 4 | ResolvedContext 接线 / 参数解析 / 解锁注释 |
| `src/render/kernel/cuda/render_megakernel.cu` | 4 | ResolvedContext 接线 / 参数解析 / 解锁注释 |
| `src/render/kernel/cuda/render_shade.cu` | 5 | 其它 |
| `src/render/kernel/cuda/render_shadow.cu` | 3 | 其它 |
| `src/render/kernel/megakernel.rs` | 5 | ResolvedContext 接线 / 参数解析 / 解锁注释 |
| `src/render/kernel/megakernel_arrays.rs` | 7 | ResolvedContext 接线 / 参数解析 / 解锁注释 |
| `tests/render_context_params.rs` | 1 | ResolvedContext 接线 / 参数解析 / 解锁注释 |
| `tests/render_e2e.rs` | 4 | **旧拒绝契约（本卡修复为接受断言）** |
| `tests/render_kernel_flags.rs` | 4 | S2b 已改为接受断言 |
| `tests/render_kernel_rgb.rs` | 5 | S2b 已改为接受断言 |
| `tests/render_lighting_kernel.rs` | 9 | ResolvedContext 接线 / 参数解析 / 解锁注释 |
| `tests/render_lighting_params.rs` | 5 | ResolvedContext 接线 / 参数解析 / 解锁注释 |
| `tests/render_shadow_reference.rs` | 2 | S3b 阴影测试（接受语义） |
| `tests/support/render_bvh/shadow_scene.rs` | 5 | S3b 阴影测试（接受语义） |

### 2.1 关键行逐条

**`src/render/kernel/megakernel.rs`**
- 38: shading_words, span_addr, to_i32, upload, use_shadows_arg,
- 77: /// 发射一帧（默认 context 口径：spp=1、预计算射线、无天空盒/splat/AA；阴影由 `use_shadows` 控制）。
- 97: // R09-S2b：`use_shadows` 产品路径已解锁（S7b3 前置拒绝已移除；数值验收在 S3b 窗）。
- 404: // R09-S2b：第 40 槽（0-based `MEGAKERNEL_SLOTS[40]`）阴影总开关，值取 `resolved.use_shadows`。
- 405: args.push(use_shadows_arg(resolved.use_shadows));

**`tests/render_e2e.rs`**
- 434: /// `use_shadows=true` 已由 R09-S2b 解锁 → 改为独立「接受 → Ok(())」断言（不在拒绝面）。
- 444: // R09-S2b：`use_shadows=true` 已解锁（前置拒绝已移除）→ 独立「接受」断言（与 flags/rgb 用例同口径）。
- 445: let shadow_params = ContextParams { use_shadows: true, ..default_params() };
- 448: assert!(shadow_result.is_ok(), "use_shadows=true 必须被接受（实际 {shadow_result:?}）");

**`tests/render_kernel_flags.rs`**
- 412: // 3) R09-S2b：`use_shadows=true` 已解锁（host 前置拒绝移除）→ 必须被接受并发射成功。
- 414: use_shadows: true,
- 421: Err(_) => panic!("use_shadows=true 的 context 必须可创建（R09-S2b 解锁）"),
- 434: "use_shadows=true 必须被接受（实际 {shadow_result:?}）"

**`tests/render_kernel_rgb.rs`**
- 217: // R09-S2b：`use_shadows=true` 已解锁（host 前置拒绝移除）→ 必须被接受并发射成功。
- 219: use_shadows: true,
- 226: Err(_) => panic!("use_shadows=true 的 context 必须可创建（R09-S2b 解锁）"),
- 232: "use_shadows=true 必须被接受（实际 {shadow_result:?}）"
- 243: // 注：`samples_per_pixel=2`（AA）已由 S7b3 实现、`use_shadows` 由 R09-S2b 解锁，

## 3. 拒绝面关键词命中（`Unsupported` / 「写前拒绝」/ 「不含阴影」/ `rejected` 且含 shadow/阴影）
- `src/render/kernel/megakernel.rs:9` — //!   背面剔除；阴影产品路径 R09-S2b 起已解锁（天空盒/splat/flex/hfield/SDF 仍写前拒绝）。
- `tests/render_e2e.rs:497` — println!("s8[negative] shadows accepted; 2 类写前拒绝 + AA 射线数（nsamples×total_rays）判定成立");
- `tests/render_kernel_flags.rs:437` — // 4) 其余 2 类仍写前拒绝（AA 与阴影均已从拒绝面移除）。
- `tests/render_kernel_flags.rs:473` — "s7b3 rejection surface: AA + shadows supported; precomputed=false/skybox rejected; bad AA ray count rejected"
- `tests/render_kernel_rgb.rs:268` — "s7b2 rejection surface: shadows supported; precomputed=false/skybox rejected (AA moved to S7b3)"

以上命中**全部为注释或 println 说明**（无断言）；旧断言只剩本卡已修的 `tests/render_e2e.rs` negative control。

## 4. 本卡修改
- `tests/render_e2e.rs`：negative control 的 `use_shadows=true` 从 cases 循环（断言 `Unsupported`）移出，改为独立「接受 → `Ok(())`」断言（与 flags/rgb 用例同口径）；
  其余 2 类（`use_precomputed_rays=false`、`render_skybox`）保持断言 `Unsupported` 全强度；`rejected` 计数 3 → 2；文件行数 499 → 498（≤500 棘轮）。
- 未改判据/容差/参考/夹具；未删除或降级任何拒绝面用例。
