# R02-S3：天空盒 validation 窗 attempt=2（validation-only 例外 util<60）

任务：`2026-10-29-render-bvh-r02-skybox-s3-validate`（P8-渲染与BVH，owner Backend，Arch 登记）
本片性质：**窗口执行与证据收口**。用户已签 `validation-only 例外 util<60`；本片记录 attempt=2 的窗口事实、
天空盒位级判据、8 回归零漂移、冻结身份、例外留痕，以及 attempt=1（INVALID_ROUND）与两条修复链的历史。

> **不是常规门槛通过**：常规 `util<40` 在 12:00 时段失败（354 样本 / longest_clean=18 / 75 个 ≥40 / exit 20），
> attempt=2 用的是用户签署的 `mode=validation_only_exception_util_lt_60`（attempt2 monitor 60/60、12 个 ≥40 样本
> clean、streak 60）。

## 1. 窗口六项（attempt=2）

| 项 | 值 |
| --- | --- |
| 实体 | **9/9 exit 0**（body_wall 48.7 s；逐实体 wall 见 `attempt2/entity-results.csv`） |
| sha_guards | **77/77**（9 exe + 68 只读输入），mismatches=0 |
| 窗后 exe 复查 | **9/9**，mismatches=0 |
| 零 Rust 编译 | **True**（deps 1839→1839，max write `10/29/2026 18:12:48`，早于窗口开始 18:22:08） |
| 采样/阈值 | samples=166、util 18–40、free_min **12 587 MiB**、baseline_free 12 858、**floor 9 786 未破**、used 增量 271 ≤3 072、`residual_processes=0` |
| 收尾标记 | `PRECHECK_OK open=3204/12858/37…40` → `GUARDS_OK=77/77` → **`WINDOW_RUN_DONE`** |

窗口：`2026-10-29T18:22:08.9+08:00` → `18:23:00.8+08:00`；attempt=2；`attempt_valid_rounds=2`。

## 2. 天空盒判据（`render_skybox_reference`，两相机 ×120 000 像素，全位级）

| 相机 | RGB | depth | seg | depth_zero |
| --- | --- | --- | --- | --- |
| cam0（76 800） | compared=76 800 / **mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0** | **mismatch=0 / max_ulp=0 / beyond_ulp=0** | **mismatch=0** | 16 995 |
| cam1（43 200） | compared=43 200 / **mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0** | **mismatch=0 / max_ulp=0 / beyond_ulp=0** | **mismatch=0** | 27 498 |

两跑 sha 相同（日志打印）：rgb `92ac188d1cab24258e777461f44087d747e711f70b4943d0c9f48a727a58b3cc`、
depth `de732643aa3e9afb2a477b259dd1128139929c29ba7d8935d23096d767e42445`、
seg `065caf08c1759c34be2b9d9e2b2f8d62e874f219d8d6bf93c8d9e2408b293821`。
判据沿用既有位级口径（depth 位级优先 + `max_ulp<=4`/`beyond_ulp==0`），**无新增容差、无放宽**。

## 3. 8 回归实体零漂移

`render_camera_rays`(2)、`render_context_lifecycle`(5)、`render_e2e`(5)、`render_kernel_depth_seg`(6)、
`render_kernel_flags`(6)、`render_kernel_rgb`(3)、`render_r08_cameras`(6)、`render_r10_multi_world`(2)
—— 全部 `test result: ok`、exit 0；逐实体日志在 `attempt2/logs/`。

## 4. 冻结身份（attempt=2）与 superseded

* suite `r02-skybox-validate`、**attempt=2**、**77 条目 = 9 exe + 68 只读输入**；
  guards sha256 **`2d55c0b3fc14a33b2acce159bf68f474131f2819e3b2230f17ece11021b2e6a3`** / **28 931 B**；
  aggregate **`ca1fcaa52c0dfda4e7ae473eb195d7b53c1f57dc3ae420526b1a0644b6aa6550`**；
  `-GuardsOnly` → **`GUARDS_OK=77/77`**（stash 9/9）。
* attempt=1 guards `b5a2d4d7…`/28 931 B 归档 `target/render-bvh/r02-skybox-window/guards-r02-skybox-attempt1-superseded.json`，标 **superseded**。
* R05 `scripts/validate/render-bvh/guards.json`（`e2444a96…`/27 670 B/suite `r05-s3b-hfield-validate`/attempt=2）**未动**，按历史记录保持。

## 5. 例外 amendment 留痕（attempt=2）

* attempt=2 例外 monitor（**重跑，不沿用 attempt1**）：
  `monitor-pre-window-attempt2.csv`，`-Required 60 -MaxMinutes 30 -MaxUtil 60` → **60 samples / 60 clean / longest 60**、
  util 32–41、**12 个 util≥40 样本全部 clean=True 且 streak 连续到 60**、ge60=0、free_min 12 751、filtered=0、build=0、exit 0。
  **生效门槛=60 的主证据就是这 12 个 ≥40 clean 样本 + streak**（precheck 只作记录，不作主证）。
* 常规失败件原样保留：`monitor-pre-window.csv`（354 samples、longest_clean=18、util 2–43、ge40=75、exit 20）
  + `monitor-pre-window-result.txt`（`decision=STOP_NORMAL_THRESHOLD_NOT_MET`、`window_commands_executed=0`）。
* `attempt2/exception-block.txt` 与 `attempt2/facts.txt` 例外块字段：
  `mode=validation_only_exception_util_lt_60`、`approved_by=user (via Chief) 2026-10-29T15:3x`、
  `not_a_normal_threshold_pass=true`、`authoring_exception_used=false`、`signature_unchanged=true`、`open_util_limit=60`。
* `attempt2/window-command.txt` 逐字：`… -GuardsFile scripts\validate\render-bvh\guards-r02-skybox.json -OpenUtilLimit 60`。

## 6. attempt=1（INVALID_ROUND）与两条修复链（历史，非法轮）

* attempt=1：`INVALID_ROUND exit10`，4 实体失败（`render_skybox_reference`、`render_e2e`、`render_kernel_flags`、
  `render_kernel_rgb`）；其余 5 实体 exit 0；守卫面/零编译/floor 当时均正常。
  留档：`attempt1/{invalid-round-note.txt,facts.txt,entity-results.csv,guard.log,window-command.txt,exception-block.txt}`。
* **失败 A（3 个 stale 负控）**：旧断言「`render_skybox` 必须写前拒绝」与 R02-S1 解锁冲突（S1 scope 缺口）。
  修复链：`f2ee32d0`（测试：e2e 500→**496** 净 −4、flags 495、rgb 309；新行为断言 + 保留 dynamic-ray/splat 负控）
  → `1d3c7159`（证据）→ `df24283c`（收口，unit 1104/0/390）。
* **失败 B（天空盒 depth 1–4 ULP）**：参考采集用了 `use_fast_math=true`，而产品内核配方 `WARP_1_15_RENDER`
  恒为 `fast_math=false`。修复链：capture 脚本 `812a2f77…`（S2 原交付）→ `d17d2418…`（窗口期确定性自检修复）
  → **`d6f9864c…`**（`use_fast_math True→False`，其余 27 参数未改，对照见 S2 证据 `materials/params28-audit.txt`）；
  提交 `b3a47aca`。重采后 **depth 位级 0**（attempt1 为 max_ulp 2/4），证明根因结论成立。
* stage A attempt=1 失败（脚本自检 bug）留档：`attempt1/stage-a-attempt1-failure.txt` + `attempt1/capture-skybox-fix.diff`；
  旧参考 4 文件归档 `target/render-bvh/r02-skybox-window/stage-a-attempt1-reference-superseded/`
  （capture sha `e81e098d…`；rgb `b49053a1…`、depth `0fa55821…`、seg `2153fa72…`）。
* attempt=2 参考（本次生效）：`capture.json` sha `47f704de…`、`use_fast_math=false`、params28 sha
  `e2bae199…`、两跑 sha 相等、恰 4 文件；rgb `cf65a570…`、depth `28c1ea4b…`、seg `2153fa72…`。

## 7. capture 脚本 sha 链与 params28 对照

| 阶段 | sha256 | bytes / LF | 原因 |
| --- | --- | --- | --- |
| S2 原交付 | `812a2f778c21bb5cb8b24a96f6dce052be7f1f0d923174d0dcb75442e5d0c7a8` | 16 084 / 425 | 初版 |
| 窗口期修复 1 | `d17d24183627bcd843f21e604fb6a411d4a4682b0f921105023bed0211fea15c` | 16 494 / 428 | 确定性自检改为同 writer 两跑文件 sha 比较（attempt1 stage A 失败根因） |
| 窗口期修复 2 | `d6f9864cc29fb452e1914f135fe10d55c65751a89ad9a84244cfa362fabcc16d` | 16 495 / 428 | `use_fast_math` 对齐产品 recipe（attempt1 depth ULP 根因） |

28 项参数逐项对照产品 `resolve_context` 解析值见 S2 证据 `materials/params28-audit.txt`：唯一改动项为
`use_fast_math`，其余 27 项一致。

## 8. 门禁与证据

fmt 0 / clippy 0 / clippy-cuda 0 / unit 0 / `validate --fingerprints` 错误 0（本片不改 Rust 代码；窗口内
`zero_rust_compile=True` 由 facts 证明）。证据 = 本目录（README + receipt + manifest + 33 个原始产物/留档文件），
manifest 按 committed blob 且不自引用、LF 无 BOM。

## 9. 非声明与限制

* **不是常规门槛通过**：例外是用户签署的 `validation_only_exception_util_lt_60`；常规失败记录原样保留。
* attempt=1 为 **INVALID_ROUND（非法轮）**，不作为有效窗口结论。
* 本窗判据为既有位级口径，未改判据/容差/参考（参考仅配方对齐 `fast_math=false`，见 §6）。
* 不写「精度达标/上游等价/产品渲染验收通过」；结论以逐相机 0 差异与两跑 sha 相同表述。
