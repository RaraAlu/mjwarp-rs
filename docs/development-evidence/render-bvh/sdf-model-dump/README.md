# SDF 场景产品侧模型 dump（U4a）

任务：`2026-10-31-render-sdf-model-dump`（Arch；P8；离线、零 GPU、零窗口）
前置：U1b `2026-10-31-render-sdf-host-unlock`（verified）、U3 `2026-10-30-render-sdf-capture`（verified）
后续：U4 材料落卡（pin 本片 dump sha）→ U4 验证窗

## 1. 做了什么

用 S0a 已验收离线 dumper（`target/render-bvh/s0a/compile_scene_312.exe`，MuJoCo **3.12.0** 产品版本，sha256 `cd91e099…` / 161,280 B）把两个 SDF 场景 XML dump 成 render 模型 JSON，供 U4 新 target `render_sdf_reference` 作为 host 场景输入：

| 输出（`tests/support/render_bvh/sdf/`） | 字节 | sha256 |
| --- | --- | --- |
| `model-312.json`（scene.xml） | 53,849 | `7de15ff458aa8f6b1c026bbed5d9b1fbaf8f5dd39b385bab0b79d5c289b40d36` |
| `model-mixed-312.json`（scene-mixed.xml） | 54,205 | `60698f6baaeff8dcc11e725e56242bd5d6f947741cb7070b4e1f32ec13765578` |

- 输入签字值：`scene.xml` 3,388 B / `376eb81c…`；`scene-mixed.xml` 3,062 B / `7869cfae…`（见 `materials/dump-identity.txt`）。
- 中间 `.mjb`（约 9.4 MB）只落 `target/render-bvh/sdf-model-dump/`，**不入库**。

## 2. 语义自检（`materials/dump-counts.json`）

| 项 | `model-312.json` | `model-mixed-312.json` |
| --- | --- | --- |
| `mujoco_version` | 3.12.0 | 3.12.0 |
| `loaded_mjb` | true | true |
| `counts.ngeom` | 3 | 4 |
| `geom_type` | `[0, 8, 2]` | `[0, 8, 2, 6]` |
| `geom_dataid` | `[-1, 0, -1]` | `[-1, 0, -1, -1]` |
| `counts.nmesh / nmeshvert / nmeshface` | 1 / 8 / 12 | 1 / 8 / 12 |
| `counts.ncam / nlight / nq` | 2 / 2 / 0 | 2 / 2 / 0 |

要点：SDF geom（type=8）的 `geom_dataid` 指向 mesh 0；SDF 的 leaf bounds 必须用 mesh bounds（不是 `geom_size=[0.35,0.35,0.35]`），U4 host 用例会硬断言这一点。

## 3. 确定性

同一命令连续两次 dump 到**同一路径**，逐字节相等（dump 内 `model_file` 字段记录输出路径，故路径必须一致；重复运行不改变 sha）：
- `model-312.json`：两次 `7de15ff4…` / 53,849 B
- `model-mixed-312.json`：两次 `60698f6b…` / 54,205 B

## 4. 边界

- 只在 `tests/support/render_bvh/sdf/` 与证据目录落地；**不动 `fixtures/**`**、不改 `src/**`、`tests/*.rs`、`Cargo.*`、`scripts/**`、不改判据/容差/参考。
- 零 GPU、零内核身份变化、零产品构建参与（dumper 是 offline 工具，不参与产品 targets/tests/gates）。

## 5. 证据清单

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文 |
| `receipt.json` | 任务 id、工具/输入/输出 sha、counts、确定性、边界 |
| `manifest.txt` | 本目录除自身外全部文件 sha256/bytes |
| `materials/dump-commands.txt` | 两条 dump 命令逐字（含 PATH 前置要求） |
| `materials/dump-identity.txt` | 工具 sha、输入场景 sha、输出 sha/bytes、中间 mjb |
| `materials/dump-counts.json` | 两个 dump 的版本/counts/geom_type/geom_dataid/mesh 计数 |
| `materials/determinism.txt` | 两次同路径 dump 逐字节相等记录 |
