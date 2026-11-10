# perworld L v2 产品侧扩展像素 dump（opt-in 诊断缓冲；零 GPU；算术未变证明）

- 任务：`2026-11-11-render-perworld-l-v2-diag-dump-ext`（owner Backend；p90；P8-渲染与BVH）。恢复被暂停的卡片（此前在登记/改动前暂停、无 WIP）。
- 依据：Arch 字段规格 `target/render-bvh/gl2-ptx-diff/texture-chain-and-dump-spec.md` §C（打包前 color、base_color、tex_uv、tex_color、求交、normal、view_dir、每灯中间量）。
- 口径：**纯诊断**——零 GPU、不采集、不跑诊断、不改判据/参考/容差；env 未设时**零副作用**；只新增 store/分支，不改任何 f32 表达式/结合序（离线 NVRTC PTX 证明算术指令流与 `755b3d07`（P9 版）逐条一致）。不宣称修复；G-L2 维持未通过。

## 1. 改动清单（本卡 10 个 src/tests 文件 = 8 src + 2 tests；配套卡 5 个 = `diag_ext.rs` + 4 个 ABI 棘轮测试；合计 15；不新增 .cu 文件，避免改精度源面快照）

| 文件 | 作用 |
| --- | --- |
| `src/render/kernel/mod.rs` | 仅新增 `pub mod diag;`（`MEGAKERNEL_SOURCE` 保持原 **7 文件拼接**不变：`scene.cu` + `render_math.cu` + `render_flex.cu` + `render_common.cu` + `render_shadow.cu` + `render_shade.cu` + `render_megakernel.cu`；**没有**新增 `render_diag.cu`，DIAG 布局与 store helper 内置于 `render_megakernel.cu`） |
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
$ git show --numstat bb29e351 -- src tests        # 本卡（主提交）
101    4  src/render/kernel/cuda/render_megakernel.cu
 14    4  src/render/kernel/cuda/render_shade.cu
224    0  src/render/kernel/diag.rs（新增）
  5    2  src/render/kernel/megakernel.rs
  3    0  src/render/kernel/megakernel_args.rs
  2    1  src/render/kernel/megakernel_arrays.rs
  2    0  src/render/kernel/mod.rs
  3    1  src/runtime/launcher/render/args.rs
  3    0  tests/render_per_world_light.rs
  3    0  tests/support/render_bvh/light/diag.rs
10 files changed, 360 insertions(+), 12 deletions(-)

$ git show --numstat e7fc06f7 -- tests           # 配套卡（82 槽 ABI 棘轮 + diag_ext）
  2    2  tests/render_kernel_rgb.rs
  3    3  tests/render_launch_abi.rs
  1    1  tests/render_lighting_kernel.rs
  2    2  tests/render_skybox.rs
 76    0  tests/support/render_bvh/light/diag_ext.rs（新增）
```

## 2. dump schema（`MJWARP_DIAG_PIXEL_DUMP` opt-in；schema 2）

- **门控**：env `MJWARP_DIAG_PIXEL_DUMP` 非空才启用（产品与测试共用）；未设/空 = 不分配、不回读、不输出、内核 `diag_ptr=0` 跳过所有 store。
- **设备缓冲**：每次 `render_impl` 分配 `launch_worlds * total_rays * 64` 个 f32。
- **记录下标（errata 更正）**：内核按 `记录下标 = rayid * 64` 写，`rayid` 是**单次发射内**的射线号（不是全局下标，也没有 `worldid` 项）。测试侧在 **per-world 单次发射**下按 `(world, cam*PIXELS+pixel)` 取单条（此时 `rayid = cam*PIXELS+pixel`），每个 world 的捕获单独回读保存 ⇒ 与逐 world 取值等价。**约束/风险**：若将来改成多 world 单次发射，各 world 会把记录写到同一 `rayid` 区间而**静默互相覆盖**（host 侧只能拿到其中一个 world 的数据或取空），诊断数据失效但不影响渲染数值；本实体测试走 per-world 单发射，不触发。该约束已同步写入 `src/render/kernel/diag.rs` 与 `tests/support/render_bvh/light/diag_ext.rs` 的文档。
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
- **灯槽约束（backlog）**：`lights[4]` 中灯 `0..nlight-1` 用原下标、headlight 固定第 4 slot；内核把 `light >= DIAG_LIGHT_SLOTS-1` 的灯 clamp 到第 3 槽，因此 `nlight >= 4` 时第 3 号灯与 headlight 共槽、更后面的灯也被并入该槽。仅影响诊断字段、不影响渲染数值；本场景 `nlight = 3` 不触发；登记 **backlog**（如需支持 `nlight>=4` 应扩槽或按灯数动态分配）。
- **未命中/非命中分支像素**：不写记录（缓冲为 0），host 解码为 `null`；本次目标 4 px 均为命中像素。
- **JSON 位置**：`pixel-dump.json` 的每个 `rgb_mismatches[i].diagnostics`——**该键始终存在**：未启用 env（`renderer=None`）时为 `null`，启用但该 ray 未命中/越界时也为 `null`，不会缺失（`attach_diagnostics` 契约；顶层 `schema` 由 1 → 2）。

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

- **拼接文件（7 个，未新增）**：`MEGAKERNEL_SOURCE` = `scene.cu` + `render_math.cu` + `render_flex.cu` + `render_common.cu` + `render_shadow.cu` + `render_shade.cu` + `render_megakernel.cu`；old/new 两版都只含这 7 个文件，**没有** `render_diag.cu`（DIAG 布局与 helper 内置于 `render_megakernel.cu`）。
- **PTX 字节口径（errata）**：`nvrtcGetPTXSize` 含尾部 `NUL`（+1 B）。本卡离线工装落盘的 PTX **文件字节**为 **344,182 / 347,071**（含 NUL），identity 用例按 PTX **文本长度**（去 NUL）报 **344,181 / 347,070**。实测 `target/render-bvh/gl2-ptx-diff/ptx/p9_landed_128.ptx` = 344,182 B、末字节 `0x00`，去掉该 NUL 后 344,181 B / sha `7fd86b85…` 与 identity 完全一致 ⇒ 两处数字是同一 artifact 的两种计长口径，不矛盾。

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

- 边界：零 GPU、不采集、不跑诊断、不重冻、未动 guards/冻结件与已签材料；`unchanged fixtures/`、`unchanged scripts/`、其它 `.cu`（`render_common/math/flex/shadow/render/aa_resolve/scene.cu`）零改动；文件行数（实测，errata 更正）：`render_shade.cu` **478**、`render_megakernel.cu` **499**、`megakernel.rs` **500**（500 行棘轮内）、测试 `tests/support/render_bvh/light/diag.rs` **496**、`tests/render_per_world_light.rs` **494**、`src/render/kernel/diag.rs` 224、`tests/support/render_bvh/light/diag_ext.rs` 76。
- 非声明：不宣称修复生效、不宣称渲染数值等价；不改判据/阈值/参考/容差；G-L2 判定维持未通过（判据 A FAIL）；诊断轮 #7 由 Chief 另发 token。

## 7. 后置 errata（2026-11-11，QA 复核 `00bfaef8` 后的修复）

> **schema 声明 errata**：原提交说明（`bb29e351`）与本文 §2/验收声称 dump schema 2，但 `tests/support/render_bvh/light/diag.rs` 的 `SCHEMA` 常量实为 **1**、dump 实际输出 `"schema":1`。该问题已在修复卡 `2026-11-11-render-perworld-l-v2-diag-dump-ext-fix` 修复（`SCHEMA = 2` + 纯 CPU 单测断言）；历史提交说明不可改写，以本 errata 与本目录 `receipt.json` 的 `errata_fix` 节为准。

| # | QA 项 | 修复 |
| --- | --- | --- |
| B1（阻断） | `SCHEMA` 常量 1 与声明 schema 2 不符 | `SCHEMA: u32 = 2`；新增 2 个纯 CPU 单测：顶层 `schema == 2`、rgb 失配条目恒带 `diagnostics` 键（未启用 env 为 null，不缺失） |
| G1 | README/代码引用不存在的 `render_diag.cu`；§1 与 7 文件拼接矛盾 | §1/§4、`src/render/kernel/diag.rs`、`tests/support/render_bvh/light/diag_ext.rs` 更正：无新 `.cu`、helper 在 `render_megakernel.cu`、拼接仍 7 文件；并注明原卡 `schedule.files` 曾规划该文件但最终未创建 |
| G2 | 记录下标口径 `worldid*total_rays+rayid` 与实现 `rayid*STRIDE` 不符 | §2 与代码文档更正为 `rayid*STRIDE`（单发射内）；写清 per-world 单发射等价、multi-world 单发射静默覆盖风险与约束 |
| G3 | README 行数 473/414、文件计数错误 | §6/§1 更正为实测 478/499/500/496/494；计数按实际：本卡 10 + 配套卡 5 = 15 |
| G4 | 缺纯 CPU 合成记录往返单测 | `src/render/kernel/diag.rs` 新增 64 f32 编码→解码往返单测（face/mesh_id 位型、`view_dir` 精确负号派生、保留槽 0、长度校验），零 GPU |
| 低优先 | ABI 棘轮文字陈旧（7 标量、`is_unchanged`）、PTX 字节口径、`nlight>=4` 灯槽冲突 | 三处测试注释/名称更正（82 槽 = 上游 81 + 尾随诊断槽、8 标量）；§4 补 NUL 计长口径；§2 记 `nlight>=4` 槽冲突 backlog |

- **dump-ext 残留更正（2026-11-11，QA 复验低项）**：§2/receipt `json_location` 原写「未启用 env 时不存在该字段」，与新契约「键始终存在、未采集为 null」矛盾，已更正（§2 与 receipt 同步）；提交信息 `8b37670f` 的「配套卡 +86/−8」实为 **+84/−8**（numstat 2/2 + 3/3 + 1/1 + 2/2 + 76/0），历史说明不改写，记录于 errata 批 receipt 与本卡 addenda。
- **500 行棘轮处置**：B1/G4 修复使 `tests/support/render_bvh/light/diag.rs` 由 496 行增至 572 行而触达 500 行上限，故按「按功能抽出子模块」把 near-miss 统计（`near_miss` + `distance_owner`，共 104 行）**原样搬迁**到 `tests/support/render_bvh/light/diag_ext.rs`（纯搬迁、行为不变）：修复后 `diag.rs` **466** 行、`diag_ext.rs` **186** 行（均 ≤500）。
- 修复后内核身份不变（本卡 `unchanged src/render/kernel/cuda/`，未改任何 `.cu`）：megakernel source `624ab298…`、PTX `1fd53131…` / 347,070；aa_resolve `0bf8af5a…` / `b28e155c…` / 27,536；depth_seg `b546ef08…` / `e8d84963…` / 138,540。
- 修复卡只改测试/注释/本目录文档，不改渲染算术与判据；G-L2 判定维持未通过（判据 A FAIL）。
