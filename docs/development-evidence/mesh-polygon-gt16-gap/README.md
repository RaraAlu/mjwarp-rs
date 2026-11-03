# mesh polygon 容量差距评估（U8/U7）与设计方案（评估稿，未实现）

任务：`2026-10-27-mesh-polygon-gt16-device-gap`（Arch 2026-10-27 登记；2026-11-03 更新为 A1 后的评估稿）
性质：**差距评估 + 设计方案**。不改设备代码、不改判据/容差/参考/夹具、不开 GPU 窗口、不冻结；
任何设备代码或判据变更需**用户批准**。评估时点：HEAD `5be57e86`（2026-11-03 23:1x +08:00）。

> 本文件替换 2026-11-03 12:07 的占位稿；占位稿只写了范围与两个方案名，没有事实与结论。

## 0. 结论摘要（TL;DR）

1. 卡登记时的 U8 差距（设备固定 `G5_MC_MESH_FACE_MAX=16` → prism-20 回退单接触，上游给 4）**已被
   A1（`2026-10-31-g10-p5-mesh-cap20`，提交 `307219fe`）离线修复为 20**，clip 缓冲 32→40。
   **但 A1 没有跑 GPU 窗口**：U8 设备 `count=4` 是上游参考的期望，不是设备实测；A1 证据明确
   「不宣称 U8 通过」。
2. A1 后仍有**两维残留差距**，且只覆盖到 20/10：
   - **face 维度**：单 polygon 顶点数 > `20` → `g5_mc_mesh_face` 返回 0 → 回退单接触。
   - **degree 维度（A1 完全没碰）**：单顶点相邻 polygon 数 > `10` → `g5_mc_mesh_normals` /
     `g5_mc_mesh_edge_normals` 返回 0 → 回退单接触。三角化网格的**扇心/极点顶点**（锥顶、UV 球极点）
     的 degree 等于扇面数，24/32/64 段很常见，因此这一维比「>20 边 n-gon 顶帽」更容易在真实资产里触发。
   - 上游两维都是**模型级动态容量**（pinned `collision_convex.py` L1262-1289、`types.py` L1661-1662）：
     `npolygonmax = max(m.npolygonmax, 16|4)`、`nmeshdegmax = max(m.nmeshdegmax, 3)`，设备是写死常量。
3. 影响面：现有 M2/M3/S4 判据夹具（cube：polygon 3、degree 5）**不触发**；U8/U7 在 S4 是
   **record-only**（不进 pass/fail 分母）；渲染/其它家族零影响。A1 已改变 collision 内核身份
   （LF source `254,005 B / 9b7435ab…` → `254,263 B / 12d50d68…`，PTX 3,883,740 → 3,883,769 B，
   6 入口）→ **任何后续 collision 窗口都必须全新冻结**；上游参考无需重采（pinned raw 未变）。
4. 方案（详见 §3–§6）：
   - **A 提高固定上限**（建议 20→32 / 10→32 一次做完）：成本最低、可预测；仍有界（>32 仍回退）。
   - **B 动态容量 + 分派**：B2a 全局 scratch（逐值对齐上游语义，显存代价大）/ B2b 容量档入口分派
     （折中：模型级选档 + 超档显式拒绝）。
   - **C 宿主前置诊断**（最小缓解）：用已有 mesh 数组在宿主算出模型级上限，超界时把「静默回退单接触」
     变成显式诊断；设备零改动、无新冻结、无窗口。
5. 建议顺序：**C（0.5 人日）→ A（1 窗口）→ B2b（若用户需要「一般 mesh 覆盖」）**；B2a 只在必须
   任意模型容量逐值对齐上游时采用。两条硬边界：**判据/容差/参考不变**；**不得宣称 mesh 一般覆盖**。

## 1. 事实基础（A1 之后的当前状态）

### 1.1 设备侧容量与回退路径

mesh 多接触只在 `contact.cu` 的 multiccd 分支里发生（`want_multi && has_polytope && multiccd_idx > -1`
且两侧类型 ∈ {BOX, MESH, CYLINDER}，`contact.cu:212-215`）。scratch 是**每线程本地数组**
（`contact.cu:218-228`），容量由两个宏与调用方一致：

| 限位 | 定义位置 | 当前值 | 超界行为 |
| --- | --- | --- | --- |
| `G5_MC_MESH_FACE_MAX`（单 polygon 顶点数，上游 `npolygonmax`） | `src/physics/collision/cuda/multicontact_geom.cu:385` | **20**（A1 由 16 提升） | `g5_mc_mesh_face`（`multicontact_mesh.cu:180-182`）`nvert > 20` → `return 0` |
| `G5_MC_MESH_DEG_MAX`（单顶点相邻 polygon 数，上游 `nmeshdegmax`） | `src/physics/collision/cuda/multicontact_geom.cu:384` | **10**（A1 未动） | `g5_mc_mesh_normals` 1-顶点特征（`multicontact_mesh.cu:120-122`）与 `g5_mc_mesh_edge_normals`（`:148-150`）`v1_num > 10` → `return 0` |

回退语义（逐行核对）：

1. 助手返回 0 → `multicontact.cu:194-198`（面恢复失败）或 `:110-159`（法线/边恢复失败）→
   `return out;`，而 `out.n` 初始为 **1**、`witness1[0]/witness2[0]` 就是 EPA 的 `x1/x2`
   （`multicontact.cu:37-45`）。
2. `contact.cu:233` 判定 `mc.n >= 1` 恒成立 → 采纳 `n=1` 的单接触结果。
   **因此「回退」= 1 个 EPA 单接触，不是 0 接触**；U8 的 `device_count=1` 就是这么来的。
3. 触发点：face 维度在 `g5_mc_mesh_face` 每次恢复 polygon 面时检查；degree 维度只在
   `feature_dim == 1`（EPA 特征退化到单个 mesh 顶点）的法线/边法线路径检查；
   `feature_dim == 2` 的边路径不检查 degree（`intersect1` 最多 2 个面）。
   换言之：**>cap 的 n-gon 只要被恢复成接触面就触发 face 回退；顶点度超限只在单顶点特征路径触发**。

scratch 清单（`contact.cu:218-228`，A1 后）：`polygon[40]`、`clipped[40]`、`pnormal[20]`、
`pdist[20]`、`idx1/idx2[10]`、`n1/n2/endvert[10]`、`face1/face2[20]`。全部为 per-thread 本地数组，
**不进外部 ABI / 产品 JSON / launch 签名**——这是 A1「零 ABI」结论的依据。

### 1.2 上游动态容量（pinned revision `71da24d956378a87a703b6e1442b13aec0c4ac29`）

- 容量规则：`collision_convex.py` L1262-1289（冻结副本
  `docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/collision_convex.py`）：
  `npolygonmax` 先取 4（有 box-box 时）；multiccd 且含 cylinder 时 `max(m.npolygonmax, 16)`；
  仅 mesh 组合时 `max(m.npolygonmax, minval)`；`nmeshdegmax = max(m.nmeshdegmax, 3)`。
- scratch 分配：同文件 L1399-1419，全部是 `(naccdmax, 2*npolygonmax)` / `(naccdmax, npolygonmax)` /
  `(naccdmax, nmeshdegmax)` 的**全局数组**，宽度随模型动态，不是每线程固定数组。
- 模型字段来源：`types.py` L1661-1662（`npolygonmax: int`、`nmeshdegmax: int`），由 MJCF 编译期从
  mesh 资产算出；L1126-1127 是字段注释。
- 结论：上游对任意 mesh 资产都不会因「polygon 边数/顶点度」整体回退；设备固定 20/10 是**语义差异**，
  不是上游也有的限制。

### 1.3 A1 已做/未做

- 已做（提交 `307219fe`，7 文件）：`contact.cu` 6 处数组 16→20 / 32→40；`multicontact_geom.cu`
  face 16→20；`clip.rs` 镜像 `pn/pd[20]`、`polygon/clipped[40]`；2 条 CPU 单测
  `polygon_clip_accepts_twenty_vertex_face1/face2`（`gjk/multicontact/mod.rs:287`、`:311`）；
  注释契约 5 处。**degree 仍 10**。
- 内核身份已变：LF canonical source `254,005 B / 9b7435ab…` → `254,263 B / 12d50d68…`；
  PTX `3,883,740 B / 6 入口` → `3,883,769 B / 6 入口`
  （`docs/development-evidence/g10-p5-mesh-cap20/logs/nvrtc-identity.txt`；旧 CRLF 测量值已作废）。
  ⇒ 后续任何 collision 窗口必须新冻结，旧 guards 不复用。
- **未做**：GPU 窗口（A1 README §5 明写「后续窗口等用户签」）；U8 设备 `count=4` 未实测；
  degree 维度未动；超界回退仍是静默行为。

### 1.4 最近一次设备实测记录（S4，仍是 A1 之前的内核）

`docs/development-evidence/g10-p5-branch-closure-validation/README.md` L232/L236：

- U7 `mesh-limits-cone-box`：`device_count=1` / `ref_count=1`，device/ref dist、pos 近似相同
  ⇒ **不能**据此判「设备与上游一致」，只能记「count 偶合」（S4 定死口径）。
- U8 `mesh-limits-prism-box`：`device_count=1` / `ref_count=4`，ref dist 全为 `-1.0e-2`，
  ref pos 为 `(±0.05, ±0.05, 0.045)`（盒 4 角）；那时 `FACE_MAX=16`，20-gon 顶帽触发回退。
- 两组都是 **record-only**，不进 pass/fail 分母；判据不变。

## 2. 触发条件与影响面

### 2.1 数学触发条件

一次 mesh multiccd 接触要走满 ≤4 接触，需要：① pair 含 mesh 且 multiccd 生效；② GJK/EPA 的
最终穿透特征落在 mesh 面/顶点上；③ 恢复该特征所需的 polygon 顶点数 ≤ `FACE_MAX` 且（单顶点路径时）
`polymapnum ≤ DEG_MAX`。任一超限即回退 1 个 EPA 单接触。
换言之差距的触发条件是「**mesh 资产存在 >20 顶点的 polygon，或 >10 条面共享的顶点**」，
与接触配置（穿透深度/姿态）无关——只要该面/顶点成为最终接触特征就会发生。

### 2.2 夹具几何（实测，取几何定义直接统计）

| 夹具 | verts | faces | max 面顶点数 | max 顶点度 | 当前（20/10）是否触发 |
| --- | --- | --- | --- | --- | --- |
| `fixtures/g10-p5-mesh-multicontact/cube.obj`（570 B / `90a8a990…`） | 8 | 12 | 3 | 5 | 否 |
| `fixtures/g10-p5-mesh-multicontact/prism-20.obj`（1819 B / `a4477d3b…`） | 40 | 42（40 三角 + 2×20-gon） | 20 | 4 | 否（A1 前 face=16 时触发 → 即 U8） |
| `fixtures/g10-p5-mesh-multicontact/fan-cone-24.obj`（1042 B / `4bc34a51…`） | 25 | 25（24 三角 + 1×24-gon） | **24** | **24**（锥顶） | **face + degree 两维都触发 → 即 U7** |

场景：U8 = prism-20 顶帽（z=+0.05）压 box 底面（z=0.04），穿透 0.01，上游 4 接触
（`mesh-limits-prism-box.xml` / `1bee35cf…`）；U7 = 24 边锥压 box（`mesh-limits-cone-box.xml` /
`6ce3552f…`），上游 count 也是 1（30 边锥面与 box 的实际几何只产生 1 个接触），所以 U7 记 count
不具判别力，必须同时记 dist/pos。

### 2.3 真实网格里谁会踩到

- **degree 维度（更容易）**：三角形扇的公共顶点 degree = 扇面数。锥顶、UV 球两极、圆盘/圆柱端面
  的扇心都是这种结构；24/32/64 段的低多边形资产会直接超 10。上游对这类顶点是动态容量，设备固定 10。
- **face 维度**：未三角化的程序化棱柱/圆柱端面（n-gon 顶帽）、CAD 导出的共面大面片；>20 边上限。
- 三角化资产通常 face ≤4 不会超 face 维度，但 pole/apex 仍可能超 degree 维度。
- 本仓库自己的夹具只有 prism-20 / fan-cone-24 触发；产品 fixture 集与渲染家族不受影响（渲染不用
  collision multicontact）。

### 2.4 对既有 M1/M2/M3/S4 判据与证据的影响

- **判据不变**：M1（dispatch）、M2/M3（cube fixture，degree 5 / polygon 3-4，完全不触发）与 S4
  （U7/U8 record-only）都不需要改判据、容差、参考或夹具。
- **证据身份**：A1 已改 collision 内核 source/PTX sha ⇒ 旧的 collision 冻结与 guards 作废；
  M2/M3/S4 的历史 verdict 绑定各自窗口的旧内核身份，**不能与 A1 之后的内核混用**。任何窗口重放
  必须新 suite + 新冻结，不能「代码没变所以直接沿用」。
- **期望值变化面**：在 A1 之后重新跑 S4 的 limits 组，U8 的设备记录会从 `count=1` 期望变成 4
  （上游参考不变）；U7 设备仍可能单接触（degree 超限）或产生多接触（取决于实际接触特征是否落在
  锥顶顶点路径上）——两者都只能按 record-only 记，不能升级为 pass/fail（除非用户另行批准）。
- **参考无需重采**：上游参考是 pinned GPU raw（`upstream-mesh-limits-gpu.json`，
  `b6abb8fa5820c103f6d880936d6ae95fa60035e5b9ce49a0364b15de1def19ad`，4581 B），
  fixture 未变，重放时只做 sha 绑定，不需要重新捕获。
- **文档/台账**：`g10-p5-acceptance-map` README L140、`unc3` 系列仍写 `FACE_MAX=16`（A1 未回写），
  属历史证据不可改；若将来实现，建议由治理片以 append 方式更新 living map，而不是改写历史记录。

## 3. 方案 A：提高固定上限（建议 face 32 / degree 32）

### 3.1 档位与 per-thread 预算

per-thread 本地数组大小（按 `contact.cu:218-228` 结构，`v3` 3×f32、int/float 4 B）：

| face / degree | 数组浮点+整型预算 | per-thread 本地字节 | 相对现状 |
| --- | --- | --- | --- |
| 20 / 10（现状，A1 后） | 550 f32 + 20 i32 | 2,280 B | 1.0× |
| 24 / 24 | 792 + 48 | 3,360 B | 1.47× |
| **32 / 32（建议）** | 1,056 + 64 | **4,480 B** | 1.96× |
| 64 / 64 | 2,112 + 128 | 8,960 B | 3.93× |

口径：`22×face + 11×degree` 个 f32 + `2×degree` 个 i32（`polygon/clipped=2×2×face`、
`pnormal=3×face`、`pdist=face`、`face1/face2=2×3×face`；`n1/n2/endvert=3×3×degree`）。这些数组在
`g5_ccd_pair_write` 内、会被 inline 进 `mjwarp_collision_contact` 与 `mjwarp_collision_contact_multi`
两个入口（`product.cu:219`、`:411`），因此**帧开销对该入口的所有线程生效**（包括本次 pair 不是
mesh 的线程）。当前 2.2 KiB 的帧在 32/32 下变成 4.4 KiB：是否影响 occupancy/吞吐必须在实现片用
`ptxas -v`（本地可跑，报告 stack frame / spill / local）与窗口内 sweep 实测，**不得口头判断**。

选 32 的理由：程序化网格最常见的段数是 16/24/32/64；32 覆盖 24/32 段端面与 pole/apex 顶点；
64 档帧翻倍到 8.8 KiB，收益（64 段资产）相对成本不划算，留给方案 B。

### 3.2 改动清单（若用户批准）

| # | 文件 | 改动 |
| --- | --- | --- |
| 1 | `src/physics/collision/cuda/multicontact_geom.cu:384-385` | `G5_MC_MESH_DEG_MAX` 10→32、`G5_MC_MESH_FACE_MAX` 20→32（同一次改，避免只修 face 后 U7 仍回退） |
| 2 | `src/physics/collision/cuda/contact.cu:216-228` | 注释容量口径 20/10→32/32；`polygon/clipped[64]`、`pnormal/pdist[32]`、`idx1/idx2[32]`、`n1/n2/endvert[32]`、`face1/face2[32]` |
| 3 | `src/physics/collision/cuda/multicontact.cu:13-15` | 调用方容量注释同步 |
| 4 | `src/physics/collision/cuda/multicontact_clip.cu:10-11` | clip 缓冲/平面数组口径 20/40→32/64 |
| 5 | `src/physics/collision/cuda/multicontact_mesh.cu:10-14` | 容量边界注释同步（U7 不再是 10 回退） |
| 6 | `src/physics/collision/gjk/multicontact/clip.rs:37-38, :48-50` | 宿主镜像 `pn/pd[32]`、`polygon/clipped` 64 |
| 7 | `src/physics/collision/gjk/multicontact/mod.rs` | 新增 32 顶点 face1/face2 CPU 单测（镜像 `:287`/`:311` 的 20 顶点用例） |

不动：`product.cu` 6 个入口（数量不变）、`CollisionArgs`/launch 签名/产品 JSON 字段、
`src/runtime/collision_gpu.rs` 的入口常量；唯一的身份变化是 collision **kernel source/PTX sha**
（⇒ 新冻结）。

### 3.3 成本、风险、前置

- 成本：离线实现+单测+`ptxas`/NVRTC 身份观测 ≈ 0.5–1 人日；1 个 GPU 窗口（新 suite：U8 断言 +
  U7 record + M1/M2/M3/S4 回归重放，全新冻结）；证据 ≈ 0.5 人日。
- 风险：① 仍是有界覆盖（>32 回退），不消除静默回退，需配合方案 C；② 本地帧 ~2× 影响
  contact/contact_multi 两个入口的全部线程，occupancy/吞吐必须先测（无性能声明，但不能引入未记录的
  回退风险）；③ 设备代码变更需用户批准；④ 新内核身份 ⇒ 既有 collision 窗口证据全部变成「旧内核」，
  历史记录保留但不得混用。
- 前置：用户批准档位（24/24 vs 32/32）与「U8 是否升级为 pass/fail」（A1 悬置项）。

## 4. 方案 B：动态容量 + 分派

### 4.1 B2a 全局 scratch（逐值对齐上游语义）

- 宿主侧：从**已有输入数组**算模型级上限，不新增模型字段——`mesh_polyvertnum` 的最大值 = `npolygonmax`，
  `mesh_polymapnum` 的最大值 = `nmeshdegmax`（两者都已在
  `src/runtime/collision_gpu/types.rs` 的 `CollisionGpuInputs` 与上传计划里）。
- 设备侧：按上游形状分配 9 个 `(naccdmax, 宽度)` 全局数组（对齐 `collision_convex.py:1399-1419`），
  指针 + 两个容量作为参数进入 collision args struct（`product.cu` 的 `mjwarp_collision_args`），
  内核按 EPA ccd 槽号索引。当前 `g5_claim_ccd_slot`（`contact.cu:115-123`）只返回 bool 且丢弃
  `ccdid`，需要改为保留槽号。
- 语义：最接近上游（宽度随模型动态，任意模型容量，受 int / 显存限制）。
- 代价：显存 = `naccdmax × 4 × (25·face + 11·degree)` B（每个 ccd 槽 2,440 B @20/10、
  4,608 B @32/32、9,216 B @64/64；`naccdmax=8192` 时分别约 20 / 36 / 72 MB），会进入窗口的
  显存峰值/floor 判定面；host packing / abi / meta / buffers 全链路改动；内核 source 与 args
  布局都变 ⇒ 新冻结；实现 3–5 人日 + 1–2 窗口（卡 note 的预估与此一致）。
- 风险：改动面最大；显存增量在 T4 窗口可能逼近峰值判据；本仓库以 per-thread 本地数组为既有实现，
  迁移到全局 scratch 会改变访存局部性，吞吐影响需实测。

### 4.2 B2b 容量档入口分派（折中，推荐 B 路线）

- 把 multiccd 入口做成**多档**（如 (20,10)/(32,32)/(64,64)），执行期内核代码不变，只是容量宏在
  多份入口里展开；宿主按模型算出的 `npolygonmax/nmeshdegmax` 选入口（入口加载与选择在
  `src/runtime/collision_gpu.rs:436-446` 附近；当前 6 个入口常量为 `collision_gpu.rs:68-72`）。
- 超最高档 → **宿主显式拒绝**（`Unsupported`/诊断，不静默回退）；档位边界写进错误与证据。
- 代价：artifact 入口数 6→6+N、编译时间/字节数增加；`abi_tests.rs` 与 `multicontact_tests.rs`
  的入口声明断言要更新（入口集合在 `abi_tests.rs:136-141`）；每入口有独立帧，**非 mesh pair 不会
  被迫承担最高档的帧开销**（这是相对方案 A 的主要优势）；仍是「档位覆盖」，不是任意容量；实现
  2–3 人日 + 1 窗口 + 新冻结。
- 风险：档位选择错误的兜底必须显式（拒绝而不是静默）；入口集合进冻结/guards，窗口材料要枚举全部入口。

## 5. 方案 C：宿主前置诊断（最小缓解，可独立先行）

- 做法：载入/plan 时用已有 mesh 数组算 `npolygonmax/nmeshdegmax`，与设备固定容量比较；超界时当前
  产品是**静默回退单接触**，改为显式诊断（记录/告警/`Unsupported`，策略由用户/Arch 定）。
- 代价：设备零改动、无新冻结、无 GPU 窗口（CPU 测试 + 四门禁）；≈0.5 人日。
- 收益：消灭「静默近似」的口径风险；给出真实资产的触发统计；为 A/B 提供选档依据。
- 限制：不改任何数值行为；属于产品行为面（从静默到显式），需要批准。

## 6. 方案对比与建议

| 方案 | 覆盖 | 设备代码 | kernel args/外部 JSON | 新冻结+窗口 | 预计人日 | 主要风险 |
| --- | --- | --- | --- | --- | --- | --- |
| A face32/degree32 | 固定上界（≤32） | 7 文件（宏/数组/注释/镜像/测试） | 否 / 否 | 是 / 1 窗口 | 1–1.5 | 有界覆盖、本地帧 2×、静默回退仍在 |
| B2a 全局 scratch | 任意模型容量 | 大（args+scratch+槽号） | args 是 / 否 | 是 / 1–2 窗口 | 4–6 | 显存增量、改动面最大 |
| B2b 档位入口 | 档位上界（≤64 等） | 中（入口多份+选档） | 否 / 否 | 是 / 1 窗口 | 2.5–4 | 档位边界、入口集合进冻结 |
| C 宿主诊断 | 不改数值，只显式化 | 无 | 否 / 否 | 否 / 不占窗口 | 0.5 | 产品行为变更需批准 |

建议：
1. **先做 C**（若用户接受「超界显式诊断」），把静默近似关掉，并拿到真实资产触发面。
2. **A 作为近期主线**（用户批准 32/32 档 + 1 个窗口）：一次消除两维回退到 32，成本最低。
3. **B2b 作为「一般 mesh 覆盖」承诺时的升级路径**；B2a 不主动建议（显存与改动面大，只有在必须
   任意模型容量逐值对齐上游时才值得）。
4. U8 是否从 record-only 升为 pass/fail、以及是否把「mesh 一般覆盖」列入对外声明，属用户签名级议题。

## 7. 共性影响（A/B/C 任一落地时）

- 判据/容差/参考/夹具**零改动**；U7/U8 维持 record-only 直到用户明确升级。
- 设备源码改动 ⇒ collision 内核身份变化 ⇒ 既有 guards 全部作废，新窗口必须新 suite + 新冻结；
  M1/M2/M3/S4 回归需在新内核下重放（不能据「只改容量」口头推断不变）。
- 上游参考无需重采（pinned raw 不变），只做 sha 绑定。
- 历史证据目录不改写；living ledger（`g10-p5-acceptance-map` 等）以 append 方式更新，并如实写
  「容量上界/触发面」，不得出现「mesh 容量缺口已清 / 上游等价 / 精度达标」类宣称。

## 8. 不宣称

本评估**未跑 GPU**、未改代码、未冻结、未开窗。不宣称：U8 已通过、U7 已覆盖、mesh 容量缺口已清、
设备与上游等价、任意 mesh 覆盖、性能结论。A1 的设备行为（U8 count 1→4）仍是**未验证的期望**。

## 9. 证据链与复现（只读）

| 事实 | 位置 |
| --- | --- |
| face 限位 20 / degree 限位 10 | `src/physics/collision/cuda/multicontact_geom.cu:384-385` |
| face 回退 / degree 回退 | `src/physics/collision/cuda/multicontact_mesh.cu:180-182`、`:120-122`、`:148-150` |
| 回退=单接触（n=1, x1/x2） | `src/physics/collision/cuda/multicontact.cu:37-45`、`:194-198`；`contact.cu:233` |
| scratch 清单 20/10 版 | `src/physics/collision/cuda/contact.cu:218-228` |
| 上游动态容量 / 分配 | frozen `collision_convex.py:1279-1289`、`:1399-1419`；`types.py:1661-1662` |
| A1 改动与身份 | `docs/development-evidence/g10-p5-mesh-cap20/README.md`、`logs/nvrtc-identity.txt`；提交 `307219fe` |
| S4 设备记录（A1 前） | `docs/development-evidence/g10-p5-branch-closure-validation/README.md:232,236` |
| 上游 limits 参考 | `docs/development-evidence/g10-p5-branch-closure-capture/upstream-mesh-limits-gpu.json`（`b6abb8fa…`/4581 B） |
| 夹具几何 | `fixtures/g10-p5-mesh-multicontact/{cube,prism-20,fan-cone-24}.obj`（见 §2.2 的 sha/bytes） |
| 机器可读事实 | 本目录 `facts.json`；`receipt.json` |

复现方式（本次评估实际执行）：只读 grep/读文件 + 对 3 个 OBJ 做面顶点数/顶点度统计；未运行任何
GPU/cargo 命令（评估本身），设备行为结论全部引用既有证据。

## 10. 待决事项（交 Arch/用户）

1. 是否做 C（超界诊断策略：计数/告警/`Unsupported`），以及是否作为首个落地片。
2. A 的档位：24/24 vs 32/32（建议 32/32）与是否同窗口把 degree 一起升。
3. U8 是否升为 pass/fail；U7 保持 record-only 还是也升级。
4. B 是否入 backlog、B2b 是否作为「一般 mesh 覆盖」的承诺路径。
5. 任何设备改动前：用户批准 + 新窗口计划（新 suite/新冻结/U8 断言方式）。
