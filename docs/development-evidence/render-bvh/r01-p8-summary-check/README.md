# R01@P8 渲染上下文聚合收口（B）QA 复核 ——**PASS-with-observations（0 阻断）**

- 复核卡：`2026-11-04-render-r01-p8-closeout-check`（owner QA，p85，depends 收口卡）
- 对象：`2026-11-04-render-r01-p8-closeout`（提交 **`e492c979`**，verified）——`docs/development-evidence/render-bvh/r01-p8-summary/`（README 7,351 B / `c7b30fc9…`、receipt 4,770 B、manifest 2/2）+ `plan.json` + precheck `README.md` §7 addendum
- 纪律：只读、零 GPU、未改任何证据；写锁仅用于本证据落库。

## 1. 复核项与结果

| # | 复核项 | 独立核对结果 |
| --- | --- | --- |
| 1 | **10 项映射** | `plan.json.r01_p8_mapping` 14 条（10 项主体 + `resource/rejection` 拆 2 + 3 条补充）；主体 10 项**全部 `planned → delivered`**（`shadows_skybox_splats_flex_hfield` → `delivered-except-r07`），每条含 `slices` 与 note 内证据指针；**mapping note 内 26 个 task id 全部可解析到 `verified` 卡**；R07 明确「未做、不在 R01 范围」 |
| 2 | **NP 对账** | 21 个 NP 证据卡**全部存在且 verified**；NP01 = 已核对**但 E 指标口径 1 项待用户签（A2）**，并写死禁止「100% 类型匹配 / 零精度偏离」；NP02 = 满足（frozen inventory / evidence gate / receipt rehash / new kernel / ptx entity restore）；NP03 = 不适用不宣称（用户 2026-10-24 终止 Linux）；NP04 = 适用项满足（相对基准）——与 receipt `np_check` 及 README §3 三方一致 |
| 3 | **已知限制** | 7 条齐全：release 全量未跑、R07 不在范围、更高 spp、Linux/T4、与上游完全一致未声明、R05/R09 窗口例外、R06 单帧边界（多帧未覆盖，指向跟踪卡） |
| 4 | **非声明** | 6 条齐全：不声明完整 R01、不声明双平台、不声明上游一致、不声明 release 全量、不声明 R07/范围外能力、不作对外发布依据 |
| 5 | **记账与原文** | `plan.json` diff **只动两处**：`r01_p8_mapping`（planned→delivered 刷新 + 证据指针）与 `delivery_tracking`（**+1 行 `R01-P8`**、**+`N15`**；`updated_at` 已是 2026-11-04 未再改）——其余顶层键与 **R06 九片定义逐字节不变**；`N15` 覆盖十项映射/家族交付/R07 出界/NP01 待签与禁语/NP02… 等；precheck `README.md` diff **+8 / −0**，addendum 明写「不改上文 2026-10-25 结论原文」，我核原文段落未被删改 |
| 6 | **边界** | 提交 6 文件（card、precheck README、plan.json、summary 3 件），`src/`/`tests/`/`fixtures/` **零 diff**；summary manifest **2/2** 自核、首行 `#`、LF 无 BOM；receipt `gpu_used/product_code_changed/criteria_changed/reference_changed=false`、`status=closeout-evidence-only` |

## 2. 观察（低，不阻塞）

1. **O-B1 映射粒度计数**：`plan.json` 把 `resource_and_layout_checks / rejection_surface` 拆成 2 条（合计 11 条主体 + 3 补充 = 14），而 README 表格与 receipt `ten_item_mapping` 合并为 10 行（补充项单独 3 行）。语义等价、无遗漏，仅「10 vs 11 vs 14」计数口径不同，建议汇总引用时统一说法（如「10 项主体，其中 1 项拆 2 子项」）。
2. **O-B2 addendum 措辞**：precheck §7 addendum 写「2026-10-25 列出的『未满足项：R05–R10』中，R02/R05/R06/R08/R09/R10 已交付」——原「未满足项」列表是 R05–R10，R02 是后补交付；表述稍有跨列表，建议补「R02 亦已交付」式的明确区分（不影响结论）。
3. **O-B3 `updated_at`**：B 卡 acceptance 写「`updated_at` 推进到 2026-11-04」，实际 A 卡时已推进，B 未再改动（diff 无此行）。状态正确，仅记账动作与文字描述略有出入。
4. **O-B4 证据目录命名**：README §6 用 `……/r08-summary/`、`……/r10-...` 等缩写；实际 R08/R10 目录名与本卡引用的 task id 不完全同形（如 `r08-camera-output/`、`r10-s2-multi-world-e2e/`），按 task id 检索更稳妥（mapping note 已内嵌 task id，可解析）。

## 3. 结论

**PASS-with-observations（0 阻断）**：B 收口把十项映射从 planned 刷新为 delivered 并补齐证据指针、NP 四类核对与已知限制/非声明齐全；记账只动 plan.json 的 R06 之外两项与 precheck addendum、原文未被改写；边界零代码改动。可交 Chief 汇总（A + B）。上述 4 条观察均为低级别计数/措辞/命名差异，不改变结论。

不宣称完整 R01 / 双平台 / 上游完全一致 / release 全量 / 对外发布依据；不重开任何已 verified 判定。

## 4. 复现

```powershell
$ev = "docs\development-evidence\render-bvh\r01-p8-summary-check"
target\mjwarp-parallel\venv\Scripts\python.exe -B "$ev\scripts\qa_b_check.py" --out "$ev\logs\b-check.log"
target\mjwarp-parallel\venv\Scripts\python.exe -B "$ev\scripts\qa_b_plan_diff_check.py" --out "$ev\logs\plan-diff-check.log"
```

## 5. 产物清单

- `README.md`（本文）、`facts.json`、`manifest.txt`
- `scripts/qa_b_check.py`、`scripts/qa_b_plan_diff_check.py`
- `logs/b-check.log`、`logs/plan-diff-check.log`
