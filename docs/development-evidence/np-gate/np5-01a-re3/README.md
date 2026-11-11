# NP5-01a RE-3：B 批全量 + C 批可映射子集设备级行映射（零 GPU）

- 任务：`2026-11-11-np5-01a-re3-tendon-and-c-mappable-device-level`（owner Backend）。
- 范围：B 批 7 行 78 站点 + C 批 17 行 226 站点 = **24 行 304 站点**的行级设备级映射；
  B-1 与 C-1..C-4 的逐站点 source 级分类**不重判**（device 行结论只属映射层）。
- 材料：冻结 `fixtures/gravity-upstream-abi/independent-inputs/smooth.ptx`（48 entries、993,601 B、
  `4179acd0…`）与 `fixtures/gravity-upstream-abi/generated/module.ptx`（15 entries）——两者均无 `.loc`；
  产品侧为 `src/physics/**` 的 CUDA raw string 离线编译 PTX（target 工作材料，**不入仓**）。
- 词表口径：`target/render-bvh/gl2-ptx-diff/np5-01a-vocab-addendum.md`（`453c5c91…`/6,771 B，六类词表 +
  `not-decidable` 六原因 + `reference_anchor.status` + quote 复核约定）。

## 1. 方法（可复核）

1. **产品编译单元 = 产品真实 launch unit**：按 `resident/plan_kernels.rs` / `attached.rs` /
   `plan/*.rs` 的 `format!` 装配方式取 CUDA 源（含 `FIELD_BATCH_CUDA`/`KINEMATICS_CUDA` 前导与
   `#define` 尺寸前缀），NVRTC **12.8.93 / compute_70 / `--fmad=true` / `-lineinfo`** 离线编译
   （产品 recipe：cudarc `CompileOptions::build` + `recipe.nvrtc_options()`）。
2. **产品 entry 集合由源码决定**；`#define` 尺寸前缀用**合成实例**填充（每行实例值见
   `target/.../RE-3-draft/units/manifest.json`）。实例只影响立即数/分支消除，
   **不参与分类**；行分类只基于「产品 entry 是否存在 + 与参考 entry 的语义对应关系」。
3. **参考 entry 匹配**：按冻结件 entry 名去 warp hash 后缀（`^_<name>_[0-9a-f]{8}_cuda_kernel_forward$`）
   精确匹配；B 行用 smooth.ptx 的 tendon/transmission 系列，C 行用 smooth/module 的 flex/passive/
   fluid/gravity/rne/camlight 系列（行→entry 名单见 `evidence.json.rows[].reference_entries`）。
4. **分类规则**（写进 `evidence.json.method_rules`，与 RE-2 先例一致）：
   - `engine-semantic`：产品 entry 与参考 entry 有清晰语义对应，但**内核组织/数据流不同**
     （合并、拆分、产品自加 validate/clear/support 内核）；不写数值或指令等价结论，`bit_level_guarantee:false`。
   - `not-decidable`：对应关系无法建立（参考无 entry、无独立 entry、冻结件无 lineinfo 无法做站点级对齐），
     `not_decidable_reasons[]` 按词表 §3 逐条写。
   - `n/a`：行语义为传输/索引搬运或纯 host 打包，非 PTX 可比。
   - 本片**不产生** `identical`/`recipe-difference`/`pin-needed` 行级结论：产品为手写 CUDA、
     参考为 warp 生成 PTX，两侧编译形态与 entry 粒度均不同，不具备逐指令/站点对齐前提。

## 2. B 批（7 行 78 站点）

| 行 | 站点 | 产品 entry→参考 entry | 分类 |
| --- | --- | --- | --- |
| fixed_tendon | 7 | `fixed_tendon`（长度累加+力臂写入，算术仅 1 fma）→ `_tendon_armature` + `_transmission` | `engine-semantic` |
| spatial_tendon | 20 | `spatial_site`（合并 site+geom）/`spatial_moment`/`spatial_wrap` → `_spatial_site_tendon`/`_spatial_geom_tendon`/`_transmission_body_moment*`/`_spatial_tendon_wrap` | `engine-semantic` |
| tendon | 10 | `tendon_values`/`tendon_indices`（整数寻址，无浮点算术）→ 参考算术 entry 不对应 | `n/a` |
| tendon_wrap | 9 | 产品 launch unit 无 entry（device 函数被 spatial 内核内联）→ 参考 wrap 数学内联在 `_spatial_tendon_wrap` 内 | `not-decidable`（frozen-artifact-only,no-lineinfo） |
| tendon_velocity | 7 | `tendon_velocity_validate/compute` → 参考 `_tendon_velocity` 在 **forward.py（未冻结）** | `not-decidable`（no-reference-entry） |
| tendon_passive | 12 | `validate/clear/compute` → `_spring_damper_tendon_passive`（单 entry） | `engine-semantic` |
| joint_passive | 13 | `validate/compute/sum` → `_spring_damper_dof_passive`（单 entry） | `engine-semantic` |

B 小计：`engine-semantic` 52 站点（4 行）、`not-decidable` 16（2 行）、`n/a` 10（1 行）= 78。

## 3. C 批（17 行 226 站点）

| 行 | 站点 | 产品 entry→参考 entry | 分类 |
| --- | --- | --- | --- |
| flex_position | 9 | `flex_positions` 合并 `_flex_vertices`+`_flex_nodes`；`flex_invalidate` 产品自加 | `engine-semantic` |
| flex_edge | 9 | `flex_edges` ↔ `_flex_edges`（1:1 同名） | `not-decidable`（no-lineinfo,frozen-artifact-only） |
| flex_face | 8 | `flex_faces` ↔ `_flex_face_kinematics`（1:1 同名） | `not-decidable`（no-lineinfo,frozen-artifact-only） |
| flex_hessian | 15 | `flex_hessian` 合并 `_flex_hessian_elem/diag/clear`；`flex_hessian_validate` 产品自加 | `engine-semantic` |
| flex_edge_passive | 11 | `validate/clear/compute` → `_spring_damper_flexedge_passive` | `engine-semantic` |
| fluid_force | 27 | `validate/body/project/clear` → `_fluid_force`（单 entry） | `engine-semantic` |
| gravity_compensation | 13 | `clear/validate/compute/sum` → `_gravity_force`（单 entry） | `engine-semantic` |
| rne_bias | 12 | `rne_bias` 合并 `_qfrc_bias`+`_cfrc`+`_cacc_world` | `engine-semantic` |
| rne_acc | 18 | `rne_cacc/rne_cfrc/rne_backward/rne_qfrc` → 参考 5 entry | `engine-semantic` |
| camlight | 17 | `camlight_rigid` 合并 `_cam_local_to_global`+`_light_local_to_global`；`camlight_com` COM 段；launch unit 含 FIELD_BATCH+KINEMATICS 前导 | `engine-semantic` |
| smooth_force | 9 | `smooth_force_validate/smooth_force` → `_joint_tendon`/`_transmission`/`_transmission_body_moment` 合成路径 | `engine-semantic` |
| passive_force | 7 | `clear/sum/clear_gravity/add_fluid` 合成 → 参考 5 entry 的合成/加法路径 | `engine-semantic` |
| attached | 14 | `rigid_kinematics` 前导 + `attached_kinematics` 合并 `_site_local_to_global`/`_geom_local_to_global` | `engine-semantic` |
| applied_force | 12 | `applied_force_validate/delta/publish` → 参考无 entry（`_apply_face_forces` 为 passive.py 内部函数） | `not-decidable`（no-reference-entry） |
| kinetic_energy | 13 | `kinetic_energy_validate/reduce32` → sensor.py tile reduce，无冻结 entry | `not-decidable`（no-reference-entry） |
| sleep_wake | 15 | `sleep_copy/tendon_wake` → sleep.py，无冻结 entry | `not-decidable`（no-reference-entry） |
| field_batch | 17 | `FIELD_BATCH_CUDA` 无 `__global__` entry（0 entry/202 B），pack 为 host 打包 | `n/a` |

C 小计：`engine-semantic` 152 站点（11 行）、`not-decidable` 57（5 行）、`n/a` 17（1 行）= 226。
合计：**304 站点 = B 78 + C 226**；行数 24；`engine-semantic` 204 站点/15 行、
`not-decidable` 73 站点/7 行、`n/a` 27 站点/2 行。

## 4. 关键观察与限制

- **entry 粒度差异是本片主因**：产品把上游多个 kernel 合并为单 entry（rne_bias、flex_hessian、
  rne_cacc 链、camlight）或拆出 validate/clear/sum 支撑内核（passive/fluid/gravity/passive_force），
  参考侧为 warp 单 forward kernel——属结构/数据流差异，只能记 `engine-semantic`，不得当数值等价。
- **无 lineinfo**：两份冻结件均 0 `.loc`；即使 1:1 同名（flex_edge/flex_face），也只能到 entry 级，
  无法把站点映射到参考源码行 → `not-decidable`（升级需参考侧 lineinfo 或站点级参考表达式）。
- **缺参考模块**：tendon_velocity 在 forward.py；applied_force/kinetic_energy/sleep_wake 在
  passive.py/sensor.py/sleep.py 的内核未被冻结 → 逐条 `no-reference-entry`。
- **合成实例限制**：`#define` 尺寸前缀用合成实例编译，直方图只作形态观察（证据中
  `synthetic_instance_note` 与每行实例值可复核）；entry 集合与分类不受实例影响。
- **PTX 实体不入仓**：产品 PTX 由 `target/render-bvh/gl2-ptx-diff/np5-01a/RE-3-draft/` 的
  `extract_cuda.py`/`gen_units.py` + `nvrtc_ptx.py`（pinned 12.8.93）确定性重生成；
  `evidence.json.product_units.units` 只记每单元 sha/bytes/entry 名单。
- **与 source 级证据的关系**：B-1（78）/ C-1..C-4（226）逐站点分类不变；本片行级 `not-decidable`
  不推翻 source 级 `identical` 分类（两层粒度不同，如 flex_edge/flex_face），也不构成覆盖率声明。

## 5. 非声明

不宣称 NP01 通过/精度完成/上游等价/双平台；不写位级保证；不改判据/参考/容差/相机/recipe；
零 GPU、零窗；不写覆盖率或通过率。
