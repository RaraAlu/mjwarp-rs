# perworld L v2 采集脚本双相机：`camera_recipe` 补 `sensorsize`/`intrinsic`（spec v3 §2.4 / §11.1-6③；零 GPU）

- 任务：`2026-11-08-render-perworld-l-v2-capture-dual-camera`（owner Backend、p60、P8-渲染与BVH）
- 定位：**工具/元数据层**（采集脚本相机快照补全），零 GPU、不采集、不开窗、不建 CUDA、不落参考；
  真实渲染上下文里的 10 字段**实拍**内容等 G-L1 采集，本卡证据只到「CPU stub 自证 + 静态断言」。
- 前置：spec v3 已签生效（`bb064fe0…`/39,379 B；签名留证 `023c69fa`）、场景/相机改卡 verified（`3d546fbe`）、
  采集元数据卡 verified（`b0af1be4`）、P0-1 refit 卡 verified（`e34df3b7`）。
- 基线修订：`19d5fb51`（本卡 start 过渡；本文所有 before 值取自该提交）。

## 1. 结论与改动

### 1.1 `camera_recipe` 从 8 字段补到 10 字段（`scripts/perworld_capture_meta.py`）

仅 active 相机（`rc.cam_id_map`；旧 `slot` 标签语义不变），键序固定：

| # | 键 | 来源（原样，不换算） |
| --- | --- | --- |
| 1 | `slot` | `rc.cam_id_map` 枚举序 |
| 2 | `mujoco_cam_id` | `rc.cam_id_map` 值 |
| 3 | `name` | `mjm.camera(cam_id).name` |
| 4 | `mode` | `mjm.cam_mode[cam_id]` |
| 5 | `xpos` | `mjm.cam_pos[cam_id]` |
| 6 | `xquat` | `mjm.cam_quat[cam_id]` |
| 7 | `fovy` | `mjm.cam_fovy[cam_id]` |
| 8 | `resolution` | `rc.cam_res[slot]` |
| 9 | **`sensorsize`** | `mjm.cam_sensorsize[cam_id]`（新） |
| 10 | **`intrinsic`** | `mjm.cam_intrinsic[cam_id]`（新） |

pinned MuJoCo 3.15.0 解析现 fixture 的**模型原值**（f64 字段承载 f32 量化值；只复制、不换算）：

| 相机 | `sensorsize` | `intrinsic` | `fovy` | `resolution` |
| --- | --- | --- | --- | --- |
| `cam-persp` | `[0.035999998450279236, 0.024000000208616257]` | `[0.017999999225139618, 0.01600000075995922, 0.0, 0.0]` | `73.73979461915735` | `[320, 240]` |
| `cam-hinge` | `[0.0, 0.0]` | `[0.009999999776482582, 0.009999999776482582, 0.0, 0.0]` | `30.0` | `[320, 240]` |

- **fovy 路径如实记录**：`cam-hinge` 的 `sensorsize=[0,0]`、`intrinsic` 为模型默认值 `[0.01,0.01,0,0]`，
  即它走 **fovy 路径**；证据与代码注释均**不声称它有 intrinsics**。
- 说明：`intrinsic` 上表数值即 Arch 预研给的 `[0.018,0.016,0,0]` 在 f32→f64 下的精确模型原值
  （`cam-persp`），`sensorsize` 同理；本卡按「原样模型数组」记录，不四舍五入成十进制。

### 1.2 500 行天花板处置（建议 ⓑ）：纯搬到已挂模块，不新建模块文件

| 文件 | 行数 before | 行数 after | 增量 | 说明 |
| --- | --- | --- | --- | --- |
| `scripts/capture_render_perworld.py` | **500** | **487** | −13 | 删除 12 行函数 + 1 行重复空行（保持既有单空行函数间隔）；import 行仅加名字不加行 |
| `scripts/perworld_capture_meta.py` | **244** | **270** | +26 | 搬入函数 22 行（含 docstring）+ 模块 docstring 第 5 项 4 行 |
| `scripts/perworld_light_prep.py` | 160 | 160 | 0 | 未改 |

- 函数体逻辑与返回结构**纯搬**（除按 spec v3 新增 2 字段）；两文件均 ≤500。
- `git diff` 只有两个脚本：主脚本 = 1 行 import 变化 + 12 行函数删除；meta = docstring 第 5 项 + 函数搬入
  （`artifacts/scripts.diff`，UTF-8/LF）。
- 行数用 `git cat-file` 换行口径；before 取自 `19d5fb51`，after 为工作树。

## 2. 静态断言（`scripts/verify_camera_recipe.py`，零 GPU）

日志：`logs/verify-camera-recipe.log` → **SUMMARY PASS 19/19**（含 `exit_code=0`）。要点：

- `camera_recipe` 只在 `perworld_capture_meta.py` 定义 1 次，主脚本内**无定义**；
- 主脚本恰 **1 次 import**（来自 `perworld_capture_meta`）、恰 **1 次调用**；
- 主脚本 append 的字典键 = **10 字段且顺序固定**（AST 读 `ast.Dict.keys`）；
- meta 模块顶层 import 仍只有 `__future__/datetime/pathlib/subprocess/sys`（**无** warp/mujoco/numpy 等产品面依赖）；
- `19d5fb51 →` 工作树 diff 仅两个脚本、`--diff-filter=A` 为空（**无新增模块文件**）；
- 行数断言 500→487、244→270，均 ≤500。

## 3. CPU stub 自证（pinned venv MuJoCo，零 GPU）

- 两场景各一组：`scene-a-light`（3 相机、active 2：persp/hinge）与 `scene-a`（2 相机、active 1：persp）；
- stub `rc` = `cam_id_map` + `cam_res`（`NpWrap` 模拟 `.numpy()`），调真实 `camera_recipe`；
- 断言：每台 active 相机**恰 10 键且键序固定**；`xpos/xquat/fovy/sensorsize/intrinsic` 与 `mjm.cam_*`
  数组**逐位（struct.pack f64）一致**；`resolution` 与 stub `cam_res` 一致；`name/mode/mujoco_cam_id/slot` 一致；
- 负例 3 条（缺键 / 多键 / 值不符）被同一断言器**按预期拒绝**；
- 日志同 `logs/verify-camera-recipe.log`（其中打印了两场景的完整 10 字段 JSON）。

## 4. 默认行为不变（dry-run 逐叶对照，零 GPU）

原始日志：`logs/before-*.json` / `logs/after-*.json`；机器可读 diff：`artifacts/dryrun-diff.json`（**unexpected=0**）。

| 用例 | before exit | after exit | 差异 | 结论 |
| --- | --- | --- | --- | --- |
| 默认 `--dry-run`（默认 out-root/out-dir） | 0 | 0 | 仅 `/recipe/script/{sha256,bytes}`（主脚本被编辑）；removed=0、added=0 | ✅ |
| 显式 `--scene …/scene-a-light/scene.xml --out-root … --out-dir …/nw3-light-hinge02040 --worlds 3 --hinge-deg 0,20,40 --dry-run` | 0 | 0 | 同上仅 script 身份 | ✅ |
| `--worlds 1 --dry-run` | 1（按设计拒绝） | 1 | stderr **逐字相同**、stdout 空、**不建 out_dir** | ✅ |

- 3 相机事实不变：`model_static.cameras = [[0,0,"cam-persp"],[1,0,"cam-hinge"],[2,3,"cam-track"]]`、
  `cam_active_fixed_only = [true,true,false]`；`legacy_manifest_entries` 默认 **107**、显式 **112**（前后一致）。
- `--worlds 1` stderr：`perworld capture environment/reference mismatch: --worlds must be >= 2 (this is a per-world reference)`。
- **如实标注**：dry-run 不执行渲染路径，因此**不产出 `camera_recipe`**；10 字段的真实取值与上述 CPU stub 一致，
  但「渲染上下文中实拍产生的 `capture.json.recipe.cameras`」要等 **G-L1 采集**。

## 5. 消费者盘点（详见 `artifacts/consumer-inventory.md`）

- `camera_recipe`：本卡生产/消费 = 主脚本 1 import + 1 调用、meta 1 定义；`capture_render_hfield.py` /
  `capture_render_sdf.py` 的同名函数是**独立脚本副本**（不在本卡范围，未改）；`tests/`、`src/` **0 处引用**。
- perworld 参考的消费者：`tests/render_per_world.rs` 只读 `*.npz` + 绑定 npz sha256；`guards-sdf-validate.json`
  把 `capture.json` 当冻结输入做 sha256/bytes 绑定（哈希绑定，非 schema）；**没有代码解析
  `capture.json.recipe.cameras`**。
- 无任何消费者断言「8 键封闭集合」→ 8→10 为纯增量；未触发「tests/ 需改→停手报 Chief」条件，
  `unchanged tests/`、`unchanged src/` 成立。

## 6. 边界与非声明

- 范围：只改 `scripts/capture_render_perworld.py`、`scripts/perworld_capture_meta.py` + 本卡证据；
  `unchanged fixtures/`、`unchanged src/`、`unchanged tests/`；不改判据/阈值/参考/产品代码。
- 四门禁 fmt/clippy/clippy-cuda/unit 全过；模块自检 `logs/module-selfcheck.log` → **SELFCHECK PASS 14/14**。
- **依赖**：G-L1 采集窗材料依赖本卡 verified + QA pass；pilot/采集窗/验证窗均另卡，本卡不开窗。
- **非声明**：不宣称判别力恢复、不宣称采集是否有效、不宣称 per-world 灯验收、不宣称与上游实现等价、
  不宣称任何渲染数值结论；也不宣称 10 字段的**实拍**内容已产生（本卡只有 CPU stub + 静态断言）。

## 7. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `manifest.txt` | sha256 + bytes 自核清单（首行 `#`、`# base dir:`、不自列） |
| `artifacts/scripts.diff` | 两个脚本的完整 diff（before/after） |
| `artifacts/line-counts.txt` | 行数/sha256 前后对照（git-blob 换行口径） |
| `artifacts/dryrun-diff.json` | dry-run 逐叶对照（unexpected=0 + allowed 清单） |
| `artifacts/worlds1-behavior.json` | `--worlds 1` 前后行为（exit/stderr/stdout/out_dir） |
| `artifacts/consumer-inventory.md` | 消费者盘点与判定 |
| `logs/before-*.json`、`logs/after-*.json` | 三个 dry-run 用例的原始 stdout/stderr |
| `logs/before-runs.json`、`logs/after-runs.json` | 用例 exit code 与 out_dir 状态 |
| `logs/before-meta.txt`、`logs/after-meta.txt` | 前后脚本 sha256/bytes/行数、HEAD、场景 sha |
| `logs/verify-camera-recipe.log` | 静态断言 + CPU stub 自证（SUMMARY PASS 19/19） |
| `logs/diff-dryrun.log` | dry-run 对照（SUMMARY PASS 17/17） |
| `logs/module-selfcheck.log` | 模块自检（SELFCHECK PASS 14/14） |
| `scripts/verify_camera_recipe.py` | 静态断言器 + CPU stub 自证器（含 3 条负例） |
| `scripts/diff_dryrun.py` | dry-run 逐叶对照器（生成两个 artifacts） |

> 注：dry-run 原始日志由 Python 在 Windows 重定向下输出为 CRLF，落库前已统一归一化为 LF（仓库证据卫生要求，除换行符外不改内容）；manifest 记录归一化后的 sha256。

