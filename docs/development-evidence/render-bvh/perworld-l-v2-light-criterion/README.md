# perworld L v2 灯判据规格（spec v3）证据目录

- 任务：`2026-11-07-render-perworld-l-v2-light-criterion-spec-v3`（owner Arch）；本目录由该卡创建，现由签名留证卡 `2026-11-08-render-perworld-l-v2-light-criterion-spec-v3-signature` 记录签名事实
- 状态：**用户已签 S-1（2026-11-08）、spec 已生效**；S-2（采集窗双签）与 S-3（验证窗双签）**尚未签署**
- 边界：规格与签名记录；本目录**不产参考、不采集、不开窗、零 GPU**

## 1. 文件清单

| 文件 | 说明 |
| --- | --- |
| `spec.md` | 判据规格（签名对象，**字节冻结**） |
| `receipt.json` | 机器可读回执：版本、supersedes、修订、事实地基、预审、**签名块**、**S-2 预批块**、判据/布局/GO/边界/非声明 |
| `README.md` | 本文件：版本链与签名记录 |
| `manifest.txt` | 本目录 `sha256 + bytes` 自核清单（首行 `#`，不自列） |

## 2. 版本链

| 版本 | 日期 | 说明 |
| --- | --- | --- |
| v1 | 2026-11-06 | 初版草案（sha256 `1821be98…` / 16,625 B） |
| v2 | 2026-11-06 | QA 读前预审 O1/O4/O5 文本修订 + S-2/采集窗前置（`6cfaa57e…` / 20,681 B） |
| v3 | 2026-11-07 | 用户拍板「rgb 变化 + 换相机都做」→ 双相机 + C1/C2 两套判别力（`4bfc521b…` / 32,970 B） |
| v3 rev2 | 2026-11-07 | QA 预审 O1–O3 签名前修订（`34bf4b0c…` / 37,346 B） |
| v3 rev3 | 2026-11-07 | QA 第二轮 2 条阻断项（相机条目数口径）+ 5 低观察修正（签名前） |
| **v3（签名版）** | **2026-11-08** | **用户签署 S-1，冻结为 `bb064fe0…` / 39,379 B / 344 行 / 15 节** |

## 3. 用户签名记录（2026-11-08）

| 项 | 值 |
| --- | --- |
| signer | user |
| signed_at | **2026-11-08**（Asia/Shanghai） |
| method | 聊天卡选择（「签 S-1 + 预批 S-2」） |
| scope | **S-1**：spec v3 sha256 / bytes / 344 行 / 15 节 |
| 签名对象 | `spec.md` sha256 **`bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d`** / **39,379 B** |
| 冻结自核 | **签名前后 sha256 逐位相同**：before `bb064fe0…`/39,379 B → after `bb064fe0…`/39,379 B（本卡提交前后各算一次，见 receipt `signature.spec_sha256_before/after`） |
| 备注 | `spec.md` 头注中的「待用户签名、未生效」字样按**字节冻结不动**；签名事实以本 README §3 与 `receipt.json.signature` 为准 |

## 4. S-2 预批（用户 2026-11-08，范围不得扩张）

> **预批「采集窗 authoring 例外」——常规 util<40 连续不达标时允许 authoring-only util<60；条件：① 仅采集窗生效，G-L0 pilot 仍按常规阈值、不适用例外；② 仍须 S-2 双签（计划 + 基线 + signed floor = baseline_free − 3072 MiB）后才开 G-L1；③ 失败即停、不重试、不放宽；④ 开窗前 5 分钟内三点基线 + monitor 60/60 + 零编译仍全适用。**

- 该预批**仅**覆盖「采集窗（G-L1）的 authoring 例外」；不豁免 S-2 双签、G-L0 常规阈值、G-L2 验证窗规则、失败即停纪律，也不改任何判据/阈值/容差。
- S-2 签名清单仍为三项：S-1（已签）、S-2（采集窗计划 + 基线 + signed floor 双签，**未签**）、S-3（验证窗双签 + Chief token，**未签**）。

## 5. 签名前后 spec 自核证据

```
spec.md BEFORE: sha256 bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d  bytes 39379  lines 344
spec.md AFTER : sha256 bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d  bytes 39379  lines 344
identical     : true
```

## 6. 边界与非声明

- 本目录只放规格与签名记录：不采集、不开窗、零 GPU、不写 `fixtures/`、不改 `src/`/`tests/`/`scripts/`、不改 criterion v2.2。
- **不宣称**：不宣称采集有效、不宣称 per-world 灯位姿已覆盖、不宣称与上游实现等价、不宣称任何渲染数值结论；§3 的两套判别力数值是 CPU 预测量，真渲染实测以 pilot/采集窗为准。
- 「S-1 已签」只意味着判据规格生效，**不等于** G-L0/G-L1/G-L2 已启动或通过。
