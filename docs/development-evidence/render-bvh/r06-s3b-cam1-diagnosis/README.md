# R06-S3b 诊断：attempt 3 `cam1 rgb max_byte_diff=255` 的根因（CPU/只读，零 GPU）

任务：`2026-11-04-render-r06-s3b-cam1-rgb-diagnosis`（p77，owner Backend；depends 窗口卡 `2026-11-04-render-r06-s3b-flex-validate`）。
输入：窗口 attempt 3 证据 `2f346c43`（**VALID_ROUND，判据性红**，`cam1 rgb max_byte_diff=255`；`cam0` rgb 先过、`depth/seg/两跑 sha` 未评估、`cam2`（cam-flex，654 flex 像素）未跑到）。
本片**零 GPU、不开窗、不请求第 4 次窗口**；不改判据/容差/参考、不动 `fixtures/` 与 `src/`；四门禁。

## 0. 结论（先给判定）

**不是产品映射/写入缺陷，也不是 flex 数值红；根因是 harness 的相机位姿来源缺陷（记 D5）。**

- 参考是用上游 **data 位姿**（`d.cam_xpos` / `d.cam_xmat`，即 forward 之后的相机世界位姿）渲染的：
  上游 `mujoco_warp/_src/render.py:1491` 的 `render(m, d, rc)` 把 `d.cam_xpos`/`d.cam_xmat` 作为内核输入
  （`render.py:1550`）；采集脚本 `scripts/authoring/capture_render_flex.py:226` 调用的正是 `mjw.render(m, d, rc)`。
- 用例喂给产品的却是**模型静态位姿** `model-312.json` 的 `cam_pos0`/`cam_mat0`
  （`tests/render_flex_validate.rs:246-252` `camera_poses()`，GPU 用例在 `:485` 使用）。
- 本场景 `cam_mode = [0, 3, 3]`、`cam_targetbodyid = [-1, 6, 4]`：`cam-persp` 是固定相机（mode 0），
  `cam-track` / `cam-flex` 是 **mode 3 = TARGETBODY 跟踪相机**。tracking 相机的 data `cam_xmat` 是
  look-at 乘积（上游 `smooth.py:829-886`），而 model `cam_mat0` 对 mode 3 是**单位矩阵**。
- 位级对表（`diagnosis.txt` §1）：cam0 测试位姿 == data 位姿（**位级相等**，f32 bits equal）；
  cam1/cam2 位置相同但朝向分别差 **85.517° / 75.562°**、矩阵最大绝对差 1.693 / 0.968。
  85° 的朝向差足以让 cam1 渲染出完全无关的视角 → `max_byte_diff=255` 在**不涉及 flex** 的情况下被完整解释。
- 输出映射核对（§2）**全部一致**：`cam_id_map=[0,1,2]`、`cam_res` 与参考 npz 形状一致、
  像素数 `[76800, 43200, 76800]`、`rgb_adr=[0, 76800, 120000]`、npz 槽位顺序 == mujoco cam id。
- 因此：**attempt 3 对 cam1/cam2 的判据无效**（输入错误，不能作为产品 red）；cam0 通过是「位姿正确时
  产品 rigid 路径 rgb ≤1 LSB」的**弱正面数据点**；flex 数值（cam2 的 654 个 `(flexid,9)` 像素）**仍未评估**。
  **不放宽任何判据、不宣称 flex 通过/不通过。**

## 1. 方法与输入（全部 CPU-only）

工具脚本：本目录 `diag_cam_pose.py`（原始输出 `diagnosis.txt`）。输入只用仓库内 fixture：

| 输入 | 用途 |
| --- | --- |
| `fixtures/render-bvh/scene-flex/scene.xml` + `model-312.json` | 场景/模型；`cam_mode/cam_bodyid/cam_targetbodyid/cam_pos/cam_pos0/cam_mat0` |
| `fixtures/render-bvh/scene-flex/reference/base/state.npz` | `qpos` / `geom_xpos` / `geom_xmat` / `light_xpos` / `light_xdir` / `flexvert_xpos`（**只有这 6 项，无相机位姿**） |
| `.../reference/base/capture.json` | 采集 recipe 与 `context.resolved`（`cam_id_map/cam_res/total_rays`） |
| `.../reference/base/{rgb,depth,seg}.npz` | 输出映射/形状/flex 像素核对 |
| MuJoCo **3.15.0**（authoring pinned，venv） | `mj_forward(qpos)` 复算 `d.cam_xpos/d.cam_xmat`（上游 data 位姿语义） |

方法：用 `state.npz` 的 `qpos` 跑 MuJoCo 3.15 CPU `mj_forward` 得 data 位姿（上游 warp 的
`smooth.py::_cam_local_to_global` 同语义，f64 复算与 look-at 公式差 ≤1.1e-16，见 `diagnosis.txt` §2），
再与用例实际喂入的 `cam_pos0/cam_mat0`、参考 npz 形状、`context.resolved` 逐项对表。

## 2. ① 位姿对表（测试输入 vs 参考 data 位姿）

| 槽位 | 相机 | mode | 目标体 | pos Δ (f32) | xmat max abs Δ (f32) | xmat 位级相等 | 朝向差 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | cam-persp | 0 固定 | — | 0 | 0 | **True** | ≈0（位级相等） |
| 1 | cam-track | 3 TARGETBODY | tri_body(6) | 0 | 1.693267 | **False** | **85.517°** |
| 2 | cam-flex | 3 TARGETBODY | flex-target(4) | 0 | 0.968418 | **False** | **75.562°** |

- 用例喂的 mode 3 相机 `xmat` = 单位矩阵（model `cam_mat0`），data `cam_xmat` = look-at 矩阵；
  例：cam1 data `xmat = [[-0.693267, 0.056325, -0.718476], [-0.720681, -0.054182, 0.691146],
  [0.0, 0.996941, 0.078155]]`（`diagnosis.txt` §1 有 cam2 与完整矩阵）。
- `cam_id_map=[0,1,2]` ⇒ 激活槽位顺序 == mujoco cam id 顺序；参考 `image_meta.cam_ids=[0,1,2]` 一致。
  位姿位置（pos）三项都相同，**错的只有 mode 3 相机的朝向**——正好只影响 cam1/cam2。

## 3. ② 输出映射核对（全部一致）

| 槽位 | camera | 参考 npz 形状 (rgb/depth/seg) | 像素数 | rgb_adr（按 plan 顺序累加） | npz 槽位顺序 |
| --- | --- | --- | --- | --- | --- |
| 0 | cam-persp | (240,320,3) / (240,320) / (240,320,2) | 76800 | 0 | rgb_cam0 |
| 1 | cam-track | (180,240,3) / (180,240) / (180,240,2) | 43200 | 76800 | rgb_cam1 |
| 2 | cam-flex | (240,320,3) / (240,320) / (240,320,2) | 76800 | 120000 | rgb_cam2 |

- `pixels` 合计 196800 == `capture.json.context.resolved.total_rays`；
  `OutputLayout::resolve` 按 plan（`cam_id_map`）顺序累加 `rgb_adr`（`src/render/context/output.rs:39-97`）；
  用例 `compare()` 按 `RefCamera.index` 取 `layout.output.rgb_adr[camera.index]`，npz 槽位与之一一对应。
- 参考侧 flex 像素（`seg` 的 `object_type==9`）：**slot0=0、slot1=0、slot2=654**；
  seg 唯一对：slot0 `[(-1,-1),(0,5)]`、slot1 `[(-1,-1),(0,5),(1,5),(2,5),(3,5)]`、slot2 含 `(0,9)/(1,9)`。
  ⇒ cam1 的红与 flex 无关（它本来就没有 flex 像素），与 §2 的朝向差一致。

## 4. ③ 判定（按诊断卡三步）

1. **位姿对表**：与 `context.resolved.cam_id_map` 的**槽位映射一致**，但**用例实际喂入的位姿与参考渲染所用
   data 位姿不一致**（mode 3 相机朝向差 85.5°/75.6°）→ 不一致出在 **harness 输入侧**。
2. **输出映射核对**：`cam_id_map`、`cam_res`、像素数、`rgb_adr`、npz 槽位顺序、`RefCamera.index` **全部一致**；
   cam0（位姿位级正确）在 attempt 3 实际通过了 rgb 判据 → 写入路径本身没有问题。
3. **判定**：**不是真 red（产品渲染输出错）**，也**不是**产品映射/写入缺陷；是 **harness/reference 输入缺陷 D5
   （相机 data 位姿来源）**。attempt 3 对 cam1/cam2 判据**无效**，不得据此判产品通过或不通过；
   cam0 过只算弱正面数据点；flex 数值仍未被评估（cam2 未跑到）。**不放宽度差**。

## 5. 修复建议（明确位置与修法；修复另立卡，不在本诊断卡内改代码）

**方案 A（推荐，A 类证据：位级可复现）——参考补相机 data 位姿，用例直接消费**
1. `scripts/authoring/capture_render_flex.py` 的 `state_arrays(d)` 增加 `cam_xpos` / `cam_xmat`
   （f32 位型，与 `render()` 内核输入**同源**：`d.cam_xpos/d.cam_xmat`），写入参考（新文件如
   `reference/base/camera-state.npz`，或扩展 `state.npz`）并在 `capture.json` 记录 sha/来源；
   需要一次**秒级 GPU forward-only 重采**（不是验收窗口；按 Chief go 制，单次 ≤2 分钟、跑完即释放）。
2. `tests/render_flex_validate.rs::camera_poses` 改为读参考记录的 data 位姿（不再从 model `cam_pos0/cam_mat0`
   派生）；`camera_poses` 的 model 派生只保留给固定相机校验。
3. **CPU 前置自证加一条位级断言**：用例将要传入 render 的 `poses` == 参考记录位姿（f32 bit 相等）。
   这条以后会在普通 `cargo test` 抓到本类 D5，不再拖到 GPU 窗口。
4. `guards-r06-s3b-flex.json` 的 `reference/base/` 输入条目（+新增文件 sha/bytes）更新 → 重新真冻结 →
   attempt 4 仍需 Chief GO。

**方案 B（备选，B 类前提：host 复刻，不推荐作首选）——用例自己从 model+state 复算 data 位姿**
- cam-track 目标 `tri_body` 有 geom，可从 `state.npz` geom 位姿反解：实测 f32 最大差 2.235e-8、
  xmat 差 ≤5.96e-8，但**不是位级相等**；cam-flex 目标 `flex-target` **没有 geom**，且 `model-312.json`
  没有 `body_pos/body_quat` 数组 → 无法纯 state 推导（需先扩 model dump，属 CPU authoring 可再生）。
- 且 host 侧 f32 复刻与 GPU 收缩序（FMA）未证 → 只能记为 B 类参考-前提型，仍需窗口验证。

## 6. 边界与复现

- 零 GPU、零窗口、未请求 attempt 4；不改判据/容差/参考、不改产品代码、不动 `fixtures/`、不宣称 flex 通过/不通过。
- 复现（CPU-only）：
  `target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/r06-s3b-cam1-diagnosis/diag_cam_pose.py`
  （原始输出已存档 `diagnosis.txt`，环境：python 3.12.13 / mujoco 3.15.0）。
- 四门禁：`fmt` / `clippy` / `clippy-cuda` / `unit` 全过（见提交与 `.normify/runs/`）。
