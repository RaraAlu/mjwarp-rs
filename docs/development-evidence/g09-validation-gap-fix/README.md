# G09 复核修正：退化尺寸校验落空与回执 sha256 登记

任务：`2026-10-23-g09-validation-gap-fix`。日期：2026-10-23。执行：Backend。
性质：批次 4 复核发现两项「一般」问题的小修（窄相校验 + 测试 + 证据/回执），不改其他产品路径。

## 1. 复核发现与修正

| # | 发现（批次 4 复核） | 修正 |
| --- | --- | --- |
| 1 | acceptance 要求「退化输入（尺寸非正/非有限）显式拒绝」，但 `narrowphase::validate` 只校验数组长度，`plane_box`/`primitive_narrowphase` 对 `geom_size` 无符号/有限性校验，也无拒绝测试 | 新增 `validate_geom_sizes`：plane x/y 半径 >0 且三轴有限、z 分量不得为负；sphere 半径 >0 且三轴有限；box 三轴半尺寸 >0 且有限；新增 `NarrowphaseError::InvalidGeomSize { geom, axis }`；补拒绝测试覆盖 0/负/NaN/±Inf 与合法 fixture 通过 |
| 2 | g09 回执 `results.references.model_sha256` 误填 `model.xml` 的 git blob hash `63743a8a…`（实际文件 sha256 为 `a78b1547…`） | 回执改为文件 sha256 `a78b15472ffe8d0e3a8f6e61b8c4512dfc94c889ceca8ea344b76dab3cdf1586`；files 中本次改动文件 blob 同步刷新 |
| 3 | QA 建议：`primitive_narrowphase` 注释「两类基础几何对」、证据 README 未直列 model sha256、「去重后最多 4 接触」措辞与上游实际不符、缺 box-box 拒绝用例 | 注释改「三类基础几何对」；证据 README 直列 model sha256；多接触措辞改为「上游无去重步骤，8 候选全部参与，数量由 margin/gap 过滤决定（面/棱/角 4/2/1）」；补 box-box 候选拒绝测试并说明 fixture 候选只含 plane-box |

修复不改变任何物理数字：15 接触、逐世界 4/2/1/0/4/4/0、上游 5.960e-08、原生 1.192e-07 保持成立。

## 2. 实现与测试

| 文件 | 变更 |
| --- | --- |
| `src/physics/collision/narrowphase.rs` | `InvalidGeomSize` 错误 + `primitive_narrowphase` 注释改为三类几何对；`mod validate` 挂接 |`n| `src/physics/collision/narrowphase/validate.rs` | `validate`/`validate_geom_sizes`（窄相入口调用） |
| `src/physics/collision/narrowphase/tests/plane_box.rs` | 重构 fixture/状态/候选辅助；新增 `rejects_degenerate_geom_sizes`、`rejects_box_box_candidates` |
| g09 证据 README / 回执 / 批次 4 索引 | sha256 修正、校验与措辞修正登记、复核修正说明 |

新增拒绝测试覆盖：box 三轴半尺寸 0/负/NaN/±Inf；sphere 半径 0/负/NaN/Inf；
plane x/y 非正、z 负、NaN/Inf；合法 fixture 仍通过；box-box 候选返回 `UnsupportedPair`。

## 3. 验证（本次实测）

- `cargo test --locked --lib plane_box -- --nocapture`：**3 过 0 败**
  （原对照 1 + 退化尺寸拒绝 1 + box-box 拒绝 1）。
- `cargo test --locked --lib`：**360 过 0 败**。
- `cargo fmt --check` 与 `cargo clippy --all-targets --locked -- -D warnings`：通过。
- 后续 `normify fix 2026-10-11-roadmap-g09-p5` 追加修正记录；g09-p5 保持 verified。

## 4. 边界

- 只改 `src/physics/collision/` 窄相校验与测试、g09 证据/回执与批次 4 索引；
  不改其他产品路径、既有容差/断言/fixtures/authoring 脚本，不碰 QA 文件。
- 未新增几何对支持；box-box 等未支持对仍显式拒绝。

## 5. 复现

```powershell
cargo test --locked --lib plane_box -- --nocapture
cargo test --locked --lib
```
