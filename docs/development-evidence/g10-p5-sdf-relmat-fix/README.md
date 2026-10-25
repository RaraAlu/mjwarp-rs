# G10@P5 G3 修正片：产品相对旋转约定改为上游 `transpose(rot1)*rot2` + fma，并补非单位旋转 SDF 验收

- 任务：`2026-10-25-g10-p5-sdf-relmat-fix`（纯 CPU，不需要 GPU）
- 上游：`mujoco_warp 3.15` @ `71da24d956378a87a703b6e1442b13aec0c4ac29`
- 收口依据：4 个非单位旋转场景 × 2 world × 40 接触 **80/80 位级 0 ULP**（每场景），
  `sdf-box` 80/80 全字段位级无回归；gate `fmt/clippy/clippy-cuda/unit` 全过。

## 1. 根因：两处缺陷都在 `rel_mat` 的位型

上游 `collision_sdf.py:724`：`params.rel_mat = wp.transpose(rot1) * rot2`（行优先
`rel[row][col] = Σ_k rot1[k][row] * rot2[k][col]` = `aᵀb`）。

1. **转置**：产品旧 `sdf/descent.rs::relative_matrix` 按「逐列 `transpose_mul_mat3_vec(a,
   mul_mat3_vec(b, e_col))` 再写 `column*3+i`」构造，在行优先语义下得到的是 `bᵀa`
   （上游结果的转置）。G3 `sdf-box` 的 geom1 为单位旋转、SDF 侧也无旋转（两侧 identity），
   故 `aᵀb = bᵀa`，G3 未暴露。
2. **累加方式**：上游 `warp/native/mat.h mul(mat,mat)` 用 `muladd`（fmaf）逐项累加
   `sum = fma(a[i][k], b[k][j], sum)`（k 升序）；旧实现是普通乘加。只修转置时，
   `sdf-rot-both` 仍留残差（见 §3 QA 数字）。
   **不能用 `f32::mul_add`**：本机 dev 档（opt-level 0）实测不融合；`relmat.rs` 用 f64
   精确承载 f32 乘积 + 一次和、再单次舍入回 f32（与正确舍入 fma 位级一致，QA rel_mat 10/10）。

**为什么不需要改 `rel_pos`/梯度回投**：逐句核对上游与产品一致——
`rel_pos = wp.transpose(rot1) * (pos2 - pos1)` ↔ 产品 `transpose_mul_mat3_vec(rot1, …)`；
`x1 = rel_mat * x + rel_pos` ↔ 产品 `mul_mat3_vec(rel_mat, x)` + `add3`；
`grad1 = wp.transpose(rel_mat) * grad1` ↔ 产品 `transpose_mul_mat3_vec(rel_mat, grad1)`；
`pos = rot2 * x + pos2`、`n = rot2 * n` ↔ 产品同式。故唯一需修的是 `rel_mat` 的位型。

## 2. 改动（产品 + 测试 + authoring）

| 路径 | 改动 |
| --- | --- |
| `src/physics/collision/sdf/descent.rs` | `relative_matrix` 改为委托 `relmat::upstream_relative_matrix`（不再自建 `bᵀa`）；新增 `use super::super::relmat;`；文档写明上游语义 |
| `src/physics/collision/hfield/driver.rs` | 删除 test-only 同名 helper（旧 `bᵀa` + 普通乘加），不再保留第二份实现 |
| `src/physics/collision/hfield/tests.rs` | G4 测试改用 `crate::physics::collision::relmat::upstream_relative_matrix` |
| `src/physics/mod.rs` | 公开 re-export `SdfContact/SdfPair/SdfShape/SdfType/VolumeData/sdf_pair_contacts`（集成测试走公开 API；`sdf/mod.rs` 早有「公开导出供后续片与测试使用」注释） |
| `tests/g10_p5_sdf_relmat.rs` | 新集成测试（4 项）：fixture 结构与 provenance、4 场景 40/40 位级、`sdf-box` 80/80 位级回归、参考 `rel_mat` 编码上游约定（aᵀb 位级 + 与 bᵀa 可区分） |
| `scripts/authoring/g10-p5/capture_sdf_rotation_references.py` | 离线 authoring：只读 pinned `mujoco_warp`/`warp`，warp CPU 后端两次独立捕获（位等），不读产品源码；产品测试不跑 Python |
| `fixtures/g10-p5/sdf-rotation/sdf-rotation-cpu.json` | 新增夹具：5 场景（含 ident 对照 + 4 验收场景）× 2 world × 40 接触，含 xml/qpos/位姿/rel_mat/模型数组，provenance 完整 |
| `.normify/changes/2026-10-25-g10-p5-sdf-host-r2.json` | G3 加注（`normify fix … --reason`，见 §5），不改 G3 数值结论 |

**复用而非复制**：`relmat::upstream_relative_matrix`（G5 交付，含 fma 链与 warp 位型单测）
是唯一实现；`sdf/descent.rs` 与 G4 测试均委托它，仓库内不再有第二个 `relative_matrix` 实现体。

## 3. 判据与结果（同配方上游 f32 CPU 参考）

参考：`fixtures/g10-p5/sdf-rotation/sdf-rotation-cpu.json`
sha256 `add06bda50263d6b36454e4f33814a963375793d40245c3c902aef826df55ab5`
（119,578 B）。provenance：`device=cpu`、`warp 1.15.0`/`mujoco_warp 3.15`/`mujoco 3.15`、
`authoring_only=true`、`reads_product_tree=false`、脚本 sha256
`f7135f3123b9de73902fb449ee1f3ebdba348f922f0b910542feaf5f2e8617f8`；
每场景两次捕获逐位相等；octree 数组与 `fixtures/g10-p5/sdf-f32/oct_*` 逐字节一致
（`oct_aabb 62eb3750…`/`oct_child 7e28b042…`/`oct_coeff 3466f78e…`）。
**独立捕获交叉核对**：本片捕获与 QA `qa-sdf-rotation-cases.json`（sha256
`d7eeed76…616e`）的 output/rel_mat/qpos **全部逐位相等**（`capture-report.json`
`qa_reference_cross_check.all_output_relmat_qpos_bitwise_equal = true`）。

**修正前/仅修转置/本片修正后（QA 实测，`qa-sdf-rotation-report.json` sha256 `afc6be10…b546e7`）**

| 场景 | 修正前（旧 `bᵀa`+普通乘加）dist / pos / frame worst | 仅修转置（QA corrected） | 本片（aᵀb+fma） |
| --- | --- | --- | --- |
| `sdf-rot-ident`（对照） | 0 / 0 / 0 | 40/40 位级 | **80/80 位级** |
| `sdf-rot-sdf` | 0.02352738 / 0.314468365 / 1.69082731 | 40/40 位级 | **80/80 位级** |
| `sdf-rot-box` | 0.037304197 / 0.324073061 / 1.968242764 | 40/40 位级 | **80/80 位级** |
| `sdf-rot-both` | 0.019736722 / 0.293807119 / 1.86429441 | world0 25/40、world1 1/40 位级（dist 197 / 256,600 ULP；pos abs 2.61e-8 / 0.061881；frame abs 1.25e-6 / 1.051156） | **80/80 位级** |
| `sdf-rot-rz-rx` | 0.058964811 / 0.317684852 / 1.933387578 | 40/40 位级 | **80/80 位级** |

「只修转置不修 fma」在 `sdf-rot-both` 留下的残差与 task 记录一致（25/40、1/40），
故必须同时修 fma 累加；本片两处一起修后全场景 80/80 位级（2 world × 40）、
dist/pos/frame 最大绝对差均为 0。

**G3 回归**：`sdf-box` 80/80 接触全字段位级不变
（lib 判据输出：`dist 位等 80/80 max 0 ULP | pos max 0.00 ULP@尺度 | frame 位等分量 720/720 max abs 0e0`）。

**rel_mat 位型自证**：测试用不依赖产品的 test-local f64-fma 参考计算 `aᵀb`，
与夹具 `rel_mat_warp` **10/10 world 位级相等**；其中 8/10 world 的 `aᵀb` 与其转置不同
（证明夹具确实编码上游约定、能区分旧 `bᵀa`）。

复现命令（产品测试不启动 Python）：

```
cargo test --locked --test g10_p5_sdf_relmat -- --nocapture
cargo test --locked --lib sdf_box_host_matches_upstream_f32_within_4ulp -- --nocapture
cargo test --locked --lib physics::collision::relmat
```

输出留档：`docs/development-evidence/g10-p5-sdf-relmat-fix/test-output.txt`。

## 4. G3 加注（不改 G3 数值结论）

`normify fix 2026-10-25-g10-p5-sdf-host-r2 --reason "…"` 已追加修正记录（记录内文：

> 上游约定修正已由 `2026-10-25-g10-p5-sdf-relmat-fix` 完成：`sdf/descent.rs` 相对旋转改为
> 上游 `transpose(rot1)*rot2`（行优先）+ fma 链累加；G3 `sdf-box` 结论不受影响（geom1 为单位旋转），
> 但 G3 时相对旋转约定曾为转置；证据与 80/80 位级回归见
> `docs/development-evidence/g10-p5-sdf-relmat-fix/`，G3 既有数值结论不改。

`.normify/changes/2026-10-25-g10-p5-sdf-host-r2.json` 的 `fixes` 数组新增该条，
原 `revision`/`status`/数值证据未改。

## 5. 门禁与复现

| 项 | 结果 |
| --- | --- |
| `cargo fmt --check` | 0 |
| `cargo clippy --all-targets -- -D warnings` | 0 |
| `cargo clippy --all-targets --features cuda-probe -- -D warnings` | 0 |
| `cargo test --locked` 全目标 | 0（含新集成测试 4/4） |
| `normify gate 2026-10-25-g10-p5-sdf-relmat-fix` | 见 receipt `verification.gate` |
| `normify gate 2026-10-25-g10-p5-sdf-host-r2`（G3 fix 记录要求） | 见 receipt `verification.g3_gate` |

authoring 复现（离线，不占 GPU，产品测试不依赖 Python）：

```
target/mjwarp-parallel/venv/Scripts/python.exe -B \
    scripts/authoring/g10-p5/capture_sdf_rotation_references.py
```

## 6. 未覆盖项（unmet，不得据此扩大声明）

- 本片只修 **host SDF 路径**；设备侧 `cuda/common.cu` 的 rel_mat 已在 G5 按上游语义修正
  （`29894efb` + `01690e53`），本片未重复改设备侧、也未在 GPU 上复测。
- multicontact / primitive narrowphase / hfield 产品路径不涉及本片行为（`hfield/driver.rs`
  的 helper 仅测试使用，已删并改测试；G4 产品路径不受影响）。
- 未跑 GPU、Linux/T4、性能基准（归 G6）；GPU 结果不外推跨硬件。
- 不声称「精度达标」「上游完全等价」「原生放行」；本片判据范围仅上述 host SDF 场景。
