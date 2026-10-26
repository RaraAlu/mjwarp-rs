# G10@P5 primitive 10 组合 B0（fixtures + 上游 GPU 参考捕获）· QA 独立复核

- 任务：`2026-10-26-g10-p5-primitive-capture-independent-check`（p47，owner QA）
- 复核对象：`2026-10-26-g10-p5-primitive-capture` 交付，提交 `44a53a79`（父 `767e420e`）
- 复核 HEAD：`44a53a790ff77bb86ea129f7c8520ae69287d0fa`
- 纯 CPU 复核：只读文件/日志/`nvidia-smi`，未开 GPU 窗口、未建 CUDA context（fixture 解析用 pinned venv 的 **mujoco**（CPU 库），未 import warp/mujoco_warp）。
- 结论：**阻断 1 / 观察 8**。参考的**数值内容与两轮逐字节一致可以独立复现**，但 **provenance 的 `captured_at` 是硬编码常量（19:40:00），与真实窗口（19:23:11–19:23:33）不符**，README §6 的窗口时间也写成 19:40–19:42——不满足 acceptance「captured_at 落在窗口内」，需用真实窗口时间重生成参考并同步证据文字后才能收口。
- **修正后复验（提交 `43ca9e9f`，详见 §12 与 `qa-recheck-after-fix.json`）**：**阻断 B1 已闭环**——`captured_at=2026-10-26T19:34:44.106+08:00`（= window_start），与 baseline/run/reference/samples/facts 的 mtime 交叉一致；新参考 sha `3a6475ac…` 两轮逐字节一致、**数据与修正前参考完全相同**（仅 provenance 变）；`kernel_cache_parent` 真实存在。复验结论：**pass（阻断 0 / 观察 4：交付 README/receipt 里仍有指向 attempt3 的过期文字 R1–R3 + receipt.head 非哈希 R4）**。

## 0. 结论速览

1. **dispatch（10/10 通过）**：从 frozen `collision_driver.py` 独立解析 `MJ_COLLISION_TABLE`（33 条）+ `_narrowphase` 的 `NATIVECCD` 覆盖 + `_PRIMITIVE_COLLISIONS`（13 条），10 个组合全部 `PRIMITIVE` 且 wrapper 齐全；交付 `dispatch-audit.md` 的 10 行逐行一致。`(BOX,BOX)` 表中为 CONVEX、由 L910-912 在 `nativeccd=disable` 下覆盖为 PRIMITIVE。
2. **fixtures / 输入 sha（通过）**：11 个新文件（10 MJCF + `tetra.obj`）的 sha256/字节数与 `inputs-sha256.json` 全等；用 pinned **mujoco 3.15.0（CPU）** 独立重算 10 个 fixture 的 `key_qpos` f32 位型与 `xml_sha256`，与参考 JSON 的 `qpos_bits`/`xml_sha256` **10/10 一致**，`nativeccd=disable` 全部生效、`ngeom=2`；既有 verified fixtures 0 改动（d04 的 `sphere-sphere.xml` 未动，新文件是 `fixtures/g10-p5-primitive/sphere-sphere.xml`）。
3. **两轮逐位（通过）**：raw A/B 均 `13b2399d03ae39c5a579b7c20c6855b53e020e288d5c2f89b2e84531067d338c`（13516 B，逐字节相同）；参考文件 = A。两轮是两次独立 python 调用、各自 `Warp 1.15.0 initialized`、cache 分别为 `attempt3/warp-cache-a|b`（各 36 个文件，路径不同），日志时间相差 8 s。
4. **provenance（除 captured_at 外通过）**：GPU 名称/总显存/驱动 596.36/算力 8.9 与本机只读 `nvidia-smi` 一致；解释器路径存在且 `Python 3.12.13` 一致；版本 python 3.12.13 / mujoco 3.15.0 / warp 1.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4；脚本 sha256 与仓库 `scripts/capture_primitive_gpu.py` 一致（`efcd66c5…`）；`kernel_cache_root` 是相对路径。
5. **窗口（事实通过，口径有误）**：68 个 0.25 s 样本（中位间隔 276 ms，时间戳单调），peak/free_min/util_max = 4532/11530/37 与 CSV 自算一致；前后 `query-compute-apps` 34 条、无本任务进程，无残留；结束基线同基线；窗口日志无 cargo/rustc。
6. **范围（通过）**：31 文件 = 2 个 scripts + 11 个 fixtures + 13 个证据 + 5 个 `.normify`；产品/测试/ABI/`gates.toml` **0 改动**；旧参考（`qa-upstream-gpu.json`、gjk-epa 的 CPU/GPU 参考）git blob 与父提交相同；`abi.rs` blob 相同。
7. **脚本 diff（受控）**：与既有 GPU 参考脚本相比 47+/60−，差异全部是 docstring、fixture 清单、路径、qpos 来源（`reference.json` → fixture `key`）、provenance 字段（新增 interpreter/dispatch、device 写 `cuda:0`）、输出名；`capture()` 函数体除路径参数外逐字相同，`FLOAT_FIELDS`/`INT_FIELDS` 完全一致，数值装配逻辑未被改；`compare_primitive_capture.py` 只做整文件字节比较（0 = 相同 / 2 = 停手）。
8. **边界**：证据写了「不做 host 交叉」「nacon 只作 fixture 有效性基线」「未实现组合记未覆盖」；`capsule-capsule` GPU nacon=1 vs CPU 预检 2 已如实写在 README §6 与 receipt。**「参考尚不是判据」没有逐字写出**（只有「不判产品精度/不参与判据」的等价表述）——按观察记。

## 1. 方法与隔离

- 自写脚本 `scripts/verify-primitive.ps1`（解析 frozen 源、复算 sha、独立重算窗口统计、核对证据一致性）；`scripts/recompute_qpos.py`（pinned venv + **mujoco only** 重算 fixture `key_qpos` 位型）。
- frozen 源：`docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/`（`collision_driver.py` 984 行、`collision_primitive.py` 1611 行）。
- 所有临时输出写 `target/qa-primitive/`；不写交付证据目录。

## 2. dispatch 独立复算

| # | 组合 | 表值 | nativeccd=disable 后 | wrapper |
|---|---|---|---|---|
| 1 | PLANE-SPHERE | PRIMITIVE | PRIMITIVE | plane_sphere_wrapper |
| 2 | PLANE-CAPSULE | PRIMITIVE | PRIMITIVE | plane_capsule_wrapper |
| 3 | PLANE-ELLIPSOID | PRIMITIVE | PRIMITIVE | plane_ellipsoid_wrapper |
| 4 | PLANE-CYLINDER | PRIMITIVE | PRIMITIVE | plane_cylinder_wrapper |
| 5 | PLANE-BOX | PRIMITIVE | PRIMITIVE | plane_box_wrapper |
| 6 | PLANE-MESH | PRIMITIVE | PRIMITIVE | plane_convex_wrapper |
| 7 | SPHERE-SPHERE | PRIMITIVE | PRIMITIVE | sphere_sphere_wrapper |
| 8 | SPHERE-CAPSULE | PRIMITIVE | PRIMITIVE | sphere_capsule_wrapper |
| 9 | SPHERE-CYLINDER | PRIMITIVE | PRIMITIVE | sphere_cylinder_wrapper |
| 10 | CAPSULE-CAPSULE | PRIMITIVE | PRIMITIVE | capsule_capsule_wrapper |

- `(BOX,BOX)` 表值 CONVEX、`_narrowphase` 的 `DisableBit.NATIVECCD` 分支把它覆盖为 PRIMITIVE（L910-912）——与审计一致。
- 10/10 route PRIMITIVE 且 wrapper 齐全；交付审计表逐行匹配 **10/10**。

## 3. fixtures 与输入 sha

- `inputs-sha256.json` 11 项：sha256 与字节数 **11/11 一致**。
- 10 个 MJCF：`nativeccd="disable"` 全部存在；pinned mujoco 独立重算的 `key_qpos` f32 位型与参考 `qpos_bits` **10/10 一致**，`xml_sha256` 10/10 一致，`ngeom=2`（`nq` 7/7/7/7/7/7/14/14/14/14）。
- 说明（透明记录）：`plane-box` 的 XML `qpos` 四元数不是单位四元数，mujoco 载入时归一化，因此**原始 XML 字面量按 f32 直接转换会与参考不同**；参考的 `qpos` 取自 `host_probe.key_qpos`（归一化后）。我用 pinned mujoco 按同一语义重算后 10/10 一致，这不是参考错误。
- `existing_fixtures_changed_outside_new_dir=0`；`fixtures/mjwarp-complex-physics/sphere-sphere.xml` 未被修改（新文件独立存在）。

## 4. 两轮逐位与 provenance

- A/B：sha 均 `13b2399d…338c`，13516 B，逐字节比较相等；参考文件 sha = A。
- 独立性：runner 对 python 的两次独立调用（本地 runner 脚本可读）、两次各自打印 `Warp 1.15.0 initialized`、cache 分别为 `attempt3/warp-cache-a` 与 `warp-cache-b`（各 36 文件），文件时间 19:23:20 / 19:23:28。
- provenance 真实项：GPU 名称/显存/驱动 596.36/算力 8.9、解释器 3.12.13、脚本 sha、相对 `kernel_cache_root` 均核过。
- **问题（阻断 B1）**：provenance `captured_at=2026-10-26T19:40:00+08:00` 是 runner 里写死的常量（attempt1/2/3 的 facts 头都是同一值），而真实窗口证据是 **19:23:11–19:23:33**（baseline mtime 19:23:11、samples 19:23:12–19:23:31、A/B 19:23:20/28）。README §6 也写「窗口 19:40–19:42」。→ 「captured_at 落在窗口内」不成立。
- 历史轮：attempt2 A/B sha 不同（`f7a7f5bd…` vs `e34421a5…`），provenance `kernel_cache_dir` 为每轮绝对路径——与「绝对 cache 路径导致 raw 不一致、该轮作废」一致；attempt1 run-a.log 是 **Python SyntaxError（capture 脚本 line 105）**、`runA_wall=0`，确实零 GPU，但 README 把原因写成「运行目录未建」（观察 O1）。

## 5. 窗口事实（独立重算）

- `samples.csv` 68 行：sequence 1..68、时间戳单调、中位间隔 276.4 ms（0.25 s + 启动开销），跨度 19:23:12.66–19:23:31.20。
- 自算 peak=4532 / free_min=11530 / util_max=37，与 facts 的 `sampler_samples=68 used_peak=4532 free_min=11530 util_max=37 process_exited=True residual=0` 一致。
- 基线 `4286/11776`（3 点）、post `4286/11776`、结束基线 `4286/11776`（3 点）；前后 `query-compute-apps` 各 34 条桌面/系统上下文，无本任务计算进程；`guard.log` post 行 `used_mib=4286 free_mib=11776 delta_mib=-5 own_step=True`。
- 窗口日志无 `cargo`/`rustc`（零 Rust 编译）；attempt1/2/3 的采样会话分别为 8/100/68 个样本（README/receipt 的「窗口会话 1 次 / gpu_windows=1」若是「有效轮」应写明，见观察 O7）。

## 6. 范围与脚本 diff

- `git diff 767e420e..44a53a79`：31 文件 = `scripts/` 2 + `fixtures/g10-p5-primitive/` 11 + 证据 13 + `.normify` 5；`other=0`。
- 产品代码/测试/`Cargo*`/`gates.toml` 0 改动；`abi.rs` blob 与父相同；旧参考 blob 与父相同。
- 捕获脚本 diff：见 `logs/script-diff-capture.txt`；`capture()` 函数体只有 `str(ROOT / xml_path)` → `str(xml_path)` 的路径差异，字段元组逐字相同；`py_compile` 当前脚本 exit 0。
- 对比脚本：`compare_primitive_capture.py` 只比较整文件字节，`exit 2` 停手，不提供 canonical-only 放宽路径。

## 7. 边界与「观察」核对

- 「不做 host 交叉」：README 第 4 行有；「nacon 只作 fixture 有效性基线」：README §6 有；「未实现组合记未覆盖」：README §5 有。
- 「参考尚不是判据」：**未逐字**写出（purpose 字段有 `not a product-precision verdict`，README 有「不判产品精度」）；建议补一句。
- `capsule-capsule`：GPU 捕获 nacon=1、CPU 预检 2，README §6 与 receipt 均已如实记录，未抹平；我复核参考 count=1、precheck 表为 2、其余 9 例 CPU/GPU nacon 一致。

## 8. 观察（8 条 + 1 阻断）

**阻断 B1**：`captured_at` 为硬编码常量 19:40:00，与真实窗口 19:23 不符（README §6 窗口时间同样写成 19:40–19:42）。修复建议：runner 在窗口开始时取一次真实时间，用同一值传给 A/B 两轮（保持整文件逐字节一致），重生成参考并同步 README/receipt；数据字段本身不受影响。

1. **O1** attempt1 作废原因写「运行目录未建」，原始 `run-a.log` 是 capture 脚本 line 105 的 `SyntaxError`（源码里残留字面 `` `n ``），零 GPU 属实但根因写错。
2. **O2** README 第 4 行仍写「窗口状态：未开窗（等 Arch/Chief 双签）」，与 §6 已执行窗口矛盾。
3. **O3** 实现片 receipt `head=53f3c96a`（实际交付提交 `44a53a79`）、`status=in_progress`（任务已 verified），boundaries 里还有 `not opened yet`。
4. **O4** provenance 的 `kernel_cache_root`（`target/g10-p5-primitive-capture/warp-cache`）目录不存在；真实两轮 cache 在 `attempt3/warp-cache-a|b`（各 36 文件）。
5. **O5** 「参考尚不是判据」未逐字写出（仅等价表述）。
6. **O6** `precheck-cpu.md` 的 markdown 表头/分隔行与首行粘连（`| --- || plane-sphere ...`），表格不可渲染。
7. **O7** 「窗口会话 1 次 / gpu_windows=1」与 attempt1/2/3 三次采样会话（8/100/68 样本）的关系未写明是「有效轮计数」。
8. **O8** 两轮独立调用的 runner 脚本只存在于 gitignored `target/`，未入库；committed 的 run-a/run-b 日志与 cache 目录可作旁证。

## 9. 未覆盖 / 边界

- 未开 GPU 窗口、未复跑捕获（本复核纯 CPU）；参考数值不重算，只核 provenance/一致性/输入 sha/两轮字节。
- 未做 device 对照（B1 后续片）；不宣称产品精度或验收结论。
- 未验证 pinned frozen 源与 `71da24d9` 的 git 绑定（只核 frozen 目录内 `manifest.json`/`reference.json` 引用该 rev）。

## 10. 复现命令

```powershell
# 独立复核（纯 CPU）
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-primitive.ps1
# fixture key_qpos 位型重算（pinned venv + mujoco only，不 import warp）
& target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/recompute_qpos.py
```

## 11. 证据清单

- `README.md`、`receipt.json`、`artifacts-sha256.json`
- `qa-dispatch.json`、`qa-fixtures-inputs.json`、`qa-two-run-provenance.json`、`qa-window.json`、`qa-scope-script-diff.json`、`qa-observations.json`
- `logs/`：独立复核报告、frozen 源解析、mujoco qpos 重算、捕获/对比脚本 diff、证据一致性核对
- `scripts/`：`verify-primitive.ps1`、`recompute_qpos.py`

## 12. 修正后复验（fix commit `43ca9e9f`，base `44a53a79`）

- 方法：纯 CPU 重跑 `scripts/recheck.ps1`（`logs/recheck-report.txt`）；只读新参考/attempt4 原始产物/交付证据 diff；未开窗、未建 context、未改交付证据。
- **B1 闭环证据**：
  - 新参考 provenance `captured_at=2026-10-26T19:34:44.106+08:00`，等于 facts 的 `window_start`，落在 `[19:34:44.106, 19:35:04.312]`；facts 记 `captured_at_source=runner-window-start`。
  - mtime 交叉：baseline 19:34:43.393（窗口前）、RUN A log 19:34:51.983、raw A 19:34:51.673、RUN B log 19:34:59.894、raw B 19:34:59.595、samples 19:35:01.772、facts 19:35:04.325（窗口结束后 13 ms）；samples 时间戳 19:34:44.2599–19:35:01.7728 全在窗口内。
  - 说明：captured_at = window_start，因此比第一个 0.25 s 样本早 0.154 s，这是「窗口开始」语义，不是不一致。
- **新参考**：sha256 `3a6475acf362795ab98e8ee39309b09102c2dc76f7a57c6c6dbc95a069a98734`（13654 B），raw A/B 逐字节相同、参考 = A；**cases 数据与修正前参考逐字段相同**（只有 provenance 变）；10/10 case `two_captures_bitwise_identical=true`；RUN A/B 各 7.9 s、exit 0。
- **O4 闭环**：provenance `kernel_cache_parent=target/g10-p5-primitive-capture/attempt4`（相对路径、窗口内真实存在）；`attempt4/warp-cache-a|b` 各 36 文件，run 日志分别记录自己的 cache 目录。
- **attempt4 窗口**：64 样本、中位间隔 277.3 ms、单调；自算 peak/free_min/util_max = 4406/11656/39 与 facts 一致；基线 3 点 4192/11870、4180/11882、4170/11892；post 4161/11901；compute-apps 34 条无本任务进程；residual 0；窗口内无 cargo/rustc。
- **范围**：fix commit 17 文件 = 1 个脚本（仅 provenance 字段：`kernel_cache_root` → `kernel_cache_parent` + note，数值装配未动）+ 交付证据 + `.normify`；product/fixtures/ABI/旧参考 **0 改动**（旧参考 blob 自 `767e420e` 起未变，`abi.rs` blob 相同）；新参考的 `authoring_script_sha256=ba4142…` 与仓库脚本 sha 一致。
- **O1–O8 收口状态**：O1 在 README 已改（receipt 仍写 missing dir）；O2 部分；O3 部分；O4 闭环；O5 闭环；O6 闭环；O7 部分；O8 闭环。**复验新增观察（R1–R4）**：
  1. **R1** README 头部仍写「捕获已完成（attempt3 有效轮）；captured_at 修正需重采一次小窗口」——修正窗口已完成，文字过期。
  2. **R2** README §6 仍写「attempt3 为唯一有效轮 / 窗口会话 1 次 / kernel_cache_parent=attempt3 / RUN wall 8.4/8.2」，与 §7 的 attempt4、2 次窗口会话、wall 7.9/7.9 矛盾。
  3. **R3** receipt `window.attempt_valid` 仍 attempt3，`baseline/end_baseline` 仍是 attempt3 的 4286/11776，`attempts` 列表无 attempt4 且 attempt1 仍写 missing dir，`boundaries` 仍写 correction pending，`o7_rounds` 仍 attempt3。
  4. **R4** receipt `head` 是描述文本而非提交哈希（修正提交为 `43ca9e9f`）。
- **复验结论**：**阻断 0**（B1 真闭环，数据/provenance 可独立复现）；上述 4 条均为交付证据文字过期/口径不一致，不影响参考数据与 provenance，建议收口前做一次纯文字提交（README 头 + §6、receipt.window）。

