# NP01 host 门禁硬化（QA 观察 1/2/3）

任务：`2026-11-04-precision-np01-host-gate-hardening`（依赖 QA 复核卡 `2026-11-04-precision-np01-b2d-b2e-b3-check`，复核结论 PASS-with-observations：阻断 0 / 需修 0 / 观察 5，提交 `c7841522`）。

本片只修 QA 的**前三条观察**，不新增判据、不放宽容差、不动 `src/`：

## 一、观察 1：门禁对「重复表 / 文件名与 row_id 不一致」不设防

QA 实跑变异实验发现三种坏输入仍 PASS：重复同 `row_id` 的副本、文件名与 `row_id` 互换、字段表重复副本；而 README §十 写的是「同名 `row_id` 表」，比实现严。

**修法**（`tests/precision_type_gate.rs`）：

1. 公共形状断言 `assert_table_shape` 增加 `file_stem == row_id`（两套表同时生效，`name.strip_suffix(".json")` 与 `row_id` 逐字比较）；
2. 产品覆盖测试：`covered` 去重后 `len == tables.len()`（捕捉重复 `row_id`）；
3. host 覆盖测试：`covered.len() == host_sites.len()`（表数必须等于 `hostCoverage` 站点数，重复或多表即失败）。

说明：`file_stem == row_id` 是主防线；两条计数断言是第二道网（如大小写不敏感文件系统下的同名近似、符号链接副本），在正常目录结构下通常先被主防线拦下——本片如实记录这点，不把计数断言写成独立验证过的防线。

### 变异自证（实跑，恢复后 5/5 绿）

| 变异 | 预期 | 实测 |
| --- | --- | --- |
| `host-tables/gjk_host_copy.json`（复制 `gjk_host.json`） | FAIL | FAIL `gjk_host_copy.json 文件名≠row_id`（4 passed / 1 failed） |
| `field-tables/tendon_wrap_dup.json`（复制 `tendon_wrap.json`） | FAIL | FAIL `tendon_wrap_dup.json 文件名≠row_id`（4 passed / 1 failed） |
| 删除两个临时文件（恢复原状） | PASS | PASS 5/5 |

命令：`cargo test --locked --test precision_type_gate`。

## 二、观察 2：B2d/B2e 卡 note 缺「NP01 仍未验收」句

已用 `normify fix <task> --reason …` 给两张卡追加 addenda（不手改已关闭卡字段），文案与 README/B3 卡一致：
「NP01 仍未验收——E 指标（用户 2026-10-27 裁定的 g10-p5 native 接触字段容差）未定稿、host/device 逐表达式位级对照缺；本批只完成逐字段表，不宣称通过。」

## 三、观察 3：`gjk_host` 表未注明 `linear_combine` 位置

`host-tables/gjk_host.json` 的 `subdistance` 段写为「…（simplex.rs；`linear_combine` 定义在 `simplex.rs:298`，由 `gjk` 内联调用，无独立段）」，与 `gjk` 段 `accumulations` 里引用的 `linear_combine` 一致。

## 四、未做的两条（记录性观察）

- 观察 4（四张卡 addenda 实为 3 张 verified 卡 + 复核卡 proposed 阶段字段订正）：QA 表述订正，无需改文件。
- 观察 5（三片卡 `checks` 只有 `unchanged`、无 `.normify/runs` gate 记录）：门禁靠独立复跑（`cargo test --locked --test precision_type_gate` / `cargo test --locked`），本条维持现状并在此留档。

## 五、门禁

- `cargo fmt --check`：通过（本片改动只涉及测试与证据）。
- `cargo test --locked --test precision_type_gate`：**5/5**。
- `cargo test --locked`：unit 计数只增不减（本片不改产品代码）。
- 边界：`src/`、`include/warp-headers/`、fixtures、判据/容差零改动；零 GPU；**不宣称 NP01 通过**。
