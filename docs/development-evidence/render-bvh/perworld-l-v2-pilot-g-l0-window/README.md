# G-L0 pilot 执行窗（perworld L v2 灯）：双相机 C1/C2 判别力 pilot（attempt 3，单次、authoring、产物不入库）

- 任务：`2026-11-08-render-perworld-l-v2-pilot-g-l0-window`（owner Backend、p78、P8-渲染与BVH）
- 执行者：**Backend 直接会话（唯一写者）**；窗口内 repo-write-lock `owner=Backend`（pid 84408）。
- 依据：已签 spec v3 `bb064fe0…`/39,379 B §12 G-L0 + 材料卡 `2026-11-08-render-perworld-l-v2-pilot-g-l0-material`
  （QA 读前预审 PASS `95c70bf0`；readout scope fix `a9fbac22` + QA 最小复核 `0abaeebb`）。
- **结论：G-L0 pilot PASS —— `pass-margin`**（C1/C2 两套两对均 ≥576；`any_below_accept=false`；
  `metadata_selfcheck.failed=[]`）。该结论作为 **S-2 采集窗前置第 7 项**。

## 1. 窗口结论（attempt 3）

| 项 | 值 |
| --- | --- |
| 窗口时间 | 2026-11-08 **18:58:55 → 19:05**（+08:00） |
| monitor | exit 0、samples **60**、longest_clean **60**、**整轮全干净**（unclean=0、`compute_filtered_max=0`、`build_max=0`）、util **2–17%**（<40）、free_min **13,206 MiB** |
| 三点基线 | `2,760/13,302/2%/26` → `2,760/13,302/2%/26` → `2,765/13,297/2%/26`（19:04:20/25/30） |
| baseline_free / pilot_floor | **13,297 MiB** / **13,297 − 3,072 = 10,225 MiB** |
| `MJWARP_CAPTURE_AT` | `2026-11-08T19:00:00+08:00`（token 逐字；语义=授权窗口时间戳，非采集瞬间） |
| 采集 | wrapper 单次、exit **0**；refit 序列 `refit→render→refit→render`（refit×2/render×2，`pattern_ok=true`） |
| 判读 | readout exit **0**、verdict **`pass-margin`** |
| 两跑逐位一致 | **true**（capture.json；逐文件 sha 一致） |
| legacy_reference_untouched | **true** |
| 产物 | 仅 `target/render-bvh/perworld-l-v2-gl0/pilot-out/`；**npz 不入库**（保留供 QA/Chief 复读） |

## 2. T0 冻结与零编译

- HEAD **`0abaeebb`**、`git status --porcelain` 空、无 `cargo/rustc/nvcc/cl` 进程；锁 `owner=Backend`。
- 指纹：scene `4b354054…`/5,134 B；`capture_render_perworld.py` `55b91f23…`/23,483 B；
  `perworld_capture_meta.py` `296c2b12…`/12,508 B；`perworld_light_prep.py` `9fd5f8c2…`/7,206 B；
  **readout `readout_c1_c2.py` `4a054aba…`/16,711 B**；材料 README `8dbb9fcd…`/16,832 B、receipt `f48766ba…`/14,202 B；
  upstream pinned `71da24d9…` clean；venv `3.12.13 / 1.15.0 / 3.15.0 / 3.15.0 / 2.5.4`（`window/t0-freeze.txt`）。
- **窗口零编译**：全程未调用 cargo/nvcc；仅 pinned authoring python + warp JIT cache 命中
  （`window/zero-compile-fingerprint.txt`）。

## 3. 窗口事故与裁决（attempt 1/2 作废；attempt 3 有效）

1. **重复执行者污染**：开发团队群的 Backend 会话（锁 `owner=backend-group`）在窗口期间自行执行同一 pilot：
   - attempt 1（18:29–18:34）期间它注册窗卡（`8a2be34f`）、取锁、并在 18:34:29 留下 step-2 dry-run；
     HEAD 由 `8bc589ac` 漂移到 `7ffd4a82`（**仅窗卡 JSON 提交，`src/scripts/fixtures` 零 diff**）。
   - attempt 2 clean restart（18:38–18:47）期间它在 **18:41:38–18:41:55 实际跑了一次采集**，污染 fresh monitor
     的 18:41:43/48/53 三条样本（`compute_filtered=1`）；我的 GO 采集 18:48:04 因 out-dir 非空在 GPU 前失败
     （wrapper exit 4、refit/render=0）。Chief 裁决**窗口中止、外采整套作废**。
2. **stale 锁**：Chief 按纪律 break（`previous_owner=backend-group` pid 65080）。
3. **作废外采**：仅作异常证据保留在 `target/…/window/foreign-184138/`（含外采 pilot-out 5 件、readout、
   foreign dryrun、window 副本），**不是本卡证据、不入库**；attempt 1/2 我的记录在
   `target/…/window/attempt1-2/`（不入库）。
4. **readout 脚本 bug**：作废 readout 的 `metadata_selfcheck` 失败根因是脚本把字符串逐字符 join
   （scope fix `a9fbac22` + QA 复核 `0abaeebb`），**不是采集问题**；attempt 3 直接用修复后脚本（`4a054aba…`）。
5. **attempt 3**：T0 冻结 → step-2（本会话）→ **strict fresh monitor（60/60 全干净）** → 三点基线 → GO →
   单次 wrapper 采集 → 修复后 readout → 采后自核。**本卡全部数值的 provenance 是 attempt 3**。
   说明：attempt-3 的 npz 与作废外采**逐位相同**属上游确定性预期（同输入/同脚本/`--repeat 2`），
   不构成对外采数据的采信。

## 4. 判读 C1/C2（spec §6 写死；修复后 readout）

| 集合 | 相机/通道 | 0→1（world） | 1→2（world） | 门槛 |
| --- | --- | --- | --- | --- |
| **C1** | `cam-persp` rgb `max_channel|Δrgb| ≥ 1/255` | **38,285 px** | **40,644 px** | 每对 ≥576 pass-margin / 384 验收 |
| **C2 depth** | `cam-hinge` depth f32 位型变化 | **35,828 px** | **33,883 px** | 同上 |
| **C2 seg** | `cam-hinge` seg `(id,type)` 变化 | **16,041 px** | **16,754 px** | 同上 |

- `verdict=pass-margin`、`all_sets_pass_margin=true`、`any_below_accept=false`；两套不得互换相机归属。
- 诊断项（不设门槛、只记录）：`cam-persp` depth/seg 变化 **0/0**（两对，几何事实）；`cam-hinge` rgb 变化
  **50,363 / 52,329**（bitwise 54,380 / 54,250，交叉检查、不冒充 C1）。

## 5. 采后自核

- **两跑逐位一致**：`two_captures_bitwise_identical=true`；逐文件 sha（rgb `602574c2…`、depth `82eec6da…`、
  seg `c3f09b16…`、state `a1c5a321…`，均与 capture.json 记录一致，`window/files-sha256.txt`）。
- **legacy_reference_untouched=true**（显式断言，`window/legacy-selfcheck.txt`；采集日志 `legacy reference files=112`）。
- **元数据四项**：`command`（argv/cwd/rendered 非空）、`captured_at=2026-11-08T19:00:00+08:00`（ISO8601）、
  `reference_tag=nw3-light-hinge02040`、`per_world_scope` 新口径（light+geom，旧文案不出现）；
  `metadata_selfcheck.failed=[]`（`window/metadata-selfcheck.json`）。
- **双相机快照 10 字段**：`context.resolved={nrender:2, cam_id_map:[0,1], cam_res:[[320,240],[320,240]]}`；
  `recipe.cameras` 2 条各 10 字段（含 `sensorsize`/`intrinsic`；cam-hinge `sensorsize=[0,0]` 走 fovy 路径且不声称 intrinsics）。
- **seg 可见 `(id,type)` 集合**（`window/seg-geom-sets.json`）：cam-persp 三 world 均 `{(-1,-1),(0,5)}`；
  cam-hinge world0 `{(-1,-1),(0,5),(1,5),(3,5),(4,5),(5,5)}`，world1/2 增加 `(6,5)`。
- **per-world 摘要**：见 `window/readout.json.per_world_summary`（rgb mean/min/max、depth hit/min/max、seg_pairs）。

## 6. 复读与产物

- npz/capture.json 在 `target/render-bvh/perworld-l-v2-gl0/pilot-out/nw3-light-hinge02040/`（**不入库**）；
  QA/Chief 复读后由 Chief 决定删除。计数字段与 sha 已归档在 `window/files-sha256.txt`、`window/capture.json`。

## 7. 边界与非声明

- 不写 `fixtures/`（正式参考属 G-L1）、不改 `src/`、`tests/`、`scripts/`、材料与 spec 目录；单次、失败即停、不重试、不放宽。
- 只声明「pilot 的 C1/C2 判别力结论 = pass-margin」；**不声明** per-world 灯验收通过、不声明渲染数值等价、
  不声明与上游实现等价、不声明动画一致性、不作对外发布依据；`pilot_floor` 与用户签的 S-2 signed floor 不同、不混用。

## 8. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` / `receipt.json` / `manifest.txt` | 本文件 / 机器可读回执 / sha256+bytes 清单 |
| `window/preconditions.txt` | 前置卡片/提交/指纹锚点 |
| `window/first-minute.txt` | 材料 first-minute 逐字副本（含 source sha） |
| `window/token-capture-at.txt` | `MJWARP_CAPTURE_AT` 逐字值 + N1 三态校验 |
| `window/monitor.csv` / `monitor-summary.json` / `monitor-exit-code.txt` | 严格 monitor 原始 CSV 与摘要 |
| `window/baseline-three-points.txt` / `baseline.txt` | 三点基线、floor 算式、compute-apps |
| `window/dryrun.json` / `step2-check.json` / `dryrun.stderr.log` | step-2 本会话重跑与预期事实 |
| `window/capture-stdout.log` / `capture-stderr.log` / `capture-exit-code.txt` | 单次采集日志与 exit |
| `window/capture.json` | 采集 provenance（command/captured_at/scope/context/recipe.cameras/files sha） |
| `window/refit-counter.json` | wrapper refit 执行序列证据 |
| `window/readout.json` / `readout-stdout.log` / `readout-exit-code.txt` | C1/C2 判读、per-world 摘要、verdict |
| `window/files-sha256.txt` | 逐文件 sha 与 capture.json 对照（npz 不入库） |
| `window/metadata-selfcheck.json` / `seg-geom-sets.json` | 元数据四项+双相机自核 / seg 可见集合 |
| `window/environment-provenance.txt` | pinned venv/warp/CUDA/GPU 与 upstream |
| `window/zero-compile-fingerprint.txt` | 零编译与 HEAD/树状态指纹 |
| `window/git-before.txt` / `git-after.txt` | 窗口前后 git/锁/构建进程快照 |
| `window/legacy-selfcheck.txt` | legacy 未改动显式断言 |
| `window/post-gpu.txt` / `post-capture.json` / `capture-pre.txt` / `t0-freeze.txt` | 采后 GPU/树状态、自核 JSON、GO 前快照、T0 冻结 |
| `window/window-incidents.txt` | attempt 1/2 事故与裁决摘要（作废物仅存 target，不入库） |
