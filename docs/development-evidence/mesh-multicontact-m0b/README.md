# G10@P5 M0b：mesh multicontact 32 数组 case bins（mesh-box / mesh-cylinder）

任务：`2026-10-26-mesh-multicontact-m0b-case-production`（in_progress，owner Backend；纯 CPU）。
性质：为 M1 fixture 产出 **32 数组** 的 host 输入 bins（含 M0a 9 个 mesh polygon 数组），
供 M2 设备实现/验证窗使用。**不改** G5 已 verified 的 `prepare_cases.py` 与旧 12 个 case bins；
不改 `src/`、不改 fixtures、不改判据/容差/参考。

## 1. 产出

| 路径 | 说明 |
| --- | --- |
| `docs/development-evidence/mesh-multicontact-m0b/scripts/prepare_mesh_cases.py` | 生产脚本（pinned venv，`wp.set_device("cpu")`，不建 CUDA context） |
| `docs/development-evidence/mesh-multicontact-m0b/cases/mesh-box/` | 32 个 `.bin` + `meta.txt` |
| `docs/development-evidence/mesh-multicontact-m0b/cases/mesh-cylinder/` | 32 个 `.bin` + `meta.txt` |
| `docs/development-evidence/mesh-multicontact-m0b/mesh-cases-manifest.json` | 逐数组 sha256/count + 逐位对照 + 自检规则 + provenance |
| `docs/development-evidence/mesh-multicontact-m0b/logs/` | 装配单测 / 真实校验探针 / 门禁摘要 |
| `docs/development-evidence/mesh-multicontact-m0b/scripts/sha256.json` | 脚本 sha |

### 1.1 32 数组清单（两 case 长度相同；逐 case sha 见 manifest）

| # | 数组 | dtype | count | 来源/含义 |
| --- | --- | --- | --- | --- |
| 1 | `geom_xpos` | f32 | 6 | CPU `mjw.kinematics`（nworld×ngeom×3） |
| 2 | `geom_xmat` | f32 | 18 | 同上（nworld×ngeom×9） |
| 3 | `geom_size` | f32 | 6 | 模型 geom_size |
| 4 | `geom_margin` | f32 | 2 | 模型 |
| 5 | `geom_gap` | f32 | 2 | 模型 |
| 6 | `geom_rbound` | f32 | 2 | 模型 |
| 7 | `geom_aabb` | f32 | 12 | 模型 |
| 8 | `geom_type` | i32 | 2 | 模型 |
| 9 | `geom_dataid` | i32 | 2 | 模型 |
| 10 | `mesh_vertadr` | i32 | 1 | 模型 |
| 11 | `mesh_vertnum` | i32 | 1 | 模型 |
| 12 | `mesh_graphadr` | i32 | 1 | 模型 |
| 13 | `mesh_octadr` | i32 | 1 | 模型 |
| 14 | `mesh_vert` | f32 | 24 | 模型（8 顶点×3） |
| 15 | `mesh_graph` | i32 | 98 | 模型 |
| 16 | `mesh_polynum` | i32 | 1 | **polygon 数（逐 mesh，nmesh=1）** |
| 17 | `mesh_polyadr` | i32 | 1 | **首个 polygon 地址（逐 mesh）** |
| 18 | `mesh_polynormal` | f32 | 18 | **法线（nmeshpoly=6 ×3）** |
| 19 | `mesh_polyvertadr` | i32 | 6 | **逐 polygon（nmeshpoly）** |
| 20 | `mesh_polyvertnum` | i32 | 6 | **逐 polygon（nmeshpoly）** |
| 21 | `mesh_polyvert` | i32 | 24 | **polygon 顶点索引（nmeshpolyvert）** |
| 22 | `mesh_polymapadr` | i32 | 8 | **逐 vertex（nmeshvert）** |
| 23 | `mesh_polymapnum` | i32 | 8 | **逐 vertex（nmeshvert）** |
| 24 | `mesh_polymap` | i32 | 24 | **vertex→polygon 映射（nmeshpolymap）** |
| 25–29 | `hfield_size/nrow/ncol/adr/data` | f32/i32 | 0 | 本 fixture 无 hfield |
| 30–32 | `oct_aabb/child/coeff` | f32/i32/f32 | 0 | 本 fixture 无 SDF octree |

`meta.txt` 记录 `nworld=1 / ngeom=2 / g1 / g2 / type1 / type2 / efc_width / tolerance /
gjk_iterations / epa_iterations / margin / gap / condim / friction / solref / solreffriction /
solimp / adhesion / pairid0/1 / sdf_* / naconmax / naccdmax / hfield_id / oct_root / noct`；
接触参数行取值来自同 fixture 的 M1 GPU 参考首行（同一 pair 全行相同）。

## 2. 同一来源（Chief 硬要求 ①）：逐位对照

脚本从 `fixtures/g10-p5-mesh-multicontact/<case>.xml`（M1 捕获同一 fixture）+ pinned
mujoco 3.15 / mujoco_warp 3.15 的 `capture` keyframe 生成，并逐项对照 M1 参考
`docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json`
（sha `c83baffe…`）：

| 对照项 | mesh-box | mesh-cylinder |
| --- | --- | --- |
| `xml_sha256`（现场重算 vs 参考） | ✅ 一致（`88b8d7af…`） | ✅ 一致（`d0b75f91…`） |
| `qpos_bits`（keyframe f32 位型 vs 参考） | ✅ 逐位一致 `[0,0,1041194025,1065353216,0,0,0]` | ✅ 逐位一致 `[0,0,1044549468,1065353216,0,0,0]` |
| `geom_xpos` 位型 vs 参考 | ✅ 逐位一致 | ✅ 逐位一致 |
| `geom_xmat` 位型 vs 参考 | ✅ 逐位一致 | ✅ 逐位一致 |
| `collision` count（CPU 复算 vs 参考） | ✅ 4/4 | ✅ 4/4 |

manifest 的 `cross_check` 与 `cases.<case>.xml_sha256/qpos_bits` 记录上述事实；
`tests/g10_p5_mesh_poly_case_assembly::m0b_same_source_matches_m1_reference` 在 CPU 上独立复算
（fixture xml sha 现场重算、bins 位型 vs 参考位型）并断言一致。

## 3. CPU 自检门（Chief 硬要求 ②）

### 3.1 生产脚本内 12 条规则（manifest `cpu_selfcheck.rules`，两 case 全 PASS）

| # | 规则 | mesh-box | mesh-cylinder |
| --- | --- | --- | --- |
| 1 | 9 个 polygon 数组齐全 | PASS | PASS |
| 2 | `polynum.len == polyadr.len`（逐 mesh，nmesh=1） | PASS | PASS |
| 3 | `polynormal.len % 3 == 0` | PASS | PASS |
| 4 | `polyvertadr/polyvertnum.len == nmeshpoly`（6） | PASS | PASS |
| 5 | `polymapadr.len == polymapnum.len`（nmeshvert=8） | PASS | PASS |
| 6 | 逐 mesh `polyadr+polynum <= nmeshpoly` | PASS | PASS |
| 7 | `sum(polynum) == nmeshpoly`（6 vs 6） | PASS | PASS |
| 8 | 逐 polygon `polyvertadr+polyvertnum <= len(polyvert)` | PASS | PASS |
| 9 | `sum(polyvertnum) == len(mesh_polyvert)`（24 vs 24） | PASS | PASS |
| 10 | 逐 vertex `polymapadr+polymapnum <= len(polymap)` | PASS | PASS |
| 11 | `sum(polymapnum) == len(mesh_polymap)`（24 vs 24） | PASS | PASS |
| 12 | 所有 adr/num 非负 | PASS | PASS |

### 3.2 真实 M0a `validate_inputs`（上传路径 CPU 校验）

`validate_inputs` 是 `pub(crate)`，集成测试不可直接调用。为了拿到「喂进修正后的 M0a 上传
路径」的真实结论，本片做了一次**临时探针**（追加到 `src/runtime/collision_gpu/validate_tests.rs`
的 `#[cfg(test)]` 探针、读真实 bins + `meta.txt` 装配 32 项 `CollisionGpuInputs` 后调用真实
`validate_inputs`），跑完立即 `git checkout --` 还原：

- **最终 `git diff` 为空、`src/` 零改动、探针未入库**（如实记录，不宣称可复跑）。
- 结果 exit 0（`logs/real-validate-probe.log`）：
  - `mesh-box` → `Ok(())`，sidecar=present，9 长度 `[1,1,18,6,6,24,8,8,24]`
  - `mesh-cylinder` → `Ok(())`，同上
  - `sphere-box-shallow`（旧 23 项 case）→ `Ok(())`，sidecar=empty，9 长度全 0，`nworld=2 ngeom=2`

### 3.3 常驻 CPU 测试（4 条，非 ignored）

`tests/g10_p5_mesh_poly_case_assembly.rs`（+`tests/support/mesh_poly_case_support.rs`）：

| 测试 | 断言 |
| --- | --- |
| `m0b_bins_match_manifest_sha_and_counts` | 32 个 bin 的 sha256/count 与 manifest 全同；9 项真实长度 `[1,1,18,6,6,24,8,8,24]` |
| `m0b_assembly_matches_m0a_sidecar_contract` | 装配后 9 slice 长度/指针与 bins 一致、sidecar=present、meta nworld/ngeom 正确、M0a 规则镜像通过 |
| `old_g5_cases_keep_23_bin_compat` | 10 个旧 G5 case：9 slice 全空、sidecar=empty、目录无 9 个 bin、声明集合 23 项全在 32 项口径内 |
| `m0b_same_source_matches_m1_reference` | xml sha 现场重算、qpos_bits/geom_xpos/geom_xmat 与 M1 参考逐位一致、manifest cross_check 全 true |

## 4. `INPUT_BINS` 23→32 兼容方案

- `tests/support/primitive_qa_support.rs`、`multicontact_qa_support.rs`：`INPUT_BINS` **23→32**
  （尾部追加 9 个 mesh_poly 名，前 23 项顺序不变）。
- **sha 守卫改为按 case 参考实际声明的 `input_bins` 集合执行**：旧参考只声明 23 项 → 只校验
  23 项；声明集合内的名字必须在 32 项口径内（未知名字直接 panic）。
- **装配兼容**：`read_optional_*` 读 9 个 mesh_poly bin；文件缺失 → 空 slice（sidecar 不构造），
  并显式断言「9 个同时存在或同时缺失」（部分存在 → panic，不静默半装配）。
- 旧 12 个 G5 case 的 bin 与 QA 参考**零改动**（`git status` 对这些路径为空）。

## 5. 改动/新增文件（全部 ≤500 行）

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `docs/development-evidence/mesh-multicontact-m0b/scripts/prepare_mesh_cases.py` | 331 | 生产脚本（新增，入库于证据目录） |
| `tests/support/primitive_qa_support.rs` | 300 | INPUT_BINS 32 + 声明集合守卫 + 可选 bin 装配 |
| `tests/support/multicontact_qa_support.rs` | 313 | 同上 |
| `tests/support/mesh_poly_case_support.rs` | 328 | 新增：32 项装配/`meta.txt` 解析/旧 case 兼容 helper |
| `tests/g10_p5_mesh_poly_case_assembly.rs` | 347 | 新增：4 条 CPU 测试 |

## 6. 门禁数字

见 `logs/gates-summary.txt`：

- 兼容性单测 `cargo test --locked --test g10_p5_mesh_poly_case_assembly` → 4 passed / 0 failed
- fmt `--check` / clippy `--all-targets -D warnings` / clippy-cuda → 全 exit 0
- unit `cargo test --locked` → **986 / 0 / 372**（M0a 修正片基线 982/0/372，**+4** 新测试）

## 7. 边界与未覆盖

- 本片**不建 CUDA context、不跑 GPU 内核**；真实 `validate_inputs` 用临时探针证明（已 revert），
  常驻覆盖为规则镜像 + 装配断言；`run()` 的发射级验证在 M2 验证窗执行。
- 不改 `src/`、不改 `prepare_cases.py`、不动旧 G5 case bins/参考、不改判据/容差。
- 不宣称 mesh 已支持；M2 需要在这些 bins 上实现 mesh multicontact 设备路径。
