# z2vec 片 2：记录面修正证据（零 `.cu`、零行为变化、零 GPU）

- 任务：`2026-11-11-render-flex-z2vec-record-errata`（owner Backend；p96；P8-渲染与BVH）。依据 Chief
  2026-11-11 12:00 派工卡 A；QA 复核 z2vec 片 1（`ebc8e97e`：1 低 L1 + 1 观察 O1 + O2）与 QA 复核
  errata 批 r2（`fd72cf0b`：1 一般 G1 + 4 低 + 1 观察）；Arch 镜像非位级 oracle 观察（采纳）。
- 边界：**纯文档/测试注释**——无 `.cu`、无 `src/`、无行为变化；`fixtures/`、`scripts/` 零改动；零 GPU、
  不重跑窗口/诊断轮、不重冻 guards；不改判据/参考/容差/相机/recipe；受影响历史证据目录 **append-only**
  （原文字节不改，仅追加 §7/§G 与 manifest 重算）。

## 1. 更正清单

| # | 来源 | 更正内容 | 落点 |
| --- | --- | --- | --- |
| C1 | QA z2vec O1（Arch 同项） | 镜像 `warp_length3` 注明「数值对应、**非位级 oracle**（Rust 左结合 vs `.cu` 显式 `fmaf` 链，1 ulp 级、容差 1e-6）」 | `tests/render_flex_z2vec.rs` 文件头 + `warp_length3` 注释 |
| C2 | QA z2vec L1 | `(0,1,0,0)` 字面在 `render_flex.cu`/全 `src/` **不存在**；实际未改动的是同向路径初值 `(0,0,0,1)`（`render_flex.cu:36-39`） | 同上（`flex_quat_z2vec` 注释）；`perworld-l-v2-flex-z2vec-pin/README.md` §7 |
| C3 | QA z2vec O2 | PTX 长度双口径：**348,459 B**（去尾 NUL，产品/报告口径）vs **348,460 B**（`nvrtcGetPTXSize` 含 NUL）；sha256 按去 NUL 计 | `.../flex-z2vec-pin/README.md` §7；`artifacts/ptx-length-dual-accounting.txt` |
| C4 | QA errata r2 **G1** | diag7 `gpu_post_actual` = **errata 时段复测**（mtime 2026-11-11 08:06:18），非 run 时采样；删除「post recorded in run-summary.txt」错误引用（该文件仍为 `gpu_after=$gpu` 占位） | `perworld-l-v2-gl2-errata-batch/README.md` §G；`artifacts/c1-diag7-post-provenance.txt`；target 现场文件已标注 |
| C5 | QA errata r2 低 L1 | `b-field-change.diff` **名不副实**：只含 3 个测试从 `diag.rs` 搬到 `diag_ext.rs` 的 hunk，不含 B 语义改动；old blob `25affdc8` 为编辑中间态（`84ed67d2^` 实为 `02533c7d`） | `.../gl2-errata-batch/README.md` §G |
| C6 | QA errata r2 低 L2 | `single-normalize/receipt.json` 为**结构重排 + addenda**（非纯追加）；除 `addenda[]` 外逐键与原值相同 | 同上 §G |
| C7 | QA errata r2 低 L3 | `diag.rs` / `diag_ext.rs` **物理行数 484 / 237**（非 `Get-Content` 的 466/231）；两者均 < 500 棘轮 | 同上 §G；`artifacts/physical-line-counts.txt` |
| C8 | QA errata r2 低 L4 | 证据 `task_id` 指向被 abandon 旧卡；补 successor 卡 `…-gl2-errata-batch-r2`（历史 task_id 保留不改写） | 同上 §G |
| B1 | Chief 派工 + QA 建议 | **backlog**：megakernel 身份已变 `cf356b72…`/`b63a51dd…`（348,459 B）⇒ **后续任何渲染 GPU 窗口必须重新冻结**；本卡不重冻、不占窗 | `.../flex-z2vec-pin/README.md` §7；本卡 `note` |

## 2. 验证

- 宿主镜像测试：`cargo test --locked --test render_flex_z2vec` ⇒ **9 passed / 0 failed**
  （注释改动；9 个断言与镜像逻辑零改动）。
- 内核身份：本卡无 `.cu`、无 `src/` 改动 ⇒ megakernel source `cf356b72…`(110,879 B) / PTX
  `b63a51dd…`(348,459 B) 不变；depth_seg `1e3f273c…`/`31912377…`(138,857 B)、aa_resolve
  `0bf8af5a…`/`b28e155c…`(27,536 B) 不变（沿用片 1 记录，未重算——无源码变化，重算无信息增益）。
- 四门禁（`.normify/runs/2026-11-11-render-flex-z2vec-record-errata/`）：fmt / clippy / clippy-cuda 通过；
  unit **1,403 过 / 0 败 / 402 ignored**（与片 1 基线一致，注释不改变用例数）。
- 编码：本卡新增/修改文件均为 UTF-8/LF、无 BOM；三个目录 manifest 重算自核（本目录 5/5、片 1 目录 11/11、
  errata 目录 8/8）。
- 边界：`unchanged src/`、`unchanged fixtures/`、`unchanged scripts/`；`git diff -- '*.cu'` 为空。

## 3. 非声明

不宣称设备端渲染变化/修复生效/上游等价；CPU 镜像断言不等价于设备端验收；本卡不重冻 guards、不占 GPU 窗口；
不改判据/阈值/参考/容差/相机/recipe；不重新解释历史 dump 语义（旧字节与旧记录不改写）。