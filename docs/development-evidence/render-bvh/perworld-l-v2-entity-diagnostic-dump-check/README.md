# QA 复核：G-L2 判据 A 像素诊断 dump 代码卡（`6dc5e5f6` + addenda `b526d4f1`）

- 卡：`2026-11-11-render-perworld-l-v2-entity-diagnostic-dump-check`（owner QA）
- 被复核：`6dc5e5f6`（实现）+ `b526d4f1`（卡 addenda：Chief 修订字段口径，代码零改动）；HEAD `b526d4f1`
- 性质：**只读复核**，test-only，零 GPU；未运行诊断轮、未碰冻结件与 G-L2 证据目录。
- 结论：**PASS（0 阻断）→ 可发诊断轮 token**。诊断 dump 只服务 RCA，**不参与验收判定**；不改变 G-L2 判定（未通过），不建议任何放宽。

## 1. 只加诊断、不改判据（逐行 diff）

| 文件 | 变化 | 行数 | 结论 |
| --- | --- | --- | --- |
| `tests/support/render_bvh/light/diag.rs` | 新增 | **493**（≤500） | 纯诊断模块，`#![allow(dead_code)]` |
| `tests/render_per_world_light.rs` | **+14 / −1** | 478→**491**（≤500） | 仅 `mod light_diag;` 声明 + 一处 hook |
| `tests/support/render_bvh/light.rs` | **0 diff** | **499**（未动） | 判据/统计逻辑所在文件零改动 |

`render_per_world_light.rs` 的 diff 只有两个 hunk：
1. 追加 `#[path = "support/render_bvh/light/diag.rs"] mod light_diag;`；
2. 把 `for (world, row) in light::criterion_a(&reference, &product).iter()` 改为
   `let criterion_a_stats = light::criterion_a(&reference, &product);` + `light_diag::dump_if_enabled(&DumpInput{…})` + `for … in criterion_a_stats.iter()`。

⇒ **断言 `assert_eq!`、阈值、depth/seg 比较、参考、相机归属、判据 A 顺序全部未出现在 diff 中（逐字节相同）**；`light.rs` 499 行零改动。

## 2. 默认零副作用（env 未设）

`dump_if_enabled()` 第一行即 `let path = output_path()?;`：
- `MJWARP_DIAG_PIXEL_DUMP` **未设或 trim 后为空** → 立即返回 `None`；
- `build_document()`（JSON 构造、模型数组解析、邻域/材质遍历、near-miss 统计）**完全不被调用**；
- 无 `println!`、无 `create_dir_all`、无 `fs::write`、无额外渲染/GPU 读取；
- 调用方只多构造一个全借用的 `DumpInput`（无拷贝、无分配）；
- 该 hook 位于 `#[ignore]`（需 GPU + NVRTC）用例内，默认门禁不执行；
- `unit` 仍 **1387 passed / 0 failed / 402 ignored**（未新增测试、默认行为不变）。

## 3. 字段符合修订口径（对照 `b526d4f1` addenda）

| 修订口径要求 | 实现位置（diag.rs） | 结果 |
| --- | --- | --- |
| 两侧字节 + packed u32 + 0/255 饱和标记 | `packed_u32`/`bytes`/`saturation()`（每通道 zero/full/saturated） | ✅ |
| 3×3 邻域 | `neighborhood_3x3()` 产品+参考两侧、越界 `null` | ✅ |
| seg + geom/mat 归属 + 材质标量 | `segment_attribution()`：`geom_id`/`geom_type`/`mat_id` + `mat_texid_row`/`mat_shininess`/`mat_specular`/`mat_emission`/`mat_reflectance`/`mat_metallic`/`mat_roughness`/`mat_rgba` | ✅ |
| depth/seg 位型 + ULP | `product_bits`/`reference_bits`/`ulp()`；seg 两侧 | ✅ |
| ray 位型 | `camera_ray_bits()`：`ray_index`/`direction_bits`/`offset_bits` | ✅ |
| 相机位型 | `xpos` 3 位型 + `xmat` 9 位型 | ✅ |
| 三灯两侧位型 | `world_light_bits()`：每 world × 3 灯 × 参考 `state.npz` / 产品 `WorldLightPoses`，`get()` 失败给 `null` | ✅ |
| near-miss：同字节像素饱和区比例 + 最接近失配 5 像素分 geom 归属 | `near_miss()`：`equal_byte_pixels`/`mismatched_pixels`/`saturated_any_pixels`/`saturated_any_ratio_of_equal`/分通道 zero+full/`nearest_to_mismatch`（含距离、最近失配像素、seg+归属）/`nearest_geom_histogram` | ✅ |
| **移除无判别力的 `\|ref_f32*255−round\|` 边界距离** | 全文无该字段（仅模块 doc 说明「已移除」） | ✅ |

## 4. 门禁 / 边界 / 卫生

- `6dc5e5f6` 门禁 runs：fmt / clippy / clippy-cuda / unit **全部 `passed=true` exit 0**；unit **1387 / 0 / 402**。
- `normify check` 该卡：全部检查通过（exit 0）；`normify validate`：**错误 0**（5 条预存在架构 warning，与本卡无关）。
- 边界：本卡 diff 面 = 2 个 tests 文件 + 4 个 runs + 卡 JSON；`unchanged src/`、`fixtures/`、`scripts/`（commit numstat 无这两类路径）。
- 编码：两个改动文件 **BOM=False、CR=0**（UTF-8/LF）。
- 本复核零 GPU、未跑诊断轮、未改冻结件/G-L2 证据。

## 5. 观察项（信息级，不阻断）

1. **O1**：卡 `acceptance[2]/[3]` 仍是**修订前**口径（含 `|ref_f32*255−round|` 边界距离、旧 near-miss「距整数 <1e-5/<1e-4」）；`b526d4f1` addenda 已记录「代码按修订实现、运行中任务的 acceptance/checks/intent 冻结」⇒ 阅读以 addenda 为准；建议后续 errata 在 acceptance 旁加一行指针。
2. **O2**：`depth_mismatches`/`seg_mismatches` 条目只含位型/ULP/seg，不附 ray/灯/相机/材质归属（全量归属仅在 rgb 失配条目）。本次 G-L2 只有 rgb 2 px，不影响本轮 RCA。
3. **O3**：`near_miss` 的「最近失配」计算为 O(PIXELS × 失配数)，失配数很大时会慢（诊断 opt-in 专用；本次 2 px 无影响）。
4. **O4**：模块级 `#![allow(dead_code)]` 会掩盖未用代码（当前各函数均被引用，仅日后清理提示）。

## 6. 非声明

本复核不宣称判据 A 已修复、不宣称 G-L2 通过或恢复、不宣称任何数值等价；dump 只服务 RCA，不参与验收判定；未改任何判据/阈值/参考/相机/recipe/冻结件。
