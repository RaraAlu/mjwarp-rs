# B0 fixture CPU 预检（窗口外，2026-10-26）

## 解释器与版本（pinned，与捕获 provenance 一致）
- 解释器：`D:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Scripts\python.exe`（python 3.12.13）
- mujoco 3.15.0 / warp 1.15.0（CPU 后端 wp.set_device("cpu")）/ mujoco_warp 3.15.0 / numpy
- 性质：**预检只证 fixture 合法且能产生有效 primitive 接触**；参考仍来自窗口内 GPU 捕获，本预检不上判据、不是参考。
- 脚本（不入库，临时）：`target/g10-p5-primitive-precheck/precheck_cpu.py`；原始输出 `precheck-cpu.json`。

## 逐 case 结果（CPU 后端 kinematics+collision；ngeom=2、NATIVECCD 均置位）
| case | ngeom | nativeccd disabled | nacon（预期=实际） | 前 2 条 dist（预检量级，非参考） |
| --- | --- | --- | --- | --- || plane-sphere | 2 | True | 1 | -1.000e-002 |
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
- 结果：**PASS**（failures=[]）。预检通过 → fixture 可进入窗口 GPU 捕获；未发现解析错/0 接触/数量异常。
- 预期 nacon 取自本预检实测并与 fixture 几何一致（capsule/box/mesh 的接触条数由穿透顶点/角点决定），窗口内 GPU 捕获应逐位复现同一结构。
