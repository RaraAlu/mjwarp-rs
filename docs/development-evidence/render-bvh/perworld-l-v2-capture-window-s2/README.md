# S-2 采集窗材料（perworld L v2 灯 · G-L1）

- 任务：`2026-11-08-render-perworld-l-v2-capture-window-s2-material`（owner Arch、p70）
- 依据：**已签 spec v3**（S-1，2026-11-08）`docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md` sha256 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / 39,379 B / 344 行 / 15 节；材料逐条对齐 **§11.1（S-2 前置六项 + 第 7 项 G-L0 结论）**、**§12 G-L1**、**§9.3（采集窗自查顺序）**、**§10.2（纪律）**、**§9.1/§9.2（环境与命令）**
- 状态：**QA 读前预审 PASS 后已 fix 收口**（fix 见 §8）；待**用户 S-2 双签 + Chief G-L1 token**
- 本卡**零 GPU、不采集、不改 `fixtures/`**；G-L1 执行者为 **Backend（唯一写者）**，**单次、失败即停**
- 关键数字：参考产物 **5 文件 ≈ 11.08 MB**；通过条件 **C1 与 C2 两套、两对 world 全部 ≥576**；[384,576) 停手、<384 拒通过

## 1. 前置核对表（§11.1 六项 + 第 7 项；红项禁签/禁开）

| # | 前置 | 证据锚点（当前状态） |
| --- | --- | --- |
| P0 | **S-1 已签且 spec 冻结**（G-L1 的规格前提） | spec v3 `bb064fe0…`/39,379 B；receipt `status=effective`/`spec_frozen=true`；签名卡 `2026-11-08-…-spec-v3-signature`（`ba420eb6`）、QA `5e82feed` |
| P1 | **§11.1-1 `command` 字段**：采集脚本写 `{argv 逐字, cwd, rendered}`，采集后自核非空 | `2026-11-07-render-perworld-l-v2-capture-meta`（实现 `b0af1be4`、QA `6419eb40`）；pilot `capture.json.command` 非空（窗卡 `window/capture.json`） |
| P2 | **§11.1-2 `captured_at` + env**：必须显式非空 `MJWARP_CAPTURE_AT`（ISO8601），缺/空在 GPU 前 exit 1；开窗前 N1 校验、采集后自核 | 同上 capture-meta 卡；pilot 显式值 `2026-11-08T19:00:00+08:00`（窗卡 `window/token-capture-at.txt`）；本次窗卡写死 S-2 的具体值 |
| P3 | **§11.1-3 `per_world_scope` 新口径**（覆盖 per-world 灯位姿 + geom 位姿），旧文案出现即拒 | pilot `metadata-selfcheck.json`：`scope_covers_light_and_geom=true`、`scope_no_old_wording=true`、`failed=[]` |
| P4 | **§11.1-4 `recipe.task_id` / `reference_tag`**：保留 `task_id`/`task_id_source`，新增 `reference_tag`（本次显式 `nw3-light-hinge02040`） | capture-meta 卡；pilot `capture.json.reference_tag=nw3-light-hinge02040` |
| P5 | **§11.1-5 场景变体 verified**：`cam-hinge` 按 §2.1、索引 0/1/2、LF 属性；场景 sha 与采集窗材料一致；`scene-a/` 零改动 | 场景卡 `2026-11-08-render-perworld-l-v2-scene-camera`（实现 `3d546fbe`）；sha **`4b354054…`/5,134 B/86 行**；QA `…-scene-camera-check`（`d38661a5`）；O1 普查修正 `e8a07167`；QA 复核 `241a2e36` |
| P6 | **§11.1-6 采集脚本双相机 verified**（含 `camera_recipe` 10 字段 `sensorsize`/`intrinsic`） | `2026-11-08-render-perworld-l-v2-capture-dual-camera`（实现 `b98b39ff`）；QA PASS `…-capture-dual-camera-check`（`241a2e36`） |
| **P7** | **第 7 项：G-L0 pilot 结论 = `pass-margin`**（pilot 先于 S-2；结论是 S-2 前置） | 窗卡 `2026-11-08-render-perworld-l-v2-pilot-g-l0-window`：收口 **`268380d8`** + addenda **`66bac4f6`**；README **`f5d52348…`/9,228 B**、receipt **`e4204f8f…`/6,845 B**、manifest **36/36**；C1 **38,285 / 40,644**；C2 depth **35,828 / 33,883**、seg **16,041 / 16,754**；`verdict=pass-margin`、`any_below_accept=false`；readout fix QA `0abaeebb`；**QA 收口独立复核卡** `…-pilot-g-l0-window-check`（证据 **`c5a56639`**，PASS 0 阻断）；npz 不入库（`target/…/pilot-out/` 供复读） |

> 缺任一项 → **S-2 不得签**、G-L1 不得开；不得为凑条件改阈值/相机/recipe/参考范围。

## 2. G-L1 采集窗计划（写死）

| 项 | 值 |
| --- | --- |
| 执行者 | **Backend**（唯一写者；窗口内 repo-write-lock `owner=Backend`；Arch 只读/判旁证） |
| 次数 | **单次**；失败即停、不重试、不放宽 |
| 两遍采集 | `--repeat 2`，两遍必须**逐位一致**（逐文件 sha256） |
| out-root | **`fixtures/render-bvh/scene-a-light/perworld/`**（正式参考根，允许落库；与 pilot 的 `target/` 临时根不同） |
| out-dir | `fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/` |
| tag | `--reference-tag nw3-light-hinge02040` |
| 位形 | `--worlds 3 --hinge-deg 0,20,40`；`--seed 20261030`；`--use-fast-math false`；`--use-shadows true` |
| 采集时间 | 显式 `MJWARP_CAPTURE_AT`（S-2 窗卡写死具体 ISO8601；N1 校验步） |
| 环境 | pinned venv 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0（editable `71da24d9`）/ numpy 2.5.4 / RTX 4070 Ti SUPER driver 596.36 SM89 |
| 产物 | 5 文件：`rgb.npz` 5,531,048 B + `depth.npz` 1,844,672 B + `seg.npz` 3,687,848 B + `state.npz` 2,486 B + `capture.json` ≈10 KB = **≈11.08 MB** |
| 判读 | 复用材料脚本 `perworld-l-v2-pilot-g-l0/materials/readout_c1_c2.py`（修复版 `4a054aba…`/16,711 B；arg 名 `--pilot-dir` 保留，指向正式参考目录即可；C1/C2 口径与 spec §6 相同） |
| refit 证据 | 采集经 `run_pilot_with_refit_counter.py`（`refit×2`、`render×2`、顺序 `refit→render→refit→render`），counter JSON 入证据 |

## 3. 窗口纪律（§10.2 / §12 G-L1，写死）

- **三点基线**：开窗前 **5 分钟内**三次采样（~5 s 间隔），`baseline_free = min(free)`（**采样取严**：三点取最差值；这是采样规则，与下面的 signed/现场合并规则不是一回事）。
- **现场 floor** = `baseline_free − 3,072 MiB`（现场计算，写入窗口证据）。
- **signed floor 取严**：`floor_effective = max(用户 S-2 签名的 signed floor, 现场 floor)`；**判定依据**：验收条件是 `free ≥ floor_effective`，**floor 越高越严**，所以合并取**更高值**（取 `min` 等于放宽）；窗口全程（基线三点 + monitor 全部样本 + 采集期间抽检）free ≥ `floor_effective`。
- **monitor**：`monitor-until-clean.ps1 -Required 60 -MaxMinutes 30 -MaxUtil 40 -OutCsv <显式路径>`；**exit 0、60/60 全干净、util<40、`compute_filtered=0`、`build=0`**；monitor 期间**不得有其它 CUDA 进程**；脚本内部 10,240 MiB 只是下界、不替代 `floor_effective`。
- **authoring 例外（用户 2026-11-08 已预批）**：仅当常规 `util<40` **不达标**时允许 authoring-only `-MaxUtil 60`；**仅本采集窗（G-L1）生效**；**仍需 S-2 双签后才可开窗**；逐样本记录、`exception_load_bearing=true`、free ≥ `floor_effective` 仍强制；**pilot 与验证窗不适用**；失败即停、不重试、不放宽。
- **零编译**：窗口前完成构建并记录指纹；窗口内禁 `cargo`/`nvcc`（warp JIT 属预期）；采集前核 HEAD/树 clean。
- **单次**：失败即停；仅清理本次新建的 out-dir 残件（不动既有参考），保留全部日志/证据。

## 4. 采集后自查（spec §9.3 顺序）

1. **两次采集逐位一致**：逐文件 sha256（rgb/depth/seg 各 6 个 world×相机键），`capture.json.two_captures_bitwise_identical=true`；
2. **C1**：`cam-persp` 相邻 world（0→1、1→2）`max_channel|Δrgb|≥1/255` 计数，两对均 **≥576**；
3. **C2**：`cam-hinge` 相邻 world 的 `depth_changed_px`（位型不等）与 `seg_changed_px`（pair 不等），两对均 **≥576**；
4. **元数据四项 + 双相机快照**：`command`/`captured_at`/`reference_tag`/`per_world_scope` + `context.resolved`（`nrender==2`/`cam_id_map==[0,1]`/`cam_res==[[320,240],[320,240]]`）+ `recipe.cameras` 2×10 字段（含 `sensorsize`/`intrinsic`；cam-hinge 走 fovy 路径）；
5. **`legacy_reference_untouched == true`**；
6. 以上全过 → **落库**（正式参考根，允许提交）；任一失败 → **停手报用户**（[384,576) 余量不足停、<384 拒通过；不重试、不改阈值、不换参考）；
7. **证据归档**（`window/*`，见 `materials/evidence-checklist.txt`）后进入验证前置（G-L2）。

附加（不设门槛、只记录）：诊断项 `cam-persp` depth/seg（预期 0/0）、`cam-hinge` rgb（预期大）；每 world 每相机 seg 可见 `(id,type)` 集合；refit counter JSON 原样保留。

## 5. 给用户的 S-2 双签摘要（可直接用）

| 签名项 | 内容 |
| --- | --- |
| **S-2a（计划）** | G-L1 采集窗计划：执行者 Backend、**单次**、`--repeat 2` 两遍逐位一致、正式参考根 `fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/`、tag/seed/fastmath/shadows/`MJWARP_CAPTURE_AT` 如上；产物 5 文件 **≈11.08 MB** |
| **S-2b（基线 + signed floor）** | **建议 baseline**（2026-11-08 零 GPU 三点实测，详见 `materials/baseline-suggestion.txt`）：used 2,760 MiB、free **13,302 MiB**、util 2–3%、26 °C → `baseline_free=13,302`、**signed floor = 10,230 MiB**（**suggested**：建议值，非签名值）；**取严规则**：开窗时现场重算，`floor_effective = max(signed floor 10,230, 现场 floor)`（**取严 = 取更高**） |
| **authoring 例外** | 用户 2026-11-08 已预批：常规 `util<40` 不达标时允许 authoring-only `util<60`，**仅本采集窗生效**，仍需 S-2 双签，逐样本记录，失败即停、不重试、不放宽 |
| **失败条款** | 任一采集/判读/元数据/legacy 失败 → 停手报用户，不重试、不改阈值/相机/recipe、不换参考；落库前失败只清本次新建 out-dir 残件 |
| **证据/QA** | 证据目录 `docs/development-evidence/render-bvh/perworld-l-v2-capture-window-g-l1/`（开窗卡命名时对齐此路径）；QA 读前预审本材料 + 采集后 QA 复核；窗口记录同 pilot 格式（基线与 monitor 用同一份原始记录） |

**签名卡建议字段**：窗口名 `G-L1 perworld L v2 灯采集窗`；依据 spec v3 sha256 `bb064fe0…`/39,379 B（S-1 已签）；S-2a 计划摘要（上表第 1 行）；S-2b `baseline_free=13,302` / `signed floor=10,230`（取严说明）；authoring 例外条款（上表第 3 行）；失败条款；执行者 Backend / 单次 / 产物落正式参考根；证据与 QA 安排。

> 现场约束：开窗时若外部 GPU 负载未退场或 monitor 不干净 → **不发 token/不开窗**；现场 floor 若比 10,230 更高（更严）则用现场值（取严 = 取更高者）。

## 6. 材料清单

| 文件 | 用途 |
| --- | --- |
| `README.md` | 本材料本体 |
| `receipt.json` / `manifest.txt` | 机器可读回执 / sha256+bytes 清单（首行 `#`，含 `# base dir:`） |
| `materials/preconditions.txt` | 前置表 P0–P7（逐项锚点，缺项禁签） |
| `materials/window-plan.txt` | G-L1 计划（路径/参数/环境/产物/单次） |
| `materials/command.txt` | 逐字命令（冻结/指纹/N1/dry-run/wrapper 采集/判读/证据/失败清理） |
| `materials/baseline-sop.txt` | 三点基线与 `floor_effective` 取严 SOP |
| `materials/baseline-suggestion.txt` | 本次零 GPU 三点实测原值与建议 signed floor（开窗现场重算） |
| `materials/monitor-plan.txt` | monitor 计划（常规阈值 + authoring 例外预批范围 + exit 语义） |
| `materials/evidence-checklist.txt` | 采后证据清单（`window/*` 落点） |
| `materials/qa-prereview-checklist.txt` | QA 读前预审清单 |
| `materials/signature-summary.txt` | 用户双签摘要与签名卡字段（可直接粘贴） |
| `materials/group-message-template.txt` | 群内留痕模板 |

## 7. 边界与非声明

- **零 GPU、不采集、不改 `fixtures/`**；`unchanged src/`、`unchanged fixtures/`、`unchanged scripts/`、`unchanged tests/`；四门禁通过；材料 sha256/bytes 记 receipt+manifest。
- 本材料**不构成 G-L1 启动**：需 QA 读前预审 PASS + **用户 S-2 双签**（计划 + 基线 + signed floor）+ Chief token；pilot 与验证窗规则不因本材料改变。
- **不宣称**采集有效、per-world 灯验收、与上游等价、任何渲染数值结论；建议 baseline 仅是开窗前参考值（非签名值），signed floor 以 S-2 签名卡为准，并在现场取更高（更严）者。

## 8. 修订记录（fix 轮，2026-11-08）

QA 读前预审 PASS（卡 `…-capture-window-s2-material-check`，`7c76ad6b`，0 阻断 / 1 建议措辞 / 2 信息项）后，按 Chief 2026-11-08 派工做一次 fix，**不改任何判据、阈值、相机归属、recipe 或参考范围**：

1. **floor 合并规则语义反转更正（判据级措辞，非阈值）**：`floor_effective` 由 `min(signed, onspot)` 改为 **`max(signed, onspot)`**。依据：验收条件是 `free ≥ floor_effective`，**floor 越高越严**；取 `min` 会放宽（例：signed 10,230 / 现场 10,300 → 必须按 10,300 卡，而不是 10,230）。同时写明 `baseline_free = min(三点 free)` 是**采样取严**（取最差样本），与 signed/现场合并规则不是同一件事。同句修正覆盖：`materials/baseline-sop.txt`、`materials/signature-summary.txt`、`materials/baseline-suggestion.txt`、`materials/command.txt`（执行单）、`materials/qa-prereview-checklist.txt`、`materials/evidence-checklist.txt`、本 README §3/§5、`receipt.json`。
2. **QA O1**：`signature-summary.txt` 的签名卡字段行标注 **suggested**，并写死「开窗重测、**stricter = higher** wins」。
3. **QA O2**：P7 锚点补 QA 收口独立复核卡 `2026-11-08-render-perworld-l-v2-pilot-g-l0-window-check`（证据 `c5a56639`，PASS 0 阻断）——本 README P7 行、`materials/preconditions.txt`、`receipt.json`。
4. **QA O3**：G-L1 证据目录名核对为 `…/perworld-l-v2-capture-window-g-l1/`（README §5 / signature-summary / receipt 三处一致），并注明开窗卡命名时对齐。

签名对象、前置项、计划、authoring 例外范围、失败条款、采后自检与 [384,576)/<384 口径均未改动；`unchanged src/, fixtures/, scripts/, tests/`，零 GPU、不采集。
