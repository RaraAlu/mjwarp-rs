# 渲染/BVH S8 独立复核（产品 camlight → 端到端渲染 / r01-p8 收口口径诚实性）

任务：`2026-10-25-render-bvh-s8-independent-check`（QA 独立复核，渲染/BVH S8）
复核对象：`d8bc9d8`（S8 主体，父 `15cb6f7`）；复核基线 HEAD：`9550900`（只含本复核任务登记，无产品路径）
结论：**验收面全部成立，0 阻断**；1 条测试严谨性问题 + 3 条建议（非阻断）。

> 口径：本复核**未跑完整 G01**；不宣称完整渲染/R01/上游完全等价/双平台。本目录只含 QA 证据，
> 未修改产品代码 / 交付测试 / 交付夹具 / 原交付证据。S8 切片本身**不改产品源码**（见 §5）。

## 1. 方法（不读交付断言）

- **独立 harness**（`qa-harness-main.rs.txt`，`target/qa-s8-harness/`，产品 crate path 依赖 +
  `cuda-probe`）：**不复用** `tests/support/**`；QA 自己从 `camlight-model.json` 的模型字段构造
  `CamLightModelInput`，自己驱动产品链
  `KinematicsPlan::with_camlight → create_data → write_qpos → update_rigid → update_attached →
  update_com → update_camlight → readback → snapshot.camlight().world(0)`；自己从模型/场景夹具
  构造 context/rays 渲染，落盘 RGB/depth/seg 原始位。
- **独立判定**（`judge_camlight.py`、`judge_evidence.py`）：位姿位型对比、端到端参考对比、
  控制实验计数、哈希复算、证据/范围/收口口径检查；比较对象是冻结夹具，不由交付测试给出。
- 仅把交付 authoring 夹具（模型字段）与冻结的 S7b1/S7b2 同配方参考当**输入/比对数据**使用。

## 2. camlight 真接上（硬证据，QA 独立复现）

### 2.1 权威口径：产品 camlight 输出 == pinned 3.15 data 位姿位型

| 项 | 结果 |
| --- | --- |
| camera0（fixed）origin / xmat | **0 位差 / 0 位差**（3+9 分量） |
| camera1（TARGETBODY→body2）origin / xmat | **0 位差 / 0 位差**（3+9 分量） |
| 三灯 `light_xpos`/`light_xdir` | 产品 camlight == pinned `light_data_check` 位型，**0 位差** |
| 产品 FK body 位姿 | body1 `[0.1,-0.2,0.3]`、body2 `[0.2132049,-0.07466385,0.5897853]`（与交付记录一致） |
| 同输入两次驱动 | 位姿位**逐位相同** |

### 2.2 判别性反证（QA 自己构造）

- **产品位姿 ≠ 被篡改的夹具位姿**：把比对用参考位姿换成 identity+原点后，产品位姿与其
  **6 个 origin 位 + 17 个 xmat 位不一致**（cam0 3+9、cam1 3+8）→ 产品输出确实不是夹具回填。
  这是交付测试反证①缺失的判别性断言（见 §6 问题 1）。
- **模型输入灵敏度（正证据，证明链真的在算）**：
  - `cam_pos[0] += 0.01` → cam0 origin 改 1 位、`xpos.x` 实际变化 `0.009999990463`，cam1 **不动**；
  - `body_pos[body2].x += 0.2` → camera1（TARGETBODY）`xmat` **8/9 分量位改变**，camera0 **完全不动**。
- **渲染输入响应**：产品位姿扰动（cam1 绕 y 0.05 rad + `xpos.x+0.3`）→ RGB **18,859**、depth **19,182**、
  seg **9,464** 像素变化；同扰动重复逐位相同、另一扰动结果不同、恢复产品位姿逐位复现。
- **夹具当输入会被检测**：篡改后的夹具位姿喂给渲染 → RGB **115,972/120,000**、depth 75,507、
  seg 49,475 像素变化。

### 2.3 已知 1 ULP：辅助口径，不是产品缺陷（复核判定）

- 权威口径（产品 camlight vs pinned data 位姿）**0 位差**；
- host `crate::math::quat_to_mat` 取位姿态路径（`CameraPose::from_fields`）与产品 cam0 `xmat`
  差 **max 1 ULP、9 分量中 2 个**——QA 独立复现；
- 该 host 函数**不参与渲染输入**（渲染只吃显式传入的 `camlight` 位姿；`quat_to_mat` 仅用于
  `CameraPose::from_fields` 这一辅助派生），因此判定为**辅助口径差异，非产品缺陷**；交付测试
  断言 `max_ulp ≤ 1` 合理。

## 3. 端到端数字（默认 28 参数、spp=1、precomputed rays）

| 项 | 相机 | QA 结果 |
| --- | --- | --- |
| RGB vs 同配方参考 | cam0 / cam1 | **0/76,800、0/43,200 像素差异**，max byte diff **0**，解包 0 LSB |
| depth vs 同配方参考 | cam0 / cam1 | **120,000/120,000 位等，max 0 ULP** |
| seg vs 同配方参考 | cam0 / cam1 | **0 不一致** |
| 运行哈希（QA 从自身 dump 复算） | — | `depth_seg_hash=03efa95416bbd008`、`rgb_hash=ed50f750936c0275`，与交付日志一致 |
| 同输入两次渲染 | — | RGB/depth/seg **逐位相同** |
| 夹具位姿 vs 产品位姿渲染 | — | **逐位相同**（位姿位等 ⇒ 来源替换无副作用） |
| 跨配方 S0b（只记录） | cam0 / cam1 | `[76798,2,0,0,0]`（max 1 byte）/ `[43200,0,0,0,0]`，与夹具记录一致 |

## 4. 生命周期 / 负向控制

- `create → render → refit_bvh → render`：`refits` 0→1，位姿不变时 refit 前后 RGB/depth/seg
  **逐位相同**（QA harness 独立复现）；
- 禁用输出：只开 rgb 的 context 对 depth/seg 读回 **`DisabledOutput`**；
- 负向控制：`use_shadows` / `use_precomputed_rays=false` / `render_skybox` 均在建 context 成功后
  由 renderer 返回 `Unsupported`（无 panic）；AA（spp=2）只给 spp=1 射线 → `InvalidArray("ray_count")`；
- **destroy 后显存增长 0 B**：来自交付 `gpu-run.log`（`baseline == free == 15,756,951,552 B`）
  与 QA 重跑的 S6b 套件（`growth=0`）。**本轮 QA 未重新插桩 device_memory**（这是本片与 S6b
  独立复核的差异，明确记录为限制）。

## 5. 不回归 / 门禁 / 证据范围

- 交付套件 QA 重跑：S8 `render_e2e` **5/5**（7.98 s）、S7b1 **6/6**、S7b2 **3/3**、
  S7b3 **6/6**（PTX `197,583 B` / `27,536 B` 与 S7b3 一致）、S6b **5/5**（`growth=0`）。
- `cargo test --locked`：**100 个测试二进制 532/0**；fmt、clippy（cuda-probe）通过。
- 证据面：交付 5 个证据文件——README/receipt/host-run.log 与 `d8bc9d8` git blob **逐字节相同**；
  `gpu-run.log`/`regression.log` 仅 CRLF 差异（EOL 归一后与 blob 相同，非内容差异）。
- 夹具/脚本：`camlight-model.json` 6,499 B / 416 行 / sha `cf7af783…`；authoring 脚本 198 行 /
  sha `02707f6f…`；counts（nq/nv/nbody/njnt/ngeom/nsite/nmocap/ncam/nlight = 8/7/3/2/7/0/0/2/3）
  与 receipt 一致。
- 范围：`git diff 15cb6f7..d8bc9d8` **14 个文件，全在 S8 路径/夹具/测试/证据/.normify 内，
  `src/` 变更 0 个**（本片确实无产品源码语义改动）；size ratchet：`tests/render_e2e.rs` 499 行 /
  `tests/support/render_bvh/e2e.rs` 496 行，均 <500 且 sha 与 receipt 一致。
- r01-p8 收口映射：10 项齐备，10 条 trace 路径**全部存在**；抽检 S6b「growth 0 B」与 S7b3 PTX
  数字与对应证据一致；README §6 明确「不宣称完整 R01 通过」。
- 未满足项：7 条如实列出且覆盖 `nworld>1`、阴影/天空盒/splat/柔体/hfield/SDF/多相机、Linux/T4、
  上游完全等价、性能、更高 spp、1 ULP 辅助口径。

## 6. 问题清单

### 一般（测试严谨性，不影响结论；反证另有判别性证据支撑）

1. **交付反证①「篡改本地夹具副本后产品位姿渲染不变」在测试里是重放同一次渲染**：
   `render_e2e.rs` 先 `tampered = tampered_camera_data(&recipe)`，随后仍以 **`&product.poses`**
   渲染并断言与篡改前逐位相同——篡改副本从未进入渲染调用，因此该断言只证明**渲染确定性**，
   不能证明「渲染不读夹具」。真正的判别性证据是 ②（篡改位姿当输入 → 变化 115,972）与
   ③（产品位姿扰动 → 变化）以及 QA 补的「产品位姿 vs 篡改夹具位 23 个位不一致」。
   **建议**：在该用例里补一条 `compare_pose_bits(&product.poses, &tampered.camera_data)` 必须
   **非零**（QA 已独立实现该断言，结果 6 origin + 17 xmat 不一致），这样反证①才具备判别力。

### 建议

2. **`camlight-model.json` provenance 引用不存在的文件**：`camera_data_reference.sources` 里
   `fixtures/render-bvh/scene-e2e/reference-camera-data.json` **不存在**（另一个 S7b1 夹具存在）。
   建议删除该悬空引用或补文件。
3. **否定式禁用词字面出现**：README §1/§6 用「不宣称『完整渲染通过』『完整 R01 通过』
   『产品渲染验收通过』『上游完全等价』」的写法，语义是正确的否定，但字面命中禁用词表
   （receipt 的 3 个机器禁用词命中 0 次）。建议收口文档改为不含原词表的表述（如「仅声明…
   不作整体收口/等价/跨平台声明」），避免自动化扫描误判。
4. **负向控制用 `catch_unwind(... continue)` 兜底**：context 创建期 panic 也会被计作「写前拒绝」。
   QA 独立判定 3 类均在建 context 成功后由 renderer 返回 `Unsupported`（无 panic），结论成立；
   建议测试断言建 context 成功后再要求 `Unsupported`。

### 如实记录的限制

5. 本轮 **QA 未重新插桩 `device_memory`** 验证 destroy 增长 0 B，依赖交付日志 + QA 重跑 S6b
   套件（`growth=0`）；如需硬证据可单开小测。
6. 反证①的判别性由 QA 补测（产品位姿 vs 篡改夹具 23 位不一致 + 模型输入灵敏度）成立，
   交付测试原文不单独具备判别力（见问题 1）。

## 7. 未覆盖 / 边界（不宣称）

- 未跑完整 G01；不宣称完整渲染/R01 通过、不宣称上游完全等价、不宣称双平台。
- `nworld > 1` 未验收；阴影/天空盒/splat/柔体/hfield/SDF/多相机/动态射线未实现（写前拒绝）。
- Linux/T4 未验收（GPU 证据仅 RTX 4070 Ti SUPER / 596.36 / SM89）；性能未验收；
  AA 仅 `samples_per_pixel=2` 有图像级参考。
- S8 只证明 scene-a 默认参数下的渲染数值对齐；其余场景/参数未覆盖。

## 8. GPU 窗口

- 设备 RTX 4070 Ti SUPER / 596.36 / CUDA 13.2 / 16 GiB；Arch 交接基线 `5,053 MiB`，QA 首次测量
  **`4,905 MiB`**；结束 **`~4,250 MiB`**，**无本任务 CUDA 计算进程**（已释放）。
- 实际 GPU 活动：QA harness 3 次 camlight 链 + 1 次 e2e（约 20 帧渲染，含 tamper/扰动/重复/
  refit/禁用/拒绝）+ 5 套交付用例（7.98+4.39+3.12+8.92+7.79 s），合计约 2–3 分钟量级；
  60 分钟窗口内、无冲突、未超窗口占用。

## 9. 证据文件

- `receipt.json`：机器可判定回执；`qa-artifact-manifest.json`：QA 运行产物哈希绑定；
  `artifact-check.json`：提交证据与交付证据哈希清单。
- `judge_camlight.json`：camlight 位姿/端到端/控制实验/哈希/1 ULP 辅助口径判定。
- `judge_evidence.json`：交付证据/日志/范围/size ratchet/R01 映射/未满足项/禁用词扫描。
- `camlight*.json`、`e2e.json`、`controls.json`：QA harness 原始结果。
- `qa-gpu-runs.log`（5 套交付用例重跑）、`qa-unit-tests.log`（100 二进制 532/0）。
- 脚本：`judge_camlight.py`、`judge_evidence.py`、`qa-harness-main.rs.txt`。
