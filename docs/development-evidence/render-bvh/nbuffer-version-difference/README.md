# nbuffer 版本差异的判据审计（B 类，参考-前提型；Chief 裁定、降级使用）

- 任务：`2026-11-04-precision-nbuffer-version-difference-audit`（QA；只读、零 GPU；不改判据/容差/参考、不改代码）
- 审计对象：
  - `fixtures/render-bvh/scene-flex/provenance.json` → `model_parity.known_difference`（`nbuffer`）
  - `docs/development-evidence/render-bvh/r06-s3b-flex-capture/parity/parity.json` → `counts` / `version_difference_classification`
- 口径依据：《机器判据裁定标准 v1》（A 本质-推导型 / B 参考-前提型 / C 完整性-哈希型）
- **裁定：B（参考-前提型），降级使用**。
  - `ruling_source`: **Chief**，2026-11-04 本会话裁决消息（QA 审计建议 → Chief 定；非 Arch、非用户签）。
  - **原话引用（Chief，2026-11-04）**：
    > 「`nbuffer` 按 B 类（参考-前提型）登记、降级使用——批准。」
    > 「你的三条前提必须写进证据并与结论同时声明：①渲染路径不读 `nbuffer`（0 命中）；②去 `<deformable>` 控制实验 32368==32368；③差值稳定恰 256。任一前提失效即升级阻断并重裁。」
    > 「**不得据此宣称 parity pass、不作发布依据**；`strict_elementwise_parity=fail` 保留。按 B 类走，不升用户签。」
  - Arch 2026-11-04 的建议是按 A 类保留性质处理（判据不动、进 `known_difference`、不进窗口判据）——**该表述属建议、不构成裁定**，本证据不作此归属，也不据此改动任何判据。

## 1. 事实（可复算）

| 项 | 值 |
| --- | --- |
| 差异字段 | `nbuffer`（MuJoCo 模型缓冲区容量类计数，非渲染数值数组） |
| `parity.json` 计数（312 / 315） | **38512 / 38256**，差 **256** |
| `provenance.known_difference`（scene-flex） | **37552 / 37296**，差 **256** |
| 差值稳定性 | 两个口径差均为 **256**（一致） |
| 控制实验（同场景去掉 `<deformable>`） | **32368 == 32368**（相等） |
| 差异归因 | MuJoCo 3.12/3.15 flex buffer capacity sizing（`attribution`） |
| 渲染路径引用检查 | `git -C target/mjwarp-parallel/upstream grep -n nbuffer -- 'mujoco_warp/_src/render*.py' 'mujoco_warp/_src/ray.py'` → **0 命中**；全 `_src` 仅 `types_test.py` 的字段顺序测试命中 |
| 严格的整体 parity | `strict_elementwise_parity = fail`（150 字段中 143 相等；7 条差异 = 6 条 f64 尾差但 f32 位级全等 + 1 条 `geom_sameframe` 语义差、渲染不读） |

## 2. 分类判定（A/B/C）

- **不是 A（本质-推导型）**：该差异不是从本质语义推导出的判据，也推不出「两版模型等价」；按 A 处理等于作出发布级等价声明（`parity.json.not_claimed` 明确禁止）。
- **不是 C（完整性-哈希型）**：`nbuffer` 不参与冻结清单/哈希完整性棘轮，不是完整性或哈希型判据。
- **是 B（参考-前提型）**：可接受性依赖外部前提（渲染路径不读该字段 + 控制实验隔离归因 + 差值幅度稳定）；前提必须与结论**同时声明**，**前提不成立即降级为阻断**。

## 3. 三条前提（与结论同时声明）

| # | 前提 | 证据 |
| --- | --- | --- |
| P1 | 渲染路径不读取 `nbuffer` | 上游 `_src` 的 render/ray 源码 grep **0 命中**；全 `_src` 命中仅为 `types_test.py` 字段顺序测试 |
| P2 | 差异可隔离到 flex/deformable 容量 | 同场景去掉 `<deformable>` 后 312/315 `nbuffer` **32368 == 32368** 相等 |
| P3 | 差值幅度稳定、可解释 | `parity.json` 38512/38256 与 scene-flex 37552/37296 的差**均为 256** |

## 4. 降级使用的具体含义

- 只登记为 `known_difference`（版本工具差异），**不进本窗口判据**；不作发布依据。
- **不得据此宣称 parity pass**、不得宣称「MuJoCo 3.12 与 3.15 等价」。
- `strict_elementwise_parity = fail` 保留（记录初始严格结论，不删除、不改写）。
- 判据/容差/参考值**零修改**；`src/`、`fixtures/` 零改动。

## 5. 失效条件（任一出现即升级阻断并重裁）

1. 渲染/射线路径开始读取 `nbuffer`（P1 失效）；
2. 控制实验（去 `<deformable>`）不再相等（P2 失效）；
3. 差值幅度不再稳定为 256（P3 失效）；
4. 上游版本或场景结构变化使该归因不再成立。

## 6. 边界

- 只读审计：未跑 GPU、未改 `src/`/`fixtures/`/判据/容差/参考。
- 本证据不含任何「parity pass / 版本等价」结论；`parity.json` 原始结论（含 `strict_elementwise_parity=fail`）不变。
- 本证据可被后续片引用；引用时必须连同 §3 三条前提与 §5 失效条件一起引用，不得只引「B 类」结论。
