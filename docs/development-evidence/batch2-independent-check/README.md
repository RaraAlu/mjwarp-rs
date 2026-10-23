# 批次 2 独立复核（7 条交付 + R01 顺延记录）

任务：`2026-10-23-batch2-independent-check`。日期：2026-10-23。执行：QA（独立复核，非交付人）。
对象：批次 2 的 7 条交付（提交链见 §2）与 `r01-p8` 顺延记录。
范围：只读复核 + 本证据目录；未改产品/测试/容差/fixtures/回执；未碰 Backend 正在做的批次 3。

## 1. 结论（先给）

- **七条交付的数值/契约声明均被证据与独立复跑支持**（逐项见 §2）。
- 独立复跑对应 lib 测试全过：`collision` 5/0、`integrator` 3/0、`inverse`（模块 2 项 + 2 项跨模块同名匹配）4/0、`derivative` 2/0、`sensor` 2/0、`external::view` 5/0、`tooling` 5/0；全量 `cargo test --locked` **461/0**（82 个 result 行）。
- 8 个模型源 fixture 的 sha256 与 `authoring.json` **8/8 一致**；交付提交内 evidence/fixtures/source blob 与回执一致，**唯一例外是 g20-p7 的 README 回执 blob**（§5-①）。
- checks 均含 fmt/clippy/unit/unchanged/json/forbid_claim；细化/交付提交**未删除验收要求、未放宽容差**；模块归属与实际代码位置一致（§5-③）。
- **`r01-p8` 顺延记录如实**：任务保持 `proposed`、未启动；note 写明阻塞于渲染/BVH 子系统并登记恢复条件；批次 2 索引 §2 与之一致；无证据目录、无交付声明。
- 无阻断问题；发现 1 条一般级回执追溯差异 + 1 条建议级文档标注问题（§5）。

## 2. 七条声明逐项核对

| 任务（提交） | 声明 | 证据位置 | 复核结果 |
| --- | --- | --- | --- |
| G07@P5（`12151dc`） | 47 用例候选 `(worldid,pair,pairid)` 多集合逐案相等；原生 27/27 接触对包含；边距 6/6 计数相等 | fixture `broadphase-cases.json`（47 cases）；`authoring.json.cases=47`、`native_cross_check{27,27,0,6,6}`；`collision/tests.rs:226` `assert_eq!(cases.len(), 47)` 及候选/原生/边距断言 | ✅ `lib collision` 5/0 |
| G16@P6（`9692c92`） | euler 位型 64/72、RK4 67/72，max abs 5.96e-08；原生 1.19e-07/6.26e-06 | `authoring.json.native_cross_check`（euler 1.192e-07、rk4 6.258e-06）；fixture `tolerance{upstream_mul:1,native_abs:1e-5}`；回执同值 | ✅ `lib integrator` 3/0（含合成 ODE 收敛阶） |
| G20@P7（`a10aea5`） | qfrc_inverse vs 上游 1.91e-06；原生 2.62e-06；回代力平衡 2.62e-06 | 回执 `results.comparison` 同值；fixture `tolerance{upstream_abs:1e-5,native_abs:1e-5,force_balance_abs:1e-5}`；测试对上游/原生/平衡三路断言 | ✅ `lib inverse` 模块 2 项全过 |
| G21@P7（`cef3357`） | 多项式 18/18 逐位一致；上游 f32 差分 1.55e-06；原生 f64 差分 4.67e-08 | `authoring.json.analytic_vs_finite_difference`（1.5497e-06/4.673e-08）；fixture `tolerance{upstream_abs:1e-6,fd_abs:1e-4}` | ✅ `lib derivative` 2/0 |
| G23@P7（`34bbd05`） | sensordata 1.19e-07；原生 5.96e-08；cutoff 0.8→0.4、时序/单位 | 回执同值；fixture `tolerance{upstream_abs:1e-6,native_abs:1e-5}`、3 用例、slide_pos cutoff=0.4 | ✅ `lib sensor` 2/0 |
| A08@P10（`55a6cb8`） | 指针==原缓冲+offset；跨流/未完成/代数/越界 5 类拒绝 | `src/runtime/external/view/tests.rs` 5 个测试（零拷贝/跨步/事件/代数/边界）；回执 verification 同 | ✅ `lib external::view` 5/0 |
| A10@P10（`3757567`） | JSON 往返、拒绝面、批次 2 公开 API 冒烟 | `src/runtime/tooling/tests.rs` 5 个测试（聚合/无头计时/状态往返/报告 JSON/批次 2 冒烟）；回执同 | ✅ `lib tooling` 5/0 |

补充：七条 README 都列出未验收边界（G07 SLEEP/incremental/窄相、G16 activation/阻尼隐式、G20 DISCRETE/inv_constraint、G21 其余导数项、G23 其余传感器/阶段、A08 真实设备路径、A10 GPU 性能），并明确不宣称完整族验收、NP04 未测。

## 3. 独立复跑（CPU，默认特性）

```text
cargo test --locked --lib collision        -> 5 passed; 0 failed
cargo test --locked --lib integrator      -> 3 passed; 0 failed
cargo test --locked --lib inverse         -> 4 passed; 0 failed（inverse 模块 2 项 + 2 项跨模块同名匹配）
cargo test --locked --lib derivative      -> 2 passed; 0 failed
cargo test --locked --lib sensor          -> 2 passed; 0 failed
cargo test --locked --lib external::view  -> 5 passed; 0 failed
cargo test --locked --lib tooling         -> 5 passed; 0 failed
cargo test --locked                       -> 461 passed; 0 failed（82 个 result 行，unit 门禁命令）
cargo test --locked --lib                 -> 341 passed; 0 failed（仅 lib 目标，对照用）
```

## 4. sha256 / 回执抽验

- 模型源 sha256 与 authoring **8/8 一致**：`broadphase/{mixed,filters,margin,mesh}.xml`、`integrator/model.xml`、`inverse/model.xml`、`deriv/model.xml`、`sensor/model.xml`。
- 输出 fixtures 均存在且被测试读取：`broadphase-cases.json`（47 用例）、`integrator/cases.json`、`inverse/cases.json`、`deriv/cases.json`、`sensor/cases.json`。
- 回执 `files[].blob` 与交付提交 `git ls-tree` 比对：**6/7 完全一致**；g20-p7 的 README 不一致（§5-①）。本批未见模块索引 blob 漂移（回执在 done 后生成，与 §9 口径一致）。

## 5. 差异清单（按严重级别）

**① 一般（回执追溯）：g20-p7 README 回执 blob 与提交内文件不一致**
- 回执记录 `ecd16262…`，提交/HEAD 为 `2638ad1c…`；`git log` 显示 README 仅在 `a10aea5` 添加、之后未修改。
- README 内容与 fixture/测试一致（1.91e-06/2.62e-06/2.62e-06 均有 fixture 与测试支撑），不影响数值声明；建议按 `batch1-independent-check/README.md` §9 的回执口径，用 `normify fix` 记录权威值并刷新该回执 blob。

**② 建议（文档标注）：unit 门禁命令被写成 `cargo test --locked --lib`**
- 批次 2 索引 §3 与 A10 回执写「全量 lib 461 过 0 败 / `cargo test --locked --lib`」；实际 `.normify/gates.toml` 的 unit 门禁命令是 **`cargo test --locked`（全目标）**，461 与之一致；单独 `--lib` 为 341。
- 数字（461/0）无误，仅命令名/口径标注需要修正为「`cargo test --locked`（全目标）」。

**③ 已核对无问题：**
- 模块归属：G07/G16/G20/G21/G23 → `mjwarp.physics.core`（代码在 `src/physics/*`）；A08 → `mjwarp.runtime.external`（`src/runtime/external/view.rs`）；A10 → `mjwarp.runtime` + `mjwarp.runtime.tooling`（`src/runtime/tooling`），均与实际位置一致。
- acceptance：细化提交仅追加切片说明/边界（末条连接行加逗号 + 新增说明），交付提交 0 行改动；无删除验收要求、无放宽容差。
- checks：七条均含 gate fmt/clippy/unit、unchanged、json、forbid_claim；`.normify/runs/<task>/` gate 记录均 passed。
- NP01：物理切片全链 f32；A10 诊断统计 f64 仅统计、不反馈产品物理，已声明。

## 6. r01-p8 顺延记录核对（结论：如实，无交付声明）

- 任务状态：`proposed`（未启动、未交付）；`schedule.files` 为空；无 `docs/development-evidence/backlog-batch2/2026-10-11-roadmap-r01-p8/` 证据目录。
- note（附加段）如实写明：「批次2顺延（Chief 裁决 A，Backend 执行）：保持 proposed、不启动；阻塞于渲染/BVH 子系统——上游 `render_util.create_render_context` 依赖 BVH 构建/重拟合、网格与纹理资产上传、CUDA 渲染内核及相机/采样参数（约 30 个参数），当前引擎未实现该能力；不做宿主侧布局探针（假语义）；顺延到渲染能力具备后（批次 3 或独立渲染计划）再按原验收细化交付」。
- 批次 2 索引 `docs/development-evidence/backlog-batch2/README.md` §2 与之一致（顺延、未宣称交付）；提交 `ed7de14` 只改该任务 note，未附带任何证据/代码。
- 未发现任何将 r01-p8 表述为「已交付/已验收」的文字。

## 7. 边界

- 只做独立复核与证据记录；未改产品/测试/容差/fixtures/回执；未 start/回退任务。
- 未重新生成 offline authoring 参考（需 pinned venv + GPU）；未复核 GPU 内核（七条均 host 侧切片）。
- 未碰 Backend 正在做的批次 3（g08-p5 等 5 条）。
- 复跑时 Backend 的批次 3 WIP 尚未进入 `src/`（仅任务 JSON/fixture/authoring），lib 编译目标与 HEAD 一致。

## 8. 证据索引

- 本文件：逐项核对、复跑计数、sha256、差异、r01 核对。
- 被复核证据：`docs/development-evidence/backlog-batch2/<task-id>/{README.md,authoring.json}`、`docs/development-evidence/changes/<task-id>.json`、`.normify/runs/<task-id>/`、`docs/development-evidence/backlog-batch2/README.md`。
