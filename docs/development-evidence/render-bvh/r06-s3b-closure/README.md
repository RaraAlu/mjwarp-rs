# R06-S3b 验收收口证据（attempt6 VALID_ROUND / PASS 接受）

任务：`2026-11-04-render-r06-s3b-closure`（owner Arch；只写证据，不改代码/判据/attempt6 证据）。
依据：Chief 2026-11-04 裁**选项①**——attempt6（`7a48205d`）**PASS 接受**，不做 attempt7 复跑；缺的数值明细由跨轮证据闭环补齐（见 §4）。
**本文件是 R06-S3b 的验收收口记录**；结论范围与前提见 §1、§5、§6。

## 1. B 类前提声明（参考型判据的成立前提）

R06-S3b 的渲染参考比对属**参考-前提型（B 类）**判据：结论只在下列前提同时成立时有效，任一前提失效 → 需重跑/重判。

| # | 前提 | 取值 |
| --- | --- | --- |
| P1 | **冻结参考**（`fixtures/render-bvh/scene-flex/reference/base/`） | `cam-poses.json 9b6eabf2…`（3,987 B）、`capture.json 4713e2c7…`（16,166 B）、`depth.npz c3ee4adc…`（787,918 B）、`rgb.npz 5ed50287…`（2,362,306 B）、`seg.npz f3b00fa8…`（1,575,106 B）、`state.npz 7f2fff50…`（1,748 B）——本收口提交时逐字节复算一致 |
| P2 | **同 GPU** | RTX 4070 Ti SUPER / driver 596.36 |
| P3 | **同 recipe** | `use_fast_math=false`、`use_shadows=true`、`samples_per_pixel=1`、`flex_render_smooth=true`、`use_precomputed_rays=true`、`render_skybox=false`、`enable_backface_culling=true`（与采集 provenance 一致） |
| P4 | **同编译形态** | attempt6 **窗口内 canonical rebuild** 产物：`render_flex_validate` **`e48c7268…` / 107,854,859 B**（23:03:33 同刻记录）、`mjwarp-rs` **`c5014d83…` / 85,041,741 B**；冻结 `attempt=2 status=frozen`、`16/16 exes=2 inputs=14` |
| P5 | **同一判定实现** | allowlist 断言实现提交 `8c15ad75`（只动 `tests/`，见 §4a） |

## 2. 用户签名判据放宽（签名级）

- 签名卡：`2026-11-04-render-r06-s3b-flex-allowlist-signature`（提交 `3406f1ca`，verified；用户 **2026-11-04 22:14** 签名级裁定）；
- **最小范围（逐字）**：仅 `cam2 / (x=134, y=128) / flexid0` 允许 `ULP ≤ 4` **且** `beyond4 = 0`；**`beyond4 ≠ 0` 即失败**；
- **其余全部 bit-first**：cam0 / cam1、cam2 除该像素外全部像素、所有 seg、所有 rgb；
- 不得扩大范围、不得二次放宽；仅限 R06-S3b 本次验收。

## 3. 与 D8 的 321 px 分开登记（不混算）

- **D8**（`2026-11-04-render-r06-s3b-flex-cam2-depth-gap`，`a1cc22ae`）记录的是 f32 次序修复**前** cam2 的 **321 px**（`beyond4 = 297`、`maxULP = 259`、`seg 0/76,800`）——历史诊断记录；
- **f32 修复 r2**（`d35cab68`）把其降到 **1 px**（`beyond4 = 0`）；
- **attempt6 的 PASS 建立在该 1 px 走签名 allowlist 之上**；两条记录数字分开引用、不重算、不重解释。

## 4. 跨轮明细闭环（三条证据链，均可独立验证）

### a) 判定实现改动未触及渲染器源码
```
git diff --name-only d35cab68..8c15ad75 -- src/     →  （空输出）
```
即 `8c15ad75`（allowlist 断言实现）只改 `tests/`；渲染器行为与 r2（`d35cab68`）一致。

### b) 两跑输出与明细采集轮逐位相同
attempt6 用例打印的两跑 sha256：
- `rgb   = d84ecaedfa839eadb47a7cb21fc0156805b6602ec0c52b1c69455936e287a4f9`
- `depth = 9829215335d2cb2fd22b02a4d4a8354cc1a5db80c806b94a8888ef632c717326`
- `seg   = c4be985324b883abcbaa63fa44888b016fd28d2d266858aaa493877b49a64185`

与 r2 / D8 采集轮记录**逐位相同** → attempt6 的渲染输出与「有逐像素明细采集的那一轮」是同一输出，故 r2 证据中的像素明细对本轮直接有效。

### c) r2 证据已含该像素明细
- 像素：`(x=134, y=128)` / idx **41094** / `flexid0`；
- 产品 bits **`0x400015ba`**、参考 bits **`0x400015b7`**、**ULP = 3**（≤4）、**beyond4 = 0**；
- `seg` 双方 **`[0, 9]`**；参考侧 cam2 flex 覆盖 **654 px**（`flexid0` 403 + `flexid1` 251）。

## 5. 验收结论

- **R06-S3b attempt6 = VALID_ROUND / PASS**（用例断言层面：rgb bit-first、depth「除签名 allowlist 外 bit-first」、seg 精确、两跑 sha 相同，全部通过；`running 1 test → 1 passed`）；
- **冻结与纪律**：窗口规则 v2 全程合规（窗口内一次 canonical rebuild → 同刻记指纹 → `freeze -Attempt 2` 16/16 → **冻结后零编译/零 relink**）；monitor 60/60 连续干净（util 2–7%、free_min 14,414 ≥ floor 11,688）；
- **不含**：逐相机 `(mismatched,beyond4,max_ulp)` 与 allowlist 像素明细的**运行时打印**（用例通过时不打印）；该缺项由 §4a–c 跨轮闭环覆盖，并已登记后续增强卡 `2026-11-04-render-r06-s3b-flex-attempt6-stats-print`（backlog `5dc62d66`，不阻塞收口）。

## 6. 非声明

- **不宣称上游等价（upstream equivalence）**；
- **不宣称 parity pass**；
- **不宣称 flex 渲染整体可用 / R06-S3b 可对外发布**；本收口仅作为项目内部验收记录；
- 签名 allowlist 仅限 R06-S3b 本次验收，不得推广到其它场景/实体/相机/像素；
- D5/D7/D8/tie（`9a1f102b`）负结果与相应修复均已分别留档，本收口不重新解释其结论。

## 7. 证据索引

| 对象 | 提交 | 用途 |
| --- | --- | --- |
| attempt6 窗口证据 | **`7a48205d`** | PASS 原始记录（canonical rebuild 指纹、freeze attempt2、monitor、entity log、two-run sha） |
| f32 次序修复 r2 | **`d35cab68`** | cam2 depth 321 → 1 px；像素明细来源 |
| allowlist 断言实现 | **`8c15ad75`** | 用例实现签名 allowlist（只动 `tests/`） |
| allowlist 实现 QA 复核 | **`1927d62e`** | PASS-with-observations、0 阻断 |
| 签名治理卡 | **`3406f1ca`** | 用户 22:14 签名级裁定、最小范围 |
| attempt5 材料（口径） | **`ecc93b56`** | attempt5/6 判据落点 |
| D8 诊断（321 px） | **`a1cc22ae`** | f32 修复前历史记录 |
| tie 负结果 | **`9a1f102b`** | 固定叶序/遍历不可行（已 abandon 尝试的留档） |
| attempt5（历史） | **`6e9aa585`** | FAILED_ROUND（判据实现缺口，非数值红） |
| attempt6 计划 | **`0a176abf`** | 窗口计划（canonical rebuild 同刻指纹、冻结 -Attempt 2、floor 11688） |
| attempt6 统计打印 backlog | **`5dc62d66`** | 用例通过时打印逐相机/allowlist 统计（backlog `…-attempt6-stats-print`，不阻塞收口） |
