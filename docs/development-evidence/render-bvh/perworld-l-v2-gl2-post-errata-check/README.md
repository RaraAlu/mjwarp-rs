# QA 复核：G-L2 后置 errata 批（`dde275c1` + `26d3cada`）

- 任务：`2026-11-11-render-perworld-l-v2-gl2-post-errata-check`（owner QA；零 GPU、只读复核，未改任何源码/判据/参考/容差）。
- 对象：errata 批主提交 **`dde275c1`**（20 文件；`2026-11-11-render-perworld-l-v2-gl2-post-errata`，verified）+ 口径更正 **`26d3cada`**（1 文件，card addenda 16→15）。复核基线 HEAD `00bfaef8`（其上 `00bfaef8` 为我此前的 dump-ext 复核，不影响本批证据）。
- **结论：不通过（1 严重 + 2 低 + 2 观察）。** 四项 errata 内容主体成立、边界零越界、manifest 独立重算全对、`26d3cada` 确实只改口径；但**项 4 的「`logs/src.diff` 已 LF / CRLF 0」结论不实**——该文件实为 **UTF-16LE + BOM**（解码后 60×CRLF），`CRLF 0 / 已是 LF / UTF-8` 三项均错，需一张小修卡更正记录面并决定该历史证据文件的处置。此项不影响 G-L2 判定（仍为未通过 / 判据 A FAIL），也不影响任何数值证据。

## 1. ① 边界：只动文档/证据，代码与冻结面零 diff ✅

| 检查面 | 结果 |
| --- | --- |
| `dde275c1` 文件（20） | 7 个既有证据文件 + 8 个新证据文件 + 4 个 gate run + 1 个卡记录；无 `docs/`、`.normify/` 之外的路径 |
| `26d3cada` 文件（1） | 仅 `.normify/changes/2026-11-11-render-perworld-l-v2-gl2-post-errata.json` |
| `src/`、`tests/`、`fixtures/`、`scripts/` | `git diff bb29e351..26d3cada` 全空 |
| guards JSON | 零 diff；实测仍 `fad2ed37…` / 43,729 B；12 个冻结 exe stash 未动 |
| G-L2 `window/**` 运行记录 | 零 diff（字节不动） |
| 已签 S-1 spec | `perworld-l-v2-light-criterion/spec.md` 仍 39,379 B / `bb064fe0…` |
| S-2 / S-3 签名材料目录 | `perworld-l-v2-capture-window-s2/materials/`、`perworld-l-v2-validate-window-s3/materials/` 零 diff |

## 2. ② 四项 errata 落地情况

| # | 项 | 结果 |
| --- | --- | --- |
| 1 | body 环境契约 + 两次 INVALID_ROUND 归因 | ✅ 新增 `perworld-l-v2-validate-window-g-l2/body-env-contract-errata.md`（6,009 B / `cf0e7b90…`）；该目录 README 仅追加 §11 指针（+5 行，判定句未动）；契约描述与源码/窗口证据逐条一致（见 §4） |
| 2 | guards `body_env` 缺字段 | ✅ 审计属实：guards 顶层 19 键无 `body_env`；只写文档 + `artifacts/guards-json-body-env-audit.txt`，未改冻结器/guards/冻结件 |
| 3 | dump 字段表述 `geom_type` → `objtype` | ✅ 更正正确：`seg[1]`=MuJoCo object type（`mjOBJ_GEOM`=5）；模型 fixtures 实测 `geom_type=[0,6,2,7,3,5,4]` → 命中 `geom_id=3` 是 `geom_type=7`（MESH）、`geom_matid[3]=0`（mat 0 = mat-red，`mat_texid` 行 slot1=0）；README 仅改表述与追加 §5，表格数字/结论未动；代码/已生成 receipt 只记 backlog |
| 4 | 修复卡-1 证据 errata（P2 撤销交叉引用 + `logs/src.diff` LF 复核） | ⚠️ **P2 交叉引用 ✅、+17/−3 口径 ✅**（`../perworld-l-v2-ptx-order-pins-2/` 目录存在，R1 撤销 P2/新增 P5/P6 与卡-2 记录一致）；**但 `src.diff` 的 LF 结论不实 → 见 §3 严重 S1** |

## 3. 严重 S1：`logs/src.diff` 实为 UTF-16LE + BOM，errata 的「已 LF / CRLF 0」不实

- 文件：`docs/development-evidence/render-bvh/perworld-l-v2-ptx-order-pins/logs/src.diff`，6,772 B、sha256 `12be6af5…`（与 HEAD blob 逐字节一致 ✅ —— bytes/sha/identical 三项 errata 说对了）。
- 实测：首 4 字节 `FF FE 64 00` ⇒ **UTF-16LE + BOM**；`valid UTF-8 = False`；按 UTF-16 解码为 3,385 字符、**60 个 CRLF**、0 裸 LF/CR、0 TAB。
- errata 在四处claim「CRLF 0 / LF 62 / 已是 LF / 无需归一化」：README §3 L41、`receipt.json` `fix_card_1_errata.src_diff_lf_verification`、`artifacts/ptx-order-pins-src-diff-eol.txt`（`worktree_crlf=0`）、`dde275c1` 提交信息；实测三项全错（UTF-8=False、行尾=CRLF、BOM 存在）。「LF 62」是原始字节里 `0x0A` 的个数（UTF-16 下部分来自 CJK 代码单元低字节，不是行数）。
- 该文件是 pins-1 证据（非本批新建），本批项 4 的职责正是关闭我 pins-1 复核的 EOL 观察；结果以错误测量关闭。另：本批「证据编码自检 15 文件」的范围是 7 改 + 8 新，**不含** src.diff，故该自检本身没错，但项 4 的独立审计结论错。
- 建议：① 更正 errata README §3、receipt 字段、artifact 测量与说明（写明「UTF-16LE+BOM 历史文件，非 UTF-8/LF」）；② 由 Chief 裁该文件处置：(a) 转 UTF-8/LF 会改历史证据字节 → 需同步 manifest 行并留痕；或 (b) 登记为「`-text` 二进制证据，不做 EOL 声明」，并补进证据卫生口径。我不擅自改历史证据。

## 4. ③ body env 契约描述 vs 实际证据（全部一致 ✅，附实测锚点）

- `tests/support/render_bvh/outline.rs`：`enabled()` 在 **:51**（errata 写 50-68，含 :50 doc 注释，可接受）；语义与描述逐条一致——两者全缺 → `false`（调用方回落 raw 位级比较）、只设一个 → `panic!`、两者都在且 `REQUIRE=1` → `true`；`diff_depth_with_outline()` 在 **:363**（errata 写 359-391，含 doc 注释）：`!enabled()` → raw，启用后 `violations!=0` → `assert` fail-closed panic。
- `scripts/validate/render-bvh/run-window.ps1`：outline 成对导出 **:291-292** ✅；PATH 前置 **:307-308** ✅（`g01-fast.ps1:40-41` 同口径 ✅）；pre-body 校验 **:283**、post-window 校验 **:358**（低 L2：errata 把 pre/post 一并引为「280-298」，只覆盖 pre）。
- guards `outline_lists`：recipe `cfd1038a…`/10,099 B/rows 183/first_pixel 28,969/exact_max_ulp 73；aa2 `14893fc6…`/9,509 B/rows 172/first_pixel 29,448/exact_max_ulp 91；`outline-lists.json` `c2dd876d…`/2,808 B、`bound_ulp=128`、`criterion_version=render-sdf-criterion-v2.2` —— 与 errata/窗口证据逐项一致。
- 两次 INVALID_ROUND：`window/window-incidents.txt` 实测 #1 = 04:16:39 run-0 直接启冻结 exe 无 pinned CUDA bin → `NvrtcUnavailable` exit 101 @0.209 s（0 轮消耗）✅；#2 = 04:20:16/51 run-1 漏两个 outline env → `render_e2e` cam1 183/73/28,969 假红（0 轮消耗）✅ —— errata 表逐项一致。
- canary：`window/run/exe-logs/render_e2e.stdout.log` `outline[recipe] cam1: list_rows=183 outside_mismatch=0 in_list_differing=183 in_list_max_ulp=73 violations=0` ✅（errata 说 canary violations=0 正确）。
- 12 exe 环境需求（`canonical-env-checklist.txt` §5）：6 个 scene-a 回归 = PATH+outline、其余 6 个 = PATH only ✅；`ENV_SELFCHECK_PASS failures=0` ✅；`window/body-env.txt` 实记 PATH+两 env ✅。

## 5. ④ manifest 独立重算（行级） ✅ + 低 L1

| 目录 | 条目 | bad | missing | extra | ASCII 有序 |
| --- | --- | --- | --- | --- | --- |
| `perworld-l-v2-validate-window-g-l2` | 66 | 0 | [] | [] | True |
| `perworld-l-v2-ptx-order-pins` | 12 | 0 | [] | [] | True |
| `perworld-l-v2-entity-diagnostic-dump` | 12 | 0 | [] | [] | True |
| `perworld-l-v2-gl2-post-errata` | 7 | 0 | [] | [] | True |

- 逐条重算 sha256/bytes 全对、无缺项/多项；7 个改动文件的 old/new sha+bytes 与 `bb29e351`/`dde275c1` blob 逐项一致（`changed-files-sha256.txt` 可信）。
- **低 L1**：三个旧 manifest 在本批被**重排为 ASCII 有序**（如 G-L2 的 `window/baseline.txt`、`monitor.csv`、`run1-rootcause…`，pins-1 的 `README.md`，诊断轮的 `preflight-attempt1.txt`/`test-attempt1.*` 换了行位），而行内容未变。提交信息称「其余行逐字节不变 / 行级差异仅 README 一行 + 新增一行」——**内容层成立、行序层不成立**；新顺序是规范 ASCII 排序（改进），建议把措辞改成「内容不变、按 ASCII 重排」。

## 6. ⑤ `26d3cada`：只改口径、证据零字节变化 ✅

- 该提交只改 1 个文件（errata 卡 JSON）；`docs/` 零 diff（`dde275c1..26d3cada` 唯一变更就是该卡 JSON）。
- 改动内容 = `updated_at` + 新增 `addenda`：「提交说明中『证据编码自检 16 个涉及文件』更正为 15（7 改 + 8 新增）；证据内容零改动」。我核数：本批 docs 改动 = 7 修改 + 8 新增 = **15** ✅（16 确为笔误），addenda 措辞与实际相符。

## 7. 低项与观察

- **低 L2**：行号引用不精确——`enabled()` 实际 :51（写成 50-68）、`diff_depth_with_outline()` 实际 :363（写成 359-391）、pre/post 校验实际 :283/:358（写成「280-298」，post 不在该范围）。均含 doc 注释或沿用了窗口 README §6.2 的旧措辞，读数不受影响。
- **低 L3**：项 4 的「诊断轮 #4 实测（A 仍 FAIL、13268 回退、world1 三 px 不变）」未给可核证据路径；事实与 Chief 2026-11-11 05:49 通告一致（我记录在 pins-2 复核卡 note 与 `ptx-order-pins-2-check/README.md`），建议注明来源或链接。
- **观察 O1（系统性）**：`docs/development-evidence/` 下共 **169 个 UTF-16 BOM 文件**（历史遗留，多为 PS 5.1 重定向日志）；本批只涉及 1 个。建议单独立项做一次证据编码普查，不在本批解决。
- **观察 O2（过程）**：Backend 披露 `dde275c1`/`26d3cada` 时段**无锁写仓**；仓内无锁审计日志可回溯（我无法自证/证伪），按披露记为过程观察。补救：Chief 说已在下一张治理卡 addenda 安排——我在当前库内**未找到**该 addenda（11-11 无相关卡记录），待下一张卡落地后核。
- 补充更正：我此前 pins-1 复核只记「`logs/src.diff` 含 64 个 CR（CRLF）」，本次更正为 **UTF-16LE + BOM**（64 个 `0x0D` 字节含 CJK 低字节，不是 64 个 CR）；当时判读不完整，特此更正记录。

## 8. 复现方法（零 GPU）

`artifacts/` 五个文件均由脚本现场实测生成：`scope-zero-diff.txt`（两提交文件清单 + 8 条零 diff 检查 + spec/guards/stash）、`body-env-crosscheck.txt`（源码行锚点 + guards 元数据 + window-incidents/canary/checklist 摘录）、`manifest-check.txt`（四目录重算）、`src-diff-encoding.txt`（BOM/解码/CRLF 实测）、`26d3cada-check.txt`（提交面 + addenda + 15 文件核数）。

## 9. 非声明

- 本复核**不改变 G-L2 判定**（仍为未通过 / 判据 A FAIL），不宣称任何修复生效，不改判据/阈值/参考/容差，未重跑/重冻/采集。
- S1 的修法是**记录与证据卫生更正**，不属于放宽判据；src.diff 字节如何处置属 Chief 裁决事项（改历史证据字节 vs 登记为二进制证据）。
