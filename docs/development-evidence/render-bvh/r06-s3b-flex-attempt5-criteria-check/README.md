# R06-S3b attempt5 判据口径独立复核（用户签名 allowlist）——**PASS-with-observations（0 阻断）**

- 复核卡：`2026-11-04-render-r06-s3b-flex-attempt5-criteria-check`（owner QA，p80；**attempt5 开窗前必须 verified**）
- 复核对象（只读）：
  - 材料：`docs/development-evidence/render-bvh/r06-s3b-flex-validate/`（提交 `ecc93b56`）——`materials/attempt5-criteria.md` 3,934 B、README 11,208 B、`receipt.json` 9,548 B、`materials/window-plan.txt` 6,195 B、`materials/entity-table.json` 6,988 B；
  - 签名治理卡：`2026-11-04-render-r06-s3b-flex-allowlist-signature`（`3406f1ca`，**verified**）+ `docs/development-evidence/render-bvh/r06-s3b-flex-allowlist/README.md`；
  - QA 决策支持：`docs/development-evidence/render-bvh/r06-s3b-flex-f32-order-check/DECISION-SUPPORT-user-signature.md`（`a3f690df`，`ab35859d…`/3,871 B）。
- 纪律：只读、零 GPU、**未执行窗口**、未改任何对象证据、未改判据文本；写锁仅用于本证据落库。
- **结论：PASS-with-observations，0 阻断**。签名级 allowlist 在材料/签名卡两处**逐字收敛**、无扩大措辞；其余判据 bit-first 完整且有逐条阈值；D8 的 321 px 与本次 1 px 明确分开；冻结输入清单不含判据文件有明确回答；receipt 的 4 个材料文件 sha256/bytes 与当前文件一致；非宣称齐全。**可用于申请 Chief GO**（开窗前仍需按材料 P4–P7）。

## 1. 复核项与结果（脚本 `scripts/qa_criteria_check.py`，日志 `logs/qa_criteria_check.log`）

| # | 复核项 | 方法 | 结果 |
| --- | --- | --- | --- |
| 1 | 签名元数据 | 读签名卡 JSON + 签名记录 README | 卡 `verified`、owner Arch、`user_signed_at=2026-11-04 22:14`、`criterion relaxation`；9 条 acceptance |
| 2 | 最小范围（三处逐字） | 材料 / 签名 README / QA 决策支持逐项匹配 `cam2`+像素+`flexid`+`ULP≤4`+`beyond4=0` | 材料与签名 README **全部命中**；QA 决策支持命中相机/像素/ULP/beyond4，两处**同义不同写法**见观察 O1 |
| 3 | 抗扩大措辞 | 扫「≤4 ULP 即可」「邻近像素」「相邻像素」「adjacent/neighbor」 | 材料、材料 README、签名 README **0 命中** |
| 4 | 其余 bit-first 完整性 | 材料 §2 表逐条阈值 + README §9 | rgb `mismatched=0/max_byte_diff=0/beyond_1_lsb=0`；depth cam0/cam1 与 **cam2 除该像素** `mismatched=0/max_ulp≤4/beyond_0_ulp=0`；seg 精确相等（含 `(flexid,9)`）；两跑 sha256 相等；回归实体位级零漂移——**齐全** |
| 5 | D8 分开、不混算 | 材料 §3 + 签名卡 §4 + receipt | D8 `a1cc22ae` = 修复前 **321 px**（beyond4 297/maxULP 259）；f32 修复 `d35cab68` 后 **1 px**；声明「只覆盖这 1 px、不重新解释、不混算」 |
| 6 | 冻结清单是否含判据文件 | 材料 §5 + README §9 + receipt flag | 明确回答「**不含**」：`guard_inputs` 14 条只冻结代码/场景/参考/脚本；开窗前须记本文件与 README 的 sha256，文本被改即本次 attempt5 无效；receipt `frozen_input_list_includes_criteria=false` |
| 7 | receipt 材料指纹 | 逐文件 sha256+bytes 对当前文件 | **4/4 OK**：entity-table `575d049a…`/6,988、README `a3201050…`/11,208、attempt5-criteria `1ae0c97b…`/3,934、window-plan `849bb959…`/6,195 |
| 8 | 非宣称 / 不二次放宽 | 材料 §6 + 签名卡 §5 | 不宣称上游等价 / parity pass / flex 渲染通过 / R06-S3b 整体通过；失败即停、不重试、不二次放宽、仅限本次 attempt5 |
| 9 | 引用链未改 | 本次复核前后 sha | QA 决策支持 blob 保持 `ab35859d…`/3,871 B 未变（材料 §5 约束） |

## 2. 观察（不阻塞，建议 attempt5 之后处理）

- **O1（低）**：QA 决策支持是**签字前建议稿**，与签名后材料存在**同义不同写法**：`flexid=0` vs 材料 `flexid0`；`beyond4ULP = 0` vs 材料 `beyond4 = 0`；并含两条条件式备选（「若要写成族口径…」与「（或该子集）」）。材料/签名卡均采用**严格 1 px + 显式 `beyond4=0` 硬条款**，**未采用族口径、未扩大**。因材料 §5 规定「引用链被改动即本次 attempt5 无效」，本复核**不改该文件**；签名状态改由本证据与签名卡记录。建议 attempt5 结束后另立治理修订（重写该文档为签字后版本 + 标注签名时间/范围），**非开窗前**。
- **O2（低）**：签名「原文」为 Chief 转述（材料 §1、签名卡 §1「记录人：Chief」），仓内无独立用户消息快照；符合既往签名治理做法，仅记录以便审计。
- **O3（低）**：材料 §2 对 cam0/cam1 与 cam2-除该像素额外写 `max_ulp ≤ 4`——与冻结判据 `MAX_DEPTH_ULP=4`（`src/render/kernel/output.rs:15`）一致；在 `mismatched=0` 下属冗余但无害，**不构成扩大**。
- 附带提示（非本卡范围）：材料未引 tie 根因卡；若 GO 文案需要引用，应引**负结果卡提交 `9a1f102b`**（verified），旧的 `3e3f363d` 已 abandoned。

## 3. 结论与开窗前剩余项

**PASS-with-observations（0 阻断）**：本卡 verified 后即可向 Chief 申请 GO。开窗前仍按材料 §7 P4–P7 执行：本卡 verified → attempt5 执行卡 → 开窗前 5 分钟三点基线 + floor 取严 → 窗口规则 v2（一次 canonical rebuild → 记录 exe sha/bytes → `freeze -Attempt 1`（16/16、exes=2、inputs=14）→ 冻结后零编译）。

## 4. 非声明与边界

- 不宣称上游等价/parity pass/flex 渲染通过/R06-S3b 整体通过；不执行窗口、不重冻、不改判据/参考/`fixtures/`/对象证据。
- 只核「口径是否按用户签名的 allowlist 正确落点、其余是否保持 bit-first」；不评价产品实现与数值（那些由 attempt5 窗口实体判据回答）。

## 5. 复现

```powershell
# 仓库根执行；CPU-only，不触 GPU。--out 以 LF 写日志（字节确定）。
$ev = "docs\development-evidence\render-bvh\r06-s3b-flex-attempt5-criteria-check"
target\mjwarp-parallel\venv\Scripts\python.exe -B "$ev\scripts\qa_criteria_check.py" --out "$ev\logs\qa_criteria_check.log"
```

环境：Windows 11 / python 3.12.13 / numpy 2.5.4。

## 6. 产物清单

- `README.md`（本文）、`facts.json`、`manifest.txt`
- `scripts/qa_criteria_check.py`、`logs/qa_criteria_check.log`
