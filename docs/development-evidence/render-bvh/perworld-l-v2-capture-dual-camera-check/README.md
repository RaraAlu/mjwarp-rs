# 双相机 camera_recipe 卡 + 场景卡旧 sha 普查修正 复核（QA 只读）

任务：`2026-11-08-render-perworld-l-v2-capture-dual-camera-check`（owner QA）。日期：2026-11-08。
复核目标：**B（主）** `2026-11-08-render-perworld-l-v2-capture-dual-camera`（verified；登记 `3b984af3` → start `19d5fb51` → 实现 **`b98b39ff`**）；**A（轻核）** 场景/相机卡 O1 证据层 fix **`e8a07167`**。
复核时 HEAD = `ef7a8877`（G-L0 pilot 材料卡；`b98b39ff..HEAD` 只新增 pilot 卡与证据，未触及被复核面）。
边界：只读核验 + 只写本卡证据目录；零 GPU、不采集、不开窗；不改被复核内容。

**结论：PASS（B、A 全部要点通过；0 阻断；3 条信息项）。**

## B. 采集脚本 `camera_recipe` 10 字段（`b98b39ff`）

### B1. 字段、来源与 f64 位型（独立复算）

独立脚本（pinned venv MuJoCo 3.15.0，CPU）直接调用 `perworld_capture_meta.camera_recipe(mjm, stub_rc)`：

| 场景 | active 相机 | records | 逐位比对问题数 |
| --- | --- | --- | --- |
| scene-a-light（3 相机） | `[0, 1]` | 2 | **0** |
| scene-a（2 相机） | `[0]` | 1 | **0** |

- 10 键且顺序固定：`slot, mujoco_cam_id, name, mode, xpos, xquat, fovy, resolution, sensorsize, intrinsic`。
- 位级比对口径：`struct.pack("<d", v)` 逐位相等（不换算、不降 f32）——`xpos/xquat/sensorsize/intrinsic/fovy` 全部 0 差异；`resolution` 与 stub `cam_res` 一致；`name/mode/mujoco_cam_id/slot` 一致。
- 仅 active 相机（`rc.cam_id_map` 语义未变）；cam-hinge `sensorsize=[0,0]`、`intrinsic=[0.01,0.01,0,0]`、走 **fovy 路径**的说明如实（"do not claim intrinsics"）。

### B2. 纯搬与行数

| 文件 | 行数（git-blob） | ≤500 | 说明 |
| --- | --- | --- | --- |
| `scripts/capture_render_perworld.py` | 500 → **487** | ✅ | 删 12 行函数 + 1 处空行；import 行加 `camera_recipe` |
| `scripts/perworld_capture_meta.py` | 244 → **270** | ✅ | 同函数体移入（+docstring）+ 模块 docstring 第 5 条 |
| `scripts/perworld_light_prep.py` | 160 → **160** | ✅ | 未动（sha256 同） |

- `git diff 19d5fb51 b98b39ff -- scripts/`：仅 2 个既有文件（**无新增文件**）；旧函数整段删除、新函数逐行相同仅多 2 个字段（+docstring），**无其它逻辑改动**。
- `camera_recipe`：import 恰 **1** 处（L53）、调用恰 **1** 处（L451，位于 dry-run 提前返回之后 → dry-run 不触达）。

### B3. 自证脚本与模块自检（独立重跑）

- `docs/.../scripts/verify_camera_recipe.py` → **SUMMARY PASS 19/19**（负例 missing-key / extra-key / wrong-value 均按预期拒绝）。
- `python -B scripts/perworld_capture_meta.py` → **SELFCHECK PASS 14/14**。

### B4. 默认行为（同 argv 复跑 + 逐叶对照）

| 用例 | before exit | after exit | 叶子差异 | 判定 |
| --- | --- | --- | --- | --- |
| 默认 `--dry-run --out-dir …/prep-default-dryrun` | 0 | 0 | **仅** `/recipe/script/{sha256,bytes}`（24148→23483 / `6360e5d4…`→`55b91f23…`） | ✅ unexpected=0 |
| 显式 `--scene …/scene-a-light/scene.xml --out-root … --out-dir …/nw3-light-hinge02040 --worlds 3 --hinge-deg 0,20,40 --dry-run` | 0 | 0 | 同上 | ✅ unexpected=0 |
| `--worlds 1` | 1 | 1 | stderr 逐字同（104 B：`--worlds must be >= 2 …`）、stdout 0 B、**out_dir 未创建** | ✅ |

- legacy：默认 **107**、显式 **112**（不变）；显式计划 `model_static.cameras` 3 条、`cam_active_fixed_only=[true,true,false]`。
- 注：对照的 before 侧用执行方卡内 before 日志（同一命令/同一路径），我的复跑只替换 after 侧并自行分类，未采信其分类字段。

### B5. 消费者与证据

- `git grep camera_recipe -- src/ tests/` → **0 命中**；无 8 键封闭消费者；执行方 README 如实标注「dry-run 不产出 camera_recipe，真实 10 字段内容等 G-L1 采集」。
- `b98b39ff` 对 `src/`/`tests/`/`fixtures/` **零 diff**；证据目录 manifest **28/28** 自核、全部文件 UTF-8/LF/无 BOM/裸 CR·TAB 0。

## A. 场景卡旧 sha 普查修正（`e8a07167`）

- 改动面：`artifacts/old-sha-references.txt`（1,737→3,955 B，加计数方法与 pinned-rev 分类表）、`artifacts/scene-identity.json`（`old_sha_registration_refs` **0→1**）、`manifest.txt`、卡 JSON、runs；**scene-camera 的 README/receipt 未动**，`perworld-l-v2-light-criterion/` 本提交零 diff ✅。
- scene-camera 证据 manifest **15/15** 重算通过 ✅。
- 四单位普查独立复算（`git grep -a -o -n -E '4db66e2c[0-9a-f]*' <rev>`）：

| rev | total | historical | registration | card_evidence | review_artifacts |
| --- | --- | --- | --- | --- | --- |
| `3d546fbe` | 21 occ / 21 lines | 9 occ / 9 lines / 7 文件（full64 8 + abbr 1） | 1 occ / 1 line（full64 1） | 11 occ / 11 lines / 7 文件（full64 9 + abbr 2） | — |
| `d38661a5` | 29 occ / 28 lines | 同上 | 同上 | 同上 | 8 occ / 7 lines / 3 文件（full64 1 + abbr 7） |

与 `old-sha-references.txt` 记录**逐项一致**；分类规则、pinned revision（避免本文件自计）与「README/receipt 保持 old-sha-only full64 口径（8+1）」的调和说明均可复核 ✅。

## 观察项（均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 信息 | `scripts/capture_render_perworld.py` / `perworld_capture_meta.py` 未在 `.normify/modules` 登记为 module source（`git grep` 0 命中）——**既有状态**，本卡未新增文件故规则上无需登记；若团队要求「所有脚本归模块」属另一治理议题 | 知悉即可 |
| O2 | 信息 | dry-run 不触达 `camera_recipe`，默认行为对照证明的是脚本其余路径不变；10 字段真实渲染内容须等 G-L1（执行方已如实标注） | 知悉即可 |
| O3 | 信息 | 我上一轮口述「历史 8 处 occurrence」采用二进制 grep 口径；A 卡四单位普查给出 historical 9 occurrences（8 full64 + 1 abbreviated，`light-prep/README.md:31`）。以 A 的精确口径为准，我的旧结论中「8 处 full64 未回改」仍成立 | 已在 A 卡 artifact 内调和，无需再改 |

## 门禁

fmt / clippy / clippy-cuda 通过，**unit 1359 过 / 0 败**（本卡独立复跑，记录见 `.normify/runs/2026-11-08-render-perworld-l-v2-capture-dual-camera-check/`）。

## 非宣称

- 本复核只判两片证据与脚本改动是否按已签 spec v3 §2.4/§11.1-6③ 正确落地；不宣称判别力恢复、采集有效、per-world 灯验收、上游等价、任何渲染数值结论。
- 不构成 G-L0/G-L1/G-L2 启动或通过；未改被复核脚本、fixture、证据与任何卡状态。
