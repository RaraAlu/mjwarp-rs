# spec v3 签名留证复核（QA 只读）

任务：`2026-11-08-render-perworld-l-v2-light-criterion-signature-check`（owner QA）。日期：2026-11-08。
复核目标：Arch 卡 `2026-11-08-render-perworld-l-v2-light-criterion-spec-v3-signature`（verified；登记 `d2ce2ef1` → start `d72fb0b0` → 落库 **`023c69fa`**）。
复核时 HEAD = `3d546fbe`（场景/相机改卡，`023c69fa..HEAD` 仅新增卡 JSON，签名证据未被触及）。
边界：只读核验 + 只写本卡证据目录；零 GPU；**不改签名记录**（发现问题只报告 + 建议 fix）。

**结论：PASS-with-observations（0 阻断；4 条低/中观察，建议后续 fix 轮顺手收）。**

## 1. spec 冻结（逐项独立重算）

| 项 | 实测 | 判定 |
| --- | --- | --- |
| 工作树 `spec.md` sha256 | `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` | ✅ |
| `HEAD:spec.md` blob sha256 | 同上（blob `3e68178b…`，39,379 B） | ✅ |
| bytes / 行数 / 节数 | 39,379 B / 344 行（LF，CR=0，TAB=0，无 BOM）/ 15 节 | ✅ |
| `023c69fa` 是否含 `spec.md` | 否（name-status 仅卡 + runs + 证据目录 3 文件） | ✅ |
| `d2ce2ef1..023c69fa` spec diff | 空 | ✅ |

## 2. receipt 真实性

- 新 sha256 **`7f79bdf1…`/21,935 B**；旧 `8cc26b83…`/19,287 B（由 `d2ce2ef1:` blob 独立重算，与派工数字一致）。
- `status=effective`、`effective=true` ✅。
- `signature` 块齐备：signer=user、signed_at=**2026-11-08**、timezone=Asia/Shanghai、method=聊天卡选择、scope=S-1（含 sha + 39,379 B + 344 行 + 15 节）、`spec_sha256_before == spec_sha256_after`（identical）、`spec_frozen=true`、note 明写「spec 头注仍是 pending user signature，按字节冻结不动，签名事实以 receipt/README 为准」。
- `signature_status`：S-1 已签 / S-2 未签（注明仅预批 authoring 例外）/ S-3 未签 ✅。
- 新旧 receipt diff **仅** status/effective 两行 + `signature`/`s2_preapproval`/`signature_status` 三个新块；判据、阈值 384/576、容差、事实地基、布局、GO、边界内容**零改动**（逐 hunk 核对）。

## 3. S-2 预批范围（阻断级检查项）

逐条在位且未被扩张：

- **仅采集窗（G-L1）生效**；**G-L0 pilot 仍按常规阈值（util<40）、不适用例外**；
- **仍须 S-2 双签**（计划 + 基线 + `signed floor = baseline_free − 3072 MiB`）后才开 G-L1；
- **失败即停、不重试、不放宽**；
- **开窗前 5 分钟内三点基线 + monitor 60/60 + 零编译**仍全适用；
- `not_extended` 明示**不豁免** S-2 双签 / G-L0 常规阈值 / G-L2 验证窗规则 / 失败纪律 / 任何判据·阈值·容差。

**未发现任何扩张或软化措辞**（扫描 `豁免/放宽/免签/无需/可省/取消/waive/relax/exempt` 逐条核对，均为「不豁免」「no relaxation」等否定用法）。

## 4. manifest 与编码

- `manifest.txt` **3/3** 自核（证据目录相对路径）：spec `bb064fe0…`/39,379、receipt `7f79bdf1…`/21,935、README `c4f29297…`/4,109；首行 `#`、含 `# base dir:`、不自列 ✅。
- 三文件 UTF-8/LF/无 BOM/裸 CR·TAB 0 ✅。

## 5. diff 面

`d2ce2ef1..023c69fa`：`.normify/changes` 卡、`.normify/runs`（4 个门禁 JSON）、证据目录 README（新增）/receipt/manifest。`src/`、`tests/`、`fixtures/`、`scripts/` **零 diff** ✅。

## 6. 非声明与版本链

- README §6 明示不宣称采集有效、不宣称 per-world 灯位姿已覆盖、不宣称上游等价、不宣称渲染数值结论；「S-1 已签」≠ G-L0/G-L1/G-L2 启动 ✅。
- 版本链 v1 → v2 → v3 → rev2 → rev3 → **签名版（2026-11-08）** 完整，各版 sha/bytes 在位；spec §14.1 五行修订记录与之一致（签名版不新增 spec 修订行，因 spec 字节冻结）✅。

## 7. 门禁（本卡独立复跑）

fmt / clippy / clippy-cuda / unit 全过；**unit 1359 过 / 0 败**（记录见 `.normify/runs/2026-11-08-render-perworld-l-v2-light-criterion-signature-check/`）。

## 8. 观察项（按严重度）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | **中** | receipt 仍带**签名前语义残留**，与 `status=effective` 自相矛盾：`kind="criterion-spec-draft"`、`title` 含 "signable draft"、`non_claim[0]="draft only; not effective until the user signs"` | 后续 fix 轮把 `non_claim[0]` 改为「S-1 于 2026-11-08 生效；S-2/S-3 未签」，`kind`/`title` 更新或加「签名前标签保留」注记；**不改签名事实**。当前不阻断（签名事实与生效态字段均正确），但对外引用时可能被误读 |
| O2 | 低 | 预批原文含「常规 util<40 **连续**不达标时」；该「连续」限定词不在 spec §11/S-2 与派工口径中，属签署记录对用户话术的新增限定 | 若用户原话无「连续」，建议 fix 为逐字；若有，建议在 README 注明为用户原话。只会收紧不会放宽，故不阻断 |
| O3 | 低 | `signatures_required` 仍把 S-1 列为待办项（未标「已签」），与 `signature_status` 口径不完全一致 | 可与 O1 同批 fix |
| O4 | 信息 | spec 头注仍为「待用户签名、未生效」（按冻结不改，已在 `signature.note` 与 README §3 说明） | 不修；后续任何引用 spec 的场合必须同时引用 receipt `signature` 块，避免只引 spec 造成「未生效」误读 |

## 9. 非宣称

- 本复核只判签名留证证据的**真实性/一致性/范围**，不判 spec 判据内容（那由此前预审与用户签名负责）。
- 不构成 G-L0/G-L1/G-L2 启动或通过；不宣称采集有效、灯位姿覆盖、上游等价、任何渲染数值结论。
- 未改任何签名记录、spec 字节、receipt 内容与既有卡状态。
