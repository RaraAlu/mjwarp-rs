# 决策支持口径：f32 修复后残留 1 px 的放行（供用户签）

> 用途：供 Chief 在「attempt5 前置是否满足」上向用户请示时**逐字引用**。本页只陈述口径与依据，不改任何判据、容差、参考或 `fixtures/`。

## 一句话

修复后 `cam2 depth = (mismatched, beyond4) = (1, 0)`：`beyond4` 已满足，唯一未过的是冻结的 **bit-first** 判据（`mismatched` 必须为 0）。
因此「放行这 1 px」**不是 B 类前提自然适用，而是判据放宽（签名级）**，必须用户签。

## 为什么必须签

- 冻结判据：`tests/render_flex_validate.rs:344/:347` 断言 `(mismatched, beyond4) == (0, 0)`；`MAX_DEPTH_ULP = 4`（`src/render/kernel/output.rs:15`）。
- 该 px 已定性：`cam2 (x=134, y=128)`、`flexid=0`、**ULP=3**、`beyond4ULP=0`、绝对差 **7.15e-7 m**；`cam2 seg` 仍 0/76,800。
- 根因：flex-0 的 **edge0/edge1 在 `RAY_TOL_REL` 容差内平局**（`|Δd| = 7.1526e-7 < tol = 1e-6*2.0099208 = 2.0099e-6`），胜出者由**候选访问顺序**决定（参考最后访问 edge0、产品最后访问 edge1）。上游同链回放 oracle 对参考 depth **0/403 位级复现**，说明链算术已对齐；这 1 px **不是 codegen ULP 尾差**，f32 次序修法在本卡 scope 内**不可修**。
- 结论：任何形式的放行都等于**覆盖冻结判据** → 签名级；QA/Chief 不能内部消化。

## 建议用户签的内容（最小范围，写死这 5 条）

1. **豁免范围**：仅 `cam2`、`(x=134, y=128)`、`flexid=0`，且条件为 **`ULP ≤ 4` 且 `beyond4ULP = 0`**；不扩到其他像素/相机/场景。若要写成族口径，必须写成「1D flex 薄胶囊近掠射子集 `≤4 ULP`」并附当前唯一命中 px 清单（现为上述 1 px）。
2. **attempt5 判据同步**：`除该 1 px（或该子集）外 bit-first；该 px 仅要求 ULP≤4 且 beyond4=0`。不签这条文本，attempt5 仍会因 `(1,0)` 判红。
3. **与 D8 的 321 px 尾差分开登记**：D8 的 321 px 已按 **B 类（参考-前提型）codegen ULP 尾差**降级记录；**本 px 是 tie 顺序问题，不得并入 B 类记录**，两者证据分开引用。
4. **并行立根因卡**：「1D flex 候选顺序 / tie 语义对齐」——Arch 已登记 `2026-11-04-render-r06-s3b-flex-tie-order`（提交 `3e3f363d`，Backend 执行，要求 rigid/既有 render 实体位级零回归）。签豁免不等于取消根因收口；若该卡长期不做，需 Chief 明确「接受为已知限制」。
5. **记账**：用户签的日期/文本/适用像素写进 attempt5 材料与 guards 旁；D8 B 类证据里「判据未放宽」的前提改写为「本子集按用户签口径放行」。

## 不签的后果

`attempt5` 前置不满足（r2 修复卡无法 verified），**不能宣称 f32 修复通过**；只能记录「320/321 已修复、1 px 根因已定位、不可在 f32 scope 内修」。

## 依据（可复核）

- 本复核卡证据：`docs/development-evidence/render-bvh/r06-s3b-flex-f32-order-check/`
  - 独立复算：`1/76800` 失配、ULP=3、beyond4=0、seg 0/76,800、D8 321→1（修掉 320、新失配是 D8 子集）；
  - tie 数学与 oracle 复现：`d3=[2.0099208355, 2.0099215508, -1]`、Δd<tol、`depth=d*factor` 位级同时复现两侧 bits、oracle 对参考 `0/403`；
  - 复现脚本与原始日志：`scripts/qa_f32_verify.py`、`logs/qa_f32_verify.log`。
- 对象证据（r2 WIP 快照）：`docs/development-evidence/render-bvh/r06-s3b-flex-f32-order/`（manifest + README §6 残留像素判定 + `artifacts/oracle-product-sources.json`）。
- 判据定义：`tests/render_flex_validate.rs:344/:347`、`src/render/kernel/output.rs:15`、`RAY_TOL_REL`（`src/render/kernel/cuda/render_math.cu:28`）、tie 规则（`src/render/kernel/cuda/render_common.cu:455-460`、`candidate_id=-2` 在 `:377`）。
