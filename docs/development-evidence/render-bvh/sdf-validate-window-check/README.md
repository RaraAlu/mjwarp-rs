# U4 验证窗 attempt1/attempt2 + U1b 回滚 — 独立复核

任务：`2026-10-31-render-sdf-validate-window-check`（QA；只读复核）
被核：U4 验证窗 `2026-10-31-render-sdf-validate`（attempt1/attempt2）、U1b 回滚 `2026-10-31-render-sdf-host-unlock-rollback`、
attempt 产物入库 `2026-10-31-render-sdf-validate-attempts-evidence`
pin：`07252cd2`（回滚）+ 证据提交 `d52dc774` / `0192016d`
start 时 HEAD（`revision.before`）：`d3bca3615005238a5b7ca72e0fa0a322b49f0a2c`
复核时间：2026-10-31 02:49–02:56（+08:00），空闲段；**零 GPU、未重跑窗口、未重冻结、未改任何被核文件**

## 1. 结论

**pass / 阻断 0 / 需修 4 / 观察 6**

核心判断全部成立：
- 两次 attempt 都严格按条件式协议执行：**先常规 `util<40` monitor，失败（exit 20）→ 原样保留 → 启用已签 validation-only `util<60` 例外 → 60/60**；
  没有哪天写成「常规通过」，`ge60=2/1` 照实记录、不写 0。
- 两次冻结身份各自独立、可复核：`0639f0ff…` / `d921caae…`，各 **33,651 B**、`9 exe + 83 inputs = 92`（10/10/47/16）；
  aggregate 我用 **Windows PowerShell 5.1** 从 committed guards 独立重算，与记录全等；attempt2 是全新冻结（83 inputs 全同、8/9 exe 同、仅 `render_sdf_reference` 变）。
- body 一致失败（红线复现）：两 attempt 均 8/9 exit 0，`render_sdf_reference` exit 101；cam0 215/76,800 `max_ulp=191`、cam1 4/43,200 `max_ulp=25`、
  首差 (230,69) 参考 seg=(2,5)、rgb/seg 全 0；判据零放宽（`MAX_DEPTH_ULP=4`）、两相机都查、未跳球、未缩相机。
- 回滚正确：`07252cd2` 只改 `megakernel.rs`，与 `af96c048^` **零差异**（U1b 的精确逆），cuda 零 diff、内核身份仍 `778216b2…`/`34f91147…`/268,589；
  unit 1111/0/397、`--no-run` EXIT=0。
- attempt 产物入库完整：13 文件、manifest **12/12** 逐条重算与 committed blob 一致、10 个产物与 `target/` 原件**逐字节相同**；
  guards JSON 以 **CRLF 33,651 B** 原样入库（`.gitattributes` 的 `-text -diff` 生效）。
- 上一轮我提的方向措辞已修正（双向：`product>ref` 16 / `product<ref` 24、ULP 7–71、全量 max|ULP|=191），guards-only 来源已如实标注。

需修 4 条都是**证据文字/数字**级（不影响回滚判定，也不必重跑窗口）：
- **P1** `attempts/attempt2/invalid-round-record.txt` 的 `failed_tests` 抄了 attempt1 的 3 个，实际 attempt2 是 **3 passed / 2 failed**
  （`sdf_mesh`、`sdf_occluder`）；shadow 在 `82dfa0c3` 的 u8 修后已 green（attempt1 才是 3 failed，shadow 因 `sdf_compare.rs:138` 的 u8 溢出 red）。
- **P2** 窗口 README §3 attempt1 常规 monitor 的 util 范围写 `5–50`，committed CSV 实测 **3–76**。
- **P3** 窗口 README §3 attempt1 例外 monitor 写「83 样本」，实测 **69**（83 是 attempt2 的）。
- **P4** committed attempt 证据只有散文「已签条件式 validation-only 例外」+ `exception_load_bearing`，**缺五要素明文块**
  （`approved_by=user via Chief`、`mode=validation_only_exception_util_lt_60`、`original_attempts_retained`、`not_a_normal_threshold_pass=true`、
  `authoring_exception_used=false`）；建议补在窗口 README §3 或两份 invalid-round record（纯文档）。

## 2. 条件式协议（逐项自算）

| 项 | attempt1 | attempt2 |
| --- | --- | --- |
| 常规 monitor | 354 样本、longest 28/60、util **3–76**、free_min 11,545、exit 20、CSV 原样保留 | 354、longest 41/60、util 1–53、free_min 11,467、exit 20、原样保留 |
| 例外 monitor | **69** 样本、60 连清、段内 util 峰值 43、ge40_clean=9、`ge60=2`（62@#6、61@#9，段外）、exit 0 | **83** 样本、60 连清、段内峰值 45、ge40_clean=8、`ge60=1`（68@#23，段外）、exit 0 |
| 结论 | 例外 load-bearing=true（段内 43>40 且 clean，证明门槛是 60） | 同（45>40） |

`ge60` 样本均 `streak=0` 且在 60 连清段开始之前；连清段内无 ≥60，照实记录，不作通过条件。

## 3. 冻结身份与入库

- attempt1 `0639f0ff…`/33,651 B/aggregate `02b20213…`；attempt2 `d921caae…`/33,651 B/aggregate `a50e99c9…`（5.1 重算全等）。
- 计数 `9 exe + 83 inputs = 92`，分解 10/10/47/16；92 条 sha-guards 与 guards 一致；attempt2 stash 9/9 一致。
- 产物入库：manifest 12/12、committed blob 与 target 原件 10/10 逐字节相同、guards CRLF 保留（`git check-attr` = unset/unset）。
- 提交走独立任务 `…attempts-evidence`（窗口卡 in_progress、files 列表冻结），receipt 已说明原因。
- 本复核卡不引用门禁（`normify gate` 报「任务没有引用任何门禁」）；本卡检查项为 receipt `/task_id` 与 forbid_claim，均通过。

## 4. body 与判据

- attempt1：3 failed（mesh/occluder/**shadow u8 溢出**）、8/9 exit 0。
- attempt2：**3 passed / 2 failed**（mesh、occluder；shadow、DepthSeg 跨路径、两跑确定性绿）、8/9 exit 0。
- 深度偏差两轮同量级（见上），诊断双向；`MAX_DEPTH_ULP=4` 未动；`82dfa0c3` 仅测试（+2 / +73−1，u8→i32 + 只读 `println!` 诊断，返回 `()`）。
- 窗口纪律：deps 快照在 body 前（`deps_before_max_write` < `window_start`）、deps 计数前后同为 1840 ⇒ **body 开始后零 Rust 编译**；
  `post_window_exe_recheck=9/9`、`residual_processes=0`、floor 未破。

## 5. 回滚

- `megakernel.rs` 406/20,172/`9b013d90…` → **440/21,813/`fb9aae8f…`**；`git diff af96c048^..07252cd2 -- src/` **为空**（精确逆）。
- 恢复 `reject_sdf_geometry`(L55)、调用点(L121)、`mod sdf_geometry_guard_tests`(L422，2 条 host-only 单测)；host 重新 `Unsupported`。
- U1 保留：cuda 零 diff、内核身份 `778216b2…`/`34f91147…`/268,589。
- `07252cd2..d3bca361` 无 committed `src/` 改动；unit 1111/0/397（1109 + 2 条恢复单测；卡面 1108 已记为笔误）。

## 6. 观察（记录级）

- O1：9 实体日志/facts/samples/sha-guards 仅存 `target/`，committed 只有关键产物；建议 attempt3 证据把 `render_sdf_reference.log` 一并入库。
- O2：`-text -diff` 规则让 README/receipt 也变 binary（`diff: unset`），换取 guards 逐字节，记录级接受。
- O3：现盘 `target/debug/deps` 的 exe 已因回滚重建，与两次冻结都不同；冻结身份靠 stash/归档 guards 哈希，attempt2 stash 9/9。
- O4：复核时工作树有 Arch 球面 fix 的在途改动（`src/render/kernel/cuda/render_math.cu`；其后以 `8126868c` 提交，属独立 fix 卡），
  故 rollback/attempts-evidence 的 `normify check` 会在 Unchanged 检查报该路径（文件在那些 pin 之后变了）；
  我按提交范围核：`07252cd2..d3bca361` 零 `src/` 改动，该 fix 片不在本卡范围。
- O5：`ge60` 在段外、段内峰值 43/45，与「例外门槛 60 生效」一致。
- O6：窗口卡仍 `in_progress`、未宣称通过；同名 abandon 卡无 QA 工作丢失。

## 7. 不宣称

不宣称渲染数值验收通过 / 上游等价 / 精度达标；U4 **未通过**（两次 INVALID_ROUND、已回滚 U1b）；不宣称 sphere 轮廓 depth 的 RCA 结论（属 fix 片）。

## 8. 复现命令（只读）

```
git diff --stat af96c048^..07252cd2 -- src/            # 空 = 精确逆
git diff --stat 07252cd2..d3bca361 -- src/             # 空 = 回滚后无 committed src 改动
git cat-file -s HEAD:docs/development-evidence/render-bvh/sdf-validate-window/attempts/attempt1/guards-sdf-validate.json   # 33651
git check-attr text diff -- docs/development-evidence/render-bvh/sdf-validate-window/README.md                             # unset/unset
git cat-file blob HEAD:<guards.json> > tmp.json  # 再用 Windows PowerShell 5.1 重算 aggregate 与 manifest
```

`materials/selfcheck-*.txt` 是本次重算的原始输出（monitor / freeze / body / rollback / artifacts / corrections）。
