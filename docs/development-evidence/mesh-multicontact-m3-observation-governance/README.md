# M3 QA 观察 O1–O7 收口（证据文字/编码治理）

**任务**：`2026-10-27-mesh-multicontact-m3-closure-and-mesh-summary`（owner Arch）。
**触发**：M3 独立复核 `ab586682`（pass / 阻断 0 / 观察 7：3 一般 + 4 建议）。
**基准提交**：M3 实现 `f1e5cda1`。
**性质**：只改 `docs/development-evidence/mesh-multicontact-m3/` 的文字与编码；不改 `src/`、`tests/`、
`fixtures/`、判据、容差、参考与任何已 verified 数字。

## 逐条处置

| 观察 | 级别 | 处置 |
| --- | --- | --- |
| O1 拆分「纯搬移」表述不准 | 一般 | §13 改「测试函数体与 2 个 helper 体逐字一致；`CubeMeshPoly` 因 O7 新增 `vertadr`/`vertnum` 并扩展 `over()`，不是逐字节搬移」 |
| O2 stage2 record-C 数字错位 | 一般 | §9.1 改为「17 个 dump / 16 条 case-kernel 比较行 / 11 条有失败（`sdf-box failures=112` 条明细）」 |
| O3 capture 窗 delta 第三口径错 | 一般 | §4.1 注 2 改「vs 采样最小 `used=2012`（seq 81）→ **266**」；原写 2032→246 是把**首样本**当最小值；前两口径 238/234 不变 |
| O4 日志带 BOM + samples.csv 超 500 行 | 建议 | 17 个日志转 UTF-8 无 BOM（内容逐字不变）；`validation-attempt1-samples.csv` 734 行注明为原始窗口采样数据、属显式例外，权威原件在 `target/m3-mesh-mesh/attempt1/samples.csv` |
| O5 `[Insufficient Permissions]` 身份不可见 | 建议 | §9.3 补充：post/baseline 各 5 条系统进程，按关键词过滤 0 条本任务进程，身份不可见不影响「无残留」结论 |
| O6 拆前数字不可独立复算 | 建议 | §13 注明拆前 `990/0/374` 仅有 addenda 文字、拆前 WIP 未入 git、无独立 gate 日志；后续同类拆分须先记拆前树 sha/日志 |
| O7 util 80% 的 JIT 归因是推断 | 建议 | §4.1 注 1 标注「归因是推断」（RUN A ccd kernel 编译 ~10.35 s 佐证） |

## 未改动

- 原始窗口记录 `target/m3-mesh-mesh/{capture-attempt1,attempt1}/*` 不改写；`facts.txt` 的
  `guard_aggregate=…OrderedDictionary` 无效行保留原样（权威数字取 `guard.log`+`samples.csv`）。
- 数字结论全部不变：0 行设备改动、9 实体 exit 0、mesh-mesh `bit_equal 4/4`、guard 无 abort
  （delta 278/283/283、free_min 13557、util 60、residual 0）、拆后 gate 990/0/374、scope 75/0。
- QA 证据目录（`mesh-multicontact-m3-independent-check/`）不动：它是复核时点的忠实副本。

## 边界

不宣称 mesh 全类型/全分支、不宣称 G10@P5 整体通过、不宣称跨平台或精度目标达成；UNC-1 仍为 partial。
