# 渲染/BVH S2 收口：plan 六条验收逐条映射与总回执

- 任务：`2026-10-24-render-bvh-s2-close`（depends_on S2a + S2b，均已 verified）
- 日期：2026-10-24
- 结论：**pass（收口口径）**。plan S2 六条验收逐条映射到 S2a/S2b 证据；QA 对两片分别做独立复核，
  S2a `eb80566`、S2b `5ef8cb6`（对象 `c7b5d7f`）均 verified、无阻断。范围限定见 §5：
  mesh 三角形求交留 S3，本收口不宣称完整渲染/S3/全量 G01。

## 1. 结构来源

原任务 `2026-10-24-render-bvh-s2` 因「运行中父任务写路径与派生任务冲突（normify 0.7.0
schedule.rs，冻结字段不可改）」按 Chief 裁决 A 方案拆分：

- `8d4357f` 治理(排程)：abandon 原 S2；
- `2a62e2b` 工程(治理)：登记 S2a / S2b / s2-close；
- `75166cf`（原 S2 下、abandon 前）证据：S2 场景 BVH 参考捕获（AABB/refit/移动后射线命中），
  保留在 git 历史与 fixture provenance 中，是 S2a/S2b 的参考来源之一；
- `f9adf14` 功能(渲染)：S2a（主机侧 AABB 位等/构建/refit/分组/拒绝面）；
- `c7b5d7f` 功能(渲染)：S2b（CUDA 场景 BVH 遍历/逐射线语义命中）；
- 本收口任务只写 `docs/development-evidence/render-bvh/s2-scene-bvh/`，不改源码/测试/fixtures/
  authoring 脚本（`unchanged` 检查覆盖）。
- `src/render/bvh/mod.rs` 在 S2b 增加 `pub mod cuda` 挂载与条件 re-export（+4 行），属 S2b
  提交 `c7b5d7f`；S2a receipt 中该文件哈希是 `f9adf14` 时点的历史快照。S2a 的 unchanged 范围
  不含它，S2b 复核已覆盖。

## 2. plan S2 六条验收逐条映射

| # | plan S2 验收 | 实现与验证位置 | 结论 |
| --- | --- | --- | --- |
| ① | 每条固定射线 geomid 精确、distance ≤4 ULP、normal ≤1e-4 rad | `docs/development-evidence/render-bvh/s2b-scene-bvh-gpu/`（README §4、receipt.results）；`tests/render_scene_bvh_gpu.rs` 的 `ray_hits_match_reference_for_initial_and_refit` | **primitive 范围 pass**：initial 383/384 条、moved 382/384 条逐条对照；geomid 全等、distance 全部 **0 ULP**、normal 最大 **4.6e-8 rad**；mesh 命中 ray（initial 91；moved 91/92）按 plan 延后 S3，不含在通过口径 |
| ② | 构建前 AABB 逐元素位等于参考（lower/upper f32 位型） | `docs/development-evidence/render-bvh/s2a-scene-bvh-host/`（README §3、aabb-parity.json）；`tests/render_scene_bvh_host.rs` | **pass**：7 类 leaf × 8 位姿 = **56/56 位型相等，max ULP 0**；每类 8/8；模型侧 geom_size/mesh bounds 的 3.12→3.15 f32 位型也逐位相等 |
| ③ | 移动 body 后 refit，同一射线集变化与参考一致 | S2b README §4；AABB 层见 S2a README §5-3 | **primitive 范围 pass**：AABB 层 leaf 位等且变化 leaf 集合 `[1..6]`；射线层 refit 前后 geomid 变化集合与参考一致 `[99,109,116,117,155]`，distance 逐条 0 ULP；参考全部 6 条变化中 ray 92 变为 mesh 命中（S3 延后） |
| ④ | per-world group root；world groups 不跨命中 | S2b README §4、receipt.world_groups；`world_groups_do_not_cross_hit` | **pass**：双 world 复制同场景（world 1 geom id +100），各 384 条射线查询 **0 跨 world**；world 0 与单 world 结果逐条一致，world 1 命中 id = world 0 +100 且 distance 位型相同 |
| ⑤ | 空场景与越界 group 被拒绝 | 统一清单见 §2.1（R01–R19，单一来源）；执行位置 `src/render/bvh/{builder,aabb,refit,query}.rs` 与 `src/render/bvh/cuda/mod.rs`；测试 `tests/render_scene_bvh_host.rs`、`tests/render_scene_bvh_gpu.rs` 及模块单测 | **pass**：19 条均在分配/构建/发射前拒绝并返回明确错误 |
| ⑥ | 不对 BVH 节点布局做断言 | S2a README §4（只断言语义不变量：可达性/子节点并集/root 并集）；S2b README §4（只用语义命中与分组结果） | **pass**：两片测试均未断言节点编号/顺序/内部布局 |

## 2.1 拒绝面统一清单（R01–R19，定稿口径）

QA 的 S2a 复核（`eb80566`，`docs/development-evidence/render-bvh/s2a-independent-check/`）指出
拒绝面计数口径不一致（任务文字 13 / S2a README 10 类 / 交付 host 测试 11 断言 / QA 14 个具体
调用）。本收口固定下面这份**具名清单**作为唯一引用口径；S2a/S2b 已验收证据不改写，按下表映射到
R 编号。

| 编号 | 条件 | 层 | 错误 | 执行位置 | 验证位置 |
| --- | --- | --- | --- | --- | --- |
| R01 | 空场景（没有 leaf） | 构建 | `EmptyScene` | `build_scene_bvh` | host 测试、builder 单测、QA S2b 5/5 |
| R02 | `groups` 长度 ≠ `bounds` | 构建 | `LengthMismatch{"groups"}` | `build_scene_bvh` | host 测试、builder 单测 |
| R03 | `geom_ids` 长度 ≠ `bounds` | 构建 | `LengthMismatch{"geom_ids"}` | `build_scene_bvh` | builder 单测 |
| R04 | `worlds == 0` | 构建 | `InvalidWorldCount` | `build_scene_bvh` | host 测试、builder 单测 |
| R05 | leaf 的 group ≥ worlds | 构建 | `GroupOutOfRange` | `build_scene_bvh` | host 测试、builder 单测 |
| R06 | 某 world 分组没有 leaf | 构建 | `EmptyWorld` | `build_scene_bvh` | host 测试、builder 单测 |
| R07 | 构建输入 AABB 非有限或下界>上界 | 构建 | `InvalidAabb` | `build_scene_bvh` | host 测试、builder 单测 |
| R08 | refit 的 bounds 长度 ≠ leaf 数 | refit | `LengthMismatch{"bounds"}` | `refit_bvh` | host 测试、refit 单测 |
| R09 | refit 的 AABB 非法 | refit | `InvalidAabb` | `refit_bvh` | host 测试、refit 单测 |
| R10 | 不支持的 geom 类型（hfield=1；未知类型） | leaf AABB | `UnsupportedGeomType` | `LeafGeometry::from_geom_type` | host 测试 |
| R11 | leaf 几何非有限（pos/size/xmat/mesh 顶点） | leaf AABB | `NonFinite` | `compute_leaf_aabb` / `mesh_half_extents` | host 测试 |
| R12 | mesh 顶点为空（缺 mesh bounds） | leaf AABB | `MissingMeshBounds` | `mesh_half_extents` | host 测试 |
| R13 | `Aabb::new` 输入非有限或下界>上界（公共构造入口） | 基础类型 | `InvalidAabb{usize::MAX}` | `Aabb::new` | QA harness（等价入口） |
| R14 | 宿主语义查询 world 越界 | 宿主查询 | `WorldOutOfRange` | `SceneBvh::ray_leaf_candidates` | query 单测 |
| R15 | leaf 数为 0 或超过节点容量（`2n-1` 溢出） | 构建容量 | `LeafCountOverflow` | `node_capacity` | builder 单测（不实际分配） |
| R16 | GPU 查询空射线集 | GPU 查询 | `InvalidDimension{"rays"}` | `SceneBvhQuery::query` | S2b GPU 测试、QA S2b 5/5 |
| R17 | GPU 查询 world 越界 | GPU 查询 | `InvalidIndex{"world"}` | `SceneBvhQuery::query` | S2b GPU 测试、QA S2b 5/5 |
| R18 | GPU 查询 leaf 几何长度不符 | GPU 查询 | `LengthMismatch{"leaf_geometry"}` | `SceneBvhQuery::query` | S2b GPU 测试、QA S2b 5/5 |
| R19 | GPU 查询 hfield 类型 | GPU 查询 | `InvalidDimension{"geom_type_hfield"}` | `SceneBvhQuery::query` | S2b GPU 测试、QA S2b 5/5 |

另有**内部一致性防御**（打包前拒绝，不计入 R01–R19 对外计数）：BVH 深度超过 128 层栈
（`bvh_depth`）、node child/leaf 索引越界（`bvh_child`/`bvh_leaf`）、`leaf_geom_id` 缺失、
打包计数溢出、会话不一致。

### 旧口径 → 统一口径映射

| 旧来源 | 旧计数 | 对应 R 编号 |
| --- | --- | --- |
| S2a 任务验收文字（空场景 / 越界 group / 非法计数） | 3 组 | 覆盖 R01–R19（文字为分类，非具体条数） |
| `s2a-scene-bvh-host/receipt.json` rejection_surface.cases | 13 条 | R01–R12 + R15 |
| `s2a-scene-bvh-host/README.md` §5-5 散文清单 | 10 类 | R01、R04、R02+R03、R05、R06、R07、R08、R10、R11、R12 |
| 交付 `tests/render_scene_bvh_host.rs` 断言 | 11 个 | R01、R02、R04、R05、R06、R07、R08、R09、R10、R11、R12 |
| QA 复核 `s2a-independent-check` 具体调用 | 14 个 | R01–R14（与 QA 列表逐名一致） |
| QA 复核 `s2b-independent-check` 具体调用 | 5 个 | R01（空场景构建）+ R16–R19 |
| `tests/render_scene_bvh_gpu.rs` 断言 | 4 个 | R16–R19（另复用 R01 的构建拒绝） |

## 3. 参考来源与独立证据链

- S2 参考捕获（`75166cf`，原 S2 下）：MJWarp 3.15 冻结上游 `71da24d…`，pinned 环境；
  `fixtures/render-bvh/scene-a/s2/bvh_initial.npz`、`bvh_moved.npz`、`context.json`、
  `provenance.json`；初始命中与 S0b 已验收参考逐位一致；GPU 独占、跑完释放。
- S2a 参考捕获：`fixtures/render-bvh/scene-a/s2a/bounds.json`（设备侧 f32 位姿位型 + 参考 AABB
  位型，8 位姿 × 7 类），两次捕获逐位一致，与 S2 npz 交叉校验 4/4 位型相等。
- S2b 参考捕获：`fixtures/render-bvh/scene-a/s2/reference_hits.json`（**设备侧 f32 位姿位型** +
  参考命中位型），两次捕获逐位一致，与 S0b/S2 npz 命中 7/7 位型交叉校验（QA 独立重算同结论）；
  重要修正：CPU 位姿会让 cylinder 命中偏 124 ULP（超 4 ULP 预算），必须用设备侧位姿。
- 产品测试均不启动 Python；authoring 脚本仅在离线使用。

## 4. 门禁、测试与 QA 复核汇总

| 项 | 数字 |
| --- | --- |
| S2a 门禁 `f9adf14` | fmt 1260ms / clippy 354ms / clippy-cuda 8099ms / unit 16060ms（506 passed / 0 failed） |
| S2a 定向测试 | `cargo test --test render_scene_bvh_host` 5/5；`--lib render::bvh` 16/16 |
| S2b 门禁 `c7b5d7f` | fmt 1260ms / clippy 354ms / clippy-cuda 5033ms / unit 16856ms（506 passed / 0 failed） |
| S2b 定向测试 | `cargo test --features cuda-probe --test render_scene_bvh_gpu -- --ignored --test-threads=1` 3/3 |
| S2a QA 复核 | `eb80566` verified；发现 1 个非阻断观察项（拒绝面计数口径，本收口已统一） |
| S2b QA 复核 | `5ef8cb6` verified（对象 `c7b5d7f`，基线 `f43b576`）；9/9 项成立、无阻断 |

S2b GPU 环境：RTX 4070 Ti SUPER / 驱动 596.36；本任务捕获基线 3434 MiB、进程内 3692 MiB；
产品测试跑前 3439 MiB、跑后 3475 MiB 回桌面基线；窗口已释放。device 侧为真 fma.rn.f32
（NVRTC fmad=true、fast_math=false、compute_70）；S2a 的主机 f64 fma 复刻只用于 host AABB，
不构成 device fma 证明。

## 5. QA 的 S2b 独立复核（`5ef8cb6`）

QA 在 45 分钟独占 GPU 窗口内独立复核，9 项结论（详见
`docs/development-evidence/render-bvh/s2b-independent-check/`）：

1. 逐射线：initial 383 条、moved 382 条比较，geomid mismatch 0；
2. distance max ULP **0**；
3. refit 变化集合 `[99,109,116,117,155]` 与参考一致，refit vs fresh 差异 0；
4. 双 world：world0/world1 各 384 条查询错误 0、0 跨命中；
5. 拒绝面 5/5（空射线、world 越界、几何长度不符、hfield、空场景构建）全部拒绝；
6. PTX 精度：`fma.rn.f32` 166 条、`f64` 指令 **0** 条（另 `mul.rn.f32` 0、`sqrt.rn.f32` 13、
   `div.rn.f32` 15），device 路径为纯 f32 + 真 fma；
7. 设备位姿 vs CPU 位姿：xpos 8/21 分量不同（max 31 ULP）、xmat 39/63 分量不同（max 11 ULP），
   7/7 参考位型交叉校验成立（QA 独立重算，不读交付数字）；
8. 全量 CPU 测试 506 passed / 0 failed，交付 GPU 测试 3 passed / 0 failed；
9. 8/8 产物 sha256+bytes+lines 与 git blob 一致；`f9adf14..c7b5d7f` 无 runtime 文件变化；
   新文件均 <500 行；产品测试不启动 Python。

QA 观察项（**非阻断**）：参考捕获用 `use_fast_math=True`，产品内核用 `use_fast_math=false`；
本夹具仍为 0 ULP、无分歧，但该编译旗标差异需在后续切片（S3+）继续关注，不能据此宣称两条
编译路径普遍等价。

## 6. 未满足项与边界（汇总）

- mesh 三角形求交与 mesh 命中 distance/normal 留 S3：initial ray 91、moved ray 91/92 不在 ①③
  的通过口径内；本收口不宣称 S3 已实现。
- hfield 未实现（显式拒绝）；SDF 按 MESH 分支返回 miss；旋转 plane 无独立 GPU 参考。
- 单 fixture（scene-a × 384 固定射线），无 fuzz/随机射线集。
- 仅实现冻结捕获的默认剔除语义（bodyexclude=-1、全部可见、flg_static=true、geomgroup 全 -1）。
- 固定 128 层遍历栈；深树路径未由本夹具实测（超限在打包前拒绝）。
- 参考为 3.15 冻结产物；不宣称 3.12/3.15 渲染等价。
- 本轮未跑全量 G01（fast-loop 口径）。
- S2a 的 host f64 fma 复刻限制（任意输入可能 ≤1 ULP 差异）已在 S2a receipt 登记；device 路径
  不用该复刻。

## 7. 收口引用

- 两片交付证据：`s2a-scene-bvh-host/`、`s2b-scene-bvh-gpu/`（各自 README + receipt.json）。
- 两片 QA 复核证据：`s2a-independent-check/`、`s2b-independent-check/`（README + receipt.json +
  原始 harness/PTX/测试日志）。
- 本收口只新增本文档与 `receipt.json`；S2a/S2b 已验收证据未改写。
