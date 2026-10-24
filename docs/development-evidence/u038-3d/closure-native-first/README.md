# U038 口径校正：按 native_error_v2 原生放行（仅文档与排除理由文字）

- 日期：2026-10-24
- 任务：`2026-10-24-u038-native-first-closure`（owner QA，无 GPU）
- 用户决定：2026-10-24 选择 **A**——只改文档与排除理由文字，不动排除成员/探针/测试逻辑，也不重跑全量 G01。
- 依据：`959be10`（任务 `2026-10-24-u038-native-two-reference`，verified）的 `native_error_v2` 补判。

## 结论

U038-3D 该链判定口径由「已知债务（用户 2026-10-23 裁决 B）」校正为
**「按 `native_error_v2` 原生放行 + 上游逐位差异记录」**：

- **原生放行**：9/9 字段块 `native_passed`（xpos、xquat_wxyz、xipos、xmat、xanchor、xaxis、
  subtree_com、cdof、qfrc_gravcomp），覆盖 **1435 个比较标量**，`E_candidate=0.0034–0.0129`
  （全部 ≤1），`secondary_reference_used=false`，`np03_admitted=true`；放行范围**仅限该链
  已记录字段块**（`xaxis` 的 free/ball 42 项按合同排除，不逐元素择优）。
- **上游逐位差异记录**：上游严格 `2e-6` 口径仍为 **8/77 超差、max 4.291534423828125e-6、
  总位差 122**；原样记录，**不是本次准入依据**。
- **不宣称**：上游逐位对齐（bitwise）、完整 U038/三维链通过、完整产品精度或 NP01–NP04 全量验收。

## 改动（仅文字口径）

| 文件 | 改动 |
| --- | --- |
| `docs/three-layer-acceptance.md` | §5.3 改写为「U038 上游逐位差异（按 native_error_v2 原生放行）」，保留 2026-10-23 裁决 B 与 debt-2 历史段落；§4.14 收口叙述与未满足项、§7 G01 现状与 U038 条目、各节排除清单标签（`U038-3D known-debt 1` / `U038-3D 已知债务 1` → `U038 上游逐位差异 1（按 native_error_v2 原生放行）`）、§8 尾注任务列表同步 |
| `docs/compatibility-matrix.md` | 状态段新增 2026-10-24 口径校正段（保留 2026-10-23 历史叙述）；未满足项行、U038 小节同步；旧标签写明被 2026-10-24 补判取代 |
| `docs/development-evidence/u038-3d/known-deviation.md` | 顶部新增「最终口径」与「历史（2026-10-23 裁决 B 登记，保留）」分节；边界小节行文同步 |
| `scripts/test-windows-g01.ps1` | 只改 `known_debt` 注释块与 `known_debt_reason` 字符串；`$script:excludedKnownDebt` 成员、`--skip` 行为、探针断言、测试与排除数量一律未动 |

## 未变化（逐项核对）

- 排除清单仍 **11 项**：`excludedAllProbes` 3（W-D5）+ `excludedHistoricalAudits` 6（历史捕获身份）
  + `excludedEnvUnwired` 1（env/外部产物）+ `excludedKnownDebt` 1（U038）= 11；成员逐字未改，
  `--skip` 追加逻辑（4 处）未改。
- 未改产品代码/测试/断言/容差/fixtures/排除成员/ABI；未做 T4 repin。
- **未重跑全量 G01**：最近一次全量双模式仍为 HEAD `959be10` 上的 T2 收口证据（提交 `6494fe8`，
  debug/release 各 14/14 阶段 exit 0、host-msvc 840、all-probes 354/0、88 result 行、排除 11 项、
  rustdoc 0、diff 通过、墙钟 833.8s/649.9s）。本轮只改文字，不产生新的全量结论。
- 若最终收口需要把该口径绑定到更新的最终 HEAD，须由 Chief 另点名再跑一次 T2 全量；本轮不宣称。

## 核对/复现命令

```powershell
# 1) 脚本差异只应有注释与 known_debt_reason 字符串
git diff -- scripts/test-windows-g01.ps1

# 2) 排除名单仍 11 项（3 + 6 + 1 + 1）
Select-String -Path scripts/test-windows-g01.ps1 -Pattern 'script:excluded'

# 3) 门禁
cargo fmt --check

# 4) unchanged：src/tests/fixtures/Cargo.* 无改动
git status --porcelain -- src tests fixtures Cargo.toml Cargo.lock
```

## 证据来源

- 原生放行判定：`docs/development-evidence/u038-3d/native-two-reference/verdict.json`
  （`task_id=2026-10-24-u038-native-two-reference`、`policy=native_error_v2`、
  `summary.native_passed=9`、`summary.values=1435`、`upstream_strict_tolerance_record`）。
- 上游逐位差异：`docs/development-evidence/u038-3d/debt-2/run.json`（1477 项 / 122 位差 / 8 超差）。
- 本次口径校正：本目录 `README.md` + `receipt.json`。
