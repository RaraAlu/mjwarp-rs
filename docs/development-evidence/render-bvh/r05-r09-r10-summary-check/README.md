# R05/R09/R10 家族收口（hfield/光照阴影/多世界）QA 复核 ——**PASS-with-observations（0 阻断）**

- 复核卡：`2026-11-04-render-r05-r09-r10-family-closeout-check`（owner QA）
- 对象：`2026-11-04-render-r05-r09-r10-family-closeout`（提交 **`0f49dba9`**，verified）——`docs/development-evidence/render-bvh/r05-r09-r10-summary/`（README 5456 B / `9308e4aeb64aff54…`、manifest 2/2、receipt）
- 纪律：只读、零 GPU、未改任何证据；写锁仅用于本证据落库。

## 1. 复核结果

- **证据索引**：16 个 task-id 指针中 14 个解析为 verified；2 个例外均有解释：`perworld-l-v2-k-backlog` 为**仅跟踪**的 backlog（README 明写不执行），`r05-s2-hfield-intersect` 为**已 abandon 的原 id**（见观察 O-4）；其余 R05/R09/R10 逐片与 L-S1 指针、缩写证据目录全部存在。
- **判据达成值**：R05-S3b attempt=2（9/9 exit 0、sha_guards 73/73、post 9/9、零编译、floor 10,233 未破、两跑 sha 相同）；R09 阴影验证窗口与二次射线修复留痕；R10 attempt2-run（8/8 exit 0、guards 46/46、post 8/8、零 Rust 重编、floor 9,708 未破）；数值为各族已签口径，本卡不复述。
- **已知限制**：authoring-only/validation-only util<60 例外、attempt1 INVALID_ROUND 原样保留、R09 产品路径在 S2b 后才可用、R10 无上游 nworld>1 数值参考、更高 spp/Linux/T4/上游完全一致未覆盖、三族均无性能声明。
- **非声明**：不声明三族之外能力、不声明与上游完全一致、不声明对外发布依据、不替其他家族（R02/R03/R04/R06/R07）背书——逐条在场。
- **记账**：`plan.json` 只改 `delivery_tracking`：+R05/+R09/+R10 三行与 `N18`；`r01_p8_mapping` 与 R01/R06 slice 定义逐字节未变；roadmap-r05/r09/r10 未被本卡 start/done。
- **C3 等价记录（已确认）**：原 id `2026-11-04-render-r05-r09-r10-closeout` 在本机 `normify start` 稳定报 os error 5，Arch 按既有口径重试/清 tmp/`git checkout` 重建/显式 `--repo` 均无效，最终**同内容换 id** 为 `…-family-closeout`；我核：**原 id 卡文件不存在、`git ls-files` 无残留**，新卡 note 与 QA 卡 note 均写明原因，按「已确认的等价记录」接受，不按阻断。
- **边界**：提交 5 文件，`src/tests/fixtures` 零 diff，precheck README 未被本提交改动；manifest 2/2、LF 无 BOM；receipt `gpu_used/product_code_changed/criteria_changed=false`。

## 2. 观察（低，不阻塞）

{1}. **O-1（observation）**：receipt omits the reference_changed field (C receipts declare gpu/product/criteria but not reference)
{2}. **O-2（observation）**：pointer scan shows the family's own QA check card as not-yet-verified
{3}. **O-3（observation）**：delivery_tracking.updated_at was already 2026-11-04 from earlier closeouts, so these commits did not change it
{4}. **O-4（requires-fix）**：evidence README/receipt cite the abandoned card 2026-10-28-render-bvh-r05-s2-hfield-intersect instead of the verified -r2 variant。README line ~17 writes `R05-S2 2026-10-28-render-bvh-r05-s2-hfield-intersect（r2）` and receipt line ~14 uses the base id; only `-r2` is verified while the base id is abandoned → pointer not machine-resolvable to a verified card — 建议：point to 2026-10-28-render-bvh-r05-s2-hfield-intersect-r2 (or explicitly name the r2 id) in an addenda; semantics already say (r2)
{5}. **O-5（observation）**：perworld L-v2-K backlog card is intentionally referenced as tracking-only

## 3. 结论

**PASS-with-observations（0 阻断）**：逐片证据指针到 verified 实现卡、判据达成值可复算、已知限制与非声明齐全，plan.json 记账只动 `delivery_tracking` 且与 summary 一致，边界零代码改动。可交 Chief 汇总（C 系列完成后统一收口 roadmap）。

不宣称上游等价 / parity / 对外发布依据；不重开任何已 verified 判定。

## 4. 复现

```powershell
target\mjwarp-parallel\venv\Scripts\python.exe -B docs/development-evidence/render-bvh/r05-r09-r10-summary-check/scripts/qa_c_check.py C3 --out docs/development-evidence/render-bvh/r05-r09-r10-summary-check/logs/C3-check.log
```

## 5. 产物清单

- `README.md`（本文）、`facts.json`、`manifest.txt`
- `scripts/qa_c_check.py`
- `logs/C3-check.log`
