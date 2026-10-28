# R09-S3b 阴影数值验收测试（attempt1 红 → 修复 a/b → attempt2 仪器红 → attempt2-r2 成功）

任务：`2026-10-28-render-bvh-r09-s3b-shadow-validation`（P8 渲染与 BVH；p60；owner Backend）
最终树 HEAD：`415006d8a8d0c9753aa540725a60a144c10eb5ec`
阶段：**三轮收口完成**（attempt1 判据红 → a/b 修复 → attempt2 产品判据全绿但仪器编码缺陷致 runner exit 10 → runner ASCII-only 修复重签 → attempt2-r2 runner exit 0 / `WINDOW_RUN_DONE`）
资源：`gpu:windows-t4`（验证窗已按签执行；attempt1/attempt2/attempt2-r2 原始产物全部留档）

三次分写说明：**attempt1 与 attempt2 的原始数字一字未改**（产物在 `target/s3b-validation-window/{attempt1,attempt2}/`）；attempt2-r2 为仪器修复重放（不新增判据轮，总开窗 2 次），产物已入库 `windows/attempt2-r2/`。

## 1. 文件与行数（LF 口径）

| 文件 | 行数 | 内容 |
| --- | --- | --- |
| `tests/render_shadow_reference.rs` | **307** | 4 个用例（1 host + 3 GPU `#[ignore]`）：shadow-on 判据、shadow-off 逐位回归、可见性对照、参考解析 |
| `tests/support/render_bvh/shadow.rs` | **416** | `capture-exact/` 同配方 npz 参考的最小只读解析器（ZIP_STORED + npy v1.0；不引入 npz 依赖）+ 参考层差异计数 + 冻结 sha256 断言 |
| `tests/support/render_bvh/shadow_scene.rs` | **238** | scene-shadow 产品场景装配（`fields_from_model` + `light_castshadow=[1,1,1]`）、比较/断言/运行哈希 |
| `tests/render_e2e.rs` | **500** | negative control 的 `use_shadows` 旧拒绝断言改为接受（b 项；卡 abandon、改动并入本任务） |

三个测试支持文件均 ≤500 行；未改动任何既有参考/夹具/判据。

## 2. 用例清单

| 用例 | 状态 | 内容 |
| --- | --- | --- |
| `shadow_references_parse_and_cross_checks_hold` | **host（默认执行）** | 两组 `capture-exact/` npz 解析 + sha256 冻结断言 + 形状（320×240 / 240×180）+ 参考层交叉校验（depth/seg 差异 0、RGB 差异 > 0） |
| `shadow_on_matches_captured_reference` | `#[ignore]`（GPU） | 产品 `nworld=1` + scene-shadow + `use_shadows=true` 对 `capture-exact/shadow-on/`（**同配方**）：RGB 每通道 ≤1 LSB、depth ≤4 ULP（位级 `mismatched==0` 优先）、seg 全等；两次运行 rgb/depth/seg sha256 相同 |
| `shadow_off_bitwise_regression_to_pre_unlock_reference` | `#[ignore]`（GPU） | ① 对 `capture-exact/shadow-off/`（同配方）同判据；② **逐位回归解锁前既有同配方产品参考**（`s7b/*_fast_math_false.json`）；③ 两次运行 sha256 相同 |
| `shadow_visibility_binary_control` | `#[ignore]`（GPU） | 语义交叉校验：产品 shadow-on vs shadow-off 差异像素与参考层同口径计数并列；**不作等价/精度结论** |

## 3. 判据与参考口径

- **参考来源**：`fixtures/render-bvh/scene-shadow/capture-exact/{shadow-on,shadow-off}/`（R09-S3a-capture-exact 窗口，**产品同配方 `use_fast_math=false`**，`MJWARP_CAPTURE_AT=2026-10-28T19:44:02+08:00`）；参考只读（`include_bytes!` + sha256 冻结）
- **`capture/`（fast-math 组）不参与断言**：仅历史与跨配方诊断（差异记录在 `r09-s3a-capture-exact/`）
- **场景**：`scene-shadow/scene.xml` 派生自 scene-a（几何/相机/材质相同），唯一语义差异是三灯 `castshadow=1`
- **判据零改动**：复用 `s7b1`/`s7b2` 既有 diff 原语与容差（RGB ≤1 LSB、depth ≤4 ULP、seg 全等）；未新增/放宽任何容差；未删除或降级任何用例

## 4. 验证

- 作者阶段：`--no-run` 通过；host `1 passed; 0 failed; 3 ignored`；`normify gate`（in_progress 树 `61b82f9a`）全过
- 修复后（a 卡）：全量 primary-ray 回归 `render_kernel_depth_seg` 6/6、`render_kernel_rgb` 3/3、`render_kernel_flags` 6/6、`render_e2e` **5/5**
- QA 独立复核复跑（`…-r09-s3b-check`，verdict=pass / 阻断 0 / 观察 6）：修复后 `render_shadow_reference` **3 passed**、shadow-on 两相机 rgb/depth/seg 全 0、shadow-off 双重回归 0、可见性 product 4745 == reference 4745
- **最终树四门禁（HEAD `415006d8`）**：fmt 1 560 ms / clippy 5 934 ms / clippy-cuda 455 ms / unit **68 339 ms（1050 过 / 0 败）**，全部通过

## 5. 窗口轮次记录

### 5.1 attempt1（有效轮 1，**红**；`2026-10-28T20:03:15.16 → 20:04:06.54`）

- **窗口事实**：baseline `2555/13507/2/32 ×3`；`floor=min(10240, 13507−3072)=10240`；body **48.6 s**；0.25 s 采样 163 样本（peak 2 826、free_min 13 236、util_max 9%、streak 0、delta 271、floor 未破）；守卫 **56/56**；窗后 9/9；零编译；attempt=1
- **9 实体**：`render_shadow_reference` **exit 101**/4 517 ms、`render_e2e` **exit 101**/7 953 ms，其余 7 个 exit 0（camera_rays 1 132 / context_lifecycle 7 618 / depth_seg 4 349 / flags 9 162 / rgb 3 370 / r08 7 921 / r10 2 546）
- **shadow-on**：cam0 rgb `0/0/0`；**cam1 rgb `mismatched=1451 / max_byte_diff=77 / beyond_1_lsb=1436`（红）**；depth/seg 两相机 0；两次 sha 未执行
- **shadow-off**：① 对 capture-exact 同判据 **0**；② pre-unlock 逐位回归 depth/seg/rgb 全 **0**；两次 sha 相同（rgb `af9df331…`、depth `de732643…`、seg `065caf08…`）
- **可见性对照**：product `rgb_diff=3537 / depth 0 / seg 0`；reference `rgb_diff=4745 / depth 0 / seg 0`
- **红项根因与处置**（原始数字未改）：
  1. **secondary ray 背面剔除**（cam1 1436 像素漏判）：上游 any-hit 分支 mesh 只返回命中布尔、不填 normal，cull 判据对 mesh 遮挡不生效；产品对 mesh 遮挡执行了 cull。修复卡 `…-r09-s3b-shadow-secondary-ray`（`e17da2b7` + `809032dc`）：`cast_ray` 增 `shadow_ray`，mesh 遮挡不剔背、primitive 仍剔背；primary-ray 全量回归 6/6、3/3、6/6、5/5。证据 `r09-s3b-shadow-secondary-ray/`
  2. **e2e negative control 旧契约**：`tests/render_e2e.rs:475` 的 `use_shadows` 移出拒绝面改接受、其余两类拒绝全强度；全仓 grep 证明旧契约共 3 处、无第四处；卡 `…-r09-s2b-e2e-negative-control` 因与运行中 S3b 卡冻结 scope 互锁被 **abandon**，改动并入 `d501118b`（任务 id 不变），留痕见 `attempt1-fixes/README.md`；**该 abandoned 卡文件按 normify 规则无法入库**（`change/transition` 新任务不能直接为 abandoned + `commit/scope` 超任务范围），**保持未跟踪历史记录、内容不删除**
  3. **runner 仪器 bug（旧版）**：`--nocapture` 使 println 插在 `test <NAME> ...` 与 `ok` 之间，旧单行正则把两个实际通过的用例误判 `case_ok=False`；attempt1 数字未改，仪器笔记 `target/s3b-validation-window/attempt1/instrument-note.txt`
- **产物**：`target/s3b-validation-window/attempt1/`（20 文件，**原样留档、未改**；仅写关键 sha：facts `7ba67015…`、entity-results `96c781dd…`、guard.log `49f157b7…`）

### 5.1.1 QA 独立复核（`…-r09-s3b-check`，verdict=pass / 阻断 0 / 观察 6）

- **a 修复被独立复跑确认**：`render_shadow_reference` 3 passed；shadow-on 两相机全 0 且两次 sha 相同；shadow-off 双重回归 0；可见性 4745 == 4745
- **「38」的独立口径**：QA 复制 S3a 脚本仅改 `enable_backface_culling=false`、pinned env 单渲染 → cam1 `beyond_1_lsb=38` 与 probe 一致；cam0 原始差 2 像素、`beyond_1_lsb=0`、max=1 → **probe 的 diff 是「超 1 LSB」口径**；「非产品缺陷」依据是**修复后判据全 0**，`38=38` **只是诊断计数一致、非集合相等**
- **9 exe 现态（QA 观察，仅冻结前记录）**：`render_shadow_reference 34b659bf…`（20:26:42）、`render_kernel_rgb 01f249b2…`（20:26:19），其余 7 个 20:16–20:22；**窗口以冻结值（5.3）为准**
- **e2e 关联**：attempt1 暴露的 e2e 旧断言由 b 项修复，`render_e2e` 5/5 已验证；QA 复核复跑与本窗实体同源（同树），verdict 不含 e2e 阻断项
- **SDF（gtype==8）cull 缺口**：本次修复只对齐 mesh（gtype==7）any-hit 语义；SDF shadow cull 是否同缺口**未覆盖**，已登记 backlog `2026-10-28-render-shadow-sdf-gtype-cull-gap`（本窗不处理）

### 5.2 attempt2（第 1 次开窗；产品判据全绿，**仪器编码缺陷**致 runner exit 10）

- **窗口事实**：start `2026-10-28T20:42:32.317`、entity `20:42:33.266→20:43:21.059`、body **47.8 s**、window_end `20:43:22.663`；baseline `2579/13483/2/32 ×3`；floor **10 240**；0.25 s 采样 **161** 样本（peak 2 849、free_min 13 213、util_max 13%、streak 0、delta 270/260/270、sustained 270、interval median 0.2991 s）；守卫 56/56；窗后 9/9；零编译；residual 0
- **9 实体全部 exit 0**：shadow_reference 0/4 046、camera_rays 0/1 114、context_lifecycle 0/7 524、e2e 0/8 165、depth_seg 0/4 064、flags 0/9 000、rgb 0/3 475、r08 0/7 811、r10 0/2 563
- **shadow-on 全 0**（两相机 rgb/depth/seg）；**shadow-off 双重回归全 0**；**可见性 product 4745 == reference 4745**；两次 sha 相同（rgb `f044f3fa…`、depth `de732643…`、seg `065caf08…`）
- **仪器红**：facts 记录 `shadows_acceptance_* exit_ok=`（空），`name_found/status_ok/marker_ok=True` → runner 记 2 条 failures、exit 10、无 `WINDOW_RUN_DONE`。**产品侧无红项；红因见第 7 节**
- **产物**：`target/s3b-validation-window/attempt2/`（19 文件，**原样留档、未改**），入库副本 `windows/attempt2/`；monitor `windows/monitor-pre-window-attempt2.csv`
- **监测**：60 样本、最长连续 60、util 2–4、free_min 13 483、`20:37:26.112→20:42:26.272`

### 5.3 材料冻结（最终值，HEAD `415006d8`）

- **冻结协议**：等 QA 复核 verdict 后冻结；前置 = 连续 2 次 deps 快照（间隔 ≥60 s）完全一致 + `cargo/rustc/cl` 进程 0 + GPU 无作业 + 源码树 status 空；顺序 = 最后一次 cargo → 删 9 exe 强制重链 → guards → `-GuardsOnly 56/56` → 报签
- **两次 deps 快照**：`20:32:58.615` 与 `20:34:00.728`（间隔 **62.113 s**），count **1826/1826**、max write 同为 `20:26:42.825`、`snapshots_identical=True`
- **并发覆盖留痕**：首轮冻结（20:24）后 QA 复核构建以不同 feature 指纹重写了同路径两个 exe——`render_kernel_rgb`（20:26:19，`db85e233…`→`01f249b2…`）与 `render_shadow_reference`（20:26:42）；**结论：门禁/测试会重建 exe，冻结必须在最后一次构建之后**（时间线 `target/s3b-validation-window/attempt2-build/concurrent-overwrite-note.txt`）
- **9 exe 最终值（强制重链后，`cargo test --locked --features cuda-probe --no-run`）**：

| 实体 | bytes | sha256 |
| --- | --- | --- |
| `render_shadow_reference` | 99 082 840 | `e1534a9614ad4a4d1adc460d7f2ec5a3128976d2ea189ea1b1b590883516ed5f` |
| `render_camera_rays` | 89 694 775 | `e1760e7ac54c6f1844c8771784ad4ddac5f02c9abc13588214ee2819cb6c12f0` |
| `render_context_lifecycle` | 86 165 129 | `2b72a767157410c379f97174fc6ac37611970fac46f874e143d434eb0ae43b0e` |
| `render_e2e` | 94 580 731 | `f97e65ef581fe9059de1337548d9e8dabca0d617317093a837c9b4a8315eeac1` |
| `render_kernel_depth_seg` | 90 603 308 | `1a217d0ea03b33cd6ca7f9f54a03d67f619d6d437a6ea1084ee1d60c5845c71d` |
| `render_kernel_flags` | 96 002 521 | `570d5e16cd926d7c254dec30bbe8f25304fc57aa05b6812b64387598d66b9e27` |
| `render_kernel_rgb` | 94 051 133 | `0f44c8ab5838e1c56b892318d9d2d5cfca0c9b68bab697c4843b47344ed548d4` |
| `render_r08_cameras` | 94 387 629 | `56993f1a7d31e714680a6d8e97693d79fb77ac7b71b988362818ce7fcf7a4298` |
| `render_r10_multi_world` | 94 152 658 | `52fb22b9321912051ffa7bd574fae295362b8237c2a9be4db1761dc11a59690a` |

- **guards**：`a942de1dea1abb95c520916fe9c26843d8226df1f7b11bcae9fef0027bffce25` / 18 477 B（9 exe + 47 输入）；`-GuardsOnly` 56/56（`guards-check-attempt2-final/`）
- **runner**：签名版 `53dc05ed…`/16 458 B（attempt2 用）→ 修复版 **`8a73ce53e87f6b90e2a1f78eb54fac2a1958a28462b17224ce91d71bdecab926` / 16 454 B**（attempt2-r2 用，Chief 重签）
- **baseline/floor**：`2552/13510/2/32`（r2 开窗时基线 `2552/13510`、floor 10 240）
- **materials**：`target/s3b-validation-window/attempt2-build/final-freeze/{materials.txt, exes.json, exes.txt, build-*.log, guards-only.log}`（含 `features=cuda-probe`、`relink_cmd`、`cargo --version`、两次快照原始行）

### 5.4 attempt2-r2（第 2 次开窗；仪器修复重放，**runner exit 0 / WINDOW_RUN_DONE**）

- **签名范围**：HEAD `415006d8` + 9 exe（5.3 表）+ guards `a942de1d…`/18 477 B + runner `8a73ce53…`/16 454 B + baseline `2552/13510/2/32` + floor 10 240 + attempt=2（Chief 重签，仅 runner 一项变化）
- **窗口事实**：start `20:54:02.997`、entity `20:54:03.947→20:54:52.098`、body **48.1 s**、window_end `20:54:53.611`；baseline `2552/13510/2/32 ×3`；floor **10 240**；0.25 s 采样 **161** 样本（peak 2 822、used_min 2 548、free_min **13 240**、util_max 11%、streak 0、delta 270/86/274、sustained 270、interval median **0.299 s** 0.2864–0.3191）；residual **0**、floor 未破；守卫 **56/56**；窗后 **9/9**；零编译；deps `1826 / 20:34:20` 前后一致
- **runner 结果**：`PRECHECK_OK` / `GUARDS_OK=56/56` / **`WINDOW_RUN_DONE`**，**runner exit 0**
- **9 实体全部 exit 0**：shadow_reference 0/4 498、camera_rays 0/1 128、context_lifecycle 0/7 500、e2e 0/8 308、depth_seg 0/3 999、flags 0/9 044、rgb 0/3 214、r08 0/7 646、r10 0/2 770
- **shadow-on（同配方硬判据）**：cam0 rgb `pixels=76800 mismatched=0 max_byte_diff=0 beyond_1_lsb=0`；cam1 rgb `43200/0/0/0`；cam0 depth `compared=76800 mismatch=0 max_ulp=0 beyond_ulp=0 | seg_mismatch=0`；cam1 depth `43200/0/0/0 | seg 0`；两次 sha 相同 `rgb=f044f3fa…bcdfce9f depth=de732643…e42445 seg=065caf08…293821`
- **shadow-off 双重回归**：① 对 `capture-exact/shadow-off` cam0/cam1 rgb 全 `0/0/0`；② 对 pre-unlock `s7b/*_fast_math_false.json` cam0/cam1 `depth_mismatch=0 max_ulp=0 seg_mismatch=0` 且 rgb `0/0/0`；两次 sha 相同 `rgb=af9df331…5f8a depth=de732643…e42445 seg=065caf08…293821`
- **可见性（语义交叉校验）**：product `rgb_diff=4745 depth_diff=0 seg_diff=0`；reference `rgb_diff=4745 depth_diff=0 seg_diff=0`
- **S2b 解锁两条断言（四条件，原样）**：

```
shadows_acceptance_render_kernel_flags exit_ok=True name_found=True status_ok=True marker_ok=True
shadows_acceptance_render_kernel_rgb   exit_ok=True name_found=True status_ok=True marker_ok=True
```

- **产物**：`target/s3b-validation-window/attempt2-r2/`（19 文件），入库副本 `windows/attempt2-r2/`；monitor `windows/monitor-pre-window-attempt2-r2.csv`（60 样本、最长连续 60、util 2–4、free_min 13 510、`20:48:57.637→20:53:57.696`）
- **入库副本口径（CRLF→LF 归一化留痕）**：`windows/{attempt2,attempt2-r2}/guard_aggregate.json` 入库时按 committed blob（LF）口径做 CRLF→LF 归一化（数值内容不变）：target 原件 `ed7df29b…`/1 496 B → 入库 `fb81e481…`/1 455 B；target 原件 `8b65c8e5…`/1 494 B → 入库 `951f20a4…`/1 453 B。其余 36 个 windows 文件与 target 原件逐字节相同；两份 monitor CSV 与原件同 sha（`fa240e6e…`/`95aa37b2…`）

### 5.5 记账口径与 Chief 裁决（必须随证据保留）

- `attempt1` = 有效轮 1（判据红 → 派生 a/b 修复），原始产物留档不改
- `attempt2` = 本材料**第 1 次开窗**（产品判据全绿，仪器编码缺陷致 runner exit 10），原始产物留档不改
- `attempt2-r2` = 本材料**第 2 次开窗**（仪器修复重放；产品/exe/guards/判据零变化；**不新增判据轮**；总开窗 2 次）
- Chief 裁决：批准修 runner → 重签（仅 runner 一项）→ 记 r2 为重放轮，不新增判据轮

## 6. 窗口硬要求（Chief 2026-10-28 追加）

- **S2b 解锁后的两条旧断言必须在窗口内真跑**：`render_kernel_flags` / `render_kernel_rgb` 的对应用例要执行到「`use_shadows=true` → 接受 → `Ok(())`」；runner 四条件（exit=0 + `test <NAME> ...` + 其后 `ok` 行 + marker）判定，r2 两实体全部 True
- **0.25 s 连续采样器**：r2 实测 interval median 0.299 s（0.2864–0.3191）
- 9 实体预构建冻结（sha+字节）、compute-apps 过滤 0、窗后 9/9、窗内零编译

## 7. 编码根因与修复核验（runner，必须随证据保留）

- **根因**：runner 旧签名版（`53dc05ed…`）L211-212 中文注释含全角句号；**PS 5.1 按 ANSI/GBK 读取 .ps1**，`。` 的 UTF-8 字节 `E3 80 82` 在 GBK 下 `0x82` 与紧随的 LF `0x0A` 合成字符 → **吞掉换行**，L213 `$exitOk = ($code -eq 0)` 被并入注释
- **实证**：非 ASCII 字节 **129**；ANSI(936) 解码 **342 行** vs UTF-8 解码 **343 行**（elements 口径）/ 实际行数 342 vs 341；ANSI 下含 `$exitOk` 的行 L212 以 `#` 开头
- **后果**：`$exitOk` 未赋值 → facts `exit_ok=` 空 → `caseOk=False` → 2 条 failures → runner exit 10（产品侧判据全绿）
- **修复**：只改注释为 ASCII（L211-212 两行替换，diff `2 insertions / 2 deletions`，仅注释）；`$exitOk = ($code -eq 0)` 恢复独立行（ANSI L213、`is_comment=False`）
- **修复核验**：① 非 ASCII 字节 **0**；② ANSI(936) 与 UTF-8 解码行数一致；③ `^\s*#.*\$` 命中 **0**；④ code-only **336 行相同**（UTF-8 读取、剥离空行与 `#` 注释行，`code_only_diff=0`）；⑤ `PARSE_OK`；⑥ **签名 runner 本体块回放**：用修复版解析函数对 attempt2 冻结日志离线执行 → 两实体 `case_ok=True`、`ALL_TRUE`（脚本 `target/s3b-validation-window/verify-runner-ascii-fix.ps1`）
- **runner sha 变化**：`53dc05ed01853d0042deeaf8e5f9cabe03413c34f0e56b9ab4b5ebba45057709`（16 458 B）→ **`8a73ce53e87f6b90e2a1f78eb54fac2a1958a28462b17224ce91d71bdecab926`（16 454 B）**；旧版留档 `target/s3b-validation-window/runner-history/run-window-attempt1-pre-ascii-53dc05ed.ps1`
- **长期规范**：凡要签名的 `.ps1` 一律 **ASCII-only**（PS 5.1 按 ANSI 读文件，非 ASCII 字节会吞换行/吞代码行）；签名前必须跑 ASCII 专项自检

## 8. 诊断数据 provenance（QA O1/O2）

- **瞬态诊断数据在 `target/`（不入库，可能被后续构建覆盖）**：`target/dev-diag/`（secondary-ray 假说排除期）、`target/r09-s3b-check/`（QA 复核构建与上游 cull 探针产物）、`target/s3b-validation-window/attempt2-build/`（重建日志、`exes.json`、`concurrent-overwrite-note.txt`、`final-freeze/`）、`target/s3b-validation-window/runner-history/`
- **口径**：`target/` 为瞬态目录，**关键摘要与结论已入本证据目录**；`target/` 路径只作定位引用
- **`upstream-cull-probe.log` 的 diff 计数按「超 1 LSB」口径**（同 5.1.1）：cam1 `beyond_1_lsb=38`、cam0 原始差 2 像素 / `beyond_1_lsb=0` / max=1；**不与「逐字节不等」混淆**

## 9. 限制与口径（必须随证据保留）

- 本证据不写窗口判定、覆盖声明、精度结论或等价结论（含上游与双平台）；数值只作事实记录
- 参考只读：`capture-exact/`、`capture/`、`scene-a/reference`、`s7b/*.json` 零改动；判据/容差/夹具零改动；`capture/` 不参与断言
- attempt1/attempt2 原始产物与 `instrument-note.txt` 未改；attempt2-r2 为仪器修复重放（不新增判据轮）
- SDF（gtype==8）shadow cull 缺口未覆盖，见 backlog `2026-10-28-render-shadow-sdf-gtype-cull-gap`
- `manifest.txt` 按 committed blob（LF）记录并随证据提交；不自引用
