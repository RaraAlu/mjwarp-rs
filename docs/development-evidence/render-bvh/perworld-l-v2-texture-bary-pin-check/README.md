# QA 复核：P7 纹理重心插值舍入次序 pin（`4e6e47b1`）

- 复核卡：`2026-11-11-render-perworld-l-v2-texture-bary-pin-check`（QA）
- 被复核：`2026-11-11-render-perworld-l-v2-texture-bary-pin`（Backend；登记 `ee01764f` / start `fb0e2872` / 代码 `4e6e47b1`）
- 性质：只读复核，零 GPU；**未重新编译**（避免扰动诊断轮 #5 前的 live exe 与冻结件口径）；只写本卡与本卡证据。
- 结论：**PASS（0 阻断，4 条信息级观察）→ 可发诊断轮 #5 token**。本复核只核代码/证据/边界/口径，**不判定修复效果**；G-L2 判定维持未通过。

## 1. 改动面（逐行 diff）

`4e6e47b1` 的源码改动**只有** `src/render/kernel/cuda/render_shade.cu` **+4/−2**（单 hunk）：

```diff
             float w0 = bary_u;
             float w1 = bary_v;
             float w2 = 1.0f - bary_u - bary_v;
-            float u = fmaf(uv0.x, w0, fmaf(uv1.x, w1, uv2.x * w2));
-            float v = fmaf(uv0.y, w0, fmaf(uv1.y, w1, uv2.y * w2));
+            // P7：对齐参考核 `vec.h` 的 mul→fma→fma 收缩——单独舍入项是**中间项** `uv1*w1`；…
+            // （2 行注释）
+            float u = __fmaf_rn(uv2.x, w2, __fmaf_rn(uv0.x, w0, __fmul_rn(uv1.x, w1)));
+            float v = __fmaf_rn(uv2.y, w2, __fmaf_rn(uv0.y, w0, __fmul_rn(uv1.y, w1)));
```

- `w0 = bary_u` / `w1 = bary_v` / `w2 = 1.0f - bary_u - bary_v` 三行**字面未动**（diff 中为上下文）；`uv0/uv1/uv2` 取值行未动。
- 其它 `.cu`（`render_common.cu`/`render_math.cu`/`render_megakernel.cu`/`render.cu`/`render_shadow.cu`/`scene.cu`）**0 diff**。
- 文件行数 **462 → 464**（blob 字节/LF 独立核实）。

## 2. 落地式 vs spec §A

`target/render-bvh/gl2-ptx-diff/texture-chain-and-dump-spec.md` §A 参考真值（PTX 7900-7936）：
`mul(uv1*bary_v) → fma(uv0*bary_u) → fma(uv2*w)`，**单独舍入项 = 中间项 `uv1*w1`**。

实现：`__fmul_rn(uv1,w1)` 单独舍入 → `__fmaf_rn(uv0,w0,·)` → `__fmaf_rn(uv2,w2,·)` ⇒ **逐项同形** ✅；原产品单独舍入第三项 `uv2*w2`，缺陷方向与 Arch 定位一致。

## 3. 离线 NVRTC + lineinfo 站点证明（Backend 证据，核其自洽）

- old：`mul.f32(uv2.x,w2)` → `fma(uv1.x,w1)` → `fma(uv0.x,w0)`（单独舍入第三项）。
- new：`mul.rn.f32(uv1.x,w1)` → `fma(uv0.x,w0)` → `fma(uv2.x,w2)`（单独舍入中间项）；v 链同形。
- 附带寄存器-行号映射（`%f5057`/`%f5058`/`%f3069` 等）与 stitched source/lineinfo PTX 的 sha/bytes（old `92771543…`/104,103 B → new `eb59dd78…`/104,382 B；lineinfo PTX old `6cec0f62…`/580,289 B → new `cf619209…`/580,295 B），并声明 NVRTC 12.8.93、`--generate-line-info`、零 GPU 会话。
- 结论与 spec §A 一致。**我未重跑 NVRTC**（理由见观察 O1）。

## 4. 内核身份（source 独立复算；PTX 取自 identity 日志）

| artifact | source old → new | PTX old → new |
| --- | --- | --- |
| megakernel | `92771543…` → **`eb59dd78…`（我复算一致）** | `e44ac85c…`/344,169 B → **`e91b7d85…`/344,175 B** |
| aa_resolve | `0bf8af5a…`（未变，复算一致） | `b28e155c…`/27,536 B（未变） |
| depth_seg | `b546ef08…`（未变，复算一致） | `e8d84963…`/138,540 B（未变） |

`render_shade.cu` 不在 DEPTH_SEG/AA_RESOLVE 拼接内 ⇒ 「未变」逻辑自洽；属**实现修复新基线**，非判据/阈值/参考变更。

## 5. 门禁 / 边界 / 卫生

- 四门禁 runs 全 `passed=true`、unit **1387/0/402**。
- `tests/`/`fixtures/`/`scripts/` **0 diff**；G-L2 证据与修复卡-2 证据目录 **0**；guards JSON **0**（`fad2ed37…`/43,729 B）、**冻结 stash 12/12 仍 == guards**。
- manifest **17/17** 独立重算全对（README `008d8cbb…`/6,651 B、receipt `c734694c…`/7,233 B、manifest `ca45bb29…`/1,857 B 与自报一致）。
- 17 个证据文件 **无 CR、无 BOM**（上一轮 `src.diff` 的 CRLF 问题未重现）。

## 6. 口径（非声明）

README L6/L92 与 receipt `non_claims`：**不宣称修复生效**（需诊断轮 #5 token 验证）、**G-L2 判定维持未通过（判据 A FAIL）**、不改判据/阈值/参考/容差、内核身份变更属实现修复新基线；全文无「已修复/已通过/等价」正面表述 ✅

## 7. 观察项（信息级，不阻断）

1. **O1**：站点级 PTX 对照与 PTX 摘要取自 Backend 的离线 NVRTC 证据（方法/输入/行号映射完整、可复现），我**未重跑 NVRTC、未编译**——避免扰动诊断轮 #5 前的 live exe 与冻结件状态；若需完全独立，可在轮 #5 结束后用同一命令复现 PTX sha。
2. **O2**：源码再改 ⇒ 12 个 live exe 再次与 guards 不同（预期），下次正式窗口必须重新冻结建新基线。
3. **O3**：「残余 4 px」前提来自诊断轮 #4 通告（不在本提交内），本卡已如实标注待轮 #5 验证。
4. **O4**：`w0/w1/w2` 定义与 `w2=(1-u)-v` 链未动，与 spec §A 的寄存器映射（`%f5057`/`%f5058`/`%f3069`）一致。

## 8. 非声明

不宣称判据 A 已修复、不宣称 G-L2 通过/恢复、不宣称上游等价、不宣称任何渲染数值结论；不改判据/阈值/参考/相机/recipe。