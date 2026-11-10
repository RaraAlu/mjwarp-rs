# QA 复核：P9 spot 灯 cos_theta dot 次序 pin（`e22891da`）

- 复核卡：`2026-11-11-render-perworld-l-v2-cos-theta-dot-pin-check`（QA）
- 被复核：`2026-11-11-render-perworld-l-v2-cos-theta-dot-pin`（Backend；登记 `fd6b0d45` / start `d137ca66` / 代码 `e22891da`）
- 性质：只读复核，零 GPU；**未重新编译**（避免扰动诊断轮 #6 前的 live exe 与冻结件）；只写本卡与本卡证据。
- 结论：**PASS（0 阻断，4 条信息级观察）→ 可发诊断轮 #6 token**。本复核只核代码/证据/边界/口径与等价性，**不判定修复效果**；G-L2 判定维持未通过。

## 1. 改动面（逐行 diff）

`e22891da` 源码改动**只有** `src/render/kernel/cuda/render_shade.cu` **+5/−1**：

```diff
         if (lighttype == 0) {
-            float cos_theta = dot3_ba(-L.x, -L.y, -L.z, lightdir.x, lightdir.y, lightdir.z);
+            // P9：参考真值 `wp.dot(-L, lightdir)` 为 3×plain `mul.f32` + 1 `neg` + 2 `sub.f32`（**无 fma**），
+            // 数值 = `-(rnd(rnd(Lx·dx + Ly·dy) + Lz·dz))`；`neg` 精确，可外提。
+            float cos_theta = -__fadd_rn(
+                __fadd_rn(__fmul_rn(L.x, lightdir.x), __fmul_rn(L.y, lightdir.y)),
+                __fmul_rn(L.z, lightdir.z));
```

- `dot3_ba` 本体（`render_math.cu`）**0 diff**；`cos_cutoff` 比较行、`attenuation * powf(fmaxf(cos_theta,0), lightexp)` 行、spot 的 L 归一化（P4 三次除法 / P6 `length3_midpref`）均**未出现在 diff**。
- 其它 `.cu` 0 diff；文件 **468 行**（blob LF 独立核实）。

## 2. 与参考序列的逐舍入点等价性

参考真值（审计 §B / PTX 8715-8720）：`mul;mul;neg;sub;mul;sub`（无 fma），数值 = `-(rnd(rnd(m0+m1)+m2))`，`mi = rnd(·)`。

落地式：`-(add(add(mul(Lx,dx), mul(Ly,dy)), mul(Lz,dz)))` = 3 次 `mul.rn` 各自舍入 + 2 次 `add.rn` 各自舍入（结合序 `((m0+m1)+m2)`）+ 外提 `neg`。

**解析**：`neg` 精确；`sub(x, −y)` 与 `add(x, y)` 同舍入；`(−x) − y` 与 `−(x+y)` 位型相同 ⇒ 参考链 `sub(sub(neg(m0), m1), m2)` 与落地式在每个舍入点一致。

**数值复算**（f32，4000 随机例，脚本 `target/qa-gl2/p9-numeric-check.ps1` 思路）：
- `sub(a, −b)` vs `add(a, b)`：**0/4000 差异**
- 参考 `sub(sub(neg(m0),m1),m2)` vs 落地式：**0/4000 差异**
- 旧 `dot3_ba` vs 新式：**1447/4000（36%）输入不同** ⇒ pin 确实改变数值（与「≈5 px 翻转」预期同向）

## 3. 离线 NVRTC + lineinfo 站点证明（Backend 证据，核其自洽）

- old：`neg×3 + mul.rn + fma.rn + fma.rn`（`dot3_ba` 收缩）
- new：`mul.rn×3 + add.rn×2 + neg.f32`
- 寄存器映射：`L.x/y/z` = `%f5156/%f5157/%f5158`，由 spot 分支 3 次 `div.rn` 定义（P4 落地）；`lightdir.x/y/z` = `%f911/%f912/%f913`，由 `ld.global.f32` 定义。
- 推导 `value = -(rnd(rnd(m1+m2)+m3))`，与参考同结论；与我的独立推导一致。**我未重跑 NVRTC**（见 O1）。

## 4. 内核身份（source 独立复算；PTX 取自 identity 日志）

| artifact | source old → new | PTX old → new |
| --- | --- | --- |
| megakernel | `eb59dd78…` → **`02160146…`（我复算一致）** | `e91b7d85…`/344,175 B → **`7fd86b85…`/344,181 B** |
| aa_resolve | `0bf8af5a…`（未变） | `b28e155c…`/27,536 B（未变） |
| depth_seg | `b546ef08…`（未变） | `e8d84963…`/138,540 B（未变） |

`render_shade.cu` 不在 DEPTH_SEG/AA_RESOLVE 拼接内 ⇒ 「未变」自洽；属实现修复新基线。

## 5. 门禁 / 边界 / 卫生

- 四门禁 runs 全 `passed=true`、unit **1387/0/402**。
- `tests/`/`fixtures/`/`scripts/` **0 diff**；P7 证据目录 **0**；guards JSON **0**（`fad2ed37…`/43,729 B）、**冻结 stash 12/12 == guards**。
- manifest **17/17** 独立重算全对（README `0bcff5e3…`/6,215 B、receipt `262b770d…`/6,788 B、manifest `e7a2189c…`/1,862 B 与自报逐字节一致）。
- 17 个证据文件 **无 CR、无 BOM**（LF）。

## 6. 口径（非声明）

README L6/L92 与 receipt `non_claims`：**不宣称修复生效**（需诊断轮 #6 token）、**G-L2 判定维持未通过（判据 A FAIL）**、不改判据/阈值/参考/容差、内核身份变更属实现修复新基线；无「已修复/已通过/等价」正面表述 ✅

## 7. 观察项（信息级，不阻断）

1. **O1**：PTX 层对照取自 Backend 的离线 NVRTC 证据（方法/输入/寄存器映射完整可复现），我**未重跑 NVRTC、未编译**（避免扰动轮 #6 前 live/冻结件）。
2. **O2**：±0 角——三项乘积全为同号零时，`neg` 外提的落地式可能给 `-0`，参考 `neg;sub` 链取决于被 neg 的项可能给 `+0`；两者数值相等，`cos_theta < cos_cutoff` 与 `fmaxf` 判定不受影响（无实际影响）。
3. **O3**：源码再改 ⇒ 12 个 live exe 与 guards 再次不同（预期），下次正式窗口必须重新冻结建新基线。
4. **O4**：残余像素前提来自轮 #5 零效果后的站点审计（不在本提交），卡内已如实标注待轮 #6 验证。

## 8. 非声明

不宣称判据 A 已修复、不宣称 G-L2 通过/恢复、不宣称上游等价、不宣称任何渲染数值结论；不改判据/阈值/参考/相机/recipe。