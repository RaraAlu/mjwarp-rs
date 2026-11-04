# R03/R04 输出族收口（深度/分割）QA 复核 ——**PASS-with-observations（0 阻断）**

- 复核卡：`2026-11-04-render-r03-r04-output-closeout-check`（owner QA）
- 对象：`2026-11-04-render-r03-r04-output-closeout`（提交 **`948eddc6`**，verified）——`docs/development-evidence/render-bvh/r03-r04-summary/`（README 4048 B / `d380ddb1292a9fa1…`、manifest 2/2、receipt）
- 纪律：只读、零 GPU、未改任何证据；写锁仅用于本证据落库。

## 1. 复核结果

- **证据索引**：8 个 task-id 指针全部解析（其中 1 个是本复核卡自身，复核后即 verified）；缩写证据目录 `……/s7b1-depth-seg/`、`……/s8-e2e/`、`……/r08-camera-output/`、`……/r10-s2-multi-world-e2e/` 等全部存在。
- **判据达成值**：README 明写深度 `120,000/120,000` 位等、`max 0 ULP`、分割精确相等；并写「深度 ≤4 ULP 且 beyond4=0，**位级优先**，实测位级 0 ULP」，未把「≤4 ULP」当作放宽后的通过。
- **已知限制**：AA 第一趟语义（S7b3）、更高 spp、上游 nworld>1 无数值参考、无性能声明、家族边界（skybox/hfield/flex/SDF 不背书）——逐条在场。
- **非声明**：不声明 R03/R04 之外能力、不声明与上游完全一致、不声明对外发布依据、不替其他家族背书——逐条在场。
- **记账**：`plan.json` 只改 `delivery_tracking`：+R03/+R04 两行（task=本卡、evidence=本目录、normify_status=verified）与 `N16`；`r01_p8_mapping` 与 R01/R06 slice 定义逐字节未变；roadmap-r03/r04 节点未被本卡 start/done。
- **边界**：提交 5 文件（card、plan.json、summary 3 件），`src/tests/fixtures` 零 diff，precheck README 未被本提交改动；manifest 2/2 自核、首行 `#`、LF 无 BOM；receipt `gpu_used/product_code_changed/criteria_changed=false`。

## 2. 观察（低，不阻塞）

{1}. **O-1（observation）**：receipt omits the reference_changed field (C receipts declare gpu/product/criteria but not reference)
{2}. **O-2（observation）**：pointer scan shows the family's own QA check card as not-yet-verified
{3}. **O-3（observation）**：delivery_tracking.updated_at was already 2026-11-04 from earlier closeouts, so these commits did not change it

## 3. 结论

**PASS-with-observations（0 阻断）**：逐片证据指针到 verified 实现卡、判据达成值可复算、已知限制与非声明齐全，plan.json 记账只动 `delivery_tracking` 且与 summary 一致，边界零代码改动。可交 Chief 汇总（C 系列完成后统一收口 roadmap）。

不宣称上游等价 / parity / 对外发布依据；不重开任何已 verified 判定。

## 4. 复现

```powershell
target\mjwarp-parallel\venv\Scripts\python.exe -B docs/development-evidence/render-bvh/r03-r04-summary-check/scripts/qa_c_check.py C1 --out docs/development-evidence/render-bvh/r03-r04-summary-check/logs/C1-check.log
```

## 5. 产物清单

- `README.md`（本文）、`facts.json`、`manifest.txt`
- `scripts/qa_c_check.py`
- `logs/C1-check.log`
