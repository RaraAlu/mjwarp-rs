# R06 多帧实体 A 小修：cam2-only 跨帧判据 + 选项 A 诚实口径（零 GPU）

- 任务：`2026-11-05-render-r06-flex-multiframe-entity-fix`（owner Backend、p75、P8-渲染与BVH）
- 依赖：实体 A 卡 `2026-11-05-render-r06-flex-multiframe-entity`（verified；实现 `2a27c2fd`、证据校正 `241d702f`）
- 来源：Chief 2026-11-05 派工（Arch 旁证 7 项里 5 PASS，2 项验证窗前必修）
- 边界：**零 GPU、不开窗、不采集、不执行 freeze**；不改 fixtures/spec/判据；`tests/render_flex_validate.rs` 零 diff

## 1. 结论

- **跨帧判据改为 cam2-only**：新增 `change_pixels_cam2()`（只取 cam2 槽位）作为唯一断言口径
  （`verify_change_pixels`、CPU 自证 ⑤⑥、GPU 用例跨帧段）；原 3 相机求和实现改名
  `change_pixels_all_cameras()`，只在 CPU/GPU 用例里 `println!` 诊断、不参与断言。实测 cam0/cam1 帧间变化为 0，
  cam2-only 与全相机合计逐对相同：`000→001 = (1167, 1074)`、`001→002 = (1262, 1126)`（与 G-2 采集侧数字一致）。
- **选项 A 口径诚实化（Chief 2026-11-05 裁决）**：不补 `body_pos` 偏移步骤（产品渲染路径不消费 `body_pos`，
  补一步是演戏）。口径 =「逐帧 state 刚性位姿建 3 个 context + 断言三者刚性签名一致 + 同一 context 渲染对照 +
  `refits()==0`」；并写明「spec §2 的实现映射：模型 `body_pos` 偏移对 flex 渲染惰性（产品每次渲染用通道
  positions 重建合并 BVH）」。
- **证据校正**：实体证据 README §2 表格 support 模块 476→**475**（`241d702f` 漏行）、「逐帧逐相机两跑 sha」→
  「**整块多相机缓冲 sha**（更强，等价逐相机相等）」、跨帧段改 cam2-only 口径；行数/`script-identity.json`/
  `receipt.json`/`manifest.txt` 同步本卡改动后的实际值。
- **backlog**：recipe 参数比对 7 项 vs 单帧 28 项（低风险）→ `2026-11-05-render-r06-flex-multiframe-recipe-params-backlog`
  （status backlog），本卡不修。
- CPU 自证 `38 passed / 0 failed / 1 ignored`（exit 0）；四门禁全过（unit **1359 过 0 败**）；
  `tests/render_flex_validate.rs` 498 行零 diff。

## 2. 改动与行数

| 文件 | 改动 | 行数 |
| --- | --- | --- |
| `tests/render_flex_multiframe.rs` | CPU 自证 ⑤⑥ 与确定性测试改 `change_pixels_cam2`；GPU 用例跨帧段 cam2 断言 + 全相机诊断打印；选项 A 注释诚实口径 | 489 → **499** |
| `tests/support/render_bvh/multiframe.rs` | 新增 `CAM2`/`cam2_slot`/`change_pixels_cam2`；原实现改 `change_pixels_all_cameras`（仅诊断）；`verify_change_pixels` 改 cam2-only | 475 → **496** |
| `docs/.../r06-flex-multiframe-entity/README.md` | §1/§2/§3/§4/§5 校正（cam2-only、选项 A 裁决、行数、sha 措辞） | 126 |
| `docs/.../r06-flex-multiframe-entity/receipt.json` | `criteria.cross_frame/option_a` 增补、行数、`revisions` | — |
| `docs/.../r06-flex-multiframe-entity/logs/line-counts.txt` | 行数同步 | 7 |
| `docs/.../r06-flex-multiframe-entity/artifacts/script-identity.json` | 两文件行数/bytes/sha256 同步 | 17 |
| `docs/.../r06-flex-multiframe-entity/manifest.txt` | sha256/bytes 重算（8 文件） | 11 |

## 3. cam2-only 前后对照

| 项 | 修改前 | 修改后 |
| --- | --- | --- |
| 实现 | `change_pixels()` 对 3 相机求和 | `change_pixels_cam2()` 只算 cam2；`change_pixels_all_cameras()` 仅诊断 |
| 断言口径 | 全相机合计 ≥384 | **cam2-only ≥384**（spec §3/§5） |
| cam0/cam1 变化 | 被静默计入合计 | **不参与判定**；仅 `println!` 打印 |
| 实测 | 合计 (1167,1074)/(1262,1126) | cam2 (1167,1074)/(1262,1126)（当前 cam0/1=0，两者恰好相同） |

放宽风险定位：spec §3 写死「cam0/cam1 的帧间变化像素数只作诊断、不参与跨帧判定」。若未来 cam0/cam1 出现
意外变化，求和口径会让变化像素数变大、掩盖 cam2 判据质量（静默放宽）。改 cam2-only 后判据只反映判据相机的
可见性变化；全相机合计仍打印，供诊断对照。

## 4. 选项 A 裁决留痕（Chief 2026-11-05）

- 裁决要点：「不补 `body_pos` 偏移步骤（产品路径不消费 body_pos，补一步是演戏）；把口径改成逐帧 state 刚性
  位姿建 3 个 context + 断言三者刚性签名一致 + 同一 context 渲染对照 + `refits()==0`」。
- 落地位置：本卡 acceptance #2；实体证据 README §5（含裁决段与 spec §2 实现映射）；
  GPU 用例选项 A 注释（`tests/render_flex_multiframe.rs`）；`receipt.json.option_a.chief_ruling`（修复卡 receipt 为**顶层 `option_a`**）。
- 实体 A 卡 acceptance #4 不可改写（normify 禁止改写已启动任务的验收）；裁决以本卡 acceptance +
  实体证据 README §5 为准，不返工既验证结论。
- spec §2 实现映射（写进 README §5）：「模型 `body_pos` 偏移对 flex 渲染惰性（产品每次渲染用通道 positions
  重建合并 BVH）」。

## 5. 证据校正（实体证据目录）

- README §2：`tests/support/render_bvh/multiframe.rs` 行数 476 → 475（`241d702f` 校正漏行；本卡改动后同步为 496）。
- README §3：两跑 sha 措辞 → 整块多相机缓冲 sha（更强，等价逐相机相等）。
- README §1/§4/§5：跨帧段改 cam2-only；选项 A 改诚实口径 + spec §2 实现映射。
- README 顶部「修订」注记；receipt `revisions`；line-counts / script-identity / manifest 同步。
- 未改：12 npz、3 文本夹具、`capture.json`/`cam-poses.json` 内容、spec、`tests/render_flex_validate.rs`。

## 6. CPU 自证（零 GPU）

```text
cargo test --locked --features cuda-probe --test render_flex_multiframe -- --nocapture
exit 0；38 passed / 0 failed / 1 ignored
pair 0: cam2=(1167, 1074) all=(1167, 1074)
pair 1: cam2=(1262, 1126) all=(1262, 1126)
```

原始输出：`logs/cpu-tests.log`；22 项实体 CPU 自证数量未变（仅改判定口径与诊断打印）。

## 7. 门禁与边界

- `normify gate`：fmt 通过 / clippy 通过 / clippy-cuda 通过 / unit 通过（**测试 1359 过 0 败**），结果在
  `.normify/runs/2026-11-05-render-r06-flex-multiframe-entity-fix/`。
- `unchanged`：`src/`、`fixtures/`、`tests/render_flex_validate.rs`（498 行零 diff）、
  `docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/`（spec sha `1d9fa5ce…`/10,506 B 不变）。
  对照：`artifacts/unchanged.txt`。
- 零 GPU、不开窗、不采集、未执行 freeze。
- 行数：`render_flex_multiframe.rs` **499**、`multiframe.rs` **496**（均 ≤500）；见 `logs/line-counts.txt`。

## 8. 已知项 / backlog

- recipe 参数比对：实体 A 的 `assert_params_match_capture` 比对 7 项，单帧验收 28 项；风险低（同一 pinned 环境/
  同一 recipe 采集 + provenance 已锚定），登记 backlog 卡不修。
- G-1 负结果与 G-2 证据保留不删；本卡不产生验收结论。

## 9. 非声明

本卡只修跨帧判据实现口径与证据数字，不产生验证窗结论；不写 R06 整族通过、动画通用性、与上游实现的等价结论；
GPU `#[ignore]` 用例真实结论留验证窗执行（Chief token）。

## 10. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `logs/cpu-tests.log` | CPU 自检原始输出（command + exit_code + 38 用例 + cam2/全相机诊断） |
| `logs/line-counts.txt` | 收口时各文件 git blob 行数 |
| `artifacts/fix.diff` | 相对本卡 start 之后 HEAD 的完整 diff（tests + 实体证据） |
| `artifacts/unchanged.txt` | `src/`、`fixtures/`、`render_flex_validate.rs`、spec 目录 unchanged 对照 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 11. 复现

```powershell
cargo test --locked --features cuda-probe --test render_flex_multiframe -- --nocapture
git diff --stat HEAD -- src/ fixtures/ tests/render_flex_validate.rs docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/
```
