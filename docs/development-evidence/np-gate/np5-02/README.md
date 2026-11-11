# NP5-02：NP01 精度偏离审计（全路径汇总；零 GPU）

- 任务：`2026-11-11-np5-02-np01-precision-deviation-audit`（owner Backend）。
- 依据：W5 计划 §3（`target/render-bvh/gl2-ptx-diff/w5-np01-02-plan.md`）；判据 NP01 = **类型映射 100% +
  精度偏离 0 项**（`docs/decisions/d05-numerical-performance-gates.md:30-31`）。
- 方法：只读聚合现有证据（不改判据/参考/容差、零 GPU、零窗）；复用 NP5-01a **608 站点（唯一 598）**
  逐站点类型映射/分类清单与 `np01-host-device-*`（B1 表达式 PTX 定序、B2 字段级位差分账）工具。
- 输出：本目录 `evidence.json`（全部数字/哈希由 `target/render-bvh/gl2-ptx-diff/np5-02-draft/gen_np5_02.py`
  从归档证据重算）+ README/receipt/logs/selfcheck/manifest。

## 1. 审计对象（全路径）

- **NP5-01a 608 站点（唯一表达式 598）**：11 片逐站点 type_mapping + 分类。
  - 站点分类（landing 直方图合计）：`n/a` 320 / `identical` 262 / `engine-semantic` 20 /
    `recipe-difference` 2 / `pin-needed` 1 / `not-decidable` 3（最终口径另见 §4）。
  - `type_class`：f32 510、`int-encoded-f32` 10、bool/bool(f32) 20、int 41、n/a 16、其它 f32 派生；
    **`double_in_product` 仅 1 条**（`field_batch` wrap-pulley `1.0/wrap_prm`，已记
    `double_precision_parity`：上游 numpy float64→f32 同路径，非产品单方面升精度）。
- **类型表**：30 条产品路径 `np01-type-mapping/field-tables/*.json` + 3 条 host 站点
  `host-tables/*.json` = **33 表**；CUDA surface 快照（文件/`__global__` 计数，ratchet 维护）。
- **B1 表达式 PTX 定序**（12 条，gjk/ccd/epa host 站点）+ **B2 字段级位差分账**
  （5 case×3 字段 recipe 差、4 case 同 recipe 残差、1 条 known deviation）+ A0 引擎语义差引用。
- **RE 设备级片**：RE-1（mass_solve 3 站点）、RE-2（A-2 3 行）、RE-3（B/C 24 行 304 站点）。

## 2. 类型偏离 = 0 项（不从数值反推）

| 依据 | 内容 |
| --- | --- |
| NP5-01a 逐站点 | 608 站点 type_mapping 全为 f32 或 f32 派生（无 f64/f16/tf32）；唯一 f64 接触点已 parity 解释 |
| 逐字段类型表 | 30 产品路径 + 3 host 站点表（33 表） |
| CUDA token 门禁 | `cuda-surface.json` 快照 + ratchet（产品 `.cu` 代码区不得含 double/half/bfloat16/tf32） |
| host source 门禁 | `src/**/*.rs` 精度 token 白名单（host-source-gate 卡） |

**明确不从数值完全相等/ULP 计数反推类型偏离**（数值差可由 recipe/引擎语义产生，反推会把已量化的
融合现象误判为类型问题）。

## 3. recipe 差（已量化，单列不计入偏离）

- 机制（B1，12 条表达式）：带乘加表达式在 `--fmad=true` 下全部出现 `fma.rn`、`mul.rn/add.rn` 归零；
  `--fmad=false` 下与 host「每乘加两次舍入」逐项对应。**源码写法挡不住 device 收缩**
  （`face_norm2` head→tail→norm2 仍被 fuse）。
- B2 5 case × 3 字段（CPU 参考 vs GPU 参考）：`max_abs` 1e-9–8e-7（dist/pos/frame；用 `max_abs`
  判读，`max_ulp_nonzero` 在近零分量失真）——属 recipe 差、非类型偏离、非实现缺陷。
- NP5-01a 最终口径 recipe-difference **3 站点**（A-1a `acc:3`、A-1b `acc:5`、RE-1 的 `acc:5` 对应
  `rcp.rn` vs `div.rn` 值等价形态差，均带 `value_equivalent/analytic, not bit-tested`）。

## 4. 引擎语义差（单列）与实现残差

- **引擎语义差**：NP5-01a source 级 `engine-semantic` **20 站点**（A-1a 1 / A-1b 3 / A-2 4 /
  A-3 3 / B-1 6 / C-1 2 / C-3 1）；RE-3 设备级 15 行 204 站点单列；A0 的 hfield/sdf 接触选择差引用
  ——**不进类型结论**。
- **同 recipe 实现残差**（B2 4 case）：hfield 两例位等；sdf/ellipsoid 少量 ULP 级
  （max_abs 7.45e-9/3.80e-7）；known deviation `witness/cylinder-box-sweep`（x1/x2 极少量 ULP，
  已登记 G6 后处理卡）——归「实现/求和顺序差」，不归 recipe/类型。

## 5. 未覆盖/阻塞项（NP01 精度偏离 0 项未完成）

| 项 | 状态 | 影响 |
| --- | --- | --- |
| E 指标定稿 | **阻塞在用户裁决（2026-10-27）** | 精度偏离判据/容限无法冻结 |
| Windows/Linux 双平台精度复验 | Linux 用户 2026-10-24 暂停 | 无跨平台结论 |
| 跨 recipe 位级判据 | 前提不成立（RE-0 仅 G1-partial；B1 两套 recipe） | 只允许类型/顺序审计 |
| device 级不可判站点 | 逐条 `not-decidable`（host 驱动/no-reference-entry 等） | device 覆盖为可映射子集 |
| 7 未实现子系统 | NP02 缺口账本（NP5-03 处理） | 不得计为覆盖 |

## 6. 结论（含强制未完成句）

- **类型偏离 0 项**：依据类型表 + 两道 token 门禁 + NP5-01a 608 站点 type_mapping（非数值反推）。
- recipe 差已量化（B2 + NP5-01a 3 站点）并单列；引擎语义差已单列（20 站点 + RE-3 204 站点行级）；
  同 recipe 实现残差与 1 条 known deviation 已登记。
- **「NP01 精度偏离 0 项」未完成**：E 指标未定稿、跨 recipe 位级判据前提不成立、device 级仅可映射子集、
  7 未实现子系统与 Linux/双平台项仍阻塞。**不宣言 NP01 通过。**

## 7. 非声明与边界

不宣称 NP01 通过/E 指标完成/上游等价/双平台/位级保证；不改判据/参考/容差/相机/recipe；
零 GPU、零窗；只读聚合，不改任何历史片字节。
