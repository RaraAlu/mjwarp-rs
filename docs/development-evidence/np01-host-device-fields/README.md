# NP01 B2：host/device 字段级位差分账（四类差异分开算、分开报）

任务：`2026-11-04-precision-np01-host-device-fields`（B1 之后；B4 QA 复核前）。
脚本：`scripts/authoring/np01-host-device-fields.py`（离线、只读归档、零 GPU）。
输出：同目录 `host_device_fields.json`（**本 README 数字全部由脚本产出**）。

复跑：

```powershell
& target\mjwarp-parallel\venv\Scripts\python.exe scripts\authoring\np01-host-device-fields.py
```

本片存在的唯一目的：**把「数值差」拆成类型 / recipe / 同 recipe 实现 / 引擎语义四类，防止把 recipe 差写成类型偏离**。

## 一、类型偏离 = 0 项（但不从数值相等反推）

| 依据 | 内容 |
| --- | --- |
| 逐字段类型表 | 30 条产品路径 + 3 条 host 站点（NP01 B1–B3 批） |
| CUDA token 门禁 | 产品 `.cu` 代码区不含 `double`/`half`/`bfloat16`/`tf32`（`np01-type-mapping/cuda-surface.json`） |
| host source 门禁 | `src/**/*.rs` 代码区精度 token 白名单活性（host-source-gate 卡） |

**明确不从「数值位级相等/ULP 计数」反推类型偏离**：数值差可以来自 recipe 或引擎语义，反推会把 B1 已证明的融合现象误判成类型问题。

## 二、recipe 差（同码不同后端：CPU 两次舍入 vs CUDA fma.rn）

来源 `g10-p5-g2-gpu-reference-capture/compare-cpu-vs-gpu.json`（CPU 参考 vs GPU 参考，5 case 全 GREEN、两跑位级一致）。机制见 B1：`--fmad=true` 下 8/12 条带乘加表达式全部融合成 `fma.rn`。

| case | dist（位等/总数，max_abs） | pos | frame |
| --- | --- | --- | --- |
| cylinder-box | 8/8，0 | 19/24，1.49e-8 | 72/72，0 |
| ellipsoid-box | 0/2，1.86e-9 | 2/6，7.45e-9 | 8/18，8.19e-7 |
| mesh-mesh | 1/2，9.31e-9 | 4/6，7.45e-9 | 2/18，5.36e-7 |
| convex-mesh | 0/2，1.49e-8 | 1/6，1.12e-8 | 2/18，2.98e-7 |
| degenerate-mesh | 0/2，1.05e-8 | 4/6，1.86e-9 | 16/18，2.65e-11 |

读法：**`max_ulp_nonzero` 在近零分量上会失真**（如 frame 出现 4294967295 是跨 0 号比较导致），判读请用 `max_abs`；本表所有 `max_abs` 都在 1e-9–8e-7 量级，与 B1 的「多一次舍入」机制自洽。这是 **recipe 差，不是类型偏离、也不是实现缺陷**。

## 三、同 recipe 实现残差（设备 vs 上游 CUDA，两侧都是 fma.rn + 全 f32）

来源 `g10-p5-gpu-kernels/gpu-window/result.json`（`failed_cases=[]`）。

| case | dist | pos | frame |
| --- | --- | --- | --- |
| hfield-sphere | 8/8 位等 | 24/24 位等 | 72/72 位等 |
| hfield-capsule | 7/7 位等 | 21/21 位等 | 63/63 位等 |
| sdf-box | 80/80 位等 | 203/240，max_abs 7.45e-9（1.0 ULP@scale） | 675/720，max_abs 2.38e-7 |
| ellipsoid-box | 1/2，2 ULP | 5/6，max_abs 3.02e-9（0.81 ULP@scale） | 10/18，max_abs 3.80e-7 |

读法：两侧同为 fma.rn recipe、同为 f32 → 残差**既不能归 recipe、也不能归类型**，只能归「实现/算法顺序差」（内核分解、累加顺序、horizon/裁剪顺序）。hfield 两例逐位全等是强证据：同 recipe 下同实现即可位级一致。

## 四、引擎语义差（引用 A0，不在本片判定）

native MuJoCo C 与 warp 是不同引擎：`hfield-sphere`/`hfield-capsule` 接触数不等（6/6、6/3 vs 8、7）不可配对；可配对的 `ellipsoid-box`/`sdf-box` 位姿系统性差（pos 4.55e-4 / 7.34e-2，frame 1.17e-3 / 1.58）→ 属接触选择/裁剪语义差。详见 A0 `np01-e-recompute/README.md`；本片只引用，不重复判定、不参与类型结论。

## 五、已知偏差（引用 verified 卡，不冒充新发现）

`witness/cylinder-box-sweep`：`dist_bad 0 / x1_bad 1 / x2_bad 3`（超 4·ULP@scale），ncontact 与 overflow 相等。登记卡 `2026-10-25-g10-p5-deep-epa-witness-sampling`（verified，排 G6 之后处理、不阻塞 G6）。**归类：同 recipe 实现残差**（witness 分量对表达式顺序敏感），不是类型偏离、不是 recipe 差；本片不据它改判据。

## 六、结论

1. **类型偏离 0 项**（靠类型表 + 两道 token 门禁，不从数值反推）。
2. **recipe 差已量化**（5 case × 3 字段，max_abs 1e-9–8e-7），机制由 B1 的 PTX 表解释（fma.rn vs mul/add）。
3. **同 recipe 实现残差已量化**（4 case，hfield 全位等、sdf/ellipsoid 少量 ULP 级），说明「同 recipe 下可接近位级」。
4. **引擎语义差单列**（A0），不进类型结论。
5. **不宣称位级等价、不宣称 NP01/NP03 通过**；「类型偏离 0」≠「host/device 位级一致」——后者需同 recipe + 同编译形态前提。

## 七、边界

- 只读已归档证据（未新采样、零 GPU、零窗口）；不改产品代码/判据/容差/fixtures。
- 归类口径是「按证据链归因」，不是「按数值大小猜」；四类差异互斥且可追溯到具体归档文件。
- 后续 B4：QA 独立复核本片，重点核「没把 recipe 差写成类型偏离」。
