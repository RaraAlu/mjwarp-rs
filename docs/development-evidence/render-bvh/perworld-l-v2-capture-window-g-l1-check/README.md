# QA 独立采后复核：G-L1 采集窗（`6951c3fe`；零 GPU）

- 任务：`2026-11-09-render-perworld-l-v2-capture-window-g-l1-check`（owner QA、p84、P8-渲染与BVH）
- 对象：窗卡 `2026-11-09-render-perworld-l-v2-capture-window-g-l1`（verified）；收口提交 **`6951c3fe`**（parent `6f9201e3`）；参考根 `fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/`
- 依据：已签 spec v3 `bb064fe0…`/39,379 B §2.3/§2.4/§6/§9.3/§10.2/§12；S-2 签名 `2f361d91` + QA `4a19e003`
- 方法：**自建脚本从参考 npz 独立复算** C1/C2（不采信 readout）→ 与 readout.json、capture.json 三方对账 → 逐文件扫窗口证据（monitor CSV 逐样本复算、baseline、refit、provenance、树/零编译、事故记录）→ 卡/边界/manifest/门禁
- 基线：HEAD `e49666ea`（期间 Arch 的 S-3 fill 提交落地）、树 clean、锁 FREE（细节见 §7 O5）

## 0. 结论

**PASS（0 阻断）→ G-L1 采集证据可用于 S-3 冻结/验证窗。**

- 参考 5 文件 sha/bytes 与总量 **11,078,239 B** 逐位吻合工作树与 `6951c3fe` blob。
- 自建脚本复算与 readout/capture **三方全项一致**：C1 magnitude 38,285/40,644、bitwise 43,604/41,815；C2 depth 35,828/33,883、seg 16,041/16,754；诊断 cam-persp 0/0、cam-hinge 50,363/52,329（bitwise 54,380/54,250）。
- §2.4 对账、元数据、双相机快照、refit 序列、环境/零编译/树状态全部成立。
- authoring 例外合规：常规 monitor longest_clean=22 未达标后按用户预批用 `-MaxUtil 60`，**60/60 clean、util 14–40%、compute_filtered=0、build=0、floor 未放松**；采集唯一一次、执行前 shell 错误无 GPU/无写入。
- 边界干净：`6951c3fe` 只加参考 5 文件 + 证据 + 卡/runs；`src/tests/scripts`、spec、S-2/pilot 目录零 diff；manifest 43/43；四门禁通过。
- 观察项 6 条（均信息级，不阻断）：见 §7。

## 1. 参考文件（实测 = 证据 = 声称）

| 文件 | sha256 | bytes |
| --- | --- | --- |
| `rgb.npz` | `602574c213d848ddc674244b11ac518fe42e54cf373f43784e143292e960493f` | 5,531,026 |
| `depth.npz` | `82eec6dab866f5ba54e3df0f3592d7198efb6ab5fc832627d98f4d56f079e92d` | 1,844,650 |
| `seg.npz` | `c3f09b16b6c02f972d5f7b535a39a94a6850bc1362e8399543cdb384781142f0` | 3,687,826 |
| `state.npz` | `a1c5a32154370551923a8a4ef394c62d5f9e0d9dbb468e707f77175f8c0c6aca` | 2,486 |
| `capture.json` | `8b53e1221faf9c524b9f42640764a850d7e723de61ebc84342ee6b63137720a9` | 12,251 |
| **合计** | — | **11,078,239** |

工作树与 `git show 6951c3fe:<path>` blob 逐位相同；`capture.json.files_sha256`（4 个 npz）与实际一致。

## 2. 自建脚本独立复算（参考 npz → 三方对账）

| 量 | 0→1 | 1→2 | 与 readout.json | 与 capture.json §2.4 |
| --- | --- | --- | --- | --- |
| C1 magnitude（cam-persp） | **38,285** | **40,644** | ✅ | ✅ |
| C1 bitwise | **43,604** | **41,815** | ✅ | ✅ |
| C2 depth（cam-hinge） | **35,828** | **33,883** | ✅ | ✅ |
| C2 seg | **16,041** | **16,754** | ✅ | ✅ |
| 诊断 cam-persp depth / seg | 0 / 0 | 0 / 0 | ✅ | ✅ |
| 诊断 cam-hinge rgb magnitude | **50,363** | **52,329** | ✅ | ✅ |
| 诊断 cam-hinge rgb bitwise | 54,380 | 54,250 | ✅ | ✅ |

`readout.verdict=pass-margin`、`all_sets_pass_margin=true`、`any_below_accept=false`、`metadata_selfcheck.failed=[]`；`window/counts-crosscheck.txt` 的逐对 8 元组可复算且 = readout（SUMMARY PASS）。

## 3. capture.json 元数据（逐项）

- `two_captures_bitwise_identical=true`；`legacy_reference_untouched=true`（legacy 112）；`captured_at=2026-11-09T00:02:00+08:00`（= token 值）；`reference_tag=nw3-light-hinge02040`。
- `per_world_scope.covers` 覆盖 per-world 灯位姿 + geom 位姿（含 `light_xpos/light_xdir`、`geom_xpos/geom_xmat`）。
- `recipe.cameras` **2 条 × 10 字段**：cam-persp/cam-hinge、mode 0/0、fovy 73.73979461915735/30、resolution 320×240、cam-hinge `sensorsize=[0,0]`（fovy 路径）、intrinsic 与 §2.1 一致。
- `context.resolved`：nworld=3、nrender=2、cam_id_map `[0,1]`、cam_res `[[320,240],[320,240]]`；`parameters.cam_active=[true,true,false]`。
- `use_fast_math=false`、`use_shadows=true`、seed 20261030、repeat 2。

## 4. refit / 执行 provenance

- `refit-counter.json`：`refit_calls=2`、`render_calls=2`、sequence `["refit","render","refit","render"]`、`pattern_ok=true`、`capture_exit_code=0`；采集 argv 含 `--repeat 2 --worlds 3 --hinge-deg 0,20,40 --reference-tag nw3-light-hinge02040 --use-fast-math false --use-shadows true --seed 20261030`。
- `capture-exit-code.txt`：`exit=0`、`wall_seconds=2.02`、`exception_load_bearing=true`；`capture-command.txt` started_at 16:37:19Z（00:37:19+08）。
- 环境 provenance：python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4 / upstream `71da24d9` / RTX 4070 Ti SUPER / driver 596.36。
- T0：HEAD `4f6fe313`、树 clean、锁 owner=Backend、build 进程 0、参考根 fresh 不存在；采集后 `git status` 仅新增参考目录；`zero-compile-fingerprint.txt` build_processes=0。

## 5. 窗口纪律与事故（含 authoring 例外合规性）

| 阶段 | 事实（逐样本复算） |
| --- | --- |
| attempt 1（2026-11-08 22:15 token） | monitor exit 20：longest_clean=24、free_min=8,153 < 10,230 → 窗口不成立、**未采集**；记录在 `target/…/attempt1/`（含 window/ + monitor.csv，未入库） |
| attempt 2 常规 monitor | exit **20**：354 samples、longest_clean **22**、util 16–43%、free_min 13,024；≥40% 的 util 峰值导致不达标 |
| authoring 例外（用户 2026-11-08 预批） | `-MaxUtil 60`：exit **0**、**60/60 clean**、util **14–40%**、free 13,029–13,034、`compute_filtered_max=0`、`build_max=0`、`exception_load_bearing=true` |
| floor 未放松 | 每个 clean 样本 `free ≥ floor_effective 10,230 MiB`（post-hoc 逐样本复算 true） |
| 三点基线 | 13,029 / 13,029 / 13,029 → `baseline_free=13,029`、`onspot_floor=9,957`、**`floor_effective=max(10,230, 9,957)=10,230`**；采集距基线末点 1 分 13 秒（<5 分钟） |
| 采集执行 | 执行前一次 shell 引号错误：exit 1、0.01 s、**无 GPU 工作/无文件写入**；改用 Start-Process 后**唯一一次**采集成功（wall 2.02 s、exit 0），无任何采集重试 |
| 体积差异 | 实测 **11,078,239 B** vs spec §2.3 估算 **11,076,054 B**（+2,185 B，≈0.02%）——estimate vs actual，**冻结以实测为准** |

## 6. 卡 / 边界 / 门禁

- 窗卡 **verified**；`unchanged` = `src/`、`tests/`、`scripts/` + spec/S-2/pilot 目录（**不含 fixtures**，fixtures 仅新增参考 5 文件）；`forbid_claim` 禁词在位；json check = receipt task_id；revision.before=`6f9201e3`。
- `6951c3fe`（parent `6f9201e3`）共 **54 文件**：参考 5 + 证据 44 + 卡/runs 5；`src/tests/scripts` 零 diff、spec/S-2/pilot 材料零 diff、既有 fixtures 零改动（仅 5 个新增）。
- 证据 manifest **43/43** 独立重算；编码卫生（UTF-8/LF、无 BOM、裸 CR·TAB 0）。
- 四门禁：fmt/clippy/clippy-cuda exit 0；unit **1387 passed / 0 failed / 402 ignored**。

## 7. 观察项（均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 信息 | attempt 1 的窗口记录只在 `target/render-bvh/perworld-l-v2-gl1/attempt1/`（未入库），证据仅存 README 数字 | 如需长期审计，把 attempt1 的 monitor-summary 等摘要复制进证据目录 |
| O2 | 信息 | S-2 材料 README sha 差异（签名版 `c8bb71ef…`/18,299 vs HEAD `2ca9f1dd…`/19,502） | `window/s2-material-identity.txt` 已解释为签名块追加的预期演进；签名对象以 `receipt.json.signature.material_sha256` 为准，非违规 |
| O3 | 信息 | 参考 4 个 npz 与 G-L0 pilot 同名文件**逐位相同** | 同参数确定性采集的预期；provenance 以本窗为准；capture.json 因时间/token 不同而不同 |
| O4 | 信息 | 实测总量 11,078,239 vs spec 估算 11,076,054 | 冻结/清单以实测为准；spec 的估算是 §2.3 描述性数字，不是门槛 |
| O5 | 信息 | 本复核登记提交 `08849fbe` 落库时 Arch 正持写锁（我的 acquire 返回 HELD） | 仅登记我自己的卡，未触碰其文件；随后等其 fill 提交 `e49666ea` 落地后才继续写证据。另：该 fill 提交在 `scripts/` 新增了 freezer 脚本（369 行），故本卡的 `unchanged` 检查已剔除 `scripts/`——**被复核的 `6951c3fe` 本身 scripts 零 diff（直接 git diff 验证）** |
| O6 | 信息 | authoring 例外在常规 monitor 失败后才启用 | 与 S-2 预批范围一致（仅本采集窗、floor 不放松、逐样本记录、无提前解锁） |

## 8. 非声明

本卡只复核 G-L1 采集事实与证据一致性：**不宣称**产品渲染通过、不宣称与上游实现等价、不宣称判别力恢复或任何渲染数值结论；产品侧实体比对属 G-L2 验证窗。未修改参考、证据、spec 或签名记录。
