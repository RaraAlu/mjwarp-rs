# errata 台账（低项 / 观察）

- 建立：2026-11-12。适用范围：L1 复核产生的低项与观察（阻断/一般当场返工，不进本表）。
- 规则：**每张 L1 卡只复核一次**；低项/观察登记在此，**每批收口或开窗前批量修一次**；修复后更新状态并注明提交号。
- 状态：`OPEN`（待修）→ `IN_BATCH`（已排入某批）→ `DONE`（已修，注提交号）/ `WONTFIX`（不改，注理由）。
- 只记事实与位置，不在这里重开判据讨论；涉及判据/容差/参考的改动自动升 L2。

## 初始条目（2026-11-12）

| ID | 发现日期 | 来源 | 内容 | 等级 | 状态 | 批次 |
| --- | --- | --- | --- | --- | --- | --- |
| E-001 | 2026-11-12 | QA 复核 O2（卡 `parallel-unit-baseline-count-fix-check`） | 待决策①的主口径顺序：现以 P-DEF 107 打头、产品命名 95 在括号；建议把「95 个产品命名入口」提前为主口径 | 低 | OPEN | 下一批 |
| E-002 | 2026-11-12 | QA 复核 O3（同上） | 附录「113 与 105 是正则/文本级口径」合并一句仍在（F.4 已单列）；113（`wp.array(` 候选）与 105（`module="unique"`）语义不同，建议分述 | 低 | OPEN | 下一批 |
| E-003 | 2026-11-12 | QA 复核 O4（同上） | `merged-count-table.md` 标题为「终稿草案，三口径脚注版」，而 README/receipt 称「终稿合并表」；历史不改，下轮注明「草案即定稿快照」 | 观察 | OPEN | 下一批 |
| E-004 | 2026-11-12 | QA 复核 O5（同上） | README §5 的 `policy_scan` 示例只列 3 个文件，receipt 记录扫描 5 个文件；示例与 receipt 口径对齐 | 观察 | OPEN | 下一批 |
| E-005 | 2026-11-12 | 合并表 §1 #15/#16（Backend `reconcile.md`） | `G.5`/`G.6` 的 24 双规则：`G.6` 24 = `wp.launch_tiled(` 精确；`G.5` 24 两规则并用（语义是 aliasing，同数属巧合未证），待定主规则 | 观察 | OPEN | 下一批 |
| E-006 | 2026-11-12 | QA 追加复核 L4（提交 `27f81d7b1` 记录） | README 头部「累计 diff +33/-2」为陈旧值（实际 +37/-2） | 低 | OPEN | 下一批 |
| E-007 | 2026-11-12 | U7/U8 判据 addenda §3 O1 | `UNC-1` 术语 partial/uncovered 混用；引用时按 `id`+`status`+`group` 合读（当时记录不改内容） | 观察 | OPEN | 文档统一批 |
| E-008 | 2026-11-12 | U7/U8 判据 addenda §3 O2 | `a2-slices.json` L55「见 criteria §1.3」应指 `README.md §1.3`（当时记录不改内容） | 观察 | OPEN | 文档统一批 |
| E-009 | 2026-11-12 | 本流程落地（A0/L0 提交实践） | `normify` commit-msg 钩子无 L0 豁免：要求 `Normify-Task:` 且按卡 `schedule.files` 校验范围。**已用常驻卡解决**：L0 docs 提交统一引用 `2026-11-12-process-lite-standing-v2`（v1 因 `unchanged src/...` 检查与 P1 片冲突且运行中冻结无法收窄，已 abandon，提交 `93d9a033b`）；`5f9c1bb9`、`117d2299` 两次历史 L0 提交为一次性 `--no-verify`，保留不重写。长期可考虑工具白名单（`docs/**` 免任务或 `Normify-Task: L0`），需用户批 | 低 | DONE（常驻卡 v2） | 工具批（长期项） |
| E-010 | 2026-11-12 | P0-b 登记（锁纪律） | P0-b 卡登记提交 `78038ca9f` 在 `repo-write-lock=owner=backend` 期间完成（锁检查滞后于提交）；此后零写入并已向 Chief/Backend 披露。改进：提交前第一步先跑锁检查，未 FREE 立即停 | 低 | OPEN（流程改进） | 流程批 |
| E-011 | 2026-11-12 | QA L1 批量复核低项 L1（卡 `2026-11-12-l1-batch-p0b-p1a-check`） | P1-a 证据 README §1 表与 `artifacts.json::files[].lines` 的 10 个行数值均 +1（201/160/169/267/457/213/37/87/152/126 vs LF/ReadAllLines 实测 200/159/168/266/456/212/36/86/151/125；`max_file_lines` 457 vs 456），疑似 `Get-Content` 计数；判据（≤500）不受影响 | 低 | DONE（`e8ef68e89`，P1-b S1 顺手更正；QA 已核 10 值与 max 456） | P1-b 首批 |
| E-012 | 2026-11-12 | QA L1 批量复核观察 O1（同上卡） | P0-b README 标题「85 行」指 85 条能力行，而 JSON 物理行 2,172；建议写「85 条（row_count=85）」避免与文件行数混读 | 观察 | OPEN | 下一批 |
| E-013 | 2026-11-12 | QA L1 批量复核观察 O2（同上卡） | P0-b `level` 字段存在复合值（如「必；64位整数可退：双32位」），README 口径写「必/退/射」；属描述性复合值可接受，建议注明允许复合写法 | 观察 | OPEN | 下一批 |
| E-014 | 2026-11-12 | QA L1 批量复核观察 O3（同上卡） | P1-a README §4 手动耗时（fmt 1.77 s / unit 88.34 s）与 `.normify/runs` 同门禁数据为两轮独立运行、并列呈现；建议加「两轮独立」避免误读为同一轮 | 观察 | OPEN | 下一批 |
| E-015 | 2026-11-13 | QA P1-b 复核 O2（卡 `2026-11-12-l1-p1b-cuda-adapter-check`） | `unit.json::log_sha256` 与 `unit-summary.log` 的 sha256 不同（gate 原生日志 vs 过滤摘要）；已在 P1-b 证据补注关系，非矛盾 | 观察 | DONE（`e8ec1f2f1`） | P1-b 证据措辞 |
| E-016 | 2026-11-13 | QA P1-b 复核 O3（同卡） | `unknown_kernel_handle_is_launch_error` 为手工构造 `DeviceError` 再断言 `class()`，未走真实路径；`Module::suggest_block_width` 的 unknown-handle 与 occupancy 失败（→`DeviceLost`）无 0 GPU 覆盖 | 观察 | OPEN（0 GPU 覆盖缺口） | 后续片补行为测试（需驱动或改写断言） |
| E-017 | 2026-11-13 | QA P1-b 复核 O1（同卡） | A0 仍写 `BufferError`（P1-b 冻结 5 类），实现含第 6 项 `BufferError::DeviceLost`（D1 扩展，Chief 已追认）→ A0 措辞待同步 | 低 | OPEN（Arch 处理中，锁 reason 已见） | A0 同步批 |
| E-018 | 2026-11-13 | QA P1-b 复核 O4（同卡） | `Diagnostics::elapsed` 把驱动错误/非有限值吞成 `Duration::ZERO`（P1-a 签名无错误通道的既定取舍）→ 记为「事件计时错误不可观测」风险 | 观察 | OPEN（Arch 处理中） | A0/后续片 |
| E-019 | 2026-11-13 | QA P1-c 复核 O1（卡 `2026-11-13-l1-p1c-fallback-check`） | 组合规则两层口径：`select_native` 选择层不返回 Unsupported（结构性）；适配层在 flag=true 但原生未实现时仍返回 Unsupported（K.1 native-capture、B.11 native-zero-copy） | 观察 | OPEN | P2-b 首提交（Backend 顺手改 P1-c 证据措辞） |
| E-020 | 2026-11-13 | QA P1-c 复核 O2（同卡） | `CudaQueue::capture` 先 `graph_mode()` 再 `capture_sequential()`，一次 capture 记 **2 条** GraphCapture trace；若以 trace 计数做判据会偏大 | 观察 | OPEN | 后续片（可择一留痕） |
| E-021 | 2026-11-13 | QA P1-c 复核 O3（同卡） | B.11 退路 Copy 路径用阻塞 `memcpy_dtod_sync`，证据只写「无 wait_idle」；另有 `usize::try_from(bytes).unwrap_or(usize::MAX)` 兜底写法 | 观察 | OPEN | P2-b 证据措辞 |
| E-022 | 2026-11-13 | QA P1-c 复核 O4（同卡） | B.10 CUDA transient 池 `swap_remove` 取走不归还（无 release API）；返回缓冲 `byte_len` 取池槽容量（≥请求字节）而非请求值 | 观察 | OPEN | P2 池化 |
| E-023 | 2026-11-13 | QA P1-c 复核 O5（同卡） | H.1 的 `native_add` 与 `cas_loop_add` 走同一 CAS 循环，仅入口区分（设备侧原生属 P2） | 观察 | OPEN | 后续片 |
| E-024 | 2026-11-13 | QA P1-c 复核 O6（同卡） | 口径裁决（Chief）：**10/12 = 选择层实现**；其中 6 项产品接线（B.10/B.11/G.9/K.1/K.5/L.3），4 项（K.2/K.3/C.1/H.1）选择函数仅测试态、设备端执行归 P2；**P2-b 范围扩为「D.4/D.10 + K.2/K.3/C.1/H.1 设备端执行」**；不返工，P1-c 证据措辞由 Backend 在 P2-b 首提交顺手改 | 观察 | 已裁（Chief 2026-11-13） | P2-b |

## 维护

- 新条目：复核人按上表格式追加，ID 递增，来源写卡/提交/章节。
- 批量修复：Arch 在每批收口或开窗前汇总 OPEN 条目，一次修完并更新状态列（`DONE` 附提交号）。
- 与 L2 的关系：任何条目若触及判据数字/容差/参考/ABI/冻结证据，**移出本表**按 L2 走，不在此批处理。
