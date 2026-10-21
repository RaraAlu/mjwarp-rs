# 常驻运动学超长源码拆分

任务：`2026-10-21-refactor-split-resident`。
基线提交：`360718f`。

## 结果

`src/physics/resident.rs` 由 6878 行拆为 315 行模块根加 26 个子模块。
子模块位于 `src/physics/resident/`：产品 16 个、测试 10 个。
全部文件不超过 500 行，最大 `data.rs` 483 行。
`report.json` 由 `build-report.ps1` 扫描实际文件生成。

## 布局偏差

本任务保留 `src/physics/resident.rs` 作为 315 行模块根。
它替代 `src/physics/resident/mod.rs` 形式，语义等价。
原因是不改动 `tests/support` 的 `include_str!` 源码锚点。
`tests/` 在本任务中零改动。
t4 证据快照测试的源码锚点语义迁移另立后继任务。

## 验证

`normify gate` 四项全部通过：fmt、clippy、clippy-cuda、unit。
unit 门禁 329 项通过、0 失败。
`normify validate` 为 0 错误。
`report.json` 的 `over_limit_files` 为 0。
