# R05-S3a hfield 参考采集独立复核（窗口留档 / 例外记录 / 旧参考守卫 / 证据）

任务：`2026-10-28-render-bvh-r05-s3a-hfield-capture-check`（P8 渲染与 BVH；p57；owner QA）
被复核：`8741e8e4`（采集产物）+ `aab27d9b`（窗口证据）+ `926b6488`（收口）
派工消息 HEAD：`a6bcca80`；**复核期间 HEAD 漂移到 `4bb9cd15`**（S3b 验证窗 authoring 两提交，见观察 O-4；我核过未触动被核路径）
范围：只读复核 + 本证据目录；**不开 GPU、不重跑采集、不做渲染数值结论**。

## 1. 结论

**verdict：pass / 阻断 0 / 观察 4**（均为建议/记录级，不阻断）。

**例外不当成常规通过**这条重点成立：README + receipt + `logs/window-facts.txt` 三处均记录 attempt1/2/3 的常规 `util<40` 失败（STOP，未开窗），attempt4 明确标为 **Chief 预批的 authoring-only 例外 `util_lt_60`**，并写明「**本窗不是常规门槛通过、原 `util<40` 记录保留**」；我另按监测 CSV 独立重算了四轮样本/最长清洁段/util 区间/free_min，与三处文档一致。

| 复核点 | 结果 |
| --- | --- |
| 签字物未变 | 脚本 `cb1808f8…`/20 314 B/425 行、`scene.xml` `b3b9a6b4…`/3 590/63、`scene-mixed.xml` `bf9bb966…`/3 180/60 —— 我按 committed blob 重算，全对 |
| 8 个捕获文件 | sha+bytes 逐项与 manifest 一致，**合计 5 784 264 B**；三个 npz 的 sha 与各自 `capture.json.files_sha256` 一致；npz 结构（zip/npy 头）cam0=(240,320,3)、cam1=(180,240,3) |
| capture.json | 两组 `group`/`use_shadows=true`/`use_fast_math=false`（各 7 处 false、0 处 true）/`two_captures_bitwise_identical=true`/`legacy_reference_untouched=true`/`legacy_reference_files=38`/`captured_at` 两组同值 `2026-10-29T00:00:00+08:00`；`recipe.scene_xml`（组1 `b3b9a6b4…`/3590、组2 `bf9bb966…`/3180）、`recipe.script.sha256=cb1808f8…`；相机 `[[320,240],[240,180]]` |
| 例外留档 | 三处记录齐全（见 §2）；attempt3 实测 354 样本/最长 39/util 2–49/free_min 13 109/STOP；attempt4 例外含批准人+时间+理由+监测脚本 `689fd7eb…`/2 529 B/非 ASCII 0/ANSI=LF=34 行（我重算一致） |
| 旧参考零读写 | 38 文件 / 25 320 515 B / 聚合 `b5cb50af…`（我按上游口径用 PowerShell 复现，另得 `983617f1…` = R09 行拼接口径）；逐文件 sha 与 `materials/legacy-baseline.txt` 一致、`git diff` 自 authoring 起为空；`scene-shadow` 17 文件/11 566 120 B（16 产物/11 561 363 B）与窗口前快照 17 行逐文件 0 差异 |
| 窗口事实 | 两条命令 exit 0、wall 9 952 + 4 764 = 14 716 ms；开窗瞬间 gate 3 物 match、free_min 13 268、floor=10 196、filtered=0、build=0；GPU 已释放（residual_render_procs=0）、窗后构建进程 0、证据日志与 `target/` 原始日志逐字节相同 |
| 只采集不比数 | 全证据扫描无 rgb/depth/seg 差异数字或「通过/达标/等价」类结论（详见 §5） |
| 证据清单 | manifest 28 行 = 8 捕获（仓库相对）+ 20 证据（本目录相对）；两种路径基准在 manifest 头与 README §18 写明；不含自身；全部对 committed blob 相符；无 BOM、禁词 0 |

## 2. 例外记录（重点项的核对结果）

- `logs/window-facts.txt`：attempt1/2/3 各有 `decision=STOP: normal threshold util<40 not met ...window NOT opened, no capture command executed`（共 3 处）；attempt4 块含 `authoring_only_exception=util_lt_60`、`authoring_only_exception_approved_by=Chief`、`..._reason=load-attribution: …`、`..._note=… this window is NOT reported as a normal-threshold pass`、`the original util<40 threshold record is retained`，以及监测脚本 sha/bytes/ASCII 自检行。
- `README.md`：§0/§13/§17 同步写明「attempt1–3 常规门槛均未达标并停手；attempt4 为 Chief 预批 authoring-only 例外；**attempt4 不记为常规门槛通过，原始 `util<40` 失败记录保留**」；§17 再写「只采集不比数」。
- `receipt.json`：`pre_window_monitor.attempts[1..3].decision` 均为 STOP；`authoring_only_exception` 含 `name=util_lt_60`、`approved_by=Chief (pre-approved 2026-10-29 00:5x, relayed by Arch)`、`reason`（load-attribution）、`original_threshold_record_retained`（含 NOT reported as a normal-threshold pass）、`monitor_script`（sha/bytes/ascii/行数）。
- **我另按 CSV 独立重算**：`monitor.csv` 177 样本、util 2–43、free_min 13 134、最长清洁 24；`attempt2` 177/2–43/13 188/34；`attempt3` **354**/2–49/13 109/39 且 `util_ge_40=40`、`util_ge_60=0`；`attempt4` 60/4–48/13 166/**60**（`util_ge_60=0`）——与三处文档全部相符。
- 监测脚本我重算：`689fd7eb…`、2 529 B、非 ASCII 0、`lf_lines=ansi_lines=34`、注释含 `$` 行 0。
- floor 公式逐轮自校：`min(10240, baseline_free-3072)` → 10 230 / 10 222 / 10 218 / 10 220 / 10 196 全对。

## 3. 旧参考守卫（零读写）

- `materials/legacy-baseline.txt` 38 行（`scene-a` 37 + `scene-e2e` 1），我逐文件按**工作树字节**重算 sha/bytes：**mismatch=0**、总字节 25 320 515。
- 聚合：按上游口径（PowerShell `Sort-Object` 路径序 + 小写 hex 拼接再 sha256）复现 = **`b5cb50af…`**；同行拼接口径复现 = **`983617f1…`**（与 R09-S3a facts 内值一致，交叉印证）。复现脚本入 `logs/aggregate-repro.ps1`。
- `scene-shadow`：当前目录 17 文件 / 11 566 120 B（16 采集产物 / 11 561 363 B）；与窗口前快照 `target/render-bvh/r05-s3a-window/scene-shadow-before.txt` 的 17 行**逐文件 0 差异**；`git diff 926b6488..HEAD`、`4bb9cd15..HEAD` 对该目录均空。`git status --short -- scripts fixtures/render-bvh/scene-shadow scene-a scene-e2e` = **0 行**（复核时）。
- **口径提醒（观察 O-2）**：该基线按工作树字节记录，`core.autocrlf` 下文本文件是 CRLF（例：`scene-e2e/camlight-model.json` 工作树 6 434 B / committed blob 6 020 B）；按 committed blob 复算会 mismatch。守卫逻辑自洽（窗口前后同一工作树），但外部复核需按工作树字节或 LF 归一口径复算。

## 4. 窗口事实

- 采集两条命令：`hfield-only` exit 0 / 9 952 ms，`hfield-primitive` exit 0 / 4 764 ms（合计 14 716 ms）；`window_open_at=2026-10-29T00:59:35.527+08:00`、`closed 01:00:20.539`。
- 开窗瞬间 gate4：free_min **13 268** MiB、floor **10 196**（=min(10240,13268−3072)）、compute_filtered=0、build=0、3 个基线采样、签名 match=True。
- GPU 释放：`residual_render_procs=0`、`build_processes_after=0`、`gpu_released=True`（窗后 free=13 302 MiB / util≈40% 桌面基线）。
- 证据日志与窗口目录原始日志逐字节相同（capture 2 份 + monitor 4 份 + 监测脚本）。
- **零编译口径（观察 O-3）**：证据记录为「窗内零 cargo/rustc、构建进程=0」；采集日志中的 `Module … (compiled)` 是 Warp 的 Python 侧 JIT 模块编译，非 Rust 构建。证据**未记录 `target/debug/deps` 前后计数**（R09 口径里的 deps 快照在本片缺失），"窗内零编译"由构建进程计数与时间窗佐证。

## 5. 只采集不比数

证据中不存在 rgb/depth/seg 差异数字、`beyond_1_lsb`/`max_ulp`/`mismatch=` 类渲染数值结论，也没有对窗口结果、精度、上游比对或产品验收作任何正向结论表述（本卡 4 条禁词字面均 0 命中）。命中的 `mismatch` 仅出现在：① `materials/reverse-calc*`（elevation 存储约定反算，authoring 交叉核对，非渲染数值）；② README §16 的旧参考快照比对（守卫事实）；③ README §11 的判据口径声明（S3b 验收口径描述，非本片结论）。我逐行核过这三类上下文，均不构成渲染数值结论。

## 6. 观察（4 条，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| **O-1** | 建议 | 实现证据中 **4 个文件按 committed blob 含 CRLF**：`logs/capture-hfield-only.log`、`logs/capture-hfield-primitive.log`、`materials/dryrun-hfield-only.json`、`materials/dryrun-hfield-primitive.json`；manifest 头写「committed blobs, LF」，与卡面「LF 无 BOM」不符（BOM=0，这些文件无 BOM） | **修/注明**：改为 LF，或在 manifest/README 注明「窗口原始日志与 dry-run JSON 保留 Windows CRLF」 |
| **O-2** | 建议 | 旧参考基线按**工作树字节**记录（autocrlf 下文本文件 CRLF），与仓库惯例「committed blob / LF」不同；外部按 blob 复算会 mismatch（我实测 `camlight-model.json` 6 020 vs 6 434）。聚合口径亦依赖 PowerShell 路径排序（文化序） | **记录/补写**：在 `materials/legacy-baseline.txt` 或 README §6 写明「工作树字节 + PS 路径序小写 hex 拼接」，或改 LF 归一 |
| **O-3** | 记录 | 「窗内零编译」证据只到「构建进程=0」与时间窗，**未记录 `target/debug/deps` 前后计数**（R09 窗口协议有此快照，本片缺失）；另采集日志里 Warp JIT `(compiled)` 行属 Python 侧模块编译，建议证据显式区分 | **接受或补一句**（不改结论；若 Arch 要求 R09 口径则补 deps 计数） |
| **O-4** | 记录 | 复核期间 HEAD 从 `a6bcca80` 漂移到 `4bb9cd15`（S3b 验证窗 authoring 2 提交）；我核过 `git diff a6bcca80..HEAD` 未触动被核路径（S3a/S3b-authoring/场景/脚本/测试），结论不受影响；建议后续派工写「提交号」而非「HEAD」 | **记录** |

## 7. 边界与未覆盖

- 零渲染数值结论：不比较 rgb/depth/seg，不做正确性判断；所有捕获文件只做 sha/bytes/结构（npz 形状）核对。
- 不重跑采集（不开 GPU）：窗口事实以留档 + `target/` 原始日志逐字节比对（今日仍在）为准；`target/` 属瞬态，关键值已入证据。
- 未独立重算 `capture.json` 内两次运行 A/B 的原始数组（只核其 flag 与文件 sha）；未核 pinned python/驱动版本以外的环境细节。
- 旧参考逐文件核对依赖当前工作树；若未来 autocrlf/EOL 设置变化，需按 O-2 口径重算。

## 8. 收口状态

- `normify gate 2026-10-28-render-bvh-r05-s3a-hfield-capture-check` → exit 1「任务没有引用任何门禁」属预期（receipt `gate_note`）。
- 两段提交：证据（本目录 + 任务卡）→ `normify done` → verified 记录单独提交；显式路径，无 `git add -A`。
