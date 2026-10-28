# R09-S2 内核光照与阴影实现（方案 A：复用 first-hit `cast_ray`；无窗口）

任务：`2026-10-28-render-bvh-r09-s2-lighting-kernel`（P8 渲染与 BVH；仅实现 + 单测，无数值结论）
判据树 HEAD：`5cc6a3d556b252fed8af6496b8a85a1c5c49cacc`
上游锚点：冻结 MJWarp 3.15 `render.py`（`71da24d`）L830 `_make_compute_lighting`、L850 `use_shadows`、
L866 `lightcastshadow`、L880 `shadow_light_fraction`、L919-961 阴影分支、L1457-1459 headlight 不投影。

## 1. 实现路径（证据口径，照抄）
> 实现路径：复用 first-hit `cast_ray`（非 any-hit）；遮挡判定目标与上游 `max_t` 内任意命中一致，但实现路径不同；本片不作任何数值结论，数值验收在 S3b 窗

- any-hit **方案 A**：未新增 `scene_any_hit`/`mesh_any_hit`；零新增 API、零 first-hit 行为变化。
- 判定要点（`render_shadow.cu::shadow_visibility`）：
  - 仅 `use_shadows && cast_shadow` 才发射阴影射线（上游 L919）；
  - `shadow_origin = hitpoint + normal * 1.0e-4f`（上游 L922 自相交规避）；
  - 非方向光 `max_t = dist_to_light - 1.0e-3f`；方向光 `max_t = 1.0e8f`（上游 L924-926）；
  - 命中（`geom_id != -1`）→ `visible = shadow_light_fraction`，否则 1.0；
  - headlight 传 `cast_shadow = false`（上游 L1457-1459）。

## 2. 槽位复用（S7a 81 槽签名零改动）
- `shadow_light_fraction` 放 `scale_constants[0][2]`（内核 `splat_scale.data[0][2]`），**原占位 unused 改作 shadow_light_fraction**；
- 开写前核查留痕：`target/r09-s2-prep/scale-constants-check-2.txt`（kernel 仅读 `[0][1]`=zfar；`[0][0]`/`[0][2]` 无读取点）；
- `use_shadows` 用现有 scalar 槽，host 从硬编码 `false` 改为 `resolved.use_shadows`；**但 host 仍在 `megakernel.rs` 前置拒绝 `use_shadows=true`（S7b3 既有策略），本片不解锁，见 §6**；
- `MEGAKERNEL_SLOTS.len() == 81` 不变（测试断言）。

## 3. 改动文件与行数（LF 实数；全部 ≤500）
| 文件 | 改动前 | 改动后 | 说明 |
| --- | --- | --- | --- |
| `cuda/render_shadow.cu` | — | **30** | 新设备源：`shadow_visibility`（方案 A 封装） |
| `cuda/render_shade.cu` | 346 | **359** | `compute_lighting` 加阴影参数 + `weight = attenuation * shadow_visibility(...)` |
| `cuda/render_megakernel.cu` | 332 | **340** | 移除 `use_shadows` 直接 return；读 `splat_scale.data[0][2]`；逐灯传 `use_shadows`/`light_castshadow`，headlight 传 `false` |
| `kernel/megakernel.rs` | 491 | **492** | `use_shadows` 槽接 `resolved.use_shadows` |
| `kernel/megakernel_arrays.rs` | 235 | **241** | `scale_constants[2]` = `resolved.shadow_light_fraction` |
| `kernel/mod.rs` | 472 | **474** | `MEGAKERNEL_SOURCE` 拼接 `render_shadow.cu`（common→shadow→shade→megakernel） |
| `tests/render_lighting_kernel.rs` | — | **144** | 5 条宿主结构断言 |

**未动**：`cuda/render.cu`（499 贴线）、`kernel/launch.rs`（500 满线）、`kernel/cast.rs`（494）、`model/render.rs`(500)、`camera_output.rs`(499)、collision `DEVICE_SOURCES`=28、`CollisionArgs` 504 B、判据/参考/夹具/容差。

## 4. 验证
- **Rust 编译**：`cargo test --locked --features cuda-probe --lib --no-run` exit 0。
- **内核 NVRTC 编译 + f32-render 门禁**（`megakernel_source_compiles_under_render_gate`，仅编译不发射）：
  - `source_sha256=d38d50677959e65693f3e2cf84c25553dfa9ef19ebf094574281f8833de9d6d6`
  - `ptx_sha256=1e37c50ffc55a5eeeef0520a432d3f56e89a99985f8e204b5a24120f8e311b54`
  - `ptx_bytes=259107`；`check_render_ptx` 通过；执行前后 GPU 显存 5350/10712 MiB 无变化（纯编译）。
- **CPU 单测**（`cargo test --locked --features cuda-probe --test render_lighting_kernel`）：**5 passed / 0 failed**
  1. `megakernel_source_includes_shadow_visibility_source`
  2. `shadow_source_precedes_shading_in_concat_order`
  3. `resolved_context_carries_shadow_parameters`
  4. `megakernel_slots_signature_is_unchanged`
  5. `shadow_light_fraction_default_matches_upstream_signature`
- **四门禁**（`normify gate` 绑最终树）：fmt 1613 ms / clippy 405 ms / clippy-cuda 404 ms / unit 70193 ms（**1045 通过 / 0 失败**）
- `normify module source mjwarp.render --add src/render/kernel/cuda/render_shadow.cu`：render sources 33 → **34**

## 5. 限制与后续（必须随证据保留）
- **本片不写任何渲染数值结论**：`use_shadows=false` 逐位一致与 `use_shadows=true` 的数值/像素验收全部留 **S3b 窗**；
- any-hit 为后续性能优化候选（本轮按方案 A 不做）；
- 灯数组单份（上游 2D `[world, nlight]`）：per-world 灯数据缺口见 backlog `2026-10-28-render-per-world-geom-light-gap`；
- 不写「窗口通过/已覆盖/精度达标/上游等价」。

## 6. 复核观察与后续片（QA `2026-10-28-render-bvh-r09-s2-check`，固定口径）

QA 复核 verdict **pass / 阻断 0 / 观察 8**（`023eafdd` + `66c0fa43`）。与本片证据相关的四条：

1. **O1（严重、不阻断本片）产品路径暂不可达 —— 必须随证据保留**：
   - `src/render/kernel/megakernel.rs:97-99` 仍**前置拒绝** `use_shadows=true`（S7b3 引入的既有产品策略，**本片未改**），第 40 槽恒 `false`；
   - 两条既有 `#[ignore]` GPU 用例仍把「拒绝」当契约：`tests/render_kernel_flags.rs` L412-435、`tests/render_kernel_rgb.rs` L209-224；
   - 因此本片交付口径是 **「内核分支实现就位、产品路径待解锁」**，**不是**「阴影已可用/已解锁/已覆盖」；
   - 解锁动作（移除拒绝 + 同步改上述两条断言 + 头注释更正 + 接线 in-crate 单测）登记为后续片 `2026-10-28-render-bvh-r09-s2-host-guard-unlock`，**必须在 R09-S3b 验证窗之前完成**，否则窗口无法从产品入口发射阴影。
2. **O2（已更正）**：`tests/render_lighting_kernel.rs` 实测 **144** 行（原文误写 145，README §3 与 receipt 已改）。
3. **O4（接受延后）**：`cuda/render_megakernel.cu:15` 头注释仍写 `splat_scale = [znear, zfar, unused]`；本片不改 `.cu`（改注释会变 `MEGAKERNEL_SOURCE`/PTX sha，与本节 §4 已冻数值冲突），随 `…-r09-s2-host-guard-unlock` 片更正。
4. **O7（接受）**：本片**无拆分/移动对象**——卡面 acceptance#1「拆分核验」不适用；贴线文件（`render.cu` 499 / `launch.rs` 500 / `cast.rs` 494）一行未动。

另两条口径说明（QA O5/O8）：
- **manifest 基准**：本目录 `manifest.txt` 的源码行按 **HEAD committed blob（LF）** 记 sha256/bytes，不按工作树字节（`core.autocrlf=true` 下工作树是 CRLF，两者不同）；
- **receipt `head`**：`5cc6a3d5` 是**证据时点的 HEAD**（两段提交前），不是最终树绑定；最终树由 `1ad2f268` 与门禁记录确定。
- QA O6（`tests-and-lint.log` 保留 fmt 修前中间态）接受：最终树 fmt 通过（QA 已复跑）。
