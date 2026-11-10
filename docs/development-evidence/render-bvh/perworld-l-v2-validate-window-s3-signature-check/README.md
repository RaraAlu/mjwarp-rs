# QA 复核：S-3 签名留证卡（`ea91108e`；2026-11-11）

- 卡：`2026-11-11-render-perworld-l-v2-validate-window-s3-signature-check`（owner QA）
- 被复核：`2026-11-11-render-perworld-l-v2-validate-window-s3-signature`（Arch）；登记 **`426fadfb`** → 证据 **`ea91108e`**（parent `426fadfb`）
- 性质：**只读复核**；未改任何被复核内容（三份 receipt/manifest/README、S-3 materials 逐字节未动）；零 GPU、不采集、不开窗。
- 结论：**PASS（0 阻断，4 条信息级观察）→ 支持 Chief 发 G-L2 token**。**签名与 token 都不等于开窗**（见 §2.3）。

## 1. 复核方法（独立重算，不采信描述）

1. 用 `git cat-file` 从**父提交**取「用户签署时的 README blob」，独立算 sha256/bytes，与 receipt `signature.material_sha256` 比对（确认不是拿签名后新 sha 冒充签名对象）。
2. 从工作树、`HEAD:` blob、父提交 blob **三处**取 spec.md 独立算 sha256/bytes。
3. 独立重算三份 manifest（S-1 3 条 / S-2 12 条 / S-3 15 条）。
4. 逐字符比较三张 receipt 的 `signature_status` 三个键，并列出 S-1 `signatures_required` 全行。
5. 逐行看 `ea91108e` 对 S-1/S-2/S-3 receipt 的 diff（判断是否「只动签名相关行」）。
6. 扫所有 `NOT SIGNED / not signed / 未签` 命中并逐条判读是否当前态或历史记录。
7. 边界：`git diff --name-only` 全量路径 + 分路径计数（含 S-3 `materials/`）；编码卫生逐文件扫 BOM/CR/TAB。
8. 门禁：HEAD 上独立复跑 fmt / clippy / clippy-cuda / `cargo test --locked`；`normify check` 复核该卡。

## 2. 逐点结论

### 2.1 签名事实 —— 通过
- `status = signed_pending_window`、`signed_object = true`。
- `signature`：`signer=user`、`signed_at=2026-11-11`、`timezone=Asia/Shanghai`、`method=chat card selection (dual sign S-3: plan + signed floor 9,956)`。
- `scope` 逐项覆盖：**S-3a 计划**（guards **107 = 12 exe + 93 输入 + 2 条 v2.2 清单**、旧 suite/stash 只读、参考 5 文件 **11,078,239 B**、实体 `render_per_world_light` 最终 **`bab01507`** + QA 链 `7d089165`/`6a75ceee`/`be12145e`、判据 A/B/C 与 §2.4 对账 `check_capture_c1 == Compared`（`Skipped` ⇒ 停手）、冻结在 monitor PASS 后、body 零编译、`live==guard==stash`、单次失败即停）+ **S-3b**（`baseline_free 13,028 MiB` / signed floor **`9,956 MiB`**、**明确 NOT S-2 的 13,302/10,230**、开窗前 5 分钟内现场重测、`floor_effective = max(signed, 现场)` 更高更严）+ **旧 stash 漂移已知事实与裁定** + **失败条款**。

### 2.2 签名对象绑定（关键）—— 通过
| 项 | 值 |
| --- | --- |
| receipt `signature.material_sha256` / bytes | `437dc793d13d4f1bc997bd73a554f50b7351a7e8394b9fd6a2ad2f3d2edd816a` / 18,694 |
| 我从父提交 `426fadfb` 取 README blob 复算 | **`437dc793…` / 18,694 B（相等）** |
| 签名后工作树 README（当前 manifest 记录） | `ddb15baa…` / 21,718 B（追加 §10 版本记录 + 顶部签名状态行） |
| 两者关系是否写明 | `signature.note` + README §10 均明写「manifest 是新 sha、签名绑定旧 sha、变化属预期」✅ |

`does_not_open_window` 六项齐全：QA PASS → Chief G-L2 token → 开窗前 5 分钟内三点基线 + `floor_effective=max(9,956, 现场)` → monitor 60/60 util<40 → body 零编译 → 12 个冻结 exe `live==guard==stash`。

### 2.3 三处 `signature_status` 一致性（按实际，不按描述）—— 通过（含 1 条预存在差异）
- **S-1 行**：三张 receipt 逐字符一致。
- **S-3 行**：三张 receipt 逐字符一致（`signed 2026-11-11 … signed floor 9,956 MiB … does NOT open G-L2 …`）。
- **S-2 行**：**S-1 与 S-3 两处一致；S-2 自身 receipt 措辞不同**（`…(this signature card; plan + baseline 13,302 MiB / signed floor 10,230 MiB + authoring exception + failure clauses)`）。该差异由 **`2f361d91`（2026-11-08 S-2 签名卡）** 引入，**本卡未触碰**，两处事实一致（signed 2026-11-08 / 13,302 / 10,230）→ 观察 O1，不阻断。
- S-1 `signatures_required` 的 S-3 行已 = **SIGNED 2026-11-11**；S-1 receipt 的 diff **只有 2 个逻辑行**（`signature_status.S-3`、`signatures_required[S-3]`），S-1 已签事实、S-2 行、判据/测量字段逐字节未动；S-2 receipt diff 只有 1 行。

### 2.4 spec 字节未动 —— 通过
| 位置 | sha256 / bytes |
| --- | --- |
| 工作树 | `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / 39,379 B |
| `HEAD:` blob | 同上 |
| 父提交 `426fadfb` blob | 同上 |
| `ea91108e` 是否含 spec.md | **否** |

### 2.5 S-1 滞后措辞（越界防护正确）—— 通过
- S-1 receipt `non_claim[1]` 仍为 2026-11-08 旧句「S-1 signed and effective 2026-11-08; **S-2 and S-3 not signed**; …」——**本卡未改**（S-1 只改 2 行签名相关）。
- S-3 receipt `non_claim[5]` 已如实登记：该旧句**故意不编辑**（避免改写已验证证据），S-3 签名事实以三处 `signature_status` 与 `signature` 块为准 → 不构成误导；建议后续 fix 收（非阻断）。

### 2.6 两提交形态 —— 单个提交形态下无违规面
- `426fadfb`：**1 个文件**（卡记录），该时刻 `status=proposed`、`closed_at` 空 → 纯登记。
- `ea91108e`：**12 文件** = 卡（`proposed→verified` + `closed_at`）+ 4 门禁 runs + S-3 README/manifest/receipt + S-1 manifest/receipt + S-2 manifest/receipt。
- 结论：**不存在「新卡首次提交即 verified」**；verified 与证据同批落地，`normify check` 该卡「全部检查通过」，`normify status` 错误 0 警告 0。

### 2.7 边界 / 证据 / 门禁 —— 通过
- `ea91108e` 只含上述 12 文件；`src/`、`tests/`、`fixtures/`、`scripts/` 各 **0** diff；三个 perworld 证据目录外 **0**；**S-3 `materials/` 0 diff**（README §10「签名摘要保持签署前定稿原文」属实）。
- 三份 manifest 独立重算：**S-3 15/15、S-1 3/3、S-2 12/12 全对**；关键 sha/bytes 全部相符（S-3 README `ddb15baa…`/21,718、receipt `3262f72e…`/26,121、manifest `1b20d5b5…`/1,656；S-1 receipt `3698ce40…`/23,280；S-2 receipt `867a0fad…`/13,865）。
- 7 个被改证据文件 **BOM=False、CR=0、TAB=0**（LF）。
- 四门禁 HEAD 复跑**全部 exit 0**、unit **1387 passed / 0 failed / 402 ignored**（与 runs 记录一致）；`normify check` exit 0；`normify status` 已验收 **829**、运行 0。

## 3. 观察项（4 条，均信息级、不阻断）

1. **O1**：三处 `signature_status` 中 **S-2 行**在 S-2 自身 receipt 的措辞与 S-1/S-3 不一致（预存在，`2f361d91` 引入；本卡未改；无事实冲突）。若 Chief 要求「三块完全逐字符一致」，需另开 fix 并重算 S-2 manifest（属改已验证证据，建议谨慎）。
2. **O2**：S-3 receipt `title` 仍是 fill 轮文案「… remaining PENDING-FREEZE / PENDING-QA / PENDING-REFRESH」；PENDING-QA/REFRESH 已于 2026-11-09 填，`non_claim[2]` 已澄清，本卡未改该字段（预存在）。
3. **O3**：S-3 README 的 fill 轮小结（§8 附近）仍写「不构成 G-L2 启动：需实体 QA PASS + 用户 S-3 双签 + Chief G-L2 token + 窗口条件」——属 fill 轮历史口径；顶部签名状态行、§7、§10 已是已签口径，且三处反复写明「签名不构成开窗」，全文无矛盾陈述。
4. **O4**：`normify audit` 在本环境运行 **>9 分钟无任何输出**（命令存在，但 `--help` 报未知选项），已终止；两提交形态的合规性改用等价证据核（见 §2.6）。若需 audit 输出，建议单独跑并留痕。

## 4. 非声明

签名留证与记录一致性已核；**签名不构成开窗**。本卡不宣称采集有效、灯位姿验收、上游等价或任何渲染数值结论；G-L2 是否开窗由 Chief token + 现场条件（三点基线 / `floor_effective=max(9,956, 现场)` / monitor 60/60 / 零编译 / `live==guard==stash`）决定。
