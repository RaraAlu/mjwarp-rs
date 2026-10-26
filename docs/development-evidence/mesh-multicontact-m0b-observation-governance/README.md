# M0b QA 观察 O1–O6 收口（纯证据文字/编码治理）

**任务**：`2026-10-27-mesh-multicontact-m0b-observation-closure`（owner Arch）。
**触发**：M0b 独立复核 `f191fdba`（pass / 阻断 0 / 观察 6：2 一般 + 4 建议）。
**基准提交**：M0b 交付 `ac01d86d`（父 `d9c31703`）。
**性质**：只改 `docs/development-evidence/mesh-multicontact-m0b/` 下的**文字与编码**，不改 32 个 bin、
`mesh-cases-manifest.json`、`scripts/prepare_mesh_cases.py`、`tests/`、`src/`，不改任何数字/判据/参考。

## 1. 逐条处置

| 观察 | 级别 | 处置 | 落点 |
| --- | --- | --- | --- |
| O1「旧 12 个 G5 case」与事实不符（实际 10） | 一般 | 改 10；注明交付 commit message 中「12」为笔误、历史提交不改写、口径以 10 为准 | M0b README §1、§4 |
| O2 README §5 行数与实测不符 | 一般 | 改为交付提交 `ac01d86d` 时点 raw/非空行数：`primitive_qa_support` 331/312、`multicontact_qa_support` 347/326、`mesh_poly_case_support` 362/338、`g10_p5_mesh_poly_case_assembly` 385/361、`prepare_mesh_cases.py` 372/331；并注明后续片只增不减、仍 ≤500 | M0b README §5 |
| O3 manifest `cross_check` 只记 `match=true` | 建议 | **裁定不补**：补 actual/reference 位型会改 `mesh-cases-manifest.json` 的 sha → 级联已 verified 的 M0b 复核证据与生产脚本 provenance；改为在证据层写明自证力弱 + 三条冗余证据 | M0b README §2 |
| O4 交付证据 2 个日志带 BOM | 建议 | `logs/assembly-test.log` 925→922 B、`logs/real-validate-probe.log` 1638→1635 B，转 UTF-8 无 BOM，内容逐字不变 | `mesh-multicontact-m0b/logs/` |
| O5 探针日志无 exit code、探针源码未留档、不可复跑 | 建议 | 显式注明：探针仅历史留档；**常驻覆盖 = M2 的 `validate_collision_inputs`（`src/runtime/collision_gpu.rs`，`#[doc(hidden)]`）+ `real_cpu_validation_accepts_m0b_and_old_cases`（非 ignored，读真实 M0b bins）** | M0b README §8 |
| O6 临时证据建议附探针源码/sha | 建议 | 探针源码跑完即 revert、未归档，**无法补**；仅日志可佐证真实编译执行（`validate_tests.rs:456 unused mut` + 3 行 `PROBE_REAL_VALIDATE_OK` + `test result: ok`），如实记为不可复跑 | M0b README §8 |

## 2. 为什么 O3 选择「不改 manifest」

`mesh-cases-manifest.json` 的 sha `b7c0ed16…` 被 M0b 交付证据与 QA 复核证据（`f191fdba`）共同引用；补字段会
同时改 manifest sha 与生产脚本重跑口径，而该结论本身已有三条独立冗余证据（逐文件 bin sha256/count 双处复算、
常驻测试逐位断言、QA 现场 pinned mjModel 复算）。在「不改已 verified 生产物」与「自证力」之间，选前者。

## 3. 未改动清单

- 32 个 bin（2 case × 32）与 `meta.txt`；`mesh-cases-manifest.json`；`scripts/prepare_mesh_cases.py`、`scripts/sha256.json`；
- `tests/`、`src/`、`fixtures/`、`Cargo.*`；
- M0b 交付 `receipt.json`（观察项不涉及 receipt 内容）；
- **QA 证据目录不改**：`mesh-multicontact-m0b-independent-check/manifest.json` 只覆盖该目录自身 17 个文件，
  未包含交付 README/receipt/logs，故本片编辑不破坏其 sha 校验；其 `logs/` 下的 BOM 副本是复核时点忠实副本，保留。
- 任何已 verified 数字不变：CPU count 4/4、9 数组长度 `(1,1,18,6,6,24,8,8,24)`、`sum` 恒等式、交付时点门禁 986/0/372。

## 4. 边界

不宣称 mesh 已支持、不宣称 M0b 有任何 GPU 证据；M0b 仍是**纯 CPU** 的 32 数组 bins 生产片。
