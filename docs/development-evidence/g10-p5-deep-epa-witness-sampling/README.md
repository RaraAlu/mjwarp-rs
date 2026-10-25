# 深穿透 EPA witness 采样复核（cylinder-box 24 随机姿态，定性）

- 任务：`2026-10-25-g10-p5-deep-epa-witness-sampling`（P5，定性片）
- 背景：G5 独立复核的发现 2——`cylinder-box` 深穿透 24 个随机姿态 world 中，`x1` 1/24 超 `4·ULP@scale`（max 8.926e-7）、`x2` 3/24 超（max 1.527e-7）；`dist` 无越界（max 4 ULP）、`ncontact` 24/24 相等、`overflow` 全 0。G5 交付的 3 个 witness case（每例 2 world）不在此额外采样面内。
- 做法：**纯 CPU 定性**——只读 QA 已有 24 姿态原始数据（上游 f32 GPU 参考 + QA 设备探针输出）重新独立复算、做偏差分解与归因；**未改产品代码、未开 GPU 窗口、未复制 QA 的 `.bin`、未改任何已 verified 数字/判据**。
- 结论摘要：**不是系统性实现缺陷；基于现有证据不建议登记修正片（口径选择需 Arch→Chief→用户，本片不自行放宽）**；是**单个姿态（world 8）在深穿透下的派生法线差（7.7e-6 rad；机制未定：本片无法区分「不同选面 tie-break」与「同一选面但 EPA 法线数值差/权重不同」）被穿透深度 0.134 m 放大为 ~1.0e-6 m 的 witness 位置差**，`dist`/`ncontact`/接触槽/`overflow` 全部一致；其余 23/24 姿态在界内或边界舍入级。**「选面是否与上游一致」= 未覆盖**（本片无 `face_idx`/三顶点/权重，见 §5）。建议（不改判据，需 Arch→Chief→用户决定）：深穿透 witness 位置不宜用 `4·ULP@scale` 逐分量判据，改用 `dist` + `ncontact`/`index`/`overflow`，witness 位置只作信息量；或把该采样面列为未覆盖。若需终局归因（选面/权重逐项对照），需补 GPU 窗口 + 上游 `face_idx`/三顶点/权重导出（见 §7）。

## 1. 数据源与可重建性

| 项 | 内容 |
| --- | --- |
| 上游参考（f32 GPU） | `docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-witness-gpu.json` → `sweep[case=cylinder-box].results`（24 world 的 dist/x1/x2/ncontact/index/overflow，QA 自采） |
| 设备输出 | 同目录 `qa-gpu/out/cylinder-box-sweep/{dist,x1,x2,ncontact,index,overflow}[-run2].bin`（QA 探针未跟踪产物，**本片只读**；12 个 bin 的 sha256 已记入 `analysis.json.source.device_bin_sha256`） |
| 姿态生成方法 | QA `qa_prepare_capture.py`：`np.random.default_rng(20251025)`；base qpos + 随机轴角（`angle∈[-1.4,1.4]`）+ free joint 位置 `uniform(-0.08,0.08)`×3，共 24 world |
| 判据 | G5 现行口径：`dist` 按自身量级 ≤4 ULP；`x1`/`x2` 用 `scale=max_i|ref_i|` 的 `4·ULP(scale)`；`ncontact`/`index`/`overflow` 位级 |
| 独立复算 | `analyze_deep_epa.py`（本片新增，纯 CPU）→ `analysis.json`（摘要/越界详情/交叉核对/数据源 sha256）+ `analysis-worlds.jsonl`（24 world 明细，每行一个）+ `analysis.log`（逐 world 人读表） |

**与 QA 汇总的交叉核对**：本片独立重算的 9 项计数/极值（`x1_bad=1`、`x2_bad=3`、`dist_bit_equal=17`、`dist_max_ulp=4`、`dist_bad=0`、`ncontact`、`index` 配对、`overflow`、两次运行位等）与 `qa-compare-gpu.json` 的 `witness_cases.cylinder-box-sweep` **9/9 一致**（`analysis.json.cross_check_with_qa_summary` 全为 true）。

## 2. 复算结果（24 world）

| 指标 | 结果 |
| --- | --- |
| `dist` | 位等 17/24，max **4.0 ULP**（恰在界内 0 越界） |
| `x1` 越界 world | **1 个：world 8**（max abs 8.926e-7 = 29.95× 界、119.8 ULP@scale） |
| `x2` 越界 world | **3 个：world 8、10、15**（max abs 1.527e-7 / 5.122e-9 / 2.235e-8） |
| `ncontact` / `index` / `overflow` | **24/24 / 24/24 相等 / 全 0** |
| 两次设备运行 | **逐位相同** |
| 穿透深度范围 | 0.0803–0.1470 m（中位 0.1101，**上中位数口径**=排序第 13 小；两中位平均口径为 0.1089，偏差 1%，不影响结论） |
| 越界 world 的深度 | world 8：0.1342；world 10：0.0882；world 15：0.0996（不集中最深端；最深 world 5=0.1470 偏差为 0） |
| 深度相关性（Pearson） | `x1` 比值 0.247、`x2` 比值 0.185、`x1` 法向分量 0.090、`x1` 切向分量 0.251 → 该样本下**无显著线性深度趋势**（n=24，双侧 p≈0.244/0.387/0.677/0.237，均不显著；样本量限制见 §5） |

逐 world 表见 `analysis.log`（含每个 world 的 x1/x2 最差分量、界、比值、法向/切向分量、`index` 相等、法线角差、witness 自洽残差、预测比）。

## 3. 归因（逐条排除 + 定量证据）

**world 8（唯一明显越界）关键数字**（`analysis-worlds.jsonl` 第 9 行 / `analysis.json.summary.over_bound_details`）：

- `x1` 偏差 `Δx1=[5.998e-7, -8.926e-7, -2.384e-7]`（|Δx1|=1.101e-6），相对上游派生法线分解：**切向 1.101e-6、法向 2.348e-8**（法向分量在 x1 界 2.98e-8 内）。
- `x2` 偏差 `[1.527e-7, 1.118e-8, -3.725e-9]`（|Δx2|=1.531e-7），box 侧点几乎不动。
- 设备由自身 witness 反解的派生法线与上游差 **δθ=7.71e-6 rad（0.000442°）**；`|dist|·δθ = 1.0348e-6`，与实测点对差 `|Δx1-Δx2|=1.0352e-6` 的比值 **1.00043** —— 即 witness 点对差 ≈ 穿透深度 × 法线微小转动（误差 0.04%）。**注意这是定义自洽分解**（法线由 `(x2-x1)/dist` 反解、`Δdist` 仅占点对差的 0.011%）：它证明「点对差等价于一个微小派生法线转动」，**不能判定哪一方正确**，也不能区分「不同选面 tie-break」与「同面但 EPA 法线数值差/权重不同」（QA 独立复核 `2c052637` 观察 1）。
- `dist` 只差 **1 ULP**（-1.49e-8）；`ncontact=1/1`、`index` 相等、`overflow=0`；设备两次运行逐位相同。
- witness 自洽残差 `|(x2-x1)-dist·n|/|dist|` = **1.35e-6**（其余 world ≤2.1e-7），与「同一公式、仅选点/面不同」一致。

**worlds 10/15（边界舍入级，不构成明显偏离）**：`x2` 超界 1.375×/1.5×，绝对差 5.12e-9 / 2.24e-8（1–3 ULP@scale）；`x1` 在界内（0.25×/0.81×）；`dist` 0 / 4 ULP。

**逐条排除**：

1. **EPA horizon/迭代退化（溢出分支）**：`overflow` 24/24 为 0、`ncontact` 24/24 相等、`dist` ≤4 ULP 且非 `FLOAT_MAX`、无强制 horizon 路径 → 排除。
2. **witness 提取公式/存储错误**：设备自洽残差 ≤1.35e-6（world 8）且其余 ≤2.1e-7；若公式/索引错，残差或点对关系会系统性破坏 → 排除；偏差可由「穿透深度 × 派生法线差」定量解释（1.00043）。
3. **f32 累加顺序随机性**：设备两次运行逐位相同；偏差是单姿态确定性大偏离（119.8 ULP@scale），不是全姿态随机散布 → 排除随机舍入；也不符合累加顺序缺陷（后者会影响同一场景所有姿态）。
4. **深穿透 `multiccd_idx`/接触槽选择**：`index` 24/24、`ncontact` 24/24 相等，`overflow` 0 → 同一接触槽/特征，仅 witness 位置不同 → 排除槽位/特征选择错误。
5. **系统性偏差**：24 中仅 1 个明显越界、2 个边界舍入级；越界与深度无显著线性相关（Pearson ≤0.25，n=24 双侧 p≈0.24–0.68 均不显著）；最深 world 偏差为 0 → 非系统性（受样本量限制，仅说明该 24 姿态样本下无显著趋势）。

**归因结论**：world 8 是深穿透（depth 0.1342）下的**派生法线差（机制未定）**：两实现的派生接触法线相差 7.7e-6 rad，在深穿透放大下使 witness 点对相差 ~1.0e-6 m（约为几何尺度 0.08–0.1 m 的 1e-5）。这是接触流形内 witness 位置的不确定度，**不是系统性实现缺陷**；但本片**不能**区分「不同选面 tie-break」与「同一选面但 EPA 法线数值差/权重不同」，也**不能**断言「设备选面完全等价于上游」——本片没有 `face_idx`/三顶点/权重原始对照（见 §5），只能给出定量分解与「不支持系统性缺陷修正片」的建议。

## 4. 结论与建议（不改判据）

1. **是否需要修正片**：基于现有证据**不需要**（建议，最终口径需 Arch→Chief→用户）。没有可定位的公式/索引错误；没有系统性偏差；偏差集中在单姿态且无显著深度趋势。若要以「与上游 witness 逐位对齐」为目标，需要对齐 EPA 选面 tie-break（**若机制确为选面；本片未判定**；代价高、收益仅此额外采样面的 1/24 姿态），不建议为本片登记修正片。
2. **最小复现姿态**：QA case `cylinder-box-sweep` 的 **world 8（0-based）**；姿态生成方法/种子见 §1；上游参考值、设备值、偏差分解见 `analysis-worlds.jsonl` 第 9 行（含 `ref`/`dev`/`delta`/派生法线/预测比）。
3. **判据影响面**：G5 交付的 3 个 witness case 与交付接触数组判据不受影响（本片只分析额外采样面）。若未来声明面要覆盖「深穿透 witness 位置」，现有 `4·ULP@scale` 逐分量判据不适用：建议用 `dist` + `ncontact`/`index`/`overflow` 作为判据，witness 位置（`x1`/`x2`）只作信息量；或显式把该面列为未覆盖。**任何判据/声明面调整都需 Arch→Chief→用户决定，本片不自行放宽。**
4. **本片不宣称**：不表示精度/等价/放行结论；G5 交付结论不变。

## 5. 未覆盖 / 限制

- **无 `face_idx`/EPA 三顶点/权重导出**：设备探针与上游 witness 捕获都只导出 `dist/x1/x2/ncontact/index/overflow`，无法做「同一 world 选面逐项对照」；本片用派生法线 + 深度放大关系做定量归因，若要终局判定选面是否与上游一致，需补 GPU 窗口（设备/上游均加导出）。
- **witness 捕获无 `frame`/`normal` 字段**：无法直接判断该姿态在接触数组 `frame`/`normal` 判据下是否越界；本片只讨论 witness 位置面。
- QA 设备输出为未跟踪产物：本片只读、不复制 `.bin`（sha256 已记）；若 QA 产物被清理，`analysis.json` 中的派生数字仍可复核，但不能重跑原始比对。
- 未改 S2–S8 证据、G5 已验收数字、判据/容差、fixtures/Cargo；未开 GPU 窗口、未改产品代码。
- **机制未定**：本片量化的是「派生法线差 + 穿透深度放大」，无法区分「不同选面 tie-break」与「同一选面但 EPA 法线数值差/权重不同」；「选面是否与上游一致」保持未覆盖（QA 复核 `2c052637` 观察 1）。
- **样本量限制**：24 姿态、单 case（`cylinder-box`，QA 自造姿态）；Pearson 四值 n=24 双侧 p≈0.244/0.387/0.677/0.237 均不显著，只能说明「该样本下无显著线性趋势」，不能外推为「与深度无关」（QA 观察 3）。

## 6. 文件

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文件（方法、复算结果、归因、结论、限制） |
| `receipt.json` | 任务回执（task_id、数据源、复算与交叉核对、归因、gate） |
| `analysis.json` | 摘要与派生数字：越界详情、相关性、数据源 sha256、与 QA 汇总交叉核对 |
| `analysis-worlds.jsonl` | 24 world 完整派生字段（每行一个 world，第 9 行 = world 8） |
| `analysis.log` | 人读逐 world 表（偏差/界/比值/法向切向分解/法线角/预测比） |
| `analyze_deep_epa.py` | 纯 CPU 分析脚本（只读 QA 数据，不占 GPU） |
