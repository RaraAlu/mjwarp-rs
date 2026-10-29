# 架构树补登：24 个模块源（render / model.core / io.core / io.render / math）

任务：`2026-10-28-arch-volume-module-sources`（Arch 2026-10-28 登记；Chief 2026-10-29 批准 24 文件方案）。
性质：只改架构树（`.normify/modules/*.md`）与证据，不改产品代码、判据、参考或夹具。

## 1. 结论

- 补登 **24** 个此前未出现在任何模块 `sources` 清单里的文件：`mjwarp.render` +13、`mjwarp.model.core` +3、`mjwarp.io.core` +5、`mjwarp.io.render` +2、`mjwarp.math` +1。
- 登记后 `normify impact <文件>` 对这 24 个路径全部返回模块/依赖/约束，不再报「没有模块对应」（抽样见 §6）。
- `normify validate --fingerprints`：错误 **0**；警告由 5 条到 6 条，唯一新增为 `structure/leaf-too-coarse` 计数 33→34（原因与处置见 §5）。
- 全仓仍未登记文件剩 **154**（physics 91 / runtime 27 / vendored WARP 头 36），另立 backlog 卡 `2026-10-29-arch-unregistered-sources-remaining`，本卡不做。

## 2. 盘点方法（只读）

1. 从 `.normify/modules/**/*.md` 的 `[[source]] path` 汇总登记源：**1091 个不同路径（1108 条 `[[source]]` 条目，跨模块共享 17 条）**；差集按去重后的路径集合计算。
2. 与 `git ls-files` 的 `src/` + `include/` + `native/` + `build.rs` 比对：**823** 个文件。
3. 差集得未登记 **178** 个；本卡按 Chief 批准收 24 个（18 主 + 6 顺手），剩余 154 个记录在 backlog 卡。

## 3. 登记清单（LF 行数、归属模块、依据）

| 模块 | 文件 | LF | 归属依据 |
| --- | --- | --- | --- |
| mjwarp.render | `src/render/kernel/cast.rs` | 481 | 同目录 `launch.rs`/`mod.rs` 已登记；内核执行链调用 |
| mjwarp.render | `src/render/kernel/megakernel.rs` | 410 | 同目录已登记文件调用；S7b2 执行器 |
| mjwarp.render | `src/render/kernel/megakernel_aa.rs` | 93 | `megakernel.rs` 子模块 |
| mjwarp.render | `src/render/kernel/megakernel_arrays.rs` | 332 | `megakernel.rs` 子模块 |
| mjwarp.render | `src/render/kernel/output.rs` | 450 | 渲染输出宿主布局 |
| mjwarp.render | `src/render/kernel/renderer.rs` | 270 | depth/seg 渲染器 |
| mjwarp.render | `src/render/kernel/cuda/render.cu` | 499 | 同模块已登记 `scene.cu`/`mesh.cu`（`src/render/bvh/cuda/`）与同目录 `render_shadow.cu` |
| mjwarp.render | `src/render/kernel/cuda/render_common.cu` | 410 | 同上 |
| mjwarp.render | `src/render/kernel/cuda/render_math.cu` | 309 | 同上 |
| mjwarp.render | `src/render/kernel/cuda/render_megakernel.cu` | 368 | 同上 |
| mjwarp.render | `src/render/kernel/cuda/render_shade.cu` | 457 | 同上 |
| mjwarp.render | `src/render/kernel/cuda/aa_resolve.cu` | 48 | 同上 |
| mjwarp.render | `src/render/context/tests.rs` | 379 | `context.rs` 的单测模块 |
| mjwarp.model.core | `src/model/volume.rs` | 253 | core 职责=模型布局与输入契约；`volume_input.rs` 直接引用 |
| mjwarp.model.core | `src/model/volume_input.rs` | 310 | 同上 |
| mjwarp.model.core | `src/model/collision_inputs.rs` | 211 | 输入字段规范（32 项）；Chief 2026-10-29 裁定归 core |
| mjwarp.io.core | `src/io/volume.rs` | 213 | io.core 已拥有 G01/collision 原生读取；volume 同族 |
| mjwarp.io.core | `src/io/collision_inputs.rs` | 474 | 与 `io/collision.rs`/`io/collision_native.rs` 同族 |
| mjwarp.io.core | `src/io/state/tests.rs` | 278 | `io.core` 已登记 `state/extended/tests.rs` 先例 |
| mjwarp.io.core | `include/mjwarp_collision_inputs_probe.h` | 69 | `io/collision_inputs.rs` 注释指向该 ABI；同族 `mjwarp_collision_probe.h` 已登记 |
| mjwarp.io.core | `native/model_probe_collision_inputs.cpp` | 236 | 上述 ABI 的原生实现（被 `model_probe_sparse.cpp` 同 TU 引入） |
| mjwarp.io.render | `include/mjwarp_render_probe.h` | 50 | `io/render.rs` 注释指向该 ABI |
| mjwarp.io.render | `native/model_probe_render.cpp` | 233 | 上述 ABI 的原生实现（被 `model_probe.cpp` 同 TU 引入） |
| mjwarp.math | `src/math/fma/tests.rs` | 461 | `math/fma.rs` 的单测模块 |

逐文件 LF 计数原始输出：`logs/line-counts.txt`。

## 4. 模块源计数（前后）

| 模块 | before | after | delta |
| --- | --- | --- | --- |
| mjwarp.render | 40 | 53 | +13 |
| mjwarp.model.core | 2 | 5 | +3 |
| mjwarp.io.core | 11 | 16 | +5 |
| mjwarp.io.render | 1 | 3 | +2 |
| mjwarp.math | 2 | 3 | +1 |
| 合计 | 56 | 80 | +24 |

原始记录：`logs/module-counts.txt`；登记命令输出见 §11。

## 5. validate 前后与警告处置

- before（改动前同一命令，2026-10-29 22:07）：错误 0，警告 **5**（`logs/validate-before.txt`）。
- after（登记后、收口刷新指纹前）：错误 0，警告 **6**（`logs/validate-after.txt`）；多出的一条是 `evidence/fingerprint-drift 5 个模块指纹已过期`，属 `done` 前的中间态，收口后消失（`logs/validate-final.txt`）。
- 收口后最终：错误 0，警告 **5**（警告条数与基线相同；`logs/validate-final.txt`）。相对基线唯一的变化是 `structure/leaf-too-coarse` 一条警告里内嵌的叶子计数 **33→34**；按 QA 2026-10-29 复核（重建两版模块树、59 个叶子），与 (33,34) 相容的口径下**唯一翻转的叶子是 `mjwarp.io.render`（源 1→3）**；`mjwarp.model.core` 基线即 2 源/479 行、本已计入 33 条 coarse，本片 2→5（479→1253 行）不改变其 coarse 状态。该警告与既有 33 条同类、属记录级；叶子拆分（含 `model.core`/`io.render` 规模）留待 `2026-10-29-arch-unregistered-sources-remaining` 的架构设计，本卡不拆。另：`structure/leaf-too-coarse` 的精确规则在仓库内无文档（normify 二进制），本证据采用 QA 重建的「叶子源数 ≥2（或再加总行 >220）」口径且与 (33,34) 相容；该口径文档待补（QA O5）。
- 两条 `layout/order-incomplete` 与两条 `api/leaf-empty` 为改动前既有警告，本卡未触碰，状态不变。
- 若最终输出与上述任何一条不符，以 `logs/validate-final.txt` 为准并在本节追记。

## 6. impact 抽样（5 处，登记后可用）

| 路径 | 命中的模块 | 证据 |
| --- | --- | --- |
| `src/render/kernel/cast.rs` | mjwarp.render（源码 53，API 11） | `logs/impact-cast.txt` |
| `src/model/volume_input.rs` | mjwarp.model.core（源码 5，API 28） | `logs/impact-volume-input.txt` |
| `src/io/volume.rs` | mjwarp.io.core（源码 16，API 17） | `logs/impact-io-volume.txt` |
| `src/io/render.rs` | mjwarp.io.render（源码 3，API 0） | `logs/impact-io-render.txt` |
| `src/math/fma/tests.rs` | mjwarp.math（源码 3，API 32） | `logs/impact-math-fma-tests.txt` |

五处均返回模块、依赖/被依赖、约束规则与相关任务；未出现「没有模块对应」。QA 复核时对全部 24 个路径逐一跑过 impact，24/24 命中。注：本目录 impact 日志采集于任务 in_progress（收口前），尾行「未关闭任务」计数比终态多 1，属时间性差异（QA O4 记录，非缺陷）。

## 7. 零代码改动边界

- 改动范围：`.normify/modules/`（5 个模块 md 的 `source` 与 `fingerprint`）、`.normify/changes/`（本卡 + QA 复核卡 + 剩余源 backlog 卡）、`.normify/runs/<本任务>/`（门禁记录）、本证据目录。
- 产品代码零改动：`src/**`、`tests/**`、`fixtures/**`、`include/**`、`native/**`、`Cargo.*` 在任务前后无内容变化（这些文件被登记但未修改；以 git diff 为准）。
- 未改任何判据、容差、参考、夹具、冻结物。

## 8. 未覆盖（本卡不做）

全仓仍剩 **154** 个未登记文件：physics 91（collision 72、callback 2、derivative 2、history 4、integrator 4、inverse 2、mass_matrix 1、rne_bias 1、sensor 2、tendon_velocity 1）、runtime 27（collision_gpu 19、backend 2、collision_upload 1、external 2、launcher 2、presence 1）、`include/warp-headers/*.h` 36（vendored 第三方头；建议不入架构树，论证见 backlog 卡）。已登记 backlog 卡 `2026-10-29-arch-unregistered-sources-remaining`，不 start。

## 9. 证据文件

| 文件 | 内容 |
| --- | --- |
| `logs/validate-before.txt` | 改动前 validate 输出 |
| `logs/validate-after.txt` | 登记后、收口前输出（含 fingerprint-drift 中间态） |
| `logs/validate-final.txt` | 收口后最终输出 |
| `logs/module-counts.txt` | 五模块前后计数 |
| `logs/line-counts.txt` | 24 文件 LF 计数 |
| `logs/impact-*.txt` | 五处 impact 抽样 |

无 manifest：主卡证据未提交 manifest，QA 复核以 `git diff HEAD -- .normify/modules <本证据目录>` 替代核验并接受（QA O6）。
QA 复核卡 `2026-10-29-arch-volume-module-sources-check`：verdict pass / 阻断 0 / 观察 6（commit `6d374cc9`/`9986bab2`）；其中 O1（coarse 归因）、O2（登记源计数口径）、O3（`.cu` 依据措辞）已在本文件与 receipt 的 `Normify-Fixes` 提交中修正，O4/O5/O6 记录级已接受。

## 10. 非声明

- 本卡只做架构树登记与指纹刷新；不宣称架构树已覆盖全部源码（剩 154 个未登记，见 §8），也不对产品渲染/数值/性能作任何结论。
- 不把 vendored 头或测试文件的登记解释为第三方代码归属或接口契约。

## 11. 登记命令（逐字）

```text
normify promote 2026-10-28-arch-volume-module-sources
normify start 2026-10-28-arch-volume-module-sources
normify module source mjwarp.render --add src/render/kernel/cast.rs --add src/render/kernel/megakernel.rs --add src/render/kernel/megakernel_aa.rs --add src/render/kernel/megakernel_arrays.rs --add src/render/kernel/output.rs --add src/render/kernel/renderer.rs --add src/render/kernel/cuda/render.cu --add src/render/kernel/cuda/render_common.cu --add src/render/kernel/cuda/render_math.cu --add src/render/kernel/cuda/render_megakernel.cu --add src/render/kernel/cuda/render_shade.cu --add src/render/kernel/cuda/aa_resolve.cu --add src/render/context/tests.rs
normify module source mjwarp.model.core --add src/model/volume.rs --add src/model/volume_input.rs --add src/model/collision_inputs.rs
normify module source mjwarp.io.core --add src/io/volume.rs --add src/io/collision_inputs.rs --add src/io/state/tests.rs --add include/mjwarp_collision_inputs_probe.h --add native/model_probe_collision_inputs.cpp
normify module source mjwarp.io.render --add include/mjwarp_render_probe.h --add native/model_probe_render.cpp
normify module source mjwarp.math --add src/math/fma/tests.rs
```

输出摘要：render 新增 13 共 53；model.core 新增 3 共 5；io.core 新增 5 共 16；io.render 新增 2 共 3；math 新增 1 共 3。
