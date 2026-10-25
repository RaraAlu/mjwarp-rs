# 宿主 CCD 溢出位误报修正（EPA `FLOAT_MAX` 时不得多置 `OverflowType.CCD`）

- 任务：`2026-10-25-g10-p5-host-ccd-overflow-bit-fix`（P5，G6 前置；纯 CPU，无 GPU 窗口）
- 状态：修正 + 验证完成，待提交（提交由 Arch 串行执行）
- 起点 HEAD：`622094fb260d4180747cfa59e19a7f0016a9a4e8`
- 上游 pinned：`71da24d956378a87a703b6e1442b13aec0c4ac29`
- 设备侧同一语义偏差已由 `2026-10-25-g10-p5-ccd-overflow-bit-fix`（`99c91469`）修正；本片只改宿主，不改 `src/physics/collision/cuda/`，也不改任何已 verified 证据数值。

## 结论

宿主 `gjk/ccd.rs`（`ccd()` 内部）与 `gjk/contact.rs`（`ccd_pair()`）不再在 EPA 返回 `FLOAT_MAX` 时置 `OverflowType.CCD(1<<4)`，只透传 EPA 阶段的 `EPA_HORIZON(1<<8)`；`CCD` 位只由**真实容量截断**（`ccd_slot >= naccdmax`，上游 `eval_ccd_write_contact` 语义）置。

强制 horizon（等价容量 2）回归：

| 入口 | 修前（旧代码） | 修后（本片） | 断言 |
| --- | --- | --- | --- |
| `gjk/ccd.rs::ccd()`（内部 `ccd_internal`） | 272 = `EPA_HORIZON\|CCD` | **256 = `EPA_HORIZON`** | `overflow==256`、`&CCD==0`、`!=272`、`dist==FLOAT_MAX`、`ncontact==0`、`multiccd_idx==-1` |
| `gjk/contact.rs::ccd_pair()` | 272 | **256** | 同上 + `contacts.is_empty()` |

修前 272 由被删除代码语义直接给出（旧 `if dist == FLOAT_MAX { overflow \|= overflow_type::CCD }`）；测试除断言 256 外还显式断言 `overflow != 272`，防止回归。

防矫枉过正（容量门仍按上游置 `CCD`）：

| 场景 | `overflow` | 接触 |
| --- | --- | --- |
| 真实截断：`ccd_slot=1 >= naccdmax=1`（深穿透但槽位不足） | **16 = `CCD`**（不含 `EPA_HORIZON`，不进入 EPA） | 空（上游同样直接返回不写接触） |
| 容量足够 + 强制 horizon 容量 2 | 256 = `EPA_HORIZON`（不含 16） | 空（`dist=FLOAT_MAX`） |
| 容量足够 + 正常穿透（box-box 0.05 穿透） | 0 | 正常产生 |

## 改动（4 文件，+211/−16，`src/` 内）

| 文件 | 改动 |
| --- | --- |
| `gjk/ccd.rs` | 删除 `ccd_internal` 中 `let mut overflow = ...; if dist == FLOAT_MAX { overflow \|= CCD }`（旧 L225-228），改为 `overflow: epa_phase_result.overflow`；为可测性抽取 `epa_phase_with_horizon` / `ccd_internal_with_horizon`（产品入口固定 `MJ_MAX_EPAHORIZON`，行为不变） |
| `gjk/contact.rs` | 删除 `ccd_pair` 中 `if dist == 1.0e30_f32 { overflow \|= CCD }`（旧 L128-130）；新增 `CcdCapacity { ccd_slot, naccdmax }` 与 `ccd_pair_with_capacity(...)`（产品入口 `ccd_pair` 传 `None`，**默认行为不变**）：`needs_epa` 且 `ccd_slot >= naccdmax` → 只置 `CCD` 并直接返回（对齐上游容量门） |
| `gjk/mod.rs` | `Polytope::with_capacity` 委托到新增 `with_capacity_and_horizon`（产品固定 `MJ_MAX_EPAHORIZON=24`；回归测试用等价容量 2 强制 `EPA_HORIZON`） |
| `gjk/tests.rs`（+95 行） | 新增 2 个回归测试：`host_ccd_forced_horizon_reports_only_epa_horizon`、`host_ccd_capacity_gate_still_sets_ccd_bit` |

`OverflowType` 位定义（`gjk/mod.rs`：`CCD=1<<4`、`HFIELD=1<<5`、`EPA_HORIZON=1<<8`）未改；产品 ABI/结构体未改；`cuda/` 目录未改。

## 上游依据（与设备片同源，pinned 71da24d）

- `collision_gjk.py:2663-2667`：`_epa(... idx == -1)` → `return FLOAT_MAX, 0, ...`（不置额外位）。
- `collision_gjk.py:1423-1430` / `1446-1453`：horizon 容量不足（`_add_edge` 返回 -1）→ 只 `wp.atomic_or(overflow_out, worldid, OverflowType.EPA_HORIZON)`。
- `collision_gjk.py:2712-2729`：`ccd()` 直接返回 `epa_phase(...)`，无 `FLOAT_MAX → CCD`。
- `collision_convex.py:860-870`（`eval_ccd_write_contact`）：仅 `ccdid >= naccdmax_in` 时 `atomic_or(OverflowType.CCD)` 并 `return`——这是 `CCD` 位唯一来源。
- `types.py:166`：`CCD = 1<<4`、`EPA_HORIZON = 1<<8`。

## 回归（纯 CPU）

| 判据 | 命令 | 结果 |
| --- | --- | --- |
| G2 五 case（`upstream_f32_within_4ulp`） | `cargo test --locked --lib physics::collision::gjk::case_evidence` | 3 passed / 0 failed / 1 ignored（探索项）；`upstream_f32_within_4ulp` + `our_contacts_match_upstream_f32_within_4ulp` ok |
| G3 `sdf-box` 80/80 位级 | `cargo test --locked --test g10_p5_sdf_relmat` | 4 passed / 0 failed（`sdf_box_g3_regression_remains_bitwise` ok） |
| G4 hfield 三 case | `cargo test --locked --lib physics::collision::hfield::tests` | 3 passed / 0 failed（`hfield_host_matches_upstream_f32_within_4ulp` ok） |
| G5 宿主 `ellipsoid-box` 三方对照 | `cargo test --locked --features cuda-probe --test g10_p5_gpu_kernels ellipsoid_box_host_ccd_pair_vs_cuda_reference -- --ignored --test-threads=1` | 1 passed；hits=2、dist 2 ULP、pos 7.451e-9、frame 8.193e-7（vs CUDA fixture）；vs 上游 CPU fixture：dist 0、pos 7.451e-9、frame 3.838e-7——与修正前一致（本片未动数值路径） |
| 库单元 | `cargo test --locked --lib` | **440 passed / 0 failed / 2 ignored**（原 438 + 本片 2 个新测试） |
| 本片新增测试 | `cargo test --locked --lib physics::collision::gjk::tests::host_ccd` | 2 passed / 0 failed |

门禁（本片 checks：fmt/clippy/clippy-cuda/unit）：

- `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` / `cargo clippy --all-targets --features cuda-probe -- -D warnings`：全 0。
- `normify gate 2026-10-25-g10-p5-host-ccd-overflow-bit-fix`：见 receipt `gate` 字段（结果 runs 见 `.normify/runs/2026-10-25-g10-p5-host-ccd-overflow-bit-fix/`）。

## 复现

```powershell
cargo test --locked --lib physics::collision::gjk::tests::host_ccd -- --nocapture
cargo test --locked --lib physics::collision::gjk::case_evidence
cargo test --locked --test g10_p5_sdf_relmat
cargo test --locked --lib physics::collision::hfield::tests
cargo test --locked --features cuda-probe --test g10_p5_gpu_kernels `
  ellipsoid_box_host_ccd_pair_vs_cuda_reference -- --ignored --test-threads=1
```

## 证据文件

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文件（结论/改动/上游依据/位型对照/回归） |
| `test-output.txt` | 上述命令的真实输出摘要（新测试、G2/G3/G4/G5 回归、clippy/fmt） |
| `receipt.json` | 任务回执（scope、位型对照、回归数值、gate、unmet） |

## 未覆盖 / unmet

- 本片只修宿主，未改设备（`cuda/`，前片 `99c91469` 已修）；本片不改任何 GPU 证据与 G5 设备数值。
- 宿主 `ccd_pair` 的**产品入口默认不模拟** `nccd`/`naccdmax` 计数（调用方持有计数器，与设备 `contact.cu` 的 `g5_claim_ccd_slot` 同语义）；本片新增的可选容量门 `ccd_pair_with_capacity` 供需要容量语义的调用方/G6 映射使用，产品默认路径行为不变。
- 强制 horizon 用**等价容量 2** 构造（上游 `horizon` 数组长度即容量，不改上游源码）；产品默认 `MJ_MAX_EPAHORIZON=24` 下未构造真实复杂 mesh 的 >24 边 horizon 用例。
- 未与上游 GPU 直接对照（本片为宿主 CPU 路径，判据沿用 G2/G5 已验收口径）；只本机 Windows x64 实测，未跑 Linux/T4。
- 不声称「精度达标」「上游完全等价」「原生放行」；判据范围仅上述 overflow 位与 G2/G3/G4/G5 宿主回归。
