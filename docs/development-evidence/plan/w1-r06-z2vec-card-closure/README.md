# R06 z2vec backlog 卡治理关闭证据（修复已由 19a68b49 落地）

- 任务：治理卡 `2026-11-11-w1-r06-z2vec-card-closure`（owner Backend）；被关闭卡：
  `2026-11-04-render-r06-s3b-flex-z2vec-parallel-branch`（原 backlog → proposed → in_progress → verified）。
- 日期：2026-11-11（空闲段；**零 GPU、零窗**）。
- 性质：**治理关闭证据**——修复已由既有提交 `19a68b49`（11-11 08:18）落地；
  本片**不重做修复、不改 `.cu`**、不改卡面 acceptance（5 条原文不变）。
- 边界：不改 `src/`/`tests/`/`fixtures/`/`scripts/`/判据/参考/容差；不宣称设备端渲染变化生效、
  不宣称等价/准入/发布结论。

## 1. 一句话结论

该 backlog 卡 5 条 acceptance **全部满足**（证据见 §2），按治理片关闭为 verified；
其板面归类由审计原「① 真可开工（代码修）」更正为「④ 陈旧可关闭（治理）」（Chief 2026-11-11 核定）。

## 2. 逐条 acceptance 核对（5/5 satisfied）

| # | acceptance 要点（原文摘） | 结论 | 证据 |
| --- | --- | --- | --- |
| 1 | **修复**：近平行负方向分支改为 `quat[0]=1.0f; quat[3]=0.0f;`（或整体赋值 `(1,0,0,0)`），与上游 `math.py:quat_z2vec` 逐位置一致；常规分支不动 | **满足** | 修复提交 `19a68b49`：`src/render/kernel/cuda/render_flex.cu` **+6/−1**（现源码 `:47-56`，四行赋值 `:52-55`；当前文件 sha256 `97d5fb2c…`/7,426 B）；`.cu` 仅该函数区域 6 行改动（常规分支未动） |
| 2 | **可判口径用矩阵、不用字段序**：① `vec=(0,0,-1)` → 单位阵；② `vec=(0,0,1)` → `diag(-1,-1,1)`；③ `vec=(0,1,0)` → `[[1,0,0],[0,0,1],[0,-1,0]]`；host 复刻 CPU-only | **满足** | `tests/render_flex_z2vec.rs`（sha256 `334100b7…`/8,956 B，**9 tests**）：`probe_1` 精确单位阵（`assert_eq!`）、`probe_2` = `DIAG_NEG_NEG_POS`、`probe_3` = y 轴目标矩阵；另有阈值/近平行两支/`-0.0`/零向量与 `.cu` 文本锚点防漂移。CPU 测试 **9 passed / 0 failed**（`logs/test-output.txt`） |
| 3 | **证据**：`docs/development-evidence/render-bvh/r06-s3b-flex-z2vec/` 收 README + 把 `probe_quat.log` + `check_z2vec_equivalence.py` 入库 | **满足** | 该目录已在仓：README `e28cbeb9…`/9,605、receipt `5bc16d57…`/7,103、`artifacts/probe_quat.log` `1c57ed71…`/1,934（pinned 上游 CPU warp 1.15.0 实测 7 向量）、`artifacts/check_z2vec_equivalence.py` `3bbc4fb6…`/3,358、`check_z2vec.log`、`render_flex-z2vec-excerpt.txt`、manifest 733 |
| 4 | **边界**：不改判据/容差/参考、不动 `fixtures/`、不改 `scene-flex`；零 GPU | **满足** | `19a68b49` numstat 仅：`render_flex.cu` +6/−1、`tests/render_flex_z2vec.rs` +215/−0、`docs/development-evidence/render-bvh/perworld-l-v2-flex-z2vec-pin/`（证据）、`.normify/`；`fixtures/`、`scripts/`、`scene-flex` 零改动；零 GPU |
| 5 | **门禁**：fmt / clippy / clippy-cuda / unit 全过 | **满足** | `normify gate`：fmt 1762ms / clippy 555ms / clippy-cuda 455ms 通过；unit 14484ms **1406 过 / 0 败**（`logs/gate-z2vec.txt` + `.normify/runs/2026-11-04-render-r06-s3b-flex-z2vec-parallel-branch/`） |

## 3. 修复证据与内核身份

- 修复提交：`19a68b49`（2026-11-11 08:18:29 +0800）——`render_flex.cu` 近平行反向分支由「只写 `quat[0]`」改为
  **显式写全四位** `(1,0,0,0)`；另新增 `tests/render_flex_z2vec.rs`（宿主逐式镜像 + 文本锚点，测试专用、不进产品路径）。
- 修复证据目录：`docs/development-evidence/render-bvh/perworld-l-v2-flex-z2vec-pin/`
  （README `d5262719…`/7,436、receipt `50388381…`/2,798、manifest `9e848448…`/1,251，11/11；含 host-mirror-tests、
  old/new identity、offline-probe 分支对照日志）。
- 修复前诊断证据：`docs/development-evidence/render-bvh/r06-s3b-flex-z2vec/`（Arch 2026-11-11 只读诊断；
  旧版 `(1,0,0,1)` 矩阵差 2.0；`probe_quat.log` / `check_z2vec_equivalence.py` 由 target `s3b-d8` 入库）。
- **当前 megakernel 身份即修复后值**：source `cf356b728c563f7da89e33e1d06b0765028e4861bee990c4a1db54843395ab48`；
  PTX `b63a51dd…`/348,459 B；`aa_resolve`/`depth_seg` 未变（见 pin README §identity:47/:92）。
- **强制标注**：设备端行为仅由内核身份（megakernel source/PTX 前后基线）+ 离线 PTX 证明；
  CPU 镜像断言**不等价于**设备端验收。

## 4. 边界与非声明

- 不重做修复、不改 `.cu`；不改卡面 acceptance；历史证据字节未改（本片只新增本证据目录 + 关闭卡状态）。
- 零 GPU、零窗；不改判据/参考/容差/夹具。
- **不宣称**：设备端渲染变化/修复生效、上游等价、准入/发布、NP01/NP02 通过、双平台等价。

## 5. 来源

- 被关闭卡：`.normify/changes/2026-11-04-render-r06-s3b-flex-z2vec-parallel-branch.json`
- 治理卡：`.normify/changes/2026-11-11-w1-r06-z2vec-card-closure.json`
- 板面审计归类更正：`target/render-bvh/gl2-ptx-diff/board-audit-2026-11-11/README.md` §9（target 工作材料，不入仓）

## 6. 证据文件与 manifest

- `README.md`（本文件）
- `acceptance.json`（5 条逐条核对，机器可读）
- `receipt.json`（结论/修复提交/身份/门禁/非声明）
- `logs/test-output.txt`（`cargo test --locked --features cuda-probe --test render_flex_z2vec -- --test-threads=1`，9 passed）
- `logs/gate-z2vec.txt`（四门禁输出）
- `manifest.txt`（5 文件 sha256+bytes 逐文件自核，不列自身）
