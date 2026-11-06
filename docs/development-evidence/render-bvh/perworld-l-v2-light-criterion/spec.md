# per-world L v2 灯判据规格（可签草案）

- 任务：`2026-11-06-render-perworld-l-v2-light-criterion`（owner Arch）；v2 修订卡 `2026-11-06-render-perworld-l-v2-light-criterion-spec-v2`
- 日期：2026-11-06
- 版本：**v2**。取代 v1（sha256 `1821be98953e834265e6e8b3a8e6e363f0b59306b9625ed96b6a39cb570ab9e8` / 16,625 B）。v2 修订 = QA 读前预审 O1/O4/O5（签名前文本改正）+ O2/O3（写入 §11 S-2 / §12 G-L1 采集窗前置）；**判据语义、阈值（384/576）、失败条件、不继承 v2.2 清单、事实地基均未变**。
- 状态：**待用户签名、未生效**（签名对象 = 本文件 sha256 + bytes；用户选定 perworld L-v2 方向后，由 Chief 按 **v2 sha** 整理签名卡）
- 前置：备选准备片 `2026-11-06-render-perworld-l-v2-light-prep`（`b0b8cc02`）已落库：`scene-a-light` 场景变体 + 采集脚本 `--scene`/`--out-root` 参数化 + 三 world 灯位姿前置自算
- 上游依据：pinned checkout `target/mjwarp-parallel/upstream` @ `71da24d9`（mujoco_warp 3.15.0）；既有 per-world geom 参考 `fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/`（L v1 验证窗使用）
- 边界：**规格文本**。不采集、不开窗、不动 `fixtures/`、不改产品代码、不改任何生效判据/容差/criterion v2.2/既有冻结件。**本草案未经用户签名不生效。**

---

## 1. 适用与边界

- 本规格只判 **per-world 灯位姿（L v2 灯）**：在 nworld>1 时，每 world 的灯位姿不同，渲染输出必须与该 world 的参考一致，且 world 间差异必须真实可见（判别力门槛）。
- 本规格**替代不了** per-world geom 位姿判据（L v1 已 verified）与既有单帧/多帧判据；三者各自独立判定。
- **不宣称**：不宣称 per-world 灯位姿已覆盖、不宣称与上游实现等价、不宣称性能、不宣称 R06/G10 等其它家族结论、不作对外发布依据。
- 本规格通过 ≠ perworld L v2 全链通过：实现卡、采集证据、验证窗三者齐备才谈「该实体在该参考上通过」。

## 2. 参考范围（写死）

### 2.1 场景变体（写死）
- 文件：`fixtures/render-bvh/scene-a-light/scene.xml`（备选准备片已落库，sha256 见其证据）。
- 与 `fixtures/render-bvh/scene-a/scene.xml` **同源**，差异仅限三灯挂载方式：`sun`/`lamp`/`spot` 三灯以 **MJCF 嵌套**方式放入 `<body name="tri_body">`（MJCF 没有 `<light body="...">` 属性）。
- 三灯的 `type/pos/dir/diffuse/specular/ambient/attenuation/cutoff/exponent/softness/castshadow/active/bulbradius/intensity/range` **逐字不变**；`cam-persp` 保持 mode 0、`cam-track` 保持 mode 3。
- `fixtures/render-bvh/scene-a/` 既有文件**零改动**。

### 2.2 参考 tag 与位形（写死）
| 项 | 值 |
| --- | --- |
| tag | **`nw3-light-hinge02040`** |
| 输出根 | `fixtures/render-bvh/scene-a-light/perworld/` |
| 参考目录 | `fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/` |
| worlds | **3**（nworld=3） |
| hinge 角（度） | **0 / 20 / 40**（写入各 world 的唯一 hinge `tri-hinge` qpos） |
| 相机 | **仅固定相机 `cam-persp`（mode 0）**，320×240 = **76,800 px/world**；`cam-track`（mode 3）不启用（`cam_active=[true,false]`） |
| recipe.parameters | **仅 `parameters` 块**逐字同既有 perworld 参考 `nw3-hinge02040/capture.json.parameters`（`use_fast_math=false`、`use_shadows=true`、`samples_per_pixel=1`、`enabled_geom_groups=[0,1,2,3]`、`background_color=[0,0,0,1]`、`shadow_light_fraction=0.3`、`flex_render_smooth=true`、`use_precomputed_rays=true`、其余同源） |
| recipe.scene_xml / recipe.script | **sha256/bytes 必然变化**（场景变体 `scene-a-light/scene.xml` + 采集脚本参数化），**不得**要求与旧参考相同；`recipe` 的 `nworld/hinge_deg/seed/repeat/versions/upstream_revision` 与旧参考同口径 |
| 变化机制 | 只在**模型层外部注入**：各 world 的 hinge qpos = 0/20/40 度；灯随 `tri_body` 刚性转动 → `light_xpos/light_xdir` 逐 world 不同。**不做物理步进/时间积分/加外力**，world 间差异 100% 来自位形注入 |

### 2.3 产物布局与编码（写死；按既有 perworld 参考实际编码，v2 修正）

目录：
```
fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/
  rgb.npz / depth.npz / seg.npz     # 每 world 独立键（见下）
  state.npz                        # 按 nworld 首维堆叠（见下）
  capture.json                     # 位形、recipe、pinned 环境、文件 sha256、两次采集逐位一致标记、per_world_scope
```

**rgb/depth/seg：每 world 一个独立键**（`cam0` 是唯一启用相机 `cam-persp`；world 号写在键名里，**不是** `[nworld, ...]` 堆叠）：
| 文件 | 键 | shape | dtype |
| --- | --- | --- | --- |
| `rgb.npz` | `rgb_w{0,1,2}_cam0` | `(240, 320, 3)` | f32 |
| `depth.npz` | `depth_w{0,1,2}_cam0` | `(240, 320)` | f32 |
| `seg.npz` | `seg_w{0,1,2}_cam0` | `(240, 320, 2)` | i32 |

**state.npz：无 world 键，数组以 nworld 为首维堆叠**（nworld=3）：
| 键 | shape | dtype |
| --- | --- | --- |
| `qpos` | `(3, 8)` | f32 |
| `geom_xpos` | `(3, 7, 3)` | f32 |
| `geom_xmat` | `(3, 7, 3, 3)` | f32 |
| `light_xpos` | `(3, 3, 3)` | f32（nworld × nlight × 3） |
| `light_xdir` | `(3, 3, 3)` | f32（nworld × nlight × 3） |

体积估算（按既有 `nw3-hinge02040` 实测）：rgb 2,765,524 B + depth 922,336 B + seg 1,843,924 B + state 2,486 B + capture.json ≈ 8.8 KB ≈ **5.55 MB/tag**。

### 2.4 确定性与 provenance（写死）
- **两次采集逐位一致**：`--repeat 2`，两次运行的每文件 sha256 相等，否则采集失败。
- `capture.json` 必含：`nworld`、`hinge_deg`、`recipe`、`parameters`、pinned 环境（python 3.12.13 / warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4）、pinned commit `71da24d9`、每文件 sha256、两次采集对比结果、采集时间、命令。
- `capture.json.per_world_scope` 必须写明本参考覆盖「per-world 灯位姿 + per-world geom 位姿」，并注明设备路径的灯语义前提（见 §10.3）。

## 3. 事实地基：三灯判别力（写死，不得省略）

备选准备片已用 CPU `mj_forward`（不建 CUDA device、不渲染）逐 world 实测（数值见其证据 `light_pose_precheck`）：

| 灯 | 类型 | 差异键（world 1 / world 2 相对 world 0） | 结论 |
| --- | --- | --- | --- |
| `sun` | directional | xdir 夹角 **10.745878447° / 21.255926664°** | **有判别力** |
| `lamp` | point | xpos 位移 **0.222378165 m / 0.437999482 m** | **有判别力** |
| `spot` | spot | **0 / 0**（`pos=(0,0,2)`、`dir=(0,0,-1)` 落在 `tri_body` 的 z 旋转轴上） | **恒 0，无判别力（几何事实）** |

- **本参考只有 `sun`/`lamp` 两灯具备跨 world 判别力**；`spot` 的恒定是场景几何决定，不是实现缺陷，也不得在判据里当作「不同」。
- 若用户要求三灯均可判别：属**改参考范围**（需改场景变体的灯位/朝向或换旋转轴）→ 必须出新 spec、新 sha、用户重签；本规格不覆盖。
- 上述数值是**前置自算**（host CPU），不替代 §6 的渲染侧判别力门槛；两者都要满足。

## 4. 判据 A：逐 world 渲染（写死，严格口径）

对每个 world（0/1/2）的 `cam-persp` 输出：

1. **rgb：位级 0 字节差**（`max_byte_diff = 0`，即逐字节全等；沿用 perworld S1 实际口径，不使用 ≤1 LSB 放宽）；
2. **depth：位级全等**（`mismatched_pixels = 0`，同时 `max_ulp ≤ 4` 且 `beyond_ulp = 0` 由位级全等自动满足）；
3. **seg：逐像素 `(id, type)` 全等**；
4. **两次运行 sha256 相等**（rgb/depth/seg 各自、逐 world）。

补充：
- 若实测出现任何 rgb 字节差或 depth 位差 → **失败**（见 §8），**不得**引用 criterion v2.2 清单或任何 allowlist 洗成通过。
- 判据相机只有 `cam-persp`；不存在「其它相机只作诊断」的放宽空间。

## 5. 判据 B：灯字段数据面（写死）

- 参考 `state.npz` 的 `light_xpos`（`[nworld, nlight, 3]` f32）与 `light_xdir`（同形）必须与产品侧逐 world 数据**位级对齐**（f32 位型相等，**0 例外**）。
- 允许的实现获取方式：产品侧按 world 从自身位姿/灯定义重算，或直接消费采集侧 `state.npz`；**两条路都必须给出位级相等**，不得用「近似相等」替代。
- 若产品侧灯语义与参考不同源（例如仍用单 world 灯数组），本判据会直接失败——这是**预期**行为，不得为其开 allowlist。

## 6. 判据 C：world 间判别力（写死；参考侧自算，不用产品侧数据）

**变化定义**（在参考侧计算，避免循环论证）：对相邻 world 对（0→1、1→2）：
- **depth 变化像素** = `bits(depth_world_k) != bits(depth_world_{k-1})` 的像素数；
- **seg 变化像素** = `seg_pair(world_k) != seg_pair(world_{k-1})` 的像素数。

**验收判据（写死）**：两个相邻 world 对都要满足 depth 变化像素 ≥ **384** 且 seg 变化像素 ≥ **384**（= 0.5% × 76,800）。

**采集前置门槛（留余量）**：depth 与 seg 变化像素都需 ≥ **576**（= 1.5 × 384）才允许落库并进入验证；
- 实测落在 **[384, 576)** → 判「余量不足」，**采集失败、停手报用户**，不得改阈值；
- 实测 < 384 → 判「无判别力」，**拒绝通过**（不是通过）。

**退化保护**：若参考 world 间变化 < 384，整轮判「无判别力、拒绝通过」；禁止现场调阈值、禁止用产品侧数据替代参考侧自查。若需换位形（例如 hinge 角加大或改灯），属**改规格**：必须出新 spec、新 sha、用户重签，阈值 0.5% 不变。

## 7. 与 criterion v2.2 的关系（写死）

- **本实体的判据不继承 criterion v2.2 的两条已签清单**（recipe 183 px / `cfd1038a…` / exact max 73；aa2 172 px / `14893fc6…` / exact max 91）与 `|ULP|≤128` 口径。
- v2.2 清单**只服务既有 10 个回归实体**（`render_e2e` / `render_kernel_rgb` / `render_kernel_depth_seg` / `render_kernel_flags` / `render_r10_multi_world` / `render_skybox_reference` / `render_context_lifecycle` / `render_mesh_bvh` / `render_hfield_reference` / `render_shadow_reference`），在验证窗中**继续按 v2.2 判定，不得因本实体降级或改清单**。
- 本规格**不改动** criterion v2.2 的任何文本/清单/阈值；本实体若需放宽，只能走用户签名（新 spec/新判据），不能在实现或窗口内放宽。

## 8. 失败条件（写死；失败即停、不重试、不放宽）

任一条成立即整轮失败：

1. 任一 world 的 rgb 有字节差、depth 有位差、seg 有不全等；
2. 两次运行 sha256 不同；
3. 相邻 world 的 depth 或 seg 变化像素 < 384（无判别力）；
4. 采集阶段 world 间变化落在 [384, 576)（余量不足）或 < 384（无判别力）；
5. `state.npz` 的 `light_xpos/light_xdir` 位级对齐失败；
6. 出现任何未申报的像素级放宽 / allowlist / 引用 v2.2 清单来洗通过；
7. 采集确定性失败（两次采集非逐位一致）或既有参考被改动（`legacy_reference_untouched=false`）。

失败处置：保留现场（日志/产物原样）、停手报用户；**不重试、不改阈值、不换参考、不调步进**。

## 9. 采集方案（供采集窗卡引用）

### 9.1 环境（写死）
- 离线 authoring：`target/mjwarp-parallel/venv`（Python 3.12.13）；
- `mujoco_warp` editable 装于 pinned checkout（`71da24d9`，mujoco_warp 3.15.0），**禁止 PyPI 轮子**；
- `warp-lang 1.15.0`、`mujoco 3.15.0`、`numpy 2.5.4`；
- GPU：与既有 perworld 参考同一台（RTX 4070 Ti SUPER / driver 596.36 / SM89）。

### 9.2 脚本与命令（写死）
- 脚本：`scripts/capture_render_perworld.py`（备选准备片已加 `--scene` / `--out-root`；默认路径不变）。
- dry-run（零 GPU）：
  `--scene fixtures/render-bvh/scene-a-light/scene.xml --out-root fixtures/render-bvh/scene-a-light/perworld --out-dir fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040 --worlds 3 --hinge-deg 0,20,40 --dry-run`
- 正式采集（一次，不重试）：
  同上参数去掉 `--dry-run`，并要求 `--repeat 2`。

### 9.3 采集窗内自查顺序（写死）
1. 两次采集逐位一致（逐文件 sha256）；
2. 计算相邻 world 的 depth/seg 变化像素（参考侧自算）；
3. 都 ≥576 → 落库；任一 <576 → 停手报用户（不落库为验收材料）；
4. 既有参考 `legacy_reference_untouched = true`；
5. 采集证据（基线/floor/monitor/日志/两次一致标记/capture.json）提交后进入验证前置。

## 10. 验证方案（供验证窗口卡引用）

### 10.1 判据实体（实现要求，本规格不实现）
- 新增测试实体（如 `render_per_world_light`）：读 §2 参考 → 按 §4 逐 world 判 → 按 §5 判灯字段 → 按 §6 参考侧自算判别力 → 两跑 sha。
- 实体应同时断言「只有该 world 变化时其它 world 输出不变」的 world 隔离性（沿用 perworld S1 做法），但**该断言不替代** §4–§6。
- 判据实现需 QA 只读复核（world 顺序、像素计数方向、阈值换算、灯字段位型比较、判别力算法、无 allowlist）。

### 10.2 窗口材料与纪律（写死）
- 新实体进入 guards：既有 perworld suite 96 条（11 exe + 83 输入 + 2 条 criterion v2.2 清单）基础上，**新增 1 个 exe**（`render_per_world_light`），**新增输入 ≤12 条**，其中**必须包含**：新 tag 5 文件（rgb/depth/seg/state/capture.json）、场景变体 `scene-a-light/scene.xml` 1、采集脚本 `scripts/capture_render_perworld.py` 1、新模块 `scripts/perworld_light_prep.py` 1、本 `spec.md` 1（合计 9 条必需）；其余至多 3 条可放冻结器/证据输入。**最终条目数以验证窗卡冻结清单为准，且不得少冻结上述 9 条必需输入**；旧 suite 副本与既有冻结件**只读保留**。
- 验证窗纪律：开窗前 5 分钟三点基线、signed floor = baseline_free − 3072 MiB（取严）、常规 monitor 60/60（util<40；authoring 例外仅采集窗可预批）、树干净、`cargo test --no-run` exit 0；窗口内**零编译**（冻结后只允许 `--features cuda-probe` 口径构建/运行）；冻结 exe live==guard==stash；单次运行、失败即停；证据归档（baseline/monitor/entity log/guards 副本 + manifest）。

### 10.3 实现形态与设备脱钩面（写死前提）
- 预期实现为 **host-only per-world 灯位姿**：产品内核灯数组是 1-D、per-world 发射时内核 `worldid` 恒 0（靠输出 adr 偏移定位），因此每 world 重传该 world 的灯数组即可，**`.cu` 零改动、内核身份不变**。
- 若实现中发现必须改内核（如改 2-D 灯数组/单次发射/AA×perworld），**立即停手报 Chief/用户**：属设备源与内核身份变更，需重算证据重放范围（M2/M3/S4 等面另行评估），不得在窗口内擅自变更。
- `AA×perworld` 仍按现状对 `nsamples>1` 显式 `Unsupported`；本规格**不覆盖** AA。

## 11. 签名清单（用户级）

| # | 事项 | 说明 |
| --- | --- | --- |
| **S-1** | **本判据规格**（`spec.md` sha256 + bytes 写进签名卡） | 参考范围、判据 A/B/C、不继承 v2.2 清单、失败条件、事实地基（spot 恒 0） |
| **S-2** | **采集窗计划**（用户双签：计划 + 开窗前 5 分钟三点基线 + signed floor） | 含 **authoring 例外是否预批**（常规 `util<40` 不达标时是否允许 authoring-only `util<60`）；不预批则等常规阈值 |
| **S-3** | **验证窗计划**（用户双签）+ Chief token | 窗口清单、guards 冻结、基线/floor、失败条款 |

### 11.1 S-2 采集窗前置（写死；QA 读前预审 O2/O3，签名前固定）

S-2 签名前，下列四项必须在采集脚本/采集窗卡中落实，否则 S-2 不得签：

1. **`command` 字段**：当前 perworld 采集脚本**不写** `command`；采集前必须补写（记录实际命令行），采集后自核非空。
2. **`captured_at` 与 env**：当前 `captured_at` 依赖 `MJWARP_CAPTURE_AT`，而既有参考里为空串 `""`；采集窗**必须显式设置 `MJWARP_CAPTURE_AT`**（非空 ISO8601），开窗前核 env，采集后自核 `captured_at` 非空。
3. **`per_world_scope` 文案**：既有参考的 `per_world_scope` 仍是旧口径（`does_not_cover: per-world light poses`）；采集前必须同步改为**新参考覆盖口径**（覆盖 per-world 灯位姿 + per-world geom 位姿），采集后自核文案与覆盖声明一致。
4. **`recipe.task_id` 旧卡 id**：既有参考 `recipe.task_id` 仍是旧采集卡 id（`2026-10-30-render-perworld-capture-authoring`）；采集前二选一处置——**参数化 task_id** 或 **新增 `reference_tag` 字段**，处置决定与前后值写入采集窗证据，避免 provenance 混淆。

## 12. GO 清单（Chief 级）

| # | 窗口 | 前置 |
| --- | --- | --- |
| **G-L1** | 采集窗（authoring，1 个） | S-1 + S-2 已签 + 本 spec/receipt/manifest 落库 + **§11.1 四项采集窗前置全部落实（command / captured_at+env / per_world_scope 文案 / task_id 或 reference_tag）** + 采集脚本参数化已 verified + QA 只读复核 + 基线/floor 取严 + 材料贴群 |
| **G-L2** | 验证窗（1 个） | 采集证据 verified（两次采集逐位一致 + world 间 ≥576）+ 判据实体实现 verified + QA 复核 PASS + guards 冻结（含新 exe + 新参考）+ 基线/floor + 零编译纪律 |

## 13. 已知风险（如实登记）

1. **只有两灯可判别**：`spot` 恒 0（几何事实）；若用户要求三灯可判别，需改场景变体 + 新 spec + 重签。
2. **灯驱动的阴影敏感**：`sun`/`spot` 为 castshadow 灯，灯位姿变化会改变阴影；若实测阴影数值抖动导致判据不可满足，只能停手报用户（不得改 recipe/关阴影），由用户决定是否出新参考范围。
3. **host-only 假设**：若实现被迫改内核，成本与证据重放范围显著上升（见 §10.3）。
4. **world 数少**：3 world 只证明两段位形差异被正确渲染，不构成连续动画/多 world 大规模覆盖。
5. **不覆盖 AA/单发射性能**：`AA×perworld` 仍 `Unsupported`；方案 K（单次发射）不在本规格范围。
6. **重冻代价**：新实体使 perworld suite 需重冻，期间无「已验证 exe 直接复用」状态。
7. **判据严格性**：rgb 要求 0 字节差（比 ≤1 LSB 更严），若参考与产品存在已知位级差，本规格不会放行。

## 14. 变更控制

- 本文件是**签名对象**：任何改动（参考范围/tag/world 数/hinge 角/相机/recipe/字段/容差/门槛/失败条件/实现形态前提）都产生新 sha → **需用户重签**；
- 签名后本文件与 `receipt.json`、`manifest.txt` 按 sha256 冻结；实现卡只能引用、不得改写本规格；
- 历史不回改：既有 perworld S1 证据、R06 多帧证据、criterion v2.2 均保持原样。

### 14.1 修订记录

| 版本 | 日期 | 修订内容 | 依据 |
| --- | --- | --- | --- |
| v1 | 2026-11-06 | 初版草案（sha256 `1821be98…` / 16,625 B） | 备选准备片 `2026-11-06-render-perworld-l-v2-light-prep` |
| **v2** | 2026-11-06 | ① §2.2 recipe 拆分「仅 parameters 逐字同 / scene_xml 与 script sha 必然变」；② §2.3 改为实际编码（rgb/depth/seg 每 world 独立键、state 按 nworld 首维堆叠）；③ §10.2 guards 条数明确（新增 1 exe + 输入 ≤12 条、必须含 9 条必需输入）；④ 新增 §11.1 采集窗前置四项（command / captured_at+env / per_world_scope 文案 / task_id 或 reference_tag）；⑤ §12 G-L1 引用上述前置。**判据语义、阈值、失败条件、非声明均未变。** | QA 读前预审 `2026-11-06-render-perworld-l-v2-light-criterion-check`（`e9b1181b`，PASS-with-observations 0 阻断）：O1/O4/O5 签名前改、O2/O3 写进采集窗前置 |

## 15. 非声明

- 本规格是**可签草案**：**未经用户签名不生效**；签名前不得据此开采集窗/验证窗、不得开始实现承诺。
- 不构成 perworld L v2 启动；不产参考、不宣称 per-world 灯位姿已覆盖、不宣称与上游实现等价、不宣称性能、不宣称任何渲染数值结论、不作对外发布依据。
- 「本规格通过」仅意味着该 3-world 灯位姿参考上的判据被满足，不等于 perworld L v2 全链/整族通过。
