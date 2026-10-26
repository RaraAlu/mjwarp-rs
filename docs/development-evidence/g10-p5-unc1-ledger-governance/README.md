# UNC-1 台账更新（mesh M0a/M1/M0b/M2 收口后第一次更新）

**任务**：`2026-10-27-g10-p5-unc1-mesh-ledger-update`（owner Arch）。
**触发**：M0b 独立复核 `f191fdba`（pass / 阻断 0 / 观察 6）与 M2 独立复核 `271fe926`（pass / 阻断 0 / 观察 8）。
**基准提交**：M0a `097c54b9`（形状修正 `b50d93a6` + 复核 `91a9fdea`）、M1 `35192320`（QA `fe8956f4`）、M0b `ac01d86d`（QA `f191fdba`）、M2 `16214a22`（测试入口 `85cce21b`，QA `271fe926`）。
**性质**：纯台账/证据文字治理——不改 `src/`、`tests/`、`fixtures/`、`Cargo.*`，不改任何判据、容差、参考、期望值或已 verified 数字；数字一律引自各片 receipt/README 原文，本片不复算。

## 1. UNC-1 更新（前 → 后）

| 项 | 本次更新前 | 本次更新后 |
| --- | --- | --- |
| 组合层 | box/cylinder（`9a282074`） | box/cylinder **+ convex-vs-mesh**（`16214a22`）：`mesh-box`/`mesh-cylinder` 各 nacon 4/4、bit_equal 4/4、ULP 0、`contact.geom=[1,0]`、failures 0，对照 M1 GPU 参考 `c83baffe…` |
| 设备输入侧 | 旧 G5 23 数组，mesh 无输入 | 9 个 mesh polygon slice（ABI sidecar 504 / `MjwarpMeshPolyArgs` 72 B）+ **2 case × 32 数组 bins**（同一来源：xml/qpos/geom_xpos/geom_xmat 与 M1 参考逐位一致） |
| mesh 分支层 | 未实现 | **部分**：pinned `collision_gjk.py` rev `71da24d9` 的 `_intersect1/_intersect2`、`_mesh_normals`、`_mesh_edge_normals`、`_mesh_face`（逆序写出）已移植；两 fixture case 命中 face/边法线路径；swap 几何顺序沿用 G5 pinned 类型序规范化 |
| 未覆盖 | mesh multicontact 全部 | **mesh-mesh 多接触（M3 在途）**、一般 mesh（nmesh>1 / degree>10 / polygon>16 只走固定容量 clamp 回退）、`hfield-capsule` 设备接触回归（ABI 504 后无设备证据）、mesh 全类型/全分支、Linux/跨平台 |
| status | partial | **partial（不变）** |

## 2. 改动文件

- `docs/development-evidence/g10-p5-acceptance-map/map.json`：`uncovered[UNC-1]` 的 title/implementation/measured/evidence_paths（4 → 10 条）/reason/impact；`requirements[G6-2.1]` 的 measured/reason；`summary.note`。**其余 requirement 未动**，统计口径不变（12 条 requirement：met 8 / partial 4 / unmet 0；uncovered 6 条）。
- `docs/development-evidence/g10-p5-acceptance-map/README.md`：G6-2.1 行、§1「未覆盖六项」UNC-1 条改写；新增 §7「UNC-1 更新（2026-10-27）」含前→后表、mesh 侧四层拆细、窗口口径、不得声明项、证据指针。
- 本目录 `README.md` + `receipt.json`（治理回执）。

## 3. 窗口口径（沿用 Chief 2026-10-26 裁定，不改 M2 数据）

M2 验证窗 attempt4 = **数值全绿 + guard 因 free floor 环境边界 abort，经 Chief 裁定接受为有效轮**；不写「窗口通过」。attempt1–3 作废原因见 M2 证据 §8.1。M3 起 guard floor 用派生公式 `min(10240, baseline_free − 3072)`（open 门仍 `free ≥ 10240`），不追溯改 M2 数据。

## 4. 不得声明

- 不得声明 mesh 参与的多接触路径整体已覆盖、mesh-mesh 已实现、一般 mesh 已支持、mesh 全类型/全分支；
- 不得声明 G10@P5 整体通过、精度达标、上游完全等价、双平台通过、原生放行；
- UNC-1 仍为 **partial**；第二次更新（mesh-mesh）只在 M3 verified + QA 复核后做。

## 5. 证据指针

`docs/development-evidence/mesh-multicontact-m0a/`、`...-m0a-fix-mesh-poly-shapes/`、`...-m1/`、`...-m0b/`（QA `...-m0b-independent-check/`）、`...-m2/`（§7 窗口内容、§7.3 `hfield-capsule` 未覆盖、§8 attempt 与裁定、§8.5 派生口径）、`...-m2-independent-check/`（QA pass / 观察 8）。
