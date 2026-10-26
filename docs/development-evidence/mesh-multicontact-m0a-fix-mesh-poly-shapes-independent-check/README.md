# G10@P5 M0a 修正片（mesh polygon 形状语义）QA 独立复核

- 任务：`2026-10-26-mesh-multicontact-m0a-fix-mesh-poly-shapes-independent-check`（owner QA，纯 CPU）。
- 被复核片：`2026-10-26-mesh-multicontact-m0a-fix-mesh-poly-shapes`（verified）。
- 交付提交：`b50d93a6`（src 修正 + M0a 证据补注 + 门禁 runs）、`3a31e318`（证据目录归档，主任务 files 漏登记）。
- 复核时主树 HEAD：`ac01d86d`（M0b 已提交）；`b50d93a6..HEAD` 对 `src/tests/fixtures/Cargo.*` **零 diff**
  （M0b 只动 tests/ + 证据 + scripts），故 `ac01d86d` 的代码树 = 修正提交代码树。
- 结论：**pass（阻断 0 / 观察 7）**；未 commit、未 done。

## 0. 结论与判定依据

1. **形状规则与 pinned 一致**：按 pinned `types.py`（rev `71da24d9`，`source_hashes[_src/types.py]=7970e04d…`）
   L1329-1337 / L1864-1872 独立核 9 个数组 dim 与归属；再用 pinned `mujoco 3.15` mjModel 独立 dump 真实 cube
   两 case 的 9 数组，确认交付 `validate_tests::CubeMeshPoly` 与 mjModel **逐值/逐 bit 相同**（含 `-0.0`）。
2. **13 条规则逐条对代码复核**：交付审计表 6 修正 / 7 保持的判定我独立同意（见 §2 与 `qa-rule-audit.json`）。
3. **红/绿对照由我实际编译复现**（不是读交付日志）：在一次干净 detached worktree 里把 `abi.rs` 换回修正前版本
   → `accepts_real_cube_mesh_poly_shapes` **exit 101**（错误消息与交付红日志逐字相同）；换回修正后版本
   → `validate_tests` **7 passed / exit 0**。另用机制化抽取的旧/新函数做 21 case 探针矩阵（§4）。
4. **零改动核过**：修正提交只改 3 个 src 文件（2 处注释 + `validate_mesh_poly` 重写 + 单测），
   ABI 504 / sidecar 72B / entries 6 / `DEVICE_SOURCES` 27 / device 源 / fixtures / 判据全部未动；
   冒烟指纹 `238969 / ec2e7acd… / 3786520 / entries 6` 我独立重跑复现。
5. **门禁**：见 §6（交付 gates runs 与原始日志我独立核对；我自己的 fmt/clippy/clippy-cuda/unit 在
   干净 worktree `fe8956f4` 上跑；**主树当时被 M2 在途 WIP 占据，fmt 必失败，详见 §6.3**）。

## 1. 复核对象与绑定

| 项 | 值 |
| --- | --- |
| 修正提交 | `b50d93a6`（10 文件：3 src + M0a README/receipt + 4 gate run JSON） |
| 证据归档提交 | `3a31e318`（fix 证据目录；`git diff 3a31e318..HEAD -- <fix 证据目录>` 为空 → 归档后零改写） |
| 复核时主树 HEAD | `ac01d86d`；`git diff b50d93a6..HEAD -- src tests fixtures Cargo.toml Cargo.lock` 为空 |
| 门禁用干净树 | detached worktree `target/qa-m0afix/wt-head` @ `fe8956f4`（代码树 = `b50d93a6`） |
| pinned 上游源码 | `docs/evidence/d04-complex-physics/authoring-runs/run-5/installed-source/_src/`（rev `71da24d9`，见 manifest） |
| 本机 | Windows / `rustc 1.99.0 (b940084d7 2026-09-28)` / 未开 GPU 窗口、未建 CUDA context |

## 2. pinned 形状独立核（不照抄交付表）

`generated/manifest.json` 给出 `upstream_revision=71da24d956378a87a703b6e1442b13aec0c4ac29` 且
`source_hashes["_src/types.py"] = 7970e04d40dd15b33171dd1558c369cf45478b6ffdb31d8a805aec93702ec717`，
与我读的 `_src/types.py` 实测 sha 相同 → 行号/dim 可绑定到 pinned rev。

- L1329 `mesh_polynum: (nmesh,)`、L1330 `mesh_polyadr: (nmesh,)`（docstring），L1864-1865 `array("nmesh", int)`
- L1331 `mesh_polynormal: (nmeshpoly, 3)`，L1866 `array("nmeshpoly", wp.vec3)`
- L1332-1333 `mesh_polyvertadr / mesh_polyvertnum: (nmeshpoly,)`，L1867-1868 同
- L1334 `mesh_polyvert: (nmeshpolyvert,)`，L1869 同
- L1335-1336 `mesh_polymapadr / mesh_polymapnum: (nmeshvert,)`，L1870-1871 同
- L1337 `mesh_polymap: (nmeshpolymap,)`，L1872 同

设备侧用法（pinned `collision_gjk.py` L2245-2262 取 `geom.mesh_polynormal/polyvertadr/polyvertnum/polyvert/
polymapadr/polymapnum/polymap`；L2278-2310 `_mesh_normals(..., geom.mesh_polyadr, polynormal, polymapadr,
polymapnum, polymap ...)`）确认 `mesh_polyadr` 逐 mesh、`polyvert*` 逐 polygon、`polymap*` 逐 vertex。

**真实 mjModel 独立 dump**（`scripts/dump_cube_poly.py`，pinned mujoco 3.15，CPU，两 fixture）：

| 数组 | 实体数 | 真实 slice 长度 |
| --- | --- | --- |
| polynum / polyadr | nmesh=1 | 1 / 1 |
| polynormal | nmeshpoly=6 | **18**（6×3 f32） |
| polyvertadr / polyvertnum | nmeshpoly=6 | 6 / 6 |
| polyvert | nmeshpolyvert=24 | 24 |
| polymapadr / polymapnum | nmeshvert=8 | 8 / 8 |
| polymap | nmeshpolymap=24 | 24 |

`polynum=[6]`、`polyadr=[0]`、`polyvertadr=[0,4,8,12,16,20]`、`polyvertnum=[4]*6`、
`polymapadr=[0,3,6,…,21]`、`polymapnum=[3]*8`；三个 adr 均等于对应 num 的 cumsum（我逐项验证 True）。
交付 `validate_tests::CubeMeshPoly` 与上述真实值**逐值相同**，`polynormal` 的 `-0.0` 也逐 bit 相同
（`0x80000000`，见 `qa-pinned-shapes.json`）。两个 fixture（mesh-box / mesh-cylinder）9 数组一致。

## 3. 13 条规则逐条审计（独立判定）

代码位置：`src/runtime/collision_gpu/abi.rs::validate_mesh_poly`（`b50d93a6` 起 L270-392；修正前 L244-337）。
逐条见 `qa-rule-audit.json`，摘要：

- **保持**：#1 全空→Ok(null sidecar)、#2 部分空→`InvalidArgument`、#4 `polynormal %3`、
  #7 `sum(polynum)==nmeshpoly`、#8 逐 mesh 区间（+负值）。
- **修正**：#3 `polynum/polyadr` 只按 `(nmesh,)` 等长（旧规则把 6 个数组一起要求 `==nmesh`）；
  #5 `polyvertadr/polyvertnum` 长度 `==nmeshpoly`；#6 `polymapadr/polymapnum` 等长（=nmeshvert）；
  #9/#10 区间循环域从 `0..nmesh` 改为 `0..nmeshpoly` / `0..nmeshvert`；#11 负值检查域随循环域修正。
- **新增**：#12 `sum(polyvertnum)==len(polyvert)`、#13 `sum(polymapnum)==len(polymap)`。

与交付审计表结论一致（6 修正 / 7 保持）；未发现交付表漏判或错判。强一致 #12/#13 的依据是
mjModel 的 `polyvert/polymap` 是各 `*num` 的累加布局（我 dump 的两 case 全等、cumsum 校验 True），
types.py 只给 dim、未显式写 sum 约束（观察 O5）。

## 4. 红/绿对照（我实际编译复现）

干净 detached worktree `wt-head` @ `fe8956f4`（`src/tests/fixtures` 与 `b50d93a6` 零 diff），
把 `abi.rs` 替换为 `git show b50d93a6~1` 的旧文件后跑同一测试：

```text
# 红（旧规则，旧 abi.rs + 修正后 validate_tests.rs）
cargo test --locked --lib runtime::collision_gpu::validate_tests::accepts_real_cube_mesh_poly_shapes -- --nocapture
→ exit 101；0 passed / 1 failed；843 filtered
  left: Err(InvalidArgument("mesh_poly 逐 mesh 数组长度须一致（polynum/polyadr/polyvertadr/polyvertnum/polymapadr/polymapnum）"))
  right: Ok(())
# 绿（修正后 abi.rs）
cargo test --locked --lib runtime::collision_gpu::validate_tests -- --nocapture
→ exit 0；7 passed / 0 failed；837 filtered
```

红日志的错误消息与交付 `logs/red-old-rule.log` **逐字相同**；绿与交付 `logs/green-new-rule.log` 的
7 passed/837 filtered 一致（`logs/red-*`、`logs/green-*`）。

**探针矩阵**（`scripts/make_probe.py` + `scripts/probe_template.rs`，机制化抽取旧/新函数**原文**，
仅旧函数名加 `_old` 后缀；数组用我自己的 mjModel dump）：21 case，见 `qa-rule-matrix.json` /
`logs/probe-matrix.log`。关键结果：

- `real_cube`：旧 = `Err(...逐 mesh 数组长度须一致...)`，新 = `Ok(())`；
- `two_meshes_valid`（nmesh=2、polynum=[3,3]）：旧 = `Err(同一长度错)`，新 = `Ok(())` → 规则不是只对 cube 拟合；
- 13 类负例在新规则下分别命中各自错误消息（含 6 条负值分支：polynum/polyadr/polyvertadr/polyvertnum/
  polymapadr/polymapnum）；
- `partial_sidecar` 两版都 `Err(输入不完整)`；`all_empty` 两版都 `Ok(())`。

## 5. 负例覆盖核对

交付 `validate_tests.rs` 的三条测试覆盖 13 类（§4 的 ①-⑬ 与 README §4 一一对应，我逐条读了断言）：
`polynormal%3`、`polynum/polyadr` 长度、`polyvertadr` 长度、`polymapadr/num` 长度、逐 mesh/polygon/vertex
区间越界、`polynum=-1`、`sum(polynum)`、`sum(polyvertnum)`、`sum(polymapnum)`、部分空、全空→Ok；
且 `rejects_inconsistent_mesh_poly_sidecar` 末尾包含「真实 cube 数据应通过」反证断言（不会把正常数据也拒掉）。
`CubeMeshPoly` 是用真实 mjModel 值构造，不是为迁就规则而写的「自洽假形状」（§2 已逐 bit 核对）。

## 6. 零改动核对

| 不变量 | 我的核对方式 | 结果 |
| --- | --- | --- |
| 改动范围 | `git show --name-only b50d93a6`：3 src + M0a README/receipt + 4 gate run JSON；`git diff --stat b50d93a6~1..HEAD -- src tests fixtures Cargo.*` = 3 文件（253+/64−） | 仅 CPU 校验/注释/单测 |
| 结构体/ABI | abi.rs diff 中 `CollisionArgs`/`MjwarpMeshPolyArgs` 无字段增删（只加逐字段注释）；`cargo test --lib abi_` → 504 布局、sidecar 布局、六入口 by-value 三条断言 **passed** | ABI 504 / sidecar 72B 不变 |
| entries | `product.cu` 内 `extern "C" __global__ void` 计数 = **6**；冒烟 `entries=6` | 6→6 |
| 冒烟指纹 | 我重跑 `cargo test --locked --features cuda-probe --lib runtime::collision_gpu::nvrtc_smoke -- --include-ignored` → `ptx_bytes=3786520 entries=6 source_bytes=238969 source_sha256=ec2e7acd…` | 与 M0a 完全一致 |
| `DEVICE_SOURCES` | `source.rs` 声明 `[(&str, &str); 27]`；该文件 fix 提交零 diff | 27 段不变 |
| device 源 | `git diff b50d93a6~1..b50d93a6 -- src/physics/collision/cuda` 为空；`b50d93a6..HEAD`（含 M0b）也为空 | 零改动 |
| fixtures / 判据 / 参考 | `fixtures/`、`tests/`、`Cargo.toml`、`Cargo.lock` 在 fix 提交零 diff | 零改动 |
| `src/physics/mod.rs` | fix 提交零 diff | 零改动 |

## 7. 门禁数字

### 7.1 交付侧 gates runs（我读了 commit 内 `.normify/runs/<fix 任务>/`）

fmt/clippy/clippy-cuda `exit 0`；unit `passed=982 failed=0 ignored=372`（`at 15:31:48Z`，`log_sha256 cca95016…`）。
交付 `logs/gates-summary.txt` 声明的两份原始日志 sha 我实测一致：`unit.log b5d5034f…`、
`all-features.log c52f5241…`；按其内容独立聚合（不读交付摘要）：

- unit：**982 passed / 0 failed / 372 ignored / 115 binaries**
- all-features：**1031 passed / 0 failed / 430 ignored / 115 binaries**

### 7.2 我的独立复跑

- 主树（`ac01d86d`）all-features：**1035 passed / 0 failed / 430 ignored / 116 binaries**；
  与交付 1031 逐测试名对比，新增恰好 4 条且全为 M0b 新测试（`m0b_bins_match_manifest_sha_and_counts`、
  `m0b_assembly_matches_m0a_sidecar_contract`、`m0b_same_source_matches_m1_reference`、
  `old_g5_cases_keep_23_bin_compat`），ignored 同为 430 → 交付 1031 可复现（1031+4=1035）。
- 干净 worktree `fe8956f4`（代码树 = 修正提交）：fmt exit 0、clippy exit 0、clippy-cuda exit 0、
  unit **982 passed / 0 failed / 372 ignored**（见 §7.3 环境说明）。

### 7.3 门禁树的选择（主树被 M2 在途 WIP 占据）

复核期间主树并发有 M2 在途未提交改动（`src/physics/collision/cuda/multicontact_mesh.cu`（新）、
`product.cu`、`multicontact*.cu`、`source.rs`、`collision_gpu.rs`、`*_tests.rs` 等 12+1 文件）。
其中 `collision_gpu.rs` 新增的 `validate_collision_inputs` 未格式化 → 主树 `cargo fmt --check` **失败**，
且这些行不属于本修正片（`git log -S` 无提交、`git diff` 显示为未提交 WIP）。因此：
**fmt/clippy/clippy-cuda/unit 全部改在干净 detached worktree `fe8956f4`（`src/tests/fixtures` 与
`b50d93a6` 零 diff）上执行**；worktree 初始满足 autocrlf 的 CRLF 工作副本使 2 个既有 EOL 敏感测试
（`sdf_rotation_fixture_is_well_formed`、`contract::frozen_three_d_package_contract_is_self_consistent`）
报「既有 EOL 失败」，我把 4498 个干净 tracked 文件按主树字节对齐（不动 11 个主树 dirty 文件）后即
982/0/372（与 B1 基线同口径的 2 个既有 EOL 失败问题，非本片回归）。门禁 run 记录见
`.normify/runs/2026-10-26-mesh-multicontact-m0a-fix-mesh-poly-shapes-independent-check/`。

## 8. M0a 证据补注核对

- `mesh-multicontact-m0a/README.md` §4：旧「9 个非空数组逐 mesh 等长」表述被 blockquote
  「**已被修正片 `…-fix-mesh-poly-shapes` 更新**」标注作废，并写明真实三段形状、影响面与冒烟指纹；
  原文未被静默改写（我按 `git show b50d93a6 -- <M0a README>` 核 diff）。
- `mesh-multicontact-m0a/receipt.json::empty_sidecar_semantics`：新增 `shape_semantics`（真实三段形状 +
  旧表述作废）与 `shape_correction_task`（task_id/evidence/impact），`validation` 去掉过期 `:241`，
  `unit_tests` 补 2 条新测试；旧字段未被删改。
- 证据归档提交 `3a31e318` 与 HEAD 的 fix 证据目录零 diff；fix 证据 `receipt.json::task_id` 指向主任务 ✓。

## 9. 观察（按级别，均不阻断）

- **O1 一般**：交付 README §1 把「9 数组长度」写成 `(1,1,6,6,6,24,8,8,24)`、receipt
  `real_cube_shapes.lengths` 同值；其中第 3 位 `6` 是 `nmeshpoly`（实体数），**`mesh_polynormal` 的真实
  slice 长度是 18**（6×3）。真实 mjModel 长度是 `(1,1,18,6,6,24,8,8,24)`。M0b bins（polynormal.bin 72B=18 f32）
  与 M0b commit 说明用的都是 18，故未造成下游错误；但作为「数组长度」表述会误导造 fixture 的人，建议改数或改称谓。
- **O2 一般**：交付 README §5 与 receipt `changes.files[].lines` 的行数在其标注的口径
  （`Measure-Object -Line`）下也对不上：`types.rs` 94（实测非空行 115、原始 118）、`validate_tests.rs`
  374（实测 395、原始 422），`abi.rs` 378 正确。原因应是 Windows PowerShell 5.1 无 `-Encoding UTF8` 读中文
  时 ANSI 解码导致多行被合并计数；同一组数字也写进了 M0a 补注，建议重算（或直接写原始行数）。
- **O3 建议**：fix 证据 9 个文件里 5 个日志是 UTF-8 **带 BOM**（`clippy.log`、`clippy-cuda.log`、
  `nvrtc-smoke.log`、`red-old-rule.log`、`green-new-rule.log`）；团队证据口径是 UTF-8 无 BOM，建议统一（内容无损）。
- **O4 建议**：单测的负值分支只测了 `polynum=-1`；`polyadr/polyvertadr/polyvertnum/polymapadr/polymapnum`
  的负值分支（代码里各有一条错误消息）没有单测。我用机制化抽取探针逐条命中（`qa-rule-matrix.json`），
  行为正确但建议补进交付单测，避免后续重构漏掉。
- **O5 建议**：`sum(polyvertnum)==len(polyvert)` / `sum(polymapnum)==len(polymap)`（强一致）是 mjModel
  累加布局的性质，types.py 的 dim 并未显式要求；对真实 mjModel 成立（我 dump 的两 case 全等），但对
  非 mjModel 的手工稀疏布局会误拒。建议注释写明依据（`*adr == cumsum(*num)`），并在 M2 输入侧确认
  只有 mjModel 来源会走这条校验。
- **O6 建议**：校验未覆盖 `polyvert` / `polymap` **元素值域**（polyvert 应落在该 mesh 顶点数内、polymap 应
  < nmeshpoly），也未把 `nmeshvert` 与既有 `mesh_vertnum/vertadr` 交叉核对。M0a 时代同样如此，属防御性缺口、
  非本片回归；device 侧消费的是真实模型数组，实际风险低，建议列入 M2/M3 的输入可信边界。
- **O7 建议**：交付红/绿日志本身不含 exit code（红日志只有 panic + `test result: FAILED`）；exit 101/0 是我
  按实际编译复现（§4）与 cargo 返回值得出，建议以后在 gates-summary 里直接记 exit。

## 10. 边界 / 未覆盖

- 未开 GPU 窗口、未建 CUDA context、未重跑 device 对照；null / 非 null sidecar 的**发射级**行为仍留 M2 验证窗。
- 本片不宣称 mesh 已支持；M0b/M2 的结论不在本复核范围。
- 未覆盖：`polynormal` 数值合法性（NaN/归一化）、mesh 数组与 model 其他数组的交叉一致性（O6）。
- 门禁的「干净树」替代方案已写明（§7.3）；主树一旦回到干净状态，同一组 gate 命令可直接复跑。

## 11. 复现

```text
# pinned 数组 dump（pinned venv, CPU）
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-m0afix/dump_cube_poly.py

# 旧/新规则探针（机制化抽取 abi.rs 函数原文）
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-m0afix/make_probe.py
rustc --edition 2024 -O target/qa-m0afix/probe.rs -o target/qa-m0afix/probe.exe
target/qa-m0afix/probe.exe

# 绿（主树或干净树均可）
cargo test --locked --lib runtime::collision_gpu::validate_tests -- --nocapture

# 冒烟（不建 context，只需 NVRTC dll）
cargo test --locked --features cuda-probe --lib runtime::collision_gpu::nvrtc_smoke -- --include-ignored --nocapture
```

证据脚本副本在 `scripts/`（`dump_cube_poly.py`、`make_probe.py`、`probe_template.rs`、`align_worktree_bytes.py`），
逐一 sha 记录在 `manifest.json`。
