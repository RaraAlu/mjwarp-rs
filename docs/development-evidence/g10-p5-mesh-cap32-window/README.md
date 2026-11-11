# A2-3：M2/M3/S4 重放验证窗（A2 容量 32/32；U7/U8 改判）证据

- 任务：`2026-11-12-w2-a2-3-m2-m3-s4-replay-window`（owner Backend；**单窗**，由 Chief 按四前提开窗并给 GO）。
- 日期：2026-11-12；窗口开始 02:12 前后（GO 后立即执行）；窗口前 HEAD `44164965`（执行期间无仓写）。
- 冻结：A2-2 10 exe（`target/a2-freeze/`），sha 与 `refreeze-10-exe.json` 一致；工具链
  `+1.99.0-x86_64-pc-windows-msvc`；源码树 digest `266bbf94…`（29 `.cu`，与冻结记录一致）。
- 纪律：单窗互斥、窗内只跑冻结 exe、零 Rust 编译、不写仓、不发群消息；fail-stop 8 条（就绪包 §5）。

## 1. 一句话结论

**窗口有效执行且全绿**：10/10 实体 exit 0（总 ~234 s）、sha 守卫 **27/27**（10 exe + 15 fixture + 2 参考）、
monitor 60/60、三点基线、floor 生效 10,240 MiB、零 Rust 编译、无残留计算进程。
M2/M3/S4 branch 等既有判据全过；**U8 设备 count 1→4 达成**（dist 4/4 完全相同（0 ULP）、pos 四角集合完全一致）；
**U7 可观测字段（count/dist/pos）满足判据**（count=1、dist 恰 4 ULP 边界、pos 1 ULP）。
**限制（如实记录、不宣称全条件通过）**：冻结的 limits record-only 用例只输出 count/dist/pos，未输出
`worldid/geomcollisionid/contact.geom/overflow/efc_address/frame/零容差字段` → U7/U8 的 **pass_rule 全条件无法在本窗验证**；
U8 pos 的原始写出顺序与参考捕获顺序不同（集合完全一致，见 §3）。本窗不宣称 U7/U8「正式验收通过」。

## 2. 四前提与窗口条件（Chief 已核；数值按现场记录）

| 项 | 值 |
| --- | --- |
| ① fresh freeze | `src/` 自冻结零 diff（`git diff 2ee0540d..HEAD -- src/`=0、`git diff HEAD -- src/`=0）；29 `.cu` digest `266bbf94fd92cce3202159df196d84824dacbaa6d22f4b848eddd83ac57fcb44`；10 exe 与 `refreeze-10-exe.json` 10/10 全等（未重建） |
| ② monitor | `scripts/validate/render-hvh/monitor-until-clean.ps1` → **60/60 clean**（02:06–02:11，util_max=22、free_min=12,128、过滤后计算进程 0、构建进程 0）；CSV `logs/monitor-pre-window.csv` |
| ③ 三点基线 | util 2/12/2 %、used 3,080/3,081/3,081 MiB、free 12,982/12,981/12,981 MiB、temp 25°C、driver 596.36 |
| ④ floor | 硬门 free ≥ 10,240 MiB；`floor=min(10240, baseline_free−3072)=9,909`；取严后生效 **10,240 MiB**；窗内/窗后 pre free=12,954 / post free=12,958 MiB |
| 零 Rust 编译 | 起跑前无 cargo/rustc/cl/nvcc 进程；窗内未调用任何构建；窗后复核 PASS |
| 单窗互斥 | 全队同一时间仅本窗；无其它 GPU 任务 |

## 3. 实体结果（10/10 exit 0）

| # | target | exit | 耗时 s | 结果摘要 |
| --- | --- | --- | --- | --- |
| 1 | g10_p5_branch_closure_cuda | 0 | 45.49 | S4 branch **9/9 case failures=0**；limits U7/U8 facts 见下 |
| 2 | g10_p5_nvrtc_launcher | 0 | 22.23 | 既有判据全过（测试内部断言，0 failed） |
| 3 | g10_p5_multicontact_cuda | 0 | 21.21 | 既有判据全过（测试内部断言，0 failed） |
| 4 | g10_p5_primitive_cuda | 0 | 20.19 | 既有判据全过（测试内部断言，0 failed） |
| 5 | g10_p5_primitive_plane_cuda | 0 | 22.25 | 既有判据全过（测试内部断言，0 failed） |
| 6 | g10_p5_primitive_sphere_capsule_cuda | 0 | 20.22 | 既有判据全过（测试内部断言，0 failed） |
| 7 | g10_p5_primitive_branches_cuda | 0 | 20.23 | 既有判据全过（测试内部断言，0 failed） |
| 8 | g10_p5_mesh_multicontact_m2_cuda | 0 | 20.2 | 既有判据全过（测试内部断言，0 failed） |
| 9 | g10_p5_mesh_multicontact_m3_cuda | 0 | 21.23 | 既有判据全过（测试内部断言，0 failed） |
| 10 | stage2_equivalence_window | 0 | 21.23 | `window_product_equivalence_dump` record-C：9 synth 两跑一致（落 target/stage2-impl/window/product/） |

总耗时 ≈234 s（单实体 20–45 s，远低于 300 s/实体与 900 s 总限）。

### U7 `mesh-limits-cone-box`（fan-cone-24，24/24；pass/fail 提升）

| 字段 | 设备 | 参考（pinned GPU） | 判定（可观测子集） |
| --- | --- | --- | --- |
| count | 1 | 1 | **=**（零容差） |
| dist | `-0.010000005` | `-0.010000002` | **4.0 ULP@scale ≤ 4**（恰边界） |
| pos | `[3.72529e-10, 3.72529e-10, 0.095000006]` | `[3.72529e-10, 3.72529e-10, 0.095]` | x/y 0 ULP、z **1.0 ULP ≤ 4** |

### U8 `mesh-limits-prism-box`（prism-20，20/3；**设备 count 1→4**）

| 字段 | 设备 | 参考 | 判定（可观测子集） |
| --- | --- | --- | --- |
| count | **4**（A1 前设备=1） | 4 | **=**（零容差） |
| dist | `-0.009999998,-0.009999998,-0.009999998,-0.009999998` | 同值 4 条 | **4/4 完全相同（0 ULP）（0 ULP）** |
| pos | `(0.05,0.05,0.045);(-0.05,0.05,0.045);(-0.05,-0.05,0.045);(0.05,-0.05,0.045)` | `(-0.05,0.05,0.045);(0.05,0.05,0.045);(0.05,-0.05,0.045);(-0.05,-0.05,0.045)` | **四角集合完全一致**；原始写出顺序不同（设备行序 = 参考行 [1,0,3,2]） |

**U7/U8 判定限制（必须如实引用）**：本窗冻结的 limits 用例为 S4 record-only 版本，仅打印 count/dist/pos；
pass_rule 的其余条件（结构键排序配对、`contact.geom` 序、`overflow==0`、`efc_address==-1`、frame、11 个零容差字段）
**未在本 exe 输出**，故：
- 本窗只判可观测子集：U7 count/dist/pos 满足；U8 count/dist 满足、pos 集合满足。
- **不宣称** U7/U8 pass_rule 全条件通过、不宣称「U8 已验收」；若需全条件，需另开 0 窗片扩展 limits 打印并在下一窗重放
  （会改变冻结 exe 身份，按重冻规则执行）。
- 设备 count 1→4 是 S4 record-only 记录的 **1→4 提升事实**（dist/pos 见上），可作为 A2 容量效果证据。

## 4. sha 守卫（27/27 SHA_OK）

- 10 exe（与 `refreeze-10-exe.json` 逐项 sha 一致）；15 fixture（含 A2 新增 `fan-cone-33.obj`/`prism-33.obj`）；2 参考
  （`upstream-branch-closure-gpu.json` `24d89e92…`、`upstream-mesh-limits-gpu.json` `b6abb8fa…`）。
- 完整清单：`sha-guards.txt`；运行事实：`facts.txt`；逐实体：`results.json`。

## 5. M2/M3/S4/其它回归与 CPU 项

- M2（mesh-box / mesh-cylinder 2 case）、M3（mesh-mesh 1 case，count=4）、S4 branch 9 case、launcher 4、multicontact 3、
  primitive 3、plane 6、sphere/capsule 4、branches 6+4 swap：exe 内部断言全过（exit 0，0 failed）。
- A1 CPU 回归 / A2 边界 CPU：本窗不跑 cargo（零 Rust 编译），由 A2-1/A2-2 的 unit 1409/0/402 与窗后 `normify gate` 复跑覆盖。
- NP02 6 行：A2-2 已重采（PTX `78da9dea…`）；本窗不涉及。
- stage2：record-C 只记录（9 synth 两跑一致），不做等价判定。

## 6. 边界与非声明

- 不宣称 U7/U8 pass_rule 全条件通过、不宣称 U8「已验收」、不宣称 mesh 容量缺口已清、与上游等价（over-32）、
  G10@P5 整族通过、NP01/NP02 门禁通过、性能结论。
- 无失败轮、无重试、无放宽；O3（`branch_closure_case.rs` fixture 注释 13→15）已在窗后单独修正（见卡与提交）。
- 窗口期间未写仓、未发群消息；本证据为窗后落库。

## 7. 证据文件

`README.md`、`receipt.json`、`results.json`、`u7-u8-analysis.json`、`facts.txt`、`sha-guards.txt`、
`a2-3-window-readiness.md`、`logs/`（10 实体日志 + `monitor-pre-window.csv` + `window-runner-console.log`）、`manifest.txt`。
