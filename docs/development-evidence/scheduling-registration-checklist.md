# 排程登记检查清单（Normify 片登记规范）

> 来源：2026-10-26 M0a 形状修正片（`…-fix-mesh-poly-shapes`）登记时 `files` 漏写证据目录，导致任务 verified 后提交被判越界，只能另开「证据归档片」补救。本清单把该教训写死，登记任何片前逐条自查。

## 1. 登记前（新片 JSON）

`schedule.files` **必须**覆盖该片的全部写域，逐项确认：

1. **源码目录**：会改的 `src/` 子树（写目录而不是单个文件；例如 `src/physics/collision/cuda/`、`src/runtime/collision_gpu/`）。
2. **测试目录**：`tests/`（含 `tests/support/`，`#[path]` 引入的 support 文件也在 `tests/` 下）。
3. **fixtures**：会新增/修改时写具体新目录（如 `fixtures/g10-p5-mesh-multicontact/`）；**只读引用不必写**。
4. **该片证据目录**：`docs/development-evidence/<片名>/`；若 checks 里有 `json` 指向该目录文件，**必须显式写进 files**（否则提交越界，正是本次教训）。
5. **治理回执目录**（若与证据目录分开）：`docs/development-evidence/<片名>-governance/` 等。

**自查口径**：把「本片预计会 new/modify 的文件」逐条映射到 files 的一项前缀上，任何一条映射不到 → 补 files 再提交登记。

## 2. 登记后约束

- `normify start` 之后**不能**再改任务的 acceptance/checks/files（`fix` 只追加修正记录，不改声明；`waive` 只对候选/待办任务有效）。
- `verified` 任务视为终态，不能 abandon、不能改冻结字段。

## 3. 漏登记的补救路径（按代价从低到高）

1. **证据单独归档片**：证据内容零改写，只需一个只含证据目录的治理片把目录提交（本仓库已用一次，`…-fix-evidence-archive`）。
2. **abandon + 重登记 r2**：仅适用于**任务尚未 verified** 时；重登记时把 files 补全，工作树内容不动，重跑 gate/done。
3. 不推荐手改 closed 任务字段 —— 会被 hook 拒绝（`关闭时不能改写已冻结字段` / `已关闭，修正须用 Normify-Fixes`）。

## 4. 本次自查结论（2026-10-26）

| 片 | files 是否覆盖写域 | 处置 |
| --- | --- | --- |
| M0b `…-m0b-case-production` | 是（证据 scripts/ + 证据目录 + `tests/` + `tests/support/`；不写 fixtures/src） | 不动（已 start） |
| M2 `…-m2-convex-vs-mesh` | 原缺 `src/runtime/collision_gpu/`（段数/顺序断言与 test 模块）与 `tests/support/` | **已补**（`9c06b9fd`） |
| M3 `…-m3-mesh-mesh` | 同上 | **已补**（`9c06b9fd`） |
| M1 捕获片 / M0a 修正片 / 各 QA 片 | 已按实际写域登记；M0a 修正片的证据目录漏登记已由归档片补救 | 记录留痕 |

## 5. 边界

- 本清单只管「登记与范围」，不改任何判据、容差、验收口径。
- 仍遵守既有纪律：每片 QA 预登记、窗口先报基线+签名 sha、attempt 留档、作废轮不拼。
