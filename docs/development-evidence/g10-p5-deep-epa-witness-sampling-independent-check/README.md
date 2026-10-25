# 深穿透 EPA witness 采样片独立复核（cylinder-box 24 随机姿态，纯 CPU）

- 任务：`2026-10-25-g10-p5-deep-epa-witness-sampling-independent-check`（p55，纯 CPU）
- 复核对象：`c44578be`（深穿透 EPA witness 额外采样面定性片，交付证据 `docs/development-evidence/g10-p5-deep-epa-witness-sampling/`）
- 判定：**pass（阻断 0、发现 0、观察 3）**。本复核只读 QA 原始样本独立重算，**未读交付结论再点头**、未开 GPU、未复制 `.bin`、未改任何已 verified 数字/判据。

## 1. 统计独立复算（数据源：QA 原始样本，不采信交付 `analysis.json` 结论）

数据源：上游参考 = QA `qa-witness-gpu.json` 的 `sweep[case=cylinder-box].results`（24 world）；设备 = QA 探针未跟踪输出 `qa-gpu/out/cylinder-box-sweep/{dist,x1,x2,ncontact,index,overflow}[-run2].bin`（f32/i32 原始字节，自己解析，未用交付脚本）。判据沿用 G5 口径：`x1/x2` 用 `scale=max_i|ref_i|` 的 `4·ULP(scale)`，`dist` 按自身量级 4 ULP。

| 指标 | 我的独立重算 |
| --- | --- |
| `dist` | 位等 **17/24**，max **4.0 ULP**，越界 **0** |
| `x1` 越界 | **world 8**（worst_abs 8.925999281927943e-07，界 2.9802322387695312e-08，**29.95×**） |
| `x2` 越界 | **world 8、10、15**（max 1.5273690223693848e-07，10.25× 界） |
| `ncontact` / `index` | **24/24 / 24/24** 相等 |
| `overflow` | 全 0（两侧） |
| 两次设备运行 | dist/x1/x2/ncontact/index/overflow **逐位相同** |
| 穿透深度 | min 0.0803488418、max 0.1470220685（`-ref_dist`） |
| world 8 深度 | 0.1342242956 |

与交付 `analysis.json`/README 的逐项核对（数字从交付 JSON 读取后与我的重算比对，**16/16 一致**）：17/24、max 4 ULP、0 越界、x1_bad=[8]、x2_bad=[8,10,15]、x1 max 8.926e-7、x2 max 1.527e-7、w10/w15 的 x1/x2 具体值、w8 深度、深度范围、ncontact/index/overflow、两次运行位等 —— 全部一致。

## 2. 与 QA 汇总的 9 项交叉核对（独立确认「9/9 一致」）

交付称与本 QA 的 `qa-compare-gpu.json` `witness_cases.cylinder-box-sweep` 9 项一致。我把「我的直算」「QA 汇总 JSON」「交付声明」三方并列：**9 项全部一致**（`x1_bad=1`、`x2_bad=3`、`dist_bit_equal=17`、`dist_max_ulp=4`、`dist_bad=0`、`ncontact` 相等、`index` 配对相等、`overflow` 全 0、两次运行位等），且 `x1/x2 max_abs` 数值也逐位一致。

## 3. world 8 归因复算与「循环论证」评估

用原始 `dist/x1/x2` 按 witness 关系反解派生法线（`n ∝ (x2-x1)/dist`，与交付同口径）后独立重算：

| 量 | 我的重算 | 交付 | 判断 |
| --- | --- | --- | --- |
| 设备 vs 上游派生法线角差 δθ | **7.7095186e-6 rad**（0.0004417229°） | 7.7095e-6 rad / 0.00044172° | 一致 |
| `|dist|·δθ` | **1.0348048e-6** | 1.0348023e-6 | 一致（差异仅来自 `acos` vs `|Δn|` 的一阶近似，相对 2e-6） |
| 实测点对差 `|Δx1-Δx2|` | **1.0352448e-6** | 1.0352448e-6 | 一致 |
| 比值 | **1.000425** | 1.0004276 | 一致 |
| witness 自洽残差（相对） | **1.3465421e-6** | 1.3465421e-6 | 一致；为 24 world 最大，角度差也是 24 world 最大（rank 1） |
| δdist 占比 | 1.11e-7（0.011%） | — | 说明点对差几乎完全由法线转动贡献 |

**但这条等式不能证明根因**（我的判断，构成观察 1）：

- 派生法线本身就是由 `(x2-x1)/dist` 反解出来的，因此 `Δx1-Δx2 ≡ d_dev·n_dev − d_ref·n_ref` 是**定义恒等式**；`|Δx1-Δx2| ≈ |d|·δθ` 在 `δdist` 可忽略（0.011%）时**必然成立**（我实测两式差 2.07e-6，仅来自单位化/残差）。它只说明「两实现的 witness 点对差等价于一个微小的派生法线转动」，**不提供方向性信息**（哪一方更接近真值），也**不能区分**「不同选面（tie-break）」与「同一选面但 EPA 法线数值差/权重不同」。
- 该 world 的残差与角差都是 24 个里最大（rank 1），支持「该姿态的接触法线确实偏离」；但「为什么偏离（选面 vs 数值）」需要 `face_idx`/EPA 三顶点/重心权重等**未导出**的数据（交付 §5 已如实列出该未覆盖）。
- 因此「根因 = 近简并 EPA 选面/派生法线差」中的**「选面」部分超出证据边界**；建议表述为「派生法线差（机制未定）」，并把「选面 tie-break 是否与上游一致」保留为未覆盖项（对 UNC-6 处置的含义见 §6）。

## 4. 结论边界核对（能否被误读成验收结论）

| 检查 | 结果 |
| --- | --- |
| 是否把「不需要修正片」写成可误读的验收结论 | 否：README §4 明确是**建议**（「基于现有证据不需要」「不建议为本片登记修正片」），并写明任何判据/声明面调整需 Arch→Chief→用户决定、本片不自行放宽 |
| 是否出现「已修复 / 上游等价 / 精度类达标」表述 | 否：交付 README/README §4.4 与 receipt 均显式声明不宣称；关键否定语句「不能断言设备选面完全等价于上游」在位 |
| 是否诚实列未覆盖 | 是：无 `face_idx`/EPA 三顶点/权重；witness 捕获无 `frame`/`normal`；QA `.bin` 未跟踪（只记 sha256）；未改 S2–S8/G5 数字/判据/fixtures/Cargo；未开 GPU |
| G5 交付结论 | 明确写「G5 交付结论不变」，与我的 G5 证据一致 |

## 5. 范围与数据完整性

- 提交 `c44578be` 仅含 11 个文件：6 个交付证据文件 + 5 个 `.normify` 记录；**无产品代码、无 `tests/`、无 `Cargo.*`、无 `fixtures/`、无 GPU 窗口**。
- 交付目录内 **0 个 `.bin`**；交付记录的 12 个 QA 未跟踪 bin 的 sha256 我全部复算：**12/12 一致**。
- pose 生成方法可核：QA `qa_prepare_capture.py` 中 `default_rng(20251025)`、`uniform(-1.4, 1.4)` 轴角、free joint 位置 `uniform(-0.08, 0.08)`×3 均在位；`meta.txt` world=24 与数据一致。

## 6. 观察（3 项，均非阻断；影响 G6 map UNC-6 的处置选项）

1. **归因机制超出证据边界（建议级）**：如 §3，「差 = 派生法线差 × 深度」是定义自洽的分解，不能定位「选面 tie-break」；建议把根因表述收窄为「派生法线差（机制未定）」，并在未覆盖里保留「选面是否与上游一致」。
   - **对 UNC-6 的含义**：现有证据支持「不需要修正片」（没有可定位的公式/索引错误、无系统性趋势）；若要**改判据**（witness 位置不再逐分量）属于声明面调整，需用户拍板；若要**修 tie-break** 需补 `face_idx`/三顶点/权重 + GPU 窗口另立片。三者都不被本片证据强制，现状最保守选项是「列为未覆盖」。
2. **中位数口径瑕疵（建议级）**：交付 `penetration_depth_median = 0.1100506335` 等于排序后第 13 小值（上中位数，交付脚本 `depths[len//2]`），严格两中位平均为 **0.1089157052**；README「中位 0.1101」偏差 0.0011（1%），不影响越界集合/极值/相关性/结论。建议改为 `statistics.median`（或注明口径）。
3. **统计力度标注（建议级）**：交付用 Pearson |r|≤0.25 说明「与深度不相关」。我重算四个相关系数完全一致（0.2473 / 0.1850 / 0.0896 / 0.2511），但 n=24 时对应双侧 p 约 **0.244 / 0.387 / 0.677 / 0.237**，均不显著——这只说明**该样本下无显著线性趋势**，不能当作「不存在系统性深度依赖」的强证据；结论宜标注样本量限制（交付 README 未展开 p 值）。

## 7. 方法与复现

```powershell
cd D:\Rust\mjwarp-rs\docs\development-evidence\g10-p5-deep-epa-witness-sampling-independent-check
python qa_deep_epa_check.py   # 只读 QA 原始样本 → qa-deep-epa-check.json
```

脚本内部：解析 12 个原始 `.bin`（f32/i32）→ 逐 world 重算 dist/x1/x2 判据与 ncontact/index/overflow/两次运行 → 与交付 `analysis.json` 数字逐项比对 → 与 `qa-compare-gpu.json` 9 项交叉核对 → 复算 world 8 派生法线角差/预测/实测/残差/恒等式差 → 复算 4 个 Pearson 并补 p 值 → 结论边界/未覆盖/范围核对。

## 8. 提交路径与不入库

入库（`docs/development-evidence/g10-p5-deep-epa-witness-sampling-independent-check/`）：

```
README.md  receipt.json  artifacts-sha256.json  qa_artifacts_sha256.py
qa_deep_epa_check.py  qa-deep-epa-check.json  qa-deep-epa-check.log
```

不入库：本目录无 `.bin`/`.pyc`/`.o`；QA 的 12 个 `.bin` 全程只读、未复制（sha256 已记入交付 `analysis.json` 与本复核 JSON）。

## 9. 结论

- **阻断 0 / 发现 0**：24 world 的统计与交付声明**逐项一致**（17/24、max 4 ULP、x1_bad=[8]、x2_bad=[8,10,15]、ncontact/index 24/24、overflow 0、两次运行位等）；与 QA 汇总 9/9 一致；12 个 bin sha256 12/12 一致；world 8 的「法线差 × 深度」定量关系复算正确（δθ 7.7095e-6 rad、预测 1.034805e-6 vs 实测 1.035245e-6、比值 1.000425）。
- **观察 3**：机制归因（选面 tie-break）超出证据边界、中位数用了上中位数、统计力度需标注样本量；均不影响「不需要修正片」的**建议**性质与 G5 结论。
