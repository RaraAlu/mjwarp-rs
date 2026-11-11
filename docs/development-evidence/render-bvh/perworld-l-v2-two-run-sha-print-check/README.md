# QA 复核：G-L2 实体的两跑 sha 打印补齐（`acaa57ca`，卡 D）

- 任务：`2026-11-11-render-perworld-l-v2-two-run-sha-print-check`（owner QA）。复核对象：提交 **`acaa57ca`** + 卡 `2026-11-11-render-perworld-l-v2-two-run-sha-print`。零 GPU、只读复核（除为核身份做了 CPU 重链，见 §三）。
- 基线：`328070d7`（卡 D 登记/start）。

## 结论

**复核通过（0 阻断，0 低 + 3 观察）。** 改动确为打印补齐：**+3 行（1 注释 + 2 `println!`，0 删除/0 修改）**，断言与判据逐行未动；打印内容覆盖两跑 `[rgb,depth,seg]` 十六进制值 + `equal=`；B/隔离/C/A/pilot 的既有打印经归档 stdout 逐项核实；边界、身份 caveat、证据与门禁均成立。

## 一、改动面与打印正确性（✅）

- `tests/render_per_world_light.rs`：`git diff` 仅 3 个新增行，位置在两跑 sha 的 `assert_eq!` **之前**；`assert_eq!` 本体零改动（before/after 逐行对比一致）。物理行数 **494 → 497**（<500 棘轮）。
- 打印正确性：`tests/support/render_bvh/light.rs:371` 的 `sha256_frames` 返回 **`[String; 3]`**，元素为按 `"rgb"/"depth"/"seg"` 帧序列累积 sha256 的 `{:x}` 十六进制串；`{first_sha:?}`/`{second_sha:?}` 因此输出三个 hex 串；`equal=` 用原比较表达式 `first_sha == second_sha`，不改变断言语义。
- 若两跑不相等，打印发生在 panic 之前 ⇒ 失败时也能拿到具体值（这正是 backlog R1 的目的）。

## 二、打印覆盖审计（✅ 我核到归档原件）

- 权威样本 `perworld-l-v2-validate-window-g-l2/attempts/attempt4/window/run/exe-logs/render_per_world_light.stdout.log`：我实测 **1,837 B / `9f1325594fab6b0ef5d30934c1b0db57f1c88081e59ff3a411eb787a7991c5fd`**，与卡内声称一致；逐行读原件确认包含：
  - C 数值 + `verdict=PassMargin`、C diag（`campersp`/`camhinge`）、`capture.json C1 交叉核对 = Compared`；
  - **6 行** `A world<W> cam<C>: FrameStats {…}`（A 逐帧）；
  - `B: LightStats { worlds: 3, lights: 3, xpos_mismatched: 0, xdir_mismatched: 0 }`（`LightStats` 定义我核为 4 字段 `worlds/lights/xpos_mismatched/xdir_mismatched`）；
  - `isolation changed=[[false,false],[true,true],[false,false]]`；
  - pilot C 数值 + 旧 capture `Skipped(...)`；
  - **确实没有两跑 sha 具体值**（只有断言）——本卡补齐点准确。
- 另核 `tests/render_per_world_light.rs` 现存打印行：327/331/341（C）、378（A）、404（B）、**425/426（新增两跑 sha）**、449（隔离）、475/481（pilot）——与审计表一致。

## 三、测试 exe 身份（✅ 记录 + 我独立验证「不可复现」）

- 我实测：卡 D 后的 on-disk 实体测试 exe = **92,631,489 B / `55d0c32c…`**，与证据一致；我另对这同一源做过 **3 次强制重链**，得到 **3 个不同 sha256（`c5f3c990…`/`0c8d3c6b…`/`2ca7f097…`）、字节数全为 92,631,489 B** —— 独立验证了卡内「同源重建 exe 不保证逐字节可复现」的 caveat，支持其「身份以冻结产物为准」的口径。
- 因此：old→new 的**可核**信息是 ①源 diff（+3 行打印）、②字节数增量 **+671 B**（92,631,489 − 92,630,818，与两条 `println!` 的格式化/字符串常量量级相符）；「pre-card live `6908c5a2…`」是一次性重建测量、无法二次复核，卡已如实记录；attempt4 冻结值 `73245bbf…`/92,590,602 B 在 P10/z2vec 后已 drift，卡亦如实说明。
- guards 未重冻：`guards-perworld-l2-light.json` 仍 **`94478b3d…`/43,729 B**（我只读核对未变）；guards 仍记录旧 exe 身份 —— 卡已写明「后续渲染窗必须重冻」，正确。

## 四、边界 / 证据 / 门禁（✅）

- 提交 14 files = `tests/render_per_world_light.rs`（+3） + 证据目录 8 文件 + `.normify` 5 文件；`git diff 328070d7..acaa57ca` 无 `src/`/`fixtures/`/`scripts/`、无 `.cu` ⇒ 内核身份不变；不改判据/参考/容差/相机/recipe；零 GPU、未执行 `--ignored` 实体用例（卡已写明真实 sha/`equal=true` 待下一渲染窗观测）。
- 证据 `perworld-l-v2-two-run-sha-print/` manifest **7/7** 独立重算全对（bad=0/missing=0/extra=0、不自列）；README/receipt/logs（diff-stat/prints-audit/identity-measurements/guards-untouched/line-counts）齐。
- 编码：提交 blob 实测 **UTF-8 / LF（25,384 B、CR=0）**、无 BOM。注：仓库 `core.autocrlf=true`，该文件 worktree 检出为 CRLF（`git ls-files --eol` = `i/lf w/crlf`）——**提交字节是 LF**，工作区形态差异不是本卡问题（见观察 O3）。
- 门禁 `.normify/runs/…-two-run-sha-print/`：fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**。

## 五、观察（不阻断）

- **O1（身份测量 caveat，已被我独立验证）**：同源重建不可逐字节复现（我 3 次重链 3 个 sha、同 size）⇒ 任何「old→new sha」只能作为测量时点记录，不能当可复现指纹；冻结/复核窗口应以**冻结产物本身**的哈希为准。卡内已如此记录。
- **O2（target 瞬态）**：我复核期间的重链使 `target/debug/deps/render_per_world_light-*.exe` 的 on-disk sha 变为 `c5f3c990…` 等（同 size）；这是 target 目录瞬态，不影响提交与证据（证据里的 `55d0c32c…` 是卡 D 的测量时点值）。
- **O3（方法学提醒）**：本仓库 `core.autocrlf=true`，tracked 文件 worktree 常为 CRLF 而 blob 为 LF；后续编码/卫生核对应核 **提交 blob 或 `git ls-files --eol`**，不能只看工作区字节（本卡提交 blob 已核为 LF）。

## 六、口径

不宣称设备端/数值变化（打印-only，且实体用例未执行）；不改判据/参考/容差；不重冻、不占窗；零 GPU。
