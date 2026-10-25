# G10@P5 G0b：夹具与参考独立复核（QA）

- 任务：`2026-10-25-g10-p5-fixtures-independent-check`（依赖 `2026-10-25-g10-p5-fixtures`，G0b）
- 复核对象：**`f0e43bb`**（G0b 交付，父 `87bc042`）
- 复核基线：开工 HEAD `f0e43bb`；复核期间 HEAD 前移到 `1fc9782`（只含 `.normify` 任务登记，产品路径 0 个；`git diff f0e43bb..HEAD` 只有 2 个 `.normify/changes/*.json`）
- 环境：Windows；MuJoCo **3.12.0**（`target/toolchains/mujoco-3.12.0/package`，MSVC 编译 QA 自己写的 C++ dump）；pinned venv Python **3.12.13** / MuJoCo **3.15.0** / warp **1.15.0** / mujoco_warp **3.15.0** / numpy **2.5.4**；**未使用 GPU**（捕获重采用 warp 的 `cpu` 后端）
- 结论：**0 阻断，G0b 从 QA 侧可定稿**。`octree parity` 位级成立、MJB provenance 成立、既有参考审计 12/12 逐项一致、10 个新捕获可复现且 offset/字段/顺序观察项如实、nworld=2 spike 的代码依据成立。一般 1 条 + 建议 4 条（均非阻断，见 §9）。

> 口径：本复核**只做独立复核与复跑**，不修产品代码、不改交付夹具/G0b 原证据；本目录只写复核证据。**未跑全量 G01**（本片无论文构建变更，按约定只跑证据门禁 fmt + json + forbid_claim）。

## 1. 独立口径（不复用 `scripts/authoring/g10-p5/` 的 dump/比较/捕获代码）

| 项 | QA 自写 harness | 独立性 |
| --- | --- | --- |
| 3.12 侧字段 dump | `qa_mjb_dump.cpp` + `build_qa_312.ps1`：自己 `mj_loadXML → mj_saveModel → mj_loadModel → 逐字段写原始小端字节` | 只借用同一份 MuJoCo 3.12 package 与 MSVC；代码与交付 `dump_collision_fields_312.cpp` 无复用 |
| 3.15 侧字段 dump | `qa_dump_315.py`：自己加载/保存/重载并写原始字节 | 与交付 `dump_fields_315.py` 无复用 |
| parity 比较 | `qa_parity_compare.py`：自建字段表、自算 sha256、位型比较、octree 结构自检 | 不读交付 `octree_parity.py` |
| 既有参考审计 | `qa_audit_existing.py`：自己解析 d04 run-5 12 个 JSON，逐项重算 count/每 world/geom 对/dist/17 字段/sha，并用 MuJoCo 3.15 读模型核 geom 类型 | 不读交付 `audit_existing_references.py` |
| 10 个捕获校验 | `qa_verify_references.py`：自己从模型 XML 重建 `qpos/qvel` 期望位型、做 FK、逐字段/语义校验 | 不读交付捕获断言，只把交付夹具当被测数据 |
| 两次捕获复跑 | `qa_recapture.py` / `qa_recapture_diff.py`：QA 自己的捕获协议 + 原始位比 + 与交付夹具逐字段差 | 不调用交付 `capture_g0b_references.py` |
| 接触判重 | `qa_duplicate_contacts.py`：按 `worldid/dist/pos/frame/geom` 位型判重 | 独立于交付 |
| 证据/范围 | `qa_evidence_hash.py`：69 项哈希、blob、脚本行数、提交范围、受保护路径、GPU 日志核对 | — |

运行日志：`logs/`（`01`–`09`），命令与退出码逐条记录；报告：`parity_qa_report.json`、`qa_315_report.json`、`audit_qa_report.json`、`references_qa_report.json`、`recapture_qa_report.json`、`recapture_diff_report.json`、`branch_geometry_report.json`、`evidence_hash_report.json`。

## 2. octree parity 独立复算（本片最关键）——位级全等成立

方法：再用 QA 自己的 3.12 C++ dump 与 3.15 Python dump 从**各自版本**加载交付 MJB（3.12 产物只由 3.12 加载、3.15 只由 3.15 加载），逐字段写原始小端字节后做逐字节比较；**不看交付 `octree-parity.json` 的 equal 结论**（最后再拿它和我的重算对账）。

| 模型 | 字段数 | 逐字节全等 | 关键字节数 |
| --- | --- | --- | --- |
| `hfield-box` | 18 | **18/18 全等** | `hfield_data` 36 B（9×f32，`[0,0.5,0,0.5,1,0.5,0,0.5,0]`）、`hfield_size` 32 B（4×f64，`[1,1,0.1,0.1]`）、oct_* 全 0 宽（noct=0） |
| `sdf-cylinder` | 18 | **18/18 全等** | `oct_depth` 253,988 B、`oct_child` 2,031,904 B、`oct_aabb` 3,047,856 B、`oct_coeff` 4,063,808 B、`mesh_octadr/octnum` 各 4 B；noct **63,497 = 63,497** |

- f64 位型层：`oct_aabb` / `oct_coeff` / `hfield_size` / `geom_size` / `geom_margin` **逐字节相同**；f32 位型层：`hfield_data` **逐字节相同**；结构层：`counts` / `nq_nv` / `oct_depth` / `oct_child` / `mesh_octadr/octnum` / `hfield_nrow/ncol/adr` / `opt_sdf` / `geom_type` / `geom_dataid` 全等。
- octree 结构自检（我独立对 3.12 字节做，不依赖 equal 结论）：节点数 63,497；深度分布 `{0:1, 1:8, 2:64, 3:512, 4:3584, 5:18944, 6:40384}`；子索引全部合法或 -1（负子 444,480、被引用节点 63,496、无未被引用的非根节点）；父→子深度单调（0 违规）；半边长 >0；子 AABB 全部包含在父 AABB 内；根中心为原点。
- **对账**：我把交付 `octree-parity.json` 的每个字段条目（dtype/count/bytes/sha256_312/sha256_315/equal）逐项与我的重算比较，**全部命中**（2 模型 × 18 字段）；结论 `all_equal=true` 与我的独立结果一致。
- **因此「SDF/hfield 数值验收走位级」的决策前提成立**（不启用「统一 3.15 oct_* 夹具」退路）；该结论支持 G3/G4 的位级判据，但 G3/G4 本身仍需各自落地验证。

## 3. MJB provenance 与同版本重载——成立

| 产物 | 交付声明（B / sha256 前 8） | QA 独立复算 | 我的 XML 重建 MJB 是否字节相同 |
| --- | --- | --- | --- |
| `mjb-312/hfield-box` | 5,267 / `39e74291` | 字节、sha 一致 | **相同** |
| `mjb-312/sdf-cylinder` | 9,405,930 / `9dfd077e` | 一致 | **相同** |
| `mjb-315/hfield-box` | 5,259 / `cf60c77e` | 一致 | **相同** |
| `mjb-315/sdf-cylinder` | 9,405,922 / `f1df72cf` | 一致 | **相同** |

- 源 XML sha256 与仓库文件一致（`hfield-box.xml` `e0fe921c…`、`sdf-cylinder.xml` `178bc914…`）；同版本重载：3.12 产物用 3.12 加载、3.15 产物用 3.15 加载均成功，且我用 XML 从零编译→保存→重载得到的 MJB 与交付四个 MJB **逐字节相同** → provenance 不是「从一个 MJB 拷另一个」。
- 跨版本：**3.15 加载 3.12 产物失败**（`ValueError: mj_loadModel: failed to load from mjb`，附警告 `Model and executable have different number of sizes in mjModel`）；**3.12 加载 3.15 产物也失败**（`mj_loadModel` 返回 NULL + 同一警告）。交付「实测 different number of sizes、两侧各自生成」的记录**如实**。
- 参考侧未偷用产品 MJB：3.15 侧产物可由 3.15 从源 XML 独立重建（字节相同），捕获脚本走 `MjModel.from_xml_path`，未读取 `fixtures/g10-p5/mjb-312/*`。

## 4. 既有参考审计复核——12/12 逐项一致

用 QA 自己的解析器重算 d04 run-5 `authoring-runs/run-5/generated/cases/collision-*.json`（12 个 case），与交付 `existing-reference-audit.json` 逐项比较：**count / 每 world / geom 对 / dist min·max / 17 字段名 / 文件 sha256 / worlds / path 全部一致，mismatches=0**；交付 case 集合与 d04 碰撞 case 集合完全相同（无多、无漏）。

- 可复用判定有事实依据（geom 类型由 MuJoCo 3.15 读模型解析）：`box-box-multiccd` → box+box（multiccd 启用、margin=0 走离散路径）、`convex-mesh` → box+mesh、`degenerate-mesh` → mesh+mesh、`hfield-box` → hfield+box、`sdf-cylinder` → cylinder+sdf；`sphere-sphere` 仅回归（解析路径非 GJK）。
- 缺口清单 9 项与 10 个新 case 的映射成立；**既有 case 一个都没有重采**（新捕获文件名与既有 case 无交集）；审计只读 d04 路径。→「先审计后补采」在证据层面成立。

## 5. 10 个新捕获复核

### 5.1 覆盖矩阵（QA 从夹具重算）

| case | count | 每 world | geom 类型对 | dist 范围 | 覆盖判定 |
| --- | --- | --- | --- | --- | --- |
| sphere-box-shallow | 2 | 1/1 | box+sphere | [-0.060000, -0.060000] | 浅穿透：球心在盒外、点到盒距离 0.04 > tolerance → GJK shrink+_inflate 分支前提成立 |
| sphere-box-deep | 2 | 1/1 | box+sphere | [-0.140000, -0.120000] | 深穿透：球心在盒内（点-盒距离 0）→ 需重置跑 GJK+EPA 的分支前提成立 |
| capsule-box | 2 | 1/1 | box+capsule | [-0.100000, -0.090000] | capsule shrink/support 覆盖 |
| cylinder-box | 8 | 4/4 | box+cylinder | [-0.060000, -0.060000] | 多接触（4/world，geomcollisionid 0..3） |
| ellipsoid-box | 2 | 1/1 | box+ellipsoid | [-0.010000, -0.010000] | ellipsoid support 覆盖 |
| hfield-sphere | 8 | 4/4 | hfield+sphere | [-0.056554, -0.019377] | hfield CCD + sphere 组合；**4 接触/world 中有重复点**（4 行中 2 行 dist/pos/frame 相同，仅 geomcollisionid 不同），见 §5.5 |
| hfield-capsule | 7 | 4/3 | capsule+hfield | [-0.170000, -0.170000] | hfield 与第二类型配对；mjwarp 警告该 CCD 对不支持 multicontact；**world0 的 4 个接触全部同点、world1 的 3 个接触全部同点**，仅 geomcollisionid 不同，见 §5.5 |
| sdf-box | 80 | 40/40 | box+sdf | [-0.111180, -0.003344] | SDF narrowphase + Halton initpoints，40 接触/world |
| mesh-mesh | 2 | 1/1 | mesh+mesh | [-0.028868, -0.017321] | 非退化 mesh-mesh |
| box-box-margin | 10 | 5/5 | box+box | [-0.078829, -0.032945] | margin=0.01 → `is_discrete=false` 非离散路径，5 接触/world（见 §9 问题 1） |

数值来源：`references_qa_report.json`（逐接触 17 字段/宽度/有限性/几何语义均过；`frame` 正交归一违规 0、`includemargin = margin1+margin2` 违规 0、geom id 越界 0）。

### 5.2 `default_state` offset 语义（全部 10 case 逐位核对）

- 从模型 XML 独立重建期望：`qpos = tile(qpos0,2)`，`world1[0:3] += offset`；`qvel[world] = linspace(-0.02(w+1), 0.03(w+1), nv, f32)`。10/10 与夹具**位型相同**。
- world0/world1 的 qpos 位差异**只出现在 offset 非零分量**（例：sphere-box-shallow offset `[0.005,0,0]` → 仅 index 0；sphere-box-deep offset `[0,0,-0.06]` → 仅 index 2）；每个模型恰 1 个 freejoint，`jnt_qposadr[0]=0`。
- 独立 FK（MuJoCo 3.15 `mj_kinematics`）：world1 相对 world0 **只有 free body 移动**，位移与 offset 一致；其余 body/静态 geom 不动 → 「offset 只作用于第一个 freejoint 前 3 维」成立（Arch 要求的抽样检查扩到全部 10 case）。

### 5.3 两次捕获与夹具复现（warp CPU 后端重采）

- QA 用**自己的捕获协议**（同 `worlds=2`、`nconmax=4096`、`njmax=20000`、`kinematics+collision`）在两个独立进程内各捕获两次：**10/10 case 两次原始位等**（全部 17 字段原始数组逐位相同）。
- 与交付 CUDA 夹具按「worldid 分组 + 排序」比较：**4 个 case 逐位相同**（sphere-box-shallow/deep、capsule-box、hfield-capsule），**6 个 case 数值一致但非逐位**：dist ≤ 6.9e-7、pos ≤ 4.6e-7、frame ≤ 2.8e-5（sdf-box 最大），属 warp CPU/CUDA 后端浮点差；contact 数、world 归属、geom 对、字段宽度全部一致。
- **限制（如实）**：交付的两次捕获是 **CUDA** 跑的，第二次捕获数组没有落盘，`two_captures_*` 布尔无法从夹具文件本身复算；我用 warp CPU 后端验证了「同协议两次捕获原始位等」，但**没有独立复跑交付 CUDA 的两次捕获**（见 §10）。

### 5.4 字段与 provenance
- `contact` 17 个字段记录齐全（float32/int32、shape/sha 自洽、有限）。**注意**：`flex`/`elem`/`vert` 在 10 个模型上都是 **0 宽**（模型 `nflex==0`；对应 mjwarp `io.py` 的 `wp.empty(0, dtype=wp.vec2i)` 分配），不是 2 宽（见 §9 建议 2）。
- provenance：每 case 的 `task_id`、`upstream_revision=71da24d9…`、捕获脚本 sha、models index sha、源 XML sha、`authoring_only=true`、`reads_product_tree=false`、`product_python_dependency=false`、版本三元组全部与 receipt/仓库文件一致（10/10）；`references/index.json` 的每 case sha/count/per_world/flag 与 case 文件一致。

### 5.5 hfield 两个 case 含重复接触点（重要数据性质）

用 QA 自己的判重（按 `worldid/dist/pos/frame/geom` 逐位分组）：

| case | 接触数 | 不同 (dist,pos,frame) 组 | 重复行（额外副本） | 说明 |
| --- | --- | --- | --- | --- |
| hfield-sphere | 8 | 6 | 2 | 每 world 4 行中 2 行完全同点（`dist/pos/frame/geom` 相同），仅 `geomcollisionid` 不同 |
| hfield-capsule | 7 | 2 | 5 | world0 4 行全部同点（`pos=(0.04,0,0.185)`、dist=-0.17），world1 3 行全部同点（`pos=(0.06,0,0.185)`），仅 `geomcollisionid` 不同 |

- 这是**上游 mjwarp 输出本身的性质**（我用 warp CPU 后端重采 hfield-capsule 逐位复现、hfield-sphere 数值 ≤1.2e-7），不是交付伪造；d04 既有 `hfield-box` 8 行位置互不相同，没有该现象。
- 对 G4 的影响：这两个 case 的「4/4、4/3」**不代表 4 个不同接触点**；按 `(dist,pos)` 排序时重复行无法定序，若按全字段位比需要把 `geomcollisionid` 也纳入排序键，否则重复行的相对顺序/数量差异可能比对不出来（见 §9 建议 2、6）。G4 落地时建议用「world 分组 + 全字段多重集比较（含 geomcollisionid）」作为判据。

## 6. 顺序口径执行核对

- 实现已按声明执行：`worldid` 分组 + 排序键逐位（`worldid, dist bits, pos bits, frame bits, geom, type, dim`），比较的是排序后的键序列；`raw_identical` 走**全字段**比较（`first.contact == second.contact`）。
- 原始顺序观察项**全部强制记录且可复核**：10/10 `order_observation.worldid_sequence` 与 `output.worldid` 一致；`grouped_by_world_in_raw_order` 与我的重算一致。具体：`sdf-box` 交错（0,1,0,1,…，raw=false）；`cylinder-box`/`hfield-sphere`/`hfield-capsule`/`box-box-margin` raw=true 但交错；其余 5 个按 world 分组。
- **跨后端不稳定**：我用 warp CPU 后端重采时，`cylinder-box`/`hfield-sphere`/`box-box-margin` 的原始顺序是**按 world 分组**（例 `[0,0,0,0,1,1,1,1]`），与交付 CUDA 的交错不同；`sdf-box` 在 CPU 上两次 raw 也位等（原子写在 CPU 上串行化）而交付 CUDA raw=false。→ **原始顺序是后端/写序属性、不是夹具稳定属性**；G3/G5 必须用「分组+排序」判据（交付结论正确），但不能把 raw 顺序当固定判据（见 §9 建议 3）。
- mjwarp 原子写序解释有源码依据：`collision_core.py:271 cid = wp.atomic_add(nacon_out, 0, 1)`。

## 7. nworld=2 spike 复核——2A 依据成立（读源码核实）

| 交付结论 | QA 核实 |
| --- | --- |
| broadphase 按 `world in 0..state.nworld` 外层循环 | 属实：`nxn.rs:16`、`sap.rs:82`；`sap.rs:40` 用 `world*ngeom` |
| 按 `world*ngeom` 索引 | 属实：`filter.rs:223-224`、`narrowphase.rs:312-314`；`io`/`collision.rs` 校验 `geom_xpos` 长度 = `nworld*ngeom` |
| narrowphase 校验 `worldid < nworld` | 属实：`narrowphase.rs:299-303`（越界返回 `InvalidIndex{field:"worldid"}`） |
| 接触写 worldid | 属实：`narrowphase.rs:272`（`worldid: worldid as i32`）等 |
| 缺口 1：`hfield_data`/`oct_*` 无 world 维上传 | 属实：产品 `src/` 中目前**不存在**这些字段（全树 grep 0 命中）；参数批 world 维机制已有（`BroadphaseModel` 的 `margin/gap/rbound/aabb` 按 `world % batches` 取） |
| 缺口 2：接触写入顺序 / `naconmax` | 与捕获观察一致（raw 顺序后端相关；容量截断在 `CandidateWriter`） |

**结论**：2A 无需降级到 2B 的代码依据成立；spike 仍是「只读代码」结论，G1/G5 落地时需实测上传链路与接触顺序对齐（交付 unmet 已列）。

## 8. 证据/边界/诚实性

- **范围**：`git diff 87bc042..f0e43bb` 共 43 文件，全部落在 `scripts/authoring/g10-p5/`、`fixtures/g10-p5/`、`docs/development-evidence/g10-p5-fixtures/`、`.normify/` 内；`src/`、`tests/`、`Cargo.*`、`docs/evidence/d04-complex-physics/`、`docs/development-evidence/render-bvh/`、`fixtures/mjwarp-complex-physics/` **零改动**；工作区这些受保护路径干净。
- **哈希**：QA 69 项检查全过（0 失败）：G0b 证据三文件与 git blob 逐字节相同；receipt 声明的 4 个 MJB bytes/sha、2 个源 XML sha、`octree-parity.json` sha、`existing-reference-audit.json` sha、references/models index bytes+sha、10 个 case sha、7 个交付脚本 lines/bytes/sha 全部一致；README 的 MJB 前缀与 noct 数字一致。
- **GPU**：未申请/未占用 GPU；交付 GPU 窗口（`gpu-run.log`）起止 **4,156 → 4,156 MiB**、前后 `--query-compute-apps` 进程列表**逐行相同**（30 行）、无本任务计算进程。
- **receipt**：`task_id` 正确，7 条 unmet 如实（含 Linux/T4、性能、组合矩阵、hfield 四接触语义、nworld 上传链路、SDF 顺序、本片范围）。
- **诚实性**：本复核不声称 G10 整体实现、物理整体通过、双平台、与上游等价（`not_claimed` 见 receipt）；未跑全量 G01（本片无构建变更）；S0b fast-math 跨配方等**历史数字**未被我用作通过依据。

## 9. 问题清单（按严重级别，均非阻断）

**一般**

1. **`box-box-margin` 的 multiccd 覆盖描述与模型配置相反**。README §5 与 receipt `captures.cases.box-box-margin.covers` 写「margin≠0 非 `is_discrete` 路径 + multiccd」，但模型 `fixtures/g10-p5/models/box-box-margin.xml` 显式 `<flag nativeccd="disable" multiccd="disable"/>`（我读模型：`disableflags=0xA0000`，multiccd/nativeccd 均禁用；`geom_margin=[0.01,0.01]`）。该 case 的 5 接触/world 来自 **margin≠0 非离散路径**（mjwarp `collision_gjk.py:2526`：`is_discrete = _discrete_geoms && margins==0`），不是 multiccd；multiccd 的真实覆盖来自既有 `box-box-multiccd`（该模型 multiccd=enable）。§5 顺序观察项把 `box-box-margin` 的交错归因于「broadphase/multiccd」也因此不准确。**期望**：删去该 case 的 multiccd 表述、改为「非 is_discrete 多接触路径」，multiccd 覆盖只挂 `box-box-multiccd`。**实际**：交付措辞与模型配置矛盾（不影响覆盖结论，因为 multiccd 已有既有参考）。建议 `normify fix`。

**建议**

2. **排序后位等判据只覆盖 7 个字段，且排序键不含 `geomcollisionid`**：`capture_g0b_references.py` 的 `row_key` 只含 `worldid/dist/pos/frame/geom/type/dim`，`two_captures_sorted_bitwise_identical` 比较的是排序键而不是全 contact 行；只有 `two_captures_raw_bitwise_identical` 是全字段比较。对 `sdf-box`（raw=false）而言，其余 10 个字段（includemargin/friction/solref/solreffriction/solimp/adhesion/flex/elem/vert/geomcollisionid）的两次捕获位等**没有证据**。另外 `hfield-sphere`/`hfield-capsule` 存在 `(dist,pos,frame)` 完全相同的多行（§5.5），这些行在排序键上相同、无法定序，全字段多重集比较必须把 `geomcollisionid` 纳入排序键。风险低（多数字段是模型参数常量、flex 三字段 0 宽），但判据应写全：排序后按全字段比较（含 geomcollisionid），或注明覆盖字段集。

3. **「contact 17 字段」需注明 3 个 flex 字段为 0 宽**：10 个夹具的 `flex`/`elem`/`vert` 记录 shape 为 `[0,2]`（模型 `nflex==0`，mjwarp `io.py` 分配 0 长度），不是 2 宽。记录存在，但下游 G2/G3/G4 比较不能要求这 3 字段有值，也不能把它们当 flex 路径（G11）判据。

4. **原始顺序观察项的适用边界**：交付把 `raw_identical=true` 记为夹具/捕获属性，但顺序实际取决于后端写序——我 warp CPU 重采下 `cylinder-box`/`hfield-sphere`/`box-box-margin` 为 world 分组而 CUDA 交付为交错，`sdf-box` CPU 两次 raw 位等而 CUDA raw=false。建议在证据里注明「raw 顺序只对本次 CUDA 运行成立，跨后端/跨设备不保证」，避免后续误当固定判据。

5. **捕获日志可读性**：`gpu-run.log` 为 UTF-16，且把 mjwarp/SDK 的 stderr 警告包成 PowerShell `NativeCommandError`（含 `MULTICCD ... without multicontact support` 警告），与 Python 包装层报错混排；不影响捕获结果，但复现日志易误读。建议捕获脚本统一 stderr 采集/编码。

6. **hfield 两个 case 的「4 接触/world」是重复接触点**：`hfield-capsule` 7 行只有 2 个不同 `(dist,pos)`（world0 4 行同点、world1 3 行同点），`hfield-sphere` 8 行有 6 个不同（重复 2 行）；重复行仅 `geomcollisionid` 不同。上游行为已由 QA CPU 重采复现，但 README/receipt 的覆盖表写成「4/4、4/3」容易让 G4 误认为 4 个不同接触点。建议证据注明该性质，并让 G4 判据按「world 分组 + 全字段多重集（含 geomcollisionid）」比较。建议 `normify fix`（README §5 表 + receipt 说明）。

## 10. 限制与未做

- **未独立复跑交付 CUDA 的两次捕获**：交付第二次捕获数组未落盘，只有布尔；我用 warp **CPU** 后端验证同协议两次原始位等并复现夹具数值（4 例逐位、6 例 ≤6.9e-7/2.8e-5），但 CUDA 侧原始位等/原始顺序仍依赖交付布尔 + 我的跨后端对照。若 Arch 给约 2 分钟 GPU 窗口，我可用已写好的 QA harness 补跑 CUDA 两捕获并逐位对夹具（未在本轮执行，遵守「需要 GPU 先报」）。
- **CPU/CUDA 数值差**：warp CPU 后端与 CUDA 后端在这些算例上有 ≤6.9e-7（dist/pos）、≤2.8e-5（frame）差异，属后端浮点差，不是夹具缺陷；不能据此判「夹具不位等」。
- 未跑全量 G01、未跑产品测试/产品代码、未改夹具与 G0b 原证据；只审 10 个补充 case 与 5 个既有 case 的覆盖，不是全组合矩阵；未覆盖 Linux/T4（G0b unmet 已列）；未验证 G1–G5 的产品侧落地（后续切片）。
- 捕获输入仅为静态 `qpos/qvel`（`kinematics+collision`，无积分/步进），不覆盖时序/约束求解语义。

## 11. 复现

```powershell
# CPU 侧全套（QA 自己写；不占 GPU、不跑产品代码）
& target\qa-g0b\build_qa_312.ps1
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g0b\qa_dump_315.py
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g0b\qa_parity_compare.py
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g0b\qa_audit_existing.py
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g0b\qa_verify_references.py
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g0b\qa_branch_geometry.py
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g0b\qa_duplicate_contacts.py
# 两次捕获重采（warp cpu 后端）：
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g0b\qa_recapture.py
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g0b\qa_recapture_diff.py
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g0b\qa_evidence_hash.py
# 或一键：& target\qa-g0b\qa_run_all.ps1（日志写 target\qa-g0b\logs）
```
