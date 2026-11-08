# S-2 材料 floor 合并语义 fix 最小复核（QA 只读）

任务：`2026-11-08-render-perworld-l-v2-capture-window-s2-material-fix-check`（owner QA）。日期：2026-11-08。
复核对象：S-2 材料 fix 提交 **`ade516b5`**（HEAD）；前一版材料 `1e27ff59`。
边界：只读 + 只写本卡证据；零 GPU。

## 结论

**PASS（0 阻断；2 条观察：材料任务卡 acceptance 残留 min 文本、我上一版复核记录为历史版本）。→ 可发用户 S-2 双签卡。**

## 1. floor 合并语义更正（min → max）

- 现行口径：`floor_effective = max(signed floor, onspot floor)`；依据写明「验收条件是 `free ≥ floor_effective`，**floor 越高越严**，取 min 等于放宽；例：signed 10,230 / 现场 10,300 → 必须按 10,300 卡」。
- 区分采样规则：`baseline_free = min(三点 free)` 标注为**采样取严（取最差样本）**，与 signed/现场合并规则不是同一件事 ✅。
- 覆盖逐处核对：README §3（L45）/§5（L68）、`baseline-sop.txt`（L11/L20）、`signature-summary.txt`（L22-23/L51）、`baseline-suggestion.txt`（L5-6）、`command.txt`（L54-55）、`qa-prereview-checklist.txt`（L14）、`evidence-checklist.txt`（L11）、`receipt.json`（L109/L136）——**全部为 max 口径，全文无 `floor_effective = min` 残留** ✅。
- 全仓 `git grep -a floor_effective` 扫描：S-2 材料目录内仅 max 口径 + 采样 min 说明（正确）；其余命中为材料任务卡 acceptance 与我的上一版复核记录（见 O1/O2）。

## 2. 我的 O1–O3 收口

- **O1（S-2b 可粘贴行）**：现为 `S-2b (suggested, re-measured at the window with stricter = higher wins): baseline_free 13,302 MiB; signed floor 10,230 MiB; floor_effective = max(signed floor, on-the-spot floor)` ✅。
- **O2（P7 锚点）**：已补 QA 收口独立复核卡 `2026-11-08-…-pilot-g-l0-window-check`（证据 `c5a56639`）✅。
- **O3（证据目录名）**：README / signature-summary / receipt 统一为 `…perworld-l-v2-capture-window-g-l1/` ✅。

## 3. 新 sha/bytes 与 manifest

| 文件 | 旧（1e27ff59） | 新（ade516b5） |
| --- | --- | --- |
| `README.md` | `3215e4ee…`/11,612 B | **`d46e023068e162fa51a98edc41859a0d7fbe8a1c76fd2f3589157086724ee91a`/13,984 B** |
| `receipt.json` | `9e116955…`/8,349 B | **`3692c374ab158f8f17fdf2d43ac2027087e822f183eab29f27fea4b691220f56`/8,864 B** |
| `manifest.txt` | — | `a2fd8c71…`/1,341 B（**12/12** 独立重算通过） |

三文件 UTF-8/LF/无 BOM/裸 CR·TAB 0 ✅。

## 4. 边界与门禁

- `ade516b5` 仅动 S-2 材料证据（README/receipt/manifest + 7 个 materials 文件）+ 材料卡 + runs；`src/`、`fixtures/`、`scripts/`、`tests/` **零 diff** ✅；材料卡 addenda 已记录 min→max 更正理由 ✅。
- 四门禁独立复跑：fmt/clippy/clippy-cuda 通过，**unit 1384 过 / 0 败** ✅。

## 5. 观察项

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低 | 材料**任务卡** acceptance 文本（`.normify/changes/2026-11-08-…-s2-material.json`）仍写 `floor_effective = min(...)`（fix 只加了 addenda，未改 acceptance 原文） | 该卡为历史任务定义、非可签材料，不阻断；可在下一次 `normify fix` 时把 acceptance 改为 max 或补一句「以 ade516b5 修订为准」 |
| O2 | 信息 | 我上一版复核卡（`…-s2-material-check`，证据 `7c76ad6b`）记录的是 fix 前版本，其中 `floor_effective=min` 与今天的 max 口径相反 | 历史记录，保留原文；本卡即 fix 后的复核结论；后续引用以本卡为准 |

## 非宣称

- 本复核只判 floor 合并语义更正与 O1–O3 收口、证据卫生；不判 G-L1 结果，不宣称采集有效/灯验收/上游等价/渲染数值结论。
- 不构成 G-L1 启动；S-2 需用户双签、G-L1 需 Chief token。
