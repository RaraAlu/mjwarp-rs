# QA 复核：4 张 NP 实现阻塞卡 + W9 采集脚本抽模块 backlog 登记（`0cf8307e` 登记 + `f5be136b` 收口，卡 H）

- 任务：`2026-11-11-np-blockers-and-w9-backlog-registration-check`（owner QA）。复核对象：提交 **`0cf8307e`**（登记 5 张卡 + 卡 H）+ **`f5be136b`**（卡 H 收口/门禁）+ 卡 `2026-11-11-np-blockers-and-w9-backlog-registration`。零 GPU、只读复核。
- 基线：卡 G 复核落库后（`251ed793`）。

## 结论

**复核通过（0 阻断，0 低 + 2 观察）。** 4 张 NP 实现阻塞卡与 1 张 W9 backlog 卡登记正确（`status=backlog`、`checks=[]`、id 与 NP5-00 账本/README 的 4 个缺口名**逐字对应**、acceptance 只写「启动时细化」不预设通过、note 明写不纳入排程/待用户裁决）；卡 H 本身 10 个检查（4 门禁 + `unchanged src/tests/fixtures/scripts` + **5 条 `json /status == backlog` 守卫**）verified；两个提交**只写 `.normify`**，无代码/判据改动。

## 一、4 张 NP 实现阻塞卡（✅）

| 卡 id | status | checks | deps | priority | 与 NP5-00 账本对应 |
| --- | --- | --- | --- | --- | --- |
| `2026-11-11-np-blocker-flex-collision` | backlog | [] | 空 | 20 | ✅ 柔体碰撞（`collision_flex.py`） |
| `2026-11-11-np-blocker-constraint-solver` | backlog | [] | 空 | 18 | ✅ 约束求解 |
| `2026-11-11-np-blocker-solver-pipeline` | backlog | [] | 空 | 18 | ✅ 求解器管线 |
| `2026-11-11-np-blocker-island-sleep` | backlog | [] | 空 | 15 | ✅ 岛屿休眠 |

- 四个 id 与 NP5-00 README §4.3 / 账本「7 个未实现子系统重核」的 4 个真未实现项**逐字一致**（其余 3 个已实现项不建阻塞卡，走 NP5-01b 证据登记——与账本一致）。
- 每张卡：intent 记录上游文件与「产品无对应路径」的缺口事实；acceptance 三条 = 所需实现 / **验收判据（启动时细化并补机器检查）** / 依赖波次；note 明写「**只登记、不排期**」「是否排进 W2/W3/R07 由**用户裁决**」「`checks=[]`（**不预设通过**），启动前必须补机器检查项」「不宣称 NP 通过或实现阻塞解除」「零 GPU、零窗口、不改判据/参考/容差」。无任何结论性表述。

## 二、W9 采集脚本抽模块 backlog 卡（✅）

- `2026-11-11-scripts-w9-capture-extraction-backlog`：`status=backlog`、`checks=[]`、priority 12；intent 记录 `scripts/capture_render_perworld.py` 现 **490 行**（500 行棘轮）、拟纯移动 56–196 行（141 行、18 常量 + 6 函数）→ 新 `scripts/capture_perworld_env.py`（主脚本 ≈355–360 行）。
- **阻塞口径明确**：新模块是否纳入 **guards 93 输入**（纳入则 93→94，破坏已签 S-3 spec 的 107 口径，需用户重签；不纳入则缺 guards sha 保护）——**待用户裁决，未裁决前不 start、不写仓**；并写明需同步 `freeze-guards-perworld-l2-light.ps1` 第 167 行 pinned sha、下次渲染窗重冻。
- acceptance：裁决后执行口径（逐字节搬迁证明、AST 等价（pinned venv 3.12.13）、py_compile/`--help` 冒烟、脚本身份三值、guards 影响清单）；边界不改既有采集脚本语义/参数/输出、不改判据、不重写历史证据、零 GPU/零窗。

## 三、卡 H 本身 / 边界 / 门禁（✅）

- 卡 H：`status=verified`、`closed_at` 已填、10 检查 = 4 门禁 + `unchanged src/ tests/ fixtures/ scripts/` + **5 条 `json …/status == backlog`**（防 5 张登记卡被误启动/改态）；门禁 fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**。
- 两个提交：`git show --name-only` 全部为 `.normify/changes/*`（6 个卡 JSON）与 `.normify/runs/*`（门禁记录）——**无** `src/`/`tests/`/`fixtures/`/`scripts/` 改动、guards/冻结件未动、不改判据/参考/容差；零 GPU、零窗。
- 登记来源可核：卡 H note 指向「NP5-00 账本 190 条（`875ae41d`）+ np5-rulings-memo §2.1」，与 NP5 证据一致（该账本/README 我已在卡 F 复核中核过）。

## 四、观察（不阻断）

- **O1**：4 张阻塞卡 `depends_on` 为空、priority 20/18/18/15，且 `status=backlog`（非 proposed）——符合「只登记、不排期、待用户裁决是否排入 W2/W3/R07」的定位；建议在用户裁决后再补 `depends_on`/`checks` 与 phase，避免被误当就绪卡。
- **O2（正向）**：延续卡 G 的做法，H 用 5 条 `json /status == backlog` 把「登记卡不得被顺手启动」做成机器可验证约束，建议后续登记型卡沿用。

## 五、口径

不宣称 NP 通过、不宣称实现阻塞解除；不改判据/参考/容差；零 GPU、零窗口；本复核只核登记内容/依赖/checks/写域与边界。
