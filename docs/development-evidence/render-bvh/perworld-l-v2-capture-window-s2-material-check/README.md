# S-2 采集窗材料读前预审（QA 只读）

任务：`2026-11-08-render-perworld-l-v2-capture-window-s2-material-check`（owner QA）。日期：2026-11-08。
复核对象：`2026-11-08-render-perworld-l-v2-capture-window-s2-material`（verified；落库 **`1e27ff59`**；README `3215e4ee…`/11,612 B、receipt `9e116955…`/8,349 B、manifest 12/12）。
依据：已签 spec v3 原文（`bb064fe0…`/39,379 B）§9.3、§10.2、§11.1、§12 G-L1。复核时 HEAD `c5a56639`（pilot 收口复核证据，未触及 S-2 材料）。
边界：只读 + 只写本卡证据；零 GPU、不采集、未改 fixtures。

## 结论

**PASS（0 阻断；1 条建议措辞、2 条信息项）。→ 可发用户 S-2 双签卡。**

## 1. 前置 6+1（§11.1 六项 + 第 7 项）

材料 P0–P7 与 §11.1 逐项对应，锚点均可核（我逐条 verify）：

| 项 | 锚点 | 复核 |
| --- | --- | --- |
| P0 S-1 已签/冻结 | spec `bb064fe0…`/39,379 B、receipt effective、签名卡 `ba420eb6`、QA `5e82feed` | ✅ |
| P1 command 字段 | capture-meta `b0af1be4`、QA `6419eb40`（commit 存在） | ✅ |
| P2 captured_at + 显式 env | capture-meta；pilot `2026-11-08T19:00:00+08:00` | ✅ |
| P3 per_world_scope 新口径 | pilot `metadata failed=[]`、旧文案不出现 | ✅ |
| P4 task_id / reference_tag | capture-meta；pilot `reference_tag=nw3-light-hinge02040` | ✅ |
| P5 场景变体 verified | `3d546fbe`；sha `4b354054…`/5,134 B/86 行；QA `d38661a5`/`241a2e36` | ✅ |
| P6 脚本双相机 10 字段 | `b98b39ff`；QA `241a2e36` PASS | ✅ |
| **P7 pilot=pass-margin** | 收口 `268380d8`+addenda `66bac4f6`；README `f5d52348…`/9,228 B、receipt `e4204f8f…`/6,845 B、manifest 36/36；C1 38,285/40,644；C2 depth 35,828/33,883、seg 16,041/16,754；npz 不入库 | ✅ 我独立复核（卡 `…-window-check`，证据 `c5a56639`）已按 npz 复算逐项一致 |

缺项禁签、不得改阈值/相机/recipe/参考凑条件 —— 材料写死 ✅。

## 2. G-L1 计划（可执行性）

- 执行者 **Backend（唯一写者；窗口锁 owner=Backend）**、**单次**、失败即停；`--repeat 2` 两遍逐位一致 ✅。
- 正式参考根 **`fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/`**；我核 `Test-Path fixtures/render-bvh/scene-a-light/perworld` 当前 **ABSENT**（fresh root，与命令「必须 False」一致）✅。
- `--reference-tag nw3-light-hinge02040`、`--worlds 3 --hinge-deg 0,20,40`、`--seed 20261030`、`--use-fast-math false`、`--use-shadows true` ✅。
- 显式 `MJWARP_CAPTURE_AT` + N1 ISO8601 校验（命令第 2 步）✅；pinned venv 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mjwarp 3.15.0（editable `71da24d9`）/ numpy 2.5.4 / RTX 4070 Ti SUPER 596.36 SM89 ✅。
- 产物 5 文件 ≈ **11.08 MB**（11,076,054 B 口径）✅。

## 3. 纪律与 authoring 例外

- monitor：`-Required 60 -MaxMinutes 30 -MaxUtil 40`、exit 0、60/60 全干净、util<40、**`compute_filtered=0`、`build=0`**、monitor 期间无其它 CUDA 进程 ✅。
- 三点基线在开窗前 5 分钟内；`floor = baseline_free − 3,072` 现场计算；**signed floor 取严：`floor_effective = min(signed floor, 现场 floor)`**，基线三点 + monitor 全部样本 + 采集抽检全程 free ≥ floor_effective ✅（口径与 §10.2/§12 一致）。
- **authoring 例外逐字核**：用户 2026-11-08 预批；**仅本采集窗（G-L1）生效**；常规 util<40 不达标时才允许 authoring-only util<60；**仍需 S-2 双签后才可开窗**；逐样本记录 + `exception_load_bearing=true`；**floor 不放松**；失败即停、不重试、不放宽；**pilot 与验证窗不适用**（用户原文仅 G-L1）✅。材料未把例外写成「提前解锁」。
- 零编译+指纹；单次失败即停、只清本次新建 out-dir；材料状态行/§7 明确「不构成 G-L1 启动（需 QA PASS + 用户 S-2 双签 + Chief token）」✅。

## 4. 采后自查（§9.3 顺序）

两跑逐位一致（逐文件 sha、rgb/depth/seg 各 6 键）→ C1 两对 ≥576 → C2 depth/seg 两对 ≥576 → 元数据四项 + 双相机快照 2×10 字段（nrender=2 / cam_id_map=[0,1] / cam_res 320×240×2；cam-hinge 走 fovy）→ `legacy_reference_untouched=true` → 全过才落库；**[384,576)=不足停手、<384=拒通过**；诊断项只记不判；seg 可见集合与 refit counter 原样保留在列 ✅。

## 5. 双签摘要（可直接用）

- `materials/signature-summary.txt`：S-2a 计划 / S-2b 基线+floor / 例外条款 / 失败条款 / 签名卡建议字段齐备；自述 **「Signs NOTHING here; only the user card signs」** ✅。
- **baseline 明确标为建议值**：README §5「**建议 baseline**」、signature-summary「**Suggested baseline**」、`baseline-suggestion.txt`「**suggested signed floor**」并注明开窗前现场重算取严 ✅。
- 数字自洽：19:14 三点 **13,302 / 13,302 / 13,302** → `baseline_free=13,302` → **signed floor = 13,302 − 3,072 = 10,230 MiB** ✅（算式逐项核对）。
- 「现场重算 + 取严」口径在 README §3/§5、signature-summary、baseline-suggestion 三处一致 ✅。

## 6. 边界/卫生/门禁

- `1e27ff59` 仅卡 + runs + 证据目录；`src/`、`fixtures/`、`scripts/`、`tests/`、pilot 目录、spec 目录**零 diff** ✅。
- 材料 manifest **12/12** 独立重算通过；首行 `#` + `# base dir:`；编码卫生（无 BOM/裸 CR/TAB）✅。
- 零 GPU、不采集、未改 fixtures ✅；四门禁独立复跑 fmt/clippy/clippy-cuda 通过、**unit 1371 过/0 败** ✅。

## 7. 观察项

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低（建议改措辞） | `materials/signature-summary.txt` 的「Suggested signature-card fields」子弹行 `S-2b: baseline_free 13,302 MiB; signed floor 10,230 MiB` **未标「建议值」**（同文件正文与 README §5 已标 Suggested/建议）；签名卡若直接粘贴该行，可能被读成已签事实 | 该行改为 `S-2b (suggested, re-measured at the window with stricter-wins): baseline_free 13,302 MiB; signed floor 10,230 MiB`；README §5 表格无需改 |
| O2 | 信息 | P7 锚点可再补一条我的收口独立复核卡（`2026-11-08-…-pilot-g-l0-window-check`，证据 `c5a56639`） | 可选；便于 Chief 引用「pilot pass-margin 已独立复核」 |
| O3 | 信息 | README §5 的 G-L1 证据目录写作 `…capture-window-g-l1/`（计划值） | 开窗卡命名时对齐即可 |

## 非宣称

- 本预审只判材料与已签 spec v3／用户预批口径的一致性、计划可执行性与证据卫生；**不判** G-L1 结果、不宣称采集有效/灯验收/上游等价/渲染数值结论。
- 不构成 G-L1 启动；S-2 需用户双签、G-L1 需 Chief token；未改被审材料与任何卡状态。
