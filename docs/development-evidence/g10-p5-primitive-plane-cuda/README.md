# G10@P5 B1：plane 族 6 组合 device primitive 实现（对照 B0 上游 GPU 参考 + 回归）

- 任务：`2026-10-26-g10-p5-primitive-plane-cuda-r2`（p46；实现 Backend，审查/判据实现/窗口执行/证据收口 Arch）。交付提交：`249a4dbd`。
- **GPU 窗口已执行（attempt4 有效轮）**：6/6 组合对照通过 + 既有 primitive 3 / launcher 3 回归 bit-exact。

## 1. 范围与红线

- 允许改动：`src/physics/collision/cuda/primitive_plane_{common,convex,shapes,wrapper}.cu`（新增）、
  `product.cu` 的 primitive 入口分派、`src/runtime/collision_gpu/source.rs`（`DEVICE_SOURCES` 追加）、
  `src/runtime/collision_gpu.rs`（**CPU 侧 `validate_primitive_types` 扩到 plane 族**，见 §7）、
  `src/runtime/collision_gpu/{multicontact_tests,primitive_tests}.rs`（片段数/顺序/组合校验断言）、
  `tests/g10_p5_primitive_plane_cuda.rs`（新）、本证据目录。
- **0 新 ABI 字段**：`CollisionArgs`/`CollisionContactOut`/`collision_gpu/abi.rs` 未动；**0 新入口**：入口仍是
  `mjwarp_collision_primitive`，6 组合在其内部按 type-pair 分派。
- 行数棘轮：4 个新 `.cu` 均 ≤500 行；`product.cu` 356 行（<450，不拆分）。
- 语义依据：pinned `collision_primitive.py` rev `71da24d9`（`plane_convex` L52-278；wrapper L281/L600/L686/L761/L838-914/L1000）
  与 `collision_primitive_core.py`（`plane_sphere` L50、`plane_capsule` L258、`plane_ellipsoid` L311、`plane_box` L342、`plane_cylinder` L465）；
  设备侧保持 pinned GPU `fma.rn` 融合语义，按 pinned 算式形态转写（不改写入门、不加特判/豁免）。

## 2. dispatch

`nativeccd=disable` 下 6 组合均为 `CollisionType.PRIMITIVE`：

| 组合（含几何顺序互换） | wrapper |
| --- | --- |
| (PLANE,SPHERE) | `g5_plane_sphere_wrapper`（1 接触，id_=0） |
| (PLANE,CAPSULE) | `g5_plane_capsule_wrapper`（2 接触，共享 `(n,b,c)` 帧） |
| (PLANE,ELLIPSOID) | `g5_plane_ellipsoid_wrapper`（1 接触） |
| (PLANE,CYLINDER) | `g5_plane_cylinder_wrapper`（4 候选，detected 门过滤） |
| (PLANE,BOX) | `g5_plane_box_wrapper`（8 角点候选，id_=角点序号） |
| (PLANE,MESH) | `g5_plane_convex_wrapper`（≤4 接触，mesh 只用 `a.mesh_vert`/`a.mesh_graph`，无需新 ABI） |

写入门复用既有 `g5_write_contact`（`detected`/`MJ_MAXVAL`/`id_`→`geomcollisionid` 语义与 pinned `write_contact` 一致）。

## 3. 文件与行数

| 文件 | 行数 | 内容 |
| --- | --- | --- |
| `primitive_plane_common.cu` | 44 | 定义/常量/`g5_plane_frame` |
| `primitive_plane_convex.cu` | 162 | 唯一 `g5_plane_convex`：穷举 + graph hillclimb（running best）+ 4 点去重收集 |
| `primitive_plane_shapes.cu` | 171 | sphere/box/capsule/ellipsoid/cylinder 数值核心 |
| `primitive_plane_wrapper.cu` | 113 | 6 wrapper 写入门 + `g5_plane_mesh_view` |
| `product.cu` | 356 | 含 6 组合 dispatch（未拆） |
| `tests/g10_p5_primitive_plane_cuda.rs` | 494 | 单 test：一次 NVRTC + 一次 context，循环 6 case |

`DEVICE_SOURCES` **21→25**：`…→primitive→primitive_wrapper→boxbox→boxbox_wrapper→primitive_plane_common→convex→shapes→wrapper`；
`product.cu` 仍由 `PRODUCT_SOURCE` 追加。plane 四段由 `source.rs` **本地 `include_str!`** 引入（与 `PRODUCT_SOURCE` 同口径），
不经 `cuda/mod.rs` 常量 re-export —— 避免改 `src/physics/mod.rs` 触发 T4 `physics_core` 身份漂移。

## 4. 设备源指纹（CPU 冒烟，未变）

```
nvrtc_smoke_ok ptx_bytes=3713524 entries=6 source_bytes=222849
source_sha256=59631d8377e3a587cb98d396742f9698ea17105847a33586e8b7a89b8c84b917
```
（`--features cuda-probe --lib runtime::collision_gpu -- --include-ignored --nocapture`，25 passed / 0 failed，含 `nvrtc_smoke`；
PATH 前置 `target/toolchains/cuda-12.8.1/bin`。只编译设备源、不建 CUDA context。）

- **入口 6→6，0 新入口**：6 个 plane wrapper 是 `mjwarp_collision_primitive` 内部 type-pair 分派，不是新 PTX 入口。
- `make_frame` 单一定义：整串设备源 NVRTC 编译通过即证。
- 旧 21 段对照（Arch 独立复算，Python↔Rust 双向一致）：`source_bytes=198815`、
  `sha256=52b225c2830863cd8ce58d1fd7275a50fa1beeac1e9fc9fe40ec025faaebca6c`；**变化是预期**（新增 plane 四段）；
  **旧 21 段相对顺序未变**，4 段全部追加在 `boxbox_wrapper` 之后。
- 源改动过程中的**作废中间值**（留痕、不采用）：`224056/0b398009…`、`224018/ac2c1a17…`、`223355/a945e75a…`、`221968/89dc0964…`。

## 5. GPU 窗口结果（attempt4 有效轮，2026-10-26 20:48:30–20:48:59）

**逐 case N/N（6/6 通过，failures=0）**

| case | dev nacon | ref nacon | 结果 |
| --- | --- | --- | --- |
| plane-sphere | 1 | 1 | pass |
| plane-capsule | 1 | 1 | pass |
| plane-ellipsoid | 1 | 1 | pass |
| plane-cylinder | 1 | 1 | pass |
| plane-box | 3 | 3 | pass |
| plane-mesh | 3 | 3 | pass |

判据：位级优先；非位级 `dist` ≤4 ULP@自身量级（ref=0 用 4 ULP@1.0）、`pos` 4·ULP(scale)、`frame` 4 ULP@1.0；
`count`/结构（worldid/type/dim/geom/geomcollisionid/includemargin/adhesion/friction/solref/solreffriction/solimp）零容差；
`efc_address`/`overflow` 设备侧不变量（参考未采集）；行序不作判据（按 `geomcollisionid` 配对）；`nacon` 按 GPU 参考。

**回归（不许回退，均 bit-exact）**

| 二进制 | case | 结果 |
| --- | --- | --- |
| `g10_p5_primitive_cuda` | sphere-box-shallow / sphere-box-deep / capsule-box | 80/80 bit-equal、max ULP 0 |
| `g10_p5_nvrtc_launcher` | hfield-sphere / sdf-box / ellipsoid-box | 314/314、3122/3122、80/80 bit-equal、max ULP 0 |

**窗口事实**：body wall 23.5 s（3 个预构建二进制并行跑）；采样 93 点（0.25 s）；`used_peak=3367 MiB`、`free_min=12695 MiB`、
`util_max=41%`、`residual=0`；基线 `3352/12710`、结束基线 `3352/12710`；post compute-apps 无本任务进程；窗口内零 Rust 编译。
预构建二进制 sha256：plane `e5f94074…`、primitive `6aec50d0…`、launcher `bcd43c6e…`。

窗口原始证据：window-attempt4/（aseline.txt/acts.txt/guard.log/samples.csv/三个 un-*.log/console.log）；作废轮留档 window-attempt1/（host 校验 panic 的 .err.log 与 facts）与 window-attempt3/（plane-mesh 输入错误的 log/facts）。

**尝试轮如实记录（不拼接）**

- **attempt1**（20:44:09–20:44:38）：设备可跑但 plane 测试在 **CPU 侧组合校验**处 panic
  （`primitive 只支持 SPHERE-BOX / CAPSULE-BOX`）——host 侧 `validate_primitive_types` 未随设备 dispatch 扩展，
  本片缺项（见 §7）；该轮 regression 已 bit-exact，但 plane 未产出结果 → 作废。
- **attempt2**：runner 脚本自身 bug（guard 初始化参数被多插了一个反斜杠），零设备工作 → 作废。
- **attempt3**（20:46:55–20:47:23）：5/6 通过，`plane-mesh` dev nacon=1 vs ref 3、dist -4.64e-2 vs -2.0e-2。
  根因：测试喂的是 `tetra.obj` 原始坐标，而 MuJoCo 设备侧 `mesh_vert` 是经 `mesh_quat`（惯性系）旋转后的
  `model.mesh_vert`；属**测试输入错误**而非设备实现错误（用 pinned mujoco 3.15 dump 的 `model.mesh_vert` 独立复算可逐位复现参考值）。
  修正测试输入后重跑。
- **attempt4**（20:48:30–20:48:59）：6/6 + 3+3 回归全部通过（本证据）。

## 6. CPU 侧测试与门禁

- `cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored`：**25 passed / 0 failed**（含 `nvrtc_smoke`）。
- 默认 `cargo test --locked`：**971 passed / 0 failed / 370 ignored**（新增 1 个 ignored 窗口测试）。
- `cargo fmt --all` / `cargo clippy --locked --all-targets --features cuda-probe -- -D warnings`：exit 0。

## 7. host 侧组合校验扩展（B1 补项）

`src/runtime/collision_gpu.rs::validate_primitive_types` 原来只接受 {SPHERE,BOX}/{CAPSULE,BOX}，B1 设备 dispatch 落地后
必须同步放行 plane 族 6 组合（无序对）——否则 `CollisionKernel::Primitive` 在 CPU 侧直接 `InvalidArgument`（attempt1 实测）。
现语义：允许 {SPHERE,BOX}、{CAPSULE,BOX}、{PLANE, SPHERE/CAPSULE/ELLIPSOID/CYLINDER/BOX/MESH}（含互换）；
拒绝 plane-plane 与其它组合；单测 `primitive_tests.rs` 覆盖正/负例。该文件不在 T4 `product_sources` 内（无身份漂移）。

## 8. 输入来源

- 参考：`docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json`（B0 GPU 捕获，sha
  `3a6475acf362795ab98e8ee39309b09102c2dc76f7a57c6c6dbc95a069a98734`，10 case 取 plane 6）。
- 输入 `geom_xpos`/`geom_xmat` 取自该参考的捕获事实（同一 qpos）。
- `geom_size` 与 mesh 顶点来自 `fixtures/g10-p5-primitive/*.xml` + `tetra.obj`（sha/字节数见
  `docs/development-evidence/g10-p5-primitive-capture/inputs-sha256.json`）。**mesh 顶点必须用 MuJoCo
  `model.mesh_vert`（`mesh_quat` 旋转后的 mesh 局部系坐标），不是 `tetra.obj` 原始坐标**（attempt3 的教训）。

## 9. T4 身份 / ABI / fixture

- **T4 `physics_core` 组合身份未变**：`src/physics/mod.rs` 零 diff；**未另立 repin**；**0 fixture 改动**；
  `fixtures/precision/t4-current-kernels.json` 未动（其 20 条 `product_sources` 不含 `source.rs`/`cuda/*.cu`/`collision_gpu.rs`）。
- 0 新 ABI 字段 / 0 新入口；既有 verified fixtures 与旧参考零改动。

## 10. 未覆盖 / 边界（如实）

- `plane_convex` 的 **graph 分支**（`graphadr != -1 && vertnum >= 10`）**无 case 覆盖**：本片 `plane-mesh` 是 4 顶点走穷举分支；
  实现按 pinned 转写但未实机验证。
- `plane-mesh` 只需 mesh 顶点/图，未需要 polygon 等额外数据；若后续需要 → 停手报 Arch、记未覆盖。
- 未做 host 交叉、未做 release/Linux；不宣称「primitive 全组合通过」「精度达标」「跨环境全绿」。
- 本次只判「plane 族 6 组合在 B0 给出的 6 个 case 上、按上述判据」的结果。

## 11. 协作与判据修正留痕

- 实现（4 个 `.cu`、dispatch、测试骨架、证据初稿）：Backend。
- Arch 审查后修正：① `g5_plane_convex` graph hillclimb 改为 pinned 的 **running best 严格比较**（原实现取「最后一个未 mask 邻居」，语义错误）；
  ② `g5_plane_box` 改回 pinned 的 `center_dist + dot(n, rot@corner)` 算式与左结合；③ `g5_plane_cylinder` 接触 3/4 位置结合顺序改回 pinned；
  ④ capsule/ellipsoid/cylinder 的 `normalize` 改逐分量除法形态；⑤ 清过期 `SKELETON/TODO` 注释与重复 `#include`；
  ⑥ **测试判据收紧**：`[count][width]` 参考解析、`dist` 4 ULP 判据（撤销 ~500 ULP 的放宽写法）、结构零容差、
  `geomcollisionid` 配对、`efc`/`overflow` 设备侧不变量；⑦ **host 组合校验扩展**（§7）；⑧ **mesh 顶点输入修正**（§5 attempt3）。
