# perworld L v2 残余 3 px：R1+P5+P6 PTX 次序 pin 修正（修复卡-2 证据；零 GPU）

- 任务：`2026-11-11-render-perworld-l-v2-ptx-order-pins-2`（owner Backend；p80；P8-渲染与BVH）。
- 依据：Arch 诊断轮 #3 后的站点级修正 —— `target/render-bvh/gl2-ptx-diff/README.md` 附录 D + 离线补丁 `product_megakernel_patched2.cu`（地面真值=采集时 warp 缓存模块/PTX；零 GPU）。诊断轮 #3 结果：残余 4 px → **3 px**，world0 cam1 归零，剩余 3 px 全在 world1 cam1（`10744` G−1、`22903` B−1、`39492` G+1），depth/seg 全等、无新增。
- 改动全在 `.cu`：**不改判据/参考/容差/相机/recipe/tests/fixtures/scripts**；零 GPU、不采集、不跑诊断；未动冻结件/guards。
- 非声明：本卡不宣称修复生效（需 Chief 诊断轮 #4 token 验证 3 px 是否归零）；不改变 G-L2 判定（判据 A FAIL 维持）；内核身份变化属实现修复新基线。

## 1. 改动（2 个 helper + 3 处调用点；diff `logs/src.diff`）

### 1.1 helpers（`render_common.cu`，新增 2 个；既有 helper 原样保留）

```cpp
// 上游 `wp.length(vec3)`：平方和中**中间项单独舍入**（mul dy*dy → fma dx*dx → fma dz*dz）。
// G-L2 残余像素的 PTX 次序 pin（P6）：仅用于点/spot 灯 L 的模；`scene.cu::length3` 其它调用方共用，保持不动。
__device__ __forceinline__ float length3_midpref(V3 a) {
    return sqrtf(__fmaf_rn(a.z, a.z, __fmaf_rn(a.x, a.x, __fmul_rn(a.y, a.y))));
}

// 上游 `wp.dot(normal, L)`（ndotl）的收缩次序是**首项单独舍入**：
// mul(nx*Lx) → fma(ny*Ly) → fma(nz*Lz)。G-L2 残余像素的 PTX 次序 pin（P5）：仅用于 ndotl；
// `dot3_ba` 的中间项单独舍入次序保持不变。
__device__ __forceinline__ float dot3_first(float a0, float a1, float a2, float b0, float b1,
                                            float b2) {
    return __fmaf_rn(a2, b2, __fmaf_rn(a1, b1, __fmul_rn(a0, b0)));
}
```

`normalize3_rays`（S5 已验路径）字面未动；上一卡的 `normalize3_midpref` 保留（P3 仍用）。helper 定义位于 `render_common.cu`（拼接顺序在 `render_shade.cu`/`render_megakernel.cu` 之前），下游可见。

### 1.2 R1（撤销上一卡 P2）：`render_shade.cu` half-vector

```diff
-            // P2：对齐参考核 half-vector 归一化的中间项单独舍入次序（量化影响最大站点）。
-            V3 half = normalize3_midpref(add(L, view_dir));
+            // R1：参考核 half-vector 归一化是**首项**单独舍入（Arch 附录 D），保持 `normalize3_rays`。
+            V3 half = normalize3_rays(add(L, view_dir));
```

依据：Arch 用地面真值 PTX 逐站点读出 half 归一化是 `mul(Hx²) → fma(Hy²) → fma(Hz²)`（首项单独舍入），P2 方向反了；诊断轮 #3 的 3 px 产品字节与 dump2 完全相同 ⇒ P2 未改变它们。

### 1.3 P5（新）：`render_shade.cu` ndotl

```diff
-    float ndotl = fmaxf(0.0f, dot3_ba(normal.x, normal.y, normal.z, L.x, L.y, L.z));
+    // P5：对齐上游 `wp.dot(normal, L)` 的首项单独舍入次序（原为 `dot3_ba` 的中间项）。
+    float ndotl = fmaxf(0.0f, dot3_first(normal.x, normal.y, normal.z, L.x, L.y, L.z));
```

依据：参考 `wp.dot(normal, L)`（ndotl）是 `mul(nx·Lx) → fma(ny·Ly) → fma(nz·Lz)`（首项单独舍入）；产品原用 `dot3_ba`（中间项）。预期翻转量级 ≈2.5 px。

### 1.4 P6（新）：`render_shade.cu` 点/spot 灯 L 的模

```diff
-        float len = length3(delta);
+        // P6：对齐上游 `wp.length` 的中间项单独舍入次序（原为 `length3`，未 pin）。
+        float len = length3_midpref(delta);
```

依据：参考 `wp.length(delta)` 是 `mul(dy²) → fma(dx²) → fma(dz²)`（中间项单独舍入）；`scene.cu::length3` 其它调用方共用，保持不动。预期翻转量级 ≈0.5 px。

### 1.5 保留/不动

- **P3 保留**：`render_megakernel.cu:274` `normal = normalize3_midpref(mat_vec(geom_mat, vn));`（上一卡改动，本卡未动该文件）。
- **P4 保留**：`render_shade.cu` 点/spot 灯 L 的逐分量三次除法（上一卡改动，本卡未动）。
- 不动：`normalize3_rays`、`scene.cu::length3`、`dot3_ba`、`cos_theta`（`render_shade.cu:423`，README 附录 D 明确留作后备候选）。

### 1.6 文件 sha/bytes（old → new）

| 文件 | old sha256 / bytes | new sha256 / bytes |
| --- | --- | --- |
| `render_common.cu` | `887c300075007180d2b1e350292331d626e267b3e7fe580d4fadc99c436ba9a9` / 21,495 | `9116ce26ae60542ee8f89de9067528cda47a28b693d27a67923135827e4fd434` / 22,348 |
| `render_shade.cu` | `69d3a2e26f3a875d7dfe4065b479319557cb961806298ae555174d3f34247299` / 20,758 | `27dfcfd36e1f69ce924eef5459bf6ee9eec8053d9fdee02c5b878bf3f4eee91f` / 20,989 |

改动统计（实际 `git diff --numstat`）：2 个 `.cu`，**+21 / −5**（`render_common.cu` +15/−1、`render_shade.cu` +6/−4，含注释）；`render_megakernel.cu`、`render_math.cu`、`render.cu`、`render_shadow.cu`、`render_flex.cu`、`aa_resolve.cu` 等**零改动**。

## 2. 内核身份新基线（`logs/kernel-identity-old-new.txt`；NVRTC 12.8.93 纯编译，无 GPU 会话）

| artifact | 值 | old | new |
| --- | --- | --- | --- |
| megakernel | source sha256 | `d3e04aa7…b0eec4f7` | **`92771543…effec30f`** |
| megakernel | PTX sha256 / bytes | `98f09afc…cf1f9e6c228` / 344,166 | **`e44ac85c…2bdae8d2`** / **344,169** |
| aa_resolve | source sha256 | `0bf8af5a…e104bc76` | 未变 |
| aa_resolve | PTX sha256 / bytes | `b28e155c…366ec8a70` / 27,536 | 未变 |
| depth_seg | source sha256 | `b546ef08…114d931` | 未变 |
| depth_seg | PTX sha256 / bytes | `e8d84963…4e5c7` / 138,540 | 未变 |

- 采集方式（NVRTC 纯编译 + f32-render 门禁，**不建 GPU 会话、不跑渲染**）：
  `cargo test --locked --features cuda-probe --test render_kernel_flags flag_and_aa_sources_compile_under_render_gate -- --ignored --nocapture`
  `cargo test --locked --features cuda-probe --test render_kernel_depth_seg depth_seg_source_compiles_under_render_gate -- --ignored --nocapture`
- 独立复算：按 `mod.rs` 的 `include_str!` 顺序对 old（HEAD blobs）/new（工作树）做字节拼接 sha256，均与测试打印值逐位一致（`logs/old-source-recompute.txt`、`logs/new-source-recompute.txt`）。
- 注释差异说明：old/new 使用**同一 NVRTC 12.8.93 与同一编译选项**，PTX 差异来自本次源码 pin（megakernel 344,166 → **344,169 B**，指令级变化），**不涉及**跨版本 `.version 8.7→8.8` 或 build-id 注释差异；跨版本「仅注释差异」的结论见 Arch 证据 `gl2-ptx-diff/README.md` §2。
- 属实现修复新基线，不改判据/阈值/参考。

## 3. 构建与静态验证（零 GPU）

| 项 | 结果 |
| --- | --- |
| `cargo build --locked --features cuda-probe` | exit 0（`logs/build-cuda-probe.log`） |
| `cargo test --locked --features cuda-probe --no-run` | exit 0（`logs/no-run-cuda-probe.log`） |
| 四门禁 fmt / clippy / clippy-cuda / unit | 全过；unit **1,387 过 / 0 败 / 402 ignored** |
| 内核身份用例（NVRTC 纯编译） | 2 个用例各 1 passed / 0 failed |

## 4. 边界与非声明

- 边界：`unchanged fixtures/`、`unchanged tests/`、`unchanged scripts/`；其它证据目录只读（上一卡 pin 证据、G-L2 窗口证据、诊断轮证据、Arch RCA 证据、mesh-normal 修复证据）；本卡只改 2 个 `.cu` 与新增证据目录；零 GPU、不采集、不跑诊断、未改 guards/冻结 stash、未重编译冻结 exe 证据。
- 非声明：不宣称修复生效（需 Chief 诊断轮 #4 token：重建实体 exe 跑 A/B/C + dump，验 3 px 是否归零；若仍有残余用 dump 离线收敛）；不改 G-L2 判定（判据 A FAIL 维持）；不改判据/阈值/参考/容差；诊断轮/重冻/G-L2 attempt 3 由 Chief 另发 token。
