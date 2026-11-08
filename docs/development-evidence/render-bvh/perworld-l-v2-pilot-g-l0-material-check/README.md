# G-L0 pilot 材料读前预审（QA 只读）

任务：`2026-11-08-render-perworld-l-v2-pilot-g-l0-material-check`（owner QA）。日期：2026-11-08。
复核对象：`2026-11-08-render-perworld-l-v2-pilot-g-l0-material`（verified；材料 **`ef7a8877`**；README `7d5fd874…`/10,982 B、receipt `bf3c1f3a…`/8,146 B、manifest 12/12）。
依据：**已签 spec v3 原文**（`bb064fe0…`/39,379 B）§9.3、§10.2、§11.1、§12 G-L0 —— 本预审逐条从 spec 原文取要求，不采信材料总结。
边界：只读核验 + 只写本卡证据；零 GPU、不采集、不开窗；不改被审材料与任何卡状态。

## 结论

**PASS（8/8 通过，0 阻断，4 条信息项）。可开 G-L0 —— 等待 Chief 窗口条件（Chief token + 开窗前 5 分钟内三点基线 + 常规 monitor 60/60 + floor 现场计算 + 零编译指纹）。**

## 1. 前置核对表（P1–P10）与「B 卡 QA PASS」前置

逐项对照 §12 G-L0 前置，材料 P1–P10 齐全：

| 项 | 材料锚点 | 我的复核 |
| --- | --- | --- |
| P1 S-1 已签/effective | `perworld-l-v2-light-criterion/receipt.json`（`status=effective`、`signature.spec_sha256_before==after`） | ✅ spec blob/工作树均 `bb064fe0…`/39,379 B/344 行；receipt `effective=true` |
| P2 场景变体 verified | 卡 `…-scene-camera`（`3d546fbe`）；场景 `4b354054…`/5,134 B/86 行 | ✅ 与工作树/HEAD blob 一致 |
| **P3 采集脚本双相机 verified + B 卡 QA PASS 为 pilot 前置** | 卡 `…-capture-dual-camera`（`b98b39ff`）；**「QA PASS required before the pilot」** | ✅ **明确列为前置**；该 QA PASS 即本仓 `…-capture-dual-camera-check`（`241a2e36`，PASS） |
| P4 P0-1 refit verified | 卡 `…-capture-perworld-refresh`（`e34df3b7`）；QA `3e08be09` | ✅ |
| P5 本材料 QA 预审 | 本卡 | ✅ 本预审 |
| P6–P10 基线/floor/monitor/零编译/token | `baseline-sop.txt`、`monitor-plan.txt`、`evidence-checklist.txt`、Chief | ✅ 与 §10.2/§12 一致 |

缺一项即禁开、不得提前签 S-2 换 authoring 例外 —— 材料已写死 ✅。

## 2. 逐字命令与隔离（`materials/command.txt`）

- **out-root/out-dir 全部在 `target/render-bvh/perworld-l-v2-gl0/pilot-out/…`**；明确写「**严禁**写 `fixtures/render-bvh/scene-a-light/perworld/`」（正式参考根属 G-L1）✅。
- `--scene fixtures/render-bvh/scene-a-light/scene.xml`、`--reference-tag nw3-light-hinge02040`、`--worlds 3 --hinge-deg 0,20,40`、`--use-fast-math false --use-shadows true`、`--seed 20261030`、`--repeat 2` ✅。
  - seed/repeat/nworld/hinge 与既有 L v1 参考 `capture.json.recipe`（seed 20261030 / repeat 2 / 3 world / 0,20,40）**同口径** ✅。
- **N1 显式 `MJWARP_CAPTURE_AT` + ISO8601 先校验**：我实测该片段 valid→exit 0、invalid→exit 1、empty→exit 1 ✅；dry-run 只回显 `captured_at_env{present,non_empty}`（实测 present=false）✅。
- pinned 环境：venv python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4 / upstream `71da24d9` editable / RTX 4070 Ti SUPER / driver 596.36 / SM89 ✅（dry-run 的 `environment` 块逐项吻合）。
- 产物不入库、失败只清 `target/` 本次临时树、保留日志/readout/counter 副本 ✅。

**我实跑材料 dry-run（零 GPU）**：exit 0、**未创建 out_dir**、`model_static.cameras=[[0,0,'cam-persp'],[1,0,'cam-hinge'],[2,3,'cam-track']]`、`cam_active_fixed_only=[true,true,false]`、`legacy_manifest_entries=112`、`paths.effective_out_root=target/…`、`scene sha=4b354054…/5134`、`script sha=55b91f23…/23483` —— 与材料 §2 预期逐项一致 ✅。

## 3. 判读口径（对照 §6 原文）

| 项 | spec §6 | 材料 | 判定 |
| --- | --- | --- | --- |
| C1 | `cam-persp` 两对（0→1、1→2）`max_channel\|Δrgb\|≥1/255` 计数 | 同 | ✅ |
| C2 | `cam-hinge` 两对 depth **位型**变化 + seg (id,type) **pair** 变化 | 同 | ✅ |
| 门槛 | 两套两对全部 ≥576 = pass-margin；[384,576)=停手；<384=拒通过 | 同 | ✅ |
| 诊断 | `cam-persp` depth/seg 预期 0/0；`cam-hinge` rgb 预期大；**只记不判** | 同 | ✅ |
| 禁止 | 换相机归属、诊断项冒充、改阈值/相机/recipe、产品侧数据替代 | 材料「不得」段逐条写明 | ✅ |

## 4. 脚本代码级复核（两个脚本）

**readout_c1_c2.py**（305 行 / `456da2dc…` / 13,366 B）：
- C1：`delta = max_channel |rgb_f64差|`，`count(delta >= 1/255)`；另记 bitwise 计数 ✅。
- C2：`depth.view(int32) !=` 位型比较；`seg.any(pair!=)` ✅。
- verdict：`all>=576 → pass-margin`；否则 `any<384 → non-discriminating`；否则 `insufficient`；exit 0/3 ✅。
- metadata_selfcheck 覆盖 §9.3-4 全部字段（command/argv 非空、captured_at ISO、reference_tag、scope covers 灯+geom 且旧文案不出现、`parameters.cam_active==[T,T,F]`、`context.resolved` nrender/cam_id_map/cam_res、`recipe.cameras` 2 条且 10 字段、cam-hinge sensorsize=[0,0]、两跑 bitwise 标记、files_sha256 4 键）✅。
- 只读 pilot 产物；仅 `--json-out` 写盘；authoring-only、产品路径 0 引用 ✅。

**run_pilot_with_refit_counter.py**（107 行 / `f90f896a…` / 4,389 B）：
- patch `capture.refit_scene_before_render`（capture 以模块全局名调用）与 `mujoco_warp.render`（capture 内 `mjw.render` 属性查找），记录顺序；期望 `refit→render→refit→render`（repeat=2 两遍）✅。
- 负例自洽：dry-run 透传时 **exit 4 + 空序列 + `pattern_ok=false`**，并写 counter JSON，**不会静默通过**（我实测：`REFIT_COUNTER {"sequence": [], "refit_calls": 0, "render_calls": 0, "pattern_ok": false, "capture_exit_code": 0}`，stderr `REFIT_PATTERN_MISMATCH …`，exit 4）✅。
- 写盘仅 `--counter-out`；authoring-only、产品路径 0 引用（`git grep` src/tests/scripts 无命中）✅。

**独立重跑自证**：`readout_c1_c2.py --selftest` → **SUMMARY PASS 6/6**（pass-margin / insufficient / non-discriminating / **1-ULP-only 被 1/255 规则拒绝** / identical-zero / **仅 C1 达标被拒**因 C2 必须同时达标）✅；**L v1 参考 rgb 三 world 负对照**：C1 `0/0`（bitwise 也 0）✅。

## 5. 采后自核七项

两跑逐位一致、refit 实际执行（计数+顺序）、每 world 每相机 seg (id,type) 集合、per-world rgb/depth/seg 摘要、元数据四项 + 双相机快照（nrender==2 / cam_id_map==[0,1] / cam_res==[[320,240],[320,240]] / recipe.cameras 2×10 字段）、树 clean + fixtures 未动 —— 材料 §4 + `evidence-checklist.txt` 的 `window/*` 落点齐全；readout 覆盖 §9.3-4，采集脚本对 `legacy_reference_untouched=false` 会硬失败，evidence-checklist 另记 git-before/after ✅。

## 6. 窗口纪律（§10.2 / §12 G-L0）

- **顺序**：G-L0 先于 S-2，pilot 结论是 S-2 第 7 项前置 ✅。
- **基线**：开窗前 5 分钟内三点采样，`baseline_free=min(free)` ✅。
- **floor**：`pilot_floor = baseline_free − 3072 MiB` **现场计算并记录**；用户签的 signed floor 属 S-2，**不是 G-L0 前置** ✅（§12 原文）。
- **monitor**：常规阈值 `-Required 60 -MaxMinutes 30 -MaxUtil 40`、60/60 util<40；脚本内部 10,240 MiB 仅为下界、低于 pilot_floor 须人工 STOP ✅（我核 `monitor-until-clean.ps1` 参数存在且默认参数被显式覆盖）。
- **authoring 例外不适用**：达不到常规阈值 → 停手报，不得先签 S-2 ✅。
- **零编译**：开窗前构建+指纹，窗口内无 cargo/nvcc（warp JIT 属预期）✅。
- **失败即停**：不重试、不改阈值、不落库、不删证据 ✅。

## 7. 证据卫生与边界

- 材料 `manifest.txt` **12/12** 独立重算通过；首行 `#`、`# base dir:` 在位、不自列；目录内全部文件 UTF-8/LF/无 BOM/裸 CR·TAB 0 ✅。
- `ef7a8877` 只改本卡 + 证据目录 + runs；`src/`/`fixtures/`/`scripts/` **零 diff**；材料期无采集/开窗/GPU；`target/render-bvh/perworld-l-v2-gl0/` 当前不存在（无残留临时树）✅。

## 8. 口径不扩张

全文扫描无「pilot 可省 / 阈值可调 / 失败可重试 / 放宽 / 跳过」等扩张措辞；authoring 例外仅出现在「不适用于 pilot」语境；材料明确「不构成 G-L0 启动（需 Chief token）」「不是签名对象」「不宣称判别力/采集有效/灯验收/上游等价」✅。

## 观察项（均不阻断，建议窗口卡顺手补）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 信息 | P3 只写「QA PASS required」，未写 QA 复核卡 id/提交（实际 `…-capture-dual-camera-check` / `241a2e36`） | 窗口卡 P3 锚点补 QA 卡 id+提交 |
| O2 | 信息 | readout 的 metadata_selfcheck 未显式断言 `legacy_reference_untouched`（采集脚本已硬失败兜底 + checklist 记 git-before/after） | 可在窗口自核里补一行显式断言，做纵深防御 |
| O3 | 信息 | §9.3-6/7（落库/提交采集证据）对 pilot 经 §12「不落库」不适用；材料多处写明不落库，但 §4 自核表未显式映射该例外 | 窗口卡注明「§9.3-6/7 对 pilot 不适用（§12 不落库）」 |
| O4 | 信息 | wrapper 正向路径（refit×2/render×2/顺序）只能在真 pilot 窗口证明；负向（dry-run 空序列 exit 4）已实测 | 窗口卡保留 counter JSON 原样作为执行证据 |

## 门禁

fmt / clippy / clippy-cuda 通过，**unit 1359 过 / 0 败**（本卡独立复跑）。

## 非宣称

- 本预审只判材料与已签 spec v3 的一致性、脚本口径与窗口可执行性，**不判** pilot 结果；不宣称判别力恢复、采集有效、灯验收、上游等价、任何渲染数值结论。
- 不构成 G-L0 启动；pilot 启动需 Chief token 且窗口条件满足；未改被审材料与任何卡状态。
