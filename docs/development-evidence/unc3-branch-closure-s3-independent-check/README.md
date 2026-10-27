# UNC-3 分支收口 S3 独立复核（fixtures / 上游参考 / 捕获窗 attempt1）

**任务**：`2026-10-27-unc3-branch-closure-s3-independent-check`（owner QA）。
**被复核提交**：`43ec5676`（父 `96112ddf`）；fixtures/脚本在 `96112ddf`；复核时 HEAD = `448d4400`
（其后只有本复核任务的登记提交，`git diff 43ec5676 448d4400 --stat` 仅 `.normify/changes/<本任务>.json`）。
**结论**：**pass / 阻断 0 / 观察 10**（2 一般 + 8 建议）。未 commit、未 done。

本片是只读复核：不建 CUDA context、不跑 GPU、不重跑捕获；只读 fixture/脚本/证据/窗口日志。
**本片无门禁项**（任务 `checks` 只有 json + forbid_claim）：`normify gate` 返回「任务没有引用任何门禁」，
仅 `normify check` 可跑；实况写在 `receipt.json`。

## 0. 边界

- 未重跑上游 GPU 捕获（无窗口）；A/B 参考与窗口数字全部来自已交付证据 + 本地 `target/` 原始件。
- 上游参考正确性只做**CPU 侧独立复算**（pinned 公式、double 复算）与同源性（qpos 位型）核对；
  mesh multicontact 的 CPU 复算不在本环境（无 pinned venv 的 warp CUDA/CPU 上下文），
  mesh 三例只对 count/precheck/qpos_bits 与 fixture 几何做核对，未重算接触数值。
- 不宣称任何分支「已覆盖」、不写「窗口通过」；U7/U8 仍 record-only，不进分母。

## 1. 复核方法

1. 核提交范围与红线：`git show --name-status 43ec5676`、`96112ddf`；`git diff 509c9819 43ec5676 -- src tests Cargo.* fixtures/g10-p5/references`。
2. 13 fixture + 3 脚本 + runner：逐个 sha256/字节，三方比对（**runner 内嵌签名表**、**README §2 表**、**实际文件**）。
3. 两份参考：evidence 副本 / `run-a` raw / `run-b` raw 三处 sha256，加 `compare.log` 原文。
4. 11 个 case：xml sha、fixture keyframe qpos 的 f32 位型 vs 参考 `qpos_bits`、count（参考 vs run 日志 vs precheck）、
   `two_captures` 标志、raw A vs raw B 逐字节。
5. 窗口：`samples.csv` 134 行独立重算（samples/used_peak/delta/free_min/util/streak/temp），与 `guard.log` 聚合逐项比对；
   floor 用开窗 3 点基线重算；compute-apps 计数与过滤独立重算；零编译用 target 指纹 mtime 独立核。
6. **独立复算**（`verify_s3_deep.py`）：8 个 plane case 按 pinned 公式（`collision_primitive_core.py` L366-390 /
   L283-308 / L495-557）从 fixture qpos 复算 dist/pos，与上游 GPU 参考比 ULP；两个 OBJ 的拓扑/几何。
7. README/receipt 逐条数字抽查（`spotcheck_claims.py`，41 条）。
8. 旧参考：11 文件的 mtime/字节 + `git status` 干净；脚本/runner grep 无旧参考读写路径。
9. 交付记录 `docs/development-evidence/changes/…-s3-capture.json` 的 27 条 blob + sha256 逐条复验。

## 2. 复核点结果（逐条）

| # | 复核点 | 结果 |
| --- | --- | --- |
| 1 | 13 fixture sha256+字节，与 runner 内嵌 sha 守卫逐条 | **13/13 一致、0 不符**（字节全对；README 表用 `…/` 缩写 11 行，按 basename 对齐后 sha/字节也全对）；签名材料 16/16 一致（13 fixture + capture `71c91ee8…` + compare `27ecb54a…`）；runner `5f8f80c5…`、precheck `e26f936e…` 与 README 一致 |
| 2 | 两份参考 A/B raw + evidence 三处 sha | **三处同 sha**：`upstream-branch-closure-gpu.json` 16932 B / `24d89e92…`；`upstream-mesh-limits-gpu.json` 4581 B / `b6abb8fa…`；`compare.log` 两文件均 `RAW BYTE-IDENTICAL: PASS`（compare_exit=0）；我另做 raw A vs raw B 逐字节比较 = 完全相同 |
| 3 | 11 case `two_captures` 独立复算 | **11/11**：9 branch + 2 limits；每 case `two_captures_bitwise_identical=true`；raw A==B（整文件逐字节）⇒ 每 case 输出必然一致；另核 count 与 `run-a.log` 11/11 一致、xml sha 11/11、qpos_bits 11/11（见 §3） |
| 4 | 窗口采样重算 | **全部一致**：samples=134；baseline_used=2849；used_peak=3132；delta=283（三口径 **283 / 227 / 283**）；free_min=12930 > floor 10085；util_max=94、80% 以上连续 **2** 样本（abort 阈值 3）；temp_max=31；residual=0（见 §4） |
| 5 | 旧参考零读写 | **11 文件 mtime 均 2026-10-25**、字节与 HEAD 一致（`git status` 干净）；capture/precheck/compare/runner 无旧参考路径（grep 只命中一句「old references are never read」注释）；capture 只写两个新文件名 |
| 6 | 窗内零编译与签名守卫 | 零编译**独立证实**：`target/debug/.fingerprint` 最新 mtime 2026-10-27 01:24:42、`target/release` 2026-10-26；窗口时间窗内 `target/` 除 `s3-branch-closure` 外**无任何文件被改写**；日志无 `Compiling`/`rustc`。签名守卫：`SHA_OK` 行**未归档**（见 O3），但 16/16 签名我独立重算一致，且守卫失败会 `exit 4`、而参考已复制、facts 已产出 ⇒ 实质通过 |

## 3. 11 个 case 的同源与结构（独立核）

- 9 branch（U1/U3×5/U4/U5/U6）+ 2 limits（U7/U8）= 11；`purpose` 明写 branch=验收组、limits=record-only。
- 每 case：`xml` 路径存在、`xml_sha256` = 实际 fixture sha（11/11）；fixture `<key qpos>` 的 **f32 位型 = 参考 `qpos_bits`（11/11 精确相等）**；
  参考 `qpos_bits` = precheck `qpos_bits`（11/11）；count：参考 = run 日志 = precheck（11/11）。
- 拓扑（两个 obj 独立复算）：
  - `fan-cone-24.obj`：25 verts / 25 polys（24 三角侧面 + 1 个 24-gon 底面）/ max face 24 / max degree 24 → 与 precheck `nmeshvert=25 / nmeshpoly=25 / max_degree=24 / max_polygon_vertices=24` 全对；apex `(0,0,0.1)`，box 底 `0.09` → 穿透 0.01 → count=1 单顶点接触，与参考一致。
  - `prism-20.obj`：40 verts / 声明 42 面（40 三角 + 2 个 20-gon 帽），MuJoCo 合并共面三角后 = 20 侧面四边形 + 2 个 20-gon = **22 polys、max degree 3、max face 20**，与 precheck 完全一致；几何：顶帽 z=+0.05（20-gon），box 底 z=0.04 → 穿透 0.01，参考 4 条接触 pos=(±0.05,±0.05,0.045)。
  - U6 `nmesh=2 / nmeshvert=12 / nmeshpoly=10` 与 M0b/M3 同源口径一致（cube 6 polygon + tetra 4）。
- CPU precheck：`result=PASS`、`failures=[]`、U3 五 pose 角点并集 `[0..7]`、`u3_covers_8=true`；
  per-pose 角点 `face-down=[0,1,2,3] / face-up=[4,5,6,7] / tilt-x=[0,1] / tilt-y=[1,3] / single=[7]` 与 README §3 一致。

## 4. 窗口数字（独立重算，`samples.csv` + `guard.log`）

| 项 | 交付值 | 我重算 | 一致 |
| --- | --- | --- | --- |
| samples | 134 | 134 行（135 行含表头） | ✓ |
| baseline_used | 2849 | guard baseline 2849 | ✓ |
| used_peak | 3132 | max used = 3132 | ✓ |
| delta_peak | 283 | 3132−2849 = 283 | ✓ |
| delta 三口径 | 283 / 227 / 283 | vs guard 2849 → 283；vs 基线点 2905 → 227；vs 采样最小 used 2849 → 283 | ✓ |
| free_min | 12930 | min free = 12930 | ✓ |
| floor | 10085 | `min(10240, 13157−3072)` = 10085；12930 > 10085 | ✓ |
| util_max / 连续 ≥80 | 94 / 2 | 94 / 2 | ✓ |
| temp_max | 31 | 31 | ✓ |
| residual | 0 | guard 聚合不含该字段（见 O5）；post compute-apps 无本任务进程 → 佐证 0 | 部分 |
| window wall | 41.5 s | 09:37:42.194→09:38:23.688 = 41.494 s | ✓ |
| A/B | exit 0，29.4 / 7.4 s | facts 原文 + 日志 mtime（09:38:11 / 09:38:18）+ 两套独立 warp cache 目录 | ✓ |
| 3 点基线 | 2905/13157、2899/13163、2868/13194 | baseline.txt 原文一致 | ✓ |

## 5. 参考数值的独立复算（8 个 plane case，pinned 公式 + fixture qpos，double）

| case | 我复算 count / 参考 count | 参考↔复算 ULP（dist / pos） | 备注 |
| --- | --- | --- | --- |
| plane-cylinder-degenerate | 3 / 3 | 2 / 2 | 退化分支：vec 回退 `(r,0,0)`，dist 全 −0.009999998 |
| plane-box-corner-face-down | 4 / 4 | 0 / 0 | 位级一致 |
| plane-box-corner-face-up | 4 / 4 | 0 / 0 | 位级一致（Rx180 翻转，覆盖 {4..7}） |
| plane-box-corner-tilt-x | 2 / 2 | 25 / 25 | double vs GPU f32 融合；pos 与 dist 同量级 |
| plane-box-corner-tilt-y | 2 / 2 | 31 / 31 | 同上 |
| plane-box-corner-single | 1 / 1 | 4 / (near-zero 无意义) | pos x≈1.17e-8（近零，ULP 不可比） |
| plane-capsule-lying | 2 / 2 | 1 / 5 | 横躺：轴⊥法线（b_norm=1），双端点 x=±0.12 |
| plane-cylinder-deep-tilt | 4 / 4 | 0 / 8 | 4 条候选全写入，dist 与 pinned 逐式吻合（−0.2166 / −0.0434 / −0.1716 ×2） |

结论：**count 8/8、同源 qpos_bits 8/8；数值在 f32/双精度与融合顺序差异的预期范围内**，
未见参考与 fixture 不符（tilt 的 25–31 ULP 是「double 复算 vs GPU f32 融合」的口径差，
与交付自述的「CPU f32 vs GPU 8–16 ULP」不是同一比较，二者都不作判据）。

## 6. 观察（按级别）

### 一般（2）

- **O1 一般｜「dist/pos 位级一致」的 case 数与实际不符，且 README 与 receipt 互相矛盾。**
  README §7.5 写「位级一致的有 **6** 个 case」；receipt `cpu_precheck_vs_gpu_reference.dist_pos_bitwise_equal_cases`
  列 **5** 个（`plane-cylinder-degenerate / plane-capsule-lying / plane-box-corner-face-down / face-up / mesh-mesh-two-assets`）。
  我按 precheck 数据独立复算：dist+pos **全部位级一致 = 4 个**（degenerate、face-down、face-up、capsule-lying）；
  `corner-single` 仅 dist 位级一致（pos 差 3.2e-9）；3 个 tilt（tilt-x 8/8、tilt-y 8/8、deep-tilt 16/48 ULP）。
  另：`mesh-mesh-two-assets` 的 precheck 记录里**没有 dist/pos**（只有 count/geom_dataid/mesh），
  把它列入「dist/pos 位级一致」无数据支撑。
  建议统一为「8 个 plane case 中 dist+pos 位级一致 4 个；corner-single 仅 dist 位级；3 个 tilt 8–48 ULP；
  3 个 mesh case 只对 count」——结论（参考一律取 GPU）不变。
- **O2 一般｜U8 的「4 个接触来自 ≤16 顶点侧面」推断与 fixture 几何不符。**
  棱柱顶帽是 **20-gon**（z=+0.05），box 底面 4 角在 z=0.04、穿透 0.01，参考 4 条接触 pos 正是
  (±0.05, ±0.05, 0.045)（box 4 角与顶帽的中点）；即接触面是 **>16 顶点的 20-gon 顶帽**，
  上游 `npolygonmax` 由模型取 20（动态容量）所以不整体回退。原推断（≤16 顶点侧面）与几何相反。
  建议 S4/S5 改为「顶帽 20-gon 参与、上游不整体回退」，或补打印 polygon 号证实（仍标推断）。

### 建议（8）

- **O3 建议｜签名守卫的 `SHA_OK` 行没有归档。** runner 会把 `SHA_OK fixtures=13 capture=71c91ee8 compare=27ecb54a`
  打到 stdout，但 stdout 未落盘（`logs/` 只有 baseline/compare/facts/guard/run-a/run-b/samples；baseline.txt 只有基线/compute-apps/负载）。
  我独立重算签名 16/16 一致、守卫失败路径会 exit 4 而参考已复制/facts 已产出 ⇒ 实质通过；建议今后把 runner stdout 一并归档。
- **O4 建议｜4 个日志 UTF-8 带 BOM**（`compare.log`、`guard.log`、`run-a.log`、`run-b.log`，PowerShell `Out-File -Encoding utf8` 所致）；
  建议转无 BOM 或在 README 注明「原始输出保留 BOM」。
- **O5 建议｜`pre` 采样标注可疑。** receipt `capture_window.pre` = `2844,13218,23,31` 与 `post` 完全相同
  （README §4 同），但 guard.log 的 `phase=preconditions` 采样是 `used_mib=2849 free_mib=13213 util=39`；
  runner 也只采 post 与 3 点基线。疑为把 post 复制到 pre。不影响 floor（用开窗基线 13157）与结论。
- **O6 建议｜compute-apps 条数口径差 1。** facts 原文 post 与 baseline 各 **29** 条（各 6 条 `[Insufficient Permissions]`），
  README/receipt 写「28 条」；按 `python|g10_p5|cargo|mjwarp` 过滤后 0 条的结论不变。建议统一为 29。
- **O7 建议｜U6 的 `geom_dataid=[0,1]` 来源标注。** README §5.1 把它列在 GPU 参考「关键数值」，
  但 GPU 参考的 `output.fields` 里没有 `geom_dataid`（只有 `geom`）；该值来自 CPU precheck。
  建议注明来源；S4 对照 U6 用 `geom` 字段即可。
- **O8 建议｜S3 任务 acceptance 的「U7/U8 …超界进入回退」与证据不一致。** S3 证据只能证明模型级超界
  （degree=24>10、polygon=20>16）与上游行为（cone count=1、prism count=4）；
  「设备回退」是 S4 才验证的事。README §7.4 已避免该表述，建议 S5 汇总也不用「进入回退」。
- **O9 建议｜U7 的 count 巧合风险未写明。** 设备回退也是单接触（count=1），与上游 cone count=1 相同，
  S4 若只看 count 会「通过」却无法区分是否回退。建议 S4 对 U7 同时记录 dist/pos 并与参考逐值对照，
  在证据里写「count 相同不足以区分回退」（README 已有「不写证明限制触发」，但没写 S4 对照口径）。
- **O10 建议｜两个交付 JSON 的行数**：`receipt.json` 1047 行、`precheck-cpu.json` 648 行。若团队
  「证据单文件 ≤500 行」的规范也覆盖交付侧 JSON，建议拆分或注明「机器可读 JSON 例外」；
  本复核未把它当阻断（S3 任务 acceptance 未写该条，且 JSON 是数据文件）。

## 7. 红线与交付记录

- `43ec5676` 只改/加：`.normify` 任务记录、`docs/development-evidence/changes/…-s3-capture.json`、本证据目录的 12 个文件；
  `96112ddf` 只加 13 fixtures + 2 个 authoring 脚本 + precheck-cpu.json。
- `git diff 509c9819 43ec5676 -- src tests Cargo.toml Cargo.lock fixtures/g10-p5/references scripts/compare_primitive_capture.py` = **空**
  → 产品代码/设备源/判据/容差/旧参考/期望值零改动；fixtures 只有 13 个新增（无修改/删除）。
- 交付记录 27 条文件（13 fixture + capture + precheck 脚本 + README/receipt/precheck-cpu/2 参考/7 日志）
  的 **blob + sha256 逐条复验 0 不符**。
- S3 任务 checks 手工核：json pointer `changes/…-s3-capture.json:/task_id` 成立；forbid_claim 6 词在 S3 证据 **0 命中**。
- 证据卫生：S3 证据 UTF-8 除 O4 的 4 个日志外无 BOM；README 134 行、receipt 1047 行、precheck 648 行（见 O10）。

## 8. 证据清单 / 复现

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本报告 |
| `claims-audit.json` | 逐条复核点/数字 ↔ 我的核对结果 |
| `verify_s3.py` / `verify_s3.json` | fixtures/脚本/参考/case/采样/旧参考/窗口日志 的机器可读复算 |
| `verify_s3_deep.py` / `verify_s3_deep.json` | 8 plane case pinned 公式独立复算（ULP）+ OBJ 拓扑/几何 + 窗口杂项 |
| `spotcheck_claims.py` / `spotcheck.json` | README/receipt 41 条数字抽查（全 match） |
| `manifest.json` / `receipt.json` | 本目录 sha256 清单 / 任务判定 |

复现（只读，不会写仓库）：
`python verify_s3.py && python verify_s3_deep.py && python spotcheck_claims.py`
（依赖 numpy、可读 `target/s3-branch-closure/` 的 raw A/B 与 runner；若 target 被清理，脚本会明确指出缺失项）。
