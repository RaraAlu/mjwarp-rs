# SDF 场景产品侧模型 dump（U4a）独立复核（QA，只读）

任务：`2026-10-31-render-sdf-model-dump-check`
被核：`2026-10-31-render-sdf-model-dump`（U4a；主卡 verified）
被核 pin：**`60541da3`**；核 start 时 `revision.before = abbb19ed`。
**结论：pass / 阻断 0 / 需修 0 / 观察 7。**
本卡只读（dumper 是 offline 工具；重跑只写 `target/` 与同字节覆盖测试 JSON）。

## 0. 一句话

U4a 用 S0a 已验收的 offline dumper（MuJoCo **3.12.0** 产品版本）把两个 SDF 场景 XML dump 成 render 模型 JSON，落到 `tests/support/render_bvh/sdf/`。我**独立重跑 dumper 两次（同路径）**：两个 dump 的 sha/bytes 与 committed 完全一致、仓库树除本复核卡外零变化；dump 语义（counts / geom_type / geom_dataid / mesh bounds / 相机分辨率）与证据 `dump-counts.json` 逐项一致，SDF geom 的 `geom_size=0.35` 与 mesh bounds `±0.35` 并存，正好为 U4 host 规则「SDF leaf 用 mesh bounds 而非 geom_size」提供可断言输入。`.mjb` 中间产物只在 `target/`、未入库。

## 1. 复核方法

- dump 身份/语义/manifest：`git cat-file` 读 committed blob + 本机 pinned python 解析 JSON。
- 确定性：用证据 `materials/dump-commands.txt` 的两条命令，在 `target/toolchains/mujoco-3.12.0/package/bin` 前置 PATH 下**各重跑两次（同一输出路径）**，比对 sha/bytes，并检查 `git status`（结果写 `materials/rerun-determinism.txt`）。
- 重算脚本：`recompute.py`（8 项 check → `recompute.json`，problems 0）。

## 2. 结果（8 项 check 全过）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `dumps.identity` | ✓ | `model-312.json` **53,849 B / `7de15ff4…`**；`model-mixed-312.json` **54,205 B / `60698f6b…`**；`mujoco_version=3.12.0`（int 3012000）、`loaded_mjb=true`、`model_file` 指向 `target/render-bvh/sdf-model-dump/*.mjb`、`source_xml` 为仓相对 fixtures 路径 |
| 2 | `determinism.rerun` | ✓ | 我重跑：每个 dump **2 次 exit 0，sha/bytes 完全相同且等于 committed**（4 次运行 4 个一致 sha）；重跑后 `git status` 仅本复核卡变化（dump 字节未被改写） |
| 3 | `dumps.semantics` | ✓ | counts 与 `materials/dump-counts.json` **逐项相等**；`geom_type=[0,8,2]`/`[0,8,2,6]`、`geom_dataid=[-1,0,-1]`/`[-1,0,-1,-1]`（SDF→mesh0）；mesh 8 顶点 / 12 面、bounds **±0.349999994**；SDF `geom_size=0.35`；`ncam=2`、`nlight=2`、`nq=0`；cam_resolution `[320,240,240,180]`、light_castshadow `[1,1]` |
| 4 | `tool.identity` | ✓ | `target/render-bvh/s0a/compile_scene_312.exe` **161,280 B / `cd91e099…`** 与证据一致；MuJoCo 3.12.0 runtime（`mujoco.dll`）存在；工具是 offline（不参与产品 targets/tests/gates） |
| 5 | `scope.pin_commit` | ✓ | `4932f7f3^..60541da3` 共 **10** 路径 = 2 个 dump + 证据 7 + 卡 1；`src/`、`fixtures/`、`Cargo*`、`scripts/`、`*.rs` **0** 改动 |
| 6 | `intermediate.mjb_not_committed` | ✓ | 仓库无 `sdf-model-dump`/`tests/support` 相关 `.mjb`；`target/render-bvh/sdf-model-dump/{scene-sdf,scene-sdf-mixed}.mjb` 存在（未跟踪） |
| 7 | `evidence.manifest` | ✓ | manifest **6/6**、7 文件、无 UTF-16 |
| 8 | `card.checks_and_validate` | ✓ | 主卡 `normify check` 通过；`validate --fingerprints` = **0 错误 / 5 警告** |

## 3. 观察 7（记录级接受）

- **O1 我复现了「同路径两次 dump 逐字节相等」**：4 次运行（两文件各两次）sha 全等于 committed，说明 dumper 在这个环境/输入下确实确定；重跑只写 `target/*.mjb` 与同字节覆盖 JSON，仓库无副作用。
- **O2 dump 内 `tool` 字段只有名字**（`compile_scene_312`），工具 sha256 只在证据材料/receipt 里；若将来 dumper 被重编译，dump 自身无法证明工具身份。建议（后续，非本卡）：dump 里带上 tool sha；因会改 dump sha，需在下次 dump 时一并刷新材料。
- **O3 `.mjb` 中间产物只在 `target/`**（约 9.4 MB × 2，未跟踪）；`target/render-bvh/sdf-model-dump/` 里还有一份工作副本 `determinism.txt`，不影响仓库。
- **O4 确定性依赖同路径写**：dump 内 `model_file` 字段嵌入输出路径，命令行必须与证据逐字一致才能得到同 sha —— 证据已在 `determinism.txt` 写明，我按同路径复现成功；建议 U4 材料不要改 dump 输出路径。
- **O5 语义链对 U4 有用**：SDF geom `geom_size=0.35` 与 mesh bounds `±0.35` 同时存在，U4 host 用例可据此断言「SDF leaf 用 mesh bounds」，而不会被 `geom_size` 误导；相机分辨率/灯 castshadow 与 U3 采集场景一致。
- **O6 只加 `tests/support/**` JSON，不动 `tests/*.rs`**：U4 的 host 用例尚未接入（后续 target 会读这两个 JSON），本卡无法验证「用例如何使用 dump」——归 U4 复核。
- **O7 dump 是单行大数组 JSON**（53 KB），文本 diff 不友好但可稳定解析；证据用 sha + counts 记录而不是靠 diff，口径合适。

## 4. 边界与不宣称

- 本卡只读复核：不改 dump、不改 `src/tests/fixtures/scripts`、不改他人证据；dumper 重跑只写 `target/` 与同字节覆盖。
- 确认主卡口径：U4a 只是「产品侧模型 dump（offline）」，不跑 GPU、不做数值判定、不宣称能力；dump 是 U4 验证窗的 host 输入。
- 不宣称渲染数值验收通过、不宣称上游等价、不宣称 SDF 渲染已覆盖。

## 5. 限制

- dump 语义只能证明「模型数据与场景一致」，不能证明设备端渲染结果（U4 窗口）。
- dumper 工具身份依赖证据材料记录（dump 内无 tool sha）；我核对的是 `target/` 下现盘 exe 的 sha。
