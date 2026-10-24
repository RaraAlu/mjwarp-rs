# U038-3D 原生 f64 两级判定补判（native_error_v2）

任务：`2026-10-24-u038-native-two-reference`。日期：2026-10-24。执行：Backend。
范围：只读原生参考生成与判定；**未改产品源码/测试/容差/断言/fixtures 语义**；
G01 排除清单未动（若要改排除口径需另报 Chief + 用户批准）。

## 1. 结论（先给）

1. **按 `native_error_v2` 重判：9/9 字段块全部「原生放行」**。逐块 `E_candidate`
   在 **0.0034–0.0129** 之间（远小于 1，即全部落在原生容差内），因此按合同
   「原生达标即放行该项 NP03」——**二级参考未使用**（合同：原生通过项无需第二参考）。
2. **放行类别**：`native_passed` / 「原生放行」；`np03_admitted=true`，
   覆盖 **1435 个比较标量 / 9 个字段块**。这是本任务记录的字段块级准入，
   **不是完整 U038、整链、或完整产品精度验收**。
3. **上游逐位 2e-6 严格口径的历史事实原样保留**：全链 1477 项中
   **8/77 `qfrc_gravcomp` 超差、max 4.291534423828125e-6、总位差 122**（容差与断言未改）。
   它不是本次 `native_error_v2` 的准入依据；**不得**把本次放行写成上游逐位对齐。
4. **原生参考与冻结上游场景严格对齐**（自检，非反推）：原生 f64 → f32 与冻结上游
   f32 原始位逐项比较，运动学字段 max_abs ≤ **2.04e-7**、`qfrc_gravcomp`
   max_abs ≤ **1.01e-5**（f32 舍入量级）；两次运行逐位一致；无 NaN/Inf。
5. **上一任务（debt-2）的「最小改动不可对齐」结论仍然成立**：它描述的是
   与冻结上游逐位 2e-6 口径；本任务按用户重申的原生优先规则给出的是
   原生放行判定。两者并存、不互相改写。

## 2. 判定口径（合同）

依据 `docs/numerical-precision-contract.md`：`native_error_v2`（142–199 行）
与 NP03 原生优先顺序（326–339 行），以及测试族容差表（373–374 行）：

```text
t_i         = atol_native + rtol_native * abs(native_i)
E_candidate = max_i abs(candidate_i - native_i) / t_i
E_mjwarp    = max_i abs(mjwarp_i    - native_i) / t_i
redline     = 1.05 * E_mjwarp
```

| 条件 | 标注 | NP03 放行 |
| --- | --- | --- |
| `E_candidate <= 1` | 原生放行 | 是 |
| 原生超差且 `E_candidate <= E_mjwarp` | 对照放行 | 是 |
| 原生超差且 `E_mjwarp < E_candidate <= redline` | 红线内放行 | 是 |
| 原生超差且 `E_candidate > redline` | 失败不放行 | 否 |

本任务使用的冻结容差（不改动、不放宽）：

| 字段块 | atol_native | rtol_native | 来源 |
| --- | ---: | ---: | --- |
| 运动学八字段（xpos/xquat/xmat/xipos/xanchor/xaxis/subtree_com/cdof） | `2e-5` | `2e-5` | G01 主链与低层：`2e-5 + 2e-5*abs(expected)` |
| `qfrc_gravcomp` | `2e-4` | `2e-4` | G01 重力位证书块 U：U 绝对/相对 `2e-4` |

判定实现与判定代码见 `scripts/authoring/judge-u038-native-two-reference.py`；
公式语义与 `tests/support/two_reference.rs` 的 `native_error_v2` 实现一致
（原生参考 `Reference.values: Vec<f64>`，候选/二级参考为 f32，误差在 f64 中计算）。

## 3. 原生第一参考（离线 authoring，CPU）

- **环境自检**（`authoring.json`）：pinned venv
  `target/mjwarp-parallel/venv`，Python 3.12.13、MuJoCo 3.15.0（native version
  3015000）、NumPy 2.5.4；`sys.prefix` 必须等于该 venv；无 GPU 参与。
- **场景身份**（冻结、只读）：
  `fixtures/gravity-upstream-abi/three-d-kinematics/input.json`（4d33bc44…）、
  `output.json`（743eb543…）、`raw-bits.json`（1c03af97…）、
  `three-d-model-package.json`（42dbe0b8…）、`generated/receipt.json`
  （547f9881…）；上游 `71da24d…`、Warp 1.15.0。
- **同模型/同状态/同量化**：用冻结的模型数组与 qpos 重建等价原生 MJCF
  （body pos/quat、free/ball/slide/hinge 关节、inertial pos/iquat/mass、
  `gravcomp` 属性、`opt.gravity`）；所有输入先按 GPU 侧同样的规则量化到 f32
  （`numpy.float32`）再拓宽为 f64；qvel=0，`mj_forward` 得到 f64 原始值。
  参数周期一致性检查：gravity/mass/gravcomp 各周期行完全相同，故单一原生模型
  可代表全部 7 个 world。
- **`qfrc_gravcomp` 映射**：原生 `mjData.qfrc_gravcomp` 是广义反重力力
  `Σ_b J_bᵀ(-body_gravcomp_b · mass_b · gravity)`，与候选/上游 `qfrc_gravcomp`
  同定义（本判定采用的正是该字段）。`mjData.qfrc_bias` 在 qvel=0 时是**未乘
  `body_gravcomp`** 的重力广义力，因此 `qfrc_gravcomp != -qfrc_bias`
  （除非所有 body 的 `body_gravcomp=1`）；`qfrc_bias` 仅作为映射说明存档，
  不参与判定。
- **输出**：`native-reference.json` 给出每个字段的 **f64 原始值**（主判定参考）
  与 **f32 比较值 + f32 位型**（合同量化规则转出的比较值，仅诊断；两种口径
  状态一致）。
- **独立性**：原生参考只读冻结模型/状态输入，生成过程中不读取任何产品/上游
  输出，不做任何以 GPU 结果为目标的调整；随后判定脚本才读取候选/二级参考。

### 3.1 场景等价自检（原生 f64→f32 vs 冻结上游 f32 原始位）

| 字段 | 比较值 | 排除 | max_abs | f32 位型不同数 |
| --- | ---: | ---: | ---: | ---: |
| xpos | 105 | 0 | 1.457e-7 | 35 |
| xquat_wxyz | 140 | 0 | 1.001e-7 | 49 |
| xipos | 105 | 0 | 1.599e-7 | 34 |
| xmat | 315 | 0 | 2.041e-7 | 159 |
| xanchor | 84 | 0 | 1.615e-7 | 25 |
| xaxis | 42 | 42 | 1.654e-7 | 28 |
| subtree_com | 105 | 0 | 1.897e-7 | 61 |
| cdof | 462 | 0 | 1.982e-7 | 246 |
| qfrc_gravcomp | 77 | 0 | 1.011e-5 | 55 |

- 差异均为 f32 舍入量级（原生 f64 → f32 与上游 f32 计算路径之间的正常差），
  说明原生模型/状态与冻结场景一致；**这些值不是用上游结果反推的**。
- `xaxis` 中 free/ball 关节的 42 个条目被排除（见 §4），排除部分两边差异可达
  0.57，正说明它们不是同一字段。

## 4. 字段块与覆盖（判定前固定）

| 块 | 值数 | 容差 | 说明 |
| --- | ---: | --- | --- |
| xpos / xquat_wxyz / xipos / xmat / xanchor / subtree_com | 105/140/105/315/84/105 | 2e-5 + 2e-5·\|native\| | 完全覆盖 |
| xaxis | **42**（7 worlds × 2 joints × 3） | 同上 | 只覆盖 slide+hinge；free/ball 无原生单一轴语义 |
| cdof | 462 | 同上 | 完全覆盖 |
| qfrc_gravcomp | 77 | 2e-4 + 2e-4·\|native\| | 完全覆盖 |
| 合计 | **1435**（排除 42） | — | 1477 − 42 |

**xaxis 覆盖说明**：`xaxis` 只有对 hinge/slide 关节是单个笛卡尔轴；MuJoCo 对
ball 关节忽略 `jnt_axis`，free 关节也没有单一轴语义，冻结 Warp 参考里这两类
关节的 `xaxis` 只是其内部派生值，与原生 `mjData.xaxis` 不是同义字段。
按合同「缺少同义字段时不得强行比较」，判定前固定排除这 42 项，
**不是**看到结果后再挑元素（不得逐元素择优拼接）。

## 5. 判定结果（逐块）

| 块 | 值数 | E_candidate | E_mjwarp | redline=1.05·E_mjwarp | 状态 | 放行 |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| xpos | 105 | 0.003525 | 0.005196 | 0.005456 | native_passed | 原生放行 |
| xquat_wxyz | 140 | 0.003678 | 0.003678 | 0.003862 | native_passed | 原生放行 |
| xipos | 105 | 0.003515 | 0.005605 | 0.005885 | native_passed | 原生放行 |
| xmat | 315 | 0.006883 | 0.007084 | 0.007438 | native_passed | 原生放行 |
| xanchor | 84 | 0.003366 | 0.005335 | 0.005602 | native_passed | 原生放行 |
| xaxis | 42 | 0.006133 | 0.006133 | 0.006439 | native_passed | 原生放行 |
| subtree_com | 105 | 0.003768 | 0.006650 | 0.006982 | native_passed | 原生放行 |
| cdof | 462 | 0.006883 | 0.006883 | 0.007228 | native_passed | 原生放行 |
| qfrc_gravcomp | 77 | **0.012863** | 0.022348 | 0.023465 | native_passed | 原生放行 |

- `E_candidate <= 1` 对全部 9 块成立 → 全部原生放行，**二级参考未使用**
  （`secondary_reference_used=false`）。`E_mjwarp`/redline 仍逐块报告，仅作诊断。
- `qfrc_gravcomp` 的最大项：index 26（w2 dof4），native=1.26231076，
  candidate=1.26231658，error=5.82e-6，limit=4.525e-4 → 0.012863。
  这与上游严格 2e-6 口径下的 4.29e-6 最大差同源（原生容差 2e-4 按合同执行）。
- **f32 比较值口径诊断**：用 native→f32 重算，E_candidate 0.0035–0.0129、
  状态不变（见 `verdict.json` 各块 `f32_quantized_variant`）。

**汇总**：9 块、1435 标量、native_passed 9、mjwarp_passed 0、failed 0、pending 0；
`np03_admitted=true`，`admission_label=原生放行`。

## 6. 与历史严格口径的关系（不得混写）

| 口径 | 结果 | 用途 |
| --- | --- | --- |
| `native_error_v2`（本任务） | 9/9 块原生放行（1435 标量） | 本任务 NP03 字段块准入 |
| 上游逐位 2e-6 严格口径（历史） | 1477 项 / 8 超差 / max 4.29e-6 / 位差 122 | 原样保留；不是本次准入依据 |

- 本次放行**不表示**上游逐位对齐、也不表示上游严格口径由失败变通过；
- 原生放行也不表示物理更精确，只表示候选相对原生 f64 的归一化误差在冻结原生
  容差内（合同：三类放行具有同等效力，各自标注依据）；
- debt-2 的「不改变产品数值/最小源码改动无法达到上游逐位 0 超差」结论不变；
- **G01 排除清单未动**（U038-3D known-debt 项仍在排除清单）；若要据此调整 G01
  排除口径，另报 Chief 走用户批准，不在本任务内改。

## 7. 未满足项与边界

- 只判定本任务**已记录字段块**（U038 3D 链 9 块/1435 标量），不宣称完整 U038、
  完整三维状态空间、完整产品精度或 NP01–NP04 全量验收。
- 原生参考为 MuJoCo 3.15.0（pinned authoring venv）f64 CPU；产品与上游为 f32；
  工具链与平台差异如实记录，本轮不宣称与历史 MuJoCo 3.12.0 原生参考同版本。
- `qfrc_gravcomp` 的原生容差取合同重力位证书块 U（2e-4），不是 2e-6；
  上游严格 2e-6 口径未改。
- 未改产品源码/测试/容差/断言/fixtures 语义；未跑 GPU、G01 全量或 Linux/release。
- 未创建 `docs/development-evidence/changes/` 回执：任务文件范围只含
  `docs/development-evidence/u038-3d/` 与 `scripts/authoring/`；回执信息在
  `verdict.json`（含 task_id）与 `authoring.json`。

## 8. 复现

```powershell
# 一条命令：pinned env 自检 → 原生参考生成 → native_error_v2 判定
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\authoring\author-u038-native.ps1

# 或分步（同一 pinned venv）
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts\authoring\author-u038-native.py
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts\authoring\judge-u038-native-two-reference.py
```

判定输入固定并校验 sha256：`native-reference.json`、debt-2 的产品/上游原始位
（`product-fields-raw.json`、`product-gravity-raw.json`、`upstream-gravity-raw.json`、
`run.json`）与冻结 `raw-bits.json`；任一漂移即中止。

## 9. 文件

| 文件 | 内容 |
| --- | --- |
| `native-reference.json` | 原生 MuJoCo f64 CPU 参考：9 字段 f64 原始值 + f32 比较值/位型、qfrc_gravcomp、qfrc_bias 映射存档、provenance、环境、等价自检 |
| `authoring.json` | 环境自检、冻结输入哈希、参数周期一致性、确定性、等价自检、脚本哈希 |
| `verdict.json` | `native_error_v2` 判定：9 块 E_candidate/E_mjwarp/redline/状态/最坏项、覆盖、汇总、严格口径历史记录 |
| `../../scripts/authoring/author-u038-native.py` | 原生参考生成（473 行，只读冻结场景） |
| `../../scripts/authoring/judge-u038-native-two-reference.py` | 两级判定（409 行，只读三路证据） |
| `../../scripts/authoring/author-u038-native.ps1` | pinned env 自检 + 顺序执行 |
| `../known-deviation.md` | 已追加本次两级判定结论与放行类别 |
