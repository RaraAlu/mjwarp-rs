# per-world L v2 灯判据规格（可签草案）

- 任务：`2026-11-07-render-perworld-l-v2-light-criterion-spec-v3`（owner Arch）；v2 修订卡 `2026-11-06-render-perworld-l-v2-light-criterion-spec-v2`；初版卡 `2026-11-06-render-perworld-l-v2-light-criterion`
- 日期：2026-11-07
- 版本：**v3**。取代 v2（sha256 `6cfaa57e2c8d5317442f90fddc157c666430d726ad20d3ceace102fe0b435900` / 20,681 B），v2 已取代 v1（`1821be98…` / 16,625 B）。
- 状态：**待用户签名、未生效**（签名对象 = 本文件 sha256 + bytes）
- **用户拍板（2026-11-07，经 Chief 转派）**：perworld L v2 的**判别力两套都要做**——「rgb 变化（灯驱动）」与「换相机看几何（depth/seg 几何驱动）」**同时覆盖**；不得以「rgb 够用」「体积大」为由省掉任一套。v3 据此改为**双相机 + 双判别力判据（C1/C2）**。
- **预审修订（2026-11-07）**：QA 读前预审 PASS-with-observations（0 阻断）后，**O1–O3（相机插入位置/索引顺序、§9.2 自查字段、§2.4 相机快照字段）已在本版签名前修正**，O4–O7 低项同批修正；逐条见 §14.1。签名对象以**本文件当前 sha256 + bytes** 为准。
- 前置（均已落库）：
  - **P0-1 每 world 场景刷新**：`2026-11-07-render-perworld-l-v2-capture-perworld-refresh` verified（实现 `e34df3b7`；`render_images` 在 `mjw.render` 紧前调 `mjw.refit_bvh(m, d, rc)`，两遍采集都覆盖），QA 复核卡 `...-capture-perworld-refresh-check` PASS（`3e08be09`）。**本规格的 C2（几何驱动判别力）以 refit 生效为前提**：不 refit 则三维 BVH 是建树时 CPU 快照，几何视角会退化为 world 间逐位相同。
  - **L v1 参考 B 类降级**：`2026-11-07-render-perworld-l-v1-reference-discriminability-check` verified（证据 `c318cebb`）——既有 `nw3-hinge02040` 参考「输入位姿不同、输出逐位相同」，按《机器判据裁定标准 v1》判 **B 类（参考-前提型）**、前提「参考具备 world 间判别力」不成立 → 该参考上的逐 world 位姿对照降级/不适用，不得作为 per-world 位姿正确性依据。
  - 备选准备片 `2026-11-06-render-perworld-l-v2-light-prep`（`b0b8cc02`）：`scene-a-light` 场景变体 + 采集脚本 `--scene`/`--out-root` 参数化 + 三 world 灯位姿前置自算。
- 上游依据：pinned checkout `target/mjwarp-parallel/upstream` @ `71da24d9`（mujoco_warp 3.15.0）。
- 边界：**规格文本**。不采集、不开窗、不动 `fixtures/`（场景/相机改动在 v3 签名后另卡，见 §12）、不改产品代码、不改任何生效判据/容差/criterion v2.2/既有冻结件。**本草案未经用户签名不生效。**

---

## 1. 适用与边界

- 本规格判 **per-world 灯位姿（L v2 灯）** 在 nworld>1 时的逐 world 渲染正确性；**判别力必须两套同时成立**：
  - **C1 灯驱动**：照明视角（保留的 `cam-persp`，视野以地板为主）上，相邻 world 的 **rgb** 变化像素数达标；
  - **C2 几何驱动**：几何视角（新增固定相机 `cam-hinge`，视野含随 hinge 转动的几何）上，相邻 world 的 **depth 与 seg** 变化像素数达标。
- 本规格**替代不了** per-world geom 位姿判据（L v1 及其 B 类降级限定）与既有单帧/多帧判据；三者各自独立判定。
- **不宣称**：不宣称 per-world 灯位姿已覆盖、不宣称与上游实现等价、不宣称性能、不宣称 R06/G10 等其它家族结论、不作对外发布依据。
- 本规格通过 ≠ perworld L v2 全链通过：实现卡、采集证据、验证窗三者齐备才谈「该实体在该参考上通过」。

## 2. 参考范围（写死）

### 2.1 场景变体（写死）

- 文件：`fixtures/render-bvh/scene-a-light/scene.xml`（v3 定稿后由**另卡**改相机并重算 sha；本规格不写 fixture）。
- 与 `fixtures/render-bvh/scene-a/scene.xml` **同源**，差异**恰好两处**：
  1. 三灯 `sun`/`lamp`/`spot` 以 **MJCF 嵌套**方式放入 `<body name="tri_body">`（MJCF 没有 `<light body="...">` 属性）；
  2. **新增固定相机 `cam-hinge`**（照明视角 `cam-persp` 保留不动）：
     | 属性 | 值 |
     | --- | --- |
     | name | `cam-hinge` |
     | mode | 0（`fixed`；`cam-track` 的 `targetbody` 不用于本参考——相机位姿随 world 变会污染归因） |
     | pos | `-0.0925 0.4981 0.8296` |
     | quat | `-0.211088 -0.149763 0.558924 0.787792` |
     | fovy | **30** |
     | resolution | **320 240** |

- **插入位置写死**：`cam-hinge` 必须插在 `cam-persp` 之后、`cam-track` 之前 → 相机索引 **0/1/2 = `cam-persp`/`cam-hinge`/`cam-track`**。该顺序决定 `cam_active`（**全相机 3 条**）、`cam_res`（**仅启用相机 2 条**）与产物键 `cam1`=cam-hinge 的语义；改变插入位置属**改参考范围**（新 spec + 新 sha + 重签），窗口内不得变更。
- `cam-persp`（`pos 1.2 -1.1 0.8`、`quat 0.9 0.1 -0.2 0.3`、`resolution 320 240`、`sensorsize 0.036 0.024`、`focalpixel 160 160`、`projection perspective`）**逐字不变**；`cam-track`（`mode targetbody`、`fovy 40`、`240 180`）**逐字不变**。
- 三灯全部属性（`type/pos/dir/diffuse/specular/ambient/attenuation/cutoff/exponent/softness/castshadow/active/bulbradius/intensity/range`）**逐字不变**；几何、材质、纹理、`<visual>` **逐字不变**。
- `fixtures/render-bvh/scene-a/` 既有文件**零改动**。
- `cam-hinge` 的位姿不是随手取的：按 CPU 射线聚合在本场景上选出「视野内包含足量随 hinge 转动的几何、且相邻 world depth/seg 变化像素远超门槛」的固定位姿（见 §3.3）；pilot/采集窗**不得**再改相机位姿——若要改属**改参考范围**（新 spec + 新 sha + 用户重签）。

### 2.2 参考 tag 与位形（写死）

| 项 | 值 |
| --- | --- |
| tag | **`nw3-light-hinge02040`** |
| 输出根 | `fixtures/render-bvh/scene-a-light/perworld/` |
| 参考目录 | `fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/` |
| worlds | **3**（nworld=3） |
| hinge 角（度） | **0 / 20 / 40**（写入各 world 的唯一 hinge `tri-hinge` qpos） |
| 相机 | **两台固定相机**：`cam-persp`（照明视角，320×240 = 76,800 px/world）+ `cam-hinge`（几何视角，320×240 = 76,800 px/world），合计 **153,600 px/world**；`cam-track`（mode 3）不启用 → `cam_active=[true, true, false]` |
| recipe.parameters | **除 `cam_active` 外逐字同**既有 perworld 参考 `nw3-hinge02040/capture.json.parameters`（`use_fast_math=false`、`use_shadows=true`、`samples_per_pixel=1`、`enabled_geom_groups=[0,1,2,3]`、`background_color=[0,0,0,1]`、`shadow_light_fraction=0.3`、`flex_render_smooth=true`、`use_precomputed_rays=true`、其余同源）；**`cam_active` 由旧参考的 `[true,false]`（scene-a 2 台相机）变为 `[true,true,false]`（scene-a-light 3 台相机，`cam-track` 不启用）**——这是本参考与旧参考 `parameters` 的**唯一差异**，必须在采集证据中记录前后值（**3 条；不得按 2 条记**） |
| recipe.scene_xml / recipe.script | **sha256/bytes 必然变化**（场景变体加相机 + 采集脚本既有参数化），**不得**要求与旧参考相同；`recipe` 的 `nworld/hinge_deg/seed/repeat/versions/upstream_revision` 与旧参考同口径 |
| 变化机制 | 只在**模型层外部注入**：各 world 的 hinge qpos = 0/20/40 度；灯随 `tri_body` 刚性转动 → `light_xpos/light_xdir` 逐 world 不同；几何随 hinge 转动 → `cam-hinge` 视角可见 world 间差异。**不做物理步进/时间积分/加外力**，world 间差异 100% 来自位形注入；**两台相机都是固定相机**，相机位姿不随 world 变 |
| refit 前提 | 采集必须在**每次 `render` 前**执行 `mjw.refit_bvh(m, d, rc)`（P0-1 已实现；不得重建 context）；无 refit 时 `cam-hinge` 的几何 world 间差异不成立（C2 会退化为 0） |

### 2.3 产物布局与编码（写死）

目录：
```
fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/
  rgb.npz / depth.npz / seg.npz     # 每 world × 每相机独立键（见下）
  state.npz                        # 按 nworld 首维堆叠（见下）
  capture.json                     # 位形、recipe、pinned 环境、两台相机快照、文件 sha256、两次采集逐位一致标记、per_world_scope、C1/C2 计数
```

**rgb/depth/seg：每 world × 每相机一个独立键**（`cam0`=`cam-persp`、`cam1`=`cam-hinge`，world 号与相机号都写在键名里，**不是** `[nworld, ...]` 堆叠）：

| 文件 | 键 | shape | dtype |
| --- | --- | --- | --- |
| `rgb.npz` | `rgb_w{0,1,2}_cam{0,1}` | `(240, 320, 3)` | f32 |
| `depth.npz` | `depth_w{0,1,2}_cam{0,1}` | `(240, 320)` | f32 |
| `seg.npz` | `seg_w{0,1,2}_cam{0,1}` | `(240, 320, 2)` | i32 |

**state.npz：无 world 键，数组以 nworld 为首维堆叠**（nworld=3，与 v2 相同）：

| 键 | shape | dtype |
| --- | --- | --- |
| `qpos` | `(3, 8)` | f32 |
| `geom_xpos` | `(3, 7, 3)` | f32 |
| `geom_xmat` | `(3, 7, 3, 3)` | f32 |
| `light_xpos` | `(3, 3, 3)` | f32（nworld × nlight × 3） |
| `light_xdir` | `(3, 3, 3)` | f32（nworld × nlight × 3） |

体积估算（按既有 `nw3-hinge02040` 单相机实测线性放大，两台相机）：

| 文件 | 单相机实测 | 双相机估算 |
| --- | --- | --- |
| `rgb.npz` | 2,765,524 B | **5,531,048 B** |
| `depth.npz` | 922,336 B | **1,844,672 B** |
| `seg.npz` | 1,843,924 B | **3,687,848 B** |
| `state.npz` | 2,486 B | 2,486 B |
| `capture.json` | ≈ 8.8 KB | ≈ 10 KB（多两相机块 + C1/C2 计数） |
| **合计/tag** | ≈ 5.55 MB | **≈ 11.08 MB**（11,076,054 B） |

### 2.4 确定性与 provenance（写死）

- **两次采集逐位一致**：`--repeat 2`，两次运行的**每文件** sha256 相等，否则采集失败。
- `capture.json` 必含：`nworld`、`hinge_deg`、`recipe`、`parameters`、pinned 环境（python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4）、pinned commit `71da24d9`、每文件 sha256、两次采集对比结果、采集时间、命令（`command.argv` 逐字）、`reference_tag`、`per_world_scope`。
- `capture.json` 另须记录**两台相机的快照**，字段来源写死（不得按想象字段记录）：
  - `context.resolved.cam_id_map`、`context.resolved.cam_res`（**只含启用相机**，本参考应为 `[0,1]` 与 `[[320,240],[320,240]]`）；
  - `recipe.cameras`（现有 `camera_recipe` 产出 `slot / mujoco_cam_id / name / mode / xpos / xquat / fovy / resolution`，应为 2 条）；
  - **v3 要求 `camera_recipe` 额外补 `sensorsize` 与 `intrinsic` 两个字段**（现脚本未产出，属 §11.1 第 6 项必须落实的前置；未补则 §9.3 第 4 步自核不通过）。`cam-hinge` 的 `sensorsize` 为 `[0,0]`、`intrinsic` 为模型默认值 `[0.01, 0.01, 0, 0]`，必须如实记录其走 **fovy 路径**，**不得声称有 intrinsics**。
- `capture.json` 另须记录**判别力计数**（参考侧自算）：`c1_rgb_changed_px`（两对 world，含 bitwise 与 `max_channel|Δrgb|≥1/255` 两个计数）、`c2_depth_changed_px` / `c2_seg_changed_px`（两对 world），以及诊断项 `campersp_depth_seg_changed_px`、`camhinge_rgb_changed_px`。
- `capture.json.per_world_scope` 必须写明本参考覆盖「per-world 灯位姿 + per-world geom 位姿」，并注明设备路径的灯语义前提（见 §10.3）。

## 3. 事实地基：两套判别力（写死，不得省略）

以下数值均为 **CPU 侧预测量**（pinned venv 的 MuJoCo `mj_forward` + 逐像素 `mj_ray` 射线聚合；不建 CUDA、不渲染）。射线聚合口径已用既有 L v1 参考校验：命中/未命中像素完全一致（59,805/59,805）、depth 相对误差 max 3.52e-7、seg 逐像素 0 差异。**预测量不替代 pilot/采集窗的真渲染实测**；实测不达标按 §8 停手报用户。

### 3.1 灯位姿差异（与 prep 卡 `light_pose_precheck` 一致）

| 灯 | 类型 | 差异键（world 1 / world 2 相对 world 0） | 结论 |
| --- | --- | --- | --- |
| `sun` | directional | `xdir` 夹角 **10.745878447° / 21.255926664°**；地板 `N·L` 0.7178 → 0.7477 → 0.7863 | **有判别力** |
| `lamp` | point | `xpos` 位移 **0.222378165 m / 0.437999482 m**；`diffuse=(0.7,0.5,0.3)` 暖色 | **有判别力** |
| `spot` | spot | **0 / 0**（`pos=(0,0,2)`、`dir=(0,0,-1)` 落在 `tri_body` 的 z 旋转轴上） | **恒 0，无判别力（几何事实）** |

- **本参考只有 `sun`/`lamp` 两灯具备跨 world 判别力**；`spot` 的恒定是场景几何决定，不是实现缺陷，也不得在判据里当作「不同」。
- 若用户要求三灯均可判别：属**改参考范围**（改灯位/朝向或换旋转轴）→ 新 spec、新 sha、用户重签；本规格不覆盖。

### 3.2 视野事实（写死；解释为什么判别力必须分两台相机）

| 相机 | 视野构成（CPU 射线聚合，320×240=76,800 px） | 相邻 world 变化 |
| --- | --- | --- |
| `cam-persp`（照明视角） | **地板 59,805 px + 背景 16,995 px；几何 0 px**（hinge 0°–330° 每 30° 实测均 0 px） | depth/seg 变化 **恒 0**（几何事实）；rgb 随灯位姿变化 |
| `cam-hinge`（几何视角） | 世界变化几何约 **2.4–3.1 万 px**（tri 系列）+ 静态 `base-box` 约 0.9–1.3 万 px + 地板 1.1–2.0 万 px + 背景；两台合计覆盖照明与几何 | depth 变化 **35,812 / 33,947 px**；seg 变化 **16,041 / 16,755 px**（两对 world）；world 间 depth+seg 都未变的稳定像素 40,988 / 42,853 px |

- 即：**照明视角对世界变化几何结构性不敏感**（这是 L v1 参考退化为空转的直接原因，见前引 B 类降级）；几何驱动判别力**必须**由另一台能看见转动几何的相机承载，因此本规格写死双相机，不接受「只用 rgb 一套」。

### 3.3 rgb 灯驱动判别力（`cam-persp`，写死）

- 逐像素辐照度相对变化（world 0→1 / 0→2，含 sun+lamp，按通道）：中位 **1.7%–2.5% / 5.2%–6.7%**，最小 **0.85% / 2.4%**；**100%** 地板像素变化 ≥ 1/255。
- 拿既有参考 rgb 做 clipping-aware 预测（参考本身饱和：mean 0.9743、min 0.9020，**47.4%** 像素至少一通道已 =1.0，全饱和像素不随灯变）：`max_channel|Δrgb|≥1/255` 计数预测 **55,581 px（0→1）/ 55,839 px（1→2）**，bitwise 变化 **55,839 px**，即门槛 576 的 **≈97 倍**。
- 背景恒黑（`render_skybox=false`）→ 只有地板区贡献，计数上界 59,805。

### 3.4 几何驱动判别力（`cam-hinge`，写死）

- 相邻 world：depth 位变化像素 **35,812 / 33,947**（门槛 576 的 **62× / 59×**）；seg 变化像素 **16,041 / 16,755**（**28× / 29×**）。
- **QA 独立复算（2026-11-07 只读预审）**：depth 35,873 / 33,898（与本表差 ≤0.2%，源于 f32/f64 深度比较精度），seg 16,041 / 16,755 完全一致 → 不改变门槛量级（60× 量级）；正式判定仍以 pilot/采集窗真渲染实测为准。
- **前提**：采集必须按 §2.2 执行每 world refit（P0-1）；不 refit 时三 world 的 BVH 边界来自建树快照，`cam-hinge` 的 depth/seg 会退化为 0。

### 3.5 事实地基的效力边界

- 上述 §3.3/§3.4 是**预测量**：pilot（§12 G-L0）与采集窗（§9.3）用真渲染实测；采集实测低于 576 → 停手报用户，不得改阈值/换参考/调相机。
- 两套判别力**都要**实测 ≥576（采集前置）才允许落库：任一套不足即整轮失败。
- **QA 独立复核（2026-11-07 只读预审）**：按上游 `compute_ray` 口径独立复算并与 L v1 参考对齐（hit/miss 0 差异、seg 0 差异、depth 相对误差 3.73e-7）；cam-persp floor/bg/几何 0 px 与 cam-hinge 四类像素合计 76,800/world 逐值一致；C2 seg 16,041/16,755 完全一致、depth 35,873/33,898（差 ≤0.2%）；C1 预测 ≤ 上界；饱和统计 0.974256/0.901961/47.39% 一致。预审两轮（PASS-with-observations 0 阻断 + 第二轮 2 条阻断项「相机条目数口径」）均已在本版**签名前**修正，逐条见 §14.1。

## 4. 判据 A：逐 world 渲染（写死，严格口径；两台相机各自）

对**每台相机**（`cam-persp`、`cam-hinge`）的每个 world（0/1/2）输出：

1. **rgb：位级 0 字节差**（`max_byte_diff = 0`，逐字节全等；不使用 ≤1 LSB 放宽）；
2. **depth：位级全等**（`mismatched_pixels = 0`，`beyond_ulp = 0`）；
3. **seg：逐像素 `(id, type)` 全等**；
4. **两次运行 sha256 相等**（rgb/depth/seg 各自、逐 world、逐相机）。

补充：
- 若实测出现任何 rgb 字节差或 depth/seg 位差 → **失败**（见 §8），**不得**引用 criterion v2.2 清单或任何 allowlist 洗成通过。
- 判据相机为 `cam-persp` 与 `cam-hinge` 两台固定相机；不存在「其它相机只作诊断」的放宽空间（`cam-track` 不启用，也不作判据）。
- 归一化：判据按 world 判「该 world 的输出」，不把不同相机混在一个哈希里。

## 5. 判据 B：灯字段数据面（写死；与 v2 相同）

- 参考 `state.npz` 的 `light_xpos`（`[nworld, nlight, 3]` f32）与 `light_xdir`（同形）必须与产品侧逐 world 数据**位级对齐**（f32 位型相等，**0 例外**）。
- 允许的实现获取方式：产品侧按 world 从自身位姿/灯定义重算，或直接消费采集侧 `state.npz`；**两条路都必须给出位级相等**，不得用「近似相等」替代。
- 若产品侧灯语义与参考不同源（例如仍用单 world 灯数组），本判据会直接失败——这是**预期**行为，不得为其开 allowlist。

## 6. 判据 C：world 间判别力（写死；**拆 C1/C2 两套，都必须满足**；参考侧自算，不用产品侧数据）

**变化定义**（在参考侧计算，避免循环论证）：

- **C1（灯驱动，`cam-persp`）**：相邻 world 对（0→1、1→2）的
  `rgb_changed_px` = 像素级 `max_channel |Δrgb| ≥ 1/255` 的像素数（`1/255 ≈ 0.39%` 输出量级，远高于 f32 算术噪声；饱和像素不计）；
- **C2（几何驱动，`cam-hinge`）**：相邻 world 对的
  `depth_changed_px` = `bits(depth_world_k) != bits(depth_world_{k-1})` 的像素数；
  `seg_changed_px` = `seg_pair(world_k) != seg_pair(world_{k-1})` 的像素数。

**验收判据（写死）**：两个相邻 world 对都要满足

- **C1：`rgb_changed_px` ≥ 384**；
- **C2：`depth_changed_px` ≥ 384 且 `seg_changed_px` ≥ 384**（= 0.5% × 76,800，逐相机像素数）。

**采集前置门槛（留余量）**：两个相邻 world 对都要满足

- **C1：`rgb_changed_px` ≥ 576**；
- **C2：`depth_changed_px` ≥ 576 且 `seg_changed_px` ≥ 576**（= 1.5 × 384）。
- 任一套落在 **[384, 576)** → 判「余量不足」，**采集失败、停手报用户**，不得改阈值；
- 任一套 < 384 → 判「无判别力」，**拒绝通过**（不是通过）。

**诊断项（不设门槛，必须记录）**：

- `cam-persp` 的 depth/seg 变化像素：预期 **0 / 0**（几何事实，见 §3.2；不得据此判失败，也不得拿它冒充 C2）；
- `cam-hinge` 的 rgb 变化像素：预期很大（几何+灯共同驱动），用于交叉检查；**不得拿它冒充 C1**。

**退化保护**：任一套判别力不达标，整轮判「无判别力、拒绝通过」；禁止现场调阈值、禁止用产品侧数据替代参考侧自查、禁止交换 C1/C2 的相机定义。若需换位形/相机（例如 hinge 角加大、`cam-hinge` 改位姿），属**改规格**：必须出新 spec、新 sha、用户重签，阈值 0.5%/1.5% 不变。

## 7. 与 criterion v2.2 的关系（写死；与 v2 相同）

- **本实体的判据不继承 criterion v2.2 的两条已签清单**（recipe 183 px / `cfd1038a…` / exact max 73；aa2 172 px / `14893fc6…` / exact max 91）与 `|ULP|≤128` 口径。
- v2.2 清单**只服务既有 10 个回归实体**（`render_e2e` / `render_kernel_rgb` / `render_kernel_depth_seg` / `render_kernel_flags` / `render_r10_multi_world` / `render_skybox_reference` / `render_context_lifecycle` / `render_mesh_bvh` / `render_hfield_reference` / `render_shadow_reference`），在验证窗中**继续按 v2.2 判定，不得因本实体降级或改清单**。
- 本规格**不改动** criterion v2.2 的任何文本/清单/阈值；本实体若需放宽，只能走用户签名（新 spec/新判据），不能在实现或窗口内放宽。

## 8. 失败条件（写死；失败即停、不重试、不放宽）

任一条成立即整轮失败：

1. 任一台相机（`cam-persp`/`cam-hinge`）任一 world 的 rgb 有字节差、depth 有位差、seg 有不全等；
2. 两次运行 sha256 不同（任一台相机、任一 world、任一通道）；
3. **C1 < 384**（`cam-persp` rgb 变化像素，两对 world 任一）；
4. **C2 < 384**（`cam-hinge` depth 或 seg 变化像素，两对 world 任一）；
5. 采集阶段 **C1 或 C2 落在 [384, 576)**（余量不足）或 < 384（无判别力）；
6. `state.npz` 的 `light_xpos/light_xdir` 位级对齐失败；
7. 出现任何未申报的像素级放宽 / allowlist / 引用 v2.2 清单来洗通过 / 用诊断项冒充 C1 或 C2；
8. 采集确定性失败（两次采集非逐位一致）、既有参考被改动（`legacy_reference_untouched=false`）、或**两台相机快照与 §2.1 写死值不符**。

失败处置：保留现场（日志/产物原样）、停手报用户；**不重试、不改阈值、不换参考、不调相机/步进**。

## 9. 采集方案（供采集窗卡引用）

### 9.1 环境（写死）

- 离线 authoring：`target/mjwarp-parallel/venv`（Python 3.12.13）；
- `mujoco_warp` editable 装于 pinned checkout（`71da24d9`，mujoco_warp 3.15.0），**禁止 PyPI 轮子**；
- `warp-lang 1.15.0`、`mujoco 3.15.0`、`numpy 2.5.4`；
- GPU：与既有 perworld 参考同一台（RTX 4070 Ti SUPER / driver 596.36 / SM89）。

### 9.2 脚本与命令（写死）

- 脚本：`scripts/capture_render_perworld.py`（既有 `--scene` / `--out-root` / `--reference-tag` 参数化 + P0-1 refit；**双相机的「渲染行为」不需要改脚本**：`cam_active` 由 `PARAMS["cam_active"] = [cam_mode == 0]` 从**全部相机**派生为 `[true, true, false]`（3 条，`cam-track` 为 false），`nrender=2`；**相机快照字段补全属元数据层，需要改脚本**，见 §11.1 第 6 项③）。
- dry-run（零 GPU）：
  `--scene fixtures/render-bvh/scene-a-light/scene.xml --out-root fixtures/render-bvh/scene-a-light/perworld --out-dir fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040 --reference-tag nw3-light-hinge02040 --worlds 3 --hinge-deg 0,20,40 --dry-run`
- 正式采集（一次，不重试）：同上参数去掉 `--dry-run`，并要求 `--repeat 2` + 显式非空 `MJWARP_CAPTURE_AT`。
- 采集窗**必须**按**现有脚本的真实字段**核两处（不得用计划里不存在的字段自核；dry-run 计划**不含** `cam_res`/`nrender`）：
  - **dry-run 计划**：`plan.model_static.cameras` 顺序/模式/名字逐项为 `[[0, 0, "cam-persp"], [1, 0, "cam-hinge"], [2, 3, "cam-track"]]`，且 `plan.cam_active_fixed_only == [true, true, false]`；
  - **采集产物** `capture.json.context.resolved`：`nrender == 2`、`cam_id_map == [0, 1]`、`cam_res == [[320, 240], [320, 240]]`，且 `recipe.cameras` 恰 2 条（slot 0/1，含 `sensorsize`/`intrinsic`）。
  - 任一项不符即停手。

### 9.3 采集窗内自查顺序（写死）

1. **两次采集逐位一致**（逐文件 sha256，rgb/depth/seg 各 6 个 world×相机键）；
2. **C1**：`cam-persp` 相邻 world 的 `max_channel|Δrgb|≥1/255` 像素数，两对都 ≥576；
3. **C2**：`cam-hinge` 相邻 world 的 depth 与 seg 变化像素数，两对都 ≥576；
4. 元数据四项（`command`/`captured_at`/`reference_tag`/`per_world_scope`）+ **两台相机快照**（§2.4：`recipe.cameras` 2 条且含 `sensorsize`/`intrinsic`）+ `context.resolved` 的 `nrender==2` / `cam_id_map==[0,1]` / `cam_res==[[320,240],[320,240]]` 自核；
5. 既有参考 `legacy_reference_untouched = true`；
6. 以上全部通过才落库；任一步失败 → 停手报用户（不落库为验收材料、不重试、不调阈值）；
7. 采集证据（基线/floor/monitor/日志/两次一致标记/判别力计数/capture.json）提交后进入验证前置。

## 10. 验证方案（供验证窗口卡引用）

### 10.1 判据实体（实现要求，本规格不实现）

- 新增测试实体 `render_per_world_light`：读 §2 参考（**6 个 world×相机 rgb 键 + 6 depth + 6 seg + state**）→ 按 §4 对**两台相机各自**逐 world 判 → 按 §5 判灯字段 → 按 §6 参考侧自算 C1/C2 → 两跑 sha。
- 实体应同时断言「只有该 world 变化时其它 world 输出不变」的 world 隔离性（沿用 perworld S1 做法，两台相机都要），但**该断言不替代** §4–§6。
- 判据实现需 QA 只读复核（world/相机顺序、像素计数方向、阈值换算、`1/255` 比较口径、灯字段位型比较、C1/C2 不得互换、无 allowlist）。

### 10.2 窗口材料与纪律（写死）

- 新实体进入 guards：既有 perworld suite 96 条（11 exe + 83 输入 + 2 条 criterion v2.2 清单）基础上，**新增 1 个 exe**（`render_per_world_light`），**新增输入 ≤12 条**，其中**必须包含**：新 tag 5 文件（rgb/depth/seg/state/capture.json，双相机内容但仍是 5 个文件）、场景变体 `scene-a-light/scene.xml` 1、采集脚本 `scripts/capture_render_perworld.py` 1、采集元数据模块 `scripts/perworld_capture_meta.py` 1（P0-1 refit helper 所在）、备选模块 `scripts/perworld_light_prep.py` 1、本 `spec.md` 1（合计 10 条必需）；其余至多 2 条可放冻结器/证据输入。**最终条目数以验证窗卡冻结清单为准，且不得少冻结上述 10 条必需输入**；旧 suite 副本与既有冻结件**只读保留**。
- 参考体积 ≈ **11.08 MB**（11,076,054 B，5 文件），冻结清单必须如实记录字节数。
- 验证窗纪律：开窗前 5 分钟三点基线、signed floor = baseline_free − 3072 MiB（取严）、常规 monitor 60/60（util<40）、树干净、`cargo test --no-run` exit 0；窗口内**零编译**（冻结后只允许 `--features cuda-probe` 口径构建/运行）；冻结 exe live==guard==stash；单次运行、失败即停；证据归档（baseline/monitor/entity log/guards 副本 + manifest）。

### 10.3 实现形态与设备脱钩面（写死前提）

- 预期实现为 **host-only per-world 灯位姿**：产品内核灯数组是 1-D、per-world 发射时内核 `worldid` 恒 0（靠输出 adr 偏移定位），因此每 world 重传该 world 的灯数组即可，**`.cu` 零改动、内核身份不变**。
- **双相机不需要新内核路径**：两台相机共用同一 megakernel，仅按相机索引取 `cam` 数组与输出 adr；不得为此改内核。
- 若实现中发现必须改内核（如改 2-D 灯数组/单次发射/AA×perworld/多相机新路径），**立即停手报 Chief/用户**：属设备源与内核身份变更，需重算证据重放范围（M2/M3/S4 等面另行评估），不得在窗口内擅自变更。
- `AA×perworld` 仍按现状对 `nsamples>1` 显式 `Unsupported`；本规格**不覆盖** AA。

## 11. 签名清单（用户级）

| # | 事项 | 说明 |
| --- | --- | --- |
| **S-1** | **本判据规格**（`spec.md` sha256 + bytes 写进签名卡） | 参考范围（含双相机写死）、判据 A/B/C1/C2、不继承 v2.2 清单、失败条件、事实地基（spot 恒 0；两套判别力预测量） |
| **S-2** | **采集窗计划**（用户双签：计划 + 开窗前 5 分钟三点基线 + signed floor） | 含 **authoring 例外是否预批**（常规 `util<40` 不达标时是否允许 authoring-only `util<60`）；不预批则等常规阈值；**采集窗前置见 §11.1（六项）+ §12 G-L0 pilot 结论（第 7 项）** |
| **S-3** | **验证窗计划**（用户双签）+ Chief token | 窗口清单、guards 冻结（双相机参考 + 1 新 exe）、基线/floor、失败条款 |

### 11.1 S-2 采集窗前置（写死；v2 四项 + v3 两项）

S-2 签名前，下列**六项** + **§12 G-L0 pilot 结论（第 7 项）** 必须在采集脚本/采集窗卡/场景变体中落实，否则 S-2 不得签：

1. **`command` 字段**：采集脚本必须写 `command`（`{argv 逐字, cwd, rendered}`），采集后自核非空（v2 已实现，`2026-11-07-render-perworld-l-v2-capture-meta` verified）。
2. **`captured_at` 与 env**：采集窗**必须显式设置** `MJWARP_CAPTURE_AT`（非空 ISO8601），开窗前核 env，采集后自核非空（v2 已实现：缺/空在 GPU 前 exit 1）。
3. **`per_world_scope` 文案**：必须为新覆盖口径（覆盖 per-world 灯位姿 + per-world geom 位姿），采集后自核；旧口径字符串出现即拒（v2 已实现）。
4. **`recipe.task_id` / `reference_tag`**：选 **(b) 新增 `reference_tag`**（默认=参考目录名，`--reference-tag` 覆盖），`task_id`/`task_id_source` 保留（v2 已实现；v3 采集必须显式 `--reference-tag nw3-light-hinge02040`）。
5. **场景变体相机改动已 verified**（v3 新增）：`scene-a-light/scene.xml` 的 `cam-hinge` 按 §2.1 写死值加入、`cam-persp`/`cam-track`/灯/几何逐字不变；改卡 verified 且场景 sha 与采集窗材料一致；`fixtures/render-bvh/scene-a/` 零改动。
6. **采集脚本双相机行为与相机快照补全已 verified**（v3 新增）：
   - ① dry-run 显示 `model_static.cameras` 顺序/模式为 persp/hinge/track（索引 0/1/2），且 `cam_active_fixed_only == [true, true, false]`；
   - ② 参考产物 / 采集前材料显示 `nrender == 2`、`cam_id_map == [0,1]`、`cam_res == [[320,240],[320,240]]`；
   - ③ **`camera_recipe` 补 `sensorsize` 与 `intrinsic` 两字段并自核**（现脚本未产出，属本项必须落实；未补则 §9.3 第 4 步会失败）；
   - ④ 参考产物键名按 `*_cam0`/`*_cam1` 生成、两遍逐位一致逻辑不受相机数影响。

## 12. GO 清单（Chief 级）

| # | 窗口 | 前置 |
| --- | --- | --- |
| **G-L0** | **pilot（authoring，1 个，不落库；在 S-2 之前执行）** | **顺序写死：G-L0 先于 S-2**（S-2 的前置包含 pilot 结论，两者不得倒序）。前置：S-1 已签 + **场景变体相机改动 verified** + **采集脚本双相机验证 verified** + P0-1 refit verified + 材料读前预审 PASS + 开窗前 5 分钟内三点基线 + **常规 monitor 阈值（util<40、60/60）** + **floor 取严：按 §10.2 公式 `baseline_free − 3072 MiB` 现场计算并记入 pilot 材料（用户签的 signed floor 属 S-2，不是 G-L0 前置）** + 零编译。**authoring 例外（util<60）不适用于 pilot**（只在 G-L1 由 S-2 预批）；pilot 在常规阈值下达不到 monitor 条件 → 停手报用户，**不得为开 pilot 先签 S-2 放宽**。**通过条件 = C1 与 C2 两套均 `pass-margin`（两对 world 均 ≥576）**；失败即停、不重试、不落库 |
| **G-L1** | 采集窗（authoring，1 个） | S-1 + S-2 已签 + **G-L0 pilot pass-margin** + 本 spec/receipt/manifest 落库 + **§11.1 六项 + G-L0 pilot 结论（7 项）前置全部落实** + QA 只读复核 + 基线/floor 取严 + 材料贴群 |
| **G-L2** | 验证窗（1 个） | 采集证据 verified（两次采集逐位一致 + **C1 与 C2 均 ≥576**）+ 判据实体实现 verified + QA 复核 PASS + guards 冻结（含新 exe + 双相机参考 5 文件）+ 基线/floor + 零编译纪律 |

## 13. 已知风险（如实登记）

1. **只有两灯可判别**：`spot` 恒 0（几何事实）；若用户要求三灯可判别，需改场景变体 + 新 spec + 重签。
2. **双相机成本**：参考体积 ≈ **11.08 MB**（11,076,054 B）、guards 输入 10 条必需（≤12）、实体判两台相机；**这是完整覆盖两套判别力的必要成本，不做取舍**。
3. **C2 依赖 refit**：若采集/实现遗漏每 world refit，几何视角 depth/seg 会退化为 0（C2 失败）；refit 已在 P0-1 verified，但采集窗必须自核 refit 实际执行（调用计数/日志）。
4. **`cam-hinge` 近景**：相机距 tri 几何中心实测 **0.689 m**（QA 2026-11-07 复核；初稿写 ≈0.65 m 为估计值），`near` 0.025，几何在 0/20/40° 下均在视野内；若实测出现近平面裁剪/自遮挡，按 §8 停手报用户，不得现场改相机。
5. **灯驱动的阴影敏感**：`sun`/`spot` 为 castshadow 灯，灯位姿变化会改变阴影；若实测阴影数值抖动导致判据不可满足，只能停手报用户（不得改 recipe/关阴影），由用户决定是否出新参考范围。
6. **host-only 假设**：若实现被迫改内核，成本与证据重放范围显著上升（见 §10.3）。
7. **world 数少**：3 world 只证明两段位形差异被正确渲染，不构成连续动画/多 world 大规模覆盖。
8. **不覆盖 AA/单发射性能**：`AA×perworld` 仍 `Unsupported`；方案 K（单次发射）不在本规格范围。
9. **重冻代价**：新实体使 perworld suite 需重冻，期间无「已验证 exe 直接复用」状态。
10. **判据严格性**：rgb 要求 0 字节差（比 ≤1 LSB 更严），若参考与产品存在已知位级差，本规格不会放行。
11. **饱和区不敏感**：既有参考 47.4% 像素至少一通道已饱和；C1 计数从 59,805 上界降到约 5.58 万（仍 ≫576），但饱和区本身不携带灯效信息——这是场景曝光事实，不在本规格内调整。

## 14. 变更控制

- 本文件是**签名对象**：任何改动（参考范围/tag/world 数/hinge 角/**相机集合或位姿/fovy/resolution**/recipe/字段/容差/门槛/C1·C2 定义/失败条件/实现形态前提）都产生新 sha → **需用户重签**；
- 签名后本文件与 `receipt.json`、`manifest.txt` 按 sha256 冻结；实现卡只能引用、不得改写本规格；
- 历史不回改：既有 perworld S1 证据、L v1 参考（含其 B 类降级限定）、R06 多帧证据、criterion v2.2 均保持原样。

### 14.1 修订记录

| 版本 | 日期 | 修订内容 | 依据 |
| --- | --- | --- | --- |
| v1 | 2026-11-06 | 初版草案（sha256 `1821be98…` / 16,625 B） | 备选准备片 `2026-11-06-render-perworld-l-v2-light-prep` |
| v2 | 2026-11-06 | ① §2.2 recipe 拆分「仅 parameters 逐字同 / scene_xml 与 script sha 必然变」；② §2.3 改为实际编码（rgb/depth/seg 每 world 独立键、state 按 nworld 首维堆叠）；③ §10.2 guards 条数明确；④ 新增 §11.1 采集窗前置四项；⑤ §12 G-L1 引用上述前置。**判据语义、阈值、失败条件、非声明均未变。** | QA 读前预审 PASS（O1/O4/O5 签名前改、O2/O3 进前置） |
| **v3** | **2026-11-07** | ① **双相机**：`cam-persp` 保留（照明视角）+ 新增固定相机 `cam-hinge`（几何视角，pos/quat/fovy 30/320×240 写死），`cam_active=[true,true,false]`；② **判别力拆两套**：§6 改 C1（`cam-persp` rgb，`max_channel|Δrgb|≥1/255` 计数）+ C2（`cam-hinge` depth/seg 变化像素），阈值仍 384/576、两套都必须实测达标；③ §2.3 布局/体积改双相机（≈11.07 MB）；④ §3 事实地基改为两套判别力的 CPU 预测量 + 视野事实（附 L v1 参考 B 类降级依据）；⑤ §8 失败条件扩到两台相机与 C1/C2；⑥ §9.3 自查顺序加入 C1/C2 与双相机自核；⑦ §10.2 guards 必需输入 9→10（加 `perworld_capture_meta.py`）；⑧ §11.1 前置 4→6（加场景相机卡、采集脚本双相机验证）；⑨ §12 新增 **G-L0 pilot**（C1&C2 双 pass-margin 才继续）并同步 G-L1/G-L2；⑩ §13 风险补双相机成本、C2 依赖 refit、近景相机、饱和区。**判据 A/B、阈值数值、criterion v2.2 非继承、非声明不变。** | 用户 2026-11-07 拍板「rgb 变化 + 换相机都做，两套判别力同时覆盖」；P0-1 refit verified（`e34df3b7`）；L v1 参考 B 类降级（`c318cebb`） |
| **v3（预审修订 rev2）** | **2026-11-07** | ① **O1**：§2.1 写死 `cam-hinge` **插入位置/索引顺序**（cam-persp 之后、cam-track 之前；索引 0/1/2 = persp/hinge/track），改位置视为改参考范围；② **O2**：§9.2/§9.3 自查改用**脚本真实字段**（dry-run `model_static.cameras` + `cam_active_fixed_only`；采集后 `context.resolved` 的 `nrender`/`cam_id_map`/`cam_res` + `recipe.cameras` 2 条），删去对计划中不存在的 `cam_res`/`nrender` 字段的自核；③ **O3**：§2.4/§11.1-6 明确要求 `camera_recipe` **补 `sensorsize`/`intrinsic`** 并自核；④ O4 depth 倍数改正 62×/59× 并补 QA 复算差异；⑤ O5 体积统一 **11.08 MB（11,076,054 B）**；⑥ O6 §12 明示 G-L0 **在 S-2 之前**执行、authoring 例外不适用于 pilot；⑦ O7 manifest 补 `# base dir:` 行，§13 近景距离改实测 0.689 m。**C1/C2 定义、阈值、失败条件、非声明不变。** | QA 读前预审（2026-11-07，PASS-with-observations 0 阻断；O1–O3 建议签名前改） |
| **v3（预审修订 rev3）** | **2026-11-07** | QA 第二轮最小复核发现 **2 条阻断项（同一根因「相机条目数口径」）+ 5 条低观察**，全部签名前修正：① **`cam_active` 前后值**改为「旧参考 `[true,false]`（scene-a 2 台）→ `[true,true,false]`（scene-a-light 3 台，`cam-track` 不启用）」，纠正 §2.2 与 rev2 行误写的 2 条形式；② **`cam_res` 口径残留**：§2.1「`cam_res` 三元组」改「仅启用相机 2 条」，receipt 采集前置第 6 项改与 §9.2 逐字同口径（`nrender==2` / `cam_id_map==[0,1]` / `cam_res==[[320,240],[320,240]]` / `recipe.cameras` 2 条含 `sensorsize`/`intrinsic`）；③ §9.2 措辞区分「**渲染行为**不需改脚本」与「相机快照字段补全需改脚本（§11.1-6③）」；④ §12 G-L0 的 floor 改为**按 §10.2 公式现场计算并记入 pilot 材料**（用户签的 signed floor 属 S-2，消除循环依赖）；⑤ §11.1 增补「**第 7 项 = §12 G-L0 pilot 结论**」，S-2 前置与 G-L1 表述同步；⑥ receipt `go_list.G-L0` 同步 rev2 的两条约束、`criteria.render_per_world.depth` 删除 ULP 括注（改为 f32 位型全等、无 ULP 容差）。**C1/C2 定义、阈值、失败条件、非声明不变。** | QA 第二轮最小复核（2026-11-07，2 阻断项 + 5 低观察；建议签名前修） |

## 15. 非声明

- 本规格是**可签草案**：**未经用户签名不生效**；签名前不得据此开采集窗/验证窗、不得开始实现承诺。
- 不构成 perworld L v2 启动；不产参考、不宣称 per-world 灯位姿已覆盖、不宣称与上游实现等价、不宣称性能、不宣称任何渲染数值结论、不作对外发布依据。
- §3 的两套判别力数值是 **CPU 预测量**，不是渲染实测；pilot 与采集窗实测才算数。
- 「本规格通过」仅意味着该 3-world 双相机参考上的判据被满足，不等于 perworld L v2 全链/整族通过。
