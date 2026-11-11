# QA 复核：NP5-00 全量清单与判据映射冻结（`875ae41d`，卡 F）

- 任务：`2026-11-11-np5-00-gate-ledger-freeze-r2-check`（owner QA）。复核对象：提交 **`875ae41d`** + 卡 `2026-11-11-np5-00-gate-ledger-freeze-r2`（被 abandon 的前身 `2026-11-11-np5-00-gate-ledger-freeze` 已 `769e6933` 弃用）。零 GPU、只读复核。
- 基线：`dec4e7d9`（r2 卡登记/start）。

## 结论

**复核通过（0 阻断，3 低 + 1 观察）。** 账本与 target 定稿**逐字节相同**（`6d6ff7d9…`/116,688 B）、**190 条**分项 30/3/23/100/2/32 与 totals 一致、每条有 owner、30+3 表与输入 sha 全部对得上、checker 我独立重跑 **19 PASS / RESULT: PASS** 且日志与声明逐字节一致；写域仅 `docs/development-evidence/np-gate/**` + `scripts/np-gate/**` + `.normify`；**没有任何未加否定的「NP01/NP02 通过 / 全产品精度门禁通过」表述**。

## 一、账本内容与判据映射（✅）

- 落仓 `np-gate-ledger.json`：**116,688 B / `6d6ff7d98f82e6429038d6f4c731afcc49cf693951a17e520a5a6fde6568c8ad`**，与 target 定稿（我实测同文件同 sha）**逐字节一致**（未重排）。
- 条目：**190** = `np01.paths` 30 + `np01.host_sites` 3 + `np01.unregistered_product_units` 23 + `np02.kernels` 100 + `np02.missing_kernel_rows` 2 + `np02.gaps` 32；与 `totals.ledger_entries_total=190` 一致。
- 状态/owner：`np01.paths` 30 全 `partial`、`host_sites` 3 全 `partial`、`unregistered` 23 全 `absent（注解）`；`gaps` 32 的 `ledger_action` 直方图 = archive 9 / covered-by-base 2 / reclassify-or-blocker 7 / partial 3 / n/a-archived 3 / undetermined 2 / partial-missing 2 / deferred-linux 4（与 README 一致）；**100 个 np02 kernels 与全部 other 条目均有 `owner`**（我逐条扫，缺失 0）。
- 快照时点：`generated_at=2026-11-11T12:27:08+0800`、`inputs.tree.head=f4628bd7…`（附 subject）、6 组输入各自记录 sha/计数/时间（np02 两输入注明 2026-11-04 生成）；无静默混时点（见低 L2/L3）。
- 输入可复核：我实测 `np01 inventory 5259e4b4…`、`np02 inventory c7d84fb2…`、`np02 gap b49cdac2…`、`cuda-surface 8cb8361b…` **4/4 与账本一致**；`field-tables` 30 个、`host-tables` 3 个文件实存。
- **`scene_bvh_ray` 漂移我独立复核**：账本 `recorded 8dad4d61…` vs `current_sha256_lf caac8c86…`（`changed_by d35cab68`）；我按 HEAD blob 实测 `src/render/bvh/cuda/scene.cu` = **`caac8c86a4e4f1e7942cf5c920e2406755dc2e2cade73505f45e28cfff46ab93`**（17,834 B）—— 与账本 current 值逐位一致，漂移归因成立。
- `boundaries` 明写「本账本是清单/状态登记，**不是 NP01/NP02 通过结论**；NP01 E 指标未定稿、host/device 逐表达式位级对照未补；NP02 7 子系统需 recheck」；`scope` 末尾「**不宣称任何门禁通过**」。

## 二、机读校验（✅ 独立重跑）

- `python -B scripts/np-gate/check_np5_ledger.py`（我用本机 DSH python 直接跑脚本）⇒ **19 PASS / RESULT: PASS**；覆盖 schema、np01/np02 与 inventory 一致、30+3 表 sha、23 未登记单元存在性、totals 算术语、owner/action 覆盖、9 archive+7 reclassify、7 子系统 recheck、无未加否定门禁声明、`scene.cu` 漂移一致。
- 证据日志 `logs/np-gate-ledger-check.log` = **862 B / `beae8f16…`**，与我重跑输出语义一致（原文 sha 与声明一致）。

## 三、工具落仓 / 边界 / 证据 / 门禁（✅）

- `scripts/np-gate/check_np5_ledger.py` **4,902 B/`e7174bce…`**、`gen_np5_ledger.py` **17,892 B/`bc9e9e0e…`**；与 target 源（4,817/`a7dbde52…`、17,807/`7fe344d1…`）**逐行 diff 仅各 +1 行 guards 头注**（`NOT in the 93 guards inputs; adding it to guards requires a user re-sign (NP5-00)`），**无逻辑改动**（我逐行 Compare-Object 核过）。
- 写域：`git diff dec4e7d9..875ae41d` 仅 `docs/development-evidence/np-gate/**`（7 文件）+ `scripts/np-gate/**`（2 新文件）+ `.normify`（5）；**无** `src/`/`tests/`/`fixtures/`；`scripts/` 下**其它文件零改动**（freeze script/guards JSON/既有采集脚本未触碰，guards `94478b3d…`/43,729 B 我复核未变）。
- manifest **8/8** 独立重算全对（含两条 repo-relative `scripts/np-gate/*.py` 条目、不自列）；docs 全 UTF-8/LF、无 BOM/CR/NUL；两脚本 `git ls-files --eol` = `i/lf w/lf`。
- 门禁 `.normify/runs/…freeze-r2/`：fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**；旧卡 abandoned、r2 卡 verified；receipt 记录 `supersedes_abandoned_card` 与材料版本（规格 10,610 B、旧稿 12,364 B 作废）——README 对规格 §3 残留旧 sha/186 条的观察已如实注明（观察项、不阻断）。

## 四、低项 / 观察（不阻断）

- **低 L1（机读一致性）**：`np01.unregistered_product_units` 的 23 条 `np01_status` 值 = **`absent（未入 30 条清单；2026-10-23 后新增/实现）`**（状态+理由拼接），与 `status_vocabulary.np01 = [verified, partial, missing, absent]` 的裸值**不能精确匹配**——我的机读扫描即报 23 个 unknown；checker 只覆盖 `gaps` 的状态映射，未覆盖该字段。因「与 target 定稿逐字节相同」要求，本卡不能就地改；建议下一版账本把状态与理由拆成 `np01_status`/`reason` 两字段（或把带注解形态写进词表），checker 增补该字段断言。
- **低 L2（记录面）**：README §2 写 `inputs.tree.head = f4628bd7…（生成时 HEAD，即本卡登记提交）`——对 **r2 卡**不准确：`f4628bd7` 是被 abandon 的第一版卡 F 的登记提交（生成时 HEAD），r2 卡登记提交是 `dec4e7d9`。建议改为「生成时 HEAD（第一版卡登记提交；r2 重构前）」。
- **低 L3（记录面）**：账本 `task` 字段仍为 **`np5-00（草稿；由 target/ 生成，待 Backend 落 docs/development-evidence/np-gate/）`**，落仓后已非「草稿/待落」；因逐字节复制未改，建议 README 在 §1/§2 注明该字段为生成时草稿态残留。
- **观察 O1（措辞易误读）**：账本 `scope` 首句「NP01（类型映射 100% / 精度偏差 0）与 NP02（证据覆盖 100% / 缺失 0）全量清单…」描述的是**清单完备性**（同句末尾有「不宣称任何门禁通过」），但「100%/0」易被读成验收结论；建议下一版措辞改为「清单覆盖完整（30/3/23/100/2/32）、无缺失条目」之类。

## 五、口径

不宣称 NP01/NP02 通过或全产品精度门禁通过；不改判据/参考/容差；不重采证据、不重跑窗口；零 GPU；本复核只核清单冻结的完整性/一致性/边界。
