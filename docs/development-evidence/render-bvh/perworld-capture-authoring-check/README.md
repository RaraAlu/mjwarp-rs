# 逐 world 渲染采集 · 离线 authoring · QA 独立复核

任务卡：`2026-10-30-render-perworld-capture-authoring-check`（priority 55，Owner QA）
被核对象：`2026-10-30-render-perworld-capture-authoring`（Arch，**verified**）
被核提交：实现 `28822740`（脚本 + 证据 + 主卡，5 文件）→ 收口 `a4d25ead`（主卡 verified + 本复核卡预登记）；主卡 `revision.before = 84ef186f`
复核时 HEAD：`b4e050d1`。**只读**：未开 GPU 窗口、未做任何采集、未写 `fixtures/` 参考、未改产品代码/证据；按卡面允许用 pinned python 离线复跑 `--dry-run`。

---

## 1. 结论

**verdict：pass / 阻断 0 / 需修 0 / 观察 7。**

| 复核点 | 我的独立结果 | |
| --- | --- | --- |
| 脚本事实 | `scripts/capture_render_perworld.py`：**23 521 B / sha256 `602e36da…` / 497 行 LF（≤500）/ 非 ASCII 字节 0 / pinned python `ast.parse` 通过** | ✓ 与 receipt 逐项一致 |
| pinned 环境 | python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4；python exe sha `feeec361…`；upstream `71da24d9…` 且 **clean**；scene `a9f1f0a3…`（4 639 B） | ✓ |
| 能力证据 | `put_data` 签名含 `nworld`；`Data.qpos: array("nworld","nq",float)`、`geom_xpos: array("nworld","ngeom",wp.vec3)`、`geom_xmat: array("nworld","ngeom",wp.mat33)`；`create_render_context(mjm, nworld=1, …)`；`get_rgb` 文档 `shape (nworld, height, width)`；`render.py` 按 `geom_xpos_in[worldid, gi]` 采样 | ✓ 全部可复算 |
| dry-run 复跑 | 我重跑 `--dry-run`：**exit 0**，stdout 与 `materials/dryrun-plan.json` **逐字段完全一致**（非手写） | ✓ |
| 无 device / 无写盘 | `CUDA_VISIBLE_DEVICES=99` 下仍 exit 0 且输出一致；`--out-dir` 前后都不存在；`fixtures/render-bvh` 全量 sha 清单前后相同；legacy manifest 我独立算 = **69** | ✓ |
| 零越界 | `84ef186f..a4d25ead` 的 `fixtures/src/tests/Cargo.*` diff **空**；`scripts/` 只有 1 个新增（本脚本）；仓库无 `scene-a/perworld/` 产物 | ✓ |
| 证据 | 5 文件（脚本 + README/receipt/materials + 主卡）committed blob **全部 LF 无 BOM**；receipt `/task_id` 正确；禁词 0 | ✓ |
| 范围诚实 | README §4、receipt `not_claimed`、dry-run `per_world_plan.note`、脚本 `capture.json.per_world_scope` 四处都写明「只覆盖 per-world geom 位姿、灯位姿不宣称」 | ✓ |

---

## 2. 脚本事实与静态防护

| 项 | 值 |
| --- | --- |
| 路径 | `scripts/capture_render_perworld.py`（新增，A 状态） |
| 字节 / 行（LF） | 23 521 / **497**（≤500 上限） |
| sha256 | `602e36daab79fd4dc269d9f935adc916ec8057b742e023d270541404d477fb36` |
| 非 ASCII 字节 | **0** |
| 语法 | pinned python 3.12.13 `ast.parse` → `PY_SYNTAX_OK` |

静态防护（我读代码逐条核，均有实现）：
- `--out-dir` 必须 `is_relative_to(OUT_ROOT)`（`fixtures/render-bvh/scene-a/perworld/`）；
- 目标目录已存在且非空即失败（参考只增不改）；运行前后 `legacy_manifest()` 对拍，不等则失败；
- 真正采集分支才 `check_environment(want_cuda_device=True)` 并在其中校验 GPU 名/arch；`--dry-run` 传 `want_cuda_device=False`，全脚本**唯一** `wp.get_device(` 调用在该分支内；
- 写盘只有 `out_dir` 下的 npz 与 `capture.json`（`tmp_second` 为二次校验临时目录，结束即删）；
- `capture.json` 写入 `reads_product_tree: False` / `authoring_only: True` / `product_python_dependency: False` / `per_world_scope.does_not_cover = "per-world light poses …"`；
- 默认值：`--worlds 3`、`--hinge-deg "0,20,40"`、`--use-fast-math false`、`--use-shadows true`、`--seed 20261030`、`--repeat 2`（与证据声明一致）。

派生关系：`scripts/capture_render_hfield.py`（R05-S3a）存在；本次 diff 对新脚本以外 `scripts/` **零改动**（R05/R09 脚本与既有参考不受影响）。

## 3. pinned 环境、上游、场景

我用 pinned python（`target/mjwarp-parallel/venv/Scripts/python.exe`）实测：

- 版本：**python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4**（与 receipt、dry-run 材料三处一致）；
- python 可执行文件 sha256 = `feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77`（等于 receipt）；
- upstream checkout `target/mjwarp-parallel/upstream`：`HEAD = 71da24d956378a87a703b6e1442b13aec0c4ac29`、`git status` **干净**；
- scene `fixtures/render-bvh/scene-a/scene.xml`：4 639 B、sha256 `a9f1f0a35464f4df17f191e1d68431bda0f17648273578e58fa08f84b0449289`；
- 运行期能力：`inspect.signature(put_data)` 首段含 `nworld`；`create_render_context` 前三个参数为 `(mjm, nworld, cam_res)`；`get_rgb/get_depth/get_segmentation` 均存在。

## 4. 能力证据复算（pinned 上游源码）

| 声明 | 我在 pinned 源码里核到的事实 |
| --- | --- |
| `put_data(mjm, mjd, nworld=N)` | `mujoco_warp/_src/io.py::put_data` 签名含 `nworld`；运行期签名一致 |
| `Data.qpos = (nworld, nq)` | `types.py`：`qpos: array("nworld", "nq", float)`，docstring 亦写 `(nworld, nq)` |
| `Data.geom_xpos = (nworld, ngeom, 3)` | `types.py`：`geom_xpos: array("nworld", "ngeom", wp.vec3)`，docstring `(nworld, ngeom, 3)` |
| `Data.geom_xmat = (nworld, ngeom, 3, 3)` | `types.py`：`geom_xmat: array("nworld", "ngeom", wp.mat33)`，docstring `(nworld, ngeom, 3, 3)` |
| `create_render_context(mjm, nworld=N, …)` | `render_util.py::create_render_context(mjm: mujoco.MjModel, nworld: int = 1, cam_res=…, …)` |
| `get_rgb` 输出 `(nworld, H, W)` | `render_util.py::get_rgb(rc, camera_index, rgb_out)` 文档：`rgb_out … shape (nworld, height, width)` |
| 上游按 `geom_xpos_in[worldid, gi]` 采样 | `render.py` 多处 `geom_xpos_in[worldid, gi]`（我抓到 3 处样本，非注释） |

⇒ 本片「不新增上游能力、全部用 pinned 现成 API」的可行性结论成立。

## 5. dry-run 离线复跑（不建 device、不写输出）

我执行：

```text
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/capture_render_perworld.py \
    --dry-run --out-dir fixtures/render-bvh/scene-a/perworld/nw3-hinge02040
```

结果：
- **exit 0**；stdout JSON 与 `materials/dryrun-plan.json` **逐字段相等**（`out_dir`、`recipe.script.sha256`、版本、`cam_active_fixed_only=[true,false]`、`camera_modes=[0,3]`、`legacy_manifest_entries=69` 等全部一致）；
- 复跑前后 `--out-dir` **均不存在**；`fixtures/render-bvh` 全目录 sha256 清单**前后完全相同**；
- 换 `CUDA_VISIBLE_DEVICES=99` 再跑：仍 exit 0、输出一致 ⇒ 该路径**不依赖可用 CUDA device**（代码层同一结论：`wp.get_device` 只在真实采集分支）；
- legacy manifest 我按「`fixtures/render-bvh` 排除 `scene-a/perworld/`」独立计数 = **69**（与 receipt 一致）。

## 6. 零越界与范围诚实

- `git diff 84ef186f..a4d25ead -- fixtures src tests Cargo.toml Cargo.lock build.rs` = **空**；`scripts/` 仅 `A scripts/capture_render_perworld.py`；仓库内无 `scene-a/perworld/` 产物目录。
- 范围口径四处自洽（见 §1 表末行）：参考只覆盖 **per-world geom 位姿**（`geom_xpos/geom_xmat`）；`light_xpos/light_xdir` 只为完整性落盘，设备路径仍单 world 索引灯位姿，因此**灯位姿不宣称**。
- 本片不含任何采集数值，也未写 `fixtures/` 参考。

## 7. 证据

- 5 文件（`scripts/capture_render_perworld.py`、`perworld-capture-authoring/{README.md,receipt.json,materials/dryrun-plan.json}`、主卡 JSON）：committed blob **全部 LF 无 BOM**；脚本与 README/receipt 工作区亦 LF；`materials/dryrun-plan.json` 工作区为 CRLF（作者重定向输出产物），**committed blob 为 LF**（对账以 blob 为准）。
- `receipt.task_id` = `2026-10-30-render-perworld-capture-authoring` ✓；禁词扫描（码点构造 3 词，范围 4 个证据文件）**0 命中**。
- 主卡 `status=verified`、`revision.before=84ef186f`、有 `closed_at`；本复核卡 `in_progress` 且 start pin 到 `b4e050d1`。

## 8. limits

- 本片为**离线 authoring**：没有跑任何 GPU 采集；`--repeat 2` 两跑逐字节相等、`legacy_reference_untouched`、GPU 名/arch 校验都只在真实采集分支执行，**本轮未验证**（属窗口内事项）。
- dry-run 证据是**单机口径**（绝对路径、python exe sha、platform），跨机复跑需要同一 pinned env。
- 未复核后续「采集参考 → 判定/容差纳入验收」的流程（主卡明确另走精度契约与窗口签字）。

## 9. 观察（7）

| # | 级别 | 观察 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议 | 脚本内 `TASK_ID = "2026-10-30-render-perworld-capture"`（少 `-authoring`），与主卡/证据 task_id 不同；`capture.json` 里将写入较短 id | 确认口径：若为采集产物 id 有意区分，建议 README 注明；若应跟随主卡，窗口前改一处字符串即可 |
| O2 | 建议 | README 可行性表只给文件级出处，无行号 | 补行号（我复核算出的锚点：`types.py` 的 qpos/geom_xpos/geom_xmat 声明、`render_util.py::create_render_context/get_rgb`、`render.py` 的 `geom_xpos_in[worldid, gi]`），下次复核更快 |
| O3 | 记录 | `legacy_manifest()` 排除整棵 `scene-a/perworld/` 子树（不是只排除本次 `--out-dir`）：将来同子树存在多份 perworld 参考时，旧 perworld 参考不在「只增不改」对拍范围 | 当前子树为空属前向设计；若要多参考并存，建议收窄 exclude 到本次 out_dir 或另建对拍 |
| O4 | 记录 | `materials/dryrun-plan.json` 工作区 CRLF（重定向产物），committed blob LF | 接受；核对以 blob 为准（同前几卡口径） |
| O5 | 记录 | dry-run 记录 `legacy_manifest_entries=69` 但未记录这 69 条的清单 sha（只在真实采集时前后对拍） | 可接受；窗口内采集会落 capture.json 的 `legacy_reference_untouched` |
| O6 | 记录 | 主卡 `revision.before=84ef186f` 的 diff 范围包含其间其他卡的 QA 提交（我的 154 复核证据/卡），故「只含脚本/.normify/证据」按路径类别成立；按本卡自己的文件应以 `28822740` 为准 | 记录即可；不影响本卡结论 |
| O7 | 记录 | 证据为单机口径（绝对路径/平台），跨机不可逐字节复跑 | 接受（pinned env 内可复现） |

---

## 10. 本卡文件

| 文件 | 内容 |
| --- | --- |
| `recompute.py` / `recompute.json` | **10 项**独立重算（脚本事实/静态防护/环境/能力/legacy 计数/dry-run 逐字段对账/无 device/证据/blob 对账/零越界/卡状态），problems=0 |
| `logs/recompute.log` | 上述脚本 stdout |
| `materials/dryrun-reproduce.txt` | dry-run 复跑命令、exit、与材料对账结论、无 device/无写盘记录 |
| `manifest.txt` | 本卡文件 sha256/字节/路径（不含自身） |
