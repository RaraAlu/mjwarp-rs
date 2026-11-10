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

- body 环境契约的代码级依据：`tests/support/render_bvh/outline.rs` 模块文档 1-16 行、`enabled()` 50-68 行、`diff_depth_with_outline()` 359-391 行；导出位置 `scripts/validate/render-bvh/run-window.ps1:291-292`（outline 成对）与 `:307-308`（PATH）；pre/post 列表校验 `:280-298`。摘录见 `artifacts/outline-env-contract.txt`。
- 两者全缺=静默回落 raw 位级比较；只设一个=panic 拒绝；两者都在=列表外必须位级一致、列表内 `|ULP|<=128`、违规 fail-closed panic。
- guards JSON（`scripts/validate/render-bvh/guards-perworld-l2-light.json`，sha `fad2ed37…` / 43,729 B）顶层键无 `body_env`；body env 只在窗口证据中。登记 backlog，本批不改冻结器/guards。
- `perworld-l-v2-ptx-order-pins/logs/src.diff`：6,772 B、CRLF 0、LF 62、sha `12be6af5…`，与 HEAD blob 逐字节一致 ⇒ 已是 LF，无需归一化字节改动；该目录 manifest 变化仅由 README errata 追加引起。
- 修复卡-1 的 P2 已被卡-2 R1 撤销（诊断轮 #4：A 仍 FAIL、world0 cam1 13268 回退、world1 cam1 三像素不变）；本批只在卡-1 README 加 errata 指针，不改历史记录。

## 4. 边界与非声明

- 只改本批列出的 7 个文件 + 新增本批证据目录；`window/**` 运行记录、guards、冻结件、源码、测试、fixtures、scripts、已签 spec 与 S-2/S-3 材料正文全部零改动。
- 不宣称 G-L2 通过（判定仍为未通过 / 判据 A FAIL）；不宣称修复生效；不改判据/参考/容差/相机/recipe；不重跑、不挑选结果。
- backlog 待办：dump 代码字段改名（`diag.rs` 与 receipt 字样）与 guards `body_env` schema 变更均登记 backlog，等 Chief 派工。
