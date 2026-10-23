# 批次 4 证据索引：G09 交付 1 条；G10/M10-P10 顺延（裁决 A）

日期：2026-10-23。执行：Backend。批次范围：Chief 派单的 3 条 ready 路线图任务
（`g09-p5`、`g10-p5`、`m10-p10`）。

- `g09-p5` 凸体与网格：按 Chief 边界只做不依赖 BVH 的 plane-box 凸体原语切片，已交付。
- `g10-p5` 地形与 SDF：需高度场/SDF 资产管线与对应碰撞实现（当前 `src` 无
  hfield/SDF/mesh 支持）→ 停手，Chief **裁决 A 顺延**（保持 proposed，不宣称交付）。
- `m10-p10` Rust 回调 P10：P10 阶段语义（图重放/缓存失效/性能）依赖 forward/step
  调度或图集成（未实现），上游无可冻结语义 → 停手，Chief **裁决 A 顺延**
  （P7 契约 `b9cd236` 已交付；保持 proposed，不宣称交付）。

所有已交付内容遵守：全链 f32、`cargo fmt/clippy -D warnings/unit` 门禁、
path-limited 提交、Normify 范围与回执、证据/回执双落盘。

## 1. 已交付 1 条

| # | 任务 | 提交 | 验证核心数字 | 未验收边界（摘要） |
| --- | --- | --- | --- | --- |
| 1 | `g09-p5` 凸体与网格（plane-box 多接触原语） | 见同批提交 | 7 场景 15 接触：平放/偏航 45° 4 接触、棱 2、角 1、间隙 0、薄板 4、薄板间隙 0；产品 vs 冻结上游 max abs **5.96e-08**（≤1e-6）；原生 f64 `mj_collision` max abs **1.19e-07**（≤1e-5）；接触数逐世界 4/2/1/0/4/4/0 | box-box/capsule/cylinder/ellipsoid、mesh/convex GJK/EPA、CCD/multiccd、hfield/SDF、flex、显式 pair、EFC/约束、GPU |

证据目录：`docs/development-evidence/backlog-batch4/2026-10-11-roadmap-g09-p5/`
（README + authoring.json）；回执：
`docs/development-evidence/changes/2026-10-11-roadmap-g09-p5.json`。
g09-p5 属批次 4，证据与目录均在 `backlog-batch4/` 下。

## 2. 顺延 2 条（裁决 A）

| 任务 | 状态 | 阻塞项 | 恢复条件 |
| --- | --- | --- | --- |
| `g10-p5` 地形与 SDF 碰撞 | proposed，不 start | 高度场/SDF 资产管线与对应碰撞实现缺失（src 无 hfield/SDF/mesh；上游 hfield→CONVEX/GJK、SDF→`collision_sdf`） | 地形资产管线具备后重评，或与 r01 渲染计划合并 |
| `m10-p10` Rust 回调 P10 | proposed，不 start | P10 阶段语义（图重放/缓存失效/性能）依赖 forward/step 调度或图集成；上游无 callback-graph 可冻结语义 | 图重放能力具备后重评，或与 m10-p7 合并收口 |

两条均已登记 note；不宣称交付，不计入批次完成数。与 `r01-p8`、`a10-p11` 同属
「因引擎能力缺失而顺延」清单，最终汇总统一列出。

## 3. 批次门禁与状态

- `cargo fmt --check`：通过；`cargo clippy --all-targets -- -D warnings`：通过。
- `cargo test --locked`（gates.unit，全目标）：见同批门禁记录（G09 交付时实测）。
- `normify validate`：错误 0；每任务 gate/check/done 均在 path-limited 提交前
  通过 `check --staged`。
- 精度：plane-box 全链 f32；未改动既有容差/断言/ABI/排除清单；
  W13/Linux/g03-parity 未触碰。

## 4. 复现

```powershell
scripts/authoring/author-g09-plane-box.ps1
cargo test --locked --lib plane_box -- --nocapture
```
