# 批次 3 独立复核：G08/G18/G24/M10 四条交付、证据迁移与 A10 顺延

日期：2026-10-23。执行：QA。任务：`2026-10-23-batch3-independent-check`。
复核对象（HEAD `18b3fb5` 时）：

| 对象 | 提交 |
| --- | --- |
| G08@P5 基本几何碰撞 | `2269b04`（登记 `6ebe9ac`） |
| G18@P6 离散积分 | `9239fa5`（登记 `88b3593`） |
| G24@P7 历史与延迟 | `9cdf5de`（登记 `9d9e3e5`） |
| M10@P7 Rust 回调契约 | `b9cd236`（登记 `c315f49`） |
| 证据迁移/批次 3 索引 | `f31cfc2`、`5ccfe1e`、`c7b0cfb`、`7544a45`、`6e3c427` |
| A10@P11 顺延登记 | `02faa66` |

## 1. 结论（先给）

四条声明**全部成立**，无阻断/严重问题；证据目录迁移与 `a10-p11` 顺延登记如实；未发现改断言/放宽容差/伪造数字。
发现 **1 条一般**（G24 读取「逐位一致」的断言弱于声明）与 **7 条建议**（见 §7），均不阻断。

## 2. 四条声明逐项核对

### 2.1 G08@P5 基本几何碰撞

| 声明 | 复核方式 | 结论 |
| --- | --- | --- |
| 接 G07 候选 5 条与上游逐条一致 | 测试对 `(worldid,pair,pairid)` 排序后 `assert_eq` 冻结上游 | 成立 |
| 接触数量 5 = 上游 `nacon` 5 = 原生 `ncon` 之和（世界 0:3、世界 1:2） | fixture `contacts.count=5`、`native.ncon=[3,2]`；测试逐世界 `assert_eq!(world_contacts.len(), ncon)` | 成立 |
| 接触字段 vs 上游 max abs 7.45e-09（≤1e-6） | 独立 `--nocapture` 实测 `upstream_max=7.451e-9`；对照覆盖 `dist/pos/frame/includemargin/friction/solref/solimp/adhesion` 8 类浮点 + `dim/geom/type/worldid` 4 类整数逐位 + `geomcollisionid` 断言 0 | 成立（覆盖口径见 §7-2/§7-3） |
| 原生 f64 `mj_collision` max abs 5.96e-08（≤1e-5） | 独立实测 `native_max=5.960e-8`；测试直接对比产品接触与原生 fixture | 成立（fixture 以 f32 存储原生值，见 §7-4） |

其余核对：接触输出共 14 项字段（`dist/pos/frame/includemargin/dim/geom/worldid/friction/solref/solreffriction/solimp/adhesion/type/geomcollisionid`）；
`solreffriction` fixture 全 0、实现为常量 `[0,0]`；摩擦下限 `MJ_MINMU` 有逐值断言；材质混合（condim/solmix/priority）
未单独断言混合规则，而是由 `dim/friction/solref/solimp/adhesion` 的逐字段对照间接覆盖（与上游冻结值一致）。

### 2.2 G18@P6 离散积分

| 声明 | 复核方式 | 结论 |
| --- | --- | --- |
| DISCRETE 单步位型 18/22、max abs 5.96e-08（≤1e-6） | 独立 `--nocapture` 实测 `bitwise=18/22 upstream_max=5.960e-8`；测试按位型计数并断言逐值 ≤ fixture 容差 1e-6 | 成立 |
| 原生 f64 max abs 5.96e-08（≤1e-5） | 独立实测 `native_max=5.960e-8` | 成立（fixture 以 f32 存储原生值，见 §7-4） |
| 与 G16 `euler` 直接分支逐位等价 | 同一 qacc 下 `assert_eq!(step.qpos, euler_step.qpos)`/`qvel` | 成立 |
| 有效度量分支返回 `UnsupportedDiscreteAcc` | `discrete_acc(DampedEuler,…)` `assert_eq!(err, IntegratorError::UnsupportedDiscreteAcc)`；恒等分支 `assert_eq!(identity, qacc)` | 成立 |

### 2.3 G24@P7 历史与延迟

| 声明 | 复核方式 | 结论 |
| --- | --- | --- |
| 上游回放 4 场景缓冲字逐位一致 | fixture `mechanics.scenarios` 4 个；测试逐字 `bits_equal` 且 `assert_eq!(word_mismatches, 0)` | 成立 |
| 33 次读取共 51 个值逐位一致、max abs 0 | fixture 读取次数实测 33（9+6+9+9）、值数 51；独立 `--nocapture` 实测「读取 51 值，位型不一致 0，max abs 0.000e0」 | 数值成立；**断言只到 1e-6，逐位计数仅打印**（§7-1） |
| 4 个重置用例 + SO3+QUAT 逐位一致 | 测试对每个重置用例逐字 `bits_equal` 断言；合成 SO3+QUAT 逐字断言 | 成立 |
| 原生 C f64 `mj_resetData` 2.682e-09（≤1e-6） | 独立实测 `native_max=2.682e-9`；测试比较对象为「冻结上游初始字 vs 原生 C f64」（`native_words` 为真 f64），产品与上游逐位一致故传递成立；证据 README 的「产品 vs 原生」为传递口径，本复核确认成立 | 成立 |

### 2.4 M10@P7 Rust 回调契约

| 声明 | 复核方式 | 结论 |
| --- | --- | --- |
| 七槽位契约：字段顺序/默认值与冻结上游 `types.Callback` 一致 | 测试对比 `CallbackKind::ALL` 与 fixture；**另用真实上游快照独立核对**：`target/mjwarp-parallel/upstream`（HEAD `71da24d9…`）`types.py:1061-1067` 字段顺序 `passive/control/act_dyn/act_gain/act_bias/sensor/contactfilter`、默认全 `None`，与 fixture/README 一致 | 成立 |
| `Stage.POS/VEL/ACC=1/2/3` 与上游一致 | fixture 双份记录（`sensor_stage_values`/`stage_enum_values` 均 1/2/3），测试对 `CallbackStage::sensor_value()` 断言；上游 `types.py:236-247` 用 `mjtStage` 定义 | 成立 |
| 调用计划顺序 contactfilter→sensor POS→passive→sensor VEL→control→act_dyn→act_gain→act_bias→sensor ACC | 测试对 9 个调用逐项断言并检查阶段非降序、act 阶段固定次序 | 成立 |
| ACTUATION 禁用时 control 跳过、sensor 三次触发 | 两个独立测试分别断言 | 成立 |
| 7 测试 | `cargo test --locked --lib callback` 实测 7 过 0 败 | 成立 |
| **明确不是阶段回调** | 证据 README §1「本片**不是**阶段回调：不实现 forward/step…」、§6 首条；回执 `boundaries` 首条；批次 3 索引表格边界一致 | 如实，无阶段调度/回调执行实现（`callback.rs` 无 execute/invoke/run，`plan` 仅返回 `Vec<CallbackInvocation>`） |

调用点 `file:line` 独立抽验（同一冻结上游快照）：`collision_driver.py:983-984`、`passive.py:3695-3696`、
`sensor.py:963-964`（Stage.POS）、`sensor.py:1509-1510`（VEL）、`sensor.py:2847-2848`（ACC）、
`forward.py:1916-1921`（act_dyn/gain/bias）、`forward.py:2081-2082`（control 门控）全部与证据 README §3.1 一致。

## 3. 独立复跑（CPU，默认特性）

| 命令 | 结果 |
| --- | --- |
| `cargo test --locked --lib narrowphase` | **3 过 0 败** |
| `cargo test --locked --lib discrete` | **2 过 0 败** |
| `cargo test --locked --lib history` | **6 过 0 败** |
| `cargo test --locked --lib callback` | **7 过 0 败** |
| `cargo test --locked`（全量） | **477 过 0 败**（lib 357，其余集成/文档测试；复核时 HEAD `18b3fb5`。本任务门禁重跑时为 478 过 0 败，因 Backend 批次 4 WIP 在途，不改变批次 3 结论） |
| `cargo fmt --check` / `cargo clippy --all-targets --locked -- -D warnings` | 通过 |

`--nocapture` 实测数值：`narrowphase candidates=5 contacts=5 upstream_max=7.451e-9 native_max=5.960e-8`；
`discrete bitwise=18/22 upstream_max=5.960e-8 native_max=5.960e-8`；`history 读取 51 值 0 不一致 max abs 0`、
`重置用例 4 个 native 2.682e-9`——与声明逐项一致。

## 4. fixtures / sha256 / 回执抽验

- 模型 sha256：`fixtures/narrowphase/model.xml` `686a4ff5…`、`fixtures/discrete/model.xml` `e4945358…`、
  `fixtures/history/model.xml` `5f52046d…`，与三条回执 `references.model_sha256` 及 README 一致（3/3）。
- `fixtures/callback/contract.json` `ba2b2a3f…`（回执登记 blob 一致），字段/Stage 内容与真实上游快照独立核对一致。
- 四条回执中证据/fixture/authoring 文件登记 blob：**24/24 与 HEAD 一致**。
- 交付源码登记 blob：除共享 `src/physics/mod.rs` 与 `.normify/modules/mjwarp/physics/core.md` 外全部与 HEAD 一致；
  这两个共享文件在各自交付提交内逐位一致（g08 `93d09434…`、g18 `18a4589f…`、g24 `7d0bffd8…`、m10 `bc841e4e…`；
  `core.md` 同理），后续任务刷新共享文件属 §9 已登记类，**不算回执伪造**。

## 5. 证据目录迁移核对

- `f31cfc2`：g08/g18 共 4 个证据文件为 **R100 重命名**（blob 前后 EQUAL：`37386fd7…`/`8e4af0a6…`/`5d4f772d…`/`910c52a4…`）；
  两条回执只改 `files[].path`（batch2→batch3），blob 行未变。
- `5ccfe1e`/`c7b0cfb`：`normify fix` 只追加 addenda 并重跑门禁，改动仅 `.normify/changes/<task>.json` 与 `.normify/runs/`，
  **未触碰证据文件**；addenda 写明「目录迁移、内容与 blob 不变、结论/数字/验收不变」。
- `backlog-batch3/`：索引 4 条 vs 实际 4 个证据目录，一致；`backlog-batch2/` 只剩批次 2 七条证据目录 + 迁移说明。
- `backlog-batch2/README.md` 迁移说明把 `g24-p7` 也列入「已原样迁至」（实际 g24 从未落在 batch2，见 §7-6）。
- 已关闭任务记录里 `acceptance`/`schedule.files` 仍含旧 `backlog-batch2/…` 路径（不可改写，见 §7-5）。

## 6. a10-p11 顺延登记核对

- `.normify/changes/2026-10-11-roadmap-a10-p11.json`：`status=proposed`、`checks=0`；
  无 `.normify/runs/2026-10-11-roadmap-a10-p11/`、无证据目录、无交付回执（均实测不存在）。
- note 已登记裁决 A：保持 proposed 不 start；阻塞项（Linux RTX5080 / 真实 GPU 事件计时与显存统计 /
  清洁部署 / T4+Linux 双平台准入）；P10 宿主侧能力由 `3757567` 覆盖（该提交存在且为 A10 P10 交付）；
  恢复条件（Linux 与完整 GPU 引擎具备后重评）；不硬做 `main.rs` CLI 接线（假语义）。
- 批次 3 索引 §2 已由 `6e3c427` 更新为「顺延·裁决 A」，与 note 一致；`02faa66` 仅改 note，未改源码。
- 未发现任何交付/验收声明。

## 7. 差异清单（按严重级别）

### 7.1 一般：G24 读取「逐位一致」的断言弱于声明

- 复现：`src/physics/history/tests.rs:94-104`，读取值逐项 `assert!((actual-expected).abs() <= 1e-6)`，
  `read_mismatches` 只累加并在 `println!` 打印，**未断言为 0**。
- 期望：既然声明「51 值逐位一致、max abs 0」，应有 `assert_eq!(read_mismatches, 0)`（或 `read_abs == 0`）。
- 实际：当前独立实测确为 0 不一致、max abs 0；但回归时若出现 ≤1e-6 的位型差异，测试不会失败，声明无法由断言守住。
- 影响：仅测试强度，不否定本次数值结论。建议后续补断言。

### 7.2 建议：G08 两个字段未直接对照

- `solreffriction` 在 fixture 中冻结（5 条全 0），测试未读取断言；实现为常量 `[0,0]`。
- `geomcollisionid` 断言产品值为 0，未与 fixture 值比较（fixture 亦全 0）。
- 建议补断言或在证据注明「常量字段不走数值对照」，避免「14 项字段全部对照」的表述被误读。

### 7.3 建议：G08 接触键集合断言可加强

- 测试断言：长度相等 + 每个产品接触能在上游键表中找到 + 上游键不重复。
- 未显式断言「产品键集合 == 上游键集合」：若产品出现重复键而漏掉另一个上游键，长度相等仍会通过。
- 本次实测按键唯一且完整；建议补集合相等断言。

### 7.4 建议：G08/G18「原生 f64 一级判定」经 f32 存储

- fixture 中原生值以 f32 存储（测试用 `f32_list` 读取），实际比较为「产品 f32 vs 原生 f32 舍入值」，
  而非真正 f64 比较；G24 的 `native_words` 才是 f64 存储。
- 容差 1e-5 远大于 f32 ULP（~6e-8），不影响结论；建议口径注明或原生值保留 f64。

### 7.5 建议：已关闭任务的旧路径文本不可改写（已知追溯差异）

- `.normify/changes/2026-10-11-roadmap-g08-p5.json`/`g18-p6.json` 的 `acceptance` 与 `schedule.files`
  仍写 `docs/development-evidence/backlog-batch2/2026-10-11-roadmap-*`，该目录已迁移不存在。
- 规范禁止改写已关闭任务的声明/验收，迁移 fix 的 addenda 已记录；登记为**已知追溯差异**，
  不重写历史、不影响证据与结论。

### 7.6 建议：批次 2 迁移说明措辞

- `backlog-batch2/README.md` 写「批次 3 任务（g08-p5、g18-p6、g24-p7）的证据…已原样迁至 backlog-batch3」；
  实际 `g24-p7` 从未落在 batch2（交付时即写 batch3）。建议改为只列 g08/g18，避免误导。

### 7.7 建议：G18 authoring 原生对照数值未说明

- `authoring.json` 的 `native_cross_check.max_abs=3.02e-08` 是「上游 f32 结果 vs 原生 f64」；
  证据 README 的 `5.96e-08` 是「产品 f32 vs 原生（f32 存储）」。两者对象不同、均成立，
  建议在证据中注明，避免被读成矛盾。

### 7.8 建议：交付提交说明含控制字符

- `2269b04`（G08）、`9239fa5`（G18）提交说明各含 1 个 `\x0c`（form feed），`b9cd236`（M10）含 2 个 `\x09`（TAB），
  来源应为说明文本中的 `\f`/`\t` 转义被解释，导致日志显示为「ixtures」「ypes.Callback」等字符缺失。
- 仅影响提交日志可读性，代码/证据/验收不受影响；提交已入库，不建议为此改写历史，后续提交注意转义。

## 8. checks / acceptance / 模块归属

- 四条 checks 均含 `gate fmt/clippy/unit`、`unchanged`（src/native/include/tests/fixtures/scripts/Cargo.*）、
  `json` 回执 task_id、`forbid_claim`；无豁免记录。
- acceptance 均为「最小可交付切片」并有明确未验收边界（未放宽）；交付提交未改动既有产品路径、容差与既有测试
  （改动仅新增模块 + `mod.rs` 挂接 + `integrator.rs` 新增错误变体）。
- 模块归属均登记 `modify: mjwarp.physics.core`，与实际代码位置
  （`src/physics/{collision/narrowphase, integrator/discrete, history, callback}`）一致。
- 门禁单元记录：G24 交付时 470、M10 交付时 477；G08/G18 的 `unit.json` 被迁移 fix 重跑覆盖为 470
  （fix 提交已写明重跑；交付提交说明中的 464/466 与当时时间线一致，非矛盾）。

## 9. 边界与证据索引

- 本复核只读交付与记录，只写本目录；未改产品/测试/容差/fixtures/回执，未触碰批次 4 WIP
  （`g09-p5`/`g10-p5`/`m10-p10`）与 W13/Linux 任务。
- 被复核证据：`docs/development-evidence/backlog-batch3/<task-id>/{README.md,authoring.json}`、
  `docs/development-evidence/changes/<task-id>.json`、`.normify/runs/<task-id>/`、`.normify/changes/<task-id>.json`。
- 迁移与顺延记录：`f31cfc2`、`5ccfe1e`、`c7b0cfb`、`7544a45`、`6e3c427`、`02faa66`。
- 结论：四条声明成立、迁移与顺延如实；差异见 §7（1 一般 + 7 建议，均不阻断）。
