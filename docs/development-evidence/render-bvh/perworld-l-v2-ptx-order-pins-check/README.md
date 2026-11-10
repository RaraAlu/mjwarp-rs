# QA 复核：P2/P3/P4 PTX 次序 pin 修复卡（`358b84fc` + errata `0ad4795a`）

- 复核卡：`2026-11-11-render-perworld-l-v2-ptx-order-pins-check`（QA）
- 被复核：`2026-11-11-render-perworld-l-v2-ptx-order-pins`（Backend；`358b84fc` 代码 + `0ad4795a` errata）
- 性质：只读复核，零 GPU；未跑诊断轮；只写本卡与本卡证据。
- 结论：**PASS（0 阻断，1 条低观察 + 3 条信息观察）→ 可发诊断轮 token**。不宣称修复生效；G-L2 判定维持未通过；不改判据/阈值/参考。

## 1. 改动面（逐行 diff）

| 文件 | 变化 | 内容 |
| --- | --- | --- |
| `src/render/kernel/cuda/render_common.cu` | **+11** | 新增 `normalize3_midpref`：`sqrtf(fmaf(a.z,a.z, fmaf(a.x,a.x, a.y*a.y)))` + `v3(a.x/l, a.y/l, a.z/l)` |
| `src/render/kernel/cuda/render_shade.cu` | **+4/−2** | P2 `half = normalize3_midpref(add(L, view_dir))`；P4 `L = v3(delta.x/norm, delta.y/norm, delta.z/norm)` |
| `src/render/kernel/cuda/render_megakernel.cu` | **+2/−1** | P3 `normal = normalize3_midpref(mat_vec(geom_mat, vn))` |

- 合计 **+17 / −3**，与实际 `git show --numstat` 完全一致（`render_common.cu` 11/0、`render_shade.cu` 4/2、`render_megakernel.cu` 2/1）。
- **`normalize3_rays` 函数体未出现在 diff 中（字面未动）**；helper 定义在其后，按拼接顺序下游可见。
- `render_math.cu` / `render.cu` / `render_shadow.cu` / `render_flex.cu` 等 **0 diff**；`.normify/modules/mjwarp/render/kernel.md` 仅 fingerprint 刷新 1/1。
- 文件 sha（old→new，我从父提交 blob 独立复算）：`render_common.cu` `9625d684…`/20,974 → `887c3000…`/21,495；`render_shade.cu` `f8394d73…`/20,485 → `69d3a2e2…`/20,758；`render_megakernel.cu` `6d718100…`/21,167 → `c3936f86…`/21,261。

## 2. helper 公式 vs Arch 目标 PTX 序列

Arch 离线证据（`target/render-bvh/gl2-ptx-diff/README.md:70`）目标序列：
`mul y*y ; fma x*x ; fma z*z ; sqrt.rn.f32 ; div.rn.f32 ×3`（中间项单独舍入）。

实现求值次序：`a.y * a.y`（mul）→ `fmaf(a.x, a.x, …)`（fma）→ `fmaf(a.z, a.z, …)`（fma）→ `sqrtf` → `a.x/l, a.y/l, a.z/l`（三次除法）。**逐项对应**，与上游 `vec.h:1113-1115` 语义一致；`l > 0` 分支与零向量返回零保持与 `normalize3_rays` 同语义。

P4 对照 Arch 证据第 12 项（上游 `normalize_with_norm`：`L = x / norm` 三次除法）：实现改为逐分量除法，**一致**。

## 3. 内核身份新基线（独立复算）

按 `mod.rs` include_str 顺序 LF 字节串联复算 source-sha：

| artifact | source old → new | PTX old → new |
| --- | --- | --- |
| megakernel | `4d932c90…` → **`d3e04aa7…`**（我复算一致） | `13ac8a2e…`/344,186 B → **`98f09afc…`/344,166 B** |
| aa_resolve | `0bf8af5a…`（未变，我复算一致） | `b28e155c…`/27,536 B（未变） |
| depth_seg | `b546ef08…`（未变，我复算一致） | `e8d84963…`/138,540 B（未变） |

- depth_seg 拼接（scene+render_math+render）不含本卡改动的 3 个文件 ⇒ 身份不变成立；aa_resolve 单文件未改 ⇒ 不变。
- 属**实现修复新基线**，非判据/阈值/参考变更（证据原文一致）。

## 4. 门禁 / 边界 / 卫生

- 四门禁 runs 全 `passed=true`、unit **1387/0/402**；build（dev/test 均 Finished）与 `cargo test --no-run` 成功。
- 边界：`tests/`/`fixtures/`/`scripts/` **0 diff**；G-L2 / RCA / mesh-fix 证据目录 **0**；`guards-perworld-l2-light.json` **0**（`fad2ed37…`/43,729 B）；**冻结 stash 12/12 仍 == guards**（逐文件重算）；manifest **12/12 重算全对**。
- **低观察 O1**：`logs/src.diff` 含 **64 个 CR**（CRLF），同目录其余 10 个日志与工作树 `.cu` 均为 LF ⇒ 单个 artifact 违反「证据 UTF-8/LF/无裸 CR」口径（内容与 git diff 一致，不影响结论）；建议后续 errata 归一化并重算 manifest。

## 5. diff 统计与 errata

- 证据 README/receipt 记录 **+17/−3**，与实际 git stat 一致（common +11、megakernel +2/−1、shade +4/−2）。
- `0ad4795a` 将原 **+37/−3** 更正为 +17/−3，并重算 manifest（代码零改动，仅 README/manifest/receipt + 卡 addenda + runs 刷新）。
- **主提交 `358b84fc` 的 commit message 仍含 +37/−3**（git message 不可改写）；errata/card 已成文更正 ⇒ 记录口径正确，仅从 message 处仍会看到旧数字（观察 O2，信息级）。

## 6. 观察项（信息级，不阻断）

1. **O1（低）**：`logs/src.diff` CRLF（其余皆 LF）→ 证据卫生偏差，建议 errata 归一化（会改 manifest，需重算）。
2. **O2（信息）**：主提交 message 的 +37/−3 无法改写；errata 已更正正文与卡。
3. **O3（信息）**：Arch 证据自述「dump2 CPU 重放显示该次序改变**不移动**这 4 个像素，重放 depth 3/4 像素差 1 ULP，结论不算铁」⇒ 本 pin 属**指令对齐/敏感性推导**，修复效果未证；本卡 README 已如实写「不宣称修复生效、需唯一一轮诊断轮验证」，口径正确。
4. **O4（信息）**：源码再改使 **12 个 live exe 再次 ≠ guards**（预期）⇒ 下一次正式窗口必须重新冻结建新基线。

## 7. 非声明

不宣称修复生效、不宣称判据 A 已修复、不宣称 G-L2 通过/恢复、不宣称上游等价、不宣称任何渲染数值结论；不改判据/阈值/参考/相机/recipe；G-L2 判定维持未通过。