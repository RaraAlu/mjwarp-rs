# QA 复核：修复卡-2（R1/P5/P6）`7aa99c9b`

- 复核卡：`2026-11-11-render-perworld-l-v2-ptx-order-pins-2-check`（QA）
- 被复核：`2026-11-11-render-perworld-l-v2-ptx-order-pins-2`（Backend；代码 `7aa99c9b`）
- 性质：只读复核，零 GPU；**未编译**（避免干扰并行诊断轮 #4 的冻结件/live exe）；只写本卡与本卡证据。
- 结论：**PASS（0 阻断，3 条信息观察）→ 可发诊断轮 #4**。本复核只核代码/证据/边界，不判定修复效果；G-L2 判定维持未通过。

## 1. 改动面（逐行 diff）

| 文件 | 变化 | 内容 |
| --- | --- | --- |
| `src/render/kernel/cuda/render_common.cu` | **+15/−1** | 新增 `length3_midpref`（`sqrtf(__fmaf_rn(a.z,a.z, __fmaf_rn(a.x,a.x, __fmul_rn(a.y,a.y))))`）与 `dot3_first`（`__fmaf_rn(a2,b2, __fmaf_rn(a1,b1, __fmul_rn(a0,b0)))`）；原 helper 注释由「P2/P3 两处」收窄为「P3 一处」 |
| `src/render/kernel/cuda/render_shade.cu` | **+6/−4** | R1：half `normalize3_midpref` → `normalize3_rays`；P5：`ndotl` `dot3_ba` → `dot3_first`；P6：灯 L 模 `length3` → `length3_midpref` |

- 合计 **+21 / −5**（15/1 + 6/4），与实际 `git show --numstat` 完全一致。
- `render_math.cu` / `render.cu` / `render_shadow.cu` / `src/render/bvh/cuda/scene.cu` 等 **0 diff**。

## 2. helper 展开序 vs Arch 附录 D

| 站点 | Arch 附录 D 目标 | 实现 | 结论 |
| --- | --- | --- | --- |
| R1 half 归一化 | **首项** `mul(Hx²)→fma(Hy²)→fma(Hz²)` | `normalize3_rays`：`sqrtf(fmaf(a.z,a.z, fmaf(a.y,a.y, a.x*a.x)))` | ✅ 一致（回退正确） |
| P5 `ndotl` | **首项** `mul(nx·Lx)→fma(ny·Ly)→fma(nz·Lz)` | `dot3_first`：`__fmul_rn(a0,b0)` 打头 + 两次 `__fmaf_rn` | ✅ 一致 |
| P6 `wp.length(delta)` | **中间项** `mul(dy²)→fma(dx²)→fma(dz²)` | `length3_midpref`：`__fmul_rn(a.y,a.y)` 为中间项 + 两次 `__fmaf_rn` | ✅ 一致 |

（`__fmul_rn`/`__fmaf_rn` 内在函数对应 `mul.rn.f32`/`fma.rn.f32`，比裸运算符更利于 PTX 次序确定性。）

## 3. 保留项（未动）

- P3 mesh 顶点法线仍 `normalize3_midpref`（`render_megakernel.cu` 不在 diff）。
- P4 灯 L 三次除法行未被触碰。
- `ndoth` 仍 `dot3_ba`；`normalize3_rays`、`dot3_ba` 定义未动。
- `scene.cu::length3`（`sqrtf(dot(a,a))`）未动；`cos_theta` 未动。

## 4. 内核身份（source 独立复算；PTX 取自 identity 日志）

| artifact | source old → new | PTX old → new |
| --- | --- | --- |
| megakernel | `d3e04aa7…` → **`92771543…`**（我复算一致） | `98f09afc…`/344,166 B → **`e44ac85c…`/344,169 B** |
| aa_resolve | `0bf8af5a…`（未变，复算一致） | `b28e155c…`/27,536 B（未变） |
| depth_seg | `b546ef08…`（未变，复算一致） | `e8d84963…`/138,540 B（未变） |

- depth_seg 拼接（scene+render_math+render）不含本卡 2 文件 ⇒ 身份不变成立。
- **PTX 未独立复算**：编译会重建 live exe 并与并行诊断轮 #4 的冻结件/live 一致性口径冲突，故只核 identity 日志与 source 复算。

## 5. 门禁 / 边界 / 卫生

- 四门禁 runs 全 `passed=true`、unit **1387/0/402**。
- `tests/`/`fixtures/`/`scripts/` **0 diff**；G-L2 证据与修复卡-1 证据目录 **0**；guards JSON **0**（`fad2ed37…`/43,729 B）、**冻结 stash 12/12 仍 == guards**。
- manifest **12/12** 重算全对；`logs/` 全部 LF（上一轮 `src.diff` 的 64 CR 问题本轮未重现）。
- diff 统计 **+21/−5** 与实际一致。

## 6. 观察项（信息级，不阻断）

1. **O1**：修复卡-1 的 **P2 已被本卡 R1 撤销**（Arch 附录 D 判定方向反了）；卡-1 证据未交叉引用本卡，建议后续 errata 加一行指针。
2. **O2**：本卡 README 以「残余 3 px」（world0 cam1 归零、余 3 px 在 world1 cam1：`10744` G−1、`22903` B−1、`39492` G+1）为前提，来源诊断轮 #3（该轮输出不在本提交，我未复算）。
3. **O3**：本轮源码再改 ⇒ live exe 与 guards 再次不同（预期），下次正式窗口必须重新冻结建新基线。

## 7. 后置事实（复核完成后通告，不改变本复核结论）

- Chief 通告：**诊断轮 #4 仍为 4 px**——`13268` 被 R1 翻回、world1 cam1 三 px 逐字节不变、P5/P6 零可观测效果；已转 Arch 做参考源码/头文件级穷举。
- 与本卡「不宣称修复生效」的口径一致；本复核本就只核代码/证据/边界（PASS），**不判定修复效果**；G-L2 判定维持未通过。

## 8. 非声明

不宣称判据 A 已修复、不宣称 G-L2 通过/恢复、不宣称上游等价、不宣称任何渲染数值结论；不改判据/阈值/参考/相机/recipe。