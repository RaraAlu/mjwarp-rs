# UNC-3 台账更新（B3 分支片收口）+ B3 证据 QA 观察勘误

**任务**：`2026-10-26-g10-p5-unc3-ledger-branch-closure`（owner Arch）。
**基准**：B3 实现 `e221e6c2`（verified）、QA 复核 `856b7835`（pass / 阻断 0 / 观察 6）。**纯文档与证据文字治理，不改 `src/`、`tests/`、`fixtures/`、判据、容差、期望或任何已验证数字。**

## 1. UNC-3 更新（前 → 后）

| 项 | 本次更新前 | 本次更新后 |
| --- | --- | --- |
| 组合层 | 13/13 | 13/13（不变） |
| 分支层（已覆盖） | B1/B2 覆盖的 4 类 | **+ B3：graph hillclimb、`dist==0` fallback、投影 clamp、cap、corner、non-parallel det，以及 swap 几何顺序** |
| 分支层（未覆盖） | 8 类 | **3 类**：`plane_cylinder` degenerate x 轴回退、`plane_ellipsoid` 非上游退化保护、`plane-box` 其余 5 角点（含 plane-capsule/cylinder 其余候选未 detected） |
| 债务 | sphere-box/capsule-box/box-box 分支级未标注 | 同上 **+ `capsule_capsule` 平行轴分支**在融合后端（det≈2.6e-12 → 非平行）未被 device 触发 |
| status | partial | **partial（不变）** |

**改动文件**
- `docs/development-evidence/g10-p5-acceptance-map/README.md`：§6 分支层清单改写（已覆盖 6 项 + swap；剩 3 类；债务补 capsule 平行轴）；capsule-capsule 表行措辞按融合后端实际分支修正；证据指针补 B3 与 QA。
- `docs/development-evidence/g10-p5-acceptance-map/map.json`：`uncovered[UNC-3]` 的 `measured`/`evidence_paths`/`reason`/`impact` 与 `summary.note` 同步；status 仍 `partial`，其余 requirement 未动。

## 2. QA B3 观察收口（6 条，纯文字/边界）

| 观察 | 级别 | 处理 |
| --- | --- | --- |
| O1 | 一般 | B3 README 的「13 组合位级无回退」改为**「判据内无回退」**并注明 B1/B2 只输出 `failures=0`、PTX 已变（3786439→3786510），不能先验排除 ULP 级差异 |
| O2 | 建议 | 边界说明：CPU 自检不经过 `run_case` geom 解析路径，`device_touched=false` 恒真，只能兜住参考/形状类缺陷 |
| O3 | 一般 | 执行脚本已入库为**逐字节相同**的证据副本（`9cff9b31…` / `6b0eba49…`，见 B3 `scripts/sha256.json`）；工作树中未跟踪的执行副本已清理 |
| O4 | 建议 | 去除 B3 README 的 2 个 BEL 控制字符；重复的 `## 9` 标题改 `## 11` |
| O5 | 建议 | 标注 precheck `axis_dot=0.0` 为理想化口径；按参考 `geom_xmat` f32 实算 ≈-7.57e-9、det ≈2.073600335e-4，分支判定不受影响 |
| O6 | 建议 | 说明 attempt1 branches 二进制未归档；「修复只碰 tests/」由 panic 日志 + 提交内容 + dirty 计数支持，不作二进制级对照 |

B3 证据 `receipt.json` 增加 `post_qa_closure` 字段记录以上映射；B3 README 新增 §12 收口节。

## 3. 证据指针

- 实现：`docs/development-evidence/g10-p5-primitive-branch-coverage/`（§7.4 attempt2、§12 QA 收口、参考 `a8407a90…`）。
- 复核：`docs/development-evidence/g10-p5-primitive-branch-coverage-independent-check/`（pass）。
- 台账：`docs/development-evidence/g10-p5-acceptance-map/{README.md,map.json}` §6 / UNC-3。

## 4. 边界

- 只更新证据映射与文字；不改任何判据/容差/期望/参考；不改已 verified 数字与结论。
- UNC-3 仍 `partial`：不宣称全路径 / 全分支 / 跨平台；精度与等价性结论仅限已列 case 与已批准判据内。
