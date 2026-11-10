# G-L2 后置 errata 批（body env 契约 / guards body_env / dump 字段表述 / 修复卡-1 证据）

- 任务：`2026-11-11-render-perworld-l-v2-gl2-post-errata`（owner Backend；p80；P8-渲染与BVH）。依据 Chief 2026-11-11 05:53 派工：G-L2 收口后置 errata 批，**纯文档/证据，零 GPU、零代码改动**。
- 边界：不改任何 `.cu`/`.rs`、`tests/`、`fixtures/`、`scripts/`、guards/冻结件、已签 S-1 spec（`bb064fe0…`）与 S-2/S-3 签名材料正文；不改判据/参考/容差；不重跑、不重冻、不采集。
- 本批四项：

| # | 项 | 落地物 |
| --- | --- | --- |
| 1 | body 环境契约（PATH + criterion v2.2 outline 成对 env + pre/post 列表校验）+ 两次 INVALID_ROUND 归因 | `perworld-l-v2-validate-window-g-l2/body-env-contract-errata.md`（新增）；该目录 README 追加 §11 指针 |
| 2 | guards `body_env` 缺字段（只写文档，不改冻结器/guards JSON） | 同上 §3 + `artifacts/guards-json-body-env-audit.txt` |
| 3 | dump 字段表述 `geom_type` → `objtype`（`seg[1]=mjOBJ_GEOM(5)`；真实命中 geom 3 = 模型 `geom_type=7` MESH）；代码字段改名只记 backlog | `perworld-l-v2-entity-diagnostic-dump/README.md` 更正 + 追加 §5 errata |
| 4 | 修复卡-1 证据 errata：`logs/src.diff` LF 复核（已 LF、字节未变）+ README 两条 errata（P2 被卡-2 R1 撤销；+17/−3 与提交信息 +37/−3 口径） | `perworld-l-v2-ptx-order-pins/README.md` §5；`artifacts/ptx-order-pins-src-diff-eol.txt` |

## 1. 改动文件（old = 本批登记后、编辑前 HEAD `bb29e351`；new = 本批工作树）

| 文件 | old sha256 / bytes | new sha256 / bytes |
| --- | --- | --- |
| `docs/development-evidence/render-bvh/perworld-l-v2-validate-window-g-l2/body-env-contract-errata.md` | （新增） | `cf0e7b904330d9128c7a198d83f213561c5c0027e91628ced134003bf45c7c06` / 6,009 |
| `docs/development-evidence/render-bvh/perworld-l-v2-validate-window-g-l2/README.md` | `194829f930392b87f51744ce92033e3a4e46c5747b53bc02d553e6ecdfcd7e6d` / 17,134 | `df7386a69502be4a4a58fb4b1a23221c2b5872091671c5a3c2c128b7e2a8317c` / 17,642 |
| `docs/development-evidence/render-bvh/perworld-l-v2-validate-window-g-l2/manifest.txt` | `66cee2999c09155038bc629c01cefd1d5b92e6406c29a215012672d9a78c01d4` / 7,499 | `71d870545b89b144afa6f97715f0ad476d8bba19904db6e2e1f0a1a5a54c63e9` / 7,599 |
| `docs/development-evidence/render-bvh/perworld-l-v2-ptx-order-pins/README.md` | `bb6f0b15a6727758117a11e39d682b3a6f98b26fe32c408bc5dcf1952e8e12b4` / 6,084 | `9b20398a8d9e29786a29a9f4983b7f792a24ac87511e2fed7c966e4895b9df49` / 7,142 |
| `docs/development-evidence/render-bvh/perworld-l-v2-ptx-order-pins/manifest.txt` | `c77a36a52767c95385ede9ec0f8b2ca40f1a0263df5323f61644b474d5e60160` / 1,378 | `845b110bbc2fe6fd6f2fbcc7ef6e5c44973e049b4017ccf80222aa92ad0aadcd` / 1,378 |
| `docs/development-evidence/render-bvh/perworld-l-v2-entity-diagnostic-dump/README.md` | `0981a7570420307f355f02a173fd34271521cf211c9c87d53224f10a9a37c31d` / 8,178 | `b87eb96483b21a148ef967197d4a968055b1c5b596520a4fea1eb12f157272d7` / 8,992 |
| `docs/development-evidence/render-bvh/perworld-l-v2-entity-diagnostic-dump/manifest.txt` | `39456e7642a2d30430f6af10c2a91a8ebabe44d738753bf5ada3b38ce2bbfc6b` / 1,413 | `f62671abe247f4cfb8f0cae0d680685fc861f5dda4f08a44d8e88d1975886e1e` / 1,413 |

本批自身证据目录（`perworld-l-v2-gl2-post-errata/`）：README.md、receipt.json、manifest.txt、`artifacts/` 5 件（outline-env-contract、guards-json-body-env-audit、ptx-order-pins-src-diff-eol、changed-files-sha256、manifests-selfcheck）。

## 2. manifest 重算与自核

| 目录 | 条目 | bad | missing | extra |
| --- | --- | --- | --- | --- |
| `perworld-l-v2-validate-window-g-l2` | 66 | 0 | [] | [] |
| `perworld-l-v2-ptx-order-pins` | 12 | 0 | [] | [] |
| `perworld-l-v2-entity-diagnostic-dump` | 12 | 0 | [] | [] |

## 3. 关键事实（本批只记文档，不改行为）

- body 环境契约的代码级依据：`tests/support/render_bvh/outline.rs` 模块文档 1-16 行、`enabled()` **51-68** 行、`diff_depth_with_outline()` **363-391** 行；导出位置 `scripts/validate/render-bvh/run-window.ps1:291-292`（outline 成对）与 `:307-308`（PATH）；pre/post 列表校验：pre-body `:280-298`（`Verify-OutlineLists 'pre-body'` 在 283 行）、post-window **`:357-369`**（入口 `Verify-OutlineLists 'post-window'` 在 **358** 行）。摘录见 `artifacts/outline-env-contract.txt`。
- 两者全缺=静默回落 raw 位级比较；只设一个=panic 拒绝；两者都在=列表外必须位级一致、列表内 `|ULP|<=128`、违规 fail-closed panic。
- guards JSON（`scripts/validate/render-bvh/guards-perworld-l2-light.json`，sha `fad2ed37…` / 43,729 B）顶层键无 `body_env`；body env 只在窗口证据中。登记 backlog，本批不改冻结器/guards。
- `perworld-l-v2-ptx-order-pins/logs/src.diff`（**S1 更正，见 §5**）：6,772 B、sha `12be6af5…`，worktree 与 HEAD blob **逐字节一致**（无 EOL 转换）；但该文件**不是 UTF-8/LF**——实测为 **UTF-16LE + BOM（`FF FE`）**：UTF-8 解码在首字节即失败，按 UTF-16LE 解码为 61 行、**60 个 CRLF**；原始字节扫描 `b"\r\n"=0`、`b"\n"=62` 是把 UTF-16 的 `0D 00 0A 00` / `0A 00` 误当 ASCII 换行的结果（本批原记录据此误判「已 LF」）。按 Chief 裁定**不改历史字节**（证据保全优先），仅更正记录；`docs/development-evidence/** -text -diff` 已覆盖该路径（`git ls-files --eol` = `i/-text w/-text attr/-text`），无需新增属性；不批量转换（UTF-16 BOM 文件：`docs/development-evidence` 内 **169**、全仓 **217** → backlog）。实测命令/输出见 `artifacts/ptx-order-pins-src-diff-eol.txt`。
- 修复卡-1 的 P2 已被卡-2 R1 撤销（诊断轮 #4：A 仍 FAIL、world0 cam1 13268 回退、world1 cam1 三像素不变；证据路径：`target/render-bvh/perworld-l-v2-gl2/diag4/pixel-dump.json` 44,977 B / sha `48597d0e5c42b84f6c5646e9c100fe347263b0e760289221daf876bf283de071`、`diag4/run-summary.txt` 3,931 B / sha `df653b5b440ed5386f8251418cfeb900f18b11240bc5173fa19b1b33dbb60d00`；dump 留 `target/` 不入库）；本批只在卡-1 README 加 errata 指针，不改历史记录。

## 4. 边界与非声明

- 只改本批列出的 7 个文件 + 新增本批证据目录；`window/**` 运行记录、guards、冻结件、源码、测试、fixtures、scripts、已签 spec 与 S-2/S-3 材料正文全部零改动。
- 不宣称 G-L2 通过（判定仍为未通过 / 判据 A FAIL）；不宣称修复生效；不改判据/参考/容差/相机/recipe；不重跑、不挑选结果。
- backlog 待办：dump 代码字段改名（`diag.rs` 与 receipt 字样）与 guards `body_env` schema 变更均登记 backlog，等 Chief 派工。

## 5. 复核修正（2026-11-11，QA 复核 `b332dcf4` 后；本卡 `2026-11-11-render-perworld-l-v2-gl2-post-errata-fix`）

- **S1（严重）**：本批原记录称 `perworld-l-v2-ptx-order-pins/logs/src.diff`「已 LF / CRLF 0 / 无需归一化」，**实测不成立**：该文件为 **UTF-16LE + BOM（`FF FE`）**、6,772 B、sha `12be6af5…`、UTF-8=False（首字节 `0xFF` 即失败）、按 UTF-16LE 解码 3,386 字符 / 61 行 / **60 个 CRLF**；原脚本按原始字节数 `b"\r\n"=0`、`b"\n"=62` 属误导性度量（UTF-16 的换行是 `0D 00 0A 00` / `0A 00`）。
  - 四处不实记录已更正：① 本 README 原 §3 该条（见上）；② `receipt.json` 的 `fix_card_1_errata.src_diff_lf_verification`；③ `artifacts/ptx-order-pins-src-diff-eol.txt`（重写为实测编码判定 + 原始输出）；④ 提交信息（不可改写，已在本节与 receipt 记录）。
  - **不改历史字节**（Chief 裁定：证据保全优先）：`git ls-files --eol` = `i/-text w/-text attr/-text`，`docs/development-evidence/** -text -diff` 已覆盖该路径；worktree 与 HEAD blob 逐字节一致（6,772 B / sha 相同），无 EOL 转换；**不新增/不改写 `.gitattributes`**，**不批量转换**历史文件。
- **低项 3 条**：① **manifest 行序**：本批重算 manifest 时改为按路径 ASCII 排序，行内容不变、仅行序变化（共同路径行位移实测：G-L2 12 行、修复卡-1 11 行、诊断轮 6 行），此前未如实记录，现补记；② **行号引用**：`outline.rs` 的 `enabled()` 实为 **51-68** 行、`diff_depth_with_outline()` 实为 **363-391** 行；`run-window.ps1` post-window 校验为 **357-369**（入口 358 行），原引用的 50/359/280-298 已在本 README、GL2 `body-env-contract-errata.md` 与 artifact 更正；③ **诊断轮 #4 证据路径**：补 `diag4/pixel-dump.json`（44,977 B / `48597d0e…`）与 `diag4/run-summary.txt`（3,931 B / `df653b5b…`），见上条与修复卡-1 README §5。
- **观察 2 条**：① **UTF-16 BOM 系统性**：登记 backlog（`docs/development-evidence` 内 **169** 个、全仓 **217** 个），本批不批量改；② **今后口径**：**证据 log 一律 UTF-8/LF 写出**——PowerShell 用 `[IO.File]::WriteAllText(path, text, [Text.UTF8Encoding]::new($false))` 或 `Out-File -Encoding utf8`，**禁止默认 `>` 重定向**（本次 S1 即由此产生：默认重定向写出 UTF-16LE+BOM/CRLF）。
- **dump-ext 残留 2 条**（QA 复验低项）：① `perworld-l-v2-diag-dump-ext/README.md` §2 与 receipt `json_location` 的「未启用 env 时不存在该字段」更正为「**键始终存在、未采集为 null**」（`attach_diagnostics` 契约）；② 提交信息 `8b37670f` 的「配套卡 +86/−8」更正为实测 **+84/−8**（numstat 2/2 + 3/3 + 1/1 + 2/2 + 76/0 = +84/−8）；历史提交说明不改写，记录于本批 receipt 与卡 addenda。
- 本批四目录 manifest 全部重算并自核（bad=0 / missing=[] / extra=[]）：本批 **8**、G-L2 **66**、修复卡-1 **12**、诊断轮 **12**；行序按 ASCII 路径排序（见上）。本批只改证据文档与 manifest，`src/`、`tests/`、`fixtures/`、`scripts/`、`.cu`、历史证据字节（含 `logs/src.diff`）、guards/冻结件零改动。

## 6. 过程事件（锁纪律，2026-11-11）

- **事件**：2026-11-11 06:47 取锁执行本 errata 批；主提交 `dde275c1` 后**先释放了锁**，随后为更正提交说明在**无锁状态**执行 `normify fix`/`gate`/`done` 并补提交 `26d3cada`（涉及卡状态 JSON、`.normify/runs/`、`docs/**` 文档与提交）。
- **当时状态**：无其它成员持有或申请该锁；无并发写冲突；工作树在补提交后干净；未造成数据损坏或他人工作丢失。
- **违规性质**：按既定纪律「任何仓写（含 `normify fix`/`gate`/`done` 与治理提交）必须先 acquire 锁」，属违规（卡状态 JSON 也算）。
- **补救/固化**：规则固化为硬约束——任何仓写前先 `repo-write-lock.ps1 -Action status` 必须 FREE 并 `acquire`；无锁一律不写。
- **互引**：本事件同时记入本 card `2026-11-11-render-perworld-l-v2-gl2-post-errata-fix` 的 addenda（`normify fix` 追加）；本批 receipt `process_events` 节与卡 addenda 互相引用。
