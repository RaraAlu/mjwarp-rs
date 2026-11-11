# QA 复核：R06 多帧 recipe 模块 cam_res 注释措辞修正（`a20efe2a`，卡 L）

- 任务：`2026-11-11-render-r06-flex-multiframe-recipe-params-comment-wording-check`（owner QA）。复核对象：提交 **`a20efe2a`** + 卡 `2026-11-11-render-r06-flex-multiframe-recipe-params-comment-wording`。零 GPU、只读复核。
- 基线：`4353796d`（卡 K 内容）。

## 结论

**复核通过（0 阻断，1 低 + 2 观察）。** 改动确为**注释-only**：`git diff` **4 增 / 4 删**，我程序化核过**每一条 +/− 行都以 `//`/`///`/`//!` 开头**（无代码行）；断言、`RECIPE_PARAM_KEYS`、21 项比对与防漏守卫**逐字未动**（diff 只落在 4 处注释）；文件仍 **188 行/10,100 B**（<500 棘轮）；测试 exe `0966bf4c…` → **`cdd85988…`（同 118,588,904 B）** 与我实测 on-disk 一致；证据 manifest 我重算 **6/6** 全对、门禁 unit **1,406/0/402**、测试 41/0/1。

## 一、注释-only 可复核（✅）

- `git show a20efe2a -- tests/support/render_bvh/multiframe_recipe.rs`：`+4/−4`；我逐行检查 `+`/`-` 行开头（排除 `+++`/`---`）**全部为注释**（`//!`/`///`）——`comment_only=true` 成立。
- 改动位置恰为 4 处 cam_res 口径注释（第 8/51/131/168 行），新措辞准确：**多帧 capture 无 `context.resolved` 段、无独立对照源 ⇒ 实际逐项断言 27 项（26 recorded + nworld），`cam_res` 为名义第 28 项、不新增字面断言**——与卡 C 复核 L1、卡 K C-L1 addendum 一致。
- **断言/逻辑零改动**：diff 未触及 `assert_params_match_capture`、`RECIPE_PARAM_KEYS`、21 项新增断言、防漏守卫；文件行数 **188 不变**（字节 9,812→10,100 仅注释文本变长）、`<500` 棘轮 ✅。
- 测试复跑：证据 `logs/test-log.txt` 为 **41 passed / 0 failed / 1 ignored**（42 tests；ignored = GPU 用例，零 GPU 未执行）——与卡声明一致。

## 二、exe 身份与重冻声明（✅）

- 我实测 on-disk 实体：`target/debug/deps/render_flex_multiframe-7cc46ab495d10909.exe` = **118,588,904 B / `cdd8598880b42ffffcd77adf9d70d15e6d7601d78757a8c81d0ed8041044e674`**（mtime 13:02:42），与证据/卡声明一致。
- old = 卡 E 点值 `0966bf4cccc7312c…`/118,588,904（同 size、注释改动只影响 debug info）；证据写明「**R06 后续 GPU 窗需重冻，本卡不重冻、不占窗**」与「同源重建不可逐字节复现、身份以冻结产物为准」——口径正确。

## 三、证据 / manifest / 边界 / 门禁（✅ 除 1 条低）

- 证据目录 `r06-flex-multiframe-recipe-params-comment-wording/`：README `3c4450a5…`/3,095、receipt `122ededa…`/3,336、manifest `eb5720c9…`/738——与派工值逐位一致；receipt 关键字段 `comment_sites_fixed=4`、`assertions_unchanged=true`、`behaviour_unchanged=true`、`k_l1_closed=true`、`k_backlog_note`（K backlog=1 指 F-L1）齐全。
- **manifest 我独立重算：列 6 条（README + receipt + 4 logs），全部 sha/bytes 匹配、不自列**（见低 L1：不是 7/7）。
- 边界：提交 13 files = 1 个 test 注释文件 + 证据 7 + `.normify` 5；**无** `src/`/`fixtures/`/`scripts/` 改动、guards/freeze script 未动、不改判据/参考/容差/行为；零 GPU、零窗。
- 编码：改动的 docs 与 tests 文件全 UTF-8/LF 无 BOM/CR/NUL（`git ls-files --eol` = `i/lf w/lf`）。
- 门禁：fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**。

## 四、K 的 L1 闭环判定（✅）

- 我卡 K 复核的 L1 是「K 的 `backlog=1` 未计 C-L1 留下的测试源注释 follow-up，且该 follow-up 未登记」。
- 本卡把 **C-L1 测试面**（4 处注释）改为准确表述，K 的 **C-L1 文档面**（addendum）此前已做 ⇒ **C-L1 全闭**；K 的 `backlog=1` 仍指 **F-L1**（账本拆字段/checker 断言）——本卡 README §3 与 receipt `k_backlog_note` 明确写出，口径自洽。**K 证据字节未改**（K 已 verified，历史不改写），L 目录引用即可。因此我卡 K 的 L1 **由本卡闭环**，无需再补登卡。
- 无新判据：本卡只在测试注释里措辞，未改任何判据/阈值/参考/断言。

## 五、低项 / 观察

- **低 L1（记录面计数）**：证据 manifest 实际列 **6 条**（manifest 不自列），但 README §1/提交信息/卡面写「manifest **7/7** 自核、不自列」——7 是目录文件总数（含 manifest 自身），与「不自列」自相矛盾；应为 **6/6**。我重算 6 条全对（bad=0），仅措辞/计数需更正。
- **观察 O1**：exe 同 size 不同 sha 再次印证「注释改动也会改变 debug info 导致 exe 字节变」，与卡 D「同源重建不可复现」caveat 一致；后续 R06 窗口以冻结产物为准（卡已写明）。
- **观察 O2（正向）**：本卡把「文档面已改、测试面未改」的残余 follow-up 单独成卡闭环（而非塞回已 verified 的 K），流程正确；建议后续类似 follow-up 直接在原复核卡或新小卡登记，避免出现「backlog 计数」这类口径歧义。

## 六、口径

不宣称门禁/NP01/NP02 通过、不宣称上游等价/双平台/发布；不改判据/参考/容差/断言/行为；零 GPU、零窗、不重冻；历史证据不改写。
