# R08-S1 观察收口独立复核（QA）：O1–O8 处置 + 错误优先级 + 零数值回归

- 任务：`2026-10-28-render-bvh-r08-camera-obs-closure-check`（复核对象 `2026-10-28-render-bvh-r08-camera-obs-closure`）
- 提交：`f305842e`（13 文件；代码仅 `camera_output.rs` +10/−6、`src/render/context/tests.rs` +124）→ `d92797fc`（verified + 四门禁 runs + 模块指纹 1 个）；复核时 HEAD `d92797fc`
- 结论：**pass / 阻断 0 / 观察 2（2 建议）**；只读复核，未改代码/判据/参考/容差，未重渲染、未开 GPU

## 1. 总体判定

O2/O3/O7 三条「修」与 O1/O4/O5/O6 四条「接受」、O8 引用全部落地且可独立复核；`depth()` 的
`depth_scale` 校验确实前移到任何查找之前（`DepthScale` 最高优先级），并有单测锁定三种组合；
S1 既有参考对账数字用我自己的 harness 重跑**仍是 120000/360000/120000/360000/120000 位等/全等**，
未被 O2 改动破坏；行数棘轮全部符合（`camera_output.rs` **499**、`tests.rs` **379** 等），红线路径零改动；
四门禁我独立复跑 = **1017 passed / 0 failed / 382 ignored**（125 result 行），与 Backend 一致。

## 2. O2（修，代码）：depth_scale 错误优先级

- 代码顺序核验（`camera_output.rs::depth`）：先 `if !depth_scale.is_finite() || depth_scale <= 0.0 →
  DepthScale`，之后才 `depth_raw` / `resolution`；doc 写明「`depth_scale` 先于相机/通道/缓冲查找校验」。
- 单测 `depth_scale_validated_before_channel_and_index_lookups` 覆盖：
  通道禁用 + `scale ∈ {0, -1, ±inf, -0.0, NaN}` → 全部 `DepthScale`；**越界索引 + `scale=0` → `DepthScale`**
  （优先于 `CameraIndex`）；scale 合法后才落到 `ChannelDisabled`（相机 0、通道禁用）与
  `CameraIndex { index: 9 }`。README §3 写死错误优先级段。
- 我的独立行为核对：把 S1 复核 harness 的错误优先级断言按**新**语义更新后，其余 98 项检查仍全过；
  未改的旧断言正好在「disabled + scale=0」处报 `DepthScale`——与收口意图一致（旧行为改变为预期）。

## 3. O3（修，措辞）

- 模块 doc、`clamp_unit` doc、README §2、receipt `depth_clamp_semantics` 四处一致改为：
  `-0.0 → +0.0`、`NaN → 0` 是**产品实现行为**（对齐 CUDA `fminf/fmaxf` 顺序），**不是**上游语义保证；
  夹具值域有限非负、**无 `-0`/NaN 样本** → 边界上游逐位对账**未覆盖**。
- 未发现仍在把该边界写成上游保证的残留表述。

## 4. O7（修，测试）

- `src/render/context/tests.rs`（**379** 行）新增 2 条：错误优先级 1 条 + `depth_boundary_values_are_product_clamp_not_cuda_equivalence` 1 条。
- 边界单测只断言**产品侧**行为（`-0.0/NaN/±inf → +0/1`、`MIN_POSITIVE → 1.0`、`f32::MAX → 次正规正数`），
  并显式记录「Rust `f32::clamp` 传播 NaN、保留 `-0.0`，与产品不同；这些差异没有上游 CUDA 逐位参考 →
  记为未覆盖，不宣称等价」。未把等价写成结论。

## 5. 接受项与 O8

- 收口收据 `dispositions`：O1 accept（`camera_id` 窄化留后续）、O2 fix-code、O3 fix-wording、
  O4 accept（adr 前缀和加固不做）、O5 accept（`#[path]` 维持）、O6 accept（`PixelOverflow` 防御保留）、
  O7 fix-tests、O8 reference（正确 id `2026-10-28-repo-fixture-eol-attributes`）。
- 收口收据 `/task_id = 2026-10-28-render-bvh-r08-camera-obs-closure`，193 行；README §9 与 receipt `closure`
  段逐条对应。

## 6. 不变量与棘轮（我独立复核）

| 文件 | 行数 | 备注 |
| --- | --- | --- |
| `src/render/context/camera_output.rs` | **499** | ⚠ 距 500 只剩 **1 行**；后续任何加行必须先拆文件 |
| `src/render/context/tests.rs` | **379** | 新增 2 条 depth 单测 |
| `src/render/context.rs` | 426 | 未动 |
| `tests/render_camera_output.rs` | 493 | 未动（避棘轮） |
| `src/render/kernel/launch.rs` | 500 | 未动 |
| `src/render/kernel/megakernel.rs` | 498 | 未动 |
| `src/render/kernel/cast.rs` | 494 | 未动 |
| `src/model/render.rs` | 500 | 未动 |

`f305842e^..d92797fc` 对 `src/model/render.rs`、kernel cuda/launch/output/cast/megakernel、`context.rs`、
`fixtures/`、`Cargo*`、`native/`、`include/` 的 diff 为空；代码改动只有上述 2 个文件。

## 7. S1 数字不回归（我自己重算）

用我 S1 复核时自写的独立 harness（python/numpy 期望值 + Rust 逐位比较）在收口最终树上重跑：
**packed 120000/120000、unpacked 360000/360000、depth raw 120000/120000、depth scaled（1.0/0.1/100.0）
360000/360000、seg 120000/120000** 全部逐位相等（45 项检查 0 失败）；`INV_255` 与四形态判据未受影响。
旧的错误优先级断言按预期返回 `DepthScale`（O2 的**有意**语义变更，不是回归）。

## 8. 门禁独立复跑（收口最终树）

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0 |
| `cargo test --locked` | exit 0：**1017 passed / 0 failed / 382 ignored**（125 result 行） |
| `cargo test --locked --test render_camera_output` | 4 passed / 0 failed |

与 Backend 归档（`logs/closure/*`、`.normify/runs/…` unit 1017/382/125；`directed-depth-priority.log`
2 passed）一致；`normify validate --fingerprints` 错误 0（5 条既有告警），`normify check`（收口任务）通过。

## 9. 观察（2 条，均建议级）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| F1 | 建议 | README §8 表头写「四门禁（`normify gate` 绑定最终树；同树命令输出存档在 `logs/`）」，表内仍是 S1 时的 **1010/376、日志 2132 行**；而 §9 说收口最终树 unit **1017/382**。同一 README 出现两个"最终树"口径，容易误读 | **修**：§8 标注「S1 提交时（历史）」或指向 §9/`.normify/runs/2026-10-28-render-bvh-r08-camera-obs-closure/`；不涉及数字 |
| F2 | 建议 | README §9 写「门禁与自检数字见收据 `gates` / `self_checks`」，但收口收据的键是 **`gate`**（单数，内含 fmt/clippy/clippy-cuda/unit），无 `gates` | **修**：改引用为 `gate`（或补 `gates` 别名）；数字本身在 `gate` 内齐全 |

## 10. 边界与不宣称

只读复核：未重渲染、未开 GPU、未重跑窗口、未改代码/判据/参考/容差。本片只核观察收口处置与
S1 数字不回归，不替代用户/Chief 的窗口签字，也不对后续方向作验收宣称。
