# UNC-1 台账第二次更新（M3 收口 + QA `ab586682` pass 后）

**任务**：`2026-10-27-mesh-multicontact-m3-closure-and-mesh-summary`（owner Arch）。
**触发**：M3 实现 `f1e5cda1`（QA `ab586682` pass / 0 阻断 / 观察 7）。
**性质**：只改台账文字（`g10-p5-acceptance-map/{README.md,map.json}` + 新整片汇总）；不改任何已 verified
数字、判据、容差、参考、期望值，也不改 `src/`、`tests/`、`fixtures/`。

## 1. 前 → 后

| 维度 | 第一次更新（M2 后） | 第二次更新（M3 后） |
| --- | --- | --- |
| 组合层 | box/cylinder + convex-vs-mesh | **+ mesh-mesh 多接触**（nacon=4 / bit_equal 4/4 / ULP 0/0/0 / `contact.geom=[0,1]`） |
| 设备侧改动 | M2 新增 mesh 分支（`DEVICE_SOURCES` 27→28） | M3 **0 行新设备代码**（28→28、入口 6→6、冒烟四数字与 M2 相同） |
| 设备输入 | 2 case × 32 bins | + mesh-mesh 32 数组 bins（9 slice `[2,2,36,12,12,48,16,16,48]`） |
| 输入校验 | `polyvert/polymap` 值域未校验 | **O7 已补 4 组值域规则**（负例/正例在 `validate_mesh_poly_tests.rs`，`run()` 零改动） |
| 未覆盖 | mesh-mesh、一般 mesh、hfield-capsule、全类型/全分支、跨平台 | 收窄：一般 mesh（nmesh>1 通用 / degree>10 / polygon>16）、mesh 分支层未逐条、hfield-capsule、efc_address、UNC-6 未触发、跨平台、性能 |
| status | partial | **partial（不变）**；统计口径不变（12 条：met 8 / partial 4 / unmet 0；uncovered 6） |

## 2. 改动文件

- `docs/development-evidence/g10-p5-acceptance-map/map.json`：`uncovered[UNC-1]` 的 title/implementation/measured/
  evidence_paths（10→11）/reason/impact + `summary.note`（两次更新说明）。
- `docs/development-evidence/g10-p5-acceptance-map/README.md`：新增 §7.2 第二次更新表与「不得声明（更新后）」。
- `docs/development-evidence/mesh-multicontact-mesh-summary/`：整片汇总（交付链、ABI/冒烟、判据与参考、
  覆盖结果、两窗口、未覆盖、债务、不宣称清单、证据指针）。
- 本目录 README + `receipt.json`（治理回执）。

## 3. 不得声明（引用时逐条遵守）

不宣称一般 mesh 已支持、mesh 分支层全覆盖、mesh 全类型、跨平台、性能；`hfield-capsule` 与 `efc_address`
仍为未覆盖；G10@P5 整体通过 / 精度目标达成不在本 map 的声明面；UNC-1 仍为 **partial**。
