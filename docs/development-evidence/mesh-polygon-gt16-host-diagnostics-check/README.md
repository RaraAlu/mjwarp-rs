# C 卡独立复核：mesh polygon 宿主超界诊断（QA，只读）

任务：`2026-11-03-mesh-polygon-gt16-host-diagnostics-check`
复核对象：`2026-11-03-mesh-polygon-gt16-host-diagnostics`（实现提交 `ecf78951`，14 文件）
复核方式：只读 + 独立执行 CPU 测试与门禁；未开 GPU 窗口、未改产品/测试/判据/夹具。

## 0. Verdict

**PASS**（0 阻断 / 0 需修 / 2 观察）。

## 1. 六条 acceptance 的独立核对

| # | 验收项 | 独立证据 | 结论 |
|---|---|---|---|
| ① | 范围只含宿主代码与证据 | `git show --name-only ecf78951`：4 个 `src/runtime/collision_gpu*` + 卡 json + 模块指纹 + 4 门禁回执 + 4 证据文件；`.cu` 0、`tests/` 0；`git diff ecf78951^ ecf78951 -- src/runtime/collision_gpu/validate* .../run*` 为 0 行 | pass |
| ② | 错误载荷与 Display | `error.rs`：`MeshMulticontactCapacity` 载荷含 `dimension/actual/limit/reject_path`；`Display` 打印 `{dimension.label()} actual=… limit=…` + `拒绝路径：{reject_path}` | pass |
| ③ | 产品组装点唯一 | `grep -r "CollisionGpuInputs {" src/`：非测试构造点只有 `inputs_from_snapshot.rs:156`；`inputs_from_snapshot(` 的调用方是 `product_entry::prepare_inputs`；直连 `run` 走探针路径、未改 | pass |
| ④ | 6 条测试真实执行 | 独立运行 `cargo test --locked --features cuda-probe inputs_from_snapshot::tests` → **6 passed / 0 failed**；6 个用例名：`product_entry_rejects_fan_cone_24_on_both_dimensions`、`product_entry_rejects_face_only`、`product_entry_rejects_degree_only`、`product_entry_accepts_face_20_degree_10_boundary`、`product_entry_accepts_without_mesh_sidecar`、`host_limits_match_device_macros`；模块整体 `runtime::collision_gpu` → 48 passed / 0 failed / 1 ignored | pass |
| ⑤ | 门禁与证据自洽 | 实现卡四门禁回执 `fmt/clippy/clippy-cuda` 均 passed、unit **1258/0**；证据 `manifest.txt` 3 条对磁盘 sha256+bytes **0 mismatch**；`receipt.task_id` 正确；本复核卡四门禁见 `.normify/runs/2026-11-03-mesh-polygon-gt16-host-diagnostics-check/` | pass |
| ⑥ | 禁词与口径 | 证据面（README/facts/receipt/manifest）对「U8 已通过 / mesh 容量缺口已清 / 上游等价 / 精度达标」**0 命中**；README §6 如实写探针边界（U7/U8 设备 count 仍可能为 1、不计入本片验收面） | pass |

## 2. 独立执行的命令（可复现）

```text
git show --name-only ecf78951
git diff ecf78951^ ecf78951 -- src/runtime/collision_gpu/validate* src/runtime/collision_gpu/run*
grep -rn "CollisionGpuInputs {" src/
cargo test --locked --features cuda-probe inputs_from_snapshot::tests        # 6 passed / 0 failed
cargo test --locked --features cuda-probe runtime::collision_gpu             # 48 passed / 0 failed / 1 ignored
cargo test --locked --features cuda-probe --no-run                           # EXIT=0
normify gate 2026-11-03-mesh-polygon-gt16-host-diagnostics-check
```

## 3. 观察（不阻断）

1. 「`inputs_from_snapshot(` 的调用点仅 `product_entry::prepare_inputs` 一处」当前由 README 人工核查确认，建议后续加一条源码级回归断言（与 shadow 卡的裸 `diff_depth` 白名单同型）；已记 backlog，不在已 verified 的实现卡上硬改。
2. 本片只做宿主超界诊断，**U7/U8 未关闭**；证据与措辞保持现状，不作任何容量缺口已清/上游等价的结论。

## 4. 边界

- 只读复核：未改 `src/`、`tests/`、`.cu`、判据、容差、参考、夹具。
- 未开 GPU 窗口、未跑任何 GPU 用例；本片验收面本身是宿主诊断 + CPU 单测。
- 本复核不替代上游 U7/U8 设备行为的判定；探针路径仍按设备真实行为记录。
