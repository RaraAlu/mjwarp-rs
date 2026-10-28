# R09-S3b secondary-ray 阴影漏判修复（mesh 遮挡体的 cull 语义对齐上游 any-hit）

任务：`2026-10-28-render-bvh-r09-s3b-shadow-secondary-ray`（owner Backend；scope `src/render/kernel/` + 证据）
判据树 HEAD（开写时）：`cbee1127`
背景：S3b 验证窗 attempt1（有效轮 1）的 `render_shadow_reference` 红项——shadow-on cam1 RGB 超 1 LSB 1436 像素、max_byte_diff 77；参考阴影像素 4745 中漏判约 1208。本卡定位并修复根因；判据/容差/参考/夹具**零改动**。

## 1. 根因（只读定位 + dev 单渲染诊断）

### 1.1 观测数据
| 观测 | 结果 |
| --- | --- |
| 产品 shadow-on cam1 超 1 LSB | 1436，**全部落在参考阴影像素集（4745）内**，产品偏亮（darker=0）→ 100% 漏判、0 多判；depth/seg 两相机 0 差异 |
| 参考 cam1 阴影像素特征 | 4745 像素，hit 面全为 cylinder（geom (0,5)/(1,5) 等），depth 1.25–2.07，on/off ratio 0.45–0.95（多灯部分遮挡） |
| 逐灯覆盖（产品 cam1） | sun 1152 / lamp 2541 / spot 1465；漏判集中 1195 像素「三灯全无阴影」→ 与具体灯无关 |
| **产品 cull 开关实验** | `enable_backface_culling=false`：cam1 超 1 LSB **1436 → 38**（主射线同时出现 38 depth/29 seg 差异，说明不能全局关 cull） |
| **上游语义实验**（pinned python 单次渲染，非验证窗） | 上游 cull 开关对同一场景阴影只影响 **38 像素**（cam1）、cam0 0；即产品 cull=true 相对上游 cull=true 多剔除约 1398 |

### 1.2 上游实现（冻结 `render.py`，`71da24d`）
- 阴影射线走 **`cast_ray_first_hit`**（`_make_cast_ray(..., first_hit=True)`，L585-827；`_make_compute_lighting` L830 传入）
- **mesh 的 any-hit 分支只返回命中布尔**（L740-750：`ray_mesh_with_bvh_anyhit` → `d = 0.0 if hit else -1.0`），**不填 `n`（normal）**
- 随后统一 cull 判据（L802-806）：`if cull_backfaces and d >= 0.0 and wp.dot(ray_dir_world, n) > 0.0: d = -1.0`；注释原文：「Strict `> 0` keeps tangent hits and **skips branches with a zero-vector normal (any-hit)**」
- ⇒ **上游阴影射线对 mesh 遮挡不做背面剔除（n 零向量使 cull 不生效）；对 primitive 遮挡仍剔除**

### 1.3 产品路径（S2 实现）
S2 按「方案 A 复用 first-hit `cast_ray`」实现：对 mesh 遮挡走 `ray_mesh_leaf(..., cull_backfaces)` 的背面 march（`CULL_MARCHES=8`）并执行外层 cull → 在阴影射线上**对 mesh 遮挡多剔除约 1398 个命中** → 漏判。

## 2. 修复（最小改动；primary ray 零行为变化）

| 文件 | 行数 | 改动 |
| --- | --- | --- |
| `src/render/kernel/cuda/render_common.cu` | 387 → **393** | `cast_ray` 增加 `bool shadow_ray` 参数：shadow_ray 时 (a) mesh 内部 cull 关闭（`ray_mesh_leaf(..., cull_backfaces && !shadow_ray)`），(b) 外层 cull 仅对非 mesh 生效（`!(shadow_ray && gtype == 7)`）；注释写明上游 any-hit 语义 |
| `src/render/kernel/cuda/render_megakernel.cu` | 340（+0） | primary ray 调用传 `shadow_ray=false`（行为不变） |
| `src/render/kernel/cuda/render_shadow.cu` | 30（+0） | 阴影射线调用传 `shadow_ray=true` |

零触碰：`render.cu`(499) / `launch.rs`(500) / `cast.rs`(494) / `model/render.rs`(500) / `camera_output.rs`(499) / `DEVICE_SOURCES`(28) / `CollisionArgs`(504 B)；判据/容差/参考/夹具零改动。

## 3. 验证（dev 单渲染 + 全量 primary-ray 位级回归；非验证窗）

### 3.1 dev 单渲染（`logs/dev-verify.log`）
- **shadow-on**：cam0/cam1 rgb `mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0`、depth/seg `0`（修复前 1436 → **0**）
- **shadow-off**：同判据两相机全 0；**pre-unlock 位回归**（对 `s7b/*_fast_math_false.json`）depth/seg/rgb 全 0
- **残余 38 的处置**：cull=false 时的 38 像素残余 = 上游 cull 开关自身差异（`logs/upstream-cull-probe.log`：上游 cull on/off cam1 差 38），产品精确对齐 any-hit 语义后为 **0**，无需解释性残留

### 3.2 primary-ray 全量位级回归（`logs/regression-*.log`，GPU ignored 用例）
| 实体 | 结果 |
| --- | --- |
| `render_kernel_depth_seg`（S7b1） | **6 passed**（depth/seg 120 000 位等 + 拒绝面） |
| `render_kernel_rgb`（S7b2） | **3 passed**（同配方回归 + RGB + 拒绝面） |
| `render_kernel_flags`（S7b3） | **6 passed**（flag 变体 / AA / 阴影接受断言） |
| `render_e2e`（S8） | **5 passed**（含 negative control 的 `use_shadows=true` 接受断言） |

### 3.3 四门禁
`normify gate 2026-10-28-render-bvh-r09-s3b-shadow-secondary-ray`（绑定最终树；记录见 `.normify/runs/`）。

## 4. 限制与口径

- 本卡不产窗口结论：数值验收在 **attempt2 验证窗**（最后一次有效轮）；本卡只做定位、修复与 dev/回归自查
- 不写任何窗口结论、覆盖声明、精度结论或等价结论（含上游与双平台）
- 参考/夹具只读；判据/容差零改动；未删除或降级任何用例
