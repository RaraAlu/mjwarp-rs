# U7/U8 提升后的机器可判定判据 · g10-p5「4 partial + 6 uncovered」口径表 · A2 三片范围

任务：0 窗文档证据（用户已签 U8/U7 裁决，选项 A：`degree>10` / `polygon>16` 从 record-only 提升为 pass/fail；A2 容量 **20/10 → 32/32**）。
日期：2026-11-12（北京时间 01:2x，CPU-only，零 GPU 窗口）
写域：仅本目录（`docs/development-evidence/plan/u7-u8-promotion/`）；不改产品代码/判据/容差/参考/fixtures、不动任何卡、不 commit。
性质：**判据口径 + 缺口台账 + 范围建议**；不宣称 U7/U8 已通过、不宣称 mesh 容量缺口已清、不宣称 G10@P5 整族通过。

## 0. 输入事实（仓内实测）

| 输入 | 事实 |
| --- | --- |
| 用户裁决 | U7（`degree>10`）与 U8（`polygon>16`）提升为 pass/fail；A2 容量 32/32（签名级，已生效） |
| A1 现状 | `docs/development-evidence/g10-p5-mesh-cap20/README.md`：容量 16→20 已实现（U8 路径），**U7 不动**（`G5_MC_MESH_DEG_MAX` 仍 10）；代码已改但**无 GPU 验证窗**（«GPU 验证窗排在用户签窗口后»） |
| 当前代码 | `multicontact_geom.cu:384-385` `DEG_MAX 10 / FACE_MAX 20`；`contact.cu:216-228` scratch（`polygon/clipped[40]`、`pnormal/pdist[20]`、`idx1/idx2/n1/n2/endvert[10]`、`face1/face2[20]`）；宿主 `gjk/multicontact/clip.rs`（`pn/pd[20]`、`polygon/clipped vec![..;40]`）；`runtime/collision_gpu/types.rs:141/143` 常量 20 / 10 |
| U7 参考 | 上游 pinned GPU：`upstream-mesh-limits-gpu.json`（`docs/development-evidence/g10-p5-branch-closure-capture/`，**sha `b6abb8fa5820c103f6d880936d6ae95fa60035e5b9ce49a0364b15de1def19ad`** / 4,581 B；pinned `71da24d9`；两次捕获逐位一致）：`mesh-limits-cone-box` **count=1**、dist `-0.010000001639127731` |
| U8 参考 | 同上：`mesh-limits-prism-box` **count=4**、dist 全 `-0.009999997913837433`、pos `(±0.05, ±0.05, 0.045)`（4 角与 20-gon 顶帽中点） |
| S4 设备实测（record-only 时） | `g10-p5-branch-closure-validation` §10：U7 `device_count=1` / `ref_count=1`（dist `-1.0000005e-2`）；U8 **`device_count=1` / `ref_count=4`**（dist `-9.999998e-3`）——U8 真实功能差距 |
| 案例与拓扑 | `fixtures/g10-p5-mesh-multicontact/`：`fan-cone-24.obj`（1,042 B / `4bc34a51a60d76469a68c64e6132cfd1f5713072488ea8deefd44a4ca517f73b`；25 verts / 25 polys / max_degree 24 / max_polygon 24）、`prism-20.obj`（1,819 B / `a4477d3b080479e7356ac14bcd67c33272b2771eadd40b14fe3f40360e92cee4`；40 verts / 22 polys / max_polygon 20 / max_degree 3）；模型 `mesh-limits-cone-box.xml` `6ce3552f…`、`mesh-limits-prism-box.xml` `1bee35cf…` |
| 比较器 | `tests/support/mesh_multicontact_compare_support.rs`（M1/M2/M3 判据实现，见 §1.3） |
| 计数状态 | `g10-p5-acceptance-map`（map.json，S5 更新 + 伞卡 note 2026-11-05）：12 条 requirement = met 8 / **partial 4** / unmet 0；uncovered **6** |

## 1. U7/U8 提升后的机器可判定判据

### 1.1 case 集与分母

- **U7 case**：`mesh-limits-cone-box`（`fan-cone-24`：`max_degree=24 > 10`，A2 后 24 ≤ 32 属 in-range）。
- **U8 case**：`mesh-limits-prism-box`（`prism-20`：`max_polygon_vertices=20 > 16`，A2 后 20 ≤ 32 属 in-range）。
- 提升后两条**进入 pass/fail 分母**（分母 = 2），不再以「限制存在」记录；U7 与 U8 各自独立判，不混算。
- 数据源固定：设备回读接触数组（32 数组、6 入口产物）↔ `upstream-mesh-limits-gpu.json`（`b6abb8fa…`）。**不得**用 CPU precheck 当数值判据（S3 已定：precheck 只对 count/`qpos_bits`/拓扑）。

### 1.2 判据（零放宽，复用既有比较器）

pass 当且仅当以下全部成立：

1. `nacon == ref.count`（**零容差**；U8 期望 4，U7 期望 1）；
2. 行按结构键 `(worldid, geomcollisionid)` 排序后逐行配对（不按行序猜）；
3. 数值字段：**位级优先**；非位级时按既有口径 `dist` 4 ULP@自身量级（`ref==0` → 4 ULP@1.0）、`pos` 4·ULP(参考三轴最大绝对值)、`frame` 4 ULP@1.0；
4. 零容差字段：`friction`、`solref`、`solreffriction`、`solimp`、`includemargin`、`adhesion`、`worldid`、`type`、`dim`、`geom`、`geomcollisionid`；
5. `overflow == 0`（全行）；`efc_address == -1`（参考未记录，设备侧不变量）；
6. `contact.geom` 顺序按 pinned 类型序（box 6 < mesh 7 → `[1,0]`；设备不做归一化，由调用方按规范化顺序传入）。

**U7 附加（防偶合，QA O9 结论）**：U7 上游 `count=1` 与设备单接触回退 `count=1` 可能巧合相同 ⇒ U7 的 pass **必须同时满足 `dist`/`pos` 判据**，只比 count 判 pass 视为无效。

fail 条件（任一）：count 不等 / 结构键无法配对 / 任一字段超容差 / `overflow≠0` / `efc_address≠-1`。

### 1.3 「限制语义 vs 实际触发」边界口径

| 区间 | 判法 | 证据要求 |
| --- | --- | --- |
| **in-range（degree ≤ 32 且 polygon ≤ 32）** | 按 §1.2 判 pass/fail；**必须走 multicontact clip 路径**（不得以「限制存在/回退一致」替代） | 设备 32 数组回读；U7 需 dist/pos 全等以排除回退偶合；`nface==0` 防御不得触发（count 与参考同且字段全等即间接证明） |
| **over-range（degree > 32 或 polygon > 32）** | **host 容量门**：`runtime/collision_gpu` 的模型级容量校验（`npolygonmax`/`nmeshdegmax` 与设备常量比较）必须返回确定的容量错误（机器可判：见 `inputs_from_snapshot.rs` 的 capacity violation 测试；A2 后常量 32/32）；**device 直调路径**的回退行为（`multicontact_mesh.cu` L120/L148 `v3_num > DEG_MAX → return 0`、L180 `nvert > FACE_MAX → return 0` → 单接触回退）只记录、**record-only、不进分母** | 新增边界 fixture（建议 `fan-cone-33`、`prism-33`，或等价合成）；host 拒绝错误码/`MeshCapacityDimension`；设备回退 count 记录；**不写等价、不写已覆盖** |
| **guard 保留** | A2 后 guard 仍是 defense-in-depth（`G5_MC_MESH_DEG_MAX`/`FACE_MAX` = 32）；in-range case 不应触发 | 以 §1.2 字段全等间接证明未回退；不得用「guard 存在」替代 case 判据 |

> 边界口径一句话：**16 <（degree 或 polygon）≤ 32 的 case 判 pass/fail；> 32 记「限制存在 + host 拒绝机器可判」，不进分母、不宣称与上游一致。**

### 1.4 数据来源清单（判定时逐项引用）

1. 上游参考：`docs/development-evidence/g10-p5-branch-closure-capture/upstream-mesh-limits-gpu.json`（`b6abb8fa…`）+ 同目录 README §7.3（U8 几何更正：20-gon 顶帽，非「≤16 顶点侧面」）；
2. 设备实测：A2 验证窗（待开）产出的 32 数组 + `contact.geom` 顺序断言；旧 S4 记录（U8 device 1 vs ref 4）作为「提升前基线」引用；
3. 判据实现：`tests/support/mesh_multicontact_compare_support.rs`（§1.2 的代码出处）+ `tests/g10_p5_mesh_multicontact_m2_cuda.rs`/`_m3_cuda.rs` 的用法；
4. 容量实现/边界：`multicontact_geom.cu`、`contact.cu`、`multicontact_mesh.cu`、宿主 `clip.rs`、`runtime/collision_gpu/types.rs` + `inputs_from_snapshot.rs` 容量测试；
5. 身份/冻结：A2 后 collision 内核身份（source+PTX sha）与所有窗口 guards（见 §3 A2-2）。

## 2. g10-p5「4 partial + 6 uncovered」逐条口径表

来源：`docs/development-evidence/g10-p5-acceptance-map/README.md` + `map.json`（S5 更新）+ 伞卡 `2026-10-11-roadmap-g10-p5` note（2026-11-05：met 8 / partial 4 / unmet 0 + uncovered 6）。
「提升后如何判」一列回答：U7/U8 提升是否让该行可判 pass、以及判 pass 还缺什么。**结论先行：U7/U8 提升只影响 UNC-1/UNC-3 中 U7/U8 两条（从记录改判），其余行不因此变成 pass。**

| # | 对象（id） | 现状 | 提升后如何判 | 还缺什么证据 |
| --- | --- | --- | --- | --- |
| P1 | G6-1.2 地形接触（设备 hfield，**partial**） | 主判据（hfield-sphere 8/8、hfield-capsule 7/7 全字段位级）来自 G5 交付片自身证据；但 `HFIELD(32)` 的 `overflow=32` 实测数字只在 QA 独立复核（`g10-p5-gpu-kernels-independent-check`），G5 交付片 README §9 只有实现映射 | 不受 U7/U8 提升影响；仍按原判据（count/字段位级 + overflow 语义） | G5 交付片自身窗口包含 `overflow=32` 实测数字（或 Chief 裁定「QA 独立复核可作同级引用」）；本片不改 |
| P2 | G6-1.7 越界容量截断（`naconmax`/`naccdmax`，**partial**） | 截断行为仅 QA 独立复核（verified）以自造 case 通过；G5 交付片自身无截断 case | 不受 U7/U8 提升影响；A2-3 重放矩阵可顺带把「截断 case」纳入同一次验收窗口（若窗口含该 case） | G5 交付片自身截断 case 数字；本片只建议纳入 A2-3 矩阵 |
| P3 | G6-2.1 模块与写入范围（**partial**） | 设备三缺口已补：NVRTC launcher 已 closed（限定语：默认选择层仍 Host）、primitive 组合层 13/13、multicontact box/cyl + convex-vs-mesh + mesh-mesh（M3 `f1e5cda1`）；仍缺 primitive 分支层 3 类、mesh 分支层逐条、默认产品管线接线口径 | 不受 U7/U8 提升影响（U7/U8 是容量/分支，不是模块范围） | primitive 分支 3 类 case；默认产品管线接线口径由 W9/默认管线片处置 |
| P4 | G6-2.4 NP01/NP02→NP03/NP04（**partial**） | NP03 f32 侧完成；native 接触字段按用户 B 只作诊断；其它字段 5% 红线全量未完成；NP04 只表示「已记录」 | 不受 U7/U8 提升影响；U7/U8 判据不能替代 NP 门禁 | NP01/NP02 verified（NP5 系列卡）+ NP03 其它字段全量 |
| U1 | UNC-1 multicontact 设备版（**partial**） | 已覆盖：box/cylinder（314/314、391/392）、convex-vs-mesh（4/4 位级）、mesh-mesh（4/4）；未覆盖：一般 mesh（`nmesh>1` 通用）、**degree>10 / polygon>16（U7/U8，record-only）**、mesh 分支层逐条、hfield-capsule、`efc_address`（设备全 -1）、UNC-6 深穿透、跨平台 | **受影响**：U7/U8 从「仅限制存在」改为按 §1 判；若 A2 验证窗按 §1 PASS，则该两条从 UNC-1 未覆盖清单移除（**UNC-1 仍 partial**，因其它未覆盖项不动） | A2 验证窗（fresh freeze）设备 32 数组 + §1 全字段判据 PASS；U7 需 dist/pos 全等；其它未覆盖项各自证据（不在本片） |
| U2 | UNC-2 NVRTC 产品 launcher（**closed**，限定语） | 默认构建 `cuda` 默认特性、真实设备入口被调用、三态回退不 panic；**默认选择层仍 Host、Device 需显式选择** | 不适用（已 closed，保留限定语） | 无（限定语保持；若未来默认走 Device 需另立片） |
| U3 | UNC-3 primitive narrowphase 设备版（**partial**） | 组合层 13/13 各有上游 GPU case；分支层 U1/U3/U4/U5/U6/U9 计「本窗验收覆盖」，U2 不可达/防御，**U7/U8 仅限制存在**；仍未覆盖 3 类（plane_cylinder degenerate x 轴回退、plane_ellipsoid 非上游退化保护、plane-box 其余 5 角点）；债务：sphere-box/capsule-box/box-box 分支级未逐条、capsule_capsule 平行轴未触发 | **部分受影响**：UNC-3 表内 U7/U8 的「仅限制存在」措辞随提升改为指向 §1 判据（A2 窗后判）；其余 3 类分支仍按原判据，**UNC-3 仍 partial** | A2 窗 U7/U8 判据结果入账；3 类 branch case；分支级逐条标注/债务清单（原口径不变） |
| U4 | UNC-4 Linux/T4（**unmet**） | 用户 2026-10-24 裁决 Linux 集中回测；T4 无独立证据 | 不受影响（不排窗、不宣称双平台） | 用户解冻 + 集中回测窗 |
| U5 | UNC-5 G5 窗口逐秒峰值（**unmet**） | 脚本缺陷致逐秒峰值缺失；G6 性能窗另采 0.5 s 采样，不回填 G5 历史 | 不受影响 | 不回填（G6 perf 证据另列）；不写「逐秒峰值」 |
| U6 | UNC-6 `cylinder-box` 深穿透额外采样面（**unmet**） | 4/24 单字段超 4·ULP@scale（x1 1/24 max 8.926e-7、x2 3/24 max 1.527e-7）；用户 2026-10-25 拍板 A：不修、列未覆盖；根因收窄为「派生法线差（机制未定）」 | 不受影响（U7/U8 提升不改变该声明面判据） | 若未来覆盖，需补 `face_idx`/三顶点/权重数据并另立片 |

> **能把 U7/U8 判 pass 的前提（材料包 01 §建议点名的三件）**：①§1 判据 + case 集冻结（本文件）；②A2 实现 + fresh freeze 的验证窗设备数组（A2-1/A2-2/A2-3）；③上表逐条口径（本片）——三者齐备后，**只把 U7/U8 两条从 record-only 改判 pass/fail**；UNC-1/UNC-3 的其余缺口与 G10@P5 整族状态不变。

## 3. A2 三片范围建议

### A2-1 容量 32/32 的代码面 + 边界单测清单（0 窗）

**设备（`src/physics/collision/cuda/`）**

| 文件:行 | 现值 | A2 目标 |
| --- | --- | --- |
| `multicontact_geom.cu:384` | `#define G5_MC_MESH_DEG_MAX 10` | **32** |
| `multicontact_geom.cu:385` | `#define G5_MC_MESH_FACE_MAX 20` | **32** |
| `contact.cu:218-219` | `polygon[40]` / `clipped[40]` | **64 / 64**（2×face） |
| `contact.cu:220-221` | `pnormal[20]` / `pdist[20]` | **32 / 32** |
| `contact.cu:222-226` | `idx1[10]/idx2[10]/n1[10]/n2[10]/endvert[10]` | **32**（degree 容量） |
| `contact.cu:227-228` | `face1[20]` / `face2[20]` | **32 / 32** |
| `contact.cu:216` 注释 | `npolygonmax=20`（A1） | **32（A2）** |
| `multicontact.cu:13-15`、`multicontact_clip.cu:10-11`、`multicontact_mesh.cu:12-13`、`multicontact_geom.cu:381-383` 容量注释 | 20/40/10 口径 | **32/64/32**，并写明 U7/U8 已提升、over-32 为限制 |
| `multicontact_mesh.cu:120/148/180` guard | `> DEG_MAX` / `> FACE_MAX` → 0 | **保留**（defense-in-depth，A2 后 in-range 不应触发） |

**宿主（镜像与门禁）**

| 文件 | 现值 | A2 目标 |
| --- | --- | --- |
| `src/physics/collision/gjk/multicontact/clip.rs` | `pn/pd [20]`、`polygon/clipped vec![..;40]` | `[32]`、`vec![..;64]`；**新增容量前置检查** `nface1 > 32 || nface2 > 32 → 返回空结果`（避免越界 panic；与设备 guard 语义对齐） |
| `src/runtime/collision_gpu/types.rs:141/143` | `MESH_MULTICONTACT_FACE_MAX=20` / `DEGREE_MAX=10` | **32 / 32**（与设备宏一致；`host_limits_match_device_macros` 测试自动约束） |
| `src/runtime/collision_gpu/error.rs:27-28`、`inputs_from_snapshot.rs` 注释 | 「超过设备固定容量」措辞 | 更新为 32/32 口径 |
| ABI | 81 槽、`CollisionArgs`、launch 签名、JSON 字段 | **零改动**（A1 已确立；A2 只动 per-thread scratch/Vec 容量与常量） |

**边界单测清单（CPU，随 A2-1 落地）**

1. `polygon_clip_accepts_thirty_two_vertex_face1`：face1=32 → 写 `pn/pd[31]`，断言裁剪出接触且 `ncontact ≤ 4`；
2. `polygon_clip_accepts_thirty_two_vertex_face2`：face2=32 → 需要 64 格中间缓冲，断言有接触且 `ncontact ≤ 4`；
3. `polygon_clip_rejects_thirty_three_vertex_face`：face1/face2=33 → **不 panic**、确定性空结果（容量前置检查）；
4. `product_entry_accepts_face_32_degree_32_boundary`：模型级 `npolygonmax=32`、`nmeshdegmax=32` → `inputs_from_snapshot` 接受；
5. 反例：`face=33` / `degree=33` → 各自返回 `MeshCapacityDimension::{Face,Degree}` 容量错误（机器可判）；
6. `host_limits_match_device_macros` 沿用（自动核对常量与 `.cu` 宏）；
7. 既有 A1 两条 20-vertex 测试保留（不回退）。

> 建议 A2-1 顺带核对：`contact.cu` scratch 总量增长对每线程栈/寄存器的压力（`polygon/clipped[64]` 共 64×3×2×4 B = 1.5 KiB 局部数组），在 A2 验证窗用 `nvrtc_smoke` 记录寄存器/栈指标（不作为判据，只入证据）。

### A2-2 内核身份重采影响 / 重冻清单（0 窗）

1. **collision 内核身份**（`nvrtc_smoke` 探针，28 段装配源 + 6 入口）：
   - 现值（A1 后）：assembled source **254,263 B / `12d50d68…`**；PTX **3,883,769 B / 6 入口**（`54ab59e3…`，与 NP02 `mjwarp_collision_*` 回执一致）；
   - A2 后：source/PTX sha 与 bytes **必然变化**（常量、scratch、注释）；须新记录 before/after 并在证据里声明「A1 身份作废」；
   - **全部 GPU 窗口必须新冻结，旧 guards 不复用**（A1 README 已定此口径）。
2. **需重冻的实体集合**（S4 冻结集：10 exe + 13 fixture + 2 参考）：
   - 10 exe：`g10_p5_branch_closure_cuda`、`g10_p5_nvrtc_launcher`、`g10_p5_multicontact_cuda`、`g10_p5_primitive_cuda`、`g10_p5_primitive_plane_cuda`、`g10_p5_primitive_sphere_capsule_cuda`、`g10_p5_primitive_branches_cuda`、`g10_p5_mesh_multicontact_m2_cuda`、`g10_p5_mesh_multicontact_m3_cuda`、`stage2_equivalence_window`；
   - 13 fixture 中 mesh 5 件不变；**若 A2-1 新增边界 fixture（fan-cone-33/prism-33）则 fixture 数 13→15，并新增其 sha 守卫**；
   - 2 参考（`upstream-mesh-limits-gpu.json` 等）不变；新增 33-boundary 的上游参考需按同 pinned 环境另采（**若只记 host 拒绝，可不采设备参考**，见 §1.3）。
3. **NP02 联动**：`docs/development-evidence/np02-evidence/inventory.json` 的 6 行 `mjwarp_collision_{boxbox,contact,contact_multi,hfield,primitive,sdf}` 回执 PTX `54ab59e3…`/3,883,769 B 将失效 ⇒ A2 后需 **CPU-only 重采回执**（`np02-capture-new-kernels.py` 同法，NVRTC 12.8.93、不建上下文），更新 inventory/receipt 指针与计数口径（归 NP5-03/NP02 证据面，不在本片执行）。
4. **A1 证据身份 supersession**：`g10-p5-mesh-cap20/README.md` 身份表（254,263/12d50d68、3,883,769）与 `-check` 回执在新 A2 基线后作废；**用 addendum 追加**（不改原文），写明「A1 身份 → A2 身份」。
5. **不受影响**：render guards（如 perworld-l2-light `94478b3d…`/43,729 B 与 93 输入清单）、其它子系统冻结；**不得**把 A2 身份混入 render guards。
6. **冻结纪律**：新目录（如 `target/a2-freeze/`）、fail-stop、单写者、开窗前贴基线+floor 到群、失败即停不重试。

### A2-3 M2/M3/S4 重放矩阵与判据

| 重放项 | 目标/命令（测试入口） | 判据 | 期望/记录 |
| --- | --- | --- | --- |
| **M2 convex-vs-mesh**（2 case：`mesh-box`、`mesh-cylinder`） | `cargo test --locked --features cuda-probe --test g10_p5_mesh_multicontact_m2_cuda -- --ignored --test-threads=1` | 复用 mesh 比较器（§1.2 同口径：count 零容差、结构配对、位级优先 / 4·ULP@scale 回退；`contact.geom=[1,0]`） | 必须保持 M2 已验收结果（4/4 位级、ULP 0）；A2 不引入回退 |
| **M3 mesh-mesh**（1 case/4 接触） | `--test g10_p5_mesh_multicontact_m3_cuda`（GPU）+ `--test g10_p5_mesh_m3_case_assembly`（CPU 装配自检） | 同上；mesh-mesh 参考 `96e7895d…`（count=4） | 保持 bit_equal 4/4、ULP 0；CPU 18 条自检仍 PASS |
| **S4 branch 验收组 9 case**（U1/U3×5/U4/U5/U6） | `g10_p5_branch_closure_cuda` 冻结 exe（A2 后新冻） | 原 S4 判据不变：`nacon==ref_count`、`failures=0`、位级/≤4 ULP 口径 | 9/9 `failures=0` 保持；不得因容量改动回退 |
| **S4 limits U7/U8（本片提升）** | 同上 exe 的 limits 组 | **改判**：按 §1.2 逐字段判 pass/fail；U7 需 dist/pos 全等（防偶合） | U7 期望 count=1 且字段全等；U8 期望 **count=4**（A1/A2 前设备为 1，属提升前基线） |
| **A1 CPU 回归**（2 条 20-vertex） | `cargo test --locked --lib`（multicontact 模块） | 原断言 | 保持 PASS，不回退 |
| **A2 边界（新增）** | §3 A2-1 的 7 条 CPU 测试 | 见 A2-1 | 全 PASS；33-boundary 不 panic、host 容量拒绝可判 |
| **NP02 回执**（CPU-only） | `np02-capture-new-kernels.py` 同法重采 6 条 collision | 回执 file/PTX hash 通过、6 入口 | 新 PTX sha 入 NP02 证据；计数结构不变 |

**窗口建议**：A2 验证 = **1 个 GPU 验证窗**（非 authoring；fresh freeze；monitor 60/60、floor 取严、fail-stop、单写者、零 Rust 编译、开窗前基线+计划贴群）。窗口内：跑 10 冻结 exe + A2 边界 case；产出 U7/U8 设备 32 数组与 `contact.geom` 断言；`overflow=0`/`efc_address=-1` 不变量全查；guard 无 abort。**不得**在窗口内声称 G10@P5 整族通过或 U8 提升「已验收」——只按 §1.2 判 U7/U8 两条。

## 4. 非声明与边界

- 本文件是判据口径/缺口台账/范围建议，**不是**验收结论：U7/U8 仍待 A2 实现 + 新冻结验证窗；
- 不改产品代码/判据/容差/参考/fixtures；不动任何卡（不注册/不 start/不 verified）；不 commit；
- 不宣称 mesh 容量缺口已清、不宣称 G10@P5 整族通过、不宣称 NP01–NP04 放行；
- 「4 partial + 6 uncovered」计数口径以 `g10-p5-acceptance-map`（S5 更新）为准，本表不改变计数。
