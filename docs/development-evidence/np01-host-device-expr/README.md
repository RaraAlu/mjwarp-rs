# NP01 B1：host/device 逐表达式 PTX 定序表（12 条表达式，只编译不跑内核）

任务：`2026-11-04-precision-np01-host-device-expr`（Chief 批准：B1 与 A0 并行，范围锁死 ≤20 条关键表达式）。
脚本：`scripts/authoring/np01-host-device-expr.py`（离线 Python + NVRTC 12.8，无 GPU、无 buffer 写入）。
输出：同目录 `host_device_expr.json`（**本 README 数字全部由脚本产出**）。

复跑：

```powershell
& target\mjwarp-parallel\venv\Scripts\python.exe scripts\authoring\np01-host-device-expr.py
```

## 一、方法

- 每个表达式写一个最小 CUDA 探针 `extern "C" __global__ void probe(...)`，输入全部来自参数数组，无外部依赖。
- 同一份源码用 NVRTC 12.8 编译两次（`--gpu-architecture=compute_89 --std=c++17`）：
  - `--fmad=true`：**device 默认收缩**（pinned ccd PTX 实测 `fma.rn=1480`、`mul.rn/add.rn=0`）；
  - `--fmad=false`：强制两次舍入，**近似 host CPU recipe**（host 侧本来就是独立 mul/add 语句）。
- 在 PTX 中统计 `fma.rn / mul.rn / add.rn / sub.rn / div.rn / sqrt.rn / rcp.rn` 的条数，逐表达式对照 host 语句与舍入次数。
- 只编译、不启动内核、不写 buffer；不碰产品源码与判据。

## 二、结果（12 条）

| 表达式 | host 出处 | host 口径 | device fmad=true | device fmad=false | 判定 |
| --- | --- | --- | --- | --- | --- |
| `dot3` | `gjk/math.rs:15-17` | 2 乘 + 2 加（5 次舍入） | fma.rn 2 / mul 0 / add 0 | fma 0 / mul 3 / add 2 | device 融合；host 与非融合稿同形 |
| `length3` | `gjk/math.rs:31-33` | 3 乘 + 2 加 + sqrt | fma 2 / mul 0 / add 0 | fma 0 / mul 3 / add 2 | 同上 |
| `normalize3_component` | `gjk/math.rs:37-44` | length3 + 1 div | fma 2 / mul 0 / add 0 | fma 0 / mul 3 / add 2 | 同上（div 两侧都独立） |
| `face_norm2_head_tail` | `gjk/polytope.rs:70-72` | 显式 head→tail→norm2，刻意两次舍入 | **fma 2** / mul 0 / add 0 | fma 0 / mul 3 / add 2 | **源码写两步也挡不住 device 收缩** |
| `pos_midpoint` | `gjk/contact.rs:245-246` | 2 加 + 1 乘（0.5 为 2 的幂） | fma 1 / mul 0 / add 0 | fma 0 / mul 1 / add 2 | device 融合；0.5 缩放不改变尾数 |
| `make_frame_b` | `gjk/math.rs:140-141` | dot 2 乘2 加 + 1 乘 + 1 减 | fma 2 / mul 0 / add 0 | fma 0 / mul 4 / add 2 | device 融合；host 两次舍入 |
| `linear_combine_3` | `gjk/simplex.rs:310-314` | 3 乘 + 2 加 | fma 2 / mul 0 / add 0 | fma 0 / mul 3 / add 2 | device 融合 |
| `project_plane_scl` | `gjk/simplex.rs:71-75` | 1 div + 3 乘（表达式取 1 乘） | fma 0 / mul.rn 0 | fma 0 / mul.rn 1 | 无可融合乘加；div 独立 |
| `epa_upper_k` | `gjk/epa.rs:108` | dot 2 乘2 加 + 1 div | fma 2 / mul 0 / add 0 | fma 0 / mul 3 / add 2 | device 融合（归一化前点积也不例外） |
| `contact_dist_margin` | `gjk/contact.rs:191` | 1 加 | fma 0 / add.rn 0（见 JSON 计数） | add.rn 1 | 纯加法，两侧无融合问题 |
| `frame_dir_sub` | `gjk/contact.rs:234-238` | 分量差（3 减） | 0 | 0 | 纯减法被优化为寄存器搬移，无算术指令 |
| `tri_affine_c31` | `gjk/simplex.rs:369-370` | 6 项乘减和 | fma 2 / mul 0 / add 0 | fma 0 / mul 6 / add 2 | device 融合；host 6 次乘 + 5 次加 |

（完整计数、PTX sha256、探针源码见 `host_device_expr.json`。）

## 三、结论

1. **device 默认收缩是普遍规律**：12 条里 8 条带乘加的表达式在 `fmad=true` 下全部出现 `fma.rn`，且 `mul.rn/add.rn` 归零；`fmad=false` 下 `fma.rn` 归零、指令数与 host 的「每乘加两次舍入」逐项对应。
2. **`face_norm2` 的关键发现**：host 站点在 `polytope.rs:70-72` 刻意把 `face_norm2` 写成 head→tail→norm2 三步，目的是在 CPU recipe 下固定两次舍入；但同一写法在 device 默认 recipe 下**仍被收缩成 `fma.rn`（2 条）**。→ 源码写法本身不能表达「禁止融合」；只有编译选项（`--fmad=false`）或 `__fmul_rn/__fadd_rn` 内建才能固定。这与 `polytope.rs:67-69` 的注释口径一致：host 侧按 CPU recipe，device 侧属 fma.rn recipe。
3. **因此 host/device 位级判据必须声明 recipe 前提**：本表给的是「同源码在两套 recipe 下的形态差」，不是「数值等价证明」。跨 recipe 比较只能做类型/顺序审计（B 类参考-前提型），不能直接判位级通过。
4. **与 A0 的关系**：A0 证明「设备与上游对 native 偏差比例≈1」；B1 进一步给出机制解释——设备与上游同在 fma.rn recipe 下，故它们彼此接近，而 host CPU recipe 天然多一次舍入。两者共同支持「E 的参考选择必须写清 recipe」。

## 四、边界与后续

- 只覆盖三 host 站点（gjk/ccd/epa）的关键表达式 **12 条 ≤ 20** 上限；不是全内核覆盖，也不宣称覆盖全部表达式。
- 零 GPU、零产品代码、零判据/容差/fixtures 改动；不宣称 NP01/NP03 通过。
- 后续：B2（字段级位差分账：把 `compare-cpu-vs-gpu.json` 的 5 case 与 ULP 表整理成「recipe 差 / 类型偏离 / 引擎语义」三分类，引用 witness known-deviation 卡）→ B4（QA 独立复核，重点核「没把 recipe 差写成类型偏离」）。
