# U4 SDF 参考验证目标实现（render_sdf_reference + 支撑模块 + 冻结器）

任务：`2026-10-31-render-sdf-reference-target`（Arch；P8；离线，零 GPU、零窗口）
前置：`2026-10-31-render-sdf-host-unlock`（U1b）、`2026-10-31-render-sdf-model-dump`（U4a）、`2026-10-30-render-sdf-capture`（U3）
后续：`2026-10-31-render-sdf-validate` 窗口（需用户双签；本片不跑 GPU）

## 1. 交付

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `tests/render_sdf_reference.rs` | 361 | 8 用例：host 3（模型 dump 语义 / 参考自证 / SDF 用 mesh bounds）+ GPU 5（两组参考比对 / shadow 一致性 / DepthSeg 跨路径 / 两跑确定性） |
| `tests/support/render_bvh/sdf.rs` | 372 | U3 参考加载与硬断言（capture.json 全字段 + npz sha 对 `files_sha256`，仿 `hfield.rs`） |
| `tests/support/render_bvh/sdf_scene.rs` | 323 | 产品场景装配（mesh 0 → `MeshState`、SDF leaf size=mesh bounds、pose=geom_pos/单位 quat） |
| `tests/support/render_bvh/sdf_compare.rs` | 260 | 比较统计 + 判据断言 + shadow 一致性 + DepthSeg 跨路径 |
| `scripts/validate/render-bvh/freeze-guards-sdf.ps1` | 167 | 新 suite `render-sdf-validate` 冻结器（ASCII、≤200 行、PS 5.1、硬断言 9/83/92） |

全部 ≤500 行；`sdf_scene.rs` 原本 573 行，按材料预案**拆出 `sdf_compare.rs`**（不突破 500 行棘轮）。

## 2. 用例与判据

- 判据沿用 R09-S3b / R05-S3b，**零放宽**：RGB `beyond_1_lsb=0`/`max_byte_diff≤1`；depth `mismatched=0` 且 `max_ulp≤4`/`beyond_ulp=0`；seg `mismatched=0`；两跑 rgb/depth/seg sha256 相等。
- **shadow 开/关臂只做一致性**（没有上游 shadow-off 参考）：depth/seg 两臂位级相等；rgb `off >= on`（逆序超 1 LSB 的通道=0）；差异像素>0；两臂各自两跑 sha 相等。不宣称数值参考。
- **DepthSeg 跨路径臂**：`DepthSegRenderer` 与 megakernel 在相同 context/输入下 depth/seg 位级相等（cam 全量）。
- host 用例可直接离线跑（`cargo test --locked --features cuda-probe --test render_sdf_reference`）：**3 passed / 5 ignored**；`--no-run` EXIT=0 且含 `Finished`；`unit` 门禁 **1109 过 / 0 败**（U1b 后 1106 + 本片 3 条 host 测试）。

## 3. 边界

- 零 GPU、零窗口；不改产品 `src/**`、`.cu`、既有 tests、判据/容差/参考/fixtures。
- 冻结器本片**不执行**（不产 guards 文件）：`freeze` 属窗口 SOP（monitor 通过后），执行时会跑 `cargo test --no-run` 解析 9 个 exe 并硬断言 9/83/92。
- 两个模型 dump 的 sha 在 host 用例里 pin（`sdf-model-dump` U4a 收口值）。

## 4. 证据清单

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文 |
| `receipt.json` | 任务 id、文件身份、用例数、门禁、no-run、冻结器静态检查、边界 |
| `manifest.txt` | 本目录除自身外全部文件 sha256/bytes |
| `logs/gate-*.txt` | 四门禁转写快照（权威 = `.normify/runs/`） |
| `logs/no-run.txt` | `cargo test --locked --no-run` 原样日志（EXIT=0、Finished） |
| `logs/host-test.txt` | host 用例运行日志（3 passed / 5 ignored） |
| `logs/test-list.txt` | `--list` 的 8 个用例名 |
| `materials/file-identity.txt` | 5 个新文件行数/bytes/非 ASCII/sha |
| `materials/changed-files.txt` | 纯新增文件清单（无产品源码改动） |
| `materials/freezer-checks.txt` | 冻结器 ASCII/行数/PSParser/实体顺序/预期计数 |
