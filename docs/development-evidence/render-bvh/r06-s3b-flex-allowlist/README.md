# R06-S3b attempt5 判据 allowlist 签名记录（用户签名级，2026-11-04 22:14）

任务：`2026-11-04-render-r06-s3b-flex-allowlist-signature`（owner Arch；只承载签名治理与审计，不含实现）。
本文件是**用户签名级判据放宽**的审计记录；不具备本文件与窗口材料中的逐字范围时，任何验收结论均无效。

## 1. 签名事实（原文）

- `user_signed_at`：**2026-11-04 22:14**
- `signature_level`：**criterion relaxation（签名级）**
- `decision`：**签判据 allowlist**
- 选项原文：「最小范围：`cam2/(134,128)/flexid0/ULP≤4 且 beyond4=0`；证据与前提写死」
- 记录人：Chief（2026-11-04 22:16 指令 Arch 登记）；本卡不转述为其他 Bot 的裁定。

## 2. 最小范围（逐字，不得扩大）

| 维度 | 取值 |
| --- | --- |
| 相机 | **仅 `cam2`** |
| 像素 | **仅 `(x=134, y=128)`**（idx 41094） |
| flex 归属 | **仅 `flexid0`** |
| 放行条件 | **`ULP ≤ 4` 且 `beyond4 = 0`**（**beyond4 不为 0 即失败**，不得以「仍在 4 ULP 内」为由放行） |
| 判定来源 | 产品 `depth`（f32）与该像素参考 `depth` bits 的 ULP 距离 |

## 3. 其余一律保持 bit-first（不放宽）

- **rgb**：`mismatched = 0`、`max_byte_diff = 0`、`beyond_1_lsb = 0`（全部相机）；
- **depth**：cam0 / cam1 / cam2 的除 `(134,128)` 外**所有**像素 → `mismatched = 0`、`max_ulp ≤ 4`、`beyond_0_ulp = 0`；
- **seg**：全部相机**精确相等**（含 `(flexid, 9)` 样本）；
- **两跑 sha256**：相等。

## 4. 与 D8 的 321 px codegen 尾差分开登记、不混算

- D8（`2026-11-04-render-r06-s3b-flex-cam2-depth-gap`，`a1cc22ae`）记录的是 f32 次序修复**前** cam2 的 **321 px** 差异（beyond4 = 297、maxULP = 259）；
- f32 次序修复（r2，`scene.cu` mat_t_vec/mat_vec 显式舍入中间项）把其降到 **1 px**（即本 allowlist 覆盖的这一个）；
- **本 allowlist 只覆盖这 1 px**，不覆盖也不重新解释 D8 的历史 321 px 记录；两者在 evidence 中分开引用、数字不混算。

## 5. 不扩权 / 不二次放宽 / 不宣称

- **失败即停、不重试、不二次放宽**；出现 `beyond4 > 0`、或命中该像素之外任何新差异 → 本次 attempt5 判失败并按纪律留档报 Chief；
- 本放行**仅限 R06-S3b 本次 attempt5 验收**，**不得**推广到其它场景/实体/相机/像素，**不得**写入通用判据文本；
- **不得**据此对外宣称：上游等价（upstream equivalence）、parity pass、flex 渲染通过、R06-S3b 整体通过。

## 6. 引用

- QA 决策支持：`docs/development-evidence/render-bvh/r06-s3b-flex-f32-order-check/DECISION-SUPPORT-user-signature.md`（提交 `a3f690df`，最小范围与前提写死）；
- 窗口材料（attempt5 验收口径落点）：`docs/development-evidence/render-bvh/r06-s3b-flex-validate/materials/attempt5-criteria.md`；
- f32 修复证据：`docs/development-evidence/render-bvh/r06-s3b-flex-f32-order/`（r2 提交，未在本卡改动）。

## 7. 边界（本卡未做）

不写实现、不改产品/内核代码、不碰 r2 证据目录与 `scene.cu`/`render_flex.cu`、不改冻结判据其它文本、不动 `fixtures/`、未开 GPU 窗口、未重冻。
