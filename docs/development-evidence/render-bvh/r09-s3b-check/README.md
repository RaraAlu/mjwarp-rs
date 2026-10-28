# R09-S3b 独立复核（阴影数值判据 / 三轮窗口记账 / runner 编码修复 / S2b 解锁断言 / 窗口事实）

任务：`2026-10-28-render-bvh-r09-s3b-check`（P8 渲染与 BVH；scope 仅本目录）
被复核对象：`2026-10-28-render-bvh-r09-s3b-shadow-validation`（`7ce747fd` 证据三段分写 + `2d05433f` 收口，卡 verified）
被判树 HEAD：**`446fcb03`**（`2d05433f` + 勘误 `b20d754c`/`854ac0b4` + 门禁刷新 `446fcb03`；被复核卡保持 verified，未 reopen）
上游锚点：冻结 MJWarp 3.15 `render.py` / `ray.py`（`71da24d`）；参考 `fixtures/render-bvh/scene-shadow/capture-exact/`

## 0. 结论

**verdict：pass；阻断 0；观察 7（O1–O4 已由勘误卡处置；O7 勘误致门禁记录过期→按最终树重跑并随 `446fcb03` 入库，Chief 认领为判定副作用；O5/O6 记录级、接受）。**

本复核为**只读**：未重开窗口、未跑 runner/采样器/守卫、未改任何仓库文件。窗口数字全部来自入库产物
（`windows/{attempt2,attempt2-r2}/`）+ `target/` 原始产物只读复算；四门禁未重跑（本卡复核点不含），
改为只读核对两卡 `.normify/runs` 记录——代码树自 `d501118b` 到 HEAD 对 `src/tests/fixtures/Cargo.*` **零改动**，
我上一片对同一代码树的独立复跑日志（`logs/`）可沿用。

所有数字均由我自己重算（`recompute.py` → `recompute.json`，`problems=0`），未采信 Backend/Arch 给定值。

## 1. 三层核验（勘误后，Arch 追加点）

### 1.1 attempt1 产物 sha/bytes 对原件实测

从 `target/s3b-validation-window/attempt1/`（20 文件，mtime 20:03:14–20:13:45，未改）现算：

| 文件 | 实测 sha256 | bytes | README §5.1 / receipt `attempt1.raw_products` |
| --- | --- | --- | --- |
| `facts.txt` | `b704b9bb7d69411b2b5c50315960518a5c1519d9cf0c071315918076c9a0f227` | 1 921 | 一致（各 2 处：§5.1 + 勘误小节） |
| `entity-results.csv` | `603df98b6c9b4f6245735d050178ba8ca9259aa3b440a8e5a6b26ae07c9e7677` | 1 139 | 一致 |
| `guard.log` | `c2269841a855aa5774933861f8aa18a25774bf872fdd88eeb307d4d6676de01c` | 81 | 一致 |

分区检查：README 勘误小节之前 attempt2 错值出现 **0** 次；receipt `attempt1.raw_products` 已为真值，
错值只留在 `errata` 历史字段（L223/L224：先列 attempt2 错值再列 attempt1 真值）。勘误前错值是
attempt2 同名文件（`7ba67015…/1 907`、`96c781dd…/1 135`、`49f157b7…/81`）——我独立算了 attempt2 原件 sha 确认。

### 1.2 manifest 逐行对 committed blob

- 证据目录 **53 文件**；`manifest.txt` **52 条**（不含自身、`recompute.json` 为产物不列），逐条对 **committed blob**
  （`git show HEAD:<path>`）**52/52 相符**；**无自引用**；两处新增/更新行（`README.md 23155 B`、`receipt.json 24891 B`、
  `erratum-grep-report.md 5256 B`）均正确。
- 证据文本 LF、无 BOM；禁词 0（任务卡 3 词）。

### 1.3 勘误 grep 清单独立抽查

我独立遍历证据目录扫六个 sha（`b704b9bb/603df98b/c2269841`=attempt1；`7ba67015/96c781dd/49f157b7`=attempt2），
共 **24 处**命中：

- `manifest.txt:21-23` → 全为 `windows/attempt2/` 行（**正确**，attempt2 归属）；
- `README.md:57`、`:160` → 全为 attempt1 真值（§5.1 与勘误小节，**正确**）；
- `receipt.json:54` → attempt1 真值；`:223/:224` → `errata` 字段先 attempt2 错值、后 attempt1 真值（**明示记录**）；
- `erratum-grep-report.md:37-39/45-47` → 报告内参照表（**正确**）；
- **错误语境 0 处**（无「attempt2 值写进 attempt1 段落」残留）。
- 报告本体 `3d981020…`/5 256 B，且以该 sha 入 manifest（我按 committed blob 复核）。

### 1.4 勘误后 S3b 门禁记录过期与刷新（`446fcb03`）

勘误修改了 S3b 卡 scope 内的证据文件（README/receipt/manifest/新增报告），使其**门禁记录过期**。处置：按最终树
（`854ac0b4`）重跑 S3b 四门禁，并以 `Normify-Fixes` 流程把「修正记录 + 4 条 runs」随 **`446fcb03`** 入库
（仅 5 个 `.normify` 文件；`854ac0b4..446fcb03` 对 `src/tests/fixtures/Cargo.*` **为空**；S3b 卡保持 verified 不 reopen）。

我只读核对：刷新后 `.normify/runs/<S3b>/{fmt,clippy,clippy-cuda}.json` exit 0、`unit.json` at `2026-10-28T13:16:24Z`
（21:16 +08）**1050 过 / 0 败 / 387 ignored**；`normify check <S3b>` **exit 0**、`normify check <erratum>` **exit 0**；
板面 `已验收 530 / 运行 1（本复核）/ 错误 0 / 警告 0`。21:16 的门禁重建未覆盖 9 个窗口 exe（现盘字节仍与冻结表 9/9 相符）。

## 2. 判据零放宽（从 r2 实体日志逐行核）

- shadow-on 对 `capture-exact/shadow-on`（同配方）：cam0/cam1 rgb `mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0`；
  depth `mismatch=0 max_ulp=0 beyond_ulp=0`（76 800 / 43 200）；seg `mismatch=0`；两次运行 sha256 相同
  `rgb=f044f3fa… / depth=de732643… / seg=065caf08…`（用例内断言通过）。
- shadow-off ① 对 `capture-exact/shadow-off` 同判据全 0；② 对 pre-unlock `s7b/*_fast_math_false.json`
  **逐位回归**：depth/seg `mismatch=0`、rgb `max_byte_diff=0`；sha `rgb=af9df331…`（与 shadow-on 不同）、depth/seg 相同。
- 可见性对照：product `rgb_diff=4745 / depth 0 / seg 0` == reference `4745 / 0 / 0`（只作语义交叉校验）。
- 测试源码代码行只引用 `capture-exact/`（6 处 `include_bytes!`）；`capture/`（fast-math 组）只出现在注释，
  **未进断言**；`git diff 93f983e5..HEAD` 对 `capture-exact/`、`capture/`、`scene.xml` 为空。

## 3. 三次记账与仪器缺陷处置

| 轮 | 事实（我重算） | 结论 |
| --- | --- | --- |
| attempt1（有效轮 1） | cam1 `mismatched=1451 / beyond_1_lsb=1436 / max_byte_diff=77`；product 3537 vs reference 4745；`render_shadow_reference`/`render_e2e` exit 101 | 红（a 阴影 cull 根因、b e2e 旧断言），产物留档未改 |
| attempt2（本材料第 1 次开窗） | 9 实体全 exit 0；shadow-on 两相机全 0；可见性 4745==4745；但 facts `shadows_acceptance_* exit_ok=`（空）+ `failures=shadows_acceptance_missing:render_kernel_flags;…:render_kernel_rgb`，`guard.log` 无 `WINDOW_RUN_DONE` | 产品判据全绿、**仪器红**（runner 编码缺陷） |
| attempt2-r2（第 2 次开窗，仪器修复重放） | `PRECHECK_OK`/`GUARDS_OK=56/56`/`WINDOW_RUN_DONE`；9 实体全 exit 0；`shadows_acceptance_*` 四条件全 True；判据全 0 | 绿；`attempt_valid_rounds=2`（重放不新增判据轮） |

「runner exit 10」：产物内**无**独立落盘，来源=调用方 PowerShell 返回码；产物内可复算证据=facts 两条 failures +
`guard.log` 缺 `WINDOW_RUN_DONE`（该码本身不可从产物复算）——README §5.2/§7 已按此写 provenance。

## 4. 编码根因与修复核验（Windows ANSI(936) 语义复现）

用 `MultiByteToWideChar(936)`（PS 5.1 按 ANSI 读 .ps1 的解码语义）独立复算：

| 项 | 旧签名 runner（留档） | 修复版 runner |
| --- | --- | --- |
| 字节 / sha256 | 16 458 / `53dc05ed…ba45057709`（与留档文件一致） | 16 454 / `8a73ce53…decab926` |
| 非 ASCII 字节 | **129** | **0** |
| 行数（elements / splitlines） | ANSI **342 / 341** vs UTF-8 **343 / 342** | ANSI==UTF-8（343 / 342） |
| `$exitOk` | ANSI 解码下位于第 **212** 行且该行以 `#` 开头（被并入全角句号注释、吞换行） | 第 **213** 行独立语句、非注释 |
| code-only 行 | 336 | 336（`code_only_equal=True`） |
| 注释行含 `$` | — | **0** |

**离线回放**：用四条件（entity exit=0 + `test <NAME> ...` + 其后 `ok` 行 + marker）跑 attempt2 冻结日志 →
`render_kernel_flags`、`render_kernel_rgb` 均 `ALL_TRUE`（修复版正则能正确判 attempt2 的两条实际通过用例）。

## 5. 窗口事实独立重算（attempt2-r2）

- 采样：`samples.csv` **161** 行；peak **2822**（seq 5）、used_min **2548**、free_min **13240**、util_max **11%**、streak(≥80) **0**；
  delta 三口径 **270 / 86 / 274**、sustained **270**；interval median **0.299 s**（0.2864–0.3191，0.25 s 采样器）。
- 基线/floor：`2552/13510/2/32 ×3`；floor = `min(10240, 13510−3072)` = **10240**，未破。
- 9 实体 exit 0；body **48.152 s**（entity 区间）；`attempt_valid_rounds=2`；residual 0；compute-apps post `30/0`。
- 零编译：deps `1826 / 20:34:20` 前后一致、`zero_rust_compile=True`；窗后 9/9 复查 0 mismatch。
- 守卫 **56/56**：我按 `sha-guards.txt` 对**当前盘**重哈希 56/56 全 match（9 exe+47 输入）。
- 预窗监测：60 样本、最长连续 60、util 2–4、free_min 13510、`20:48:57.637→20:53:57.696`。
- attempt2 对照重算：**160** 样本、peak 2849(seq 5)、free_min 13213、util 13%、delta 270/260/270、interval 0.2991；
  即 README §5.2 已改为 **160**，§5.4 的 r2 仍 161。

## 6. 材料冻结（HEAD `415006d8` 树，窗口前）

- deps 快照原文 **3 条**：`20:31:29.658`（GPU util **21%**）、`20:32:58.615`、`20:34:00.728`；
  采用后两条：count **1826/1826**、max_write **`20:26:42.825`**、间隔 **62.113 s**、build_procs 0；
  首条因 util 21% 舍去——README §5.3 已按此写并保留三行原文。
- `guards.json` = `a942de1d…`/18 477 B（与证据声称一致）；`guards-check-attempt2-final` 输出 `GUARDS_OK=56/56` + `GUARDS_ONLY_DONE`。
- 9 个冻结 exe：按 `final-freeze/exes.txt` 对**当前盘**逐字节复核 **9/9 相符**（窗口 exe mtime 仍 20:34:10–20:34:20；
  20:57 最终树门禁重建的是不同 feature 指纹的其它 exe，未覆盖这 9 个）。

## 7. 证据完整性

- 53 文件 / `manifest.txt` **52 条**（=52 个文件，不含顶层 manifest；含新增 `erratum-grep-report.md` 与嵌套
  `attempt1-fixes/e2e-negative-control/manifest.txt`）/ 无自引用 / LF 无 BOM / 禁词 0（见 §1.2）。
- `windows/{attempt2,attempt2-r2}/` 入库副本 vs `target/` 原件：每轮 **18 个逐字节相同 + 1 个 `guard_aggregate.json` 仅 CRLF→LF 归一化**
  （target 原件 `ed7df29b…/1 496`、`8b65c8e5…/1 494` → 入库 `fb81e481…/1 455`、`951f20a4…/1 453`，与 README §5.4 一致，数值内容不变）。
- 原始产物未改：attempt1 20 文件（mtime 20:03:14–20:13:45）、attempt2 19 文件（20:42:31–20:43:22）、attempt2-r2 19 文件（20:54:01–20:54:53）。
- 门禁记录（只读）：validation 卡记录已按最终树刷新（`446fcb03`，unit at 13:16Z）、erratum 卡记录（13:12Z），两卡
  fmt/clippy/clippy-cuda/unit 均 exit 0，unit **1050 过 / 0 败 / 387 ignored**；代码树自 `d501118b` 起对
  `src/tests/fixtures/Cargo.*` 零改动（`logs/` 内为我同树独立复跑日志）。
- 勘误提交范围：`b20d754c` 仅 5 文件（卡 + README + receipt + manifest + `erratum-grep-report.md`，+215/−8）；
  `2d05433f..854ac0b4` 对 `src/tests/fixtures/Cargo.*` 为空。

## 8. 红线 / 判据 / 参考

- `git diff 415006d8..854ac0b4` 对 `src`、`tests`、`fixtures`、`Cargo.toml`、`Cargo.lock` **为空**。
- 行数：`render_common.cu` 393、`render_megakernel.cu` 340、`render_shadow.cu` 30、`render.cu` 499、`launch.rs` 500、
  `cast.rs` 494、`model/render.rs` 500、`camera_output.rs` 499、`render_shadow_reference.rs` 307、`shadow.rs` 416、
  `shadow_scene.rs` 238、`render_e2e.rs` 500——全部 ≤500。
- `DEVICE_SOURCES=28`；capture-exact 6 个 npz sha 与 `shadow.rs` 冻结常量逐条相符（on/off depth、seg 逐字节相同）。

## 9. 观察（O1–O4 已由勘误卡处置；O5/O6 记录接受）

- **O1（一般｜已处置）** attempt1 产物 sha 误写为 attempt2 值（README §5.1 L57、receipt `attempt1.raw_products`）。
  勘误 `b20d754c`/`854ac0b4` 已修正并加勘误小节/`errata` 字段；我实测原件三处 sha/bytes 与现文一致（§1.1）。
- **O2（一般｜已处置）** README §5.2 attempt2 采样数 161 → 实为 **160**（161 是 r2）；勘误已改，我重算一致（facts/csv 160）。
- **O3（记录｜已处置）** 「runner exit 10」在产物内无独立落盘；勘误已写明 provenance（来源=调用方返回码；
  产物内证据=facts 两条 failures + guard.log 缺 `WINDOW_RUN_DONE`；返回码本身不可从产物复算）。
- **O4（记录｜已处置）** deps 快照有 3 条原文，采用后两条（62.113 s）、首条 20:31:29 因 GPU util 21% 舍去；勘误已写明并保留三行。
- **O5（记录｜接受）** `attempt1-fixes/e2e-negative-control` 的 abandoned 卡文件按 normify 规则无法入库、保持未跟踪
  （README 已说明）；我确认文件仍在盘上、内容完整、未删除。
- **O6（记录｜接受）** `windows/{attempt2,attempt2-r2}/guard_aggregate.json` 入库为 CRLF→LF 归一化副本（原件 sha/bytes 已记录）；
  我复核「18 逐字节 + 1 归一化」口径成立，数值内容不变。
- **O7（记录｜已处置，Chief 认领为判定副作用）** 勘误修改了 S3b 卡 scope 内证据文件 → S3b 门禁记录过期。Arch 按最终树
  重跑四门禁，并以 `Normify-Fixes` 随 **`446fcb03`** 入库（修正记录 + 4 条 runs；仅 `.normify`，零代码/判据改动，卡保持 verified）。
  我核对：刷新后 unit **1050 过 / 0 败**（at `13:16:24Z`）、`normify check <S3b>` **exit 0**、`<erratum>` **exit 0**、
  板面 **已验收 530 / 运行 1 / 错误 0 / 警告 0**。

## 10. 边界（我没有做的）

- **未重开窗口**：runner/采样器/守卫/9 实体均未重跑；窗口事实只来自入库产物与 `target/` 原始产物只读复算。
- **未重跑四门禁/GPU 用例**：本卡复核点不含；代码树自 `d501118b` 零改动，沿用我同树独立复跑日志并只读核对两卡门禁记录。
  上一片对同树 dev 复跑的 3 条 GPU 用例数字（shadow-on 0 / shadow-off 0 / 可见性 4745）与本窗口 r2 日志一致。
- 不分场景外推：只对 `scene-shadow` + `capture-exact` 单场景参考成立；SDF（gtype==8）shadow cull 缺口未覆盖
  （backlog `2026-10-28-render-shadow-sdf-gtype-cull-gap` 记录，本卡不 start）。
- 本证据不作窗口判定、覆盖声明、精度结论或等价结论（含上游/双平台）；数值只作事实记录。

## 11. 复算入口

```text
<repo>/target/render-bvh-precheck/venv312/Scripts/python.exe docs/development-evidence/render-bvh/r09-s3b-check/recompute.py
```

`recompute.json` 含：证据 53 文件 EOL/BOM、manifest 52 条对 committed blob、attempt1 实测 sha 与 README/receipt 对照、
分区错配扫描、erratum 报告 sha/bytes 与 24 处归属扫描、r2 判据/window facts/守卫/冻结 exe/编码 ANSI 复算/离线回放、
S2b 四条件、红线 diff、行数、两卡门禁记录、勘误提交范围——`problems = []`。
