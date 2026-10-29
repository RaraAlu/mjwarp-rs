# R02-S3a：窗口签字产物入库（天空盒参考 4 文件 + r02-skybox guards attempt=2）

任务：`2026-10-29-render-bvh-r02-skybox-s3-artifacts-commit`（P8-渲染与BVH，owner Backend，Arch 登记）
本片性质：**产物入库片**。窗口 attempt=2 依赖的两类签字产物此前未被跟踪（S3 卡只含文档），本片把它们以**字节不变**方式入库并记录身份，保证 S3 证据可追溯、后续窗口可复现。不改字节、不跑 GPU、不改判据。

## 1. 入库产物与身份

| 产物 | 路径 | bytes | sha256 |
| --- | --- | --- | --- |
| 天空盒参考 capture | `fixtures/render-bvh/scene-a/reference-skybox/capture.json` | 4 730 | `47f704de0421abbcfd89634b156323867c255aeb3f0734db7f78fc3274295a9a` |
| rgb 参考 | `fixtures/render-bvh/scene-a/reference-skybox/rgb.npz` | 1 440 478 | `cf65a5709ef01f6036e0b4abe06a11ec09e0a119f07bea18634817ac1214778f` |
| depth 参考 | `fixtures/render-bvh/scene-a/reference-skybox/depth.npz` | 480 486 | `28c1ea4bdd4533a3d4811f390051a9e0dfa75ccab2549d7b32b8b426ccc9ba14` |
| seg 参考 | `fixtures/render-bvh/scene-a/reference-skybox/seg.npz` | 960 478 | `2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97` |
| R02 guards（attempt=2） | `scripts/validate/render-bvh/guards-r02-skybox.json` | 28 931 | `2d55c0b3fc14a33b2acce159bf68f474131f2819e3b2230f17ece11021b2e6a3` |

参考目录恰好 4 文件（无部分产物、无 attempt1 残留）；attempt1 旧参考仍在 `target/render-bvh/r02-skybox-window/stage-a-attempt1-reference-superseded/`（不入库，S3 证据记录其 sha）。

## 2. 与 S3 证据自洽

S3 receipt/README 引用的 capture sha、四份参考 sha、guards sha/bytes/aggregate（`ca1fcaa5…`）与入库文件逐一相等；guards 的 68 只读输入包含这 4 个参考文件，`-GuardsOnly` 77/77 的历史记录可复核。

## 3. 已知口径与限制（同 R05）

* `guards-r02-skybox.json` 的签字字节依赖本地 `.git/info/attributes` 的 `scripts/validate/render-bvh/guards-r02-skybox.json -text`（与 R05 `guards.json` 同口径）；该属性是**本地配置、不入库**，因此 fresh clone 上 guards 的 sha256 可能需要重新归一才复现（R05 QA O7 同类限制，已登记）。
* 本片不产任何渲染数值结论；不重跑窗口、不改判据。

