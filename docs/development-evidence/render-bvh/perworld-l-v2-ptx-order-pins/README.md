# perworld L v2 残余 4 px：P2/P3/P4 PTX 归一化次序 pin（修复卡证据；零 GPU）

- 任务：`2026-11-11-render-perworld-l-v2-ptx-order-pins`（owner Backend；p80；P8-渲染与BVH）。
- 依据：Arch 离线 PTX 对齐证据 `target/render-bvh/gl2-ptx-diff/`（README 附录 A/B/C + `spec_sensitivity.json`；地面真值=采集时 warp 缓存模块/PTX；零 GPU）。量化结论：P2（half-vector 归一化次序）是唯一能解释 4 px 量级的站点（预测 2–11，实测 4）；P3 为指令对齐必要项（≈0）；P4 次要（≈0.2–1）。
- 改动全在 `.cu`：**不改判据/参考/容差/相机/recipe/tests/fixtures/scripts**；零 GPU、不采集、不跑诊断；未动冻结件/guards。
- 非声明：本卡不宣称修复生效（需 Chief 唯一一轮诊断轮验证 4 px 归零）；不改变 G-L2 判定（判据 A FAIL 维持）；内核身份变化属实现修复新基线。

## 1. 改动（4 处源码 + 1 个 helper；diff `logs/src.diff`）

### 1.1 helper（`render_common.cu`，新增；不动既有 `normalize3_rays`）

```cpp
// 上游 `vec.h:1113-1115` 的 normalize：平方和中**中间项单独舍入**（mul y*y → fma x*x → fma z*z）。
__device__ __forceinline__ V3 normalize3_midpref(V3 a) {
    float l = sqrtf(fmaf(a.z, a.z, fmaf(a.x, a.x, a.y * a.y)));
    if (l > 0.0f) {
        return v3(a.x / l, a.y / l, a.z / l);
    }
    return v3(0.0f, 0.0f, 0.0f);
}
```

`normalize3_rays`（S5 已验路径）字面未动；helper 定义位于 `render_common.cu`（拼接顺序在 `render_shade.cu`/`render_megakernel.cu` 之前），下游可见。

### 1.2 P2（主嫌疑，量化影响最大）：`render_shade.cu`

```diff
-            V3 half = normalize3_rays(add(L, view_dir));
+            // P2：对齐参考核 half-vector 归一化的中间项单独舍入次序（量化影响最大站点）。
+            V3 half = normalize3_midpref(add(L, view_dir));
```

### 1.3 P3（指令对齐必要项）：`render_megakernel.cu`

```diff
-        normal = normalize3_rays(mat_vec(geom_mat, vn));
+        // P3：mesh 顶点法线归一化对齐参考核的中间项单独舍入次序。
+        normal = normalize3_midpref(mat_vec(geom_mat, vn));
```

### 1.4 P4（次要候选）：`render_shade.cu` 点/spot 灯方向

```diff
-            L = scale(delta, 1.0f / norm);
+            // P4：对齐上游 `normalize_with_norm` 的三次除法（原为 `scale(delta, 1.0f / norm)` 倒数乘）。
+            L = v3(delta.x / norm, delta.y / norm, delta.z / norm);
```

### 1.5 文件 sha/bytes（old → new）

| 文件 | old sha256 / bytes | new sha256 / bytes |
| --- | --- | --- |
| `render_common.cu` | `9625d684…5ceacb00` / 20,974 | `887c3000…c436ba9a9` / 21,495 |
| `render_shade.cu` | `f8394d73…9d587046e` / 20,485 | `69d3a2e2…4247299` / 20,758 |
| `render_megakernel.cu` | `6d718100…5d75675` / 21,167 | `c3936f86…a3b3d32` / 21,261 |

改动统计：3 个 `.cu`，**+37 / −3**（含注释）；`render_math.cu`、`render.cu`、`render_shadow.cu`、`render_flex.cu` 等零改动。

## 2. 内核身份新基线（`logs/kernel-identity-old-new.txt`；NVRTC 12.8.93 纯编译，无 GPU 会话）

| artifact | 值 | old | new |
| --- | --- | --- | --- |
| megakernel | source sha256 | `4d932c90…225bf6080` | **`d3e04aa7…b0eec4f7`** |
| megakernel | PTX sha256 / bytes | `13ac8a2e…77eeb503d` / 344,186 | **`98f09afc…cf1f9e6c228`** / **344,166** |
| aa_resolve | source sha256 | `0bf8af5a…e104bc76` | 未变 |
| aa_resolve | PTX sha256 / bytes | `b28e155c…366ec8a70` / 27,536 | 未变 |
| depth_seg | source sha256 | `b546ef08…114d931` | 未变（P2/P3/P4 文件不在 depth_seg 拼接中） |
| depth_seg | PTX sha256 / bytes | `e8d84963…4e5c7` / 138,540 | 未变 |

- 采集方式（NVRTC 纯编译 + f32-render 门禁，**不建 GPU 会话、不跑渲染**）：
  `cargo test --locked --features cuda-probe --test render_kernel_flags flag_and_aa_sources_compile_under_render_gate -- --ignored --nocapture`
  `cargo test --locked --features cuda-probe --test render_kernel_depth_seg depth_seg_source_compiles_under_render_gate -- --ignored --nocapture`
- 独立复算：按 `mod.rs` 的 `include_str!` 顺序对 old（HEAD blobs）/new（工作树）做字节拼接 sha256，均与测试打印值逐位一致（`logs/old-source-recompute.txt`、`logs/new-source-recompute.txt`）。
- 注释差异说明：old/new 使用**同一 NVRTC 12.8.93 与同一编译选项**，PTX 差异来自本次源码 pin（megakernel 344,186 → **344,166 B**，指令级变化），**不涉及**跨版本 `.version 8.7→8.8` 或 build-id 注释差异；跨版本「仅注释差异」的结论见 Arch 证据 `gl2-ptx-diff/README.md` §2（12.8.93 vs 12.9.86 指令 0 差异）。
- 属实现修复新基线，不改判据/阈值/参考。

## 3. 构建与静态验证（零 GPU）

| 项 | 结果 |
| --- | --- |
| `cargo build --locked --features cuda-probe` | exit 0（`logs/build-cuda-probe.log`） |
| `cargo test --locked --features cuda-probe --no-run` | exit 0（`logs/no-run-cuda-probe.log`） |
| 四门禁 fmt / clippy / clippy-cuda / unit | 全过；unit **1,387 过 / 0 败 / 402 ignored** |
| 内核身份用例（NVRTC 纯编译） | 2 个用例各 1 passed / 0 failed |

## 4. 边界与非声明

- 边界：`unchanged fixtures/`、`unchanged tests/`、`unchanged scripts/`；其它证据目录只读（G-L2 窗口证据、诊断轮证据、Arch RCA 证据、mesh-normal 修复证据、criterion-spec 目录）；本卡只改 3 个 `.cu` 与新增证据目录；零 GPU、不采集、不跑诊断、未改 guards/冻结 stash、未重编译冻结 exe 证据。
- 非声明：不宣称修复生效（需 Chief 唯一一轮诊断轮 token：重建实体 exe 跑 A/B/C + dump，验 4 px 是否归零；若仍有残余用 dump 离线收敛）；不改 G-L2 判定（判据 A FAIL 维持）；不改判据/阈值/参考/容差；诊断轮/重冻/G-L2 attempt 3 由 Chief 另发 token。
