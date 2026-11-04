# R06-S3b attempt5 验收口径（用户签名 allowlist，2026-11-04 22:14）

本文件是 attempt5 的**判据落点**；权威链 = 本文件 + 签名治理卡 `2026-11-04-render-r06-s3b-flex-allowlist-signature`（`3406f1ca`，verified）+ QA 决策支持 `docs/development-evidence/render-bvh/r06-s3b-flex-f32-order-check/DECISION-SUPPORT-user-signature.md`（`a3f690df`）。

## 1. 签名事实

- 用户签名：**2026-11-04 22:14**，签名级（criterion relaxation）；
- 决定：**签判据 allowlist**；
- 原文：「最小范围：`cam2/(134,128)/flexid0/ULP≤4 且 beyond4=0`；证据与前提写死」。

## 2. 判据（逐条，不得扩大）

| 面 | 判据 |
| --- | --- |
| rgb（3 相机全部） | bit-first：`mismatched = 0`、`max_byte_diff = 0`、`beyond_1_lsb = 0` |
| depth（cam0 / cam1） | bit-first：`mismatched = 0`、`max_ulp ≤ 4`、`beyond_0_ulp = 0` |
| depth（cam2，**除 `(134,128)` 外全部像素**） | bit-first：`mismatched = 0`、`max_ulp ≤ 4`、`beyond_0_ulp = 0` |
| **depth 例外（唯一）** | `cam2 / (x=134,y=128) / flexid0`：**`ULP ≤ 4` 且 `beyond4 = 0`**；**beyond4 ≠ 0 即失败**（不得以「仍在 4 ULP 内」放行） |
| seg（3 相机） | 精确相等（0 diff px，含 `(flexid,9)` 样本） |
| 两跑 | sha256 相等 |
| 回归实体 | 既有 render 实体集 / scene-a 等位级零漂移 |

## 3. 与 D8 分开、不混算

- D8（`a1cc22ae`）= f32 修复**前** cam2 的 **321 px**（beyond4 297 / maxULP 259），已由 f32 次序修复（`d35cab68`）降到 **1 px**；
- 本 allowlist 只覆盖**这 1 px**；D8 的 321 px 记录**不重新解释、不混算**。

## 4. 窗口规则 v2（Chief 2026-11-04 批准）

开窗第一步：**一次** canonical rebuild（`cargo test --locked --features cuda-probe --no-run --test render_flex_validate`）→ 确认无构建进程 → **立即记录该次 exe sha256+bytes（+ 离线 PTX sha/bytes）** → 随即 `freeze -Attempt 1`（期望 `GUARDS_OK=16/16 exes=2 inputs=14`、aggregate 重算、exe 条目不再 pending）→ **冻结后零编译**跑 monitor 与实体体；**冻结后任何编译/relink 即判 INVALID_ROUND**。预录 sha 仅参考（debug test exe 不可预录，见材料 §6 与 `artifacts/fingerprint-canonical-rebuild.json`）。

## 5. 冻结输入清单是否含判据文件（明确回答）

**不含**。`guards-r06-s3b-flex.json` 的 `guard_inputs`（14 条）冻结的是**代码/场景/参考数据/采集与冻结脚本**的 sha+bytes，**不包含本判据文档**；判据的权威链是「签名卡 + 本材料 + QA 决策支持」三者，任何一方被改动即视为本次 attempt5 无效（需重新签/重冻）。
开窗前必须记录**本文件与 README 的 sha256**，并与签名卡提交一致；若开窗时发现文本被改动 → 停手报 Chief。

## 6. 失败即停 / 不二次放宽 / 不宣称

- 出现 `beyond4 > 0`、例外像素之外任何新差异、rigid 位差、seg 差异、两跑 sha 不等 → **立即停手、不重试、不二次放宽**，attempt 原文留档报 Chief；
- 本放行**仅限本次 attempt5**，不得推广到其它场景/实体/相机/像素，不得写入通用判据文本；
- **不得**据此宣称上游等价、parity pass、flex 渲染通过或 R06-S3b 整体通过。

## 7. attempt5 剩余前置（开窗前逐条核）

| # | 前置 | 状态 |
| --- | --- | --- |
| P1 | f32 次序修复 verified（`d35cab68`） | ✅ |
| P2 | 本材料（attempt5 口径 + 签名）收口 | 本文件（提交后 ✅） |
| P3 | 签名治理卡 verified（`3406f1ca`） | ✅ |
| P4 | QA 对本口径的复核（含「签名级放宽仅此 1 px」确认） | 待 |
| P5 | attempt5 执行卡（含 canonical rebuild + 冻结 16/16） | 待启动 |
| P6 | **Chief GO** + 开窗前 5 分钟三点基线 + floor 取严 | 待 |
| P7 | 冻结后零编译纪律确认（窗口规则 v2） | 开窗时执行 |
