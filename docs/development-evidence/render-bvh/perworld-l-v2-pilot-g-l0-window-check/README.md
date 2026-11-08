# G-L0 pilot 收口（attempt 3）独立复核（QA 只读）

任务：`2026-11-08-render-perworld-l-v2-pilot-g-l0-window-check`（owner QA）。日期：2026-11-08。
复核对象：卡 `2026-11-08-render-perworld-l-v2-pilot-g-l0-window` 收口提交 **`268380d8`** + addenda **`66bac4f6`**（复核时 HEAD `1e27ff59`，pilot 证据未被后续提交触及）。
依据：已签 spec v3 §6/§9.3/§10.2/§12 G-L0 与材料命令口径。数据：npz 保留在 `target/render-bvh/perworld-l-v2-gl0/pilot-out/nw3-light-hinge02040/`（不入库）。
边界：只读 + 只写本卡证据；零 GPU。

## 结论

**PASS（0 阻断、0 新观察）。** 收口数字、采后自核、refit provenance、事故记录与证据卫生均独立复核通过；结论仅作 **S-2 前置第 7 项**，不构成 G-L1 可签/可开的依据。

## 1. 数字复算（保留 npz + 修复后脚本）

用 `readout_c1_c2.py`（`4a054aba…`/16,711 B）独立重跑保留 npz → exit **0**、`metadata.failed=[]`：

| 项 | 我的复算 | 收口证据（window/readout.json / README §4 / receipt） | 判定 |
| --- | --- | --- | --- |
| C1 `cam-persp` rgb（0→1 / 1→2） | **38,285 / 40,644** | 38,285 / 40,644 | ✅ |
| C2 depth（0→1 / 1→2） | **35,828 / 33,883** | 35,828 / 33,883 | ✅ |
| C2 seg（0→1 / 1→2） | **16,041 / 16,754** | 16,041 / 16,754 | ✅ |
| verdict | **pass-margin**（all=true、any_below_accept=false） | pass-margin | ✅ |
| 诊断 cam-persp depth/seg | **0/0、0/0** | 0/0 | ✅ |
| 诊断 cam-hinge rgb | **50,363 / 52,329**（bitwise 54,380 / 54,250） | 50,363 / 52,329 | ✅ |

与 `window/readout.json` 逐字段比对 `match=True`。

## 2. 采后自核复算

- **两跑逐位一致**：`capture.json.two_captures_bitwise_identical=true`；npz 实际 sha256 与 `capture.json.files_sha256` 及 `window/files-sha256.txt` 逐文件一致（rgb `602574c2…`/5,531,026、depth `82eec6da…`/1,844,650、seg `c3f09b16…`/3,687,826、state `a1c5a321…`/2,486；capture.json `0a6579f4…`）；rgb/depth/seg 各 **6** 个 world×相机键 ✅。
- **legacy**：`legacy_reference_untouched=true`（`window/legacy-selfcheck.txt` 显式断言；采集日志 `legacy reference files=112`）✅。
- **元数据四项 + 双相机快照**：`metadata-selfcheck.json.failed=[]`；`nrender=2`、`cam_id_map=[0,1]`、`cam_res=[[320,240],[320,240]]`、`recipe.cameras` 2 条各 10 字段（含 sensorsize/intrinsic；cam-hinge `sensorsize=[0,0]` 走 fovy）✅。
- **seg 可见集合**：cam0 三 world 均 `{(-1,-1),(0,5)}`；cam1 world0 `{(-1,-1),(0,5),(1,5),(3,5),(4,5),(5,5)}`、world1/2 增 `(6,5)` —— 与 `window/seg-geom-sets.json` 一致 ✅。

## 3. refit provenance 与元数据

- `window/refit-counter.json`：`sequence=[refit,render,refit,render]`、`refit_calls=2`、`render_calls=2`、`pattern_ok=true`、`capture_exit_code=0` ✅。
- `capture.json.captured_at=2026-11-08T19:00:00+08:00`，与 `window/token-capture-at.txt` 逐字一致；`command.argv` 与材料命令一致（`--scene`/`--out-root target/...`/`--reference-tag nw3-light-hinge02040`/`--repeat 2`/`--use-fast-math false`/`--use-shadows true`/`--seed 20261030`）✅。
- `per_world_scope` 新口径（covers 含 light+geom；does_not_cover 无旧文案，含 AA/方案 K/连续动画/上游等价不覆盖）✅。

## 4. 事故记录如实性

- README §3 + `window/window-incidents.txt`：attempt1 重复执行者污染（锁 owner=backend-group，HEAD 漂移仅卡 JSON）、attempt2 外采 **18:41:38–18:41:55** 污染 fresh monitor（3 行 compute_filtered=1）、GO 采集因 out-dir 非空在 GPU 前失败（wrapper exit 4、refit/render=0）、Chief break stale 锁、外采整套作废；attempt1/2 记录在 `target/…/window/attempt1-2/`。
- 作废外采仅存 `target/…/window/foreign-184138/`（含 window-copies、readout、refit-counter），README/incidents 明确 **NOT evidence、不入库**；readout bug 根因（字符串逐字符 join）与修复（`a9fbac22` + QA `0abaeebb`）在案；attempt3 全部数值 provenance=attempt3；npz 未入库（提交清单无 npz）✅。

## 5. 证据与窗口纪律

- `manifest.txt` **36/36** 独立重算通过；首行 `#` + `# base dir:`；编码卫生（无 BOM/裸 CR/TAB）✅。
- `268380d8`+`66bac4f6` diff 面仅本卡证据目录 + 卡/runs；`src/`、`tests/`、`scripts/`、`fixtures/`、材料目录、spec 目录**零 diff** ✅。
- monitor.csv **60 行**、util 2–17、free_min **13,206** ≥ pilot_floor **10,225**（13,297−3,072）；三点基线三样本齐（13,302/13,302/13,297）；零编译指纹在案 ✅。

## 6. 非声明与门禁

- README §7：只声明「pilot 的 C1/C2 判别力结论 = pass-margin」；不声明 per-world 灯验收通过、渲染数值等价、上游等价、动画一致、对外发布；`pilot_floor` 与 S-2 signed floor 不混用；authoring 例外不适用 pilot；pilot 结论只作 S-2 前置第 7 项 ✅。
- 四门禁独立复跑：fmt/clippy/clippy-cuda 通过，**unit 1371 过 / 0 败** ✅。

## 7. 观察项

无阻断、无新观察。信息备注：作废外采目录在 `target/` 内无独立 marker 文件，但 README §3 与 incidents 已明确「NOT evidence/不入库」（target 本就不入库），durable 记录充分。

## 非宣称

- 本复核只判收口证据的真实性与自洽性，不重判 pilot 结论本身；不宣称 per-world 灯验收/上游等价/渲染数值等价/动画一致/发布依据。
- 不构成 G-L1 可签/可开；S-2 需用户双签、G-L1 需 Chief token；未改被复核证据与任何卡状态。
