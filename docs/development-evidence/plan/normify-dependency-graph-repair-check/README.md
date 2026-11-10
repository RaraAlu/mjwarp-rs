# QA 复核：normify 依赖图修复（`496c01e6`；2026-11-11）

- 卡：`2026-11-11-normify-dependency-graph-repair-check`（owner QA）
- 被复核：`2026-11-11-normify-dependency-graph-repair`（Arch，提交 **`496c01e6`**，parent **`c6ea92b8`**）
- 性质：**只读复核**；未改任何被复核内容（53 条任务记录、被复核卡、证据目录逐字节未动）；零 GPU、不采集、不开窗。
- 结论：**PASS（0 阻断）**，边界未越界；4 条观察项均为信息级。

## 1. 复核方法（独立重算，不采信描述）

1. 用 `git archive` 把 **parent 与 HEAD 两棵 `.normify/changes` 树**（999 / 1000 条记录）解到 `target/qa-dep-graph/`，自建 PowerShell 5.1 脚本独立重建**任务状态表与依赖图**（不调用 normify 的解析逻辑）。
2. **逐键比对**：对 53 条被改记录做 JSON **叶子路径 diff**（自建递归 flatten，比 `ConvertFrom-Json` 更细），记录「差异路径集合」，再按允许集 `{ROOT.note, ROOT.schedule.depends_on[*]}` 判定越界。
3. **逐边归因**：解析被复核证据 `artifacts/dependency-diff.tsv` 的 basis 列（`旧卡 -> 后继, 后继`），把每条 abandoned 边映射到实际 added 集合，并校验每个命名后继的**当前状态**。
4. **图完整性**：自建迭代 DFS 检测自依赖 / 环 / 悬空引用；重算 ready/blocked，与 `normify status` 对账。
5. **边界**：`git diff --name-only parent..HEAD` 分路径统计；对 W0/G-L2 冻结输入**逐文件比 `git rev-parse` blob id**（脚本、冻结器、`scene.xml`、spec、S-3 材料树 16 文件、参考 5 文件）。
6. **证据自核**：独立重算被复核 manifest 7/7 的 sha256/bytes；逐文件扫 BOM/CR/TAB；把 README §3 的「修后仍阻塞」列与实测「当前非 verified 依赖集合」**逐行**比对。
7. **门禁**：HEAD 上独立复跑 fmt / clippy / clippy-cuda / `cargo test --locked`，并对 `normify check` / `normify validate` 复核。

## 2. 逐点结论

### 2.1 边界不可越界（最高优先）—— 通过
- `496c01e6` 共 **66 文件** = 58 `.normify`（**53 条任务记录 + 1 新卡 + 4 门禁 runs**）+ 8 条被复核证据；无其它路径。
- `git diff --name-only 496c01e6^ 496c01e6` 分路径：`src/`=0、`tests/`=0、`fixtures/`=0、`scripts/`=0、`docs/` 中 plan 目录外=0。
- **W0/G-L2 冻结输入逐文件核实（parent blob == HEAD blob）**：

| 冻结输入 | blob（前 12 位） | 相同 |
| --- | --- | --- |
| `scripts/capture_render_perworld.py` | `e57d45a6bdea` | ✅ |
| `scripts/perworld_capture_meta.py` | `20216d19e676` | ✅ |
| `scripts/perworld_light_prep.py` | `d89c3fbb4ccf` | ✅ |
| `scripts/validate/render-bvh/freeze-guards-perworld-l2-light.ps1` | `06e5cbde0fa4` | ✅ |
| `fixtures/render-bvh/scene-a-light/scene.xml` | `f4376874dfb7` | ✅ |
| `docs/…/perworld-l-v2-light-criterion/spec.md`（39,379 B / `bb064fe0…`） | 未变 | ✅ |
| S-3 材料树（16 文件） | — | 16/16 ✅ |
| 参考 5 文件 `…/nw3-light-hinge02040/` | — | 5/5 ✅ |

### 2.2 只改依赖与 note —— 通过
- 53 条记录**差异路径 100%** 落在 `ROOT.note` 与 `ROOT.schedule.depends_on[*]`；**越界差异路径 0 条**（明细见 `artifacts/key-diff.txt`）。
- `title`/`intent`/`acceptance`/`modules`/`schedule.phase`/`owner`/`files`/`priority`/`status`/`revision`/`created_at`/`updated_at`/`checks` **逐字节相同**。
- 53 条 `note` 全部是**纯追加**（旧 note 是新 note 的前缀），无改写、无删除。
- **status 迁移 0 条**：abandoned 115→115、backlog 40→40、proposed 18→18、verified **826→827**（+1 = 本题新卡自身），记录 999→1000。**未 start / 未 verified 任何占位卡**。

### 2.3 死边清零且可追溯 —— 通过
- 开放任务 58；依赖 abandoned 卡的边 **65 → 0**；含死依赖的开放任务 **17 → 0**；依赖边 241 → 293（净 **+52**）。
- 算术自洽：`−65 死边 +108 后继 +1 T4 +8 Linux +5 伞卡 −5 伞卡 = +52`（伞卡 5 条为等量替换）。
- **逐边归因**：65 条 abandoned 边**全部**在 basis 中带自身依据；**62 条箭头映射 + 2 条 waive + 1 条 drop**；每个命名后继都真实出现在该任务的 added 集合且状态 **verified**（bad = **0**；命名后继引用 128 条，去重后 113 条 = 108 后继 + 5 伞卡收口）。
- added 依赖目标状态分布：**verified 113 / backlog 9**；9 条非 verified 恰为 **Linux 8 + T4 1**，与声明一致。

### 2.4 例外如实 —— 通过
- `a06-p10`：abandoned，其自身 note 逐字含「放弃：分诊b：A06 执行图已由 src/runtime/launcher/graph.rs 与 launcher_graph 测试实现。；依据：src/runtime/launcher/graph.rs；tests/launcher_graph.rs」；两文件实存（14,055 B / 14,969 B）→ **2 条 waive 有依据、无卡可指**。
- T4：drop `2026-10-16-next-t4-artifacts`（abandoned）；`next-t4-numeric` 现依赖含 `2026-10-27-t4-historical-capture-rebaseline-decision`（backlog）→ 「等用户 A/B/C」成为图上真实阻塞。
- 伞卡：`r06-p8` / `r10-p8` 下游 5 条改指 `2026-11-04-render-r06-flex-closeout`、`2026-11-04-render-r05-r09-r10-family-closeout`（均 **verified**），占位伞卡本身**仍 proposed**。
- Linux：8 条任务（`p1`/`d02`/`d03`/`d03-final`/`p11`/`layers-linux`/`np03-dual-platform`/`np04-performance`）全部追加 `2026-10-24-linux-batch-retest`（backlog），与声明清单**逐条一致**。

### 2.5 图完整性与就绪 —— 通过（含 1 条已知缺口）
- **自依赖 0、依赖环 0、悬空引用 0**。
- proposed 18 / blocked 11 / ready 7，与 `normify status`（就绪 7 · 阻塞 11 · 待办 40 · 错误 0 · 警告 0）**逐项一致**。
- ready 7 与 `artifacts/ready-after.txt`、receipt `ready_after` 完全一致：`g10-p5`、`m03-p8`、`m10-p10`、`r06-p8`、`r07-p8`、`r10-p10`、`tie-order-r4`；其中 **6 张 checks = 0**（仅 `tie-order-r4` checks = 5）。
- **ready 列表含占位卡 = normify「只看依赖、不读 note」的已知缺口**：README §4 与 receipt `ready_note` 已如实登记，并说明消除它需改 status（**超出「不改 status」边界，未做**）。**记录为已知项，不阻断、不改 status**。

### 2.6 占位 note 与保持 proposed —— 通过
- 全库扫描：统一阶段占位措辞 **45** 张、deferred/backlog 变体 **8** 张，与 `artifacts/placeholder-notes.txt` 两段清单**逐 id 完全一致（差集为空）**。
- 两类卡 `checks > 0` 均为 **0**（确实零机器检查）；开放零检查记录 **53** 张，与「53 张加 note」一致。
- `g10-p5` / `r07-p8` / `m03-p8` / `m10-p10` 均**仍为 proposed**（checks = 0）——未被强行 verified。

### 2.7 证据 / 门禁 / 卫生 —— 通过
- 被复核 manifest **7/7 独立重算逐字节一致**（README `d2fc3c29…`/12,111 B、receipt `1b24b7ae…`/3,859 B + 5 artifacts）。
- 8 个证据文件 BOM = False、CR = 0、TAB = 0（`dependency-diff.tsv` 的 96 个 TAB 是 TSV 分隔符，属设计）。
- README §3 的 **23 行「修后仍阻塞」列与实测「当前非 verified 依赖集合」逐行一致（0 mismatch）**。
- 四门禁在 HEAD 独立复跑**全部 exit 0**，unit **1387 passed / 0 failed / 402 ignored**（与 `.normify/runs/2026-11-11-normify-dependency-graph-repair/` 记录一致）；`normify check` 本卡 exit 0；`normify validate` **错误 0**（5 条架构级 warning 为预存在，与本卡无关）。

## 3. 观察项（4 条，均信息级、不阻断）

1. **O1（信息）** README §2.7 写「被改依赖的 **17** 个任务在 note 追加一行指向本目录」，实测拿到该指向行为 **23** 个任务（commit message 亦写 23）；17 是死依赖任务数，23 是被改依赖任务数，属 README 措辞与实际的口径差。
2. **O2（信息）** README §2.2/§2.3 写「drop 该依赖 + **note 记依据**」，实际 `p10`/`p11`/`next-t4-numeric` 的 note 只有通用一行（指向本证据目录），具体 waive/T4 依据落在 `artifacts/dependency-diff.tsv` 的 basis 列；**可追溯链完整**，仅 README 措辞偏强。
3. **O3（信息）** 53 条记录为手工编辑，未刷新 `updated_at`（`normify validate` 未报警、不影响 status/检查，仅时间戳语义）。
4. **O4（信息）** `normify status` 的「冲突 52」不在本卡范围，未核查其前后值；但 `files`/`modules`/`scope` 未改、开放集合未变，按实现口径不应变化。

## 4. 非声明

本复核只针对**排程图治理与边界**：不构成任何阶段验收，不改变判据/参考/阈值，不宣称任何功能完成，也不代表 W0/G-L1/G-L2 窗口状态或可开窗。
