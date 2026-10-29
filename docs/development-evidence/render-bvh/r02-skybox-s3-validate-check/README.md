# R02-S3 天空盒验证窗 attempt=2 · QA 独立复核

任务卡：`2026-10-29-render-bvh-r02-skybox-s3-validate-check`（priority 57，Owner QA）
被核对象：`2026-10-29-render-bvh-r02-skybox-s3-validate`（**verified**）+ `2026-10-29-render-bvh-r02-skybox-s3-artifacts-commit`（**verified**）
被核提交：窗口证据 `d7f705d0`/`fa5cfcfd`（+`8fecc3e4` 路径修正）→ S3a 入库 `f0213748`/`99b88d1f`/`65022c3a` → 收口 `8a967468`
复核基线 HEAD：`8a967468`。**只读**：未开 GPU、未重跑窗口、未跑 `--ignored`、未改代码/证据。

---

## 1. 结论

**verdict：需修（1 项证据措辞，按卡面第七条「禁词 0」判）；窗口事实 / 位级判据 / 冻结身份 / attempt1 与两条修复链全部独立复核通过。**

| 复核面 | 我的结论 |
| --- | --- |
| 例外合法性（主证据 = attempt2 例外监 CSV） | **成立且比 precheck 更硬**：60 样本 / 60 clean / longest 60；**12 个 util≥40 样本全部 clean=True**、streak 从 7 连续到 60；ge60=0、free_min 12 751、filtered/build 峰值 0；`-MaxUtil 60` 与 `-OpenUtilLimit 60` 逐字在案；`not_a_normal_threshold_pass=true`、`authoring_exception_used=false`；常规失败件（354 / longest 18 / 75 个 ≥40 / exit 20）原样保留；异常记录明确写「precheck 不作例外证据」 |
| 位级判据 | 两相机 RGB/depth/seg **全部 0**（cam0 76 800、cam1 43 200）；depth_zero 16 995 / 27 498；两跑 sha `92ac188d…`/`de732643…`/`065caf08…`；判据口径未改 |
| 窗口六项 | 9/9 exit 0；sha_guards **77/77**；post **9/9**；`zero_rust_compile=True`（deps **1839→1839**、max write **18:12:48** 早于窗 18:22:08）；samples 166、floor 9 786 未破、free_min 12 587、增量 271≤3 072、residual 0、`WINDOW_RUN_DONE` |
| 冻结身份 | `guards-r02-skybox.json` = **`2d55c0b3…`/28 931 B/attempt=2/77(9+68)**，aggregate **`ca1fcaa5…`**（我用证据同口径脚本复算 `matches=True`）；committed blob **==** 现盘；9 exe **live==guards==stash**（sha+bytes）；sha-guards 77/77 行对现盘全中；R05 `guards.json` `e2444a96…` 未动；attempt1 guards `b5a2d4d7…` 归档存在且标 superseded |
| attempt1 与修复链 | attempt1 INVALID_ROUND（4 失败/5 通过）**我从 target 原始日志交叉核实**（e2e `negative_controls_hold_with_product_poses` FAILED；skybox 参考 depth mismatch **19 074**）；两类根因描述与代码/提交一致；capture 脚本链 **`812a2f77`→`d17d2418`→`d6f9864c`** 我**逐步复现**（见 §5）；params28 唯一改 `use_fast_math` |
| 证据完整性 | manifest S3 **34 行**、S3a **2 行**，均 **committed blob 与现盘双向 100% 一致**、不自引用；全目录 LF/无 BOM；scope diff 只有证据 + 4 个参考文件 + `guards-r02-skybox.json` + `.normify` |
| **卡面第七条（禁词 0）** | **不满足**：S3 `README.md` 字面出现 4 个禁词（2 处在「不是…」否定句、1 处在「不写…」否定句）→ 见 §2 与 O1 |

---

## 2. 唯一必修项：README 字面禁词（O1）

卡面第七条要求「禁词 0（不出现那 4 个词）」。S3 `README.md` 实际命中：

1. 引言块：`> **不是常规阈值通过**：常规 util<40 在 12:00 时段失败…`（含「常规阈值通过」）
2. §9 第一条：`* **不是常规阈值通过**：…`（同上）
3. §9 末条：`* 不写「精度类词/等价类词/验收类词」…`（含另外 3 个）

三处都是**否定式/自我约束**表述（没有做任何结论性主张，诚实性没问题），但按卡面字面「不出现」即不合格。最小修法（不改变语义、不动任何数字）：

- 第 1/2 处 →「本窗**不是常规阈值轮**：常规 util<40 在 12:00 时段失败（354 样本 / longest_clean=18 / 75 个 ≥40 / exit 20）」
- 第 3 处 →「不写精度、等价或验收类结论；结论以逐相机 0 差异与两跑 sha 相同表述」

（我自己的证据里同样规避了字面：脚本内用码点构造词表。）

---

## 3. 例外合法性：我重算的数字（复核点 1）

`monitor/monitor-pre-window-attempt2.csv`（60 行）逐行重算：

| 项 | 值 |
| --- | --- |
| samples / clean / longest_clean | **60 / 60 / 60** |
| util 范围 | **32–41** |
| util≥40 样本 | **12**，且**全部 clean=True** |
| 其中 streak 轨迹 | seq6=40(str7) → seq18=41(str19) → seq21=40(22) → seq25=41(26) → seq27=40(28) → seq29/30=40(30/31) → seq37=40(38) → seq40=40(41) → seq44=40(45) → seq55=40(**56**) → 末尾 streak **60** |
| util≥60 样本 | **0** |
| free_min / filtered_max / build_max | **12 751** / 0 / 0 |
| 命令 | `…monitor-until-clean.ps1 … -Required 60 -MaxMinutes 30 -MaxUtil 60`（逐字在 exception-result） |

⇒ 生效门槛是 **60 而不是 40**（若按 40，这 12 个样本必须 unclean），这与证据「precheck 不作主证、以监 CSV 为准」的口径一致（R05 O1 修正已落实）。

- `attempt2/window-command.txt` 逐字含 `-GuardsFile scripts\validate\render-bvh\guards-r02-skybox.json -OpenUtilLimit 60` ✓。
- 常规失败件：`monitor-pre-window.csv` 我重算 **354 样本 / longest_clean=18 / util 2–43 / 75 个 ≥40 / ge60=0 / free_min 12 694**，`monitor-pre-window-result.txt` 含 `monitor_exit=20`、`decision=STOP_NORMAL_THRESHOLD_NOT_MET`、`window_commands_executed=0` ✓ 原样保留。
- `attempt2/facts.txt` 例外块 + `attempt2/exception-block.txt` + README §5 三处 `not_a_normal_threshold_pass=true` / `authoring_exception_used=false` / `signature_unchanged=true` 一致 ✓。

---

## 4. 判据与窗口事实（复核点 2、3、6）

`attempt2/logs/render_skybox_reference.log` 逐行解析：

| 相机 | pixels | RGB compared / mismatched / max_byte_diff / beyond_1_lsb | depth mismatch / max_ulp / beyond_ulp | seg mismatch | depth_zero |
| --- | --- | --- | --- | --- | --- |
| cam0 | 76 800 | 76 800 / **0 / 0 / 0** | **0 / 0 / 0** | **0** | 16 995 |
| cam1 | 43 200 | 43 200 / **0 / 0 / 0** | **0 / 0 / 0** | **0** | 27 498 |

两跑 sha：rgb `92ac188d1cab24258e777461f44087d747e711f70b4943d0c9f48a727a58b3cc`、depth `de732643aa3e9afb2a477b259dd1128139929c29ba7d8935d23096d767e42445`、seg `065caf08c1759c34be2b9d9e2b2f8d62e874f219d8d6bf93c8d9e2408b293821`（与 README/receipt 一致；`test result: ok. 1 passed`）。判据常量沿用 `tests/support/render_bvh/skybox.rs`（depth 位级优先 + `max_ulp<=4`/`beyond=0`），本次链未改。

窗口事实（`entity-results.csv` / `facts.txt` / `samples.csv` / `guard.log` / `guard_aggregate.json`）：

- 9 实体 exit 0（wall 1655/1312/7810/8724/4324/9222/3665/7874/2784 ms 与 facts 逐条一致）；1 new + 8 regression。
- `guard.log`：`PRECHECK_OK open=3204/12858/37/29;…37/29;…40/29` → `GUARDS_OK=77/77` → `WINDOW_RUN_DONE`。
- `samples.csv` 166 行：used peak 3 475(seq 6)、used_min 3 204、free_min **12 587**、util 17–40、floor 9 786 未破、增量 271 ≤3 072、streak≥80=0。
- `guard_aggregate.json` 与 facts：samples/peak/seq/min/free/util_max/floor/baseline/aggregate/sha_guards/post_recheck **11 项全等**（唯一不一致见 O2 的 suite 标签）。

---

## 5. 冻结身份与修复链（复核点 4、5）

### 5.1 冻结

- `scripts/validate/render-bvh/guards-r02-skybox.json`：我实测 **28 931 B / sha256 `2d55c0b3…` / suite `r02-skybox-validate` / attempt=2 / 77 = 9 exe + 68 inputs**；**committed blob == 现盘**（本地 `.git/info/attributes` 对该路径有 `-text`，与 R05 同口径，已在 S3a README 登记为已知限制）。
- aggregate：用证据同口径脚本（`r05-s3b-hfield-validate/materials/recompute-aggregate.ps1`，PowerShell culture 排序）复算 = **`ca1fcaa5…` = 记录值**，`matches_recorded=True`。⚠ 用 Python 序数/casefold 排序都会得到别的值（`7a9fedc1…`/`5ae530aa…`）——这不是缺陷，`aggregate-sort-note` 已声明口径。
- 9 exe：`guards.sha256 == guards.stash_sha256`、live 文件 sha+bytes == guards、stash 文件 sha+bytes == guards，**9/9 全对**；`sha-guards.txt` **77 行**键集与 guards 完全一致，逐行对**现盘**重算全中（9 exe + 68 输入）。
- attempt1 guards 归档 `target/render-bvh/r02-skybox-window/guards-r02-skybox-attempt1-superseded.json` 存在，sha 实测 **`b5a2d4d7…`/28 931 B**，README §4 标 superseded ✓。
- R05 `scripts/validate/render-bvh/guards.json` 实测 **`e2444a96…`/27 670 B** 未动 ✓。

### 5.2 capture 脚本 sha 链（我逐步复现，不只看文档）

| 步 | 文件 | bytes / LF | sha256 |
| --- | --- | --- | --- |
| S2 原交付 | git blob `3e3665f0:scripts/authoring/render-bvh-r02/capture_skybox.py` | 16 084 / 425 | **`812a2f77…`** |
| + `attempt1/capture-skybox-fix.diff`（窗口期确定性自检修复） | 我在**仓库外**目录把该 diff 应用到 812a2f77 并做 LF 归一 | **16 494 / 428** | **`d17d2418…`** ✓ |
| 再把 `"use_fast_math": True` 改成 `False`（+1 字节） | 字节级替换后与现值比较 | **16 495 / 428** | **`d6f9864c…`**，与现盘**逐字节相同** ✓ |

⇒ `812a2f77 → d17d2418 → d6f9864c` 全链可复现；`d17d2418` 是窗口期工作树状态（未单独提交），其身份由 diff + 该文件记录，`b3a47aca` 提交把两处修复一起入库。另核：`b3a47aca` 对脚本的改动 = `import shutil` + `use_fast_math True→False` + 自检逻辑（npz_bytes→同 writer 两跑文件 sha 比较），无模型/场景/参数/输出格式/命令改动 ✓。

### 5.3 attempt1（INVALID_ROUND）与失败证据

- `attempt1/entity-results.csv`：4 失败（`render_skybox_reference`/`render_e2e`/`render_kernel_flags`/`render_kernel_rgb`，exit 101）、5 通过 ✓；`invalid-round-note.txt` 记录两类根因 ✓。
- 我另从 `target/render-bvh/r02-skybox-window/attempt1/logs/` 的**原始失败日志**核对：
  - 失败 A：`render_e2e.log` = `test result: FAILED. 4 passed; 1 failed`，失败用例 `negative_controls_hold_with_product_poses`（= stale 负控）✓；
  - 失败 B：`render_skybox_reference.log` = depth 断言左 **19 074** / 右 0（cam0），与 note 的 `depth mismatch=19074 max_ulp=2` 一致 ✓。
- `stage-a-attempt1-failure.txt` 的旧/新脚本 sha、部分 npz sha（b49053a1/0fa55821/2153fa72）与逻辑说明齐全 ✓；旧参考归档 `stage-a-attempt1-reference-superseded/` 现盘 4 文件 sha = `e81e098d`/`b49053a1`/`0fa55821`/`2153fa72`，与 README §6 一致 ✓。

### 5.4 参考（本次生效）与 params28

- `fixtures/render-bvh/scene-a/reference-skybox/` **恰好 4 文件**：capture.json `47f704de…`、rgb `cf65a570…`、depth `28c1ea4b…`、seg `2153fa72…`，与 S3a/README 记录一致；`capture.json.params28.use_fast_math = False`、`params28_sha256 = e2bae199…`、`determinism.two_runs=true`、`sha_equal=true`（per-file sha 与 4 文件一致）。
- S2 证据 `materials/params28-audit.txt`：结论行「28 项中唯一需要修改的是 `use_fast_math`（True→False），其余 27 项与产品一致」；我核该文件 43 个非空行 = 说明 + 28 项表 + 结论，逐项表只有 `use_fast_math` 行同时出现 True/False ✓。

---

## 6. 证据完整性（复核点 6）

- S3 manifest **34 行**：逐行 `sha256+bytes` 对 committed blob **34/34**、对现盘 **34/34**、不自引用；目录 35 个文件 = 34 登记 + manifest 本身 ✓。
- S3a manifest **2 行**（README/receipt）：committed/现盘双向一致 ✓。
- LF/无 BOM：两个目录全部文件 CR=0、无 NUL/BOM；`attempt1/facts.txt`、`attempt2/facts.txt`、`attempt2/guard_aggregate.json` 的 CRLF→LF 归一已在 manifest 头与 README §8 说明，我核**逐行内容一致**（仅换行符差异）✓。
- 证据副本 vs `target/` 原始件：attempt2 下 19 个文件里 17 个逐字节相同，`facts.txt`/`guard_aggregate.json` 为 LF 归一（内容一致）✓。
- scope diff（`d7f705d0^..8a967468`）：只有 S3/S3a 证据 + 4 个参考文件 + `guards-r02-skybox.json` + `.normify`；**无 `src/`、`tests/`、判据/容差改动** ✓。
- 禁词：见 §2/O1。

---

## 7. 观察（O1 为必修，其余不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | **必修｜一般** | S3 `README.md` 字面含 4 个禁词（2 处「不是…常规阈值通过」+ 1 处「不写…」自列三词）→ 卡面第七条「禁词 0」字面不合格（内容是诚实否定句，无结论性主张） | 按 §2 的三句最小改法改写；不动任何数字，不需重跑窗口 |
| O2 | 建议 | `attempt2/guard_aggregate.json` 的 `suite` 字段 = **`r05-s3b-hfield-validate`**（本窗是 `r02-skybox-validate`）。根因：`scripts/validate/render-bvh/run-window.ps1:342` 把 `suite = 'r05-s3b-hfield-validate'` **硬编码**；`facts.txt`（`guards_suite=r02-skybox-validate`）、guards.json、guard.log 均正确 | 未来窗口把该字段改成 `$guard.suite`（runner 改动需新版本/重签）；本窗在 README 注明「aggregate 的 suite 字段为 runner 硬编码遗留，实体身份以 facts/guards 为准」 |
| O3 | 建议 | README §4 与 S3a §2 声称「`-GuardsOnly` → `GUARDS_OK=77/77`（stash 9/9）可复核」，但证据与 `target/` 均**没有**该 dry-run 的独立输出（目标目录无 guards-only 产物）；实质证据是窗口内 `guard.log` 的 `GUARDS_OK=77/77`（我已核）与 stash 9/9（我已核） | 补一份 guards-only 日志，或把措辞改为「窗口 precheck 77/77 + stash 一致性（本卡已核）」 |
| O4 | 建议 | `attempt1/exception-block.txt` 引用 `monitor-pre-window-exception.csv`（60 样本、17 个 ≥40 clean、free_min 12 833），但该 CSV/result **未随证据入库**（只在 `target/render-bvh/r02-skybox-window/`，我确认存在 3 800/1 528 B） | 标注为 target-only 瞬态，或把 attempt1 例外监 CSV/result 一并入库（attempt1 是非法轮，可不作主证） |
| O5 | 记录 | `run-window.ps1` 顶部注释仍写 R05 的 stash 路径（`target/render-bvh/r05-s3b-window/frozen-exes/`），与本窗不同；属注释残留（O2 的同一文件） | 随 O2 一并修 |
| O6 | 记录 | guards 签字字节依赖本地 `.git/info/attributes` 的 `-text`；证据已在 S3a README/receipt 登记为已知限制（R05 O7 同类） | 接受（已登记） |
| O7 | 记录 | attempt1 也使用了同一 validation-only 例外（`-OpenUtilLimit 60`，其例外监 60/17 clean）——README §5 只详述 attempt2 例外，attempt1 例外在 `attempt1/exception-block.txt` 中；两轮一致、不冲突 | 接受（记录） |

---

## 8. 边界

1. 未开 GPU、未重跑窗口：所有数字来自留档 CSV/日志/facts 的**独立重算**，不是本卡重新渲染；设备像素结果我不重跑。
2. 判据的「两跑 sha 相同」由测试内 `assert_eq` 钉住（日志只打印 run1 的 sha）；我核了断言存在与 `test result: ok`，未单独重跑两跑。
3. attempt1 的 4 个失败日志只存在于 `target/`（未入库），我从 target 原始日志交叉核对；证据里只有 note/entity-results。
4. guard aggregate 我按证据声明的口径（PowerShell culture sort）复算；Python 序数/casefold 不是该口径（已在材料说明）。
5. 未改代码/证据/卡；本卡不产任何渲染数值结论。

---

## 9. 复现命令（仓库根目录）

```text
target\render-bvh-precheck\venv312\Scripts\python.exe -B docs\development-evidence\render-bvh\r02-skybox-s3-validate-check\recompute.py
powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\render-bvh\r05-s3b-hfield-validate\materials\recompute-aggregate.ps1 -GuardsPath scripts\validate\render-bvh\guards-r02-skybox.json
# capture 脚本链复现：apply attempt1/capture-skybox-fix.diff 到 3e3665f0 的脚本 blob → LF 归一 → 再翻 use_fast_math
```

（脚本 `problems` 见 `recompute.json`；键数字摘要见 `logs/audit-numbers.txt`，修复链与例外审计见 `materials/fix-chain-and-exception-audit.txt`。）

---

## 10. 收口

- 本卡证据提交：本目录；被核对象 `d7f705d0`/`fa5cfcfd`/`f0213748`/`99b88d1f`/`65022c3a`/`8a967468` 保持不动（只读复核）。
- O1 修完（仅 README 三句措辞）即可关闭 R02 整片；O2/O3/O4 建议随后续小片处理，不影响本窗结论。
