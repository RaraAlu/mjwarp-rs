# G10@P5 G2 API 片：公开 re-export 与集成对照测试复位

任务：`2026-10-25-g10-p5-gjk-epa-api`（依赖 G2 `2026-10-25-g10-p5-gjk-epa`，HEAD 已验证）。

## 范围与变更
- `src/physics/mod.rs`：按既有模块风格 re-export GJK/EPA API（`CcdContact/CcdOptions/CcdOutput/CcdResult/ContactOptions/Geom/MeshView/ccd/ccd_pair/geom_type`）。
- `tests/gjk_epa.rs`：从 `target/g10-p5-g2/pending-api/` 搬回（121 行，≤500）；只读 fixtures/JSON，数值期望不重复维护（数值判据仍在 lib 内 `gjk/case_compare.rs` 的常驻门禁）。
- 证据：本目录。

## 验证
- `cargo fmt --check` / `clippy` / `clippy-cuda` / `cargo test` 全过（unit 418+；集成测试含 3 项）。
- 无数值口径变化；未开 GPU。

## 与 G2 的关系
G2 的主交付与数值判据见 `docs/development-evidence/g10-p5-gjk-epa/`（5 case 全部分量 ≤4 ULP vs 同配方上游 f32；`upstream_f32_within_4ulp` 常驻门禁）。本片只把公开 API 与集成测试复位，不改窄相实现。
