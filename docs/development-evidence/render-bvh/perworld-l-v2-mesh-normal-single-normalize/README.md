# perworld L v2 mesh 法线链单次归一化对齐上游（修复卡证据；零 GPU）

- 任务：`2026-11-11-render-perworld-l-v2-mesh-normal-single-normalize`（owner Backend；p75；P8-渲染与BVH）。
- 依据：Arch 只读 RCA `2026-11-11-render-perworld-l-v2-gl2-criterion-a-rca`（H2 源码级、唯一站点）：上游 mesh 法线链 = `cross(ab,ac)`（未归一化）→ `mat@n` → `normalize` 一次；我方 `render_math.cu:239` 先把 cross 归一化、`render_common.cu:200` 再对 `mat@n` 归一化 → **多归一化一次**。
- 修复：`src/render/kernel/cuda/render_math.cu` 的 `mesh_query_ray` 返回**未归一化**三角面 `cross`；消费端 `render_common.cu` / `render.cu` 保留各自唯一一次 `normalize3_mesh(mat_vec(mat, n_local))`。本卡只改代码与证据，**未跑 GPU、未采集、未重编译冻结 exe 证据**。
- 非声明：本卡不宣称修复已生效（需后续诊断轮验证）；不改变 G-L2 判定（判据 A FAIL 维持）；不改任何判据/阈值/参考/相机/recipe；内核身份变化属实现修复的新基线。

## 1. 改动（唯一代码文件；最小面）

`src/render/kernel/cuda/render_math.cu`（`mesh_query_ray`，行 239 附近）：

```diff
@@ -236,7 +236,9 @@ __device__ __forceinline__ bool mesh_query_ray(V3 pnt, V3 vec, float max_t, int
     *out_u = min_u;
     *out_v = min_v;
     *out_sign = min_sign;
-    *out_normal = normalize3_mesh(min_normal);
+    // 上游 mesh 法线链（G-L2 判据 A RCA H2）：cross(ab,ac) 未归一化 → 消费端 mat@n → normalize 一次。
+    // 本函数返回未归一化的三角面法线；唯一一次归一化由 render_common.cu / render.cu 的消费端完成。
+    *out_normal = min_normal;
     *out_face = min_face;
     return true;
 }
```

- `render_math.cu` 文件 sha256：`2c39c481b4a3f392d852f62e2adc1c6153d46c1fb7d68d405c85c2ac2a170388` / 12,325 B → `4f103495f4cc7c9a5da53f4b18e612224a1302a77092a0704bdf4fabca8780a0` / 12,566 B。
- 实际 diff：**-1 行 / +3 行**（1 行代码去掉归一化 + 2 行口径注释）；`render_common.cu`、`render.cu`、`render_shadow.cu` 零改动。
- `render_math.cu` 的 `intersect_tri` 本就返回未归一化 `cross`（`*normal = cross(ab, ac);`），未改；`mesh_query_ray` 除 `*out_normal` 外的所有输出（`t/u/v/sign/face`）零改动。

## 2. 调用链盘点（`logs/call-chain-audit.txt`；逐处确认无二次依赖）

- `mesh_query_ray` 全仓仅 2 个调用方：
  1. `render_common.cu::ray_mesh_leaf`（产品 megakernel 路径）：背面剔除 `dot(lvec, n_local) < 0` **仅用符号**（不归一化不改变符号）；命中后 `*normal = normalize3_mesh(mat_vec(mat, n_local));` 是**唯一一次**归一化（mat 变换后），与上游链路一致。
  2. `render.cu::ray_mesh_leaf`（depth/seg 内核路径）：同样仅用符号剔除，命中后 `render.cu:231` 唯一一次归一化。
- shadow：`render_shadow.cu` 的 `shadow_origin = hitpoint + normal * 1e-4` 使用的是消费端归一化后的 `hit.normal`，语义与修复前一致（仅 f32 舍入路径对齐上游）。
- hfield（gtype=1）/ SDF（gtype=8）：`render_common.cu:417` 三者（7/1/8）同走 `ray_mesh_leaf` → 同样「mat 后归一化一次」；阴影 any-hit 对 mesh/hfield/SDF 不参与背面剔除的既有语义不变；SDF 设备求交尚未解锁（host 拒绝），该分支不可达，不受影响。
- 求交/深度/seg：normal 只在命中后用于着色与阴影偏移，不参与 `d`（t）、`u/v/sign`、face 的数值；本修复不改变这些输出。
- `normalize3_mesh` 定义仍在 `render_math.cu`，由后续拼接的 `render_common.cu` / `render.cu` 消费（编译单元内顺序：`render_math.cu` 在前），无「函数未使用」问题。

## 3. 内核身份新基线（`logs/kernel-identity-old-new.txt`；NVRTC 纯编译，无 GPU 会话）

| artifact | 值 | old | new |
| --- | --- | --- | --- |
| megakernel | source sha256 | `796bade6…a6fc962` | **`4d932c90…225bf6080`** |
| megakernel | PTX sha256 / bytes | `aa51258b…726e9e256` / 345,514 | **`13ac8a2e…77eeb503d`** / **344,186** |
| aa_resolve | source sha256 | `0bf8af5a…e104bc76` | `0bf8af5a…e104bc76`（未变） |
| aa_resolve | PTX sha256 / bytes | `b28e155c…366ec8a70` / 27,536 | `b28e155c…366ec8a70` / 27,536（未变） |
| depth_seg | source sha256 | `562113a2…3f9b4a9` | **`b546ef08…114d931`** |
| depth_seg | PTX sha256 / bytes | `d1e3e576…12590748` / 138,857 | **`e8d84963…4e5c7`** / **138,540** |

- 采集方式（仅 NVRTC 编译 + f32-render 门禁，**不建 GPU 会话、不跑渲染**）：
  `cargo test --locked --features cuda-probe --test render_kernel_flags flag_and_aa_sources_compile_under_render_gate -- --ignored --nocapture`
  `cargo test --locked --features cuda-probe --test render_kernel_depth_seg depth_seg_source_compiles_under_render_gate -- --ignored --nocapture`
- 独立复算：按 `src/render/kernel/mod.rs` 的 `include_str!` 顺序对 LF 工作树做字节拼接 sha256，old/new 均与测试打印值逐位一致（`logs/old-source-recompute.txt`、`logs/new-source-recompute.txt`）。
- 说明：改 `.cu` → `MEGAKERNEL_SOURCE`/`DEPTH_SEG_SOURCE` 与两份 PTX 必变（aa_resolve 不含 `render_math.cu`，未变）；这是**实现修复的新基线**，不是判据/阈值/参考变化。

**PTX bytes 变化说明（供复核）**：megakernel 345,514 → 344,186（-1,328 B）、depth_seg 138,857 → 138,540（-317 B）——去掉一次调用链上的 `normalize3_mesh` 内联后 PTX 收缩；aa_resolve 不变。

## 4. 构建与静态验证（零 GPU）

| 项 | 结果 |
| --- | --- |
| `cargo build --locked --features cuda-probe` | exit 0（`logs/build-cuda-probe.log`） |
| `cargo test --locked --features cuda-probe --no-run` | exit 0，全部 test target 编译（`logs/no-run-cuda-probe.log`） |
| 四门禁 fmt / clippy / clippy-cuda / unit | 全过；**unit 1,387 过 / 0 败**（`normify gate` 结果） |
| 内核身份用例（NVRTC 纯编译） | 2 个用例各 1 passed / 0 failed（见 §3 日志） |

## 5. 边界与非声明

- 边界：`unchanged fixtures/`、`unchanged scripts/`、`unchanged tests/`；其它证据目录只读（G-L2 窗口证据、诊断轮证据、Arch RCA 证据、criterion-spec 目录）；本卡只改 `render_math.cu` 与新增证据目录；**零 GPU、未采集、未重编译冻结 exe 证据、未改 guards/冻结 stash**。
- 非声明：本卡不宣称修复生效（需 Chief 诊断轮 token：重建实体 exe 跑一次，验 5 px 是否归零、A/B/sha/隔离）；不改变 G-L2 判定（判据 A FAIL 维持）；不改判据/阈值/参考/相机/recipe；诊断轮、重冻与 G-L2 attempt 3 均由 Chief 另发 token。

## 6. Addenda（2026-11-11，P10 errata）

> **Addendum（2026-11-11，P10 errata）**：本卡原记「上游 mesh 法线链 = `cross` 未归一化 → `mat@n` → 归一化一次」**前提有误**。经参考侧 gated dump 与产品侧 64 字段 dump 逐站比对（`target/render-bvh/gl2-ptx-diff/ref-vs-product-station-diff.md`）与离线重放：上游用 warp 内建 `mesh_query_ray`，其在 `warp/native/mesh.h:1880` 返回前已做**局部归一化**，`mujoco_warp/_src/render.py:731-732` 再 `mat @ n` + `wp.normalize`，即上游为**两次**归一化；产品在 `f728b2d7` 删掉了第一次归一化，残余 4 px 由此而来，P10（`b0e372be`）已恢复（`render_math.cu` `normalize3_mesh(min_normal)` + `mat_vec_xpref` x-项先舍入）。原记录文字保留不改。

- 本卡**结果**（把 5 px 降到 4 px、数值向参考方向移动）仍为实测事实；仅**归因文字**更正。
- 后续落点：P10 修复卡 `perworld-l-v2-mesh-normal-chain-pin/`（提交 `b0e372be`，恢复局部归一化 + `mat_vec_xpref`）；逐站比对 `target/render-bvh/gl2-ptx-diff/ref-vs-product-station-diff.md` §4；W0 收口 brief `target/render-bvh/gl2-ptx-diff/w0-closure-brief.md`。
