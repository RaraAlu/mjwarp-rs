# Card H：host per-world 灯入口复核（QA 只读）

任务：`2026-11-08-render-perworld-l-v2-host-light-entry-check`（owner QA）。日期：2026-11-08。
复核对象：Card H `2026-11-08-render-perworld-l-v2-host-light-entry`（verified；登记 `38797130` → start **基线** `2e207e32` → 实现 **`773cbe99`**）。
依据：已签 spec v3 §10.3（host-only per-world 灯位姿、`.cu` 零改动、内核身份不变）与 §13-3（refit）/§6 判读边界。
边界：只读 + 只写本卡证据；零 GPU、不采CUDA；不改被复核代码/证据/卡状态。

**结论：PASS-with-observations（0 阻断；1 条中观察 = 失败恢复声明与代码不符、且为基线既有；2 条信息项）。**

## 1. 内核身份不变（独立重算）

| 项 | 实测 | 判定 |
| --- | --- | --- |
| `src/render/kernel/cuda/**` 文件数 | 8 / 8 | ✅ |
| 基线↔HEAD blob 比对 | `git ls-tree -r` 逐项 **IDENTICAL**；`git diff 2e207e32..773cbe99 -- src/render/kernel/cuda/` **为空** | ✅ |
| 工作树 8 个 `.cu` sha256 | 与 `artifacts/kernel-identity.json` **逐文件相同**（如 `render_megakernel.cu 6d718100…`） | ✅ |
| `MEGAKERNEL_SLOTS` | 声明 `[RenderArgKind; 81]`（`src/runtime/launcher/render/args.rs`，**本提交未改**）；单测 `slot_40_is_bool_scalar_in_frozen_signature` 断言 `len==81` 并独立重跑通过（1 passed） | ✅ 源码+单测双证 |
| 81 槽排列 / 入口签名 | `args.rs` 与本提交零 diff；`MEGAKERNEL_ENTRY` 常量未动；`.cu` 零 diff ⇒ 入口签名不变 | ✅ |

## 2. host-only 机制（每 world 重传，不是只传一次）

代码路径（逐层核）：

1. `per_world.rs::render_per_world_impl`：`for world in 0..worlds` 内每轮先 `apply_poses` 该 world geom 位姿，再调 `renderer.render_world_with_lights(..., world, lights)`（`lights` 仅在 v2 入口为 `Some`）。
2. `megakernel.rs::render_world_with_lights` → `render_impl(RenderPass{ world: Some(world), lights: Some(lights), .. })`。
3. `render_impl` 内：`let launch_worlds = if world.is_some() {1} else {nworld};`；灯参数 `match lights { Some(l) => select_world_lights(l, world, fields.light_type.len())?, None => light_data_poses(..) 展平 }`；随后**函数内**构造 `uploads = upload_megakernel_arrays(&UploadInputs{ light_xpos: &light_xpos, light_xdir: &light_xdir, .. })` 并发射。

⇒ 每次 world 发射都重新选择该 world 的灯位姿并重新上传（每次调用 `select_world_lights` + `upload_megakernel_arrays`），**不是只传一次、也不是只在首 world 传** ✅。
`adr_world = world`（输出位移）与 `launch_worlds=1` 为既有 v1 机制，未改；内核 `worldid` 恒 0 语义由 `.cu`/args 零改动保持 ✅。
无 2-D 灯数组（`light_xpos/light_xdir` 仍 `Vec<f32>` 1-D、81 槽未动）、无单发射/AA×perworld/多相机新内核路径 ✅。
**停手条款（§10.3）结论：未触发** —— 无需改内核，材料结论与代码一致 ✅。

## 3. 数据契约（`WorldLightPoses`）

- 构造校验：`worlds>=1`（`WorldCountZero`）、`lights>=1`（`LightCountZero`）、`poses.len()==worlds×lights`（`CountMismatch{expected,actual}`）、`checked_mul` 溢出（`CountOverflow`）✅。
- 越界：`get`/`world_slice` → `WorldOutOfRange`/`LightOutOfRange`；索引 `checked_mul + checked_add` ✅。
- 入口校验：world 数 ≠ context nworld → `InvalidArray("per_world_light_worlds")`；灯数 ≠ `fields.light_type.len()` → `InvalidArray("per_world_light_count")`；world 越界 → `Driver("per-world lights: …")` ✅ 全部可诊断、不 panic。
- 位透传：`flatten_light_poses` 只做 `extend_from_slice`（按 world 内灯序，先 xpos 后 xdir）——**无 FK、无 f64 重算、`xdir` 不重归一化** ✅；与默认路径旧内联循环顺序等价（旧代码逐 pose 写 xpos 再 xdir），默认路径行为不变 ✅。
- 数据来源：文档写明取自采集侧 `state.npz` 的 `light_xpos/light_xdir`（`capture_render_perworld.py::state_arrays`，data-driven，非产品物理推导）✅。
- 与 `WorldScenePoses` 同风格（world 计数、越界、可诊断错误、不 panic）✅。

## 4. 默认路径不变

- `render_per_world` 公开签名与基线**逐字相同**（8 参数、返回 `Result<(), LauncherError>`），内部委托 `impl(lights=None)`；实现体除新增校验/分支外无其它改动 ✅。
- `light_mode!=0` / `light_bodyid!=0` 拒绝仍只在默认路径：`light_data_poses`（`megakernel_arrays.rs`）**本提交未改**，且新测 `default_path_still_rejects_tracked_lights` 覆盖两字段 ✅（拒绝面未全局放开；v2 路径不经过该推导，符合 §10.3）。
- 默认路径 golden：`default_path_light_data_poses_keep_golden_bits` 的 3 组 `xpos_bits/xdir_bits` 常量与 `fixtures/render-bvh/scene-a/s7b/rgb.json.light_data_check` **逐项一致**（我逐项比对）✅。
- v1 geom-only 入口行为不变：既有 `per_world` 单测（`rejects_multi_sample_per_world_entry`、`splits_leaf_slots_…`、`replicates_target_world_poses_and_restores_caller_mapping`）全过 ✅。
- `nsamples>1`（AA×perworld）仍 `Unsupported`：`reject_multi_sample` 未改，且 `render_impl` 内对 `world.is_some() && nsamples != 1` 再拒 ✅。

## 5. 行数/新文件/模块登记

| 文件 | git-blob 行数 | ≤500 | 证据 |
| --- | --- | --- | --- |
| `world_light.rs`（新） | **371** | ✅ | sha `003b466e…`，唯一新增源文件 |
| `per_world.rs` | **338** | ✅ | |
| `megakernel.rs` | **497** | ✅ | |
| `mod.rs` | **479** | ✅ | |
| `megakernel_arrays.rs` | **411（未改）** | ✅ | sha 前后相同 `326d001d…` |

- 本提交唯一新增文件 = `world_light.rs`（`git show --name-status 773cbe99` 核）✅。
- 模块登记：`.normify/modules/mjwarp/render/kernel.md` 新增 `[[source]] path = "src/render/kernel/world_light.rs"` 并刷新 `fingerprint` ✅。

## 6. 测试独立重跑

| 命令 | 结果 |
| --- | --- |
| `cargo test --locked --lib world_light` | **8 passed / 0 failed**，exit 0（7 个 world_light 单测 + `per_world::validates_per_world_light_world_count_without_panic`） |
| `cargo test --locked --lib per_world` | **6 passed / 0 failed / 1 ignored**（既有 GPU 用例 ignored），exit 0 |
| `cargo test --locked --lib slot_40_is_bool_scalar_in_frozen_signature` | **1 passed**，exit 0 |
| 四门禁（本卡独立复跑） | fmt/clippy/clippy-cuda 通过；**unit 1367 过 / 0 败**（基线 1359，+8） |

新测断言的是真事实（非空壳）：world-major 存取 + `f32::to_bits` round-trip、计数不匹配/零 world/零灯/`checked_mul` 溢出、world/light 越界、展平顺序 + 逐位保持、`InvalidArray("per_world_light_count")`、默认路径 golden、默认路径拒绝 `light_mode/light_bodyid != 0`、入口 world 数校验。

## 7. 边界/证据/非宣称

- `git diff 2e207e32..773cbe99` 对 `fixtures/`、`scripts/`、`docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/` **均为空** ✅；diff 面仅 `src/render/...` 4 文件 + 模块 md + 卡/runs/证据 ✅。
- `artifacts/src.diff` 与实时 `git diff -- 2e207e32..HEAD -- src/` **逐字节相同**（sha `462523c1…`）✅。
- 证据 manifest **8/8** 独立重算通过、`# base dir:` 在位、UTF-8/LF/无 BOM/裸 CR·TAB 0 ✅。
- 零 GPU：本卡只跑 CPU 单测，未建 CUDA context、未采集、未开窗 ✅。
- README §8 如实写「不宣称渲染通过/灯位姿已覆盖/验收/上游等价/渲染数值；GPU 端到端留给 G-L2」✅。

## 8. 观察项（按严重度）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | **中（基线既有）** | 失败恢复声明与代码不符：`per_world.rs` 的 `let restore = match outcome.is_ok() { false => Ok(original) }` 之后，应用条件是 `if outcome.is_ok() && let Err(..) = restore.and_then(..)` —— **失败时 `outcome.is_ok()` 为 false，`Ok(original)` 从不被 apply**，即失败路径实际**不恢复**调用前位姿；但注释写「失败则恢复调用前位姿（尽力而为）」，Card H receipt 亦写「失败尽力恢复调用前场景位姿」。该段与基线 `2e207e32` **逐字相同**（本卡 diff 未触及），属 v1 既有问题，非 Card H 引入；不影响本卡的接口/内核身份/默认路径结论 | 另开小 fix：失败时也 apply `original`（注意不掩盖原始错误，可用 `outcome.and_then` 组合后保留首错），或把注释/receipt 文案改为「失败不保证恢复」。建议与 G-L2 前一起收 |
| O2 | 信息 | README §5 写「`world_light` 8 passed」，其中 1 个是 `per_world` 模块的校验测试（filter `world_light` 命中）；过滤口径而非模块口径 | 无实质影响；下次可写「filter world_light → 8 passed」
| O3 | 信息 | Card H 卡 `schedule.files` 列了 `megakernel_arrays.rs`（本提交未改） | 无害（写入范围声明）；不影响判定 |

## 门禁

fmt / clippy / clippy-cuda 通过，**unit 1367 过 / 0 败**（本卡独立复跑，记录见 `.normify/runs/2026-11-08-render-perworld-l-v2-host-light-entry-check/`）。

## 非宣称

- 本复核只判 Card H 的实现与证据是否按已签 spec v3 §10.3 落地；**不判渲染数值**、不宣称灯位姿已覆盖/判据 A/B/C 通过/与上游等价。
- GPU 端到端渲染数值由 G-L2 实体承担；未改被复核代码、证据与任何卡状态。
