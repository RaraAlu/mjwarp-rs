# pilot readout `per_world_scope` 归一化 fix 最小复核（QA 只读）

任务：`2026-11-08-render-perworld-l-v2-pilot-g-l0-readout-fix-check`（owner QA）。日期：2026-11-08。
复核对象：Arch 的 material fix 提交 **`a9fbac22`**（HEAD）：`docs/development-evidence/render-bvh/perworld-l-v2-pilot-g-l0/materials/readout_c1_c2.py` 由 `456da2dc…`/13,366 B/305 行 → **`4a054aba…`/16,711 B/364 行**。
边界：只读核验 + 只写本卡证据；零 GPU；不改被复核材料与卡状态。

## 结论

**PASS（0 阻断，0 新观察）。→ 可开 attempt 3。**

## 1. 修法正确（逐行 diff）

`git diff -a a9fbac22^ a9fbac22 -- readout_c1_c2.py` 的**全部**变更：

1. docstring 增加 2026-11-08 fix 说明（`per_world_scope` 归一化；C1/C2/阈值/相机归属未改）；
2. 新增 `scope_text(value)`：**字符串原样返回**、list/tuple 逐元素 join、其它空串；
3. 新增 `scope_checks(scope)`：`scope_covers_light_and_geom`（covers 含两短语）与 `scope_no_old_wording`（does_not_cover 不含旧文案）；
4. `metadata_selfcheck` 原两行 scope 计算 → `checks.update(scope_checks(capture.get("per_world_scope")))`；
5. `selftest` 增加 6 条 scope 用例 + `SUMMARY-SCOPE`/`SUMMARY-ALL` 汇总与总失败数返回。

**未触及** `c1_counts`/`c2_counts`/`verdict`/`THRESH_RGB`/`ACCEPT`/`MARGIN`/`CAM0`/`CAM1`/其它 metadata 判定 ✅（diff 中这些函数与常量零 hunk）。

## 2. selftest 独立重跑（12/12）

```
SUMMARY PASS 6/6          # pass-margin / insufficient / non-discriminating / 1-ULP-only 拒绝 / identical-zero / C1-only 拒绝
SUMMARY-SCOPE PASS 6/6    # string→true / list→true / string 缺 geom→false / list 缺 geom→false /
                          # 旧文案 string→scope_no_old_wording=false / 旧文案 list→同
SUMMARY-ALL PASS 12/12    # exit 0
```

6 条计数/verdict 用例未回归 ✅；6 条 scope 用例覆盖「字符串原样」「list join」「缺短语」「旧文案两形状」✅。

## 3. 诊断复跑一致性（独立，用同一份作废外采数据）

数据：`target/render-bvh/perworld-l-v2-gl0/window/foreign-184138/pilot-out/nw3-light-hinge02040/`（Backend 外采、已作废；npz 未入库）。

| 脚本 | exit | `metadata.failed` | `scope_covers_light_and_geom` | C1 rgb (0→1/1→2) | C2 depth | C2 seg | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 旧 `456da2dc…`（同数据复跑） | **2** | `["scope_covers_light_and_geom"]` | **False** | 38,285 / 40,644 | 35,828 / 33,883 | 16,041 / 16,754 | pass-margin |
| 新 `4a054aba…`（同数据复跑） | **0** | **`[]`** | **True** | 38,285 / 40,644 | 35,828 / 33,883 | 16,041 / 16,754 | **pass-margin** |

⇒ 旧 bug 独立复现（字符串 join 按字符拼接 → 检查恒 false → exit 2）；修复后 exit 0；**C1/C2/verdict 逐项相同**，证明修复只改 scope 归一化、不改判读 ✅。`cam_active=[true,true,false]`、nrender/cam_id_map/cam_res/recipe.cameras 10 字段等其它 metadata 判定在新脚本下同样通过 ✅。

## 4. 非证据标注

- `artifacts/diagnostic-foreign-rerun.txt` 首部含 **`!!! NOT PILOT EVIDENCE !!!`**，并写明来源（作废外采）、用途（只诊断脚本 bug）与「不得引用为 pilot/参考证据」✅。
- npz 未入库：`a9fbac22` 提交清单无任何 npz/state 文件 ✅。
- README §8 记录：现象（`failed=["scope_covers_light_and_geom"]`、exit 2）、根因（旧 `join` 对字符串按字符拼接）、修法（`scope_text`/`scope_checks`）、脚本内 `join`/`json.dumps` 体检、selftest 12/12、诊断对比表（旧/新计数一致）、工具哈希更新 ✅。

## 5. 边界/证据/门禁

- `a9fbac22` 仅动：材料脚本 + 材料证据（README/receipt/manifest/materials/readout-selftest.txt）+ 2 个诊断 artifact + 卡/runs；`src/`、`fixtures/`、`scripts/`、`tests/` **零 diff** ✅。
- 材料 manifest **15/15** 独立重算通过；README/receipt/材料文件 UTF-8/LF/无 BOM/裸 CR·TAB 0 ✅。
- 四门禁独立复跑：fmt/clippy/clippy-cuda 通过，**unit 1371 过 / 0 败** ✅。

## 6. 观察项

无（0 阻断、0 新观察）。诊断 artifact 的「NOT PILOT EVIDENCE」与 npz 不入库已按要求标注；材料 README §8 的根因/修法自洽。

## 非宣称

- 本复核只判 readout 修复是否闭环且不改 C1/C2 判读；不判 pilot 结果、不宣称判别力/采集有效/灯验收/上游等价/任何渲染数值结论。
- 不构成 G-L0 启动；attempt 3 仍需 Chief token 与窗口条件；未改被复核材料与任何卡状态。
