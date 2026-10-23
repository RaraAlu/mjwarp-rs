# 批次 4 独立复核：G09@P5 交付与 G10/M10-P10 顺延登记

日期：2026-10-23。执行：QA。任务：`2026-10-23-batch4-independent-check`。
复核对象（HEAD `a3214de` 时）：`g09-p5` 交付 `c757e3d`（登记 `18b3fb5`，索引 `8566d42`）；
顺延登记 `g10-p5` `42df446`、`m10-p10` `1ec0257`。

## 1. 结论（先给）

`g09-p5` 的数值/接触声明**全部成立**，边界如实（未引入 BVH/网格/地形资产）；两条顺延登记如实。
发现 **2 条一般**（回执模型 sha256 登记错误、一条验收条款未实现且未登记边界）与 **4 条建议**，均不阻断。

## 2. g09-p5 声明逐项核对

| 声明 | 复核方式 | 结论 |
| --- | --- | --- |
| 7 世界 15 接触 | fixture `nworld=7`、`contacts.count=15`；测试 `assert_eq!(contacts.len(), 15)` | 成立 |
| 逐世界接触数 4/2/1/0/4/4/0 与上游/原生一致 | 测试断言 `world_counts == [4,2,1,0,4,4,0]` 且逐世界产品数 == 原生 `ncon`；fixture `native_ncon` 同值；上游接触世界分布 0:4、1:2、2:1、4:4、5:4（世界 3/6 为 0）合计 15 | 成立 |
| 产品 vs 冻结上游 max abs 5.960e-08（≤1e-6） | 独立 `--nocapture` 实测 `upstream_max=5.960e-8`；对照覆盖 8 类浮点 + `dim/type` 整数逐位，按 `(worldid,geom)` 分组且组内顺序一致 | 成立 |
| 原生 f64 `mj_collision` 1.192e-07（≤1e-5） | 独立实测 `native_max=1.192e-7`；逐世界接触数一致 | 成立 |
| 多接触/退化场景 | flat4 / edge2 / corner1 / gap0 / yaw45_flat4 / thin_flat4 / thin_gap0 计数与 fixture authoring 一致 | 成立 |
| 未验收边界（box-box/capsule/cylinder/ellipsoid、GJK/EPA、mesh/convex、CCD、hfield/SDF、flex、EFC、GPU） | 证据 README §6 与回执 `boundaries` 一致；代码 `_ => UnsupportedPair` 对未支持对拒绝 | 成立（边界措辞见 §7-5/§7-6） |

上游实现交叉核对（`target/mjwarp-parallel/upstream`，HEAD `71da24d9…`）：
`collision_primitive_core.py:343-390 plane_box` 的 `center_dist`、`i&1/2/4` 位序取 ±半尺寸、
`dot(normal, rot*corner)`、`pos = corner + box_pos - 0.5*normal*cdist` 与
`src/physics/collision/narrowphase/primitive.rs:101-143` 逐行对应；
`collision_primitive.py:761-834 plane_box_wrapper` 对 8 个候选逐个 `write_contact`，与 Rust 分支一致。

## 3. 独立复跑（CPU，默认特性）

| 命令 | 结果 |
| --- | --- |
| `cargo test --locked --lib plane_box -- --nocapture` | **1 过 0 败**；实测 `contacts=15 upstream_max=5.960e-8 native_max=1.192e-7 counts=[4,2,1,0,4,4,0]` |
| `cargo test --locked`（全量） | **478 过 0 败**（lib 358） |
| `cargo fmt --check` / `cargo clippy --all-targets --locked -- -D warnings` | 通过 |

门禁记录：`.normify/runs/2026-10-11-roadmap-g09-p5/unit.json` = 478 过 0 败，与交付提交说明一致。

## 4. fixtures / sha256 / 回执抽验

- `fixtures/plane-box/model.xml`：**实际 sha256 `a78b1547…`**（与 `authoring.json`、`cases.json` 一致）。
  回执 `results.references.model_sha256` 登记为 `63743a8a…`，该值实为 **git blob hash**（`git hash-object` 实测），
  即回执哈希字段写错算法 → 见 §7-1（一般）。
- 回执 `files[]` 共 14 项，**14/14 blob 与 HEAD 一致**（含共享 `physics/core.md`，本批未见漂移）。
- `fixtures/plane-box/cases.json` 容差 `upstream_abs=1e-6`、`native_abs=1e-5`，与声明一致；未放宽。
- 交付提交 `c757e3d` 未改 `Cargo.toml/Cargo.lock`；新增资产仅 `fixtures/plane-box/`；
  `src` 内无 `bvh/hfield/sdf/mesh` 代码符号（仅未验收边界的文档注释）→ 未引入 BVH/网格/地形资产。

## 5. 两条顺延登记核对

| 任务 | 状态 | 交付物 | note | 索引 |
| --- | --- | --- | --- | --- |
| `g10-p5`（`42df446`） | proposed、checks 0 | 无 runs/证据目录/回执（实测均不存在） | 裁决 A 顺延；阻塞=高度场/SDF 资产管线与碰撞实现缺失（src 无 hfield/SDF/mesh；上游 hfield→CONVEX/GJK、SDF→`collision_sdf`）；恢复条件=地形资产管线具备后重评或与 r01 合并；不宣称交付 | 批次 4 索引 §2 一致 |
| `m10-p10`（`1ec0257`） | proposed、checks 0 | 无 runs/证据目录/回执（实测均不存在） | 裁决 A 顺延；阻塞=P10 阶段语义依赖 forward/step 调度或图集成、上游无可冻结 callback-graph 语义；P7 契约 `b9cd236` 已交付；恢复条件=图重放能力具备后重评或与 m10-p7 合并；不宣称交付 | 批次 4 索引 §2 一致 |

两条提交均只改 `.normify/changes/<task>.json` 的 note（各 1 行），未改源码；未发现 start 或交付声明。

## 6. 证据目录核对

- `backlog-batch4/`：索引 1 条交付 + 2 条顺延，实际目录仅 `README.md` + `2026-10-11-roadmap-g09-p5/`（README+authoring），一致。
- `backlog-batch2/` 7 个目录、`backlog-batch3/` 4 个目录，均未混入批次 4 文件；
  `git log -- docs/development-evidence/backlog-batch3/2026-10-11-roadmap-g09-p5` 为空——g09 证据从未提交到 batch3，
  提交前已放到 batch4（Backend 提前处理，未产生迁移修正）。
- g09 回执 `files[]` 的 evidence 路径全部为 `backlog-batch4/…`，无 batch2/batch3 残留。

## 7. 差异清单（按严重级别）

### 7.1 一般：g09 回执的 `model_sha256` 登记为 git blob hash

- 复现：`docs/development-evidence/changes/2026-10-11-roadmap-g09-p5.json`
  `results.references.model_sha256 = 63743a8a72d2cd965ef46e2205aedec809789adf`，
  与 `files[]` 中 `fixtures/plane-box/model.xml` 的 blob 完全相同；实际文件 sha256 =
  `a78b15472ffe8d0e3a8f6e61b8c4512dfc94c889ceca8ea344b76dab3cdf1586`（authoring.json、cases.json 均为此值）。
- 期望：回执 `model_sha256` 应为文件内容 sha256（`a78b1547…`），与 authoring/cases 一致。
- 实际：登记为 git blob hash，属**回执登记值错误**；证据本身的 sha256 正确，数值结论不受影响。
- 影响：追溯一致性（跨批次回执抽验会失败）。建议用 `normify fix` 追加修正记录并刷新回执字段。

### 7.2 一般：acceptance 条款「退化输入（尺寸非正/非有限）显式拒绝」未实现且未登记边界

- 复现：`g09-p5` acceptance 明确要求「退化输入（尺寸非正/非有限）显式拒绝」；
  实际 `src/physics/collision/narrowphase.rs` 的 `validate` 只校验各数组长度，
  `plane_box`/`primitive_narrowphase` 对 `geom_size` 无符号/有限性校验（`primitive.rs` 无任何拒绝路径），
  `tests/plane_box.rs` 也无拒绝用例；证据 README §6 与回执 `boundaries` 未把该项登记为未验收。
- 期望：要么实现尺寸校验并加拒绝测试，要么在证据/边界中如实登记「未实现」。
- 实际：条款落空且未登记（不是数值声明问题，但验收清单与交付不一致）。
- 影响：验收口径缺口；建议补实现/测试或补边界登记（由 Chief 决定归入下一批或修正证据）。

### 7.3 建议：`primitive_narrowphase` 文档注释仍写「两类基础几何对」

- `src/physics/collision/narrowphase.rs:322` 注释为「上游 `primitive_narrowphase` 的两类基础几何对
  （plane-sphere / sphere-sphere）」，加入 plane-box 后已是三类。建议更新注释（模块头已更新，仅此函数注释遗漏）。

### 7.4 建议：证据 README 的模型 sha256 只写「见 fixture 登记」

- g09 证据 README §3 未给出模型 sha256 具体值；本次回执恰好在该字段写错，建议证据直接写 `a78b1547…` 便于核验。

### 7.5 建议：「按唯一顶点索引去重后最多 4 个接触」措辞与实现/上游不符

- acceptance 文本称「按唯一顶点索引去重后最多 4 个接触」；上游 `plane_box_wrapper` 对 8 个候选逐个
  `write_contact`，接触数由 margin/gap 过滤决定，**没有去重步骤**（≤4 是几何结论，非去重结果）。
  建议改为「margin/gap 过滤后通常 ≤4」。

### 7.6 建议：未直接覆盖 box-box 候选拒绝

- 边界声明「非 plane-box 的候选对仍返回 UnsupportedPair」；fixture candidates 只含 14 个 plane-box 对
  （box-box 被排除），拒绝面仅由 G08 旧测试（capsule 类型 3）覆盖。建议补一条 box-box 候选拒绝用例。

## 8. checks / acceptance / 模块归属

- `g09-p5` checks：`gate fmt/clippy/unit` + `unchanged` + `json` + `forbid_claim`，无豁免；
  acceptance 为最小切片且列明未验收边界（除 §7-2 一项外未放宽）；模块归属 `modify: mjwarp.physics.core`，
  与实际代码位置（`src/physics/collision/narrowphase*`）一致。
- 交付提交仅新增模块/测试/fixture/authoring 并挂接 `collision.rs`、`mod.rs` 导出；既有 G08 测试只增
  `mod plane_box;` 与 import（5 行），未改断言/容差。
- 批次 4 索引 §3 的全量计数写「见同批门禁记录」，g09 门禁记录 478/0 与本次复跑一致。

## 9. 边界与证据索引

- 本复核只读交付与记录，只写本目录；未改产品/测试/容差/fixtures/回执；未触碰 Backend 新登记的
  跨批次收口审计任务与 W13/Linux 任务。
- 被复核证据：`docs/development-evidence/backlog-batch4/…/`、`docs/development-evidence/changes/2026-10-11-roadmap-g09-p5.json`、
  `.normify/runs/2026-10-11-roadmap-g09-p5/`、两条顺延任务记录。
- 结论：g09 数值/接触声明成立、边界如实；两条顺延如实；差异见 §7（2 一般 + 4 建议，均不阻断）。
