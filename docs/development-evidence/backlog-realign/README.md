# backlog 分诊后依赖重排（43 条 promoted）

任务：`2026-10-23-backlog-dependency-realign`。日期：2026-10-23。执行：QA。
范围：只改 `.normify/changes/*` 中 43 条 promoted（原 backlog d 类）任务的 `schedule.depends_on` 与 note；**不 start 任何任务、不改产品代码、不动 `.normify/runs/`**。
前置：`2026-10-23-backlog-triage-2`（提交 `95786c3`，43 条 promote 后全部阻塞）。

## 1. 结论

- **43 条全部完成依赖重排**：旧 P0–P11 聚合门禁（P2-CONTRACT、P3-BASE、P4/P5/P6/P7/P8-ENTRY 等）与已 abandon 的兄弟任务全部移除或替换为当前真实前置。
- 重排后：**12 条 ready**（依赖为空或全部 verified）、**31 条仍 blocked**（保留真实未完成前置，属批次排序）。
- `normify status`：就绪 20（原 8 + 新增 12）、阻塞 34（31 promoted + 3 既有）、待办 28、错误 0。
- **批次 1（8 条）**：**4 条可立即进入 start 准备**（g27-p2、m08-p3、g05-p4、g28-p7）；**4 条被同族前序切片阻塞**（g27-p7←g27-p2、m08-p7←m08-p3、g05-p7←g05-p4、g28-p8←g28-p7），前序验证后自动就绪。
- ⚠️ **启动前置缺口（重要）**：43 条 promoted 任务均**没有 checks**（引擎要求启动前至少一项机器可判定检查），且 `schedule.files` 为空、模块范围仍是旧计划宽范围。依赖就绪≠可直接 start；执行阶段必须先按规范补 checks、按当前布局收窄 files（Chief 派执行时一并处理）。

## 2. 重排规则（逐类）

| 原依赖类型 | 处理 | 例子 |
| --- | --- | --- |
| 旧 P0–P11 聚合门禁（keep/abandoned） | **删除**（聚合门禁不是可执行前置；对应能力已由 verified 族覆盖或应由具体族切片承担） | P2-CONTRACT、P3-BASE、P4/P5/P6/P7/P8-ENTRY、P9、P10 |
| 指向已 abandon 的兄弟任务 | **删除**（前置本身已被现有工作覆盖），note 记录依据 | g27-p2 原依赖 P2-CONTRACT（已 abandon/被 M1 契约覆盖）、m08-p3 原依赖 P3-BASE（已 abandon） |
| 同族阶段切片（P2/P3 → P7/P10） | **保留**（同一功能族的真实串行切片） | g27-p7←g27-p2、m08-p7←m08-p3、g05-p7←g05-p4、g28-p8←g28-p7、g25-p10←g25-p7、m10-p10←m10-p7、a10-p11←a10-p10、r10-p10←r10-p8 |
| 跨族真实依赖（未实现链） | **保留并具体化** | 约束 G12←碰撞 G07–G10；G13/G14/G15←G12（+G13）；G17←G15+G16；G11←G07–G10；G25←G07–G10+G12；G24←M08+G23；M10←G05+G23；渲染 R02–R09←R01；R10←R01–R09；M03@P8←R01 |
| 已 verified 能力（基础族） | **不写依赖**（verified 不阻塞；避免假依赖） | 运动学/动力学/M1–M7/A1–A9 等 |

每条的原依赖→新依赖与依据逐条记录在任务 JSON 的 `note`（追加 “2026-10-23 依赖重排…” 段）和 `realign.json`。

## 3. 重排后 ready 清单（12 条）

`a08-p10`、`a10-p10`、`g05-p4`、`g07-p5`、`g16-p6`、`g20-p7`、`g21-p7`、`g23-p7`、`g27-p2`、`g28-p7`、`m08-p3`、`r01-p8`。

## 4. 批次 1 八条状态

| 任务 | 依赖就绪 | 新依赖 | 说明 |
| --- | --- | --- | --- |
| g27-p2 | ✅ ready | （无） | 常量计算基础切片 |
| g27-p7 | ⏳ blocked | g27-p2 | 同族 P7 切片，g27-p2 验证后自动就绪 |
| m08-p3 | ✅ ready | （无） | 局部重置基础切片 |
| m08-p7 | ⏳ blocked | m08-p3 | 同族 P7 切片 |
| g05-p4 | ✅ ready | （无） | 执行器基础切片 |
| g05-p7 | ⏳ blocked | g05-p4 | 同族 P7 切片 |
| g28-p7 | ✅ ready | （无） | 查询基础切片 |
| g28-p8 | ⏳ blocked | g28-p7 | 同族 P8 切片 |

**结论**：批次 1 依赖层面 4 条可启动（还需补 checks/收窄 files，见 §1）；另 4 条在各自 P2/P7 前序完成后就绪，属预期批次内串行，不是假阻塞。

## 5. 仍 blocked 的 31 条与原因

- **约束链（7）**：G12–G18 中除 G16 外均等待碰撞（G07–G10）/约束前序；G16 已 ready。
- **碰撞链（5）**：G08–G11 等待 G07/前序碰撞切片。
- **复杂物理（6）**：G24 等 M08+G23；G25 等碰撞+约束；G20/G21/G23 已 ready。
- **渲染链（10）**：R02–R09 等 R01；R10 等 R01–R09。
- **工具（3）**：M10 等 G05+G23；A10@P11 等 A10@P10。
- **其它（0）**。
完整映射见 `realign.json.items`（原依赖/新依赖/依据/ready/阻塞项）。

## 6. 风险与边界

- 依赖重排是**排程修正**，不是实现完成：31 条 blocked 是真实未完成前置，未强行清空。
- 43 条 promoted 任务的 `checks` 为空、`files` 为空（旧计划未细化），**start 前必须补 checks 与范围**；本任务范围只含依赖字段，未代做。
- 未改产品源码/测试/容差/fixtures/脚本；未 start 任何任务；未动 `.normify/runs/`。
- 后续建议：批次 1 执行时先由执行人补 checks（gate fmt/unit、unchanged、json 回执、forbid_claim）与 files（当前布局），再 start；批次 2–5 的依赖已按真实链给出，可直接按批次派发。

## 7. 证据索引

- `realign.json`：43 条逐条 `old_deps → new_deps`、依据、ready/blocking、checks/files 数量与 start 前置说明；含批次 1 状态与 blocked 原因分组。
- 任务 JSON note：每条追加 “2026-10-23 依赖重排…” 依据段（任务记录内可审计）。
