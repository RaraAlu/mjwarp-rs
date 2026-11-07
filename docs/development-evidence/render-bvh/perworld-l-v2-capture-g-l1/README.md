# perworld L v2 · G-L1 采集窗材料（可签计划草案）

- 任务：`2026-11-07-render-perworld-l-v2-capture-g-l1-material`（owner Arch、p60、P8-渲染与BVH）
- 依据：判据 spec v2 `2026-11-06-render-perworld-l-v2-light-criterion-spec-v2`（sha `6cfaa57e…`/20,681 B）§2.2/§6/§10.3/§11.1/§12 G-L1；§11.1 前置实现卡 `2026-11-07-render-perworld-l-v2-capture-meta`（`b0af1be4`→QA PASS `6419eb40`）
- 性质：**材料/计划**，供 Chief 整理成 S-2 签名卡。本材料**不采样、不开窗、不写参考、不改判据/夹具**；零 GPU。
- 状态：**条件草案（conditional）**——见 §0 的 **P0 阻塞发现**：在 P0 修复并通过 pilot 前，S-2 **不可签**。

---

## 0. 签名前必须知晓的两件事（材料期发现）

### 0.1 路径差异（需 Chief 确认）

- 派工消息给的命令是 `--out-root fixtures/render-bvh/scene-a/perworld`；
- **S-1 已递出的 spec v2 §2.2 写死**：`输出根 = fixtures/render-bvh/scene-a-light/perworld/`、`参考目录 = fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/`。
- **本材料按 spec v2 写**（S-1 是签名对象）。若确要落在 `scene-a/perworld/`：① 属改 S-1 参考范围 → 需新 spec sha + 用户重签；② 会让 `legacy_manifest()` 排除**整个既有 L v1 参考根**（`scene-a/perworld/`），削弱「既有参考未被改」的保护；③ 且与 spec §2.2/§10.2 的输入冻结口径不一致。**建议维持 spec v2 路径**，请 Chief 确认后再发 S-2。

### 0.2 P0 阻塞发现：现有 L v1 参考无 world 间判别力（实测）

本材料期对既有 perworld 参考 `fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/` 做了只读实测（不渲染、不开窗）：

| 检查 | 结果 |
| --- | --- |
| `state.npz` 的 `qpos` | 三 world **不同**（hinge 0 / 0.349066 / 0.698132 rad），`geom_xpos/geom_xmat` 亦逐 world 不同 |
| `rgb.npz` 的 `rgb_w0/w1/w2_cam0` | **三 world 逐位完全相同**（max abs diff = **0.0**） |
| `depth.npz` 的 `depth_w0/w1/w2_cam0` | **完全相同**（0.0） |
| `seg.npz` 的 `seg_w0/w1/w2_cam0` | **完全相同** |
| 判据脚本判定 | `non-discriminating`（两对 world 的 depth/seg 变化像素均为 **0**，见 `materials/discriminability-selftest.txt` 最后一例） |

即：既有 L v1 参考的**输入位姿不同、输出却逐位相同** → 该参考**不具备 world 间判别力**，用它与产品做「逐 world 对照」时，对照是空转的。

**未定因**（如实登记，不假装已定论）：候选包括——① 采集脚本 `put_data → forward → create_render_context → render` 之间**没有每 world 的场景刷新**（`create_render_context` 用创建时快照建场景 BVH/场景数据，`render` 自身不 refit；与 R06 多帧 H1「陈旧 BVH/快照」同类）；② pinned 上游 render 的多 world 位姿消费语义与预期不同；③ 该参考的采集轮次与脚本现状不同源。**需要一次小规模 GPU pilot（或实现卡的零 GPU+小 GPU 对照）来定因**。

**影响**：
1. L v1 窗口关于「per-world geom 位姿」的**参考对照**需复核（历史证据不回改；但任何**新**声明不能建立在无判别力参考上）。
2. **G-L1 不能直接按现脚本开窗**：若按现脚本采 `nw3-light-hinge02040`，判据输入很可能同样逐位相同 → 采集必然被判 `non-discriminating`（白烧一个窗口）。
3. 因此本材料定为 **conditional draft**：前置项见 §7。

---

## 1. pinned 环境与逐字命令

### 1.1 环境（写死，同 spec v2 §9.1）
- `target/mjwarp-parallel/venv`，Python **3.12.13**；
- `mujoco_warp` 以 **editable** 装于 pinned checkout `target/mjwarp-parallel/upstream` @ **`71da24d9`**（mujoco_warp 3.15.0），**禁止 PyPI 轮子**；
- `warp-lang 1.15.0`、`mujoco 3.15.0`、`numpy 2.5.4`；
- GPU：RTX 4070 Ti SUPER / driver 596.36 / SM89（与既有 perworld 参考同一台）；
- 冻结上游 `git status --porcelain` 必须为空；环境版本写入 `capture.json.environment/recipe`。

### 1.2 开窗前准备（N1：`MJWARP_CAPTURE_AT` 的 ISO8601 显式校验）
开窗前在窗口会话里显式执行（pinned python，CPU）：
```powershell
$env:MJWARP_CAPTURE_AT = "2026-11-07T23:00:00+08:00"   # 示例：实际值由窗口卡写死
target/mjwarp-parallel/venv/Scripts/python.exe -B -c "import os,datetime; v=os.environ['MJWARP_CAPTURE_AT']; datetime.datetime.fromisoformat(v.replace('Z','+00:00')); print('CAPTURE_AT_OK', v)"
```
- 校验不通过或 env 缺失/空串 → **不开窗**；采集脚本自身也会在 GPU 初始化前 exit 1（§11.1 前置②已实现）。
- N2（已知口径）：`capture.json.command.argv` 是**重建值**（`sys.executable` + `sys.argv`，不含 `-B` 等解释器 flags；尾空白裁剪），`rendered` 用 `subprocess.list2cmdline`。
- N3（已知口径）：`materials/synthetic-capture-metadata.json` 首行是 `SYNTHETIC` 标记行，不是纯 JSON（形状样例专用）。

### 1.3 逐字采集命令（spec v2 §2.2 路径；**P0 修复后**执行）

先 dry-run（零 GPU）：
```powershell
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/capture_render_perworld.py --dry-run `
  --scene fixtures/render-bvh/scene-a-light/scene.xml `
  --out-root fixtures/render-bvh/scene-a-light/perworld `
  --out-dir fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040 `
  --reference-tag nw3-light-hinge02040 --worlds 3 --hinge-deg 0,20,40 `
  --use-fast-math false --use-shadows true --seed 20261030 --repeat 2
```
正式采集（一次，不重试；`--dry-run` 去掉）：
```powershell
$env:MJWARP_CAPTURE_AT = "<窗口卡写死的 ISO8601>"   # 已按 1.2 校验
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/capture_render_perworld.py `
  --scene fixtures/render-bvh/scene-a-light/scene.xml `
  --out-root fixtures/render-bvh/scene-a-light/perworld `
  --out-dir fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040 `
  --reference-tag nw3-light-hinge02040 --worlds 3 --hinge-deg 0,20,40 `
  --use-fast-math false --use-shadows true --seed 20261030 --repeat 2
```
- `--out-dir` 必须**不存在或为空**（append-only；脚本会拒非空）；`legacy_manifest()` 以前后比对保护既有参考。
- **本材料不执行以上命令**；执行属 G-L1 窗口卡。

---

## 2. 采集后自检（窗口内必须全过；任一失败 → 停手，不重试、不改阈值）

1. **两次采集逐位一致**：`--repeat 2`，`capture.json.two_captures_bitwise_identical = true`（脚本已在失败时非零退出）。
2. **判别力自检 ≥576**（验收判据 384 的 1.5× 余量；**depth 与 seg 都要**，两对 world 都要）：
   ```powershell
   target/mjwarp-parallel/venv/Scripts/python.exe -B `
     docs/development-evidence/render-bvh/perworld-l-v2-capture-g-l1/materials/discriminability_check.py `
     --reference-dir fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040
   ```
   - 期望 `overall = pass-margin`、exit 0；`[384,576)` → `insufficient-margin` exit 3；`<384` → `non-discriminating` exit 3（见 `materials/discriminability-selftest.txt` 的 4 个校准样例）。
   - `spot` 的 0/0 是场景几何事实（在 `tri_body` z 旋转轴上），**只记录不参与判定**；判别力只由 `sun`/`lamp` 贡献。
3. **元数据四项自核**（独立读 `capture.json`）：
   - `command.argv` 非空、`cwd` 为仓库根、`rendered` 非空；
   - `captured_at` 非空 ISO8601（等于显式 env 值）；
   - `reference_tag == "nw3-light-hinge02040"`；
   - `per_world_scope.covers` 含 per-world 灯 + geom；`does_not_cover` **不含**旧文案 `per-world light poses (device path indexes lights single-world)`；
   - `recipe.task_id` 非空且 `task_id_source` 存在（方案 (b) `reference_tag`，见 `2026-11-07-render-perworld-l-v2-capture-meta`）。
4. 产物布局与 spec v2 §2.3 对齐：`rgb.npz`/`depth.npz`/`seg.npz` 每 world 独立键、`state.npz` 按 nworld 首维堆叠、`capture.json` 齐备；体积约 5.55 MB。

---

## 3. 开窗纪律（写死）

1. **三点基线**（开窗前 5 分钟内，间隔 ~5 s）：`memory.used/free/utilization.gpu/temperature.gpu` + `--query-compute-apps`（无本任务 CUDA 计算任务）+ build 进程 0 + 树 clean；
2. **floor 取严**：`signed floor = baseline_free − 3072 MiB`（`baseline_free = min(three free)`）。注意 monitor 脚本内部只按 **10,240 MiB** 下界判定，**不能替代 signed floor**——若样本落在 `[10240, signed)` 区间，脚本不会自停，**操作员必须按 signed floor 手动 STOP**；
3. **monitor 60/60**：`scripts/validate/render-bvh/monitor-until-clean.ps1 -Required 60 -MaxMinutes 30 -MaxUtil 40 -OutCsv target/render-bvh/perworld-l-v2-gl1/monitor.csv`；连续 60 点 util<40、free≥signed floor、无外部计算任务、build=0；
4. **authoring 例外**（S-2 预批时可用）：常规 `util<40` 不达标时允许 authoring-only `util<60`——必须 `-MaxUtil 60`、**逐点记录** util、注明 `exception_load_bearing`，且 free 仍须 ≥ signed floor；
5. **零 Rust 编译**：本窗不跑 `cargo`、不重建任何 Rust exe；warp JIT（NVRTC）kernel cache（`target/render-bvh/perworld-cache`）命中/编译属预期，记录耗时与 cache 状态；
6. **失败即停**：任一自检失败 / monitor 不达标 / 基线不达标 → 停手，保留现场，报 Chief/用户；**不重试、不改阈值、不删证据、不换命令**；
7. 窗口结束：释放 GPU（无残留进程），记录 post-window GPU 状态与树状态。

---

## 4. S-2 双签拆分与 authoring 例外

| 签名 | 内容 | 时机 |
| --- | --- | --- |
| **S-2a** | **本材料（计划）**：环境/命令/自检/纪律/证据清单 | 现在（随 S-1 之后） |
| **S-2b** | **开窗基线 + signed floor**（三点原始值 + `baseline_free−3072`） | 开窗前 5 分钟内，用户第二次签 |
| 例外 | **authoring-only `util<60` 预批**：是否允许常规 `util<40` 不达标时启用 | 在 S-2 一并预批；没有预批就只能等常规阈值 |
| GO | G-L1 token（Chief） | 两个签名齐备 + 材料贴群后 |

---

## 5. 证据归档清单（G-L1 窗口收口时）

```
docs/development-evidence/render-bvh/perworld-l-v2-capture-g-l1/
  README.md                 # 本材料（计划；窗口结果另加 attempts/ 段）
  receipt.json              # 材料 id/sha、边界、P0 状态
  manifest.txt              # 首行 #，逐文件 sha256+bytes
  materials/{command.txt, baseline-sop.txt, monitor-plan.txt, evidence-checklist.txt,
             discriminability_check.py, discriminability-selftest.txt}
  window/                   # 窗口执行时产生（由 G-L1 窗口卡负责）
    baseline-three-points.txt
    monitor.csv
    capture-stdout.log / capture-stderr.log   # 含 command + exit code 行
    capture.json 副本 + 每 npz sha256 清单
    discriminability.json
    metadata-selfcheck.txt
    environment-provenance.txt
    post-gpu.txt
    git-before.txt / git-after.txt
```
- 归档要求：原始输出副本（`target/` 只作现场）、UTF-8/LF/无 BOM、manifest 首行 `#`、每文件 sha256+bytes、裸 CR/TAB 自查。

---

## 6. 非声明

- 本材料只是**计划**：不代表窗口已开/已通过；**P0 未修复前 S-2 不可签**。
- 不产参考、不宣称采集有效、不宣称 per-world 灯位姿覆盖、不宣称与上游等价、不宣称任何渲染数值结论。
- 「材料通过」只表示材料要素齐全，不构成 G-L1 或判据通过。

---

## 7. P0 阻塞前置（材料期发现，需先处置）

| # | 前置 | 说明 |
| --- | --- | --- |
| **P0-1** | **采集脚本每 world 场景刷新**（建议最小改法：`put_data → forward` 后、`render` 前显式 `mjw.refit_bvh(m,d,rc)`；与 R06 多帧修复同型） | 现有脚本无此调用；既有 L v1 参考的「输入不同、输出逐位相同」与该类快照/陈旧语义一致（未定因）。零判据改动，改后需四门禁 + CPU 自检 |
| **P0-2** | **小规模 GPU pilot 判定力**（1 次，可在 G-L1 窗口内先跑；或独立 mini 窗口） | pilot = 用修后脚本采一次（或截断到 1 次采集）→ 跑本材料判别力脚本；`pass-margin` 才继续正式采集；`non-discriminating` → 停手报用户，**不重试** |
| **P0-3** | **L v1 声明面复核** | 历史证据不回改，但「per-world geom 位姿覆盖」若引用该参考作对照，需在后续声明中加限定/复核（由 Chief 决定是否起复核卡） |
| **P0-4** | 若 P0-2 仍失败 | 说明「逐 world 场景刷新」不足以解决，需按 §0.2 候选②③继续定因（另起诊断卡），**不得**在窗口内改 recipe/判据或放宽判别力门槛 |

**建议**：先登记 P0-1 实现卡（owner Backend，零 GPU，四门禁 + CPU 自检）→ P0-2 pilot（可在 G-L1 窗口内作为第一步，失败即停）→ pilot 通过再正式采集；S-2 在 P0-1 verified 后再签。
