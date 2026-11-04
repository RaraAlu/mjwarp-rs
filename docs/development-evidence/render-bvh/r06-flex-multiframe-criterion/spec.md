# R06 多帧判据规格（可签版）

- 任务：`2026-11-05-render-r06-flex-multiframe-criterion-spec`（owner Arch）
- 日期：2026-11-05
- 状态：**待用户签名**（签名对象 = 本文件 sha256 + bytes，落库后由 Chief 报用户）
- 上游依据：方案 B 设计草案 `2026-11-05-render-r06-flex-multiframe-criteria`（证据 `docs/development-evidence/render-bvh/r06-flex-multiframe-criteria/`）
- 边界：**规格文本**。不采集、不开窗、不动 `fixtures/`、不改产品代码、不改任何已签判据/容差/参考。

---

## 1. 适用与边界

- 本规格只判**多帧**（位姿变化 → 渲染随之变化且数值正确），**替代不了**单帧判据：单帧（base 帧）仍按已签口径独立判定。
- 单帧已签口径（不重议）：同一 GPU / 同一 recipe / 同一编译形态下，对 `fixtures/render-bvh/scene-flex/reference/base/` 的 rgb ≤1 LSB 且 `beyond_1_lsb=0`、depth ≤4 ULP 且 `beyond4=0`、seg 精确相等、两跑 sha256 相同。
- **不宣称动画一致性、不宣称多帧已覆盖、不宣称与上游一致、不作对外发布依据。**

## 2. 帧集定义（写死）

| 帧 | 位姿（3 个 flex 锚点 body 的 `body_pos.z` 相对 base 偏移） | 说明 |
| --- | --- | --- |
| frame-000 | `+0.000 m`（= base 帧，与现有 `reference/base/` 逐位同源） | 与已签单帧判据共用 |
| frame-001 | `+0.100 m` | 步进 1 |
| frame-002 | `+0.200 m` | 步进 2 |

- **位姿步进参数写死**：`Δz = +0.100 m / 步`，共 2 步。
- **变化机制（参考侧与产品侧同构）**：把 `flex-anchor0` / `flex-anchor1` / `flex-anchor2` 三个 body 的 `body_pos.z` 分别加 `k·Δz`（k=0,1,2），其余模型字段**逐字节同源**；flex 顶点绑定在这三个锚点 body 上，世界坐标随之刚性上移。
  - 参考侧：pinned mjwarp 载入 315 MJB 后，在 `put_model` 前应用该偏移。
  - 产品侧：测试从 `scene-flex` 的 312 模型输入载入后，对同三个锚点 body 的 z 施加同偏移再建 context（**只改内存数组，不改 fixture 文件内容**）。
- **不做**物理步进/时间积分/加外力：帧间差异 100% 来自位姿注入，排除求解器演化噪声。
- 三帧必须来自**同一次采集运行**、同一 recipe、同一 pinned 环境；`capture.json` 记录 `Δz`、帧索引、每帧 sha256。

## 3. 相机与判据相机（写死）

| 相机索引 | 名称 | 分辨率 | 像素数 |
| --- | --- | --- | --- |
| cam0 | `cam-persp` | 320×240 | 76,800 |
| cam1 | `cam-track` | 240×180 | 43,200 |
| **cam2（判据相机）** | **`cam-flex`** | **320×240** | **76,800** |

判据相机 = **cam2**（`cam-flex`，专为 flex 覆盖设计）。cam0/cam1 的逐帧结果**也判帧内口径**，但其帧间变化像素数只作诊断、不参与跨帧判定。

## 4. 帧内判据（三相机各自，沿用已签口径）

对 frame-000 / frame-001 / frame-002 **每一帧**：

1. rgb：≤1 LSB 且 `beyond_1_lsb=0`；
2. depth：≤4 ULP 且 `beyond4=0`；
3. seg：精确相等（`(id,type)` 对）；
4. 两跑 sha256 相同（逐帧、逐相机）。

## 5. 跨帧判据（cam2，写死）

**变化定义（在参考侧计算，避免循环论证）**：对 cam2，相邻帧（000→001、001→002）：
- **depth 变化像素** = `bits(depth_frame_k) != bits(depth_frame_{k-1})` 的像素数；
- **seg 变化像素** = `seg_pair(frame_k) != seg_pair(frame_{k-1})` 的像素数。

**判据阈值（写死，验收时用）**：
- depth 变化像素数 ≥ **384** 且 seg 变化像素数 ≥ **384**（= 0.5% × 76,800；两个相邻帧对都要满足）。
- 同时：产品输出与参考在每一帧 bit-level 相等（同 §4），两跑 sha 相同。

**采集前置门槛（留余量，采集窗内自查）**：
- depth 与 seg 的变化像素数都需 ≥ **576**（= 1.5 × 384）才允许落库并进入验证；
- 实测落在 **[384, 576)** → 判「余量不足」，**采集失败、停手报用户**，不得改阈值；
- 实测 < 384 → 判「无判别力」，**拒绝通过**（不是通过）。

**退化保护**：若参考帧间变化 < 384，整轮判「无判别力、拒绝通过」；**禁止现场调阈值、禁止用产品侧数据替代参考侧自查**。若需换步进参数（例如 Δz 加大），属**改规格**：必须出新 spec、新 sha、用户重签，阈值 0.5% 不变。

## 6. allowlist 规则（写死）

- 用户签名 allowlist `3406f1ca`（仅 `cam2 / (x=134,y=128) / flexid0`、`ULP≤4 且 beyond4=0`）**只对 frame-000（base 帧）生效**。
- **frame-001 / frame-002 一律 bit-first，不继承任何放宽。**
- 若新帧在同一像素（或别处）复现该差异 → **必须报用户**，不得自动扩 allowlist、不得把该像素写成一般容差。

## 7. 失败条件（写死；失败即停、不重试、不放宽）

任一条成立即整轮失败：
1. 任一帧/任一相机超出帧内判据；
2. cam2 相邻帧 depth 或 seg 变化像素数 < 384；
3. 两跑 sha 不同；
4. 参考帧间变化 < 384（无判别力）或落在 [384,576)（余量不足）；
5. 采集确定性检查失败（两次采集非逐位一致）；
6. 出现未申报的像素级放宽（含 allowlist 被外推）。

## 8. 采集方案（写进规格，供实现卡引用）

### 8.1 环境（写死）
- 离线 authoring：`target/mjwarp-parallel/venv`（Python 3.12.13）；
- `mujoco_warp` 以 **editable** 方式装在 pinned checkout（`71da24d9`，`mujoco_warp 3.15.0`），**禁止 PyPI 轮子**（R06 S0 已记录 PyPI 轮子≠pinned commit）；
- `warp-lang 1.15.0`、`mujoco 3.15.0`；
- recipe：`use_fast_math=false`，其余参数逐字同 `reference/base/capture.json` 的 `parameters` 块；
- GPU：与 base 参考同一台（RTX 4070 Ti SUPER / driver 596.36 / SM89）。

### 8.2 脚本扩展点
- `scripts/authoring/capture_render_flex.py` 新增帧循环模式（如 `--frames 3 --frame-dz 0.1`）：对每一帧应用锚点 `body_pos.z` 偏移后 `put_model` → forward → 渲染并落盘；不改现有单帧输出路径（单帧模式行为逐字节保持）。
- 每帧输出 `rgb.npz` / `depth.npz` / `seg.npz` / `state.npz`（含 `qpos`、`geom_xpos/xmat`、`light_xpos/xdir`、`flexvert_xpos`，与 base 同字段）。

### 8.3 产物布局
```
fixtures/render-bvh/scene-flex/multiframe/
  frame-000/{rgb.npz, depth.npz, seg.npz, state.npz}
  frame-001/{rgb.npz, depth.npz, seg.npz, state.npz}
  frame-002/{rgb.npz, depth.npz, seg.npz, state.npz}
  capture.json      # 帧数、Δz、recipe、pinned 环境、每帧 sha256、两次采集逐位一致标记
  cam-poses.json    # 三相机位姿（data 位姿，f32 bit provenance，同 base 口径）
  README.md         # 生成命令与 provenance
```
体积估算：单帧 rgb 2.36 MB + depth 0.79 MB + seg 1.58 MB ≈ **4.73 MB** → 3 帧 ≈ **14.2 MB**（+ json 约 0.05 MB）。

### 8.4 确定性与 provenance（写死）
- **两次采集逐位一致**：同参数重复采集一次，逐帧逐文件 sha256 相等，否则采集失败。
- `capture.json` 必含 provenance：python / warp / mujoco / mujoco_warp 版本、pinned commit `71da24d9`、GPU 与 driver、`use_fast_math`、`Δz`、帧索引、每帧文件 sha256、两次采集对比结果、采集时间、命令。

### 8.5 采集窗内自查顺序（写死）
1. 采集两次 → 逐位一致；
2. 计算 cam2 相邻帧 depth/seg 变化像素数；
3. 都 ≥576 → 落库；任一 <576 → 停手报用户（不落库为验收材料）；
4. 采集证据（采集窗日志、基线、monitor、freeze 如需）提交后进入验证前置。

## 9. 判据实现要求（供实现卡引用，本规格不实现）

- 新增测试实体（如 `render_flex_multiframe`）：读 3 帧参考 → 逐帧 §4 判定 → cam2 §5 跨帧判定 → 两跑 sha；
- 产品侧位姿注入：内存中对三个锚点 body 的 `body_pos.z` 加偏移后建 context（不改 fixture 文件）；
- 窗口材料：新实体使当前 guards（16 条量级）需 `freeze -Attempt` 重冻（新增 exe + 3 帧输入）；
- 判据实现需 QA 只读复核（像素计数方向、阈值换算、allowlist 边界、退化保护）。

## 10. 签名清单（用户级）

| # | 事项 | 说明 |
| --- | --- | --- |
| **S-1** | **本判据规格**（`spec.md` 的 sha256 + bytes 写进签名卡） | 新判据：帧集、0.5% 阈值、退化保护、allowlist 不继承、失败条件 |
| **S-2** | **采集窗是否预批 authoring 例外**（常规 util<40 未达标时是否允许 authoring-only util<60） | 按先例（R05-S3a / R02-S2）需用户预批；不预批则等常规阈值 |
| **S-3** | **验证窗材料是否双签** | 先例：用户双签验证窗（材料 + 判据） |

## 11. GO 清单（Chief 级）

| # | 窗口 | 前置 |
| --- | --- | --- |
| **G-1** | 采集窗（authoring，1 个） | S-1 已签 + 本 spec 与 receipt/manifest 落库 + capture 脚本帧循环实现完成（实现卡 verified）+ QA 只读复核脚本 + 开窗前 5 分钟三点基线/floor 取严 + 材料贴群 |
| **G-2** | 验证窗（1 个） | 采集证据 verified（3 帧参考 + 两次采集逐位一致 + 变化像素 ≥576）+ 判据实现进用例（实现卡 verified）+ QA 复核 PASS + 窗口材料冻结（guards 条目含新 exe + 3 帧输入）+ 基线/floor |

## 12. 已知风险（如实登记）

1. **变化幅度不确定**：Δz=0.1 m 的投影位移依相机几何而定；若采集中 cam2 变化 <576，按 §5 停手报用户（不自动加大步进、不调阈值）。
2. **帧数少**：3 帧只证明「两段位姿变化被正确渲染」，不构成连续动画覆盖；本规格不声称连续演化。
3. **位姿注入是 authored**：帧间差异来自模型层锚点偏移，不是物理演化；本判据只测渲染对位姿的正确响应，不测物理/动画。
4. **夹具体积**：≈14.2 MB；若用户要更多帧，线性增长。
5. **重冻代价**：新实体使当前 guards 失效，验证窗需重冻；期间渲染线无「已验证 exe 直接复用」状态。

## 13. 变更控制

- 本文件是**签名对象**：任何改动（帧数、Δz、阈值、相机、判据、失败条件、采集环境）都产生新 sha → **需用户重签**；
- 签名后，本文件与 `receipt.json`、`manifest.txt` 按 sha256 冻结；实现卡只能引用、不得改写本规格；
- 历史不回改：本规格前的单帧口径与摘要保持原样。
