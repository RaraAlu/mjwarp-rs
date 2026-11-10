# perworld L v2 残余像素 P7：mesh 重心 texcoord 插值单独舍入项 pin（修复卡证据；零 GPU）

- 任务：`2026-11-11-render-perworld-l-v2-texture-bary-pin`（owner Backend；p85；P8-渲染与BVH）。
- 依据：Arch 零 GPU 站点级定位 `target/render-bvh/gl2-ptx-diff/texture-chain-and-dump-spec.md` §A（参考真值 PTX 指令 7900-7936 / `reference_li_128.ptx`；地面真值=采集时 warp 缓存模块）。参考序列为 `mul(uv1*bary_v) → fma(uv0*bary_u) → fma(uv2*w)`，**单独舍入项是中间项 `uv1*w1`**；产品原为 `fmaf(uv0,w0, fmaf(uv1,w1, uv2*w2))`，**单独舍入第三项 `uv2*w2`** ⇒ texcoord u/v 差 1 ULP，只命中 textured mesh 路径（残余像素全部所在的 `geom3 tri-mesh / mat 0 grid`）。
- 改动只涉及 `render_shade.cu::texture_mesh` 的 has_uv 分支两行；**不改判据/参考/容差/相机/recipe**；零 GPU、不采集、不跑诊断；未动冻结件/guards。
- 非声明：本卡不宣称修复生效（需 Chief 诊断轮 #5 token 验证残余像素是否归零）；不改变 G-L2 判定（判据 A FAIL 维持）；内核身份变化属实现修复新基线。

## 1. 改动（1 个文件，2 行 + 2 行注释；diff `logs/src.diff`）

### 1.1 变量与参考寄存器映射核对（spec §A）

| 仓库变量（`texture_mesh`） | 参考 PTX 寄存器（spec §A） | 说明 |
| --- | --- | --- |
| `w0 = bary_u` | `%f5057`（新 PTX，来自 `1.0-w0-w1` 链的 sub 操作数） | 重心权重 u，**字面不动** |
| `w1 = bary_v` | `%f5058` | 重心权重 v，**字面不动** |
| `w2 = 1.0f - bary_u - bary_v` | `%f3069 = (1.0-%f5057)-%f5058` | 与参考 `(1-u)-v` 一致，**字面不动** |
| `uv0`（`coords[0]` 的 texcoord）/`.x` | `%f3072`（新 PTX）/`.y` `%f3077` | 逐行核对于 `logs/new-lineinfo-window.txt` |
| `uv1`（`coords[1]`）/`.x` | `%f3070` /`.y` `%f3075` | 同上 |
| `uv2`（`coords[2]`）/`.x | `%f3074` /`.y` `%f3079` | 同上 |

映射结论：`uv0↔bary_u`、`uv1↔bary_v`、`uv2↔(1-u)-v`，与 spec §A 逐项一致；无疑点。

### 1.2 代码改动（`render_shade.cu`，has_uv 分支）

```diff
-            float u = fmaf(uv0.x, w0, fmaf(uv1.x, w1, uv2.x * w2));
-            float v = fmaf(uv0.y, w0, fmaf(uv1.y, w1, uv2.y * w2));
+            // P7：对齐参考核 `vec.h` 的 mul→fma→fma 收缩——单独舍入项是**中间项** `uv1*w1`；
+            // 产品原为 `fmaf(uv0,w0, fmaf(uv1,w1, uv2*w2))`（单独舍入第三项 `uv2*w2`），会差 1 ULP。
+            float u = __fmaf_rn(uv2.x, w2, __fmaf_rn(uv0.x, w0, __fmul_rn(uv1.x, w1)));
+            float v = __fmaf_rn(uv2.y, w2, __fmaf_rn(uv0.y, w0, __fmul_rn(uv1.y, w1)));
```

`w0`/`w1`/`w2` 字面不动；`texture_hfield`/fallback 分支/`sample_texture`/纹理采样/法线/灯链、`render_common`/`render_megakernel`/`render_math`/`scene.cu` 全部零改动；`render_shade.cu` 行数 462→464（≤500）。

### 1.3 文件 sha/bytes（old → new）

| 文件 | old sha256 / bytes | new sha256 / bytes |
| --- | --- | --- |
| `render_shade.cu` | `27dfcfd36e1f69ce924eef5459bf6ee9eec8053d9fdee02c5b878bf3f4eee91f` / 20989 | `bdf6c52778732b83a6f721900123565f2551e27380c702968a2bec63e07c3f27` / 21268 |

改动统计（实际 `git diff --numstat`）：1 个文件，**+4 / −2**（2 行代码 ×2 + 2 行注释）。

## 2. 内核身份新基线（`logs/kernel-identity-old-new.txt`；NVRTC 12.8.93 纯编译，无 GPU 会话）

| artifact | 值 | old | new |
| --- | --- | --- | --- |
| megakernel | source sha256 | `92771543…effec30f` | **`eb59dd78…bb051ee`** |
| megakernel | PTX sha256 / bytes | `e44ac85c…bdae8d2` / 344,169 | **`e91b7d85…0e17fac`** / **344,175** |
| aa_resolve | 三值 | `0bf8af5a…` / `b28e155c…` / 27,536 | 未变 |
| depth_seg | 三值 | `b546ef08…` / `e8d84963…` / 138,540 | 未变（`render_shade.cu` 不在 DEPTH_SEG_SOURCE 拼接中） |

- 采集/复算：两个 identity 用例各 1 passed；按 `mod.rs` `include_str!` 顺序独立拼接复算均与打印值逐位一致（`logs/old-source-recompute.txt`、`logs/new-source-recompute.txt`）。
- 注释说明：old/new 同 NVRTC 12.8.93 同选项，PTX 差异来自本次源码 pin（megakernel 344,169 → **344,175 B**），不涉及跨版本 `.version`/build-id 注释差异。

## 3. 离线 NVRTC PTX 证明（零 GPU；`logs/p7-site-proof.txt`）

用离线 NVRTC harness（ctypes，无 CUDA 上下文/GPU 会话）按 product 选项 + `--generate-line-info` 编译按 `mod.rs` 顺序拼接的产品源码：

| 项 | old（HEAD） | new（worktree） |
| --- | --- | --- |
| stitched source bytes/sha256 | 104,103 / `92771543…` | 104,382 / `eb59dd78…`（== identity 打印值） |
| lineinfo PTX bytes/sha256 | 580,289 / `6cec0f62…` | 580,295 / `cf619209…` |

站点指令序列（`texture_mesh`，u 链；v 链同形）：

```text
old: mul.f32    %f3071, %f3069(=w2), %f3070(=uv2.x)      ; 单独舍入 uv2*w2
     fma.rn.f32 %f3073, %f3072(=uv1.x), %f5058(=w1), %f3071
     fma.rn.f32 %f5100, %f3074(=uv0.x), %f5057(=w0), %f3073

new: mul.rn.f32 %f3071, %f3070(=uv1.x), %f5058(=w1)      ; 单独舍入 uv1*w1（中间项）
     fma.rn.f32 %f3073, %f3072(=uv0.x), %f5057(=w0), %f3071
     fma.rn.f32 %f5100, %f3074(=uv2.x), %f3069(=w2), %f3073
```

⇒ **new 站点 = `mul(uv1*w1) → fma(uv0*w0) → fma(uv2*w2)`，与参考真值序列（spec §A：`mul(uv1*bary_v) → fma(uv0*bary_u) → fma(uv2*w)`）逐指令同形**；old 仅单独舍入项不同（第三项 vs 中间项）。

## 4. 构建与静态验证（零 GPU）

| 项 | 结果 |
| --- | --- |
| `cargo build --locked --features cuda-probe` | exit 0（`logs/build-cuda-probe.log`） |
| `cargo test --locked --features cuda-probe --no-run` | exit 0（`logs/no-run-cuda-probe.log`） |
| 四门禁 fmt / clippy / clippy-cuda / unit | 全过；unit **1,387 过 / 0 败 / 402 ignored** |
| 内核身份用例（NVRTC 纯编译） | 2 个用例各 1 passed / 0 failed |
| 离线 PTX 同形证明 | PASS（`logs/p7-site-proof.txt`，断言 old/new 顺序与寄存器映射） |

## 5. 边界与非声明

- 边界：只改 `src/render/kernel/cuda/render_shade.cu` 两行 + 注释与新增本证据目录；`unchanged fixtures/`、`unchanged tests/`、`unchanged scripts/`、其余 `.cu` 文件与既有证据目录只读；零 GPU、不采集、不跑诊断、未改 guards/冻结 stash、未重编译冻结 exe 证据。
- 非声明：不宣称修复生效（需 Chief 诊断轮 #5 token 验证）；不改 G-L2 判定（判据 A FAIL 维持）；不改判据/阈值/参考/容差；诊断轮/重冻/G-L2 attempt 3 由 Chief 另发 token。
