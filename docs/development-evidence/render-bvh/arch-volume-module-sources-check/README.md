# 架构树 24 文件模块源补登 · QA 独立复核（P1）

任务卡：`2026-10-29-arch-volume-module-sources-check`（priority 55，Owner QA）
被核对象：`2026-10-28-arch-volume-module-sources`（**verified**）；复核范围**只有 P1 两提交**：
基线 `f86b8575` → `0dab0cd6`（启动卡）→ `f38b83fd`（24 文件登记 + 证据）。
口径：`git diff f86b8575..f38b83fd`（不使用工作区全量 status；复核期 Arch 在仓内有 P2/P4 在途文件，已避免读取）。
**只读**：未改模块树/代码/证据。

---

## 1. 结论

**verdict：pass / 阻断 0 / 观察 6。**

24 个文件的模块归属、前后计数、validate 结果、impact 可用性、零代码改动、证据卫生，我都自己重算/重跑并与主卡数字对照，全部相符。唯一实质问题是**主卡对 `structure/leaf-too-coarse` 33→34 的归因写错了模块**（写成 `mjwarp.model.core`，实测跨阈值的是 `mjwarp.io.render`），见 O1（建议级、只改文字，不动数字）。

| 复核面 | 我的独立结果 |
| --- | --- |
| 24 文件清单与归属 | 24/24 存在且被跟踪；**基线无一是已登记源**；5 模块增量 **render +13、model.core +3、io.core +5、io.render +2、math +1**，与主卡逐项相同，无删除、无重复 |
| 计数 | 66 模块/59 叶子不变；登记条目 1108→1132、不同路径 1091→1115；五个模块 40→53 / 2→5 / 11→16 / 1→3 / 2→3（合计 56→80） |
| validate | 我重跑 `normify validate --fingerprints`：**错误 0、警告 5**，与 `logs/validate-final.txt` 逐行相同（含 `leaf-too-coarse 34`） |
| impact | 我不止查 5 处，**24/24 全部跑过**，都返回模块与源码计数（render 53 / model.core 5 / io.core 16 / io.render 3 / math 3），无「没有模块对应」 |
| 零代码改动 | `git diff f86b8575..f38b83fd -- src tests fixtures include native Cargo.toml Cargo.lock` **为空**；整段 diff 只有 `.normify/` + 本证据目录 |
| committed 内容 | HEAD = `f38b83fd`；`.normify/modules/` 与主卡证据目录 `git diff HEAD` **为空**；主卡 `normify check` 通过 |
| 证据 | 12 文件（README + receipt + 10 logs），全部 LF、无 BOM；receipt `/task_id` 正确；禁词 0 |
| 未登记余量 | 我重算：跟踪的 `src/include/native/build.rs` = **823**，基线未登记 **178** → P1 后 **154**（src 118 = physics 91 + runtime 27；include 36 = 全部 `include/warp-headers/*.h`；native 0）——与主卡/backlog 卡一致 |

---

## 2. 24 文件与归属（复核点 1）

主卡 README §3 的清单我逐条核过（明细见 `materials/ownership-audit.txt`），重点几项：

- `src/model/collision_inputs.rs` → **mjwarp.model.core**：文件头写明「只定义字段规范与计数校验；原生读取位于 `io::collision_inputs`」；与 Chief 2026-10-29 裁定一致；基线 `model.core` 已有 `mod.rs`/`collision.rs` 同层.
- `include/mjwarp_render_probe.h` + `native/model_probe_render.cpp` → **mjwarp.io.render**：`src/io/render.rs`（基线已在 io.render）头注释「原生ABI见 `include/mjwarp_render_probe.h`；字段规范见 `model::render`」；`.cpp` 说明被 `native/model_probe.cpp` 同 TU 引入且不改 G01/collision/flex/sparse.
- `include/mjwarp_collision_inputs_probe.h` + `native/model_probe_collision_inputs.cpp` → **mjwarp.io.core**：同族 `src/io/collision.rs`、`include/mjwarp_collision_probe.h`、`native/model_probe_collision.cpp` 基线已在 io.core；`src/io/collision_inputs.rs` 头注释指向该 ABI.
- `src/io/state/tests.rs` → io.core：基线已有 `src/io/state/extended/tests.rs` 同型先例.
- `src/math/fma/tests.rs` → math、`src/render/context/tests.rs` → render：均有基线已登记的父模块 `fma.rs`/`context.rs` 先例.
- 13 个 render 文件：6 个 kernel `.rs` 与 6 个 `.cu` 有同目录/同模块先例（`kernel/mod.rs`、`kernel/launch.rs`、`kernel/cuda/render_shadow.cu` 等基线已登记）.
- 24 个路径在基线 1091 个不同登记路径里**零命中**（不是重复登记/抢占）；登记后 5 个模块无重复项；5 个模块 md 的 diff 只有 `fingerprint` 一行 + `[[source]]` 块，`id`/`uid`/`apis`/`tags` 未动.
- 24 文件 LF 行数：主卡 README §3 表、`logs/line-counts.txt`、现盘字节计数**三者逐项一致（0 差异）**.

---

## 3. validate 与警告漂移（复核点 2）

| 时点 | 结果 | 我的核对 |
| --- | --- | --- |
| 基线（`logs/validate-before.txt`） | 错误 0 / 警告 5（leaf **33**） | 与历史旁证一致：R02-S1 复核时（P1 之前）我自己的 validate 日志同样是 33 |
| 登记后、收口前（`logs/validate-after.txt`） | 错误 0 / 警告 6（leaf **34** + `evidence/fingerprint-drift 5 个模块指纹已过期`） | 属 `done` 前中间态（5 个模块 md 改了源但指纹未刷新），收口后消失——与说明一致 |
| 收口后（`logs/validate-final.txt`）与**我当前重跑** | 错误 0 / 警告 5（leaf **34**），其余 4 条与基线逐字相同 | **逐行一致**（`logs/validate-now.txt`） |

**警告 33→34 的真实出处（O1）**：我按基线/P1 后的模块树重建叶子口径（59 个叶子）：

| 口径 | 基线 | P1 后 | 唯一翻转 |
| --- | --- | --- | --- |
| **叶子源数 ≥ 2** | **33** | **34** | **`mjwarp.io.render`（1→3 源）** |
| 源数 ≥ 2 且总行 > 220 | 33 | 34 | `mjwarp.io.render` |
| 源数 ≥ 2 或最大单文件 > 220 | 51 | 51 | — |
| 源数 > 1 或最大单文件 > 220 | 51 | 51 | — |
| 源数 ≥ 3 | 26 | 29 | io.render + math + model.core |

⇒ 与观测 (33, 34) 相容的口径里，**唯一翻转模块是 `mjwarp.io.render`**；`mjwarp.model.core` 在基线就是 **2 源 / 479 行**，已计入 33，本片 2→5、479→1253 行**不改变**它的 coarse 状态。主卡 README §5 与 `receipt.validate.warning_delta_explained` 把它写成 model.core 越阈值，属**归因错误**（不影响任何数字/错误数）。明细与交叉证据见 `materials/leaf-too-coarse-analysis.txt`。

---

## 4. impact 可用性（复核点 3）

主卡抽的 5 处我全部重跑并与证据日志对照：

| 路径 | 命中模块（源码/API） | 与主卡一致 |
| --- | --- | --- |
| `src/render/kernel/cast.rs` | mjwarp.render（53 / 11） | ✓ |
| `src/model/volume_input.rs` | mjwarp.model.core（5 / 28） | ✓ |
| `src/io/volume.rs` | mjwarp.io.core（16 / 17） | ✓ |
| `src/io/render.rs` | mjwarp.io.render（3 / 0） | ✓ |
| `src/math/fma/tests.rs` | mjwarp.math（3 / 32） | ✓ |

我另外把**全部 24 个路径**都跑了一遍 `normify impact`（`logs/impact-24.txt`）：24/24 返回模块与计数，无「没有模块对应」。证据日志是任务 `in_progress` 时采集的，其尾行「未关闭任务」计数比现在多 1（含本卡自身）——时间性差异（O4）。

---

## 5. 零代码改动与 committed 一致性（复核点 4、5）

- `git diff f86b8575..f38b83fd`：**只有** `.normify/`（3 个 changes + 5 个模块 md + 4 个 runs）与 `docs/development-evidence/render-bvh/arch-volume-module-sources/`（12 文件）。
- 产品面零改动：`git diff f86b8575..f38b83fd -- src tests fixtures include native Cargo.toml Cargo.lock` **空**（这 24 个文件被登记但内容未改）。
- HEAD = `f38b83fd`；`git diff HEAD -- .normify/modules/ <主卡证据目录>` **空**（committed 与现盘一致）；主卡 `normify check` 通过（4 个 gate 检查 + receipt `/task_id`）。

---

## 6. 证据卫生（复核点 5）

- 主卡证据 12 个文件：README、receipt、10 个 logs；全部 `CR=0`、无 NUL、无 BOM（我逐 blob 检查）。
- receipt `/task_id` = `2026-10-28-arch-volume-module-sources` ✓；`kind`/baseline/scope/module_counts/validate/impact_samples 与我的重算一致。
- 证据目录无 manifest（复核卡写的是「manifest（若有）」）→ 可接受；committed 一致性我用 `git diff HEAD` 对 modules + 证据目录验证（O6）。
- 禁词：证据未出现复核卡 3 个 forbid 词（我按码点构造词表扫描；结果为空）。

---

## 7. 观察（6 条，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议 | **33→34 的归因写错模块**：主卡 README §5 与 receipt 写「model.core 2→5 越过叶子规模阈值」；实测按与 (33,34) 相容的叶子口径，**唯一翻转是 `mjwarp.io.render`（1→3 源）**，model.core 基线已 2 源/479 行、本就计入 33 | 修文字（不动数字）：把「哪个模块新变 coarse」改为 io.render；若按「新变 coarse」排期，需用基线的 coarse 状态判断。建议同时把「叶子源数 ≥2」口径写进架构文档 |
| O2 | 记录 | README §2 与 receipt 的「登记源 1091 条」实为 **1091 个不同路径**；按 `[[source]]` 条目数是 **1108**（跨模块共享路径 17 条重复）。178/154 差集用的是去重集合，因此数字自洽 | 建议写成「1091 个不同路径（1108 条登记条目）」，避免与条目数混淆 |
| O3 | 记录 | README §3 中 `.cu` 行的依据「同目录 `scene.cu`/`mesh.cu`/`render_shadow.cu` 已登记」：只有 `render_shadow.cu` 与新增文件同目录（`src/render/kernel/cuda/`），`scene.cu`/`mesh.cu` 在 `src/render/bvh/cuda/`（同模块、不同目录） | 措辞微调为「同模块已登记（`kernel/cuda/render_shadow.cu`；bvh/cuda/scene.cu、mesh.cu）」；归属结论不变 |
| O4 | 记录 | 证据 `logs/impact-*.txt` 在任务 `in_progress` 时采集，尾行「未关闭任务」含本卡（in_progress），计数比现在多 1（如 cast 日志显示 +9，我核对时 +8） | 时间性差异，非缺陷；如要求可重采 |
| O5 | 记录 | `structure/leaf-too-coarse` 的精确规则在仓库内无文档（normify 二进制规则），只能从数据重建 | 建议在架构规范补该口径（叶子源数阈值），便于 154 文件收口时预算警告漂移 |
| O6 | 记录 | 主卡证据无 manifest；复核卡要求「manifest（若有）」 | 可接受：现行做法以 committed blob 为准，我用 `git diff HEAD` 复核了 modules + 证据目录 |

---

## 8. 边界

1. 只复核 P1 两提交（`f86b8575..f38b83fd`）；Arch 在仓内的 P2/P4 在途文件不在范围，我未读取/未改动（diff 与 `git diff HEAD -- <scope>` 均限定在 P1 路径）。
2. 未重跑任何构建/门禁（本片只改架构树与证据；主卡 4 个 gate 的 runs 记录随 `normify check` 通过）；我自跑的是 `normify validate --fingerprints` 与 `normify impact`（只读命令）。
3. `leaf-too-coarse` 规则是从观测结果重建的（候选口径里只有一族与 (33,34) 同时吻合），不是读 normify 源码得到的。
4. 未改模块树/代码/证据；本卡不产渲染或性能结论。

---

## 9. 复现命令（仓库根目录）

```text
target\render-bvh-precheck\venv312\Scripts\python.exe -B docs\development-evidence\render-bvh\arch-volume-module-sources-check\recompute.py
target\render-bvh-precheck\venv312\Scripts\python.exe -B docs\development-evidence\render-bvh\arch-volume-module-sources-check\leaf-recompute.py
normify validate --fingerprints
normify impact src/render/kernel/cast.rs
git diff f86b8575..f38b83fd --stat
```

（脚本 `problems` 为空，见 `recompute.json`；validate 现跑、24 路径 impact、口徑重建输出在 `logs/` 与 `materials/`。）

---

## 10. 收口

- 本卡证据提交：本目录；被核两提交 `0dab0cd6` + `f38b83fd` 保持不动（只读复核）。
- O1 是纯文字更正（改「谁新变 coarse」），不影响 P1 的登记结论、计数与 validate 结果。
