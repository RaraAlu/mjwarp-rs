# NP01 A0：E 指标三候选重算（离线、只读、零 GPU）

任务：`2026-11-04-precision-np01-e-recompute`（Chief 批准排程：A0 + B1 并行；A0 出证据后连同三候选报 Chief，由 Chief 发用户决策卡）。

脚本：`scripts/authoring/np01-e-recompute.py`（离线 Python，只读输入，不启动产品链、不启 Python 进产品测试）。
输出：同目录 `e_recompute.json`（**本 README 的所有数字均由该脚本产出，没有一个手填**）。

复跑命令：

```powershell
& target\mjwarp-parallel\venv\Scripts\python.exe scripts\authoring\np01-e-recompute.py
```

## 一、口径与硬事实（先看这个）

- 公式严格按契约：`t_i = atol + rtol*|native_i|`，`E = max_i |cand_i - native_i| / t_i`，`redline = 1.05 * E_mjwarp`。
- 输入全部复用已归档证据，没有新采样：`qa-native-f64.json`（native f64 接触数组）、`qa-upstream-gpu.json`（上游 warp CUDA f32）、`qa-compare-gpu.json`（QA 的 native-vs-warp 边界材料）、`g10-p5-gpu-kernels/gpu-window/result.json`（设备 vs 上游聚合）。
- **三个决定性事实**：
  1. **hfield 两 case 不可配对**：native 每 world 接触数 6/6、6/3，warp 8、7 → 计数不等，且 HFIELD 没有用户签字分档。native 口径天然只能覆盖 `ellipsoid-box`（1:1）与 `sdf-box`（40:40）。
  2. **`sdf-box` 配对不可靠**：40:40 走位置最近邻贪心，配对距离中位数 > 最近邻间距 p50 的一半（脚本判 `pairing_reliable=false`）→ 该 case 的 native 口径结论只能作诊断。
  3. **设备原始接触数组未归档**：归档只有「设备 vs 上游」的聚合（bit_equal 数 / max_abs / max_ulp），所以 `E_candidate` 只能给**上下界**（`|dev-native| ∈ [|up-native|-e, |up-native|+e]`，`e=max|dev-up|`），脚本逐字段标注界，不冒充精确值。

## 二、候选②：native f64 作参考（重算结果）

| case / 路径 | 字段 | E_mjwarp | redline | E_candidate 界 | 判定 |
| --- | --- | --- | --- | --- | --- |
| ellipsoid-box / CONVEX | dist | 0.3921 | 0.4118 | [0.3898, 0.3985] | 界内 |
| ellipsoid-box / CONVEX | pos | **431.1** | 452.6 | [431.1, 454.6] | **界跨红线，归档数据不可判定** |
| ellipsoid-box / CONVEX | frame | **585.3** | 614.5 | [585.1, 585.4] | 界内 |
| sdf-box / SDF | dist | 4.288e4 | 4.503e4 | [4.228e4, 4.463e4] | 界内（配对不可靠，仅诊断） |
| sdf-box / SDF | pos | 1.257e5 | 1.320e5 | [6.691e4, 7.343e4] | 界内（同上） |
| sdf-box / SDF | frame | 3.243e4 | 3.405e4 | [3.152e4, 3.152e4] | 界内（同上） |

关键读法：

- **绝对量级**：`E_mjwarp` 在 pos/frame 上达数百到 1e5，说明**分档 atol（CONVEX 2e-6 / SDF 5e-5）不是 native 口径的容差**，而是 upstream-f32 口径的容差——这正是契约 2026-10-27 已写明的「E 用上游f32作产品代理」的含义。
- **相对红线**：设备与上游对 native 的偏差比例≈1（见 `redline_ratio`：ellipsoid dist 0.994–1.006、pos 0.99999–1.00001、frame 0.99968–1.00032；sdf 精确 1.0），因为设备与上游几乎同样地偏离 native。**相对判据会通过、绝对不满足**，两件事必须分开报告（契约要求的正是分开）。
- **判定能力**：即使只算 2 个可配对 case，ellipsoid 的 pos 红线判定仍因缺设备原始数组而跨界不可判定。→ **native 口径在现有归档上无法定稿**。
- 要 native 口径可定稿，至少需要：①补采 native f64 全量接触数组（离线 MuJoCo/CPU，无 GPU）；②补采设备原始接触数组以去掉上下界（小窗口或复用 QA 探针落盘）；③扩样（当前 hfield 不可配对、mesh/multicontact 未涉及）。**按 Chief 风险裁定，这三项都不进 A0 成本。**

## 三、候选①：同码跨后端（warp CPU vs warp CUDA f32）基线

依据 QA 归档 `g6_tolerance_candidates.same_code_backend_baseline`（97 个标量），与契约分档表逐项核对：

| 字段 / 路径 | 已签 atol | 实测 worst（契约 L246-L254） | 余量 | 结论 |
| --- | --- | --- | --- | --- |
| dist / CONVEX | 1e-6 | 9.31e-9 | 107.4× | ≤ atol |
| pos / CONVEX | 1e-6 | 1.86e-8 | 53.8× | ≤ atol |
| frame / CONVEX | 2e-6 | 8.19e-7 | 2.44× | ≤ atol |
| dist / SDF | 2e-6 | 6.88e-7 | 2.91× | ≤ atol |
| pos / SDF | 1e-6 | 4.62e-7 | 2.16× | ≤ atol |
| frame / SDF | 5e-5 | 2.78e-5 | **1.80×（最紧）** | ≤ atol |

注意两点，不能含糊：

1. QA 的 `same_code_backend_baseline` **不按路径拆分**（`frame.max=2.78e-5` 实际是 SDF 最坏值），所以上表的分路径 worst 取自契约 L246-L254 的「实测 worst」列；baseline 只作聚合交叉核对（脚本里 `baseline_not_split_by_path` 如实记录）。
2. 以该基线作参考时 `E_mjwarp ≡ 0`，`1.05×E_mjwarp` 退化。**定稿必须把判据写成「实测 worst ≤ 已签 atol」的核对，而不是 E 比值**——否则红线恒成立、等于没有判据。

## 四、候选③：维持现状（E 不定稿）

- 契约零改动；接触字段只保留 upstream-f32 判据（向量尺度 ULP / frame 绝对 ULP / 结构零容差），分档表只作 native 侧诊断。
- 风险：native 口径没有数值判据、只剩相对红线；若有人把分档表当 native 判据读会误判。

## 五、A0 结论（给 Chief / 用户决策用，不含生效值）

1. **候选②在现有归档上不可定稿**：2/4 case 不可配对、1 个可配对 case 的红线判定跨界、绝对 E≫1；要定稿需补采/扩样（不进 A0 成本）。
2. **候选①证据充分**：6/6 项实测 worst ≤ 已签 atol，最紧余量 1.80×（frame/SDF）；但必须把判据改成 atol 核对（E 比值退化）。
3. **候选③零改动但留空缺**：不违反任何已签口径，代价是接触字段 native 侧无判据。
4. 推荐候选①，并把范围写死：CONVEX/SDF 两路径、字段块 dist/pos/frame（normal 并入 frame）、样本 = 同码跨后端 97 标量、**明确不宣称 native 口径**；同时保留契约里 native 侧「仅诊断」的历史语义。
5. 本片**不写任何生效值、不改契约、不改判据**；等用户签后才由后续片落 text。

## 六、边界

- 零 GPU、零产品代码、零判据/容差/fixtures 改动；`scripts/authoring/` 下脚本属离线 authoring，不进产品构建与产品测试。
- 没有新采样、没有新声明：所有数字都能用上一节命令复跑。
- 不宣称 NP01/NP03 通过。E 是否定稿由用户签，A0 只提供候选与影响面。
