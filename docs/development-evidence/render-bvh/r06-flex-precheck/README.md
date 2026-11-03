# R06 柔性体渲染 S0 预检：authoring 环境恢复 + 81 槽 flex 形参对表

任务：`2026-11-03-render-r06-flex-precheck`。日期：2026-11-04（空闲段执行）。只读对表 + 环境恢复留档；
**不改产品代码/测试/判据/夹具，不跑 GPU 计算、不开窗口**。

## 1. authoring 环境恢复（离线 authoring 例外；只写 `target/`，不进产品构建与测试）

`target/` 在 2026-11-02 瘦身时被清空，导致参考采集环境与 pinned upstream checkout 缺失。本次按
`s0b-reference-capture` 记录的版本重建，路径与 pin 与之对齐：

| 项 | 值 | 校验 |
| --- | --- | --- |
| venv | `target/mjwarp-parallel/venv`（`sys.prefix` 与之完全一致） | `python -c "import sys; print(sys.prefix)"` |
| Python | **3.12.13**（uv 管理的 CPython） | `sys.version.split()[0]` |
| Warp | **warp-lang 1.15.0** | `warp.__version__` |
| Warp DLL | `venv/Lib/site-packages/warp/bin/warp.dll` | sha256 `7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6`（与 `g03-upstream-matrix/env-audit.json` 记录一致） |
| MuJoCo | **3.15.0** | `mujoco.__version__` |
| MJWarp | **mujoco-warp 3.15.0**，以 **editable** 方式从 pinned checkout 安装（`--no-deps -e target/mjwarp-parallel/upstream`） | `mujoco_warp.__file__` 指向 checkout 内包 |
| numpy | **2.5.4** | `numpy.__version__` |
| upstream checkout | `target/mjwarp-parallel/upstream` | HEAD `71da24d956378a87a703b6e1442b13aec0c4ac29`，`git status` clean，来源 `https://github.com/google-deepmind/mujoco_warp` |
| 采集脚本冒烟 | `scripts/capture_render_shadow.py --help` 在 pinned venv 下正常输出用法 | `--help` 退出码非 0 属 argparse 常规行为，不影响 |

陷阱留档：先装 PyPI 的 `mujoco-warp==3.15.0` 再与 checkout 对比，发现 5 个 `.py` 文本不一致
（`_src/math.py`、`_src/io.py`、`_src/collision_primitive_core.py`、`_src/math_test.py`、
`_src/collision_driver_test.py`）——**PyPI 轮子不是 pinned commit**；改为 editable 安装后包与 checkout
同目录、逐字节同源。后续任何参考采集都必须用 editable pin，不得用 PyPI 轮子。

## 2. 81 槽 ABI vs 上游 flex 形参（S0 核心结论）

上游签名取自 `upstream/mujoco_warp/_src/render.py::_render_megakernel`（pinned 71da24d；81 个形参 =
1 bounds + 73 数组 + 7 标量，与 S7a 冻结口径一致）。flex 相关形参 **9 个，全部已在 81 槽内**：

| 上游形参 | 上游类型 | 我们当前实参（`src/render/kernel/megakernel.rs`） | 结论 |
| --- | --- | --- | --- |
| `flex_vertadr` | `wp.array[int]` | `flex_vertadr` I32 `[1]` 占位 | 槽在；S2 填真值 |
| `flex_edge` | `wp.array[wp.vec2i]` | `flex_edge` Vec2I `[1]` 占位 | 槽在；S2 填真值 |
| `flex_radius` | `wp.array[float]` | `flex_radius` F32 `[1]` 占位 | 槽在；S2 填真值 |
| `flexvert_xpos_in` | `wp.array2d[wp.vec3]` | `flexvert_xpos` Vec3 `[1]` 占位 | 槽在；S2 改 2-D `[nworld, nflexvert]` |
| `flex_bvh_id` | `wp.array[wp.uint64]` | `flex_bvh_id` U64 `[1]` 占位 | 槽在；S3 填 BVH |
| `flex_group_root` | `wp.array2d[int]` | `flex_group_root` I32 `[1]` 占位 | 槽在；**形状须改 2-D `[nworld, nflex]`** |
| `flex_rgba` | `wp.array[wp.vec4]` | `flex_rgba` Vec4 `[1]` 占位 | 槽在；S2 填真值 |
| `flex_geom_flexid` | `wp.array[int]` | `flex_geom_flexid` I32 `[1]` 占位 | 槽在；S2 填真值 |
| `flex_geom_edgeid` | `wp.array[int]` | `flex_geom_edgeid` I32 `[1]` 占位 | 槽在；S2 填真值 |

配套标量 `bvh_nflexgeom` 也已在槽内（当前按 0 传）。**结论：R06 不需要改 81 槽 ABI**，成本维持
3–5 人日；唯一形状改动是 `flex_group_root` / `flexvert_xpos` 从 1-D 占位改 2-D——属 S2 实参填充，
不是 ABI 变更。

上游 flex 渲染语义（S3 判据依据）：

- 命中走 `ray_flex_with_bvh` / `ray_flex_with_bvh_anyhit`（`_src/ray.py`），几何 = 沿 `flex_edge` 的胶囊 + 顶点球，半径取 `flex_radius`；
- 颜色 `flex_rgba[flex_id]`，**seg 编码 `vec2i(flex_id, ObjType.FLEX)`**（`render.py:1247`）；
- 光滑/平面着色由 `flex_render_smooth` 控制（产品侧该参数已存在）。

## 3. 对切片的影响

- S1（结构拆分）不变：先拆 `megakernel.rs`（450/450）出 `megakernel_args.rs`，S2 要扩展的正是实参装配块。
- S2（宿主通道）明确为 9 个数组 + `bvh_nflexgeom` 标量 + 2 个 2-D 形状，占位换真值。
- S3（设备）需要 host 侧 flex BVH 构建/重拟合（上游 `bvh.py` 的 flex BVH 路径）+ 设备 cast 分支 + seg 编码。
- 采集脚本参照 `capture_render_hfield.py` 新建 `capture_render_flex.py`，fixture 目录 `fixtures/render-bvh/scene-flex/`。

## 4. 未覆盖 / 边界

- 本次没有生成任何参考、没有跑 GPU 计算、没有开窗口；`scene-flex` 场景与采集脚本尚未落地。
- 环境恢复只在 `target/`（gitignored）；产品构建/测试不读 venv；证据只登记 pin 与校验值。
- 未验证 `flex_render_smooth=false` 的设备行为、未定 seg 期望值、未做性能记录。
