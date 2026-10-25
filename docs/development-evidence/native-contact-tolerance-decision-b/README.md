# native 侧接触字段：用户拍板 B（待验收不放行、只作诊断）

- 任务：`2026-10-25-native-contact-tolerance-decision-b`
- 用户拍板：**2026-10-25，选项 B**——`native_error_v2` 的接触字段（`dist`/`pos`/`frame`/`normal`）native 侧**只作诊断、不做放行判定**（经 Chief 转达；用户原话「选 B——native 侧接触字段只作诊断、不做放行判定」）
- 契约落点：`docs/numerical-precision-contract.md` → `## 接触字段（native 侧）`（`## 数值参考优先级` 之后、`## 测试项与报告标注` 之前）
- 边界：不改对上游 f32 的既有判据、不写任何生效 `atol`/`rtol`、不改已 verified 证据数字（G2–G5 receipt）、不改 S2–S8 证据、不改 `Cargo.*` 与 `fixtures/mjwarp-complex-physics/*.xml`
- 上游 pinned：`71da24d956378a87a703b6e1442b13aec0c4ac29`（MJWarp 3.15）

## 1. 契约判据原文（本节落地后逐字引用）

`docs/numerical-precision-contract.md` 的 `## 接触字段（native 侧）` 小节：

```text
用户于2026-10-25拍板选项B。
接触字段的native侧只作诊断。
适用字段为dist、pos、frame与normal。
这些字段的native侧一律待验收不放行。
该判定不参与通过或失败。
`qa-native-f64.json`读数只作诊断与趋势。
该读数不构成准入依据。
接触精度仍由对上游f32的判据承担。
该判据为既有定稿口径（向量尺度ULP、frame绝对ULP、结构零容差）。
本节不表示接触字段没有精度要求。
红线`E_candidate <= 1.05 * E_mjwarp`在其它字段照旧生效。
接触字段的native侧不适用该红线。
依据：真native仅`ellipsoid-box`1个case、2个接触点。
样本不足以定容差，且不可外推。
设备与上游f32对native的pos与frame差几乎相同。
该差异属引擎接触选择语义差，非浮点精度差。
因此不采纳原生侧接触容差（选项A留待扩样后再议）。
本节不写入任何生效atol或rtol。
```

**精度要求没有取消**：接触字段仍须满足既有对上游 f32 判据，出处：

- `docs/numerical-precision-contract.md`「冻结基线」：上游采用 f32 时全链保持 f32，不得私自升/降精度；
- `docs/development-evidence/g10-p5-precheck/README.md`：接触 `dist` 位型全等目标、不可达时 ≤4 ULP；`pos`/`frame`/`normal` ≤4 ULP（normal 含归一化，frame 由 normal 派生同判据）；
- g10-p5 判据补全（Arch 2026-10-25，G5 起执行）：`pos`/`x1`/`x2` 用向量尺度 `scale=max|ref|` 的 `4·ULP`，`frame`/`normal` 用 `scale=1.0` 的 `4 ULP` 绝对差，`dist` 用自身量级 ULP，结构字段零容差；判别力有可执行断言（`criterion_test.rs`）。

## 2. 依据材料 v3（`target/g10-p5-native-tolerance-candidates-v3.md` 结论摘要）

- **v2 作废**：v2 把 `recapture_diff_report.json` 误标为「上游 f32 vs native」，那实际是 **warp CPU vs warp CUDA 的跨后端差**（QA 核实：coord/dist ≤6.9e-7、frame ≤2.8e-5）。v3 只用真 native（MuJoCo C f64）。
- 数据来源：`docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-compare-gpu.json` 的 `native_f64_readings`（QA 独立采集，已标 `usable_as_tolerance_basis=false`）。
- **唯一可比 case：`ellipsoid-box`（2 world × 各 1 接触）**

| 比较 | `dist` max | `pos` max | `frame` max | `normal` max |
| --- | --- | --- | --- | --- |
| 设备（产品）vs native | 3.979e-7（rel 3.98e-5） | **4.546e-4** | **1.170e-3** | 1.170e-3 |
| 上游 f32（warp CUDA）vs native | 3.961e-7（rel 3.96e-5） | **4.546e-4** | **1.171e-3** | 1.171e-3 |

- **结论**：设备与上游 f32 对 native 的差几乎相同 → 是 **MuJoCo C 与 warp 的接触选择语义差异**（接触点选在流形内不同位置），不是本库实现或 f32 精度问题。已排除采集错误：native `geom_xpos` 与 warp 逐位相同（max_abs 0）、每 world 接触数相等（各 1）。
- 不采用选项 A 的理由：A（粗门禁 `pos` 1e-3 / `frame`·`normal` 2.5e-3 / `dist` 1e-6+rtol 1e-4）只能抓 ≥1e-2 的 gross error，比真实缺陷（8.3e-5）松 12×；用 1 个样本 + 引擎语义差当判据会制造「已达标」假象。A 留待样本扩充后再议。

## 3. 样本量说明（不可外推）

**真 native 接触参考 = 1 个 case / 2 个接触点**（`ellipsoid-box`：2 world × 各 1 接触）。**任何数值都不可外推到其它 case、其它硬件或其它精度声明**。不可比项及原因：

| case | 不可比原因 |
| --- | --- |
| `hfield-sphere` | native 每 world 6 接触 vs 设备 4 |
| `hfield-capsule` | native 6/3 vs 设备 4/3 |
| `sdf-box` | native SDF 路径与 warp 不同（接触位置中位误差 4.08e-2） |
| `sphere-box-shallow/deep`、`capsule-box` | 上游走 primitive narrowphase，设备未实装 |

## 4. 对 G6 的约束（后端执行、QA 复核）

1. **G6-3 按 B 落框架**，不再标「待拍板」：`native-framework.md` 里 A/B/C 选项表改为「已拍板 B」+ A 留待扩样；
2. G6 首步仍做**真实产品输出对 native 的 E 复算**（标注样本量 1 case / 2 接触点，诊断性质）与 `hfield-box` 配对判定（不可配对则记「不可配对、样本不增」）；
3. G6-1 映射新增一行「native 侧接触字段：待验收不放行（用户 2026-10-25 拍板 B）」指向本证据；
4. **QA 的 G6 独立复核须包含**：核契约措辞与本片落地是否一致、是否没有把诊断写成放行。

## 5. 表述纪律

- **不得写**：「接触字段精度已达标」「native 等价」「native 放行」「原生放行」；
- **只能写**：「native 侧待验收、诊断读数见证据（`qa-native-f64.json`）」「接触精度由对上游 f32 的 ≤4 ULP 判据承担」。

## 6. 未覆盖 / 边界

- 本片只改契约措辞与归档证据，**不写生效 `atol`/`rtol`**、不新增字段块、不改 NP03 判定流程；
- 未扩样：真 native 仍 1 case / 2 接触点；「native 接触输出探针」若要做，另立小片；
- 未覆盖：Linux/T4 的 native 读数、多接触点 case 的 native 参考、`hfield`/`SDF` 的 native 接触配对（均不可比或不存在）。
