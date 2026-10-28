# R09-S2b：解锁 use_shadows 产品路径（host 前置拒绝移除 + 两条 GPU 断言同步 + 接线单测）

任务：`2026-10-28-render-bvh-r09-s2-host-guard-unlock`（P8 渲染与 BVH；p62；owner Backend）
判据树 HEAD（开写时）：`f41827a1`（R09-S3a 收口后）
性质：**CPU-only 实现片、无 GPU 窗口**；本片只解锁产品路径与同步断言/注释/单测，**不产任何渲染数值结论**（`use_shadows=false` 逐位回归与 `use_shadows=true` 的像素级行为验证在 R09-S3b 窗）
前置：R09-S2（`2026-10-28-render-bvh-r09-s2-lighting-kernel`，verified）已把阴影分支实现进内核，但 host 仍前置拒绝 `use_shadows=true`（S7b3 策略）→ 阴影分支在产品路径不可达

## 1. 改动清单（行数为 LF 字节口径实数）

| 文件 | 改前 | 改后 | 改动 |
| --- | --- | --- | --- |
| `src/render/kernel/megakernel.rs` | 492 | **490** | 移除 L97-99 的 S7b3 前置拒绝（替换为 1 行解锁说明注释）；L403-405 第 40 槽改调纯函数 `use_shadows_arg(resolved.use_shadows)`；L9 文件头与 L77 `render()` doc 的「无阴影」表述同步；`megakernel_arrays` import 增加 `use_shadows_arg` |
| `src/render/kernel/cuda/render_megakernel.cu` | 340 | **340** | **仅注释**：L11 语义边界由「阴影/天空盒/splat/flex/hfield/SDF 不在本片（`use_shadows=false` 由 host 拒绝）」更正为「阴影（R09；host 自 R09-S2b 起不再前置拒绝）；天空盒/splat/flex/hfield/SDF 不在本片」；L15 `splat_scale = [znear, zfar, shadow_light_fraction]`（注明第三分量原占位 unused，R09-S2 起改作 shadow_light_fraction） |
| `src/render/kernel/megakernel_arrays.rs` | 241 | **326** | 新增 2 个纯函数：`scale_constants(znear, zfar, shadow_light_fraction) -> Vec<f32>`、`use_shadows_arg(bool) -> RenderArg`；`shading_words` 的 `splat_scale` 三常量改调 `scale_constants`（变量名 `scale_words`）；新增 `#[cfg(test)] mod tests`（4 条单测） |
| `tests/render_kernel_flags.rs` | 457 | **475** | 用例 `aa_supported_and_remaining_rejections_pre_launch`：`use_shadows=true` 从拒绝 cases 移出 → **独立「接受 → `Ok(())`」断言**；剩余 2 类（`use_precomputed_rays=false`、`render_skybox`）仍断言 `Unsupported`；`println!` 文案更新；**用例仍 `#[ignore = "需要 GPU + NVRTC"]`** |
| `tests/render_kernel_rgb.rs` | 257 | **270** | 用例 `unsupported_configurations_are_rejected_before_launch`：同一改法（`use_shadows=true` 独立接受断言 + 其余 2 类仍拒绝 + 文案更新）；**用例仍 `#[ignore = "需要 GPU + NVRTC"]`** |

全部 ≤500 行；无新文件（证据目录除外）。

## 2. 零改动面（逐项核对）

- `src/render/kernel/cuda/render.cu` **499 行贴线**（未动）、`src/render/kernel/launch.rs` **500 行满线**（未动）、`src/render/kernel/cast.rs` **494 行**（未动）
- 81 槽签名：`src/runtime/launcher/render/args.rs::MEGAKERNEL_SLOTS` 未改；S7a 槽签名证据未改（**不重冻**）
- `camera_output.rs` 499 / `model/render.rs` 500 / 设备 `.cu`（`render_common.cu` 387、`render_math.cu` 309、`render_shade.cu` 359、`render_shadow.cu` 30）/ 碰撞 `DEVICE_SOURCES=28` / `CollisionArgs` 504 B 未动
- `use_shadows=false` 路径代码级不变（差异仅在解锁、注释与测试/单测面；`render_megakernel.cu` 除注释外零改动）
- 判据/容差/参考/夹具零改动；未删除或降级任何用例

## 3. 第 40 槽与 scale 槽（口径）

- 第 40 槽 = **0-based `MEGAKERNEL_SLOTS[40]`**（= 1-based 第 41 项 = `Scalar(Bool)`）；与 S2 receipt 的「slot 40 stays false」编号口径一致。S2 已把该槽接 `resolved.use_shadows`，本片移除 host 拒绝后该值在产品路径可达
- `splat_scale[2]`（原占位 unused）= `shadow_light_fraction`（上游 `render.py` L880；内核读 `splat_scale.data[0][2]`）；81 槽不变、零新槽
- 接线单测位置：`megakernel_arrays.rs` 的 `#[cfg(test)] mod tests`；**不构造完整 `ResolvedContext`**（30+ 字段且无 `Default`，构造属集成测试面），「`resolved.shadow_light_fraction` → `scale_constants[2]`」由纯函数映射 + `shading_words` 单一调用点
  `scale_constants(resolved.znear, resolved.zfar, resolved.shadow_light_fraction)` 共同保证

4 条 in-crate 单测（全部通过）：

| 测试 | 锁定内容 |
| --- | --- |
| `scale_constants_maps_shadow_fraction_to_third_component` | `(0.1, 100.0, 0.3)` → index 0/1/2 分别 `znear`/`zfar`/`shadow_light_fraction`（bit 级断言） |
| `scale_constants_placeholder_now_is_shadow_fraction` | 原占位 `0.0` 语义已改为「shadow_light_fraction=0」；默认 0.3 原样落到 index 2 |
| `use_shadows_arg_maps_boolean_to_slot_40_value` | `true`/`false` → `RenderArg::Scalar(RenderScalar::Bool(..))`（`matches!`） |
| `slot_40_is_bool_scalar_in_frozen_signature` | `MEGAKERNEL_SLOTS[40] == Scalar(Bool)`；槽数仍 81 |

## 4. 内核编译门（NVRTC 纯编译，已在本片内实测）

- 命令：`cargo test --locked --features cuda-probe --test render_kernel_rgb -- --ignored --nocapture megakernel_source_compiles_under_render_gate`（PATH 前置 `target/toolchains/cuda-12.8.1/bin`）
- 结果：`1 passed; 0 failed`（0.63 s），`check_render_ptx` 通过；GPU 前后 `2555 MiB / 16376 MiB / util 2%` **无变化**（未发射内核）；日志见 `logs/kernel-compile-gate.log`
- **新 `MEGAKERNEL_SOURCE`**：sha256 `170457174286d9ce128ab5b91690331fe86f5da91a4f75d6e29099c9e69d22ff`、**80 406 B**（六段 `include_str!` 字节直拼独立复核，与测试输出 `source_sha256` 一致）
- **新 PTX**：sha256 `1e37c50ffc55a5eeeef0520a432d3f56e89a99985f8e204b5a24120f8e311b54`、**259 107 B**
- 与 S2 对照：source 由 `d38d5067…`（80 305 B）变为新值（`.cu` 头注释 L11/L15 改动 → 预期）；**PTX sha 与字节数与 S2 逐位相同** → 注释不进 PTX，本片实测证实，无代码生成变化
- 无测试硬编码 `MEGAKERNEL_SOURCE`/PTX sha（全仓 grep 仅打印），无需改常量

## 5. 四门禁（绑最终树）

`normify gate 2026-10-28-render-bvh-r09-s2-host-guard-unlock` → exit 0，日志 `logs/gates.log`。
门禁在本片执行两次：in_progress 提交前首跑、提交后绑树复跑（提交改变 HEAD 后 normify 判定旧记录过期，故重跑）；两次均 exit 0、unit 均 **1049 passed / 0 failed / 384 ignored**。最终有效记录见 `.normify/runs/2026-10-28-render-bvh-r09-s2-host-guard-unlock/`。

| 门禁 | 结果 | 耗时 | 基线（S2，历史） |
| --- | --- | --- | --- |
| fmt | 通过 | 1 562 ms | 1 613 ms |
| clippy | 通过 | 5 889 ms | 405 ms |
| clippy-cuda | 通过 | 405 ms | 404 ms |
| unit | 通过 | 68 287 ms；**1049 passed / 0 failed / 384 ignored** | 1045 passed |

unit 由 1045 → 1049（+4 = 本片新增单测）；run 记录 `.normify/runs/2026-10-28-render-bvh-r09-s2-host-guard-unlock/`（scope_hash `952955eb…`）。

## 6. 历史证据边界（不重冻、不改旧值）

- S2 证据 `docs/development-evidence/render-bvh/r09-s2-lighting-kernel/` 的 `source_sha256=d38d5067…` / 80 305 B、`ptx_sha256=1e37c50f…` / 259 107 B **保留为历史**（绑 S2 树）；QA `r09-s2-check` 的 recompute 值同理
- 新值只进本目录；81 槽签名、`DEVICE_SOURCES`、S7a 签名证据均不变（**不重冻**）

## 7. 限制与口径（必须随证据保留）

- 本片**不产任何渲染数值结论**：`use_shadows=false` 的逐位回归与 `use_shadows=true` 的像素级行为仍在 R09-S3b 窗内验证；两条 GPU 断言在本片只做「编译通过 + 断言形态改造」，实际接受/拒绝行为留窗内执行
- 不写任何窗口结论、覆盖声明、精度结论或等价结论（含上游与双平台）；本片不宣称阴影行为可用性或数值结论
- 判据/容差/参考/夹具零改动；未删除或降级任何用例
