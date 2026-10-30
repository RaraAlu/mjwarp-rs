# SDF 渲染参考采集 authoring（U2）独立复核（QA，只读）

任务：`2026-10-30-render-sdf-capture-authoring-check`
被核：`2026-10-30-render-sdf-capture-authoring`（U2；主卡 verified）
被核 pin：**`65a16430`**（U2 起点 `828e37cc`）；核 start 时 `revision.before = b9bc2c1d`。
**结论：pass / 阻断 0 / 需修 1（窗口材料声明的自检脚本未实现，轻微）/ 观察 7。**
本卡只读：零 GPU、零窗口、零设备执行、零参考写入；dry-run 用 pinned 环境离线复跑（`CUDA_VISIBLE_DEVICES=99`）。

## 0. 一句话

SDF 参考采集的「场景 fixture + 派生脚本 + 窗口材料」三件我逐项独立复现：脚本身份、两个场景身份、静态模型事实、两组 `--dry-run` 输出（与证据日志**逐字段 0 diff**）、legacy 69 条全对；窗口材料是常规 `util<40`、无双签填写、命令逐字可执行。唯一需修：窗口材料/`expected-products.json` 声称「`--out-dir` 非空即拒」是脚本硬自检，但脚本**没有实现**该检查（只校验子树位置，随后 `mkdir(exist_ok=True)`）。

## 1. 复核方法

- 身份/范围/manifest：`git cat-file` 读 committed blob 现算。
- 静态模型：用 pinned MuJoCo（`target/mjwarp-parallel/venv`，CPU-only，不 import warp）独立加载两个 XML，读取 geom/mesh/camera/light 数组，与被核 `logs/scene-check.json` 逐字段比对。
- dry-run：pinned python 离线复跑两组（`CUDA_VISIBLE_DEVICES=99` 证明不建 CUDA device），输出与被核 `logs/dryrun-sdf-{mesh,occluder}.json` 逐字段比对。
- 重算脚本：`recompute.py`（12 项 check → `recompute.json`，problems 1）。

## 2. 结果（12 项 check，11 过 / 1 需修）

| # | check | 结论 | 关键实测 |
| --- | --- | --- | --- |
| 1 | `script.identity` | ✓ | `scripts/capture_render_sdf.py` **20,380 B / `9acd2130…` / 427 行 / 非 ASCII 0 / CR 0 / `ast.parse` 通过** |
| 2 | `scene.identity` | ✓ | `scene.xml` **3,388 B / 59 行 / `376eb81c…`**；`scene-mixed.xml` **3,062 B / 55 行 / `7869cfae…`** |
| 3 | `scene.static_check_reproduced` | ✓ | 我独立加载：`ngeom 3/4`、`nlight 2`、`ncam 2`、`nq 0`、`nmesh 1`、`sdf_iterations 10`、`sdf_initpoints 40`；geom 表（floor plane / **sdf-cube type=8 dataid=0 contype=0 conaffinity=0** / side-sphere / occluder box）与 mesh（cube 8 顶点 / 12 面 / bounds ±0.35）与被核 `scene-check.json` **0 diff** |
| 4 | `dryrun.reproduced` | ✓ | 两组 dry-run 我复跑 **exit 0**；输出与被核日志**逐字段 0 diff**（`dry_run=true`、`repeat=2`、`legacy_manifest_entries=69`、env/版本/upstream 全同）；**未创建 CUDA device**（CUDA 掩码为 99 仍 exit 0），**`--out-dir` 未创建** |
| 5 | `legacy.manifest_69` | ✓ | `fixtures/render-bvh` 排除 `scene-sdf/`：文件系统 **69**、已跟踪 **69**；`scene-sdf/` 下只有 2 个 xml；`capture-sdf/` 不存在 |
| 6 | `scope.u2_commits` | ✓ | `828e37cc^..65a16430` 共 **17** 路径 = 脚本 1 + fixtures 2（仅 `scene-sdf/`）+ 证据 12 + `.normify` 2（含 scene-mixed 机械入库卡）；`src/`、`tests/` **0** 改动 |
| 7 | `evidence.manifests` | ✓ | authoring 证据 8 文件 / manifest 7 行 **7/7**；窗口材料 4 文件 / manifest 3 行 **3/3**；均不自登记；authoring 侧 3 个 UTF-16 日志（观察 O1） |
| 8 | `window.material_consistency` | ✓ | 窗口计划里的两条 GPU 命令与脚本 CLI **逐字**匹配；常规 `util < 40`、**无预批例外**、不加 `-OpenUtilLimit`；`USER DOUBLE-SIGN BLOCK` 未填（待用户签）；`expected-products` 的脚本 sha/scene sha/「no numeric verdict」全对 |
| 9 | `script.out_dir_self_checks` | ✗（P1） | 材料声明「`--out-dir` 必须未存在/非空即拒」为 hard self-check，脚本只做 `is_relative_to(SCENE_DIR)` 校验，随后 `out_dir.mkdir(parents=True, exist_ok=True)`；**无非空拒绝逻辑** |
| 10 | `product.independence` | ✓ | `src/`、`tests/`、`build.rs`、`.normify/gates.toml` 中对 `capture_render_sdf` 的引用 **0**；产物测试/门禁不依赖本脚本 |
| 11 | `card.validate_and_checks` | ✓ | 我重跑 `normify validate --fingerprints` = **0 错误 / 5 警告**；`normify check <主卡>` 全部通过（U2 未改 `src`，无指纹漂移） |
| 12 | `receipt.claims_and_boundaries` | ✓ | receipt `claims={numeric_acceptance:false, sdf_rendering_covered:false}`、`boundaries` 全 false（GPU/窗口/设备执行/src/tests）、`dry_run.exit_codes={sdf-mesh:0,sdf-occluder:0}`、`cuda_device_created=false`、`out_dir_written=false` |

逐项原始数字见 `recompute.json`；我复跑的 dry-run 输出在 `materials/dryrun-qa-*.json`（与证据逐字段相等），独立静态检查在 `materials/scene-check-qa.json`，门禁退出码在 `materials/gates-summary.txt`。

## 3. 需修 1（轻微，窗口材料/脚本身份口径）

**P1 `--out-dir` 非空拒绝：材料声明 vs 脚本实现不一致。**
`docs/development-evidence/render-bvh/sdf-capture-window-material/materials/window-plan.txt` 的「HARD SELF-CHECKS」和 `expected-products.json` 的 `self_checks.out_dir_must_be_empty=true` 都声明 out-dir 非空即拒绝，但 `scripts/capture_render_sdf.py` 只做：

```python
if not out_dir.is_relative_to(SCENE_DIR.resolve()):
    fail(...)
...
out_dir.mkdir(parents=True, exist_ok=True)   # 既有内容会被直接覆盖
```

U3 窗口若对同一 group 重跑（例如上一轮中途失败后重跑），旧产物会被**静默覆盖**，与「失败即停手、不静默」的口径不一致。
修法（二选一，都不需要重跑 GPU）：① 脚本加 `if out_dir.exists() and any(out_dir.iterdir()): fail("--out-dir 非空")`，并重跑两组 dry-run 刷新证据日志 sha；② 把窗口材料/`expected-products` 的措辞改成实际行为（明确「覆盖既有 out-dir 是允许的」），dry-run 证据不必动。

## 4. 观察 7（记录级接受）

- **O1 3 个证据 JSON 为 UTF-16LE + CRLF**：`dryrun-sdf-mesh.json`、`dryrun-sdf-occluder.json`、`scene-check.json`（PowerShell 重定向产物，带 CRLF）；与仓库既有捕获模式一致（`docs/development-evidence` 下 106 个 UTF-16 blob），建议今后统一 UTF-8/LF。内容我已分别按 UTF-16 解码后逐字段核对。
- **O2 派生 helper 不入库**：`target/sdf-u2/derive_sdf_capture.py`、`target/sdf-u2/scene_check.py` 只自称存在、未提交，派生过程无法从仓库独立复算。我用两条替代证据：① 直接审 427 行脚本；② 与 `scripts/capture_render_hfield.py` 做行级 diff = **390/427 行相同（91.6%）**、21 处替换块共 37 行，符合「只换场景/组名/task id/消息」的声明。
- **O3 `--repeat 1` 仍被 CLI 接受**：窗口计划要求 `--repeat 2`；若有人传 1，脚本会用 `first==second` 平凡为真地写 `two_captures_bitwise_identical=true`，看起来像「两跑一致」。建议对真实采集强制 2（或把 1 限定为 dry-run 专用）。
- **O4 「writes only under `--out-dir`」措辞不精确**：真实采集还会把 warp kernel cache 写到 `target/render-bvh/sdf-capture-cache`（不在 `--out-dir`、不进仓库、不影响 legacy 清单）；dry-run 不编译任何 kernel，故我复跑时确实零写入（git 工作树仅复核卡变化）。
- **O5 「repeat 2 否则不写任何东西」有细微偏差**：数组不等时确实零写入；但若数组相等而 npz 字节不等，脚本先把第一份 `rgb/depth/seg.npz` 写进 out-dir 再失败（可能残留半成品，不会产生错误数字声明）。
- **O6 `scene-check.json` 由未入库 helper 产出**：我用 pinned MuJoCo 独立重做静态检查，结论与被核日志 **0 diff**，故该证据可信；若后续想完全可复算，可把 `scene_check.py` 一并入库。
- **O7 U2 范围包含旁卡提交**：`65a16430` 之前还有 scene-mixed 机械入库卡（`2f4901be`…`65a16430`），我按 17 路径统计并在 scope check 里注明；fixtures 只新增 `scene-sdf/{scene.xml,scene-mixed.xml}`，既有 69 条参考零改动。

## 5. 边界与不宣称

- 本卡只做只读复核：零 GPU、零窗口、零设备执行、零参考写入；不改 `src/`、`tests/`、不改他人证据；dry-run 只读 fixtures + pinned 环境。
- 确认主卡表述：U2 只交付「场景 + 派生采集脚本 + 窗口材料」，不产生参考、不做数值验收；SDF 数值结论只能在 U4 验证窗（U1b host 解锁后、需用户双签）之后成立。
- 不宣称 SDF 渲染已覆盖、不宣称渲染数值验收通过、不宣称与上游逐位相同；本卡只核采集 authoring 的身份、可复现性与窗口材料一致性。

## 6. 限制

- dry-run 复跑覆盖的是「环境 + 计划 + legacy 清单」路径，不覆盖真实渲染（U3 才跑 GPU）；rgb/depth/seg 的真实数值与两跑字节相等只能在窗口内验证。
- 派生过程只能以「脚本内容 + 与骨架的行级 diff」间接核对（helper 未入库）。
- 静态模型检查只证明场景结构与声明一致，不证明渲染语义（那需要 U4 的设备侧对比）。
