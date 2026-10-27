# UNC-3 分支收口 S4 独立复核（设备对照 target + 窗口 attempt1）

**任务**：`2026-10-27-unc3-branch-closure-s4-independent-check`（owner QA）。
**被复核提交**：`67810d8b`（开窗前 mesh 材料）+ `055d0c8a`（窗口回填）；复核时 HEAD `055d0c8a`。
**结论**：**pass / 阻断 0 / 观察 5（1 一般 + 4 建议）**。不 commit 以外无其它动作；本片由 QA 自行收口（check/done/commit）。

只读复核：未重开窗口、未跑 cargo/GPU；只读日志/samples/guard/冻结 exe 并用 CPU 复算。
`facts.txt` 的 `guard_aggregate=System.Collections.Specialized.OrderedDictionary` 是已知缺陷（M3 同型），
权威数字一律取 `validation-attempt1-guard.log` + `validation-attempt1-samples.csv`（我已独立重算，见 §3）。

## 1. 十条死点逐条结论

| # | 要求 | 我核到的结果 |
| --- | --- | --- |
| 1 | 10 实体 exit 全 0 + 逐实体日志 | **一致**：facts `exit_codes:` 10 项全 `=0`、`failed_entities=0`；10 个逐实体日志全部 `test result: ok`、无 `failures=[1-9]` 行；窗口 `10:36:58.831→10:40:57.659`（body 238.828 s，claimed 238.8；runner total 251.5） |
| 2 | 采样重算 + floor + guard | **一致**：samples **871**、used_peak **3144**、used_min **3035**、free_min **12918**、util_max **46**、≥80 连续 **0**、residual **0**/process_exited true；delta **95**（vs guard baseline 3049）/ **96**（vs 首基线点 3048）/ **109**（vs 采样最小 3035）；floor=`min(10240,13014−3072)`=**9942**，free_min 12918 > 9942；open 门 13014 ≥ 10240；guard.log 聚合 7/7 与重算一致，facts 采样行与之同口径 |
| 3 | sha-guards 25 行全 match + 窗后 10 exe + runner + 参考 | **一致**：25 行=10 exe+13 fixture+2 ref，`match=True` 25/25；我逐个独立哈希 25/25 与声明 sha 相同（exe 取 `target/s4-freeze/`，fixtures/参考取仓库文件）；`binary-shas.txt` 10/10 与冻结 exe 实哈希一致；runner `12013 B / e7379663…` 与声明/实文件一致；两份 S3 参考 `24d89e92…`/`b6abb8fa…` 未动；旧参考 11 文件 mtime 仍 2026-10-25、git 干净 |
| 4 | branch 9 case + U7/U8 record-only | **一致**：9 case `nacon==ref_count` 且 `failures=0`（3/4/4/2/2/1/2/4/3）；U7 `device_count=1 == ref 1` 且 dist/pos 已记；**U8 `device 1` vs `ref 4`** 如实记录，record-only 不进分母，未动 fixture/判据/代码（`tests/`、`src/`、`fixtures/` 窗口前后零 diff） |
| 5 | launcher 4 case + 既有判据不变 | **一致**：hfield-capsule **275/275** bit、ULP 0；hfield-sphere **314/314**、sdf-box **3122/3122**、ellipsoid-box **80/80** 数字与 M2/M3 记录相同；lancher 4 case 全 `failures=0`；既有 3 case 在 `b3e56a5f` 之后代码零改动 |
| 6 | 零 diff / 零编译 / 无 BOM | **一致**：`git diff 61b771ec 055d0c8a -- src fixtures scripts/compare_primitive_capture.py …` **空**；`tests/` 在 `b3e56a5f..055d0c8a` **空**；`target/debug` 窗口时段（±5 min、排除 s4 目录）**0 个文件被修改**（我独立扫描）；S4 证据全部 UTF-8 **无 BOM**；`guard.log` 证据副本 654 B `2bb90c90…`（无 BOM）vs 原始 657 B `b503d9a9…`（带 BOM），剥离后字节相同 |
| 7 | 措辞 | **一致**：所有命中「「窗口通过」类结论/「已覆盖」类结论/禁词清单（上游也限制、与上游一致 等）」的行都在**否定句或禁词清单**里（我逐行核过）；U8 定死句在 README §1.1/§10 两处；`boundaries.window_claimed_passed=false`、`branch_claimed_covered=false` |
| 8 | 二次冻结 / 重编分析 / 留档 / 历史记录 | **一致**：`target/s4-freeze` 10 exe=**2 s4-build + 8 m3-freeze**（m2 `a6b35485…`、m3 `e502b8eb…` 与 M3 signed 相同；我独立哈希 10/10 与 README 表一致）；`superseded-rebuild/` 留存 `277b0435…`/`468c1a96…`；`git diff 61b771ec 055d0c8a -- docs/development-evidence/changes/` 只新增 s4-window 记录，历史记录未动 |
| 9 | 回归面无回退 | **一致**：multicontact 3（含 box-box-margin 391/392、max_pos_ulp=1）、primitive 3、plane 6、sphere_capsule 4、branches 6+4 swap、M2 2 case（4/4 bit、`contact.geom=[1,0]`）、M3 1 case（4/4 bit、`[0,1]`）、stage2 17 dumps（8 case×2+sdf-box）均「两次一致 true」（record-C） |
| 10 | 证据/回执完整性 | **一致**：`window-receipt.json` `/task_id = …-s4-window`、22 个 evidence 条目中 17 个带 sha 全部与实文件一致（含 10 实体日志、facts/samples/baseline/guard/sha-guards/machine-load/binary-shas）；README §12 的 7 个 sha 声明全部一致；README 272 行、window-receipt 484 行（JSON 例外已注明）、新测试 341/354/418/499 行均 ≤500 |

## 2. 独立复算方法的边界

- 不重开窗、不跑 cargo/GPU：窗口数值来自已落盘 `samples.csv`/`guard.log`/逐实体日志；我用 Python 重算
  （脚本 `verify_s4.py`，输出 `verify_s4.json`），并独立哈希冻结 exe/fixture/参考。
- 设备输出数值本身不重跑（需 GPU 窗口）；我核的是「日志里的数值 ↔ 参考 ↔ 判据口径」一致性，
  以及「判据支持代码零改动」。
- mesh 常量可复现性：用 pinned venv（mujoco 3.15.0）重跑已入库的
  `scripts/authoring/g10-p5/author_branch_closure_mesh_data.py --check`：**7/7 输入 sha 命中、42/42 数组、
  FULL_FILE_MATCH=True、RESULT=PASS、exit 0**；`branch_closure_mesh_data.rs` 实测 sha `2072f4ac…` 与
  `mesh-data-provenance.md` 声明一致；`mesh-data-check.txt` sha `bf02a73f…` 与声明一致。

## 3. 采样/窗口数字重算（权威口径）

| 项 | 声明 | 我重算 |
| --- | --- | --- |
| samples | 871 | 871 行（CSV 表头 + 871 数据行） |
| used_peak / used_min | 3144 / 3035 | 3144 / 3035 |
| free_min | 12918 | 12918（> floor 9942） |
| util_max / ≥80 连续 | 46% / 0 | 46 / 0 |
| delta 三口径 | 95 / 96 / 109 | 3144−3049=95；3144−3048=96；3144−3035=109 |
| floor | min(10240,13014−3072)=9942 | 9942；首基线点 free=13014 ≥ 10240 |
| 3 点基线 | 3048/13014、3049/13013、3049/13013 | baseline.txt 原文一致 |
| post / 结束基线 | 3048/13014；3052/13010 ×3 | facts 原文一致 |
| guard aggregates | samples/baseline_used/used_peak/delta_peak/free_min/util_max/util_streak | 7/7 与重算一致 |
| temp_max | —（未声明） | 32 ℃ |

`guard.log` 原始（657 B / `b503d9a9…`，BOM）与证据副本（654 B / `2bb90c90…`，无 BOM）：
我验证副本 = 原始去掉 3 字节 BOM 后的同一内容。

## 4. 观察（按级别）

### 一般（1）
- **O1｜README §8「回归面：13 组合（G5 multicontact 3 + launcher 3 + primitive 3 + B1 plane 6 + B2 4）」
  的数字不自洽**：括号内列举 3+3+3+6+4 = **19**；且 launcher 现为 4 case（新增 hfield-capsule），
  「launcher 3」是旧数。窗口实际执行 33 个 case 级比较（multicontact 3 + launcher 4 + primitive 3 +
  plane 6 + sphere_capsule 4 + branches 6+4 + M2 2 + M3 1）＋ stage2 17 dump（record-C）。
  不影响逐项结论（各项 failures=0 我已逐日志核过），但 S5 汇总若引用「13 组合」会以错传错。
  建议统一为「G5 9 + launcher 新增 1 + B1 6 + B2 4 + B3 10 + M2 2 + M3 1 = 33 case 比较」或明确 13 的定义。

### 建议（4）
- **O2｜二次冻结的语义边界建议在 S5 写明**：m2/m3 用的是 M3 窗口冻结二进制（构建 01:06），而 M3 提交
  `f1e5cda1` 在 01:28（含 `abi.rs` O7 与测试拆分）。所以 S4 的「M2/M3 回归」跑的是 **M3 提交前一代**
  二进制（设备接触路径与 M3 窗口同源；O7 输入校验由 S4 新 target 覆盖）。结论可用，但 S5 别写成
  「当前 M3 源码回归通过」；建议写成「M2/M3 回归对象 = M3 窗口冻结二进制」。
- **O3｜`facts.txt` 的 `guard_aggregate` 字典字符串化缺陷仍在**（M3 同型），需靠 `guard.log`+`samples.csv`
  重算；本窗已如实注记。建议修 runner 的聚合输出（展开键值而不是拼接对象），减少每窗人工重算。
- **O4｜`guard.log` 的 BOM 来自共享 `scripts/gpu-window-guard.ps1`**（证据副本剥 3 字节 BOM，本窗已按
  Chief 口径注记）。建议把 guard 脚本写文件改为 `[IO.File]::WriteAllText(..., UTF8Encoding($false))`，
  未来窗口日志天然无 BOM。
- **O5｜`stage2_equivalence_window.log` 中文行被控制台代码页转码**（README §9 已注，数值/ASCII 不受影响）；
  建议 runner 在调用实体前设 `[Console]::OutputEncoding`/或让实体直接写 UTF-8 文件，避免中文行乱码。

## 5. 不宣称 / 边界

- 本片不是重跑：不宣称设备数值可复现于本机；也不宣称「窗口通过」或任何分支「已覆盖」。
- branch 组零失败仅是本窗执行记录；U7/U8 保持 record-only（U8 device 1 vs ref 4 = 行为差异/限制存在），
  不推广到一般 mesh；`hfield-capsule` 的 275/275 仅本窗 case 记录。
- 未改产品代码/设备源/fixtures/判据/容差/参考/期望值；证据为只读复算产物。

## 6. 证据清单 / 复现

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本报告 |
| `claims-audit.json` | 死点 1–10 ↔ 我的核对结果（含数值） |
| `verify_s4.py` / `verify_s4.json` | 采样/guard/floor/sha-guards/逐实体日志/文件 sha/BOM/零编译 的机器可读复算 |
| `manifest.json` | 本目录逐文件 sha256 |
| `receipt.json` | 任务 id / 判定 / 计数 / 关键数字 |

复现（只读）：`python verify_s4.py`（读 S4 证据 + `target/s4-freeze/` + `target/s4-branch-closure/attempt1/`）。
