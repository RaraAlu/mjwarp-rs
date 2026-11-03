# R06-S3b⑥：flex GPU 用例取数缺陷修复 + CPU 自证（attempt 1 INVALID_ROUND 对应）

任务：`2026-11-04-render-r06-s3b-flex-harness-schema-fix`（p76，owner Backend；depends `…-flex-device-harness`）。
基线 HEAD：`a8fd9a4b`。本片**零 GPU、零窗口、不做真冻结**；只改 `tests/render_flex_validate.rs` + 本证据目录。

## 1. 缺陷与复现（attempt 1 = INVALID_ROUND，证据 `7c4a8296`）

- 窗口 attempt 1 的实体体首跑：`panicked at tests/render_flex_validate.rs:56`，经 `ints(model, "flex_edge")`。
- 根因：**`model-312.json` 的 17 个 flex 数组里没有 `flex_edge`**（dump 字段清单只含 `flex_edgeadr`/
  `flex_edgenum`/`flex_elem`/`flex_shell` 等），`arrays.flex_edge.values` 取不到 → panic。
- 次因：`flexedge_j_rownnz: vec![1; len/2]` 属无依据硬编码；`FlexTopologyFields.nflex` 未填（CPU 自证时暴露）。
- 教训：GPU 用例只做 `--no-run` 从未执行，schema 缺陷到首跑才暴露 → 本片新增**普通 `cargo test` 可跑的 CPU 自证**。

## 2. 修复口径（`tests/render_flex_validate.rs`，358 行）

- **`flex_edge` 从 `flex_elem` 派生**（dim==1 时元素就是边；上游 `mjmodel.h`：`flex_elem` 存 `dim+1 = 2`
  个顶点 id/元素、元素数 == 边数），并加四条交叉校验确认派生前提：
  `nflexelemdata == 2*nflexedge`、`flex_edge.len() == 2*nflexedge`、`sum(flex_edgenum) == nflexedge`、
  `flex_edgeadr[0] == 0`、每个端点 `< 各自 flex_vertnum`。验证值 `flex_edge = [0,1, 1,2, 2,3, 0,1]` 与场景一致。
- **稀疏行同源推导**：每条边一行、行内 2 个非零 → `flexedge_j_rownnz = [2; nflexedge]`、
  `flexedge_j_rowadr = [0,2,4,…]`、`flexedge_j_colind = flex_edge`（不再硬编码 `j_rownnz=[1;…]`）。
- **`FlexTopologyFields.nflex` 显式填**（`counts.nflex`），其余 flex 字段逐项从 dump 取。

## 3. CPU 自证（新增硬要求；普通 `cargo test` 可跑，不碰 CUDA）

测试 `flex_model_dump_schema_self_check`（`#[test]`，非 `#[ignore]`）：
1. 17 个 flex 数组逐项断言 `kind`/`count`/`values.len()`（int32/float64/float32 与计数关系全对）；
2. `flex_model`/`flex_positions`/`flex_edges` 构造 `FlexRenderFields`/`FlexPositionFields`/`FlexEdgeFields`；
3. `FlexHostAssembly::build(...)`（CPU 校验链）→ `flex_geom_edgeid = [0,1,2,3]`、`flex_edge` 内容断言；
4. `FlexTopologyFields` + `FlexMeshChannel` → `FlexMeshAssembly::build`（tri/BVH 输入）→ `mesh_count=0`、
   胶囊叶 4、`bvh_ids = [u64::MAX; 2]`。
- 命令：`cargo test --locked --test render_flex_validate` → **12 passed / 0 failed / 1 ignored**（含支持模块用例；
  ignored 为 GPU 用例）；`--nocapture` 单测输出见 `self-check.txt`。
- `cargo test --locked --features cuda-probe --no-run --test render_flex_validate` → exit 0（`compile.log`）。

## 4. 与 attempt 1 的对应关系

- attempt 1 失败点（`flex_edge` 取不到）→ §2 派生修复；`j_rownnz` 硬编码 → §2 同源推导。
- 判据/容差/参考零改动；GPU 用例逻辑除取数/构造外不变（仍 `#[ignore]`，窗口执行）。
- 修复后 exe 会重建 → **窗口需重新真冻结**（`-Attempt 1`，exe 哈希变），再跑 attempt 2；按 Chief 指令顺延空闲块。

## 5. 门禁

- fmt / clippy / clippy-cuda / unit 全过；零 GPU；`fixtures/`、`src/` 零改动；不宣称 flex 渲染通过。
