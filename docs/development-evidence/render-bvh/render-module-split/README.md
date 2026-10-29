# mjwarp.render 结构反转 · 步 1（3 新叶 + 11 API + 6 依赖迁移；源暂留容器）

任务：`2026-10-30-render-module-split`（Arch；P8；案 2 步 1）
上游阻塞：`2026-10-25-render-bvh-module-split`（原卡，其阻塞原因在本片解除）

本片只动架构树：`src/**`、`tests/**`、`fixtures/**`、`Cargo.*` **零改动**；不改判据/容差/参考/已验收证据语义。

## 1. 为什么要先搬 API 才能建子叶

`normify module add` 在父模块声明了 API 时直接拒绝（normify `edit.rs`：「父模块声明了 API；容器不能有 API，先把 API 移到子模块」）。`mjwarp.render` 原本是叶子且持 11 个 API，因此必须先搬 API、让它晋升容器，才能建 `mjwarp.render.kernel` 等。

**额外发现（本片关键）**：`mjwarp.render` 的 6 条依赖**全部带 `from_api`**（4 条锚在 `render`、2 条锚在 `ImageBatchLayout`），而容器不能有 `from_api`（`normify-rs/src/validate.rs:162-176` 的 `dep/from-api-non-leaf`）。所以依赖必须**随 API 同批搬到对应叶**，否则报硬错误。

inbound 依赖 = 0（全仓无 `dep.to="mjwarp.render"`、无 `to_api` 指向 render 的 11 个 API）⇒ 迁移不需要改任何其它模块。

## 2. 结构（前 → 后）

| 项 | 前 | 后 |
| --- | --- | --- |
| `mjwarp.render` | 叶子：53 源 / 11 API / 6 依赖 | **容器**：53 源（暂留）/ 0 API / 0 依赖 |
| `mjwarp.render.kernel` | — | 叶子：1 API（`render`）+ 4 依赖 |
| `mjwarp.render.bvh` | — | 叶子：2 API（`refit_bvh`、`refit_splat_bvh`） |
| `mjwarp.render.context` | — | 叶子：8 API + 2 依赖 |
| 模块数 | 68 | **71** |
| 叶子数 | 61 | **63** |
| API 键总数 / 重复 | 398 / 0 | **398 / 0**（仅搬家） |

API 分配：`kernel` ← `mjwarp_rs::render::render`；`bvh` ← `bvh::refit_bvh`、`bvh::refit_splat_bvh`；`context` ← `create_render_context`、`get_depth`、`get_rgb`、`get_segmentation`、`RenderContext`、`OutputChannels`、`ImageOutput`、`ImageBatchLayout`。全部为**原文搬迁**（键与描述逐字未改）。

依赖分配：`kernel` ← 4 条（`from_api=render` → `model.core` / `runtime.completion` / `math` / `diagnostics`）；`context` ← 2 条（`from_api=ImageBatchLayout` → `model.core` / `diagnostics`）；`[deps.label]` 双语原文随迁。

布局（新建 `.normify/layouts/mjwarp/render.toml`）：`order = [bvh, context, kernel]`，`mode="layers"`，双语 `[reading]`（阅读序：场景/BVH → 上下文与输出 → 内核实参与着色）。

## 3. 源位置（步 1 不动）

53 源仍挂 `mjwarp.render` 容器（先例：`mjwarp.runtime` 容器 53 源、根容器 13 源）。步 2 会搬 46 源进三叶（kernel 19 / bvh 11 / context 16），容器只剩 7 源（`mod.rs`、`camera.rs`、`camera/cuda/build_rays.cu`、`rays.rs`、`texture.rs`、`texture/cuda.rs`、`texture/sample.cu`）。

**偏差记录（步 1 中间态）**：
1. **容器持 53 源** —— 步 2 清零；证据点名，不当终态。
2. **`OutputChannels` / `ImageOutput` 的 API 归属 ≠ 源码位置**：声明挂在 `context` 叶，但实现符号在 `src/render/mod.rs`（该文件步 1 与剩余 7 源都留容器）。`mod.rs` 最终归哪叶留步 2 定。
3. **`render` 正文末尾 LF**：原 `render.md` body 421 字节、末尾无换行；晋升容器由 `render_doc` 写出后 body 文本**逐字相同**、文件末尾补 1 个 LF（422 字节，卫生口径）。正文行数 24（ReadAllLines）不变。

## 4. 实测口径（ReadAllLines/LF）

- 迁移前 `render.md`：**393 行**、最后 `+++` 在**第 369 行**、正文 **24 行**（正文 sha256 `4f54ed56e7e57c9c88183a5d0597376ea162b9fe73bd139ae5411dfb72391f22`）
- 迁移后 `render/index.md`：**201 行**、`+++` 在**第 177 行**、正文 **24 行**（正文内容除末尾 LF 外逐字相同，正文 sha256 `483d91aedfb50e2707f2fa9a8f388baa9f18c15dd819cac0a0c92a9e5cdde6df`）

## 5. validate 结果（实测）

- 收口态：**错误 0 / 警告 5 行**
  - `api/leaf-empty` ×2（`mjwarp.io.render`、`mjwarp.model.render-model`，既有）
  - `structure/leaf-too-coarse`：**37 → 36**（实测，与 Chief 复算一致：`mjwarp.render` 原本 src=53 计入；升容器后不计；3 新叶 0 源不计）
  - `layout/order-incomplete` ×2（既有 io/model 两处；新建 `render.toml` 覆盖 3 子模块，未触发 `layout/missing`）
- 中间态（`in_progress`）：额外 11 条 `change/api-already-removed`（迁移已完成、任务未 verified 的正常现象），`done` 后消失。
- `validate --fingerprints`：`done` 后 0 错误、无指纹漂移（见 `logs/validate-final.txt`）。

## 6. impact（步 1 中间态，符合预期）

| 路径 | 步 1 结果 | 步 2 预期 |
| --- | --- | --- |
| `src/render/kernel/megakernel.rs` | 容器 `mjwarp.render`（53 源/0 API） | `mjwarp.render.kernel` |
| `src/render/bvh/refit.rs` | 容器 `mjwarp.render` | `mjwarp.render.bvh` |
| `src/render/context/lifecycle.rs` | 容器 `mjwarp.render` | `mjwarp.render.context` |

原文见 `logs/impact-*.txt`。

## 7. 门禁与零代码改动

- 门禁：fmt / clippy / clippy-cuda / unit（结果写入 `.normify/runs/`）；`cargo test --locked --no-run` 记 exit 码 + `Finished`。
- `git diff 9e1567a2..<收口>` 只含 `.normify/` 与本文证据目录；`src/**`、`tests/**`、`fixtures/**`、`Cargo.*` 零改动。

## 8. 未做 / 下一步

- **步 2 未做**：46 源搬迁 + 预期 `leaf-too-coarse` 36→39，等步 1 verified 后另起。
- 不宣称「架构完成」或「全部模块已拆分」：`mjwarp.render` 仍是容器持源、`mod.rs` 归属未定。
