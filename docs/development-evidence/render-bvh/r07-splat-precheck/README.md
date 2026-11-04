# R07 高斯泼溅 S0 预检（只读可行性；不开族）

任务：`2026-11-04-render-r07-splat-precheck`（owner Arch）。日期：2026-11-04（空闲段）。
**边界：只读对表，零 GPU、零 `src/`/`tests/`/`fixtures/` 改动、不改判据/容差/参考、不开族、不实现。**
上游依据：pinned checkout `target/mjwarp-parallel/upstream`（`71da24d9`，mujoco_warp 3.15.0；`target/` gitignored，本证据只登记 pin 与结论）。

## 1. 一句话结论

**可行但成本高、且有一个必须先解的结构问题**：上游 splat 渲染路径完整（6 个 megakernel 形参 + `ray_splat`/`shade_splats` + splat BVH 构建/refit），
我们侧 **81 槽里 6 个 splat 槽全部被现有常量机制占用**——`splat_position` 存 headlight 三色+背景、`splat_scale` 存 `[znear,zfar,shadow_light_fraction]`、`splat_bvh_id` 槽存 `(background<<32)|flags`；只有 `splat_rotation`/`splat_rgba`/`splat_group_root` 是 1 元素占位。
**开 R07 = 必须先做一次 81 槽/常量通道重设计（ABI 级，需用户签）**，不是「填 6 个占位」那么简单。
产品侧 splat 数据链路（native 字段 / io / model / BVH / 设备）**完全为零**，且 `megakernel.rs` 对 `splat_count != 0` 直接 `Unsupported`。

## 2. 上游形参 vs 我们 81 槽（逐项）

| 上游形参 | 上游类型 | 我们的槽现状 | 结论 |
| --- | --- | --- | --- |
| `splat_position` | `array[vec3]` | 槽在，但被复用为 **`[headlight_ambient, headlight_diffuse, headlight_specular, background]`（Vec3×4）** | **冲突**：要真数据必须迁走常量块 |
| `splat_rotation` | `array[quat]` | 槽在，1 元素占位（内核不读） | 可用，但只是「没被占」，不是「能直接用」——需真 2-D 形状 |
| `splat_scale` | `array[vec3]` | 被复用为 **`[znear, zfar, shadow_light_fraction]`**（R09-S2b 常量槽，内核读 `data[0][2]`） | **冲突**：必须先拆常量槽 |
| `splat_rgba` | `array[vec4]` | 槽在，1 元素占位（内核不读） | 可用，需真形状 |
| `splat_bvh_id` | `uint64` | 被复用为 **`(packed_background_u32 << 32) \| flags`** | **冲突**：splat BVH id 无槽可用 |
| `splat_group_root` | `array[int32]` | 槽在，1 元素占位 | 可用，需按 world 形状 |

> 一句话：**6 个槽里 3 个被常量/标志占用、3 个是空占位**；81 槽是已冻结口径（S7a/QA 已核），R07 要动它属架构级变更。

## 3. 上游语义（判据设计依据，逐条写死）

- **`ray_splat`**：`inverse_rotation` 把点/方向变换到 splat 局部并除以 `scale`；求射线与高斯中心平面交点 `distance = -dot(lpnt,lvec)/dot(lvec,lvec)`；`distance` 必须落在 `(min_distance, max_distance)`（否则返回 `-1,0`）；`delta = lpnt + lvec*distance`；`response = exp(-0.5*dot(delta,delta))`；`alpha = min(response*opacity, 1)`；`alpha < max(SPLAT_MIN_RESPONSE=0.01, 1/255)` 丢弃。
- **`shade_splats`**：BVH ray query 逐趟取命中 → **按距离插入排序**（bounded `_MAX_SPLAT_HITS=32`，含 `-1` 空槽）→ front-to-back 合成：`color += rgb*alpha*transmittance`、`transmittance *= 1-alpha`；`transmittance < 0.005` 停止；命中数达 32 时以 `min_distance = last_distance + 1e-6` 再开一趟（最多 32 趟）。
- **输出合成**：几何命中时 `rgb = splat_color + shade_color * splat_transmittance`；未命中时 `rgb = splat_color + (skybox 或 background) * splat_transmittance`；**`depth` 被 `splat_depth` 覆盖**（`splat_depth > 0` 时，且几何命中时也覆盖），最后乘 `-ray_dir_local_cam[2]`；**`seg` 只由几何决定，splat 不写 seg**。
- **splat BVH**（`bvh.py`）：`_compute_splat_bounds` 由 position/rotation/scale/opacity 反解 bounds——`response = clamp(SPLAT_MIN_RESPONSE/opacity, 1e-6, 0.97)`、按 `scale*response` 取 AABB；`wp.Bvh(..., groups=splat_group, constructor='sah')`；`compute_bvh_group_roots` 得每 world 根；`refit_splat_bvh` 重算 bounds 后 `refit()`。

## 4. 产品侧缺口（逐项）

| 层 | 现状 | 需要什么 |
| --- | --- | --- |
| 原生 ABI / 模型字段 | `native/`、`include/`、`src/model`、`src/io` **无任何 splat 字段** | 原生 additive 导出 `nsplat`、`splat_adr`、`splat_group`、`splat_position/rotation/scale/rgba`；并核 **产品 pinned MuJoCo 侧是否已有 splat 模型字段**（3.12 口径，未核） |
| 启动守卫 | `src/render/kernel/megakernel.rs:211`：`splat_count != 0` → `Unsupported("S7b3 不含 splat")` | 解锁 + 真正的 splat 形参装配 |
| 加速结构 | `src/render/bvh/` 只有 scene/mesh BVH | splat BVH 构建（bounds 反解）/ refit / group roots，且语义对齐上游 SAH |
| 设备内核 | 无 `render_splat.cu` | `ray_splat` + `shade_splats`（bounded 32 插入排序、多趟循环）+ 与既有 shade/skybox/background 的合成顺序 |
| 常量通道 | 6 槽被占（见 §2） | **ABI 级重设计**：迁走 headlight/background/znear/zfar/shadow_fraction/flags 或增槽 |

## 5. 判据草案（不代替签字）

- **取向**：仍走「同 GPU / 同配方 / 同编译形态」的参考型判据（B 类前提）。
- **RGB**：splat 透明混合对 f32 舍入极度敏感（`exp`、乘加、排序比较），**首选位级或 ≤1 LSB**；若实测抖动，需退到 ≤N LSB 并由**用户签**（不可自行放宽）。
- **depth**：splat 覆盖语义是新行为，需要单列判据（splat 前/后深度、无几何命中时的 splat 深度）。
- **seg**：保持「只由几何决定」，splat 场景下 seg 不应变（可作为负控制）。
- **确定性**：两跑 sha 相同；bounded 32 命中是**有损近似**，判据必须写清「命中集合规模上限」这个前提，否则不同场景会静默改变结果。

## 6. 切片草案（供开族决策）

| 片 | 内容 | 窗口 | 备注 |
| --- | --- | --- | --- |
| **R07-S0** | 本预检（形参/语义/缺口/槽冲突/判据草案） | 无 | CPU、零改动 |
| R07-S1 | 常量通道重设计（ABI 级）+ 原生 splat 数据（additive）+ io/model/physics 只读视图 | 无 | **会改 81 槽/设备源 → HEAD 与已验证 exe 脱钩，需重冻**；ABI 需用户签 |
| R07-S2 | splat BVH（bounds/group roots/refit）+ `render_splat.cu` + 守卫解锁 | 无（实现） | 同上脱钩；SAH 语义对齐 |
| R07-S3a | authoring 采集：`capture_render_splat.py` + `fixtures/render-bvh/scene-splat/`（多 splat 重叠、透明、背景分支） | 1 个 authoring 窗 | 参考来自 pinned upstream（editable pin，非 PyPI 轮子） |
| R07-S3b | 验证窗：同配方参考、RGB/depth/seg/两跑 sha | 1 个验证窗 | 判据需用户签；建议低分辨率起步 |

合计 **5 片、2 窗（+可选性能/AA 窗）**；S1/S2 是 ABI 与设备源改动，**风险最高**。

## 7. 风险

1. **ABI 冲突（最高）**：splat 槽被常量占用，R07 需要一次 81 槽/常量通道重设计——这是架构级改动，须用户签，且会连带影响现有全部渲染证据的重冻。
2. **数值/判据风险**：透明混合的 `exp`/排序/阈值对 f32 敏感；bounded 32 与多趟循环是有损近似，位级判据可能不可达（需提前定降级口径并签字）。
3. **加速结构风险**：splat BVH 的 bounds 反解与 SAH 构建顺序影响命中集合；跨实现对齐比 scene/mesh BVH 更难（现有 BVH 曾有 tie-order 类问题）。
4. **模型口径风险**：产品 pinned MuJoCo 是否具备 splat 模型字段**未核**；若没有，需要模型/ABI 侧补齐或升级口径（用户级）。
5. **成本**：5 片 + 2 窗，且 S1/S2 期间渲染线全部证据需重冻/重验。
6. **性能**：每像素最多 32 命中 × 32 趟排序，远重于现有路径；性能不作为开族前提但需单列。

## 8. 非声明

- 本卡**不宣称 splat 已实现**、**未获开族批准**、不宣称与上游一致、不宣称通过任何验收、不作为对外发布依据；
- `src/`/`tests/`/`fixtures/` 零改动；不改判据/容差/参考；
- 上游语义只作**判据设计输入**，不构成「我们已对齐」的结论。
