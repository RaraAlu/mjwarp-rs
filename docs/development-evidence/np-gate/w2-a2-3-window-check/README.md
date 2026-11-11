# QA 复核 W2/A2-3 M2/M3/S4 重放验证窗（单窗，`bd231ace` 证据 / 卡收口 `9a450a2e`）

- 复核对象：卡 `2026-11-12-w2-a2-3-m2-m3-s4-replay-window`（verified，收口 `9a450a2e`）；
  提交链 **`6cc1356c`（登记 in_progress）→ `44164965`（§1 勘误 + 三条裁决）→ `bd231ace`（窗口证据）→ `9a450a2e`（窗后收口 + addenda/manifest）**；
  证据目录 `docs/development-evidence/g10-p5-mesh-cap32-window/`（manifest `d76e6c50…`/**2,177 B**，20 条）。
- 复核基线：HEAD `f9708d14`（含后续 O3 片）；窗口时 HEAD `44164965`。
- 性质：只读复核（不占锁、不再跑 GPU）；零 GPU、零窗；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 严重、0 一般 + 3 低 + 5 观察）**。
- 窗口有效性的独立结论：**10/10 exit 0、sha 守卫 27/27、monitor 60/60、floor 生效、零 Rust 编译、无残留**均从原件独立复算成立；
  **U7/U8 可观测子集数据我按参考 JSON 原件用 f32 ULP 步进独立重算，与证据逐位一致**；pass_rule 全条件未覆盖的限制声明在位。

## 1. 窗口执行（独立复算，不只看自述）

- **10/10 exit 0 ✅**：`facts.txt` 10 条 `RESULT ... exit=0`；9 个 `g10_p5_*` 日志全部 `test result: ok`（0 failed），
  stage2 日志 `1 passed`；`results.json` 10 条 `exit_code=0`；逐项耗时合计 **234.48 s**（与 receipt `total_elapsed_s` 相等），
  单实体最大 45.49 s < 300 s、总 < 900 s，**无超时**。
- **sha 守卫 27/27 ✅**：我对 `facts.txt` 的 27 条 `SHA_OK` 逐条重算磁盘哈希——exe **10**、fixture **15**、参考 **2**，
  **全部相等、0 缺失 0 不符**；10 exe 与 `refreeze-10-exe.json` 的 sha/bytes 一致（未重建：exe mtime 01:45–01:47，早于窗口）。
- **monitor 60/60 ✅**：`logs/monitor-pre-window.csv` 实为 60 行数据（02:06:00–02:11:00，5 s 间隔），`streak=1..60`、
  `clean` 全 True、`util_percent` max **22**、`free_mib` min **12,128**、`compute_filtered` 全 0、`build` 全 0——
  与 receipt `monitor` 字段逐数一致。
- **三点基线**：util 2/12/2 %、used 3,080/3,081/3,081 MiB、free 12,982/12,981/12,981 MiB、temp 25°C、driver 596.36
  （在 README §2 与 receipt 中一致；见 §6 O2：无独立原始日志）。
- **floor 生效 ✅**：`facts.txt` `FLOOR gate: free=12954 MiB >= 10240 MiB (floor_effective=10240)`；
  窗后 `POST free=12958 MiB (floor=10240)`；`formula_value=9909`。pre/post 均远高于 floor（见 §6 O1 的公式写法问题）。
- **零 Rust 编译 ✅**：`facts.txt` `PRECHECK zero-rust-compile: no cargo/rustc/cl/nvcc process` + `POST zero-rust-compile: PASS`；
  我另扫 `target/` 在窗口时段（02:12–02:18）的全部 46 个改动文件：**无任何 `.rlib/.rmeta/.pdb/.exe/.dll` 构建产物**
  （4 个 runner 输出 + 20 个 runner 日志/stdout-stderr + 5 个 target 侧脚本/日志 + 17 个 stage2 dump）；10 个 `.err` 全 0 B。
- **无残留计算进程 ✅**：`POST compute-apps filtered residual: 0`；我另核 `run-a2-window.ps1` 的 residual 过滤为
  `python|g10_p5|cargo|mjwarp`（本机 `nvidia-smi --query-compute-apps` 在空闲时也会列出大量桌面 WDDM 进程，
  因此该过滤是必要口径；当前实机无 `g10_p5_*`/`mjwarp` 残留）。
- **stage2 两跑一致 ✅**：`stage2_equivalence_window.log` 9 个 synth（cylinder-box、ellipsoid-box、mesh-mesh、convex-mesh、
  degenerate-mesh、sphere-box-shallow、sphere-box-deep、capsule-box、sdf-box）逐条 `两次一致=true`，record-C 只记录。
- **证据来源可追溯 ✅**：10 个实体日志与 `target/a2-window/logs/` 原件**逐字节相同**；`facts.txt/sha-guards.txt/results.json`
  与原件**逐行相同**（仅 BOM/CRLF 规范化，符合卫生）；`monitor-pre-window.csv` 与 target 原件逐字节相同。

## 2. U7/U8 数据（按参考 JSON 原件独立重算）

参考：`docs/development-evidence/g10-p5-branch-closure-capture/upstream-mesh-limits-gpu.json`（`b6abb8fa…`，未改）。
判据：nacon 零容差；dist 4 ULP@scale；pos 4·ULP(参考三轴最大绝对值)。ULP 用 f32 `nextafter` 步进逐位计数。

| 项 | 设备（日志原文） | 参考（JSON 原件） | 独立重算 |
| --- | --- | --- | --- |
| U7 count | 1 | 1 | 相等 ✅ |
| U7 dist | `-1.0000005e-2` | `-0.010000001639127731` | **4 ULP（恰边界，≤4）** ✅ |
| U7 pos | `(3.72529e-10,3.72529e-10,9.5000006e-2)` | `(3.725290076417309e-10,3.725290076417309e-10,0.0949999988079071)` | **0/0/1 ULP** ✅ |
| U8 count | 4 | 4 | 相等 ✅ |
| U8 dist | 4×`-9.999998e-3` | 4×`-0.009999997913837433` | **4/4 均 0 ULP** ✅ |
| U8 pos | 4 角 | 4 角（顺序不同） | **f32 位级集合完全相等**（4 个互异角，逐轴精确相等）；设备行 → 参考行 = **[1,0,3,2]** ✅ |

- **「集合相等」不是硬凑**：四个角落的 x/y 是互异的 ±0.05，我按精确 f32 比对，device 集合与 ref 集合**每一位都相等**，
  且映射是无歧义的 [1,0,3,2] 轮换；不是容差近似配对。
- **「设备 count 1→4」历史侧可核**：旧 S4 窗证据 `g10-p5-branch-closure-validation/window-receipt.json`
  记 `u8_mesh_limits_prism_box.device_count=1 / ref_count=4`，且旧 pos 与参考不一致；本窗 count=4 且 dist/pos 对齐，提升事实成立。
- **U7 dist 恰 4 ULP 是边界**：证据 README §3 与 receipt `u7_detail.dist_ulp=[4.0]` 如实写明「恰边界，判据允许 ≤4」，
  未写成「余量充足」。

## 3. 诚实性红线

- **限制声明在位 ✅**：README §1 与 §3 明写「冻结 limits record-only 用例只输出 count/dist/pos；pass_rule 其余条件
  （结构键配对、`contact.geom` 序、`overflow==0`、`efc_address==-1`、frame、11 个零容差字段）未输出 → 不宣称 U7/U8 全条件通过、
  不宣称 U8 已验收」；receipt 有 `u7_u8_pass_rule_full_coverage=false` + 5 条 `u7_u8_limitations` + `non_claims`。
- **无越界正面声明 ✅**：我扫 window 证据全部 21 文件，禁语（全条件通过/已验收/缺口已清/整族通过/等价）**只出现在「不宣称」句**；
  `policy_scan.py` 对 21 文件 **0 命中 PASS**。
- **U8 顺序差异如实记录 ✅**：receipt `u8_pos_raw_order_equal=false`、`pos_order_rotated=true`、
  `pos_ulp_rowwise_as_printed` 四行 26,843,546 ULP（按原序逐行比会爆）如实写出，并用集合相等 + 轮换说明；
  没有把「按原序逐行比对」粉饰成通过。
- 边界：本复核只认「可观测子集满足 + 全条件未覆盖」，不将本窗读作 U7/U8 正式验收通过。

## 4. receipt 字段 ↔ 卡检查

- A2-3 卡共 13 项检查（4 gate + `unchanged src/` + 9 json）；我对 9 条 json 指针逐条解析实际文件：
  A2-2 卡 `/status=verified`、identity receipt `/a2_1_evidence_errata/policy_scan_hits=0`、window receipt
  `/task_id`、`/verdict=PASS`、`/failures=0`、`/u8_device_count=4`、`/u7_pass=true`、`/sha_guards_ok=true`、
  `/zero_rust_compile=true` —— **9/9 全部满足，0 失败**。
- 4 个 gate run：fmt/clippy/clippy-cuda/unit 全 `passed=true`、`exit=0`；unit **1409 passed / 0 failed / 402 ignored**。

## 5. §1 勘误与三条裁决（同卡第一节）

- **L1 ✅**：identity `logs/policy-scan-a2-1-evidence.log` 原 3 行失败快照**保留在文件头部**，末尾 append
  `## FINAL RESCAN` 段（`12 files 0 hits PASS` + `self-test 34/34 PASS` + 取代说明）；我独立复扫 cap32 目录 0 命中、self-test 34/34。
- **L2 ✅**：`addenda.md` 说明两处「25→33」应为 **27→33**，历史链 13→25（`25bd3479`）→27（`d46f435f`）→33（`0e5343b8`），
  以 `np02-identity-mapping.json` 的 `27/6/33` 为准（我核 mapping 实测 27/6/33）；两块 README 原文未改（历史字节不变）。
- **O1 ✅**：addenda 写死 `source_tree_digest.combined` 口径（29 个 `*.cu` 按文件名排序、LF 归一化、
  `sha256(update(name_utf8 + content))`）；**我按该口径独立复现得到 `266bbf94…cb44`**，与 `refreeze-10-exe.json` 一致。
- **O2 ✅**：addenda 声明 `a2-3-window-plan.md` 头部「不入仓」表述作废、文件已在仓内。
- **O3 ✅**：本片只记录（窗口后再修），窗口 addenda 说明 hook 拒绝原因与移出独立片的经过；独立片 `f9708d14` 已修。
- **三条裁决 ✅**：over-32 设备 case 不进窗 / inventory 全量重算不做且不得宣称全量一致 / 新 exe 身份已变需重冻，
  在 identity addenda 与 readiness §9 双处一致。
- identity manifest **12/12 逐条重算 0 错**（11→12），identity 目录 13 文件 `policy_scan` **0 命中**。

## 6. 发现

### 低
- **L1 窗口 console 日志编码损坏（证据卫生）**：`logs/window-runner-console.log` 实为 **UTF-16LE 字节流（含 BOM）被二次转码**
  —— 头 2 字节 BOM 变成两个 `U+FFFD`（`ef bf bd ef bf bd`）、其余为 ASCII 字符 + `00` 间隔，文件含 5,467 个 NUL，
  **不能作为可读 UTF-8 文本审阅**，与 manifest 头注「UTF-8/LF 无 BOM」不符。**内容可恢复且与 `facts.txt` 一致**：
  我按 UTF-16LE 还原后 27 条 `SHA_OK` 与 facts 逐字相同、全部 `RESULT/POST/SUMMARY` 行相同（仅少 `GPU pre/post` 两行），
  故不影响结论。建议：addenda 记明并以 UTF-8 规范化副本补档；组装脚本修编码（BOM 处理）。
- **L2 readiness §8 承诺的「窗内 0.25 s 采样」未实现、也未在证据中披露**：`run-a2-window.ps1` 只在窗口前后各采样一次
  GPU（pre/post），窗口内无采样循环、证据中无窗口内 monitor 日志；READINESS §8 的「窗口期间 0.25 s 采样同样监控」属于
  计划承诺，实际未落地。本窗 10 实体结果自洽、未见污染，但该控制项缺失未披露。建议 addenda 如实记录或后续窗口实现。
- **L3 identity addenda 的「不改 A2-2 任何历史字节」表述不准确**：该 §1 **同时修改了**
  `g10-p5-mesh-cap32-identity/receipt.json`（新增 `a2_3_section1_errata` 块）并重算了 identity manifest；
  addenda 只写「日志仅末尾 append」，未披露 receipt 改动。改动本身良性（收据自述、manifest 重算过），但披露不完整。

### 观察
- **O1 floor 公式写法**：README/卡/就绪包写 `floor=min(10240, baseline_free−3072)`，但「取严后生效 10,240」
  实际是 **max**（10240 > 9909）；因 pre free 12,954 同时高于两者，**结论不受影响**，建议后续统一写「取两者更严者（max）」。
- **O2 三点基线无独立原始日志**：util/used/free/temp/driver 三组数值只在 README/receipt（Chief 开窗前见证），
  证据目录无对应 raw log；建议以后把三点采样写进 facts 或单独 csv。
- **O3 addenda 的 manifest 条数表述混用**：addenda 写「manifest 20/20 自核；本 addenda 追加后 manifest 21 条」，
  实际是 **19→20 条记录**（20→21 个文件，manifest 不列自身）；属措辞，不影响自核（我复算 20/20 全对、无自列）。
- **O4 卡 addenda 记录与门禁 run 时间的可读性**：卡内 `addenda` 记录的是「为 O3 补 tests 写入范围并重跑门禁」的尝试
  （后因 hook 拒绝改为独立片），提交的 4 个 gate run `at=02:20:29–47` **晚于**卡 `closed_at=02:18:59`（run 来自该尝试的重跑）；
  两次运行均 PASS、树差异仅文档，**不影响判定**，但审计时易误读，建议在卡 note 或 addenda 点明。
- **O5 README 的 monitor 脚本路径笔误**：README §2 写 `scripts/validate/render-hvh/monitor-until-clean.ps1`，
  实际仓内为 `scripts/validate/render-bvh/monitor-until-clean.ps1`（hvh→bvh），属引用笔误。

## 7. 非声明

本复核只对 A2-3 窗口执行事实、U7/U8 可观测子集数据、证据卫生与 §1 勘误负责；**不宣称** U7/U8 pass_rule 全条件通过、
U8 已验收、mesh 容量缺口已清、G10@P5 整族通过、NP01/NP02 门禁通过、与上游等价（over-32）；**不作位级等价声明、
不提供位级等价保证**；零 GPU、零窗；不改判据/参考/容差/夹具/源码。
