# G-L0 pilot 材料（perworld L v2 灯）

- 任务：`2026-11-08-render-perworld-l-v2-pilot-g-l0-material`（owner Arch）
- 依据：**已签 spec v3**（S-1，2026-11-08）`docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md` sha256 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / 39,379 B / 344 行 / 15 节；材料逐条对齐 §9.3（采集窗自查顺序）、§10.2（窗口材料与纪律）、§11.1（S-2 前置六项 + G-L0 结论为第 7 项）、§12 G-L0（pilot 前置/判定/停手）
- 状态：**材料草案，待 QA 读前预审 PASS + Chief G-L0 token**；本卡不采集、不开窗、零 GPU、pilot 产物不入库
- 定位：G-L0 pilot 是 **authoring、1 个窗口、不落库**；判定只看 **C1 与 C2 两套判别力**是否都 `pass-margin`；pilot **先于 S-2**，且 **authoring 例外（util<60）不适用于 pilot**

## 1. 前置核对表（逐项 + 证据锚点；红项即禁开）

| # | 前置 | 证据锚点（当前状态） |
| --- | --- | --- |
| P1 | **S-1 已签**，spec v3 sha/bytes 冻结、receipt 生效 | `perworld-l-v2-light-criterion/receipt.json`（`status=effective`、`signature.spec_sha256_before==after`）；签名卡 `2026-11-08-…-spec-v3-signature`（`ba420eb6`）；QA fix-check `…-signature-fix-check`（`5e82feed`） |
| P2 | **场景变体 verified**：`cam-hinge` 按 §2.1、索引 0/1/2 = persp/hinge/track、LF 属性 | `2026-11-08-render-perworld-l-v2-scene-camera`（`3d546fbe`）；场景 sha `4b354054116ddb81c6209ed10c021d133ea30d6fd82c1a6f866bafff9f625716` / 5,134 B / 86 行；QA 预审 `…-scene-camera-check`（`d38661a5`） |
| P3 | **采集脚本双相机 verified**（`camera_recipe` 10 字段；主 487 / meta 270 行） | `2026-11-08-render-perworld-l-v2-capture-dual-camera`（实现 `b98b39ff`） |
| P3b | **B 卡 QA PASS 为 pilot 前置**：spec §12 G-L0 的「采集脚本双相机验证 verified」**= QA PASS 后成立**（实现卡 verified ≠ QA PASS） | QA 复核卡 `2026-11-08-render-perworld-l-v2-capture-dual-camera-check`（证据 `241a2e36`，**PASS**）；缺此禁开 |
| P4 | **P0-1 refit verified**（每次 render 前恰一次 refit、一次覆盖全部 world） | `2026-11-07-render-perworld-l-v2-capture-perworld-refresh`（`e34df3b7`）；QA `…-capture-perworld-refresh-check`（`3e08be09`） |
| P5 | **本材料 QA 读前预审 PASS**（本卡落库后由 QA 复核） | 本目录 + QA 复核记录（待） |
| P6 | **三点基线**（开窗前 5 分钟内） | `materials/baseline-sop.txt` |
| P7 | **常规 monitor**：`-Required 60 -MaxMinutes 30 -MaxUtil 40` exit 0、60 个样本 util<40 | `materials/monitor-plan.txt` |
| P8 | **floor 现场计算**：`floor = baseline_free − 3072 MiB` 并记入 pilot 材料（用户签的 signed floor 属 S-2，**不是** G-L0 前置） | `materials/baseline-sop.txt` |
| P9 | **零编译**：开窗前完成构建并记录指纹；窗口内无 `cargo`/`nvcc` | `materials/evidence-checklist.txt`（`zero-compile-fingerprint`） |
| P10 | **Chief G-L0 token**（pilot 是 Chief 级，不是用户签名） | Chief 消息 |

## 2. 环境与逐字命令

- pinned 环境（§9.1）：venv Python 3.12.13；warp 1.15.0；mujoco 3.15.0；mujoco_warp 3.15.0（editable @ `71da24d9`，禁 PyPI 轮子）；numpy 2.5.4；GPU = RTX 4070 Ti SUPER / driver 596.36 / SM89；`use_fast_math=false`、`use_shadows=true`、`--repeat 2`。
- **pilot 产物路径（写死，不入库）**：out-root `target/render-bvh/perworld-l-v2-gl0/pilot-out`、out-dir `.../nw3-light-hinge02040`、`--reference-tag nw3-light-hinge02040`。**严禁**写 `fixtures/render-bvh/scene-a-light/perworld/`（那是正式参考根，属 G-L1）。
- **token 附带的 `MJWARP_CAPTURE_AT`**：材料**不写占位值**；开窗时使用 Chief token 中给定的具体 ISO8601，先按 N1 校验格式，并原样记入 pilot 报告（`window/token-capture-at.txt`）。指纹命令一律用 `(Get-FileHash -Algorithm SHA256 <path>).Hash`（本机无 `sha256sum`；`certutil -hashfile <path> SHA256` 等价），完整顺序见 `materials/first-minute.txt`。
- 逐字命令见 `materials/command.txt`，含：N1 `MJWARP_CAPTURE_AT` ISO8601 校验步（dry-run 只回显 env 状态）→ dry-run（应见 `model_static.cameras` 3 条、`cam_active_fixed_only=[true,true,false]`、`legacy_manifest_entries=112`，exit 0 且不建目录）→ **经 `run_pilot_with_refit_counter.py` 的 pilot 采集**（一次性，不重试）→ `readout_c1_c2.py` 判读 → 基线/monitor → 失败清理。
- **采集必须经 refit 计数 wrapper**：直接调用采集脚本不算通过，因为缺 §9.3(4) 要求的「refit 实际执行」证据。

## 3. 判读口径（spec §6 写死，不得改）

| 项 | 定义 | 位置 |
| --- | --- | --- |
| **C1（灯驱动）** | `cam-persp` 相邻 world 对（0→1、1→2）像素级 `max_channel |Δrgb| ≥ 1/255` 的计数 | `readout_c1_c2.py` → `c1_cam_persp` |
| **C2（几何驱动）** | `cam-hinge` 相邻 world 对的 `depth_changed_px`（f32 位型不等）与 `seg_changed_px`（(id,type) 不等） | `readout_c1_c2.py` → `c2_cam_hinge` |
| 判定 | 两套、两对**全部 ≥576** → `pass-margin`；任一落 **[384,576)** → `insufficient`（停手）；任一 **<384** → `non-discriminating`（拒通过） | `readout_c1_c2.py` → `verdict`（exit 0 / 3） |
| 诊断（只记录不判） | `cam-persp` 的 depth/seg 变化（预期 0/0，场景几何事实 §3.2）；`cam-hinge` 的 rgb 变化（预期很大） | `diagnostics_no_threshold` |
| 预测量（仅供对照，不是实测） | C1 ≈ 55,581 / 55,839 px；C2 depth ≈ 35,812 / 33,947；C2 seg ≈ 16,041 / 16,755（spec §3.3/§3.4） | 材料不据此判通过 |

**不得**：换相机归属、拿诊断项冒充 C1/C2、因未达标而改阈值/相机/recipe、用产品侧数据替代参考侧自算。

## 4. 采后自核（七项，逐项留证）

1. **两次采集逐位一致**：逐文件 sha256（rgb/depth/seg 各 6 个 world×相机键；`capture.json.two_captures_bitwise_identical=true`）；
2. **refit 实际执行**：`run_pilot_with_refit_counter.py` 计数 `refit×2`、`render×2`、顺序 `refit→render→refit→render`（`pattern_ok=true`），否则 exit 4 停手；
3. **每 world 每相机可见 seg (id,type) 集合**（`readout` → `per_world_summary.*.seg_pairs`）；
4. **per-world rgb/depth/seg 摘要**（同上：rgb mean/min/max、depth 命中与 min/max、seg 集合）；
5. **元数据四项**：`command`（非空 argv）、`captured_at`（非空 ISO8601）、`reference_tag == nw3-light-hinge02040`、`per_world_scope` 新口径且旧文案不出现；
6. **双相机快照**：`context.resolved.nrender==2`、`cam_id_map==[0,1]`、`cam_res==[[320,240],[320,240]]`；`recipe.cameras` 2 条且每条 **10 字段**（含 `sensorsize`/`intrinsic`；`cam-hinge` `sensorsize=[0,0]` 走 fovy 路径）；
7. **树与隔离**：`git status --porcelain` 空、`git rev-parse HEAD` 记录；`fixtures/` 零改动。
8. **`legacy_reference_untouched` 显式断言**：`capture.json.legacy_reference_untouched == true`（采集脚本已硬失败兜底，这里再显式记一行做纵深防御）。
9. **refit counter JSON 原样保留**：`run_pilot_with_refit_counter.py` 产出的 `refit-counter.json` 作为「refit 实际执行」的**原始执行证据**（正向 `refit×2`/`render×2`/顺序只能在真 pilot 窗口证明，O4）。

**§9.3 第 6/7 步（落库参考、提交采集证据）对 pilot 不适用**：按 §12 G-L0「不落库」，pilot 只提交摘要/计数/日志（本材料 §5/§7 与 `materials/evidence-checklist.txt`），**不写** `fixtures/render-bvh/scene-a-light/perworld/`。

## 5. 窗口纪律（与 §10.2/§12 G-L0 对齐）

- **顺序**：G-L0 **先于 S-2**；pilot 结论是 S-2 的第 7 项前置。
- **基线**：开窗前 5 分钟内三点采样，`baseline_free = min(free)`；`pilot_floor = baseline_free − 3072 MiB` 现场计算并记录；用户签的 signed floor 属 S-2，pilot 不用它。
- **monitor**：常规阈值 `-Required 60 -MaxMinutes 30 -MaxUtil 40`；**exit 0 且 longest_clean ≥60、60 个干净样本 util<40** 才算窗口成立；**exit≠0（含超时 exit 20）= 窗口不成立 → 停手报 Chief、等新 token，不算 pilot 失败、不重试、不放宽**。脚本内部 `free ≥ 10,240 MiB` 是**硬编码下界**、不等于 `pilot_floor`，操作员仍须按 `pilot_floor` 人工 STOP。`-OutCsv` 父目录缺失时脚本会**尝试自动创建**（第 31–34 行），仅当 `-OutCsv` 没有父目录段（裸文件名）才 throw（第 30 行）；材料仍要求第 ④ 步前先 `New-Item -ItemType Directory -Force target/render-bvh/perworld-l-v2-gl0`。
- **零编译**：窗口前完成构建并记录指纹；窗口内只允许 python（warp JIT 属预期），不得 `cargo build/test`。
- **authoring 例外不适用**：pilot 达不到常规 monitor 条件 → 停手报用户；**不得**为开 pilot 先签 S-2 换 util<60。
- **失败即停**：任何失败保留现场、只清理 `target/` 下本次 pilot 临时树、不重试、不改阈值、不入库、不删证据。

## 6. 材料清单（本目录）

| 文件 | 用途 |
| --- | --- |
| `README.md` | 本材料本体（前置/命令/口径/自核/纪律/边界） |
| `receipt.json` | 机器可读回执（`/task_id`、状态、前置、判据、工具哈希、边界、非声明） |
| `manifest.txt` | 本目录 sha256+bytes（首行 `#`，含 `# base dir:`，不自列） |
| `materials/command.txt` | 逐字命令（含 N1、dry-run 预期、wrapper 采集、判读、基线/monitor、失败清理） |
| `materials/first-minute.txt` | **token 到手后第一分钟**顺序：冻结状态 → 指纹（`Get-FileHash`，无 `sha256sum`）→ N1 env（token 给定 ISO8601）→ 显式建目录 → 启 monitor → monitor PASS 后 5 分钟内三点基线 → floor → 采集；含全部停手点 |
| `materials/pilot-plan.txt` | 前置表 P1–P10、判定矩阵、停手规则 |
| `materials/baseline-sop.txt` | 三点基线与 floor 计算 SOP |
| `materials/monitor-plan.txt` | monitor 计划（常规阈值；例外不适用） |
| `materials/evidence-checklist.txt` | 采后证据清单（window/* 的落点） |
| `materials/readout_c1_c2.py` | **新增**判读脚本：C1/C2 + 元数据自核 + seg 集合 + per-world 摘要 + verdict；`--selftest` 6 例 |
| `materials/readout-selftest.txt` | 判读脚本/计数 wrapper 的自检与真实数据负对照记录 |
| `materials/run_pilot_with_refit_counter.py` | **新增** pilot 运行器：记录 refit/render 实际调用计数与顺序（直接调用脚本不算证据） |
| `materials/qa-prereview-checklist.txt` | QA 读前预审清单（9 项） |
| `materials/group-message-template.txt` | 群内留痕模板（开窗前/通过/停手/基建失败） |

**判读脚本新增理由**：v2 时代的 `perworld-l-v2-capture-g-l1/materials/discriminability_check.py` 只认 depth/seg（v2 §6 口径），不覆盖 v3 的 **C1 rgb + `1/255` 量级规则**、双相机归属与元数据/seg 自核；因此本材料**新增** `readout_c1_c2.py`，并**不复用**旧脚本，也不修改任何产品/采集代码。两个脚本均 **authoring-only、零 GPU、只读 npz 与 capture.json**，不得被产品路径 import。

**工具自检（已跑，CPU）**：`readout_c1_c2.py --selftest` → `SUMMARY PASS 6/6`（pass-margin / insufficient / non-discriminating / 1-ULP-only 被 `1/255` 规则拒绝 / 全 0 / 仅 C1 也被拒因 C2 必须同时达标）；真实数据负对照（既有 L v1 参考 rgb 三 world 逐位相同）→ C1 计数 **0/0**；refit 计数 wrapper 在 dry-run 透传下 **exit 4 + 空序列**，证明它只记录真实 refit/render 执行。

## 7. 已批准事项（Chief 2026-11-08）与 G-L0 token 条件

1. **临时根已批准**：pilot 产物放 `target/render-bvh/perworld-l-v2-gl0/`；**严禁**写 `fixtures/render-bvh/scene-a-light/perworld/`（正式参考根属 G-L1）。
2. **npz 处置已批准**：pilot 后 npz **保留在 `target/`** 供 QA/Chief 复读，复核完再删；**只提交摘要/计数/日志，npz 不入库**。
3. **token 条件（Chief 按 §12 G-L0 执行）**：① 本材料 QA 读前预审 **PASS**（已：`2026-11-08-render-perworld-l-v2-pilot-g-l0-material-check`，证据 `95c70bf0`，8/8）；② **B 卡 QA PASS**（已：`…-capture-dual-camera-check`，证据 `241a2e36`）；③ token 现场核：开窗前 5 分钟内三点基线 + 常规 monitor **60/60 util<40**（authoring 例外不适用）+ 零编译/构建指纹 + `floor = baseline_free − 3072 MiB` 现场计算并记入 pilot 材料；④ 全部满足才发**单次** G-L0 token（失败即停、不重试、不落库）。
4. **执行者**：**Backend 开窗**；Arch 只读设计 / 判旁证（不执行窗口）。
5. **开窗时点**：现在 GPU 有团队外负载（Chief 2026-11-08 观测 util 87% / free 8,522 MiB，非团队任务），monitor 不可能连续干净 → **等外部负载退场、monitor 干净后再发 token**。
6. **无用户签名级新增**：S-1 已签；S-2/S-3 仍待签，本材料不构成任何签署对象。

## 8. 边界与非声明

- **零 GPU、不采集、不开窗、不入库任何 pilot 数据**；`unchanged src/`、`unchanged fixtures/`、`unchanged scripts/`（材料期不改脚本；发现脚本缺陷 = 停手报 Chief）。
- 不宣称判别力恢复、采集是否有效、per-world 灯验收、与上游等价、任何渲染数值结论；§3 的预测量不是实测。
- 本材料不构成 G-L0 启动（需 Chief token），也不替代 QA 读前预审；pilot 失败即停、不重试、不放宽。
