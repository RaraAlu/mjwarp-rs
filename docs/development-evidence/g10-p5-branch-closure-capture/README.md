# S3 · UNC-3 分支收口：新 fixtures + 上游参考捕获窗

- 任务：`2026-10-27-unc3-branch-closure-s3-capture`（方向③「补未覆盖分支」第 3 片）
- 上游 pinned revision：`71da24d956378a87a703b6e1442b13aec0c4ac29`
- 上游环境（capture 脚本自采 provenance）：python 3.12.13 / mujoco 3.15.0 / warp 1.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4；`device=cuda:0`；GPU NVIDIA GeForce RTX 4070 Ti SUPER（16376 MiB，driver 596.36，cc 8.9）
- 本片只做 **fixture + 上游参考捕获**；数值验收（设备 vs 参考）在 S4 验证窗，本文不写「窗口通过」。

## 1. 签署记录

- **Chief 09:30 独立复核 16 sha 一致后签窗**（13 fixture + capture 脚本 + runner + compare；wall ≤90 s、预算 238 MiB 量级、floor 公式、零 Rust 编译）。
- **Chief 09:41 复算 samples=134 / delta 283 / free_min 12930 / util 94×2 / 旧参考 11 文件未动**，ack attempt1 = 首个有效轮、无 abort，放行写证据。

## 2. 签名集合（开窗前冻结；runner 开窗时逐项复核 → `SHA_OK fixtures=13`）

| 项 | 路径 | 字节 | sha256 |
| --- | --- | --- | --- |
| fixture | `fixtures/g10-p5-primitive-branches/plane-cylinder-degenerate.xml` | 429 | `393ff1af9811b9badf6e9c13e707fa9a843e2b1d6f3face4f98b2cc7c6157583` |
| fixture | `…/plane-box-corner-face-down.xml` | 425 | `b158ce87c6fd68b79306823cd23f0e778482c425b5f617fcfbbd67fba9b22cab` |
| fixture | `…/plane-box-corner-face-up.xml` | 438 | `0223d2a925fde62ca114b57b3baf797b40654de25d74f943e9d9f716ed7c70b7` |
| fixture | `…/plane-box-corner-tilt-x.xml` | 481 | `2d9d360c6c87b53fd7d7a03d8889bdceb947c15982d2062052218efe4dbe3e8f` |
| fixture | `…/plane-box-corner-tilt-y.xml` | 481 | `159df60fc97aa53beb2df325fdebcc9327d22bbdef354f4d01bece8e460be8c5` |
| fixture | `…/plane-box-corner-single.xml` | 519 | `fc9f710c9accb679f65b98d6484caeaab91d2e05c5f7c3f59f03bd04691ecc1c` |
| fixture | `…/plane-capsule-lying.xml` | 473 | `9d43b44e8cd53a214bf43aa4958980e764ad8f87c08c74cd7d788879b58cf5c2` |
| fixture | `…/plane-cylinder-deep-tilt.xml` | 479 | `9801a8869fbb67eb2ca401956b66e1620d3e1b92f849ac6754262905fcfc6d84` |
| fixture | `fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml` | 609 | `1d71151364f4ba910308c80a42817f7a9b7fe843e42afb8396a3b5277f1b3bfb` |
| fixture | `…/fan-cone-24.obj` | 1042 | `4bc34a51a60d76469a68c64e6132cfd1f5713072488ea8deefd44a4ca517f73b` |
| fixture | `…/prism-20.obj` | 1819 | `a4477d3b080479e7356ac14bcd67c33272b2771eadd40b14fe3f40360e92cee4` |
| fixture | `…/mesh-limits-cone-box.xml` | 569 | `6ce3552f2cab48fdbbb66a6d397790d18748dcb0a5a87b91186b5742e297f731` |
| fixture | `…/mesh-limits-prism-box.xml` | 567 | `1bee35cfa3378d96d18317ff75bbf4af0561f7bd71915574e2ea596bc8d4f3b5` |
| capture | `scripts/authoring/g10-p5/capture_branch_closure_gpu.py` | 7403 | `71c91ee8c3a9c3c689b09d124e036170e4a0befd86bc658078a9f94c73b0bd86` |
| precheck | `scripts/authoring/g10-p5/precheck_branch_closure_cpu.py` | 8624 | `e26f936ec30b52ad57b18eeb92dba979540a82a0be527ca922e86806645d8387` |
| compare | `scripts/compare_primitive_capture.py`（复用，零改动） | 849 | `27ecb54a199798f5ea00c4accb6db65e65734cd023daa9102bc151557f3ccced` |
| runner | `target/s3-branch-closure/window/run-capture-attempt1.ps1`（不入库） | 8745 | `5f8f80c5c5629bb5567fdbad1bc3f933a9b9dfb01d5e2926fe1b59cf462fe3d2` |

fixture 文件数 13 = 8 个 plane 系列 xml + 2 个 limit wrapper xml（`.obj` 必须经 wrapper xml 才能进模型）+ `mesh-mesh-two-assets.xml` + 2 个 `.obj`；模型/case 数 11（branch 组 9 + limits 组 2）。

## 3. CPU precheck（authoring-only，无 CUDA context）

`precheck_branch_closure_cpu.py` 在 pinned venv 内 `wp.set_device("cpu")` 跑同一批 fixture；结果见 `precheck-cpu.json`（`result=PASS`、`failures=[]`）：

- U1 `plane-cylinder-degenerate`：count=3，退化 x 轴回退点 z 全 `-0.004999998956918716`，dist 全 `-0.009999997913837433`
- U3 角点并集：face-down `[0,1,2,3]` + face-up(Rx180) `[4,5,6,7]` + tilt-x `[0,1]` + tilt-y `[1,3]` + single `[7]` → **8/8**（count 4/4/2/2/1）
- U4 `plane-capsule-lying`：count=2（横躺非退化帧基、双端点）
- U5 `plane-cylinder-deep-tilt`：count=4（4 条候选全写入）
- U6 `mesh-mesh-two-assets`：nmesh=2 / nmeshvert=12 / nmeshpoly=10
- U7 `fan-cone-24`：max_degree=24>10；U8 `prism-20`：max_polygon_vertices=20>16

## 4. 捕获窗事实（attempt1 = 首个有效轮，无 abort）

- 窗口：`2026-10-27T09:37:42.194+08:00` → `09:38:23.688+08:00` = **41.5 s**（目标 ≤90 s）
- A/B 两独立进程、两独立 warp cache：A exit 0 / wall 29.4 s，B exit 0 / wall 7.4 s
- 开窗前 3 点基线：`2905,13157,39,30,596.36` / `2899,13163,36,30,596.36` / `2868,13194,37,30,596.36`；free 13157 ≥ open 门 10240 → **floor = min(10240, 13157−3072) = 10085**
- pre：`2844,13218,23,31`；post：`2844,13218,23,31`；结束基线 3 点：`2844,13218,23/24/25,31/31/30`
- 窗内**零 Rust 编译**：只跑 pinned python（capture A/B + compare）+ nvidia-smi + guard sampler
- 负载 precheck：cargo/rustc/cl/nvcc **0** 进程；CPU 3 采样 `5,6,3`
- compute-apps：28 条全为桌面/系统进程（explorer、Chrome、EdgeWebView、NVIDIA Overlay、Feishu、ToDesk、DeepSeek Harness 等）+ **6 条 `[Insufficient Permissions]`**；按 `python|g10_p5|cargo|mjwarp` 过滤后 = **0**（pre/post 同口径）
- post compute-apps 无本任务残留进程

## 5. 捕获结果（A/B raw 逐字节一致）

| 参考文件 | 字节 | A sha256 | B sha256 | compare | evidence 副本 |
| --- | --- | --- | --- | --- | --- |
| `upstream-branch-closure-gpu.json` | 16932 | `24d89e9248cccece96baff2089954e438ca7956fe8a71ccc61df318208709db1` | 同 | `RAW BYTE-IDENTICAL: PASS`（exit 0） | 同 sha |
| `upstream-mesh-limits-gpu.json` | 4581 | `b6abb8fa5820c103f6d880936d6ae95fa60035e5b9ce49a0364b15de1def19ad` | 同 | `RAW BYTE-IDENTICAL: PASS`（exit 0） | 同 sha |

- 11 个 case **11-11 两次捕获一致**（`two_captures_bitwise_identical=true`）。
- 组语义写死在 capture 脚本：branch 组 = 验收组（U1/U3/U4/U5/U6，进分母）；limits 组 = **record-only**（U7/U8，不进 pass/fail 分母）。

### 5.1 逐 case 结果（上游 pinned GPU 参考值）

| case | 组 | count | 关键数值 |
| --- | --- | --- | --- |
| plane-cylinder-degenerate (U1) | branch | 3 | dist 全 `-0.009999997913837433`；pos z 全 `-0.004999998956918716`；x = 0.06 / −0.03 / −0.03 |
| plane-box-corner-face-down (U3) | branch | 4 | dist 全 `-0.010000001639127731`；pos z `-0.005000000819563866`；角点 `[0,1,2,3]`（角点号来自 CPU precheck） |
| plane-box-corner-face-up (U3, Rx180) | branch | 4 | dist 同上；角点 `[4,5,6,7]` → 与 face-down 并集 **8/8** |
| plane-box-corner-tilt-x (U3) | branch | 2 | dist `-0.010000042617321014`；pos y `-0.00707106152549386`；角点 `[0,1]` |
| plane-box-corner-tilt-y (U3) | branch | 2 | dist `-0.010000184178352356`；pos z `-0.005000092089176178`；角点 `[1,3]` |
| plane-box-corner-single (U3) | branch | 1 | dist `-0.010000184178352356`；pos x `1.1678096711875696e-08`；角点 `[7]` |
| plane-capsule-lying (U4) | branch | 2 | dist 全 `-0.010000001639127731`；端点 x = **±0.12000001221895218**（横躺、`b_norm>0.5` 非退化帧基 + 双端点） |
| plane-cylinder-deep-tilt (U5) | branch | **4** | dist `-0.21660254895687103 / -0.043397463858127594 / -0.1716025471687317 / -0.1716025471687317`（4 条候选全写入） |
| mesh-mesh-two-assets (U6) | branch | 3 | dist 全 `-0.009999997913837433`；`geom_dataid=[0,1]`；nmesh=2 / 12 vert / 10 poly（进分母） |
| mesh-limits-cone-box (U7) | limits | 1 | 模型 max_degree=24>10、max_polygon_vertices=24>16；dist `-0.010000001639127731` |
| mesh-limits-prism-box (U8) | limits | **4** | 模型 max_polygon_vertices=20>16、max_degree=3；dist 全 `-0.009999997913837433`；4 接触 pos (±0.05, ±0.05, 0.045) |

## 6. 采样摘要（`guard.log` + `samples.csv` 重算）

- `samples=134`；`baseline_used=2849`；`used_peak=3132`；**`delta_peak=283`**（guard 增量上限 3072 未触发）
- **delta 三口径**：vs guard baseline used 2849 → **283**；vs 第一基线点 used 2905 → **227**；vs 采样最小 used 2849 → **283**
- `free_min=12930` > floor 10085 ✅；`residual_processes=0`；guard post `delta=-5`；temp_max 31 ℃
- `util_max=94%`，连续 2 样本 ≥80（`util_streak=2` < abort 阈值 3）→ 未触发 abort

## 7. 偏差与推断注记（如实记，不抹平）

1. **显存预算**：报窗时按 M3 量级给 **238 MiB**，本窗实测 **delta_peak=283 MiB**（+45 MiB），仍 ≪3072 增量上限、floor 未触发；报窗估值与实际差 45 MiB 记在此处。
2. **util 峰值**：本窗 `util_max=94%` × 2 连续样本，比 M3 捕获窗的 `80×2` 更贴 abort 阈值（差 1 个样本）。归因「JIT/link 尾峰 + 采样抖动」为 **推断**（seq101/102 两点 used 3100→2850 与 util 94 并存，形态符合尾峰穿越采样点）。
3. **U8 接触来源**：「4 个接触来自 ≤16 顶点的侧面」是 **推断，未证实**（参考 JSON 不含 polygon 号，未在 pinned 侧补打印）。可确认的事实只有：模型级 `npolygonmax=20>16` 时上游 pinned GPU **不整体回退**，给出 count=4。
4. **U7 措辞**：`count=1` **与 degree>10 的限制语义一致**，不写「证明限制触发」。
5. **CPU precheck vs GPU 参考**：count 与 `qpos_bits` **11/11 一致**；dist/pos 位级一致的有 6 个 case，3 个 tilt pose 差 **dist 8–16 ULP / pos 8–48 ULP**（CPU 与 GPU 后端求和顺序不同）；corner-single 的 pos x 为 `1.1678e-8`(GPU) vs `1.4901e-8`(CPU)（绝对差 3.2e-9）。**参考一律取 GPU**；precheck 只作 count / `qpos_bits` 对账，不作数值判据。

## 8. 旧参考零读写

`fixtures/g10-p5/references/` 下 11 个文件 mtime 仍为 **10-25**、字节未变（含 `hfield-capsule.json` 9712 B、`mesh-mesh.json` 7314 B、`sdf-box.json` 62464 B、`box-box-margin.json` 12333 B、`cylinder-box.json` 10443 B、`hfield-sphere.json` 10956 B、`index.json` 4676 B、`sphere-box-deep.json` 6972 B、`sphere-box-shallow.json` 6991 B、`capsule-box.json` 6976 B、`ellipsoid-box.json` 7210 B）；capture 脚本只写两个新文件名，runner 只把这两个新文件复制到本目录，无任何指向旧参考的读写路径。

## 9. 证据文件

| 文件 | 说明 |
| --- | --- |
| `precheck-cpu.json` | CPU precheck（count/qpos_bits/角点并集/限制统计） |
| `upstream-branch-closure-gpu.json` | 上游 GPU 参考 · branch 组（验收组） |
| `upstream-mesh-limits-gpu.json` | 上游 GPU 参考 · limits 组（record-only） |
| `logs/capture-attempt1-baseline.txt` | 3 点基线 + compute-apps 原文 + CPU 负载 + top CPU 进程 |
| `logs/capture-attempt1-facts.txt` | 窗口 facts（run exit/wall、pre/post/结束基线、采样、guard） |
| `logs/capture-attempt1-compare.log` | 两文件 A/B 逐字节比对结果 |
| `logs/capture-attempt1-run-a.log` / `-run-b.log` | A/B 两次独立运行的逐 case 输出与 JIT 日志 |
| `logs/capture-attempt1-guard.log` | guard init/baseline/sample/aggregates |
| `logs/capture-attempt1-samples.csv` | 0.25 s 采样 134 行 |
| `receipt.json` | 机器可读摘要（签名集合、窗口事实、逐 case、采样、偏差、边界） |

日志 sha256（`logs/` 内 7 个文件）：

- `capture-attempt1-baseline.txt` `84e2d7aff01edeafbeba894725d48b9466bf14d19e710e5236181bfd55863a92`
- `capture-attempt1-compare.log` `6f85055904ab6b8f63db213f8a6f9131abf721a0eb3f4449eb089b54e561af8d`
- `capture-attempt1-facts.txt` `3839605bb95cfee0b22dba715d462cb6e5cde41383192d95cf38b8b77d8b234e`
- `capture-attempt1-guard.log` `aad161720430f75ab265c803dead063fdb77fbab8dd8310316acde61ae2e3400`
- `capture-attempt1-run-a.log` `7649bd7dd37c7dc208ac45f06b7437795b128e045f388ededc44761e7abac62c`
- `capture-attempt1-run-b.log` `78729a8b9f5fff4bf57ab418f9c60f8851437a8059d2a7f31f3ced9d2de12279`
- `capture-attempt1-samples.csv` `f7f7980b7e6374c6eb7b544fae7bfa57d12951b4c20d15d961beb54fd0e97813`

## 10. 边界（不宣称项）

- 本片只产出 **fixtures + 上游参考 + CPU precheck + 一次有效捕获轮（guard 无 abort、raw 逐字节一致、11-11 两次捕获一致）**；**不写「窗口通过」**，数值验收在 S4。
- U7/U8 为 **record-only**：不进 pass/fail 分母；S4 只做描述性对照，设备行为若 ≠ 上游（U8 设备若给 1 而参考 4）写「与上游行为不同 / 限制存在」，**不动 fixture、不改判据、不挪代码凑绿**。
- U2 `plane_ellipsoid` 仍为「不可达 / 非上游防御分支」（静态 diff 依据）；U6 的 mesh 一般化只覆盖 nmesh=2 一例；`hfield-capsule` 设备回归仍未覆盖（S4 另行扩 launcher）。
- 改动集：fixtures（新增 13 文件）、抓取/precheck 脚本、本证据目录；产品代码、设备源、判据、容差、旧参考、期望值 **零改动**。
