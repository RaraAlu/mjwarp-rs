# per-world 采集窗独立复核（attempt1 留档 + 窗口期修复 + attempt2）（QA，只读）

任务：`2026-10-30-render-perworld-capture-check`
被核：`2026-10-30-render-perworld-capture`（主卡 verified）
被核 pin：**`3478bbab`**（修复链 `e8339052` authoring + `787eb48a` 材料）；窗口基线 `8e40c7ea`；核 start 时 `revision.before = e0988285`。
**结论：pass / 阻断 0 / 需修 0 / 观察 8。**
本卡只读：零 GPU、零窗口、零设备执行；未重跑采集（GPU）——dry-run 用 pinned 环境离线复跑，产物用 committed blob 独立解析。

## 0. 一句话

attempt1 的失败是**脚本场景假设错**（把 FREE 当 scalar joint、且基座用全零 qpos），不是采集/渲染错：脚本在场景自检阶段 exit 1、零渲染零写盘，失败记录完整保留；窗口期修复（scalar 只取 SLIDE|HINGE、基座用模型 `qpos0`、dry-run 加静态模型自检）我逐条核过并与 dry-run 证据**逐字段一致**；attempt2 在**同一 authoring 例外**下 60/60 clean、采集 exit 0、两跑位级相等、legacy 79 未动。例外 monitor 的 `util` 全部 ≤39 ⇒ `exception_load_bearing=false`，证据**照实记录为 `exception_effectively_not_load_bearing=true`、没有声称 60 门槛生效/常规通过**。

## 1. 复核方法

- 脚本身份/修复点：`git cat-file` 读 committed blob + 文本断言；dry-run 用 pinned python（`CUDA_VISIBLE_DEVICES=99`）离线复跑并与证据 `materials/dryrun-plan-attempt2.json` 逐字段比对（唯一允许差异：out_dir）。
- 产物：npz 用 pinned numpy 解包，核对数组名/形状/dtype/有限性、自算 sha，与 `capture.json.files_sha256`、`materials/products.txt` 三方对拍。
- monitor：独立解析 attempt1/attempt2 CSV（samples/clean/streak/ge40/ge60/free/filtered/build）。
- 留档：target/ 窗口副本与证据副本逐字节对拍；零 Rust 编译用 `target/debug` 的 mtime 扫描。
- 重算脚本：`recompute.py`（8 项 check → `recompute.json`，problems 0）。

## 2. 结果（8 项 check，全过）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `script.identity_and_fix` | ✓ | 脚本 **24,008 B / `67a760fa…` / 499 行 / 非 ASCII 0 / CR 0**；`scalar` 只取 `(2,3)`=SLIDE|HINGE（不再把 FREE 计入）、要求恰好 1 个 HINGE、`FREE/BALL are not scalar` 字样；基座 `np.array(mjm.qpos0)`、qpos 地址越界检查、输出 float32 重复；dry-run 有 `model_static`（njnt/nq/jnt_type/jnt_qposadr/qpos0/cameras）；两跑比较与 out-dir 非空守卫仍在 |
| 2 | `dryrun.reproduced` | ✓ | 我复跑 dry-run exit 0、未建 CUDA、未写盘；与证据计划**逐字段 0 diff（仅 out_dir 不同）**：`njnt=2`、`nq=8`、`jnt_type=[0,3]`、`jnt_qposadr=[0,7]`、`qpos0` free 基座非零、`world_qpos` 形状 **(3,8)**、hinge 弧度 **[0, 0.3491, 0.6981]**、`cam_active_fixed_only=[true,false]`、`legacy_manifest_entries=79`；note 明写「per-world light poses are not claimed」 |
| 3 | `products.and_scope` | ✓ | 5 个产物 sha 与 `capture.json.files_sha256`、`products.txt` 全等；`task_id=2026-10-30-render-perworld-capture-authoring`、`kind=render-perworld-capture`、`nworld=3`、`hinge_deg=[0,20,40]`、`use_fast_math=false`、`use_shadows=true`、`repeat=2`、脚本 sha `67a760fa…`、场景 sha `a9f1f0a3…`、`legacy_reference_files=79`；`per_world_scope.does_not_cover = per-world light poses`；数组：`rgb_w{0,1,2}_cam0 [240,320,3] f32`、`depth_w*` 同尺寸、`seg_w*` int32、`state.qpos (3,8)`、`geom_xpos (3,7,3)`、`light_xpos/xdir (3,3,3)` |
| 4 | `boundary.legacy_unchanged` | ✓ | `fixtures/render-bvh` 排除 `scene-a/perworld/` 窗前后均 **79** 条、列表相同、**0 字节改动** |
| 5 | `exception.monitor_honest` | ✓ | attempt2 CSV 我重算：**60/60 clean**、streak 60、util **2–39**、**ge40=0**、ge60=0、free_min 12,907、filtered/build=0；`clean` 列与 `util<60` 规则逐行吻合；`exception_load_bearing=false`（重算与 receipt 一致）且 **`exception_effectively_not_load_bearing=true`**、receipt 里明写「applied 60 threshold carried no load; recorded as-is, not claimed as a regular threshold pass」；命令逐字 `-MaxUtil 60` |
| 6 | `attempt1.retained` | ✓ | `attempt1/capture-failure.txt` 写明 exit 1、停在脚本场景自检、`expected exactly one scalar hinge joint`、`products_written=none`；target/ 的 attempt1 失败记录、monitor CSV、快照、recompute 与证据副本**逐字节相同**；attempt1 例外 monitor 60/60（util 4–48、12 个 ≥40 仍 clean，load-bearing=true）也留档 |
| 7 | `scope.materials_manifest` | ✓ | `8e40c7ea..3478bbab` 改动 = 脚本 1 + fixtures 5（仅 `scene-a/perworld/nw3-hinge02040/**`）+ 证据/材料/卡；`src/tests` **0**；新脚本 sha 在窗口材料 README/window-plan/expected-products 与 authoring receipt **4 处一致**；`perworld-capture-run` manifest **14/14** 全等 |
| 8 | `card.checks_and_validate` | ✓ | 主卡 `normify check` 通过；`validate --fingerprints` = **0 错误 / 5 警告** |

逐项原始数字见 `recompute.json`；我复跑的 dry-run 在 `materials/dryrun-qa.json`（与证据仅 out_dir 不同）。

## 3. 观察 8（记录级接受）

- **O1 例外「未承载」照实记录，口径正确**：attempt2 的 util 全部 ≤39（`ge40=0`），所以 60 门槛没有实际起作用；证据同时给 `exception_load_bearing=false` 与 `exception_effectively_not_load_bearing=true`，并在 README 明确「不是常规门槛通过」。**重要的后续含义**：U4 不能把「本窗例外通过」当成「例外被验证/可复用」，U4 仍需按常规 `util<40` 或另签。
- **O2 attempt1 留档完整**：根因、退出码、失败阶段、`products_written=none`、monitor CSV、快照、重算全在；target 的 attempt1 monitor CSV 实际在窗口根目录（`monitor-pre-window-attempt1.csv`）而不是 `attempt1/` 下，不影响留档，但建议以后统一目录。
- **O3 capture.json 缺 run2 的 hash**：per-world 脚本的 `capture.json` 只有 `two_captures_bitwise_identical=true` 布尔（无 `context_repeat` 那样的两跑 sha 列表），所以两跑相等只能靠脚本自报 + 代码路径；SDF 脚本有 run1/run2 双 hash、可三方对拍。建议后续给 per-world 也补 run2 hashes（U4 前值得做）。
- **O4 窗口材料目录无 `manifest.txt`**：`perworld-capture-window-material/` 只有 README/materials/receipt（4 文件），与 SDF 窗口材料目录（有 manifest）不一致；建议补一份 manifest。
- **O5 窗口期改脚本是允许且已同步的**：`e8339052` 改脚本时同步刷新了 authoring 证据 errata（nq 1→8、qpos (3,1)→(3,8)、legacy 69→79）与 `dryrun-plan.json`（4,917 B / sha `79c1f0f2…`），我核对与 attempt2 的 dryrun-plan-atempt2 逐字节相同；U4 冻结前应以新 sha `67a760fa…` 为准。
- **O6 本窗没有自己的常规 util<40 失败记录**：按 Chief 指令直接走共享例外（引用 SDF 窗的常规失败记录），例外签字 scope 已覆盖本窗；证据里写清了这一点，口径可接受。
- **O7 零 Rust 编译**：窗口时段 `target/debug` **0 文件改动**（最新 10-30 18:02），只有 `target/render-bvh/perworld-cache` 的 warp kernel cache 更新；GPU 事实（torch/warp 初始化、sm_89 RTX 4070 Ti SUPER）在 capture.log 里。
- **O8 产物写盘严格限定**：只写 `fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/`（5 文件），legacy 79 逐条不变；`state.npz` 明确只保存 qpos/geom 位姿与灯数组（灯数组「仅为完整性捕获、不宣称 per-world 灯位姿」）。

## 4. 边界与不宣称

- 本卡只读：零 GPU、零窗口、零设备执行；未重跑采集；不改 `src/tests/fixtures/scripts`、不改他人证据。
- 确认主卡口径：本窗是 **采集（authoring），不是验收**；不宣称 per-world 灯位姿、不宣称 per-world 渲染能力已实现、不宣称任何渲染数值结论（设备侧 per-world 对比留给后续 impl S1/S2）。
- 确认例外 scope 边界：authoring-only `util<60` 例外只覆盖两个采集窗，**U4 不得复用**。

## 5. 限制

- 未重跑 GPU 采集：两跑相等只能由脚本自报布尔 + 代码路径 + 产物 sha 一致性间接确认（capture.json 缺 run2 hash，见 O3）。
- 窗口期脚本被修改：我核的是 attempt2 实际使用的 sha `67a760fa…`；attempt1 用的旧 sha `971cb8b1…` 只作历史记录。
- 未对 rgb/depth/seg 数值语义下判定（属于后续设备侧对比）。
