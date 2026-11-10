# perworld L v2 产品侧扩展像素 dump（opt-in 诊断缓冲；零 GPU；算术未变证明）

- 任务：`2026-11-11-render-perworld-l-v2-diag-dump-ext`（owner Backend；p90；P8-渲染与BVH）。恢复被暂停的卡片（此前在登记/改动前暂停、无 WIP）。
- 依据：Arch 字段规格 `target/render-bvh/gl2-ptx-diff/texture-chain-and-dump-spec.md` §C（打包前 color、base_color、tex_uv、tex_color、求交、normal、view_dir、每灯中间量）。
- 口径：**纯诊断**——零 GPU、不采集、不跑诊断、不改判据/参考/容差；env 未设时**零副作用**；只新增 store/分支，不改任何 f32 表达式/结合序（离线 NVRTC PTX 证明算术指令流与 `755b3d07`（P9 版）逐条一致）。不宣称修复；G-L2 维持未通过。

## 1. 改动清单（11 个源文件 + 4 个 ABI 棘轮测试同步 + 模块源码登记；不新增 .cu 文件，避免改精度源面快照）

| 文件 | 作用 |
| --- | --- |
| `src/render/kernel/mod.rs` | `MEGAKERNEL_SOURCE` 在 `render_common.cu` 后插入 `render_diag.cu`；`pub mod diag` 与类型导出 |
| `src/render/kernel/cuda/render_megakernel.cu` | 顶部新增 DIAG 布局常量 + store helper（tex/light/hit；`diag_ptr==0` 时全部首行返回）；签名尾随 `wp::uint64 diag_ptr`；命中分支写 color/base_color/求交/normal/ray_dir；把 diag 指针传给 texture/lighting |
| `src/render/kernel/cuda/render_shade.cu` | helper 前向声明；`sample_texture` 写 tex_uv/tex_color；`compute_lighting` 写每灯 weight/visible/ndotl/ndoth/half/pow（headlight 用第 4 slot）；`visible`/`spec_pow` 提为局部量（表达式树不变） |
| `src/render/kernel/diag.rs`（新增） | env 门控、`DiagCapture`/`DiagBuffer` 分配/回读、`DiagRecord` 解码、`MegakernelRenderer::diag_record` |
| `src/render/kernel/megakernel.rs` | `MegakernelRenderer` 增 `diag` 状态；render_impl 中 begin/回读；行数 500（500 行棘轮内） |
| `src/render/kernel/megakernel_args.rs`、`src/runtime/launcher/render/args.rs`、`src/render/kernel/megakernel_arrays.rs` | 签名 81→82 槽（尾随 `Scalar(U64)`，上游 81 槽之外；断言同步） |
| `tests/support/render_bvh/light/diag.rs`、`tests/support/render_bvh/light/diag_ext.rs`（新增）、`tests/render_per_world_light.rs` | dump schema 2：每个 rgb 失配像素附 `diagnostics` |
| `tests/render_kernel_rgb.rs`、`tests/render_launch_abi.rs`、`tests/render_lighting_kernel.rs`、`tests/render_skybox.rs` | 冻结 81 槽 ABI 棘轮断言同步为 82（1 bounds + 73 数组 + 8 标量）；上游 81 槽语义与顺序不变，只在末尾追加诊断槽；精度 CUDA 源面快照（42 文件）零改动 |

模块源码登记：`normify module source mjwarp.render.kernel --add src/render/kernel/diag.rs`（27 个源码；不新增 .cu 文件，精度源面快照保持 42 文件不变）。

实际源码 diff（`git diff --stat`，不含模块记录）：

```text
 src/render/kernel/cuda/render_megakernel.cu | 105 ++++++++++++++++++++++++++--
 src/render/kernel/cuda/render_shade.cu      |  18 +++--
 src/render/kernel/megakernel.rs             |   7 +-
 src/render/kernel/megakernel_args.rs        |   3 +
 src/render/kernel/megakernel_arrays.rs      |   3 +-
 src/render/kernel/mod.rs                    |   2 +
 src/runtime/launcher/render/args.rs         |   4 +-
 tests/render_per_world_light.rs             |   3 +
 tests/support/render_bvh/light/diag.rs      |   3 +
 9 files changed, 136 insertions(+), 12 deletions(-)
```

## 2. dump schema（`MJWARP_DIAG_PIXEL_DUMP` opt-in；schema 2）

- **门控**：env `MJWARP_DIAG_PIXEL_DUMP` 非空才启用（产品与测试共用）；未设/空 = 不分配、不回读、不输出、内核 `diag_ptr=0` 跳过所有 store。
- **设备缓冲**：每次 `render_impl` 分配 `launch_worlds * total_rays * 64` 个 f32；内核按 `记录下标 = worldid * total_rays + rayid` 写；测试侧按 `(world, cam*PIXELS+pixel)` 取单条。
- **每 ray 记录（stride 64 f32；偏移见 `cuda/render_megakernel.cu` 顶部 `DIAG_*` 常量）**：

| 偏移 | 字段 | 含义/编码 |
| --- | --- | --- |
| 0–2 | `color` | 打包前 f32 颜色（`hit_color`，clamp 后） |
| 3–5 | `base_color` | 纹理乘后 base_color |
| 6–7 | `tex_uv` | wrap 后的采样 uv（`u - floor(u)`） |
| 8–10 | `tex_color` | 硬件 `texture_sample` 返回的 rgb |
| 11–12 | `bary_u`/`bary_v` | 命中重心坐标 |
| 13 | `t` | `hit.dist`（未乘 `-dir.z`） |
| 14 | `face` | i32 位型（f32 槽位里存 `__int_as_float(face)`） |
| 15 | `mesh_id` | i32 位型（同上） |
| 16–18 | `hit_point` | 世界系命中点 |
| 19–21 | `normal` | 用后法线（含顶点法线/背面翻转） |
| 22–24 | `ray_dir` | 世界系射线方向；`view_dir = -ray_dir`（精确负号位翻转，host 解码时派生，见下） |
| 25–56 | `lights[4]` | 每灯 8 槽：`weight, visible, ndotl, ndoth, half.xyz, spec_pow`；灯 0..nlight-1 用原下标，headlight 用第 4 slot；未点亮的灯槽保持 0（缓冲已清零） |
| 57–63 | 保留 | 恒 0 |

- **编码**：所有 f32 以 `u32` 位型输出（`to_bits`，见 `diag_ext.rs`），JSON 中为整数数组；`face`/`mesh_id` 解码回 i32；`view_dir` 由 `ray_dir` 精确取负派生（避免内核为落盘额外 materialize 3 条 `neg.f32`——这正是「算术未变」要求的直接结果）。
- **未命中/非命中分支像素**：不写记录（缓冲为 0），host 解码为 `null`；本次目标 4 px 均为命中像素。
- **JSON 位置**：`pixel-dump.json` 的每个 `rgb_mismatches[i].diagnostics`（未启用 env 时不存在该字段，dump 其余行为不变；顶层 `schema` 由 1 → 2）。

## 3. 内核身份 old → new（NVRTC 12.8.93 纯编译，无 GPU 会话）

| artifact | 值 | old（HEAD `755b3d07` = P9 build） | new（工作树） |
| --- | --- | --- | --- |
| megakernel | source sha256 | `02160146…e260a1` | **`624ab298…6bb3e2`** |
| megakernel | PTX sha256 / bytes | `7fd86b85…995c7e` / 344,181 | **`1fd53131…c430b8`** / **347,070** |
| aa_resolve | source/PTX/bytes | `0bf8af5a…` / `b28e155c…` / 27,536 | 未变 |
| depth_seg | source/PTX/bytes | `b546ef08…` / `e8d84963…` / 138,540 | 未变（`render_shade.cu` 不在拼接中） |

- old source sha 由 `logs/old-source-recompute.txt` 独立拼接复算得 `02160146de4ea26b2e96efb2309f02dda0c26a525747cd2f28ea8f0d02e260a1`（与 P9 卡证据一致）；new 由 `logs/new-source-recompute.txt` 得 `624ab298e82e7c94dc66b101ca5e2abd141556beaf2d53485f0b3b8c4f6bb3e2`（与 identity 用例打印逐位一致）。
- PTX 体积 +2,889 B 全部来自新增参数/分支/store（见 §4；helper 定义位置不影响内核 PTX——与先行版本逐字节相同）。

## 4. 「算术未变」证明（离线 NVRTC，零 GPU）

**方法**：用 pinned CUDA 12.8.1 NVRTC、product 选项（`--fmad=true --include-path=include/warp-headers --std=c++17 --gpu-architecture=compute_70 --device-as-default-execution-space`）分别编译 old（HEAD blobs）与 new（工作树）拼接源码；抽取全部 f32 算术指令（add/sub/mul/fma/div/sqrt/rcp/neg/min/max/abs 的 `.f32` 形式 + `cvt.*.f32`）的 opcode 序列并逐条比对。

| 项 | old | new |
| --- | --- | --- |
| stitched source bytes / sha256 | 104676 / `02160146de4ea26b2e96efb2309f02dda0c26a525747cd2f28ea8f0d02e260a1` | 109384 / `624ab298e82e7c94dc66b101ca5e2abd141556beaf2d53485f0b3b8c4f6bb3e2` |
| 离线 PTX bytes / sha256 | 344182 / `5abc0b4e3cd7bd6ea89cd0df0aaa19bdb9e9266e4f7906fd2a4dcf8800fac853` | 347071 / `c2d8a885e3f05ba798a3b66c22065855ea8eb641d8515a3b28ca278f7d2a162a` |
| 算术指令数 | 3148 | 3148 |
| opcode 序列逐条一致 | — | **True** |
| opcode 多重集一致 | — | **True** |

⇒ 除新增 store/分支与寄存器重命名外，**算术指令流与 `755b3d07`（P9 版）逐条一致**；`logs/ptx-arith-compare.txt` 为原始对照记录，`logs/ptx-arith-per-line-diff.txt` 显示按 `.loc` 的差异仅为源码行号平移（本卡插入行导致），**无算术 opcode 变化**。

设计上避免了两类可能改变算术的写法：
1. 不复制 `powf`/`shadow_visibility` 调用——把 `visible`/`spec_pow` 提为局部量后复用（表达式树不变）；
2. 落盘 `ray_dir` 而非 `view_dir`——`view_dir` 若落盘会迫使内核 materialize 3 条 `neg.f32`（首版实测 +3 neg），改存 `ray_dir` 后 opcode 序列完全一致，host 侧按精确符号位派生 `view_dir`。

## 5. 验证（零 GPU）

| 项 | 结果 |
| --- | --- |
| `cargo build --locked --features cuda-probe` | exit 0（`logs/build-cuda-probe.log`） |
| `cargo test --locked --features cuda-probe --no-run` | exit 0（`logs/no-run-cuda-probe.log`；含新 `diag_ext` 测试往入） |
| 四门禁 fmt / clippy / clippy-cuda / unit | 全过；unit **1,387 过 / 0 败 / 402 ignored** |
| 内核身份用例（NVRTC 纯编译） | 2 个用例各 1 passed / 0 failed |
| 离线算术对照 | opcode 序列/多重集均一致（`logs/ptx-arith-compare.txt`） |

## 6. 边界与非声明

- 边界：零 GPU、不采集、不跑诊断、不重冻、未动 guards/冻结件与已签材料；`unchanged fixtures/`、`unchanged scripts/`、其它 `.cu`（`render_common/math/flex/shadow/render/aa_resolve/scene.cu`）零改动；`render_shade.cu` 473 行、`render_megakernel.cu` 414 行、`megakernel.rs` 500 行（棘轮内）。
- 非声明：不宣称修复生效、不宣称渲染数值等价；不改判据/阈值/参考/容差；G-L2 判定维持未通过（判据 A FAIL）；诊断轮 #7 由 Chief 另发 token。
