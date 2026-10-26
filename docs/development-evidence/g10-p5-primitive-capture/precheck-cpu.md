# B0 fixture CPU 预检（窗口外，2026-10-26）

## 解释器与版本（pinned，与捕获 provenance 一致）
- 解释器：`D:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Scripts\python.exe`（python 3.12.13）
- mujoco 3.15.0 / warp 1.15.0（`wp.set_device("cpu")`）/ mujoco_warp 3.15.0 / numpy
- 性质：**预检只证 fixture 合法且能产生有效 primitive 接触**；参考仍来自窗口内 GPU 捕获，本预检不上判据，**参考尚不是判据**。
- 脚本（临时、不入库）：`target/g10-p5-primitive-precheck/precheck_cpu.py`；原始输出 `precheck-cpu.json`。

## 逐 case 结果（CPU 后端 kinematics+collision；ngeom=2、NATIVECCD 置位）

| case | ngeom | nativeccd disabled | nacon（预期=实际） | 前 2 条 dist（预检量级，非参考） |
| --- | --- | --- | --- | --- |
| plane-sphere | 2 | True | 1 | -1.000e-002 |
| plane-capsule | 2 | True | 1 | -1.000e-002 |
| plane-ellipsoid | 2 | True | 1 | -1.000e-002 |
| plane-cylinder | 2 | True | 1 | -9.927e-003 |
| plane-box | 2 | True | 3 | -4.002e-003, -2.833e-002 |
| plane-mesh | 2 | True | 3 | -2.000e-002, -2.000e-002 |
| sphere-sphere | 2 | True | 1 | -4.000e-002 |
| sphere-capsule | 2 | True | 1 | -1.000e-002 |
| sphere-cylinder | 2 | True | 1 | -1.000e-002 |
| capsule-capsule | 2 | True | 2 | -1.754e-002, -1.754e-002 |

## 断言与结论
- 每 case：解析成功、`ngeom == 2`、`opt.disableflags & NATIVECCD` 为真、`nacon == 预期`（有效接触 >0）。
- 结果：**PASS**（failures=[]）。**nacon 预期由本轮 CPU 预检实测确定，只作 fixture 有效性基线，不作参考/判据。**
- 观察：GPU 捕获 `capsule-capsule` nacon=1 vs 本预检 2（其余 9 例一致），交 Arch 判定。
