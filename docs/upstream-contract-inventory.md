# MJWarp字段与接口盘点

整理日期：2026-10-06。
版本：0.1声明清单。
状态：源码盘点，尚未实现。

本表继续展开P0-03。
本表不关闭完整等价准入。
本表不冻结Rust内存ABI。
本表不证明GPU数值正确。

## 1. 基线与覆盖口径

```text
revision: 71da24d956378a87a703b6e1442b13aec0c4ac29
data_records: 12
field_declarations: 895
enum_classes: 31
enum_member_declarations: 218
public_function_signatures: 73
```

字段统计只计直接注解。
统计不计文档中的字段名。
统计不计property和动态赋值。
枚举保留声明表达式。
成员计数包含组合位与哨兵。
表达式求值仍需原生版本。
函数签名覆盖73个功能导出。
类型导出仍见公开映射表。

盘点来自静态声明读取。
读取不运行或导入Python。
读取保留名称与注解文本。
读取核对多行括号闭合。
本轮未采用Python AST检查。
语义迁移仍需人工审查。

本表关联以下设计：
[模块契约](module-contracts.md)、
[公开映射](upstream-api-map.md)、
[阶段契约](physics-stage-contracts.md)。

## 2. 声明与运行时布局

`array`声明记录形状与dtype。
尺寸字符串引用模型或状态。
星号表示声明中的动态长度。
星号不总代表世界批量维度。
`wp.array`注解可能不含尺寸。
构造函数决定实际缓冲形状。
声明不能替代分配路径审查。

渲染输出尤其需要核对构造。
颜色和深度使用世界维度。
本表不从注解猜测实际rank。
原生句柄不照搬为Rust类型。
Rust封装需保留资源所有者。

默认值按三个来源登记：

| 来源 | 处理规则 |
| --- | --- |
| 类型声明中的赋值 | 保留声明表达式；工厂表达式不当成共享实例 |
| 模型转换中的赋值 | 核对原生字段、转换、覆盖与推导 |
| 函数签名中的默认参数 | 保留None、布尔、数值及容器语义 |

字段没有赋值不等于默认零值。
分配使用empty也不等于初始化。
初始化与阶段写入需另行登记。

依据：[数组声明包装](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L897-L908)。

## 3. 数据记录总表

| 上游记录 | 声明数 | 计划归属 | 剩余审查 |
| --- | --- | --- | --- |
| `BlockDim` | 25 | physics/runtime计划配置 | 单位、分配、索引与访问阶段 |
| `Option` | 28 | model::options | 单位、分配、索引与访问阶段 |
| `Statistic` | 1 | model | 单位、分配、索引与访问阶段 |
| `TileSet` | 3 | model矩阵布局 | 单位、分配、索引与访问阶段 |
| `Callback` | 7 | physics::callback | 单位、分配、索引与访问阶段 |
| `Model` | 519 | model | 单位、分配、索引与访问阶段 |
| `Contact` | 18 | model::data | 单位、分配、索引与访问阶段 |
| `Constraint` | 20 | model::data | 单位、分配、索引与访问阶段 |
| `Data` | 157 | model::data | 单位、分配、索引与访问阶段 |
| `InverseContext` | 9 | physics::inverse工作区 | 单位、分配、索引与访问阶段 |
| `SolverContext` | 26 | physics::solver工作区 | 单位、分配、索引与访问阶段 |
| `RenderContext` | 82 | render | 单位、分配、索引与访问阶段 |

### 字段声明

下面保留上游注解与赋值。
代码块只用于源码核对。
它不是本仓库Rust接口。

#### BlockDim

声明数：25。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L57-L126)。

```text
segmented_sort: int = 128
convex_ccd: int = 64
actuator_velocity: int = 32
island_dsu: int = 32
ray: int = 64
contact_sort: int = 64
energy_vel_kinetic: int = 32
cholesky_factorize: int = 32
cholesky_factorize_solve: int = 32
cholesky_solve: int = 64
small_cholesky: int = 64
solve_LD_sparse_fused: int = 128
update_gradient_cholesky: int = 64
update_gradient_cholesky_blocked: int = 32
update_gradient_JTDAJ_sparse: int = 128
update_gradient_JTDAJ_dense: int = 128
linesearch_iterative: int = 32
update_gradient_grad: int = 256
solve_beta_accumulate: int = 256
solve_search_update_cg: int = 256
solve_init_search_cg: int = 256
contact_jac_tiled: int = 32
qderiv_actuator_dense: int = 32
eff_pcg: int = 128
render: int = 64
```

#### Option

声明数：28。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L912-L1009)。

```text
timestep: array("*", float)
tolerance: array("*", float)
ls_tolerance: array("*", float)
ccd_tolerance: array("*", float)
sleep_tolerance: array("*", float)
gravity: array("*", wp.vec3)
wind: array("*", wp.vec3)
magnetic: array("*", wp.vec3)
density: array("*", float)
viscosity: array("*", float)
integrator: int
cone: int
solver: int
iterations: int
ls_iterations: int
ccd_iterations: int
disableflags: int
enableflags: int
sdf_initpoints: int
sdf_iterations: int
impratio_invsqrt: array("*", float)
broadphase: BroadphaseType
broadphase_filter: BroadphaseFilter
graph_conditional: bool
run_collision_detection: bool
run_rne_postconstraint: bool
contact_sensor_maxmatch: int
warn_overflow: int
```

#### Statistic

声明数：1。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L1011-L1020)。

```text
meaninertia: array("*", float)
```

#### TileSet

声明数：3。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L1022-L1046)。

```text
adr: wp.array[int]
size: int
elemid: wp.array[int] = dataclasses.field(default_factory=lambda: wp.array([], dtype=int))
```

#### Callback

声明数：7。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L1048-L1069)。

```text
passive: Callable | None = None
control: Callable | None = None
act_dyn: Callable | None = None
act_gain: Callable | None = None
act_bias: Callable | None = None
sensor: Callable | None = None
contactfilter: Callable | None = None
```

#### Model

声明数：519。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L1071-L2134)。

```text
nq: int
nv: int
nu: int
nactuator: int
na: int
nbody: int
noct: int
njnt: int
ntree: int
nM: int
nC: int
nD: int
ngeom: int
nsite: int
ncam: int
nlight: int
nflex: int
nflexnode: int
nflexvert: int
nflexedge: int
nflexelem: int
nflexelemdata: int
nflexstiffness: int
nflexbending: int
nefm0dof: int
nefm0L: int
nflexelemedge: int
nflexshelldata: int
nJfe: int
nmesh: int
nmeshvert: int
nmeshnormal: int
nmeshface: int
nmeshgraph: int
nmeshpoly: int
nmeshpolyvert: int
nmeshpolymap: int
nhfield: int
nhfielddata: int
nmat: int
npair: int
nexclude: int
neq: int
ntendon: int
nJten: int
nwrap: int
nsensor: int
nkey: int
nmocap: int
nplugin: int
nJmom: int
npolygonmax: int
nmeshdegmax: int
nuserdata: int
nsensordata: int
nhistory: int
opt: Option
stat: Statistic
qpos0: array("*", "nq", float)
qpos_spring: array("*", "nq", float)
body_parentid: array("nbody", int)
body_rootid: array("nbody", int)
body_weldid: array("nbody", int)
body_mocapid: array("nbody", int)
body_jntnum: array("nbody", int)
body_jntadr: array("nbody", int)
body_dofnum: array("nbody", int)
body_dofadr: array("nbody", int)
body_treeid: array("nbody", int)
body_geomnum: array("nbody", int)
body_geomadr: array("nbody", int)
body_simple: array("nbody", int)
body_pos: array("*", "nbody", wp.vec3)
body_quat: array("*", "nbody", wp.quat)
body_ipos: array("*", "nbody", wp.vec3)
body_iquat: array("*", "nbody", wp.quat)
body_mass: array("*", "nbody", float)
body_subtreemass: array("*", "nbody", float)
body_inertia: array("*", "nbody", wp.vec3)
body_invweight0: array("*", "nbody", wp.vec2)
body_gravcomp: array("*", "nbody", float)
body_contype: array("nbody", int)
body_conaffinity: array("nbody", int)
oct_child: array("noct", vec8i)
oct_aabb: array("noct", 2, wp.vec3)
oct_coeff: array("noct", vec8)
jnt_type: array("njnt", int)
jnt_qposadr: array("njnt", int)
jnt_dofadr: array("njnt", int)
jnt_bodyid: array("njnt", int)
jnt_limited: array("njnt", int)
jnt_actfrclimited: array("njnt", bool)
jnt_actgravcomp: array("njnt", int)
jnt_solref: array("*", "njnt", wp.vec2)
jnt_solimp: array("*", "njnt", vec5)
jnt_pos: array("*", "njnt", wp.vec3)
jnt_axis: array("*", "njnt", wp.vec3)
jnt_stiffness: array("*", "njnt", float)
jnt_stiffnesspoly: array("*", "njnt", wp.vec2)
jnt_range: array("*", "njnt", wp.vec2)
jnt_actfrcrange: array("*", "njnt", wp.vec2)
jnt_margin: array("*", "njnt", float)
dof_bodyid: array("nv", int)
dof_jntid: array("nv", int)
dof_parentid: array("nv", int)
dof_treeid: array("nv", int)
dof_Madr: array("nv", int)
dof_solref: array("*", "nv", wp.vec2)
dof_solimp: array("*", "nv", vec5)
dof_frictionloss: array("*", "nv", float)
dof_armature: array("*", "nv", float)
dof_damping: array("*", "nv", float)
dof_dampingpoly: array("*", "nv", wp.vec2)
dof_invweight0: array("*", "nv", float)
dof_length: array("nv", float)
tree_bodynum: array("ntree", int)
tree_dofadr: array("ntree", int)
tree_dofnum: array("ntree", int)
tree_sleep_policy: array("ntree", int)
geom_type: array("ngeom", int)
geom_contype: array("ngeom", int)
geom_conaffinity: array("ngeom", int)
geom_condim: array("ngeom", int)
geom_bodyid: array("ngeom", int)
geom_dataid: array("*", "ngeom", int)
geom_matid: array("*", "ngeom", int)
geom_group: array("ngeom", int)
geom_priority: array("ngeom", int)
geom_solmix: array("*", "ngeom", float)
geom_solref: array("*", "ngeom", wp.vec2)
geom_solimp: array("*", "ngeom", vec5)
geom_size: array("*", "ngeom", wp.vec3)
geom_aabb: array("*", "ngeom", 2, wp.vec3)
geom_rbound: array("*", "ngeom", float)
geom_pos: array("*", "ngeom", wp.vec3)
geom_quat: array("*", "ngeom", wp.quat)
geom_friction: array("*", "ngeom", wp.vec3)
geom_margin: array("*", "ngeom", float)
geom_gap: array("*", "ngeom", float)
geom_surfacevel: array("*", "ngeom", vec6)
geom_adhesion: array("*", "ngeom", float)
geom_fluid: array("ngeom", 12, float)
geom_rgba: array("*", "ngeom", wp.vec4)
site_type: array("nsite", int)
site_bodyid: array("nsite", int)
site_size: array("nsite", wp.vec3)
site_pos: array("*", "nsite", wp.vec3)
site_quat: array("*", "nsite", wp.quat)
cam_mode: array("ncam", int)
cam_bodyid: array("ncam", int)
cam_targetbodyid: array("ncam", int)
cam_pos: array("*", "ncam", wp.vec3)
cam_quat: array("*", "ncam", wp.quat)
cam_poscom0: array("*", "ncam", wp.vec3)
cam_pos0: array("*", "ncam", wp.vec3)
cam_mat0: array("*", "ncam", wp.mat33)
cam_projection: array("ncam", int)
cam_fovy: array("*", "ncam", float)
cam_resolution: array("ncam", wp.vec2i)
cam_sensorsize: array("ncam", wp.vec2)
cam_intrinsic: array("*", "ncam", wp.vec4)
light_mode: array("nlight", int)
light_bodyid: array("nlight", int)
light_targetbodyid: array("nlight", int)
light_type: array("*", "nlight", int)
light_castshadow: array("*", "nlight", bool)
light_active: array("*", "nlight", bool)
light_pos: array("*", "nlight", wp.vec3)
light_dir: array("*", "nlight", wp.vec3)
light_poscom0: array("*", "nlight", wp.vec3)
light_pos0: array("*", "nlight", wp.vec3)
light_dir0: array("*", "nlight", wp.vec3)
light_attenuation: array("*", "nlight", wp.vec3)
light_cutoff: array("*", "nlight", float)
light_exponent: array("*", "nlight", float)
light_ambient: array("*", "nlight", wp.vec3)
light_diffuse: array("*", "nlight", wp.vec3)
light_specular: array("*", "nlight", wp.vec3)
flex_contype: array("nflex", int)
flex_conaffinity: array("nflex", int)
flex_condim: array("nflex", int)
flex_priority: array("nflex", int)
flex_solmix: array("nflex", float)
flex_solref: array("nflex", wp.vec2)
flex_solimp: array("nflex", vec5)
flex_friction: array("nflex", wp.vec3)
flex_margin: array("nflex", float)
flex_gap: array("nflex", float)
flex_selfcollide: array("nflex", int)
flex_activelayers: array("nflex", int)
flex_passive: array("nflex", int)
flex_dim: array("nflex", int)
flex_interp: array("nflex", int)
flex_cellnum: array("nflex", wp.vec3i)
flex_nodeadr: array("nflex", int)
flex_nodenum: array("nflex", int)
flex_vertadr: array("nflex", int)
flex_vertnum: array("nflex", int)
flex_edgeadr: array("nflex", int)
flex_edgenum: array("nflex", int)
flex_elemadr: array("nflex", int)
flex_elemnum: array("nflex", int)
flex_elemdataadr: array("nflex", int)
flex_stiffnessadr: array("nflex", int)
flex_elemedgeadr: array("nflex", int)
flex_bendingadr: array("nflex", int)
flex_shellnum: array("nflex", int)
flex_shelldataadr: array("nflex", int)
flex_nodebodyid: array("nflexnode", int)
flex_vertbodyid: array("nflexvert", int)
flex_edge: array("nflexedge", wp.vec2i)
flex_edgeflap: array("nflexedge", wp.vec2i)
flex_elem: array("nflexelemdata", int)
flex_elemedge: array("nflexelemedge", int)
flex_elemlayer: array("nflexelem", int)
flex_shell: array("nflexshelldata", int)
flex_vert: array("nflexvert", wp.vec3)
flex_vert0: array("nflexvert", wp.vec3)
flex_node: array("nflexnode", wp.vec3)
flex_node0: array("nflexnode", wp.vec3)
flexedge_length0: array("nflexedge", float)
flexedge_invweight0: array("nflexedge", float)
flex_radius: array("nflex", float)
flex_size: array("nflex", wp.vec3)
flex_stiffness: array("nflexstiffness", float)
flex_bending: array("nflexbending", float)
efm0_dofid: array("nefm0dof", int)
efm0_L_rownnz: array("nefm0dof", int)
efm0_L_rowadr: array("nefm0dof", int)
efm0_L_colind: array("nefm0L", int)
efm0_L: array("nefm0L", float)
flex_damping: array("nflex", float)
flex_edgestiffness: array("nflex", float)
flex_edgedamping: array("nflex", float)
flex_edgeequality: array("nflex", int)
flex_rigid: array("nflex", bool)
flexedge_rigid: array("nflexedge", bool)
flex_centered: array("nflex", bool)
flexedge_J_rownnz: array("nflexedge", int)
flexedge_J_rowadr: array("nflexedge", int)
flexedge_J_colind: array("nJfe", int)
mesh_vertadr: array("nmesh", int)
mesh_vertnum: array("nmesh", int)
mesh_faceadr: array("nmesh", int)
mesh_octadr: array("nmesh", int)
mesh_normaladr: array("nmesh", int)
mesh_normalnum: array("nmesh", int)
mesh_graphadr: array("nmesh", int)
mesh_vert: array("nmeshvert", wp.vec3)
mesh_normal: array("nmeshnormal", wp.vec3)
mesh_face: array("nmeshface", wp.vec3i)
mesh_graph: array("nmeshgraph", int)
mesh_pos: array("nmesh", wp.vec3)
mesh_quat: array("nmesh", wp.quat)
mesh_polynum: array("nmesh", int)
mesh_polyadr: array("nmesh", int)
mesh_polynormal: array("nmeshpoly", wp.vec3)
mesh_polyvertadr: array("nmeshpoly", int)
mesh_polyvertnum: array("nmeshpoly", int)
mesh_polyvert: array("nmeshpolyvert", int)
mesh_polymapadr: array("nmeshvert", int)
mesh_polymapnum: array("nmeshvert", int)
mesh_polymap: array("nmeshpolymap", int)
hfield_size: array("nhfield", wp.vec4)
hfield_nrow: array("nhfield", int)
hfield_ncol: array("nhfield", int)
hfield_adr: array("nhfield", int)
hfield_data: array("nhfielddata", float)
mat_texid: array("*", "nmat", 10, int)
mat_texuniform: array("*", "nmat", bool)
mat_texrepeat: array("*", "nmat", wp.vec2)
mat_emission: array("*", "nmat", float)
mat_specular: array("*", "nmat", float)
mat_shininess: array("*", "nmat", float)
mat_rgba: array("*", "nmat", wp.vec4)
pair_dim: array("npair", int)
pair_geom1: array("npair", int)
pair_geom2: array("npair", int)
pair_solref: array("*", "npair", wp.vec2)
pair_solreffriction: array("*", "npair", wp.vec2)
pair_solimp: array("*", "npair", vec5)
pair_margin: array("*", "npair", float)
pair_gap: array("*", "npair", float)
pair_adhesion: array("*", "npair", float)
pair_friction: array("*", "npair", vec5)
exclude_signature: array("nexclude", int)
eq_type: array("neq", int)
eq_obj1id: array("neq", int)
eq_obj2id: array("neq", int)
eq_objtype: array("neq", int)
eq_active0: array("neq", bool)
eq_solref: array("*", "neq", wp.vec2)
eq_solimp: array("*", "neq", vec5)
eq_data: array("*", "neq", vec11)
tendon_adr: array("ntendon", int)
tendon_num: array("ntendon", int)
ten_J_rownnz: array("ntendon", int)
ten_J_rowadr: array("ntendon", int)
ten_J_colind: array("nJten", int)
tendon_limited: array("ntendon", int)
tendon_actfrclimited: array("ntendon", bool)
tendon_solref_lim: array("*", "ntendon", wp.vec2)
tendon_solimp_lim: array("*", "ntendon", vec5)
tendon_solref_fri: array("*", "ntendon", wp.vec2)
tendon_solimp_fri: array("*", "ntendon", vec5)
tendon_range: array("*", "ntendon", wp.vec2)
tendon_actfrcrange: array("*", "ntendon", wp.vec2)
tendon_margin: array("*", "ntendon", float)
tendon_stiffness: array("*", "ntendon", float)
tendon_stiffnesspoly: array("*", "ntendon", wp.vec2)
tendon_damping: array("*", "ntendon", float)
tendon_dampingpoly: array("*", "ntendon", wp.vec2)
tendon_armature: array("*", "ntendon", float)
tendon_frictionloss: array("*", "ntendon", float)
tendon_lengthspring: array("*", "ntendon", wp.vec2)
tendon_length0: array("*", "ntendon", float)
tendon_invweight0: array("*", "ntendon", float)
wrap_type: array("nwrap", int)
wrap_objid: array("nwrap", int)
wrap_prm: array("nwrap", float)
actuator_trntype: array("nactuator", int)
actuator_dyntype: array("nactuator", int)
actuator_gaintype: array("nactuator", int)
actuator_biastype: array("nactuator", int)
actuator_ctrladr: array("nactuator", int)
actuator_ctrlnum: array("nactuator", int)
actuator_ctrlspec: array("nactuator", int)
actuator_actadr: array("nactuator", int)
actuator_actnum: array("nactuator", int)
actuator_trnid: array("nactuator", wp.vec2i)
actuator_cranklength: array("*", "nactuator", float)
actuator_dynprm: array("*", "nactuator", vec10)
actuator_gainprm: array("*", "nactuator", vec10)
actuator_biasprm: array("*", "nactuator", vec10)
actuator_actlimited: array("nactuator", bool)
actuator_actrange: array("*", "nactuator", wp.vec2)
actuator_actearly: array("nactuator", bool)
actuator_history: array("*", "nactuator", wp.vec2i)
actuator_historyadr: array("*", "nactuator", int)
actuator_delay: array("*", "nactuator", float)
actuator_forcelimited: array("nactuator", bool)
actuator_forcerange: array("*", "nactuator", wp.vec2)
actuator_ctrllimited: array("nu", bool)
actuator_ctrlrange: array("*", "nu", wp.vec2)
actuator_gear: array("*", "nactuator", wp.spatial_vector)
actuator_acc0: array("*", "nactuator", float)
actuator_lengthrange: array("*", "nactuator", wp.vec2)
sensor_type: array("nsensor", int)
sensor_datatype: array("nsensor", int)
sensor_objtype: array("nsensor", int)
sensor_objid: array("nsensor", int)
sensor_reftype: array("nsensor", int)
sensor_refid: array("nsensor", int)
sensor_intprm: array("nsensor", 3, int)
sensor_dim: array("nsensor", int)
sensor_adr: array("nsensor", int)
sensor_cutoff: array("nsensor", float)
sensor_history: array("*", "nsensor", wp.vec2i)
sensor_historyadr: array("*", "nsensor", int)
sensor_delay: array("*", "nsensor", float)
sensor_interval: array("*", "nsensor", wp.vec2)
plugin: array("nplugin", int)
plugin_attr: array("nplugin", vec_pluginattr)
key_time: array("nkey", float)
key_qpos: array("nkey", "nq", float)
key_qvel: array("nkey", "nv", float)
key_act: array("nkey", "na", float)
key_mpos: array("nkey", "nmocap", wp.vec3)
key_mquat: array("nkey", "nmocap", wp.quat)
key_ctrl: array("nkey", "nu", float)
M_rownnz: array("nv", int)
M_rowadr: array("nv", int)
M_colind: array("nC", int)
mapM2M: array("nC", int)
D_rownnz: array("nv", int)
D_rowadr: array("nv", int)
D_diag: array("nv", int)
D_colind: array("nD", int)
mapM2D: array("nD", int)
mapD2M: array("nC", int)
nefmK: int
nefmdof: int
nefmL: int
efm_K_rownnz: array("nv", int)
efm_K_rowadr: array("nv", int)
efm_K_colind: array("nefmK", int)
efm_dofid: array("nefmdof", int)
efm_dofblk: array("nv", int)
callback: Callback
nbranch: int
nv_pad: int
nacttrnbody: int
nsensorcollision: int
nsensortaxel: int
ntactileweld: int
nsensorcontact: int
nrangefinder: int
nmaxcondim: int
nmaxpyramid: int
nflexintcell: int
is_sparse: bool
qLD_block_total: int
qLD_block_adr: array("nv", int)
flg_adhesion: bool
has_fluid: bool
flg_surfacevel: bool
has_sdf_geom: bool
has_flex_selfcollide: bool
has_flex_passive: bool
has_flex_snh: bool
has_non_simple_flex: bool
has_tendon_stiffness: bool
has_tendon_damping: bool
has_efm_actuator: bool
efm0_active: bool
flex_interp_assemblable: bool
has_unsupported_flex_interp: bool
has_ellipsoid_geom: bool
has_plane_geom: bool
has_1d_flex: bool
has_2d_flex: bool
has_3d_flex: bool
max_flex_dim: int
block_dim: BlockDim
body_tree: tuple[array("nbody", int), ...]
body_branches: array("nbody_branches", int)
body_branch_start: array("nbranch_start", int)
mocap_bodyid: array("nmocap", int)
body_fluid_ellipsoid: array("nbody", bool)
body_is_free: array("nbody", bool)
body_fluid_ellipsoid_adr: array("nbody_fluid_ellipsoid", int)
body_fluid_box_adr: array("nbody_fluid_box", int)
body_freeadr: array("nbodyfree", int)
jnt_limited_slide_hinge_adr: array("njnt_limited_slide_hinge", int)
jnt_limited_ball_adr: array("njnt_limited_ball", int)
body_isdofancestor: array("nbody", "nv_pad", int)
dof_tri_row: array("ndof_tri", int)
dof_tri_col: array("ndof_tri", int)
nxn_geom_pair: array("nnxn_geom_pair", wp.vec2i)
nxn_geom_pair_filtered: array("nnxn_geom_pair_filtered", wp.vec2i)
nxn_pairid: array("nnxn_geom_pair", wp.vec2i)
nxn_pairid_filtered: array("nnxn_geom_pair_filtered", wp.vec2i)
geom_pair_type_count: tuple[int, ...]
geom_plugin_index: array("ngeom", int)
eq_connect_adr: array("neq_connect", int)
eq_wld_adr: array("neq_wld", int)
eq_jnt_adr: array("neq_jnt", int)
eq_ten_adr: array("neq_ten", int)
eq_flex_adr: array("neq_flex", int)
eq_flexstrain_adr: array("neq_flexstrain", int)
tendon_jnt_adr: array("ntendon_jnt", int)
tendon_site_pair_adr: array("ntendon_site_pair", int)
tendon_geom_adr: array("ntendon_geom", int)
tendon_limited_adr: array("ntendon_limited", int)
max_ten_J_rownnz: int
ten_wrapadr_site: array("nten_wrapadr_site", int)
ten_wrapnum_site: array("ntendon", int)
wrap_jnt_adr: array("nwrap_jnt", int)
wrap_site_adr: array("nwrap_site", int)
wrap_site_pair_adr: array("nwrap_site_pair", int)
wrap_geom_adr: array("nwrap_geom", int)
wrap_pulley_scale: array("nwrap", float)
actuator_trntype_body_adr: array("nacttrnbody", int)
sensor_pos_adr: array("nsensor_pos", int)
sensor_limitpos_adr: array("nsensor_limitpos", int)
sensor_vel_adr: array("nsensor_vel", int)
sensor_limitvel_adr: array("nsensor_limitvel", int)
sensor_acc_adr: array("nsensor_acc", int)
sensor_rangefinder_adr: array("nrangefinder", int)
rangefinder_sensor_adr: array("nsensor", int)
sensor_collision_start_adr: array("nsensor_collision_start_adr", int)
collision_sensor_adr: array("nsensor", int)
sensor_touch_adr: array("nsensor_touch", int)
sensor_limitfrc_adr: array("nsensor_limitfrc", int)
sensor_e_potential: bool
sensor_e_kinetic: bool
sensor_tendonactfrc_adr: array("nsensor_tendonactfrc", int)
sensor_subtree_vel: bool
sensor_contact_adr: array("nsensorcontact", int)
sensor_adr_to_contact_adr: array("nsensor", int)
sensor_rne_postconstraint: bool
sensor_rangefinder_bodyid: array("nrangefinder", int)
weld_tactile_id: array("nbody", int)
taxel_vertadr: array("nsensortaxel", int)
taxel_sensorid: array("nsensortaxel", int)
M_tiles: tuple[TileSet, ...]
qLD_updates: tuple[array("nqLD_all_updates", wp.vec3i), ...]
qLD_all_updates: array("nqLD_all_updates", wp.vec3i)
qLD_level_offsets: array("nqLD_level_offsets", int)
M_fullm_i: array("nM_fullm", int)
M_fullm_j: array("nM_fullm", int)
M_elemid: array("nv", "nv", int)  # (row, col) -> CSR madr address; -1 if col is not a chain ancestor of row
M_hinit_i: array("nC", int)  # row index of each CSR M entry (for densifying M into the dense Newton H)
M_fullm_upper_i: array("nM_fullm_upper", int)
M_fullm_upper_j: array("nM_fullm_upper", int)
M_fullm_upper_elemid: array("nM_fullm_upper", int)
qD_fullm_i: array("nqD_fullm", int)  # D-structure (full square) row indices for RNE derivatives
qD_fullm_j: array("nqD_fullm", int)  # D-structure (full square) column indices for RNE derivatives
M_mulm_rowadr: array("nv_plus_1", int)  # start address for each row [nv+1]
M_mulm_col: array("nM_mulm", int)  # column index to gather from
M_mulm_madr: array("nM_mulm", int)  # matrix address to read
flex_elemflexid: array("nflexelem", int)
flex_edgeflexid: array("nflexedge", int)
flex_shellflexid: array("nflexshelldata", int)
flex_vertflexid: array("nflexvert", int)
flex_shelladr: array("nflex", int)
flex_faceadr: array("nflex", int)
flex_simple: array("nflex", bool)
flex_cell_map: array("nflexintcell", wp.vec4i)
flexstrain_J_rownnz: array("neq_flexstrain", int)
flexstrain_J_rowadr: array("neq_flexstrain", int)
flexstrain_J_colind: array("nJfs", int)
neq_flexstrain: int
nJfs: int
nflexbend_interp: int
flex_bend_interp_map: array("nflexbend_interp", wp.vec2i)
nflexface: int
flex_face_map: array("nflexface", wp.vec2i)
flex_face: array("nflexface", 9, int)
```

#### Contact

声明数：18。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L2150-L2194)。

```text
dist: array("naconmax", float)
pos: array("naconmax", wp.vec3)
frame: array("naconmax", wp.mat33)
includemargin: array("naconmax", float)
friction: array("naconmax", vec5)
solref: array("naconmax", wp.vec2)
solreffriction: array("naconmax", wp.vec2)
solimp: array("naconmax", vec5)
dim: array("naconmax", int)
geom: array("naconmax", wp.vec2i)
flex: array("naconmax", wp.vec2i)
elem: array("naconmax", wp.vec2i)
vert: array("naconmax", wp.vec2i)
efc_address: array("naconmax", "nmaxpyramid", int)
worldid: array("naconmax", int)
type: array("naconmax", int)
geomcollisionid: array("naconmax", int)
adhesion: array("naconmax", float)
```

#### Constraint

声明数：20。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L2196-L2248)。

```text
type: array("nworld", "njmax", int)
id: array("nworld", "njmax", int)
jtdaj_adr: array("nworld", "njmax", int)
jtdaj_nrow: array("nworld", "njmax", int)
jtdaj_nblock: array("nworld", int)
J_rownnz: array("nworld", "njmax", int)
J_rowadr: array("nworld", "njmax", int)
J_colind: array("nworld", 1, "njmax_nnz", int)
J: array("nworld", 1, "njmax_nnz", float)
pos: array("nworld", "njmax", float)
margin: array("nworld", "njmax", float)
D: array("nworld", "njmax_pad", float)
vel: array("nworld", "njmax", float)
aref: array("nworld", "njmax", float)
frictionloss: array("nworld", "njmax", float)
force: array("nworld", "njmax", float)
state: array("nworld", "njmax_pad", int)
island: array("nworld", "njmax", int)
Ma: array("nworld", "nv", float)
Jqvel: array("nworld", "njmax", float)
```

#### Data

声明数：157。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L2250-L2578)。

```text
solver_niter: array("nworld", int)
ne: array("nworld", int)
nf: array("nworld", int)
nl: array("nworld", int)
nefc: array("nworld", int)
nisland: array("nworld", int)
nidof: array("nworld", int)
ntree_awake: array("nworld", int)
nbody_awake: array("nworld", int)
nv_awake: array("nworld", int)
time: array("nworld", float)
energy: array("nworld", wp.vec2)
qpos: array("nworld", "nq", float)
qvel: array("nworld", "nv", float)
act: array("nworld", "na", float)
history: array("nworld", "nhistory", float)
qacc_warmstart: array("nworld", "nv", float)
ctrl: array("nworld", "nu", float)
qfrc_applied: array("nworld", "nv", float)
xfrc_applied: array("nworld", "nbody", wp.spatial_vector)
eq_active: array("nworld", "neq", bool)
mocap_pos: array("nworld", "nmocap", wp.vec3)
mocap_quat: array("nworld", "nmocap", wp.quat)
qacc: array("nworld", "nv", float)
act_dot: array("nworld", "na", float)
userdata: array("nworld", "nuserdata", float)
sensordata: array("nworld", "nsensordata", float)
tree_asleep: array("nworld", "ntree", int)
xpos: array("nworld", "nbody", wp.vec3)
xquat: array("nworld", "nbody", wp.quat)
xmat: array("nworld", "nbody", wp.mat33)
xipos: array("nworld", "nbody", wp.vec3)
ximat: array("nworld", "nbody", wp.mat33)
xanchor: array("nworld", "njnt", wp.vec3)
xaxis: array("nworld", "njnt", wp.vec3)
geom_xpos: array("nworld", "ngeom", wp.vec3)
geom_xmat: array("nworld", "ngeom", wp.mat33)
site_xpos: array("nworld", "nsite", wp.vec3)
site_xmat: array("nworld", "nsite", wp.mat33)
cam_xpos: array("nworld", "ncam", wp.vec3)
cam_xmat: array("nworld", "ncam", wp.mat33)
light_xpos: array("nworld", "nlight", wp.vec3)
light_xdir: array("nworld", "nlight", wp.vec3)
subtree_com: array("nworld", "nbody", wp.vec3)
cdof: array("nworld", "nv", wp.spatial_vector)
cinert: array("nworld", "nbody", vec10)
flexvert_xpos: array("nworld", "nflexvert", wp.vec3)
flex_hessian_valid: array("nworld", "nflex", bool)
flexvert_hessian: array("nworld", "nflexvert", vec6)
flexedge_hessian: array("nworld", "nflexedge", wp.mat33)
flexedge_J: array("nworld", "nJfe", float)
flexedge_length: array("nworld", "nflexedge", float)
ten_wrapadr: array("nworld", "ntendon", int)
ten_wrapnum: array("nworld", "ntendon", int)
ten_J: array("nworld", "nJten", float)
ten_length: array("nworld", "ntendon", float)
wrap_obj: array("nworld", "nwrap", wp.vec2i)
wrap_xpos: array("nworld", "nwrap", wp.spatial_vector)
actuator_length: array("nworld", "nactuator", float)
moment_rownnz: array("nworld", "nactuator", int)
moment_rowadr: array("nworld", "nactuator", int)
moment_colind: array("nworld", "nJmom", int)
actuator_moment: array("nworld", "nJmom", float)
crb: array("nworld", "nbody", vec10)
M: array("nworld", "nC", float)
qLD: array("nworld", "qld_total", float)
qLDiagInv: array("nworld", "nv", float)
tree_awake: array("nworld", "ntree", int)
body_awake: array("nworld", "nbody", int)
body_awake_ind: array("nworld", "nbody", int)
dof_awake_ind: array("nworld", "nv", int)
flexedge_velocity: array("nworld", "nflexedge", float)
ten_velocity: array("nworld", "ntendon", float)
actuator_velocity: array("nworld", "nactuator", float)
cvel: array("nworld", "nbody", wp.spatial_vector)
cdof_dot: array("nworld", "nv", wp.spatial_vector)
qfrc_bias: array("nworld", "nv", float)
qfrc_spring: array("nworld", "nv", float)
qfrc_damper: array("nworld", "nv", float)
qfrc_gravcomp: array("nworld", "nv", float)
qfrc_fluid: array("nworld", "nv", float)
qfrc_adhesion: array("nworld", "nv", float)
qfrc_passive: array("nworld", "nv", float)
subtree_linvel: array("nworld", "nbody", wp.vec3)
subtree_angmom: array("nworld", "nbody", wp.vec3)
qH: array("nworld", "nC", float)
qHDiagInv: array("nworld", "nv", float)
qLU: array("nworld", "nD", float)
actuator_force: array("nworld", "nactuator", float)
qfrc_actuator: array("nworld", "nv", float)
qfrc_smooth: array("nworld", "nv", float)
qacc_smooth: array("nworld", "nv", float)
qfrc_constraint: array("nworld", "nv", float)
qfrc_inverse: array("nworld", "nv", float)
cacc: array("nworld", "nbody", wp.spatial_vector)
cfrc_int: array("nworld", "nbody", wp.spatial_vector)
cfrc_ext: array("nworld", "nbody", wp.spatial_vector)
efm_c: array("nworld", "nv", float)
efm_diag: array("nworld", "nv", float)
efm_fluid: array("nworld", "nC", float)
efm_ca: array("nworld", "nv", float)
efm_K_val: array("nworld", "nefmK", float)
efm_L: array("nworld", "nefmL", float)
qHLD: array("nworld", "qld_total", float)
efm_ts: array("nworld", "ntendon", float)
efm_as: array("nworld", "nactuator", float)
contact: Contact
efc: Constraint
tree_island: array("nworld", "ntree", int)
dof_island: array("nworld", "nv", int)
island_dofadr: array("nworld", "ntree", int)
island_idofadr: array("nworld", "ntree", int)
island_nv: array("nworld", "ntree", int)
island_nefc: array("nworld", "ntree", int)
island_ne: array("nworld", "ntree", int)
island_nf: array("nworld", "ntree", int)
island_iefcadr: array("nworld", "ntree", int)
map_dof2idof: array("nworld", "nv", int)
map_idof2dof: array("nworld", "nv", int)
map_efc2iefc: array("nworld", "njmax", int)
map_iefc2efc: array("nworld", "njmax", int)
dof_islandid: array("nworld", "nv", int)
efc_islandid: array("nworld", "njmax", int)
ncdof: array("nworld", int)
dof_cdof: array("nworld", "nv", int)
cdof_dof: array("nworld", "nvmax_pad", int)
ctol: array(1, float)
cls_tol: array(1, float)
cdof_tri_row: array("nvmax_pad_sq", int)
cdof_tri_col: array("nvmax_pad_sq", int)
cM: array("nworld", "nvmax_pad", "nvmax_pad", float)
cqLD: array("nworld", "nvmax_pad", "nvmax_pad", float)
crhs: array("nworld", "nvmax_pad", 1, float)
cx: array("nworld", "nvmax_pad", 1, float)
cJ: array("nworld", "njmax_pad", "nvmax_pad", float)
cMa: array("nworld", "nvmax_pad", float)
cqfrc_smooth: array("nworld", "nvmax_pad", float)
cqacc_smooth: array("nworld", "nvmax_pad", float)
cqacc_warmstart: array("nworld", "nvmax_pad", float)
cqacc: array("nworld", "nvmax_pad", float)
cqfrc_constraint: array("nworld", "nvmax_pad", float)
nworld: int
naconmax: int
naccdmax: int
njmax: int
nvmax: int
nvmax_pad: int
njmax_pad: int
njmax_nnz: int
nacon: array(1, int)
ncollision: array(1, int)
flex_aabb_min: array("nworld", "nflex", wp.vec3)
flex_aabb_max: array("nworld", "nflex", wp.vec3)
flexnode_xpos: array("nworld", "nflexnode", wp.vec3)
overflow: array("nworld", int)
face_xpos: array("nworld", "nflexface", 9, wp.vec3)
face_quat: array("nworld", "nflexface", wp.quat)
```

#### InverseContext

声明数：9。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L2580-L2594)。

```text
Jaref: wp.array2d[float]
search_dot: wp.array[float]
done: wp.array[bool]
quad_changed_ids: wp.array2d[int]
quad_changed_count: wp.array[int]
state_changed_count: wp.array[int]
ls_exhausted: wp.array[bool]
compact_m_full: Optional["Model"] = None
compact_d_full: Optional["Data"] = None
```

#### SolverContext

声明数：26。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L2596-L2627)。

```text
Jaref: wp.array2d[float]
search_dot: wp.array[float]
done: wp.array[bool]
grad: wp.array2d[float]
grad_dot: wp.array[float]
newton_decrement: wp.array[float]
Mgrad: wp.array2d[float]
search: wp.array2d[float]
mv: wp.array2d[float]
jv: wp.array2d[float]
quad: wp.array2d[wp.vec3]
alpha: wp.array[float]
grad_scale: wp.array[float]
state_changed_count: wp.array[int]
improvement: wp.array[float]
ls_exhausted: wp.array[bool]
search_unchanged: wp.array[bool]
prev_grad: wp.array2d[float]
prev_Mgrad: wp.array2d[float]
beta: wp.array[float]
h: wp.array3d[float]
hfactor: wp.array3d[float]
quad_changed_ids: wp.array2d[int]
quad_changed_count: wp.array[int]
compact_m_full: Optional["Model"] = None
compact_d_full: Optional["Data"] = None
```

#### RenderContext

声明数：82。
依据：[类型声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L2629-L2825)。

```text
nrender: int
cam_res: array("ncam", wp.vec2i)
cam_id_map: array("ncam", int)
use_textures: bool
use_fast_math: bool
use_shadows: bool
use_ambient_lighting: bool
background_color: wp.uint32
use_precomputed_rays: bool
render_skybox: bool
skybox_tex_id: array("*", int)
skybox_face_width: array("*", int)
headlight_active: bool
headlight_ambient: wp.vec3
headlight_diffuse: wp.vec3
headlight_specular: wp.vec3
bvh_ngeom: int
enabled_geom_ids: array("*", int)
mesh_registry: dict
mesh_bvh_id: array("nmesh", wp.uint64)
mesh_bounds_size: array("nmesh", wp.vec3)
mesh_texcoord: array("*", wp.vec2)
mesh_texcoord_offsets: array("nmesh", int)
mesh_facetexcoord: array("nmeshface", wp.vec3i)
mesh_facenormal: array("nmeshface", wp.vec3i)
samples_per_pixel: int
aa_accum: array("*", wp.vec3)
textures: array("*", wp.Texture2D)
textures_registry: list[wp.Texture2D]
hfield_registry: dict
hfield_bvh_id: array("nhfield", wp.uint64)
hfield_bounds_size: array("nhfield", wp.vec3)
flex_mesh_registry: dict
flex_rgba: array("nflex", wp.vec4)
flex_bvh_id: array("*", wp.uint64)
flex_group_root: array("nworld", "*", int)
flex_render_smooth: bool
bvh_nflexgeom: int
flex_dim_np: array("nflex", int)
flex_geom_flexid: array("*", int)
flex_geom_edgeid: array("*", int)
bvh: wp.Bvh
bvh_id: wp.uint64
lower: array("*", wp.vec3)
upper: array("*", wp.vec3)
group: array("*", int)
group_root: array("*", int)
ray: array("*", wp.vec3)
ray_offset: array("*", wp.vec3)
rgb_data: array("*", wp.uint32)
rgb_adr: array("ncam", int)
depth_data: array("*", wp.float32)
depth_adr: array("ncam", int)
render_rgb: array("ncam", bool)
render_depth: array("ncam", bool)
seg_data: array("*", wp.vec2i)
seg_adr: array("ncam", int)
render_seg: array("ncam", bool)
znear: float
zfar: float
total_rays: int
enable_backface_culling: bool
shadow_light_fraction: float
enable_vertex_normals: bool
enable_specular: bool
enable_emission: bool
enable_per_light_ambient: bool
light_attenuation_is_default: bool
has_spot_lights: bool
has_orthographic_camera: bool
splat_position: array("*", wp.vec3)
splat_rotation: array("*", wp.quat)
splat_scale: array("*", wp.vec3)
splat_rgba: array("*", wp.vec4)
splat_bvh: Optional[wp.Bvh]
splat_lower: array("*", wp.vec3)
splat_upper: array("*", wp.vec3)
splat_bvh_id: wp.uint64
splat_group_root: array("nworld", int)
splat_count: int
geom_ray_types: tuple = ()
_megakernel: Optional[wp.Kernel] = None
```

## 4. 全部枚举声明

这些声明来自冻结types模块。
它们不表示全部组合都支持。
上传及阶段检查另行约束组合。
Rust映射保留成员及位运算语义。
不支持项不扩大为整族排除。

| 枚举 | 声明数 |
| --- | --- |
| `BroadphaseType` | 3 |
| `BroadphaseFilter` | 4 |
| `OverflowType` | 14 |
| `CamLightType` | 5 |
| `ProjectionType` | 2 |
| `Stage` | 3 |
| `DataType` | 2 |
| `DisableBit` | 18 |
| `EnableBit` | 3 |
| `SleepPolicy` | 3 |
| `SleepState` | 3 |
| `TrnType` | 6 |
| `DynType` | 7 |
| `GainType` | 6 |
| `BiasType` | 5 |
| `CtrlInput` | 5 |
| `CtrlChart` | 2 |
| `JointType` | 4 |
| `ConeType` | 2 |
| `IntegratorType` | 5 |
| `GeomType` | 11 |
| `CollisionType` | 3 |
| `SolverType` | 2 |
| `ConstraintState` | 5 |
| `ConstraintType` | 8 |
| `SensorType` | 48 |
| `ObjType` | 7 |
| `EqType` | 6 |
| `WrapType` | 5 |
| `State` | 18 |
| `ContactType` | 3 |

### BroadphaseType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L127-L140)。

```text
NXN = 0
SAP_TILE = 1
SAP_SEGMENTED = 2
```

### BroadphaseFilter

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L141-L156)。

```text
PLANE = 1 << 0
SPHERE = 1 << 1
AABB = 1 << 2
OBB = 1 << 3
```

### OverflowType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L157-L205)。

```text
NONE = 0
NEFC = 1 << 0
NJMAX_NNZ = 1 << 1
BROADPHASE = 1 << 2
NARROWPHASE = 1 << 3
CCD = 1 << 4
HFIELD = 1 << 5
CONTACT_MATCH = 1 << 6
NVMAX = 1 << 7
EPA_HORIZON = 1 << 8
ITERATIONS = 1 << 9
LS_ITERATIONS = 1 << 10
TACTILE = 1 << 11
ALL = (
NEFC
| NJMAX_NNZ
| BROADPHASE
| NARROWPHASE
| CCD
| HFIELD
| CONTACT_MATCH
| NVMAX
| EPA_HORIZON
| ITERATIONS
| LS_ITERATIONS
| TACTILE
)
```

### CamLightType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L206-L223)。

```text
FIXED = mujoco.mjtCamLight.mjCAMLIGHT_FIXED
TRACK = mujoco.mjtCamLight.mjCAMLIGHT_TRACK
TRACKCOM = mujoco.mjtCamLight.mjCAMLIGHT_TRACKCOM
TARGETBODY = mujoco.mjtCamLight.mjCAMLIGHT_TARGETBODY
TARGETBODYCOM = mujoco.mjtCamLight.mjCAMLIGHT_TARGETBODYCOM
```

### ProjectionType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L224-L235)。

```text
PERSPECTIVE = mujoco.mjtProjection.mjPROJ_PERSPECTIVE
ORTHOGRAPHIC = mujoco.mjtProjection.mjPROJ_ORTHOGRAPHIC
```

### Stage

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L236-L249)。

```text
POS = mujoco.mjtStage.mjSTAGE_POS
VEL = mujoco.mjtStage.mjSTAGE_VEL
ACC = mujoco.mjtStage.mjSTAGE_ACC
```

### DataType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L250-L262)。

```text
REAL = mujoco.mjtDataType.mjDATATYPE_REAL
POSITIVE = mujoco.mjtDataType.mjDATATYPE_POSITIVE
```

### DisableBit

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L263-L307)。

```text
CONSTRAINT = mujoco.mjtDisableBit.mjDSBL_CONSTRAINT
EQUALITY = mujoco.mjtDisableBit.mjDSBL_EQUALITY
FRICTIONLOSS = mujoco.mjtDisableBit.mjDSBL_FRICTIONLOSS
LIMIT = mujoco.mjtDisableBit.mjDSBL_LIMIT
CONTACT = mujoco.mjtDisableBit.mjDSBL_CONTACT
SPRING = mujoco.mjtDisableBit.mjDSBL_SPRING
DAMPER = mujoco.mjtDisableBit.mjDSBL_DAMPER
GRAVITY = mujoco.mjtDisableBit.mjDSBL_GRAVITY
CLAMPCTRL = mujoco.mjtDisableBit.mjDSBL_CLAMPCTRL
WARMSTART = mujoco.mjtDisableBit.mjDSBL_WARMSTART
FILTERPARENT = mujoco.mjtDisableBit.mjDSBL_FILTERPARENT
ACTUATION = mujoco.mjtDisableBit.mjDSBL_ACTUATION
REFSAFE = mujoco.mjtDisableBit.mjDSBL_REFSAFE
SENSOR = mujoco.mjtDisableBit.mjDSBL_SENSOR
EULERDAMP = mujoco.mjtDisableBit.mjDSBL_EULERDAMP
NATIVECCD = mujoco.mjtDisableBit.mjDSBL_NATIVECCD
ISLAND = mujoco.mjtDisableBit.mjDSBL_ISLAND
MULTICCD = mujoco.mjtDisableBit.mjDSBL_MULTICCD
```

### EnableBit

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L308-L322)。

```text
ENERGY = mujoco.mjtEnableBit.mjENBL_ENERGY
INVDISCRETE = mujoco.mjtEnableBit.mjENBL_INVDISCRETE
SLEEP = mujoco.mjtEnableBit.mjENBL_SLEEP
```

### SleepPolicy

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L323-L337)。

```text
AUTO = mujoco.mjtSleepPolicy.mjSLEEP_AUTO
AUTO_NEVER = mujoco.mjtSleepPolicy.mjSLEEP_AUTO_NEVER
AUTO_ALLOWED = mujoco.mjtSleepPolicy.mjSLEEP_AUTO_ALLOWED
```

### SleepState

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L338-L351)。

```text
STATIC = mujoco.mjtSleepState.mjS_STATIC
ASLEEP = mujoco.mjtSleepState.mjS_ASLEEP
AWAKE = mujoco.mjtSleepState.mjS_AWAKE
```

### TrnType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L352-L371)。

```text
JOINT = mujoco.mjtTrn.mjTRN_JOINT
JOINTINPARENT = mujoco.mjtTrn.mjTRN_JOINTINPARENT
SLIDERCRANK = mujoco.mjtTrn.mjTRN_SLIDERCRANK
TENDON = mujoco.mjtTrn.mjTRN_TENDON
BODY = mujoco.mjtTrn.mjTRN_BODY
SITE = mujoco.mjtTrn.mjTRN_SITE
```

### DynType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L372-L393)。

```text
NONE = mujoco.mjtDyn.mjDYN_NONE
INTEGRATOR = mujoco.mjtDyn.mjDYN_INTEGRATOR
FILTER = mujoco.mjtDyn.mjDYN_FILTER
FILTEREXACT = mujoco.mjtDyn.mjDYN_FILTEREXACT
MUSCLE = mujoco.mjtDyn.mjDYN_MUSCLE
USER = mujoco.mjtDyn.mjDYN_USER
DCMOTOR = mujoco.mjtDyn.mjDYN_DCMOTOR
```

### GainType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L394-L412)。

```text
FIXED = mujoco.mjtGain.mjGAIN_FIXED
AFFINE = mujoco.mjtGain.mjGAIN_AFFINE
MUSCLE = mujoco.mjtGain.mjGAIN_MUSCLE
USER = mujoco.mjtGain.mjGAIN_USER
DCMOTOR = mujoco.mjtGain.mjGAIN_DCMOTOR
SO3 = mujoco.mjtGain.mjGAIN_SO3
```

### BiasType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L413-L430)。

```text
NONE = mujoco.mjtBias.mjBIAS_NONE
AFFINE = mujoco.mjtBias.mjBIAS_AFFINE
MUSCLE = mujoco.mjtBias.mjBIAS_MUSCLE
USER = mujoco.mjtBias.mjBIAS_USER
DCMOTOR = mujoco.mjtBias.mjBIAS_DCMOTOR
```

### CtrlInput

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L431-L448)。

```text
POS = mujoco.mjtCtrlInput.mjINPUT_POS
VEL = mujoco.mjtCtrlInput.mjINPUT_VEL
FF = mujoco.mjtCtrlInput.mjINPUT_FF
VOLTAGE = mujoco.mjtCtrlInput.mjINPUT_VOLTAGE
NONE = mujoco.mjtCtrlInput.mjINPUT_NONE
```

### CtrlChart

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L449-L460)。

```text
EXPMAP = mujoco.mjtCtrlChart.mjCHART_EXPMAP
QUAT = mujoco.mjtCtrlChart.mjCHART_QUAT
```

### JointType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L461-L476)。

```text
FREE = mujoco.mjtJoint.mjJNT_FREE
BALL = mujoco.mjtJoint.mjJNT_BALL
SLIDE = mujoco.mjtJoint.mjJNT_SLIDE
HINGE = mujoco.mjtJoint.mjJNT_HINGE
```

### ConeType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L477-L488)。

```text
PYRAMIDAL = mujoco.mjtCone.mjCONE_PYRAMIDAL
ELLIPTIC = mujoco.mjtCone.mjCONE_ELLIPTIC
```

### IntegratorType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L489-L506)。

```text
EULER = mujoco.mjtIntegrator.mjINT_EULER
RK4 = mujoco.mjtIntegrator.mjINT_RK4
IMPLICITFAST = mujoco.mjtIntegrator.mjINT_IMPLICITFAST
IMPLICIT = mujoco.mjtIntegrator.mjINT_IMPLICIT
DISCRETE = mujoco.mjtIntegrator.mjINT_DISCRETE
```

### GeomType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L507-L537)。

```text
PLANE = mujoco.mjtGeom.mjGEOM_PLANE
HFIELD = mujoco.mjtGeom.mjGEOM_HFIELD
SPHERE = mujoco.mjtGeom.mjGEOM_SPHERE
CAPSULE = mujoco.mjtGeom.mjGEOM_CAPSULE
ELLIPSOID = mujoco.mjtGeom.mjGEOM_ELLIPSOID
CYLINDER = mujoco.mjtGeom.mjGEOM_CYLINDER
BOX = mujoco.mjtGeom.mjGEOM_BOX
MESH = mujoco.mjtGeom.mjGEOM_MESH
SDF = mujoco.mjtGeom.mjGEOM_SDF
FLEX = mujoco.mjtGeom.mjGEOM_FLEX
TRIANGLE = 999
```

### CollisionType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L538-L551)。

```text
PRIMITIVE = 0
CONVEX = 1
SDF = 2
```

### SolverType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L552-L564)。

```text
CG = mujoco.mjtSolver.mjSOL_CG
NEWTON = mujoco.mjtSolver.mjSOL_NEWTON
```

### ConstraintState

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L565-L582)。

```text
SATISFIED = mujoco.mjtConstraintState.mjCNSTRSTATE_SATISFIED
QUADRATIC = mujoco.mjtConstraintState.mjCNSTRSTATE_QUADRATIC
LINEARNEG = mujoco.mjtConstraintState.mjCNSTRSTATE_LINEARNEG
LINEARPOS = mujoco.mjtConstraintState.mjCNSTRSTATE_LINEARPOS
CONE = mujoco.mjtConstraintState.mjCNSTRSTATE_CONE
```

### ConstraintType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L583-L606)。

```text
EQUALITY = mujoco.mjtConstraint.mjCNSTR_EQUALITY
FRICTION_DOF = mujoco.mjtConstraint.mjCNSTR_FRICTION_DOF
FRICTION_TENDON = mujoco.mjtConstraint.mjCNSTR_FRICTION_TENDON
LIMIT_JOINT = mujoco.mjtConstraint.mjCNSTR_LIMIT_JOINT
LIMIT_TENDON = mujoco.mjtConstraint.mjCNSTR_LIMIT_TENDON
CONTACT_FRICTIONLESS = mujoco.mjtConstraint.mjCNSTR_CONTACT_FRICTIONLESS
CONTACT_PYRAMIDAL = mujoco.mjtConstraint.mjCNSTR_CONTACT_PYRAMIDAL
CONTACT_ELLIPTIC = mujoco.mjtConstraint.mjCNSTR_CONTACT_ELLIPTIC
```

### SensorType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L607-L710)。

```text
MAGNETOMETER = mujoco.mjtSensor.mjSENS_MAGNETOMETER
CAMPROJECTION = mujoco.mjtSensor.mjSENS_CAMPROJECTION
RANGEFINDER = mujoco.mjtSensor.mjSENS_RANGEFINDER
JOINTPOS = mujoco.mjtSensor.mjSENS_JOINTPOS
TENDONPOS = mujoco.mjtSensor.mjSENS_TENDONPOS
ACTUATORPOS = mujoco.mjtSensor.mjSENS_ACTUATORPOS
BALLQUAT = mujoco.mjtSensor.mjSENS_BALLQUAT
JOINTLIMITPOS = mujoco.mjtSensor.mjSENS_JOINTLIMITPOS
TENDONLIMITPOS = mujoco.mjtSensor.mjSENS_TENDONLIMITPOS
FRAMEPOS = mujoco.mjtSensor.mjSENS_FRAMEPOS
FRAMEXAXIS = mujoco.mjtSensor.mjSENS_FRAMEXAXIS
FRAMEYAXIS = mujoco.mjtSensor.mjSENS_FRAMEYAXIS
FRAMEZAXIS = mujoco.mjtSensor.mjSENS_FRAMEZAXIS
FRAMEQUAT = mujoco.mjtSensor.mjSENS_FRAMEQUAT
SUBTREECOM = mujoco.mjtSensor.mjSENS_SUBTREECOM
GEOMDIST = mujoco.mjtSensor.mjSENS_GEOMDIST
GEOMNORMAL = mujoco.mjtSensor.mjSENS_GEOMNORMAL
GEOMFROMTO = mujoco.mjtSensor.mjSENS_GEOMFROMTO
INSIDESITE = mujoco.mjtSensor.mjSENS_INSIDESITE
E_POTENTIAL = mujoco.mjtSensor.mjSENS_E_POTENTIAL
E_KINETIC = mujoco.mjtSensor.mjSENS_E_KINETIC
CLOCK = mujoco.mjtSensor.mjSENS_CLOCK
VELOCIMETER = mujoco.mjtSensor.mjSENS_VELOCIMETER
GYRO = mujoco.mjtSensor.mjSENS_GYRO
JOINTVEL = mujoco.mjtSensor.mjSENS_JOINTVEL
TENDONVEL = mujoco.mjtSensor.mjSENS_TENDONVEL
ACTUATORVEL = mujoco.mjtSensor.mjSENS_ACTUATORVEL
BALLANGVEL = mujoco.mjtSensor.mjSENS_BALLANGVEL
JOINTLIMITVEL = mujoco.mjtSensor.mjSENS_JOINTLIMITVEL
TENDONLIMITVEL = mujoco.mjtSensor.mjSENS_TENDONLIMITVEL
FRAMELINVEL = mujoco.mjtSensor.mjSENS_FRAMELINVEL
FRAMEANGVEL = mujoco.mjtSensor.mjSENS_FRAMEANGVEL
SUBTREELINVEL = mujoco.mjtSensor.mjSENS_SUBTREELINVEL
SUBTREEANGMOM = mujoco.mjtSensor.mjSENS_SUBTREEANGMOM
TOUCH = mujoco.mjtSensor.mjSENS_TOUCH
CONTACT = mujoco.mjtSensor.mjSENS_CONTACT
ACCELEROMETER = mujoco.mjtSensor.mjSENS_ACCELEROMETER
FORCE = mujoco.mjtSensor.mjSENS_FORCE
TORQUE = mujoco.mjtSensor.mjSENS_TORQUE
ACTUATORFRC = mujoco.mjtSensor.mjSENS_ACTUATORFRC
TENDONACTFRC = mujoco.mjtSensor.mjSENS_TENDONACTFRC
JOINTACTFRC = mujoco.mjtSensor.mjSENS_JOINTACTFRC
JOINTLIMITFRC = mujoco.mjtSensor.mjSENS_JOINTLIMITFRC
TENDONLIMITFRC = mujoco.mjtSensor.mjSENS_TENDONLIMITFRC
FRAMELINACC = mujoco.mjtSensor.mjSENS_FRAMELINACC
FRAMEANGACC = mujoco.mjtSensor.mjSENS_FRAMEANGACC
TACTILE = mujoco.mjtSensor.mjSENS_TACTILE
USER = mujoco.mjtSensor.mjSENS_USER
```

### ObjType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L711-L732)。

```text
UNKNOWN = mujoco.mjtObj.mjOBJ_UNKNOWN
BODY = mujoco.mjtObj.mjOBJ_BODY
XBODY = mujoco.mjtObj.mjOBJ_XBODY
GEOM = mujoco.mjtObj.mjOBJ_GEOM
FLEX = mujoco.mjtObj.mjOBJ_FLEX
SITE = mujoco.mjtObj.mjOBJ_SITE
CAMERA = mujoco.mjtObj.mjOBJ_CAMERA
```

### EqType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L733-L753)。

```text
CONNECT = mujoco.mjtEq.mjEQ_CONNECT
WELD = mujoco.mjtEq.mjEQ_WELD
JOINT = mujoco.mjtEq.mjEQ_JOINT
TENDON = mujoco.mjtEq.mjEQ_TENDON
FLEX = mujoco.mjtEq.mjEQ_FLEX
FLEXSTRAIN = mujoco.mjtEq.mjEQ_FLEXSTRAIN
```

### WrapType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L754-L771)。

```text
JOINT = mujoco.mjtWrap.mjWRAP_JOINT
PULLEY = mujoco.mjtWrap.mjWRAP_PULLEY
SITE = mujoco.mjtWrap.mjWRAP_SITE
SPHERE = mujoco.mjtWrap.mjWRAP_SPHERE
CYLINDER = mujoco.mjtWrap.mjWRAP_CYLINDER
```

### State

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L772-L817)。

```text
TIME = mujoco.mjtState.mjSTATE_TIME
QPOS = mujoco.mjtState.mjSTATE_QPOS
QVEL = mujoco.mjtState.mjSTATE_QVEL
ACT = mujoco.mjtState.mjSTATE_ACT
HISTORY = mujoco.mjtState.mjSTATE_HISTORY
WARMSTART = mujoco.mjtState.mjSTATE_WARMSTART
CTRL = mujoco.mjtState.mjSTATE_CTRL
QFRC_APPLIED = mujoco.mjtState.mjSTATE_QFRC_APPLIED
XFRC_APPLIED = mujoco.mjtState.mjSTATE_XFRC_APPLIED
EQ_ACTIVE = mujoco.mjtState.mjSTATE_EQ_ACTIVE
MOCAP_POS = mujoco.mjtState.mjSTATE_MOCAP_POS
MOCAP_QUAT = mujoco.mjtState.mjSTATE_MOCAP_QUAT
NSTATE = mujoco.mjtState.mjNSTATE
PHYSICS = mujoco.mjtState.mjSTATE_PHYSICS  # includes HISTORY
FULLPHYSICS = mujoco.mjtState.mjSTATE_FULLPHYSICS
USER = mujoco.mjtState.mjSTATE_USER
INTEGRATION = mujoco.mjtState.mjSTATE_INTEGRATION
USERDATA = mujoco.mjtState.mjSTATE_USERDATA
```

### ContactType

依据：[枚举声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py#L2135-L2148)。

```text
CONSTRAINT = 1 << 0
SENSOR = 1 << 1
PASSIVE = 1 << 2
```

## 5. 全部公开函数签名

U编号沿用公开映射表。
下列签名全部属于上游。
Python类型只表达来源语义。
本产品不引入Python调用链。
Rust等价签名仍需实现验证。
必要参数与默认参数分别保留。
类型Optional不表示可以省略参数。

### U001 forward.step

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2101-L2101)。

```text
def step(m: Model, d: Data):
```

### U004 bvh.refit_bvh

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/bvh.py#L39-L39)。

```text
def refit_bvh(m: Model, d: Data, rc: RenderContext):
```

### U005 bvh.refit_splat_bvh

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/bvh.py#L1265-L1265)。

```text
def refit_splat_bvh(rc: RenderContext):
```

### U006 collision_driver.collision

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/collision_driver.py#L927-L931)。

```text
def collision(
  m: Model,
  d: Data,
  awake_prev: Optional[wp.array] = None,
):
```

### U007 collision_driver.nxn_broadphase

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/collision_driver.py#L829-L834)。

```text
def nxn_broadphase(
  m: Model,
  d: Data,
  ctx: CollisionContext,
  awake_prev: Optional[wp.array] = None,
):
```

### U008 collision_driver.sap_broadphase

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/collision_driver.py#L595-L600)。

```text
def sap_broadphase(
  m: Model,
  d: Data,
  ctx: CollisionContext,
  awake_prev: Optional[wp.array] = None,
):
```

### U009 collision_primitive.primitive_narrowphase

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/collision_primitive.py#L1521-L1521)。

```text
def primitive_narrowphase(m: Model, d: Data, ctx: CollisionContext, collision_table: list[tuple[GeomType, GeomType]]):
```

### U010 collision_sdf.sdf_narrowphase

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/collision_sdf.py#L1029-L1029)。

```text
def sdf_narrowphase(m: Model, d: Data, ctx: CollisionContext):
```

### U011 constraint.make_constraint

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/constraint.py#L5042-L5042)。

```text
def make_constraint(m: types.Model, d: types.Data):
```

### U012 derivative.deriv_smooth_vel

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/derivative.py#L1187-L1187)。

```text
def deriv_smooth_vel(m: Model, d: Data, out: wp.array2d[float]):
```

### U013 forward.discrete

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2095-L2095)。

```text
def discrete(m: Model, d: Data):
```

### U014 forward.euler

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L437-L437)。

```text
def euler(m: Model, d: Data):
```

### U015 forward.forward

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2063-L2063)。

```text
def forward(m: Model, d: Data):
```

### U016 forward.fwd_acceleration

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2010-L2010)。

```text
def fwd_acceleration(m: Model, d: Data, factorize: bool = False):
```

### U017 forward.fwd_actuation

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1867-L1867)。

```text
def fwd_actuation(m: Model, d: Data):
```

### U018 forward.fwd_kinematics

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1302-L1302)。

```text
def fwd_kinematics(m: Model, d: Data):
```

### U019 forward.fwd_position

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1322-L1322)。

```text
def fwd_position(m: Model, d: Data, factorize: bool = True):
```

### U020 forward.fwd_velocity

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1421-L1421)。

```text
def fwd_velocity(m: Model, d: Data):
```

### U021 forward.implicit

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L1247-L1247)。

```text
def implicit(m: Model, d: Data):
```

### U022 forward.rungekutta4

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L573-L573)。

```text
def rungekutta4(m: Model, d: Data):
```

### U023 forward.step1

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2118-L2118)。

```text
def step1(m: Model, d: Data):
```

### U024 forward.step2

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2138-L2138)。

```text
def step2(m: Model, d: Data):
```

### U025 history.init_ctrl_history

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L1010-L1016)。

```text
def init_ctrl_history(
  m: Model,
  d: Data,
  ctrlid: int,
  times: Optional[wp.array],
  values: wp.array2d[float],
):
```

### U026 history.init_sensor_history

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L1111-L1118)。

```text
def init_sensor_history(
  m: Model,
  d: Data,
  sensorid: int,
  times: Optional[wp.array],
  values: wp.array2d[float],
  phase: Optional[Union[float, wp.array]] = None,
):
```

### U027 history.read_ctrl

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L846-L853)。

```text
def read_ctrl(
  m: Model,
  d: Data,
  ctrlid: int,
  time: wp.array[float],
  interp: int,
  result: wp.array,
):
```

### U028 history.read_sensor

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L933-L940)。

```text
def read_sensor(
  m: Model,
  d: Data,
  sensorid: int,
  time: wp.array[float],
  interp: int,
  result: wp.array2d[float],
):
```

### U029 history.reset_history

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/history.py#L662-L666)。

```text
def reset_history(
  m: Model,
  d: Data,
  reset: Optional[wp.array] = None,
):
```

### U030 inverse.inverse

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/inverse.py#L178-L178)。

```text
def inverse(m: Model, d: Data):
```

### U031 io.get_data_into

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2396-L2401)。

```text
def get_data_into(
  result: mujoco.MjData,
  mjm: mujoco.MjModel,
  d: types.Data,
  world_id: int = 0,
):
```

### U032 io.make_data

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L1855-L1865)。

```text
def make_data(
  mjm: mujoco.MjModel,
  nworld: int = 1,
  nconmax: Optional[int] = None,
  nccdmax: Optional[int] = None,
  njmax: Optional[int] = None,
  njmax_nnz: Optional[int] = None,
  naconmax: Optional[int] = None,
  naccdmax: Optional[int] = None,
  nvmax: Optional[int] = None,
) -> types.Data:
```

### U033 io.put_data

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2075-L2086)。

```text
def put_data(
  mjm: mujoco.MjModel,
  mjd: mujoco.MjData,
  nworld: int = 1,
  nconmax: Optional[int] = None,
  nccdmax: Optional[int] = None,
  njmax: Optional[int] = None,
  njmax_nnz: Optional[int] = None,
  naconmax: Optional[int] = None,
  naccdmax: Optional[int] = None,
  nvmax: Optional[int] = None,
) -> types.Data:
```

### U034 io.put_model

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L288-L288)。

```text
def put_model(mjm: mujoco.MjModel, batch_sizes: dict[str, int] | None = None) -> types.Model:
```

### U035 io.reset_data

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2670-L2670)。

```text
def reset_data(m: types.Model, d: types.Data, reset: Optional[wp.array] = None):
```

### U036 io.reset_data_keyframe

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L3041-L3041)。

```text
def reset_data_keyframe(m: types.Model, d: types.Data, key: int | wp.array):
```

### U037 island.island

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/island.py#L361-L361)。

```text
def island(m: types.Model, d: types.Data):
```

### U038 passive.passive

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/passive.py#L3390-L3390)。

```text
def passive(m: Model, d: Data):
```

### U039 ray.ray

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/ray.py#L1179-L1188)。

```text
def ray(
  m: Model,
  d: Data,
  pnt: wp.array2d[wp.vec3],
  vec: wp.array2d[wp.vec3],
  geomgroup: vec6 | None = None,
  flg_static: bool = True,
  bodyexclude: int = -1,
  rc: RenderContext | None = None,
) -> Tuple[wp.array, wp.array, wp.array]:
```

### U040 ray.rays

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/ray.py#L1226-L1238)。

```text
def rays(
  m: Model,
  d: Data,
  pnt: wp.array2d[wp.vec3],
  vec: wp.array2d[wp.vec3],
  geomgroup: vec6,
  flg_static: bool,
  bodyexclude: wp.array[int],
  dist: wp.array2d[float],
  geomid: wp.array2d[int],
  normal: wp.array2d[wp.vec3],
  rc: RenderContext | None = None,
):
```

### U041 render.render

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render.py#L1491-L1491)。

```text
def render(m: Model, d: Data, rc: RenderContext):
```

### U042 render_util.create_render_context

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L312-L342)。

```text
def create_render_context(
  mjm: mujoco.MjModel,
  nworld: int = 1,
  cam_res: list[tuple[int, int]] | tuple[int, int] | None = None,
  render_rgb: list[bool] | bool | None = None,
  render_depth: list[bool] | bool | None = None,
  render_seg: list[bool] | bool | None = None,
  use_textures: bool = True,
  use_fast_math: bool = True,
  use_shadows: bool = False,
  use_ambient_lighting: bool = True,
  enabled_geom_groups: list[int] = [0, 1, 2],
  cam_active: list[bool] | list[str] | list[int] | None = None,
  background_color: tuple[float, float, float, float] = (0.0, 0.0, 0.0, 1.0),
  flex_render_smooth: bool = True,
  use_precomputed_rays: bool = True,
  render_skybox: bool = False,
  enable_backface_culling: bool = True,
  shadow_light_fraction: float = 0.3,
  samples_per_pixel: int = 1,
  enable_vertex_normals: bool = True,
  enable_specular: bool = True,
  enable_emission: bool = True,
  enable_per_light_ambient: bool = True,
  splat_position: np.ndarray | None = None,
  splat_rotation: np.ndarray | None = None,
  splat_scale: np.ndarray | None = None,
  splat_rgba: np.ndarray | None = None,
  splat_adr: np.ndarray | None = None,
  splat_group_id: np.ndarray | None = None,
) -> RenderContext:
```

### U043 render_util.get_depth

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L228-L228)。

```text
def get_depth(rc: RenderContext, camera_index: int, depth_scale: float, depth_out: wp.array3d[float]):
```

### U044 render_util.get_rgb

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L212-L212)。

```text
def get_rgb(rc: RenderContext, camera_index: int, rgb_out: wp.array3d[wp.vec3]):
```

### U045 render_util.get_segmentation

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L264-L264)。

```text
def get_segmentation(rc: RenderContext, camera_index: int, seg_out: wp.array3d[wp.vec2i]):
```

### U046 sensor.energy_pos

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sensor.py#L3048-L3048)。

```text
def energy_pos(m: Model, d: Data):
```

### U047 sensor.energy_vel

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sensor.py#L3131-L3131)。

```text
def energy_vel(m: Model, d: Data):
```

### U048 sensor.sensor_acc

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sensor.py#L2583-L2583)。

```text
def sensor_acc(m: Model, d: Data, skip_rne_postconstraint: bool = False):
```

### U049 sensor.sensor_pos

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sensor.py#L814-L814)。

```text
def sensor_pos(m: Model, d: Data):
```

### U050 sensor.sensor_vel

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sensor.py#L1436-L1436)。

```text
def sensor_vel(m: Model, d: Data):
```

### U051 set_const.set_const

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/set_const.py#L852-L852)。

```text
def set_const(m: types.Model, d: types.Data, restore: bool = True):
```

### U052 set_const.set_const_0

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/set_const.py#L616-L616)。

```text
def set_const_0(m: types.Model, d: types.Data, restore: bool = True):
```

### U053 set_const.set_const_fixed

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/set_const.py#L595-L595)。

```text
def set_const_fixed(m: types.Model, d: types.Data):
```

### U054 set_const.set_const_spring

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/set_const.py#L818-L818)。

```text
def set_const_spring(m: types.Model, d: types.Data, restore: bool = True):
```

### U055 set_const.set_length_range

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/set_const.py#L923-L923)。

```text
def set_length_range(m: types.Model, d: types.Data, index: int = -1):
```

### U056 smooth.camlight

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L955-L955)。

```text
def camlight(m: Model, d: Data):
```

### U057 smooth.com_pos

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L795-L795)。

```text
def com_pos(m: Model, d: Data):
```

### U058 smooth.com_vel

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L2591-L2591)。

```text
def com_vel(m: Model, d: Data):
```

### U059 smooth.crb

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L1050-L1050)。

```text
def crb(m: Model, d: Data):
```

### U060 smooth.factor_m

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L1311-L1311)。

```text
def factor_m(m: Model, d: Data):
```

### U061 smooth.flex

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L574-L574)。

```text
def flex(m: Model, d: Data):
```

### U062 smooth.kinematics

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L418-L418)。

```text
def kinematics(m: Model, d: Data):
```

### U063 smooth.rne

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L1470-L1470)。

```text
def rne(m: Model, d: Data, flg_acc: bool = False):
```

### U064 smooth.rne_postconstraint

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L1955-L1955)。

```text
def rne_postconstraint(m: Model, d: Data):
```

### U065 smooth.solve_m

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L3544-L3544)。

```text
def solve_m(m: Model, d: Data, x: wp.array2d[float], y: wp.array2d[float]):
```

### U066 smooth.subtree_vel

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L3943-L3943)。

```text
def subtree_vel(m: Model, d: Data):
```

### U067 smooth.tendon

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L4526-L4526)。

```text
def tendon(m: Model, d: Data):
```

### U068 smooth.transmission

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L3220-L3220)。

```text
def transmission(m: Model, d: Data):
```

### U069 solver.solve

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/solver.py#L3919-L3919)。

```text
def solve(m: types.Model, d: types.Data):
```

### U070 support.contact_force

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L445-L445)。

```text
def contact_force(m: Model, d: Data, contact_ids: wp.array[int], to_world_frame: bool, force: wp.array[wp.spatial_vector]):
```

### U071 support.get_state

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L674-L674)。

```text
def get_state(m: Model, d: Data, state: wp.array2d[float], sig: int, active: Optional[wp.array] = None):
```

### U072 support.jac

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L583-L590)。

```text
def jac(
  m: Model,
  d: Data,
  jacp: wp.array | None,  # wp.array3d[float]
  jacr: wp.array | None,  # wp.array3d[float]
  point: wp.array[wp.vec3],
  body: wp.array[int],
):
```

### U073 support.mul_m

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L218-L225)。

```text
def mul_m(
  m: Model,
  d: Data,
  res: wp.array2d[float],
  vec: wp.array2d[float],
  skip: Optional[wp.array] = None,
  M: Optional[wp.array] = None,
):
```

### U074 support.set_state

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L829-L829)。

```text
def set_state(m: Model, d: Data, state: wp.array2d[float], sig: int, active: Optional[wp.array] = None):
```

### U075 support.xfrc_accumulate

依据：[函数声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L314-L314)。

```text
def xfrc_accumulate(m: Model, d: Data, qfrc: wp.array2d[float]):
```

## 6. 盘点后的关闭条件

声明清单只关闭名称遗漏检查。
完整字段契约还需补充：

- 原生字段来源与版本条件。
- 实际shape、stride与对齐。
- 单位、坐标系与数值精度。
- 索引域、哨兵与变长上限。
- 初始值与首个有效写入阶段。
- 读写权限与阶段副作用。
- 参数修改后的常量刷新规则。
- 序列化、重置与状态签名归属。
- 独立样本与双平台测试证据。

所有895项声明均仍待迁移。
218项成员仍待映射验证。
所有73个函数均尚未实现。
P0-03与RV01继续保持未关闭。
