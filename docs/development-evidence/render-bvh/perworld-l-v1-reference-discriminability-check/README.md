# 限定说明：per-world L v1 参考无 world 间判别力（P0 事实确认 + 机器判据分类）

**结论：事实成立（仅读复核）；分类 = B 类「参考-前提型」，其前提「参考具备 world 间判别力」不成立 → 该参考上的逐 world 位姿对照降级/不适用，不得作为 per-world 位姿正确性的对照依据。**

- 任务：`2026-11-07-render-perworld-l-v1-reference-discriminability-check`（owner QA、P8-渲染与BVH、p70）
- 复核对象：既有 per-world L v1 参考 `fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/` 与 L v1 相关 verified 声明（impl-s1 卡、capture-run 卡、window attempt3 证据）
- 复核方式：**只读**（读参考 npz + 场景模型 + 历史证据/卡；零 GPU、不重跑窗口、不采集）+ 按《机器判据裁定标准 v1》分类
- 本卡登记提交：`2746db06`（目标基线 HEAD `e34df3b7`）
- 边界：**不改历史证据、不改任何既有卡状态、不改 fixtures/源码/判据/夹具**；本卡只写自己的证据目录

## 1. 事实（独立重算，不引用实现方日志）

| 检查 | 实测 |
| --- | --- |
| `state.npz` `qpos` hinge 列（3 world） | **0 / 0.34906584 / 0.69813168 rad**（0/20/40°），三 world **不同** |
| `state.npz` `geom_xpos` | 三 world **不同**：max abs diff 0→1 = **0.0503952 m**、0→2 = **0.1020426 m** |
| `state.npz` `geom_xmat` | 三 world **不同**：max abs diff 0→1 = **0.3359681** |
| `state.npz` `light_xpos`/`light_xdir` | 三 world **相同**（v1 灯数组模型级单份，与其「只覆盖 per-world geom 位姿」范围声明一致） |
| `rgb.npz` `rgb_w{0,1,2}_cam0` | 三 world **逐位完全相同**（sha256 相同，max abs diff = **0.0**） |
| `depth.npz` `depth_w{0,1,2}_cam0` | 三 world **逐位完全相同**；depth bit-changed pixels = **0 / 0** |
| `seg.npz` `seg_w{0,1,2}_cam0` | 三 world **逐位完全相同**；seg pair-changed pixels = **0 / 0** |
| `capture.json` | `two_captures_bitwise_identical=true`、`repeat=2`、`nworld=3`、文件 sha256 与参考文件逐一绑定 |

**即：参考记录的输入位姿逐 world 不同，但三 world 输出逐位相同、判别力 0/0。P0 事实成立。**

## 2. 结构性成因线索（实测，新增；不代替 P0-1 定因结论）

- 参考 `seg.npz` 的 cam0（唯一启用相机 `cam-persp`）**三 world 只包含两种像素**：`(geom 0, type 5=GEOM)` = floor，59805 px；背景 `(-1,-1)`，16995 px。**没有任何 world 变化的几何（body/tri）像素**。
- 对照同场景主参考 `fixtures/render-bvh/scene-a/reference/seg.npz`：`seg_cam0`（cam-persp）同为 **floor + 背景**；`seg_cam1`（cam-track）则包含 geom 0..5（floor + base-box/base-sphere/tri 系列）。
- 推论（限定）：`cam-persp` 视野内**只有世界不变的地板** → 该参考对 world 位姿变化**结构性不敏感**；「输入不同、输出逐位相同」至少部分由**场景/相机视野**决定，不能只归因于采集脚本缺少每 world 场景刷新。
- 与 P0-1（每 world `refit_bvh`）的关系：P0-1 修的是「场景 BVH 快照陈旧」，对 L v2 仍必要；但**单靠 P0-1 不能使 L v1 参考（以及任何以 cam-persp 为唯一判据相机的同视野参考）具备 depth/seg 判别力**。
- **对 L v2 的影响（请 Chief/Arch 定夺）**：spec v2 §6 的判别力门槛要求相邻 world 的 **depth 与 seg 变化像素均 ≥384**（采集前置 ≥576）；scene-a-light 只把灯挂到 `tri_body`、相机/几何与 scene-a 相同 → 灯不改变 depth/seg，视野内又没有 world 变化几何 ⇒ **按现 spec/场景，depth/seg 门槛结构性不可达**（rgb 可能因灯光位姿变化而有差异，但 §6 不含 rgb 判别力）。**建议在 S-2 签名前先决定：改场景/相机使世界变化几何可见（属改参考范围 → 新 spec + 重签），或改判别力口径（属判据签名级事项），不要等到 pilot/窗口内才发现。**

## 3. 机器判据裁定分类（按《机器判据裁定标准 v1》）

- 标准路径：`docs/development-evidence/criteria-purpose-audit/standard.md`（v1）。
- **分类 = B 类（参考-前提型）**：L v1 的「逐 world 位姿级对照」以参考为基准，属参考型判据；标准 §2 要求参考型判据写明前提、前提不成立即降级（§3：B 类仅在声明前提下有效、不得单独作发布依据；§4 高危项：同一判据跨场景复用未复核前提、用证据完整性代替数值正确性）。
- **前提是否成立**：参考型判据除「同 recipe / 同编译形态 / 同 GPU+驱动 / 同 warp 版本」外，本例还需**参考有效性前提：参考自身具备 world 间判别力**。实测 **不成立**（三 world 输出逐位相同、判别力 0/0）。
- **裁定结论**：该参考上的「逐 world 位姿对照通过」是**空转**；对应声明**降级/不适用**，不得作为 per-world 位姿正确性依据，更不得作发布依据。

## 4. 声明清单：仍成立 vs 需降级/加限定

**仍成立（但都只到相应层次）**
1. **C 类完整性**：参考字节与哈希绑定（`capture.json.files_sha256`、两跑 `bitwise_identical=true`、legacy 未被改）——只证明留痕/确定性与字节一致，不涉数值正确性。
2. **数据面事实**：`state.npz` 逐 world 记录了不同的 `geom_xpos/geom_xmat` 输入（记录本身为真；不等于渲染按 world 正确）。
3. **产品侧窄信号**：L v1 窗口 attempt3 中 `render_per_world` exit 0，其用例含 product-side world 隔离断言（仅改 world1 位姿后 world0/world2 不变、world1 输出必须变化）。该断言不依赖参考判别力，能说明「位姿变化会影响产品输出」，但**不能**证明与参考逐 world 一致、也不能单独证明 per-world 正确性。
4. `2026-10-30-render-perworld-impl-s1` 卡内「逐 world 正确性」条目中，**实现机制**（per-world 位姿容器/AA 守卫/逐 world 发射编排/CPU 单测/门禁）部分仍成立；**「用该参考逐 world 位级比对通过」这一数值结论部分**按本说明降级。

**需降级 / 加限定（引用方必须遵守）**
1. 「逐 world 位姿级对照通过（rgb mismatched=0 / depth max_ulp≤4 / seg 全等）」**在该参考上是空转**——不得再作为 per-world 位姿正确性、能力解锁或发布依据。
2. 「该参考可用于后续 per-world 位姿对照」→ **不得用于位姿正确性对照**；仅可作布局/哈希/回归留痕（且其回归价值对 world 位姿变化不敏感）。
3. `impl-s1` 卡的 verified 状态**不回改**（按指令），但引用其「逐 world 正确性」时**必须同时引用本限定说明**；新声明不得建立在无判别力参考上。
4. L v2 / 后续引用方：在 S-2 签名前先处置 §2 的「cam-persp 只看到地板」问题（改场景/相机 = 新 spec + 重签；改判别力口径 = 判据签名级事项）。

## 5. 最小结论句（供 Chief/引用方直接使用）

> **L v1 参考无 world 间判别力，不能作为 per-world 位姿正确性的对照依据；待 P0-1 修复 + pilot 通过后新参考再验证。**

## 6. 证据清单

| 路径 | 说明 |
| --- | --- |
| `scripts/verify_l1_reference_review.py` | 复核脚本（14 项只读复算：事实/哈希/可见性/分类/历史不改 + P0-3 锚点/话术/建议，可复跑且复跑不改写易变字段） |
| `logs/verify-l1-reference-review.json` | 检查结果机器可读（PASS n/n） |
| `logs/verify-l1-reference-review.log` | 脚本原始输出 |
| `artifacts/reference-facts.json` | 逐 world 逐位相等性、max abs diff、变化像素、文件哈希绑定 |
| `artifacts/state-facts.json` | qpos/geom_xpos/geom_xmat/light 数组逐 world 差异 |
| `artifacts/visibility.json` | seg 像元构成（perworld cam0 / 主参考 cam0 / cam1 对照） |
| `artifacts/l1-claims.json` | L v1 相关卡与窗口 attempt3 声明清单（原文引用） |
| `artifacts/classification.json` | 按标准的分类、前提判定、仍成立/降级清单 |
| `artifacts/unchanged.json` | 历史证据/卡零改动探针 |
| `artifacts/evidence-chain.json` | P0-3 证据链锚点映射（7 条：依赖参考像素对照 vs 产品侧/完整性） |
| `artifacts/pilot-advice.json` | P0-3 对 P0-1/pilot 的独立建议 |
| `README.md` / `receipt.json` | 本限定说明与机器可读回执 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 7. 边界与非声明

- 只读复核 + 只写本卡证据目录；**不改任何历史证据、不改任何既有卡状态**、不重跑窗口/GPU、不改 fixtures/源码/判据/夹具。
- 本说明不产生参考、不宣称 L v2 采集有效、不宣称 per-world 灯验收、不与上游等价、不宣称任何渲染数值通过；仅确认 P0 事实并给分类与限定。
- 成因线索（§2）是**实测的视野构成事实 + 由此得到的限定推论**；P0 的最终定因仍以 P0-1/P0-2/pilot 与后续诊断为准。

## 8. 复现

```powershell
cd D:\Rust\mjwarp-rs
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/perworld-l-v1-reference-discriminability-check/scripts/verify_l1_reference_review.py
# 期望末行：SUMMARY PASS 14/14
```

## 9. 证据链锚点映射（P0-3 补充）

| 证据/文件 | 锚点 | 断言 | 是否直接依赖该参考的逐 world 像素对照 | 仍成立 / 降级 |
| --- | --- | --- | --- | --- |
| `.normify/changes/2026-10-30-render-perworld-impl-s1.json` | acceptance #2「逐 world 正确性」 | rgb/depth/seg 逐 world 位级对照 + world 隔离 + 两跑 SHA | **是（像素对照部分）** | 隔离、两跑 SHA 仍成立；像素对照降级（空转） |
| `docs/development-evidence/render-bvh/perworld-impl-s1/README.md` | 「实现机制」1–4 节 | adr 位移定位、内核索引点不变、逐 world 编排、leaf 槽位校验 | 否 | 仍成立（实现面事实） |
| `docs/development-evidence/render-bvh/perworld-impl-s1/window-attempt3/entity-results.csv` + `logs/render_per_world.log` | `render_per_world,new,0`；`gpu_per_world_matches_captured_reference ... ok` | 实体通过（含参考对照/两跑/隔离） | **是（整体用例）** | 隔离/确定性仍成立；参考对照降级 |
| `tests/render_per_world.rs` | `:378-420` 参考对照 / `:422-435` 两跑 SHA / `:437-469` world 隔离 | 参考对照 + 确定性 + 隔离断言 | **部分是** | `:422-435`、`:437-469` 仍成立；`:378-420` 降级 |
| `docs/development-evidence/render-bvh/perworld-s1-window-material/materials/entity-table.json` | `render_per_world` 实体条目 | 窗口内用该参考做逐 world 对照 | **是** | 材料/冻结/纪律部分成立；参考对照判据降级 |
| `docs/development-evidence/render-bvh/perworld-capture-run/receipt.json`（及 manifest） | `capture.json` `files_sha256` / manifest 逐文件 sha256+bytes | 参考存在、哈希绑定、两跑 bitwise 一致 | 否 | 仍成立（C 类完整性/确定性） |
| `docs/development-evidence/render-bvh/perworld-capture-check/` | QA 采集复核证据 | 采集产物完整性/确定性/边界 | 否 | 仍成立（C 类）；不覆盖 world 判别力（当时未要求） |

## 10. 面向用户的限定话术（P0-3 补充，供 Chief 直接引用）

> **L v1 per-world geom 位姿覆盖中，「以 `nw3-hinge02040` 参考做逐 world 像素对照并通过」这部分为空转：该参考三 world 输出逐位相同、无 world 间判别力，不能作为 per-world 位姿正确性的对照依据；仍成立的是参考的确定性/哈希完整性、逐 world 位姿数据面记录，以及产品侧 world 隔离断言——后者只说明位姿变化会影响产品输出，不等于与参考一致。待 P0-1 修复 + pilot 通过后的新参考再验证。**

## 11. 对 P0-1 / pilot 的独立建议（P0-3 补充）

- **是否支持 refit_bvh 方向：支持（必要修复）**。理由：调用序/上游语义核对通过（见 `perworld-l-v2-capture-perworld-refresh` 与 QA 复核卡 `...-refresh-check`），它解决「render 前未刷新场景 BVH」这一根因假设；但**不改变**视野内看不到 world 变化几何的事实。
- **pilot 期望**：按现 `scene-a-light` + `cam-persp`（视野内仅 floor），depth/seg 变化很可能仍为 0；即使 rgb 因灯光位姿变化而有差，spec v2 §6 的 depth/seg 门槛仍不可达。**若 pilot 仍 `non-discriminating`，不必然否定 refit 修复**（它修的是另一问题），但说明「改场景/相机使 world 变化几何可见」或「改判别力口径」是前置（均属签名级）。
- **pilot 必须记录**：per-world rgb/depth/seg 参考与产品侧变化像素、seg 可见 geom id 集合、rgb 差异来源（灯光/阴影面 vs 几何面）、refit 是否实际执行（调用计数/日志）。
- **不替实现卡/判据下结论**；refit 修复本身按 P0-1 卡与 QA 复核结论（PASS）有效。

## 12. 修订记录（fix）

- 2026-11-07：按 Arch P0-3 acceptance 补充 §9（证据链锚点映射）、§10（面向用户限定话术）、§11（P0-1/pilot 独立建议）。**主体 §1–§8 的事实、分类与限定未改**；不改历史证据/既有卡状态。
- 2026-11-07（hold 期 O1–O3 修复）：O1 复核脚本追溯判定改为稳定判定（不再写 `current_head`/`after`），复跑不改写易变字段、复跑后树干净；O2 §6 计数 10→14 并补 `evidence-chain.json`/`pilot-advice.json` 两行、§8 复现摘要 14/14；O3 manifest 相对路径统一正斜杠。**§1–§11 正文结论未改。**

