# B3 验证窗 attempt2：全绿（本片最后有效轮）

记录时间：2026-10-26 22:18–22:19（+08:00）。与 attempt1（aborted，见
`window-attempt1-invalid.md`）分开记录、不拼接。

## 1. 窗口事实

- 开窗前基线（3 点，原文）：`5215, 10847, 34, 31, 596.36` / `5215, 10847, 33, 31, 596.36` /
  `5215, 10847, 35, 31, 596.36`；free 10847 ≥ 10240；`compute-apps` 无本任务 CUDA 进程。
- 重签集合（窗口内逐字节核对，窗口内零编译）：
  branches `25410b3d46a62a66c46e08269aa74201e542d8be9864cd36e140f76bffc74bf5`、
  plane `115b5c9907aad60e73faec44819887c4b8950a24cde83c12d7a67cffd99a1207`、
  B2 `280f2e47944b5a01fc5bc7a834cb8a77f66d7a28c854fc359d4b540ca888f0e2`、
  primitive `ec56ff7448e91fef32c955cedc7c3cc868663c2d28ec3aa802ee105dcf81fd15`、
  launcher `84ee607479e44c7dbf4cc3dfb5b204252b83b0f8ccf244a83cd2239918d97836`。
- `window_start=2026-10-26T22:18:38.121+08:00`、`window_end=22:19:09.213`，body wall 26.5 s。
- 采样（0.25 s，103 samples）：`used_peak=5242`、`free_min=10820`、`util_max=45`、
  `delta_peak=22 MiB`（limit 3072）、`util_streak_max=0`、`residual=0`、`process_exited=True`；
  guard post / 聚合断言无 abort。
- post 快照 `5215,10847,9,32`；结束基线 3 点 `5215,10847,9,32 / 5215,10847,8,32 / 5215,10847,9,32`；
  post `query-compute-apps` 无本任务进程；5 个 run 的 stderr 日志均 0 行。
- 结果：5/5 exit 0（branches 22.1 s、plane 25.4 s、B2 25.4 s、primitive 25.3 s、launcher 26.1 s）。

## 2. 6 分支 case（对照 B3 GPU 参考，`nacon` 按 GPU 参考）

| case | 走的分支 | nacon | failures |
| --- | --- | --- | --- |
| `plane-mesh-graph` | plane_convex graph hillclimb | 4 | 0 |
| `sphere-sphere-concentric` | sphere_sphere `dist==0` fallback | 1 | 0 |
| `sphere-capsule-clamp` | sphere_capsule 投影 clamp | 1 | 0 |
| `sphere-cylinder-cap` | sphere_cylinder 端面 cap | 1 | 0 |
| `sphere-cylinder-corner` | sphere_cylinder 角点 corner | 1 | 0 |
| `capsule-capsule-nonparallel` | capsule_capsule 非平行 det | 1 | 0 |

判据：位级优先；非位级 dist ≤4 ULP@自身量级（ref=0 → 4 ULP@1.0）、pos 4·ULP(scale)、
frame 4 ULP@1.0；count/结构零容差；efc 全 -1、overflow 全 0；按 `geomcollisionid` 配对。
`failures=0` 表示全部 case 结构零容差通过，且浮点字段全部落在位级优先/ULP 判据内。

## 3. 4 个 swap 变体（调用方高类在前）

| case | swap 入参（type1/type2） | nacon | failures |
| --- | --- | --- | --- |
| `plane-mesh-graph` | (7,0) | 4 | 0 |
| `sphere-capsule-clamp` | (3,2) | 1 | 0 |
| `sphere-cylinder-cap` | (5,2) | 1 | 0 |
| `sphere-cylinder-corner` | (5,2) | 1 | 0 |

`contact.geom` 顺序：swap 变体的结构零容差比较覆盖 `geom` 列（规范化后应为 `[0,1]`），
**逐 case failures=0 → 顺序与数值同时通过**；未出现「顺序不符」或「数值不符」消息
（`sphere-sphere` 两 geom 同类型，按 pinned `_add_geom_pair` 不交换，无 swap 变体）。

## 4. 13 组合位级回归（重签集合，同窗并行）

| 目标 | 结果 |
| --- | --- |
| B1 plane 6/6 | `plane-sphere/capsule/ellipsoid/cylinder nacon=1`、`plane-box nacon=3`、`plane-mesh nacon=3`，全部 `failures=0` |
| B2 sphere/capsule 4/4 | 4 case `nacon=1`，全部 `failures=0` |
| primitive 3/3 | `sphere-box-shallow` / `sphere-box-deep` / `capsule-box`：`bit_equal=80/80`、`max_dist_ulp=0`、`max_pos_ulp=0`、`max_frame_ulp=0`、`failures=0` |
| launcher 3/3 | `hfield-sphere`（314/314）、`sdf-box`（3122/3122）、`ellipsoid-box`（80/80）：`max_*_ulp=0`、`failures=0` |

结论：既有 13 组合在 swap 规范化后**位级无回退**。

## 5. 日志

`target/g10-p5-primitive-branches-verify/attempt2/`：`baseline.txt`、`samples.csv`、`guard.log`、
`facts.txt`、`run-*.log`（stderr 全 0 行）。
