# R06-S3b tie 语义可行性结论：固定叶序/遍历规则**不可行**（证据留档）

任务：`2026-11-04-render-r06-s3b-flex-tie-order-infeasibility`（零代码改动，`unchanged src/, fixtures/`）。
日期：2026-11-04。相关前置：r3 卡（LIFO 遍历实现）已 abandon（`a0d11b7d`），实验回滚；
`render_common.cu` 已恢复到 HEAD 原状，`git diff` 为空。

## 结论（一句话）

**固定叶序/遍历规则（LIFO 右优先栈 vs 入口距离最小堆）不可行**——在产品真实场景 BVH 上，
该 ray 两种顺序的候选访问序相同（`[edge0, edge1, plane]`），edge1 后访问胜，`(134,128)` 仍差
3 ULP。可行性结论的更正原因 = **离线复刻漏了 `src/render/bvh/aabb.rs` 对 box/mesh 叶 bounds 的
`fused_mul_add`（与 GPU FMA 收缩对齐）**，导致 Python 复刻出错误 BVH 树。

> 语义记录：本结论只否定「靠遍历顺序对齐」这一条路；**不是** f32 修复结论，也不影响
> attempt5 判据。让该像素逐位等于参考需要「容差内平局改稳定键（更小 d）」这类**语义偏离**
> （偏离上游「后访问者胜」），属用户/Chief 决策项；本卡不实现、不宣称上游等价。

## 1) LIFO 实验（负结果）

- 实现：`render_common.cu::cast_ray` 候选遍历由「AABB 入口距离最小堆」改为上游 warp
  `bvh_query_next` 式 LIFO 右优先栈 + 现有 `max_dist` 剪枝；树构建/host/ABI/判据/容差/参考/
  fixtures 均未动。完整实验 diff：`artifacts/lifo-experiment.diff`。
- 四门禁（LIFO 版）：clippy 5,485ms / clippy-cuda 405ms / fmt 1,764ms / unit 79,446ms
  （1,316 过 0 败）；回执 `artifacts/lifo-gate-runs/`。
- 验证短跑（单次）：`cargo test --locked --features cuda-probe --test render_flex_validate --
  --ignored --test-threads=1`，env `CUDA_PATH=target/toolchains/cuda-12.8.1` 前置；
  wall 7.12s（内核 1.98s）；GPU 前 free 14,326 MiB/4%/29°C → 后 14,326/1%/29°C；
  **cam2 depth 仍 `(1,0)`**，cam0/cam1 `(0,0)`、rgb 全过。日志 `logs/lifo-verify-run.log`。
- 诊断微跑（Chief 批准，env-gated dump，test WIP 跑后已还原）：wall 3.29s（重建 1.48s +
  内核 1.72s）；GPU 前 14,248/4%/29 → 后 14,248/1%/29。离线对拍 dump：
  cam0/cam1 depth 0 失配、三相机 seg 0 失配、**cam2 depth 1 失配 = 仍是 41094**
  （cur `0x400015ba` vs ref `0x400015b7`，ULP=3，beyond4=0）。日志
  `logs/diagnostic-run.log`，dump `artifacts/dump-depth-all.f32le` / `artifacts/dump-seg-all.i32le`，
  对比明细 `artifacts/cam2-dump-compare.json`。
  （首次尝试因缺 `CUDA_PATH` 报 `NvrtcUnavailable`、0.43s 未编译/未跑内核，属环境缺失，已记录。）
- **关键旁证**：本次 dump（LIFO 版）depth sha256 = `9829215335d2cb2fd22b02a4d4a8354cc1a5db80
  c806b94a8888ef632c717326`、seg sha256 = `c4be985324b883abcbaa63fa44888b016fd28d2d266858aa
  a493877b49a64185`，与 f32-fix 两跑（heap 版）**完全相同** ⇒ LIFO 改动在本 fixture 上输出
  **位级不变**（零回归且零收益），即候选顺序没有改变任何胜者。

## 2) 真实产品树 dump（失败根因）

用产品自身 CPU 代码（`SceneState::bounds` + `one_dimensional_scene_leaves` + `build_scene_bvh`，
临时 CPU-only test，跑后已 `git checkout --` 还原；源码片段 `scripts/dump-scene-bvh-test.rs.txt`）
dump 出场景 BVH（15 节点 / 8 叶），逐字输出见 `artifacts/real-tree-dump.txt`：

```
节点: 0 leaf0, 1 leaf5, 2 leaf4, 3 leaf6, 4 int(2,3), 5 int(1,4), 6 leaf7,
      7 int(5,6), 8 leaf1, 9 leaf2, 10 leaf3, 11 int(9,10), 12 int(8,11),
      13 int(7,12), 14 int(0,13)
叶: 0 geom0(plane), 1 geom1(box), 2 geom2(mesh), 3 geom3(capsule),
    4 flex0-edge0, 5 flex0-edge1, 6 flex0-edge2, 7 flex1-edge3
```

41094 的候选访问序（CPU 精确复放 `product_ray_aabb` + 动态 bound 剪枝，
`scripts/actual_tree_replay.py`）：**LIFO = heap = `[leaf4(edge0), leaf5(edge1), leaf0(plane)]`**
→ 两种遍历 edge1 都最后访问 → edge1 胜（`0x400015ba` = 产品值）；参考（上游 warp 树 + LIFO）
最后访问 edge0 → `0x400015b7`。=> 只改遍历不可能修该像素。

## 3) `fused_mul_add` 更正（此前可行性结论错误的原因）

- 产品 `src/render/bvh/aabb.rs::compute_leaf_aabb` 对 box/mesh 的角点用 `fused_mul_add`
  （与冻结上游 GPU FMA 收缩对齐，见该文件测试 `fused_mul_add_matches_frozen_reference_rounding`）。
- 早前用 Python 复刻 `builder.rs` 时对叶 bounds 用普通乘加 → leaf1(box)/leaf2(mesh) 的 bounds
  末位与产品不同 → 12 箱 SAH 切分点不同 → 复刻出**错误树**（flex 子树被错排成
  `node(node(leaf5,leaf6),leaf7)`，产品真实为 `node(leaf5,(leaf4,leaf6))`）。
- 因此「产品树 + LIFO → edge0 最后访问」的可行性结论**不成立**，已按真实树更正。
  早期错误树复刻脚本保留为 `scripts/rca.py` / `scripts/tie_scope.py`（仅历史参考，勿用于结论）。

## 4) 未实现项（用户决策，供 r4 占位卡启用）

- 让该像素逐位等于参考的唯一已知窄路：**flex-vs-flex 容差内平局改「严格更小 d」稳定键**
  （仅 `hit.geom_id == -2 && candidate_id == -2`）。真实树精确复放：41094 → `0x400015b7`、
  31814 不变，heap/LIFO 两序结果相同；全量 654 flex 像素 min-d == 参考 **654/654**、tie 事件 2/2。
- 该规则是对上游「容差内后访问者胜」的**已知语义偏离**，不宣称上游等价；是否采用需用户签，
  且要 QA 复核 + attempt5 前全量位级回归（r4 占位卡）。

## 5) 证据清单

| 路径 | 说明 |
| --- | --- |
| `artifacts/lifo-experiment.diff` | LIFO 遍历实验完整 diff |
| `artifacts/lifo-gate-runs/*.json` | LIFO 版四门禁回执（r3 runs，随 r3 abandon 转存） |
| `logs/lifo-verify-run.log` | LIFO 验证短跑日志（cam2 仍 (1,0)） |
| `logs/diagnostic-run.log` | 诊断微跑日志（env-gated dump） |
| `artifacts/dump-depth-all.f32le` / `artifacts/dump-seg-all.i32le` | 诊断微跑 dump（全相机） |
| `artifacts/cam2-dump-compare.json` | dump vs 参考逐相机对比 + 41094 明细 + 位等同旁证 |
| `artifacts/real-tree-dump.txt` | 产品真实树逐字 dump（节点表 + 8 叶 bounds bits） |
| `scripts/actual_tree_replay.py` | 真实树重建 + 两种遍历序 + last-visit/min-d 精确复放 |
| `scripts/dump-scene-bvh-test.rs.txt` | 临时 CPU-only dump test 源码（参考；未编译） |
| `scripts/rca.py` / `scripts/tie_scope.py` | 早期错误树复刻脚本（历史参考，见 §3） |
| `manifest.txt` | sha256 + bytes + 相对路径（不自列） |

## 6) 纪律记录

- **零代码改动**：`src/` 与 `fixtures/` 相对本任务 start 修订未变（卡检查 `unchanged`）；
  实验实现已回滚，`render_common.cu` 与 HEAD 一致。
- 未提交 LIFO/min-d 实现、未改判据/容差/参考/fixtures、未开验收窗口、未重冻。
- GPU 两次短跑均单次、跑前三点基线、前后群报、失败未重试（首次 `NvrtcUnavailable` 为环境
  缺失、未产生内核执行）。
- 与 attempt5：本卡不改变 attempt5 状态；`(134,128)` 仍按用户决策（allowlist / min-d / 记限）。
