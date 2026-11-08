# QA S-3 完整预审（sign-off 环）：验证窗材料 `aba83a97`

- 任务：`2026-11-09-render-perworld-l-v2-validate-window-s3-prereview`（owner QA、p85、P8-渲染与BVH）
- 对象：S-3（G-L2）验证窗材料最新态 **`aba83a97`**（HEAD；parent `6365a7c5`）；fill 卡 `…-validate-window-s3-fill`（verified）
- 依据：已签 spec v3 `bb064fe0…`/39,379 B §10.1/§10.2/§12 G-L2/§4/§5/§6/§8/§9.3；S-1 签名 `…-spec-v3-signature`
- 方法：逐条对 spec 原文 + 独立复算 + 独立审冻结器（含自建 PowerShell aggregate 复算与 dry-run SELFCHECK 复跑）+ 边界/门禁
- 基线：HEAD `aba83a97`、树 clean、锁 FREE；零 GPU、不采集、不开窗

## 0. 结论

**PASS（0 阻断）→ 可发用户 S-3 双签卡。**

- 前置与锚点全绿：S-1 已签、G-L1 收口 `6951c3fe` + 采后复核 PASS（`ef1bcce3`/`6365a7c5`）、实体最终 `bab01507` + QA 链 `7d089165`/`6a75ceee`/`be12145e`、pilot `pass-margin`。
- guards 算术 **107 = 12 exe + 93 输入 + 2 条 v2.2 清单**（旧 96 + 1 新 exe + 10 必需，可选 0）；参考 5 文件实测总 **11,078,239 B**；§2.4 实测计数齐；S-3b suggested `13,028 → 9,956`（明确不复用 S-2）。
- 冻结器独立审通过：dry-run 不构建、`-Execute` 唯一构建点、`live==guard==stash`、旧 suite/stash 只读、漂移默认拒绝并记录、SELFCHECK **13/13** 复现；QA 自建脚本独立复算旧 96 条 aggregate = **`8a4ddc50…`**（与旧 suite 记录一致）。
- 旧 stash 漂移裁定三处一致且与 spec 只读声明不冲突。
- 签名摘要可签未签（suggested、不构成开窗、NOT signed），无「已签/可开窗」错述。
- 边界干净：本批只动材料/卡/runs；`scripts/` 零改动（冻结器 sha 未变）、`src/fixtures/tests` 零 diff；manifest **15/15**；四门禁 unit **1387/0/402**；无窗口卡登记、零 GPU。
- 观察项：O1（低）receipt `non_claim` 有一句「PENDING-QA/PENDING-REFRESH 未填」的过时表述（实际已填）；建议签名前 1 行修，不阻断。

## 1. 前置与锚点（逐项可核）

| 项 | 状态 | 锚点 |
| --- | --- | --- |
| S-1 已签 / spec 冻结 | ✅ | `bb064fe0…`/39,379 B；S-1 signature `ba420eb6`（QA `5e82feed`） |
| §11.1 六项 + pilot | ✅ | 见材料 P1–P6 与 P7 行；pilot `pass-margin`（C1 38,285/40,644；C2 35,828/33,883、16,041/16,754） |
| **G-L1 采后复核 PASS** | ✅ | 卡 `2026-11-09-…-capture-window-g-l1-check`（verified）；证据 `ef1bcce3`、措辞更正 HEAD `6365a7c5`；三方复算一致、参考 11,078,239 B |
| **实体最终态** | ✅ | 实现 `bab01507`；QA 链 `7d089165`/`6a75ceee`/`be12145e`；`tests/*` 自 `be12145e` 起零改动（478/499/487 行）；判据 A/B/C 与阈值未动 |
| guards | ✅ | 107 = 12 + 93 + 2（旧 96 + 1 + 10，可选 0）；旧 suite `ca3ecd5c…`/38,659 B/96 条 |
| 参考 | ✅ | 5 文件实测（rgb `602574c2…`/5,531,026、depth `82eec6da…`/1,844,650、seg `c3f09b16…`/3,687,826、state `a1c5a321…`/2,486、capture.json `8b53e122…`/12,251）= **11,078,239 B** |
| S-3b | ✅ | suggested `baseline_free=13,028`、`signed floor=9,956`（2026-11-09 00:42 三点实测）；**不复用 S-2 的 13,302/10,230** |

## 2. 窗口纪律与实体期望（写死核对）

- 冻结在常规 monitor PASS 后、body 零编译；`cargo test --no-run` **只在冻结阶段一次**并留 `cargo-artifacts.jsonl`；`live==guard==stash`（12 条 path/bytes/sha）；三点基线 5 分钟内；`baseline_free=min(三点)`、`floor_effective=max(signed,现场)`；monitor 60/60 util<40（`compute_filtered=0`、`build=0`）；**authoring 例外不适用验证窗**；树 clean；单次失败即停；参考体积累计 —— 全部写死。
- 实体期望：A（两台相机逐 world rgb 0 字节差/depth 位级/seg 全等/两跑 sha）、B（灯字段位型 0 例外）、C（C1/C2 两对 ≥576；[384,576) 停手；<384 拒）、**§2.4 对账且 `check_capture_c1` 必须 `Compared`（`Skipped`=停手）**、world 隔离、诊断只记录、v2.2 非继承、C2 依赖 refit。
- G-L1 实测计数作为对账基准被写入材料（C1 magnitude 38,285/40,644、bitwise 43,604/41,815；C2 depth 35,828/33,883、seg 16,041/16,754；诊断 0/0、50,363/52,329、54,380/54,250）。

## 3. 冻结器独立审（`14d82190…`/20,802 B/非空行 360；ASCII-only）

| 项 | 结果 |
| --- | --- |
| 默认 dry-run | 无 cargo/构建/stash/guards 写入，只写 `target/…/freeze-plan-dryrun.json`；本卡复跑 **SELFCHECK PASS 13/13**（`artifacts/freezer-dryrun-rerun.log`） |
| `-Execute` | 唯一构建点：`cargo test --locked --no-run --message-format=json` 一次 + `cargo-artifacts.jsonl`；解析 12 exe → stash → `live==guard==stash` → aggregate → guards JSON → 旧 suite sha 前后复查 → round-trip 校验 |
| 旧 suite 只读 | 冻结前记录并复查 `ca3ecd5c…`；冻结后不同即 throw；`OldSuiteSha256` 参数写死 |
| 旧 stash 漂移 | 只报不修；`-Execute` 无 `-AllowOldStashDrift` 时 throw；带 flag 时 drift（recorded/actual/last_write）写入 guards JSON `old_stash_drift`；窗口证据记录理由 |
| aggregate | QA 自建 PowerShell 脚本对旧 96 条独立复算 = **`8a4ddc50be2d8e766431533e6ccd8ab4cf5082fdbd63d6638a44b128e9ad4b3e`**（路径排序 + 无分隔拼接 + sha256），与旧 suite 记录一致（`artifacts/aggregate-recompute.txt`） |
| 规模 | 369 总行 / **360 非空行**、20,802 B、ASCII-only（PS 5.1 ANSI 兼容） |

## 4. 旧 stash 漂移裁定口径（三处一致）

- 材料 §2.4/§7/§8、`signature-summary.txt` Known facts、`receipt.guards.old_suite.ruling` 一致：**预存在已知项**；旧 stash 仅历史只读参照、**不作本次证据**；G-L2 重建并重新 stash 12 exe；旧 guards JSON 不得改动（`ca3ecd5c…`/38,659 B）；冻结可带显式 `-AllowOldStashDrift` 并留痕；QA 预审确认该口径。
- 实测：旧 stash 11 文件；漂移仅 `render_shadow_reference.exe`（记录 `ce5139…`/101,102,921 vs 实际 `4500bb…`/100,859,152；last write 2026-11-03T23:09:54+08:00）。
- 与 spec「旧 suite 只读保留、不得复用为本次证据」不冲突（旧 stash 不作证据、不修改、仅历史参照）。

## 5. 签名摘要可签性

- `S-3a`：单次验证窗、执行者 Backend、guards 107（12+93+2，可选 0）、参考 11,078,239 B、实体 `bab01507` + QA 链、§2.4 `Compared` 要求、冻结/零编译/`live==guard==stash`/失败即停、旧 suite/stash 只读与漂移裁定。
- `S-3b`：`suggested` `13,028 → 9,956`；现场 5 分钟内重测；`floor_effective=max(signed,现场)`（更高更严）；**不复用 S-2 的 13,302/10,230**。
- 失败条款、证据/QA 安排、「Signs NOTHING here / NOT signed / signing does not open the window」在位；README §7 明示「定稿待审、不得据此声称已签」。全文无「已签/可开窗」错述。

## 6. 观察项

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低（不阻断） | `receipt.json.non_claim[2]` 仍写「PENDING-FREEZE / PENDING-QA / PENDING-REFRESH values are not filled yet」——PENDING-QA/REFRESH 本轮已填（`pending` 数组与 README §8 正确，仅此句过时） | 签名卡发出前 1 行修（走 fill 卡 addenda 或 `normify fix`），避免用户/审计误读 |
| O2 | 信息 | 冻结器「360 行」按非空行计（总行 369，含注释/空行） | 引用时注明口径 |
| O3 | 信息 | `PENDING-FREEZE` 26 处 token 全部指向冻结阶段（12 live exe sha/bytes+stash、aggregate、guards JSON sha），落点明确；其余 `PENDING-QA`/`PENDING-REFRESH` token 均为已填记录/提及，非占位 | 无需改 |

## 7. 最终可粘贴签名卡字段（建议）

- **窗口**：G-L2 per-world L v2 灯验证窗（单次、失败即停）
- **依据**：spec v3 sha256 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / 39,379 B（S-1 已签 2026-11-08）；G-L1 收口 `6951c3fe`（QA 采后复核 `ef1bcce3`/`6365a7c5` PASS）
- **S-3a（计划）**：执行者 Backend（唯一写者）；guards 冻结 **107 条** = 12 exe + 93 输入 + 2 条 v2.2 清单（旧 96 + 1 新 exe `render_per_world_light` + 10 必需输入，可选 0）；参考 5 文件 **11,078,239 B**；实体 `bab01507`（QA 链 `7d089165`/`6a75ceee`/`be12145e`），判据 A/B/C 与阈值不变，§2.4 对账要求 `check_capture_c1 = Compared`（`Skipped` = 停手）；冻结在 monitor PASS 后、body 零编译、`live==guard==stash`、旧 suite/stash 只读
- **S-3b（suggested，开窗重测、stricter = higher wins）**：`baseline_free 13,028 MiB`；`signed floor 9,956 MiB`；`floor_effective = max(signed, on-the-spot)`（不复用 S-2 的 13,302/10,230）
- **已知事实**：旧 stash `render_shadow_reference.exe` 漂移为预存在已知项（只报不修、不作本次证据；G-L2 重建 12 exe；冻结可带显式 `-AllowOldStashDrift` 并留痕；旧 guards JSON 不改）
- **失败条款**：任一判据/计数/元数据/legacy/冻结件/编译纪律/monitor 失败 → 停手报用户；不重试、不改阈值/参考/相机、不换冻结件
- **签名不构成开窗**：仍需 QA PASS + Chief G-L2 token + 现场 floor 取更高 + monitor 60/60 + 零编译

## 8. 非声明

本卡只做材料预审：PASS 表示材料可供用户 S-3 双签；**不宣称** G-L2 可开（仍需 QA PASS + user S-3 双签 + Chief token + 现场条件）、不宣称采集有效/灯验收/上游等价/任何渲染数值结论。未修改被审材料、参考、spec 或签名记录。
