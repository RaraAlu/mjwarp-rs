+++
uid = "ece0669c"
id = "mjwarp.physics.core"
tags = ["planned", "engine", "physics"]
fingerprint = "eb743d1a79ca3b17d56775543802fe5a545c595bde373bcbe33ebca2c8ffeb0c"

[name]
zh = "物理阶段与基础探针"
en = "Planned stages and foundational probes"

[description]
zh = """
导出G01与G02辅助接口。刚体与质心链固定f32。刚体旋转分组已对齐冻结Warp。关节被动力支持四类关节弹簧子集。其余完整阶段仍计划。
"""
en = """
Exports G01 and strict G02 subset interfaces. Rigid and COM paths remain f32. Rigid rotation grouping now matches frozen Warp. Joint passive covers all four joint spring types as a strict subset; other complete stages remain planned.
"""

[[source]]
path = "src/physics/mod.rs"

[[source]]
path = "src/physics/rne.rs"

[[source]]
path = "src/physics/rne.cu"

[[source]]
path = "src/physics/smooth_force.rs"

[[source]]
path = "src/physics/applied_force.rs"

[[source]]
path = "src/physics/kinetic_energy.rs"

[[source]]
path = "src/physics/smooth_force/joint_passive.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::forward::step"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U001；G16～G19；P6。
"""
en = """
Planned local function contract for upstream forward.step (U001). Matrix: G16～G19. Phase: P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::collision::collision"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U006；G07～G11；P5/P7。
"""
en = """
Planned local function contract for upstream collision_driver.collision (U006). Matrix: G07～G11. Phase: P5/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::collision::nxn_broadphase"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U007；G07～G11；P5/P7。
"""
en = """
Planned local function contract for upstream collision_driver.nxn_broadphase (U007). Matrix: G07～G11. Phase: P5/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::collision::sap_broadphase"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U008；G07～G11；P5/P7。
"""
en = """
Planned local function contract for upstream collision_driver.sap_broadphase (U008). Matrix: G07～G11. Phase: P5/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::collision::primitive_narrowphase"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U009；G08；P5。
"""
en = """
Planned local function contract for upstream collision_primitive.primitive_narrowphase (U009). Matrix: G08. Phase: P5. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::collision::sdf_narrowphase"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U010；G10；P5。
"""
en = """
Planned local function contract for upstream collision_sdf.sdf_narrowphase (U010). Matrix: G10. Phase: P5. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::constraint::make_constraint"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U011；G12、G13；P6。
"""
en = """
Planned local function contract for upstream constraint.make_constraint (U011). Matrix: G12、G13. Phase: P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::derivative::deriv_smooth_vel"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U012；G21；P7。
"""
en = """
Planned local function contract for upstream derivative.deriv_smooth_vel (U012). Matrix: G21. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::integrator::discrete"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U013；G18；P6。
"""
en = """
Planned local function contract for upstream forward.discrete (U013). Matrix: G18. Phase: P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::integrator::euler"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U014；G16；P6。
"""
en = """
Planned local function contract for upstream forward.euler (U014). Matrix: G16. Phase: P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::forward::forward"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U015；G19；P4/P6。
"""
en = """
Planned local function contract for upstream forward.forward (U015). Matrix: G19. Phase: P4/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::forward::fwd_acceleration"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U016；G03、G26；P4/P6。
"""
en = """
Planned local function contract for upstream forward.fwd_acceleration (U016). Matrix: G03、G26. Phase: P4/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::forward::fwd_actuation"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U017；G05；P4/P7。
"""
en = """
Planned local function contract for upstream forward.fwd_actuation (U017). Matrix: G05. Phase: P4/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::forward::fwd_kinematics"

[apis.description]
zh = """
G01入口位于physics根。目标命名空间仍属计划。当前验收另记实际证据。
"""
en = """
G01 implemented via mjwarp_rs::physics::fwd_kinematics; this target namespace remains planned; current validation is recorded separately and other feature families remain planned.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::forward::fwd_position"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U019；G01～G03、G07～G13；P4/P5/P6。
"""
en = """
Planned local function contract for upstream forward.fwd_position (U019). Matrix: G01～G03、G07～G13. Phase: P4/P5/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::forward::fwd_velocity"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U020；G02、G04；P4。
"""
en = """
Planned local function contract for upstream forward.fwd_velocity (U020). Matrix: G02、G04. Phase: P4. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::integrator::implicit"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U021；G17；P6。
"""
en = """
Planned local function contract for upstream forward.implicit (U021). Matrix: G17. Phase: P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::integrator::rungekutta4"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U022；G16；P6。
"""
en = """
Planned local function contract for upstream forward.rungekutta4 (U022). Matrix: G16. Phase: P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::forward::step1"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U023；G19；P4/P6。
"""
en = """
Planned local function contract for upstream forward.step1 (U023). Matrix: G19. Phase: P4/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::forward::step2"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U024；G19；P6。
"""
en = """
Planned local function contract for upstream forward.step2 (U024). Matrix: G19. Phase: P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::history::init_ctrl_history"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U025；G24、M08；P7。
"""
en = """
Planned local function contract for upstream history.init_ctrl_history (U025). Matrix: G24、M08. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::history::init_sensor_history"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U026；G24、M08；P7。
"""
en = """
Planned local function contract for upstream history.init_sensor_history (U026). Matrix: G24、M08. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::history::read_ctrl"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U027；G24、M08；P7。
"""
en = """
Planned local function contract for upstream history.read_ctrl (U027). Matrix: G24、M08. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::history::read_sensor"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U028；G24、M08；P7。
"""
en = """
Planned local function contract for upstream history.read_sensor (U028). Matrix: G24、M08. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::history::reset_history"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U029；G24、M08；P7。
"""
en = """
Planned local function contract for upstream history.reset_history (U029). Matrix: G24、M08. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::inverse::inverse"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U030；G20；P7。
"""
en = """
Planned local function contract for upstream inverse.inverse (U030). Matrix: G20. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::island::island"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U037；G25；P7/P10。
"""
en = """
Planned local function contract for upstream island.island (U037). Matrix: G25. Phase: P7/P10. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::passive::passive"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U038；G04；P4。
"""
en = """
Planned local function contract for upstream passive.passive (U038). Matrix: G04. Phase: P4. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::query::ray"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U039；G28；P7/P8。
"""
en = """
Planned local function contract for upstream ray.ray (U039). Matrix: G28. Phase: P7/P8. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::query::rays"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U040；G28；P7/P8。
"""
en = """
Planned local function contract for upstream ray.rays (U040). Matrix: G28. Phase: P7/P8. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::sensor::energy_pos"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U046；G23、G26；P7。
"""
en = """
Planned local function contract for upstream sensor.energy_pos (U046). Matrix: G23、G26. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::sensor::energy_vel"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U047；G23、G26；P7。
"""
en = """
Planned local function contract for upstream sensor.energy_vel (U047). Matrix: G23、G26. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::sensor::sensor_acc"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U048；G23、G26；P7。
"""
en = """
Planned local function contract for upstream sensor.sensor_acc (U048). Matrix: G23、G26. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::sensor::sensor_pos"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U049；G23、G26；P7。
"""
en = """
Planned local function contract for upstream sensor.sensor_pos (U049). Matrix: G23、G26. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::sensor::sensor_vel"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U050；G23、G26；P7。
"""
en = """
Planned local function contract for upstream sensor.sensor_vel (U050). Matrix: G23、G26. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::constants::set_const"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U051；G27；P2/P7。
"""
en = """
Planned local function contract for upstream set_const.set_const (U051). Matrix: G27. Phase: P2/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::constants::set_const_0"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U052；G27；P2/P7。
"""
en = """
Planned local function contract for upstream set_const.set_const_0 (U052). Matrix: G27. Phase: P2/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::constants::set_const_fixed"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U053；G27；P2/P7。
"""
en = """
Planned local function contract for upstream set_const.set_const_fixed (U053). Matrix: G27. Phase: P2/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::constants::set_const_spring"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U054；G27；P2/P7。
"""
en = """
Planned local function contract for upstream set_const.set_const_spring (U054). Matrix: G27. Phase: P2/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::constants::set_length_range"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U055；G27；P2/P7。
"""
en = """
Planned local function contract for upstream set_const.set_length_range (U055). Matrix: G27. Phase: P2/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::camlight"

[apis.description]
zh = """
G01入口位于physics根。目标命名空间仍属计划。当前验收另记实际证据。
"""
en = """
G01 implemented via mjwarp_rs::physics::camlight; this target namespace remains planned; current validation is recorded separately and other feature families remain planned.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::com_pos"

[apis.description]
zh = """
G01入口位于physics根。目标命名空间仍属计划。当前验收另记实际证据。
"""
en = """
G01 implemented via mjwarp_rs::physics::com_pos; this target namespace remains planned; current validation is recorded separately and other feature families remain planned.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::com_vel"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U058；G02；P4。
"""
en = """
Planned local function contract for upstream smooth.com_vel (U058). Matrix: G02. Phase: P4. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::crb"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U059；G02；P4。
"""
en = """
Planned local function contract for upstream smooth.crb (U059). Matrix: G02. Phase: P4. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::factor_m"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U060；G03；P4/P6。
"""
en = """
Planned local function contract for upstream smooth.factor_m (U060). Matrix: G03. Phase: P4/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::flex::flex"

[apis.description]
zh = """
G01入口位于physics根。目标命名空间仍属计划。当前验收另记实际证据。
"""
en = """
G01 implemented via mjwarp_rs::physics::flex; this target namespace remains planned; current validation is recorded separately and other feature families remain planned.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::kinematics"

[apis.description]
zh = """
G01入口位于physics根。目标命名空间仍属计划。当前验收另记实际证据。
"""
en = """
G01 implemented via mjwarp_rs::physics::kinematics; this target namespace remains planned; current validation is recorded separately and other feature families remain planned.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::rne"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U063；G02；P4。
"""
en = """
Planned local function contract for upstream smooth.rne (U063). Matrix: G02. Phase: P4. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::rne_postconstraint"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U064；G02、G23；P4/P7。
"""
en = """
Planned local function contract for upstream smooth.rne_postconstraint (U064). Matrix: G02、G23. Phase: P4/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::solve_m"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U065；G03；P4/P6。
"""
en = """
Planned local function contract for upstream smooth.solve_m (U065). Matrix: G03. Phase: P4/P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::subtree_vel"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U066；G02；P4。
"""
en = """
Planned local function contract for upstream smooth.subtree_vel (U066). Matrix: G02. Phase: P4. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::tendon::tendon"

[apis.description]
zh = """
G01入口位于physics根。目标命名空间仍属计划。当前验收另记实际证据。
"""
en = """
G01 implemented via mjwarp_rs::physics::tendon; this target namespace remains planned; current validation is recorded separately and other feature families remain planned.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::actuator::transmission"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U068；G05、G06；P4/P7。
"""
en = """
Planned local function contract for upstream smooth.transmission (U068). Matrix: G05、G06. Phase: P4/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::solver::solve"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U069；G14、G15；P6。
"""
en = """
Planned local function contract for upstream solver.solve (U069). Matrix: G14、G15. Phase: P6. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::query::contact_force"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U070；G26、G28；P7。
"""
en = """
Planned local function contract for upstream support.contact_force (U070). Matrix: G26、G28. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::query::jac"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U072；G28；P4/P7。
"""
en = """
Planned local function contract for upstream support.jac (U072). Matrix: G28. Phase: P4/P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::smooth::mul_m"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U073；G03；P4。
"""
en = """
Planned local function contract for upstream support.mul_m (U073). Matrix: G03. Phase: P4. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::support::xfrc_accumulate"

[apis.description]
zh = """
计划保留等价函数。保留冻结上游语义。 U075；G26；P4。
"""
en = """
Planned local function contract for upstream support.xfrc_accumulate (U075). Matrix: G26. Phase: P4. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::callback::Callback"

[apis.description]
zh = """
计划保留类型契约。保留冻结上游语义。 U079；M10；P7。
"""
en = """
Planned type contract for upstream types.Callback (U079). Matrix: M10. Phase: P7. Not implemented; RPC is only an abstract contract notation.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::HistoryLayout"

[apis.description]
zh = """
基础辅助接口已有源码。 检查历史形状与时间间隔。 不代表产品模块完成。
"""
en = """
Implemented foundation helper. Host history shape and explicit minimum-interval checks only; not GPU history initialization, interpolation or dynamics.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::probe_kinematics"

[apis.description]
zh = """
实现GPU刚体子集。 不替代完整G01阶段。
"""
en = """
Implemented synchronous GPU rigid-body/joint/inertial transforms for a strict shared-model subset. No full U018/U062/G01 equivalence.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::KinematicsOutput"

[apis.description]
zh = """
持有已完成宿主结果。 世界索引检查范围。
"""
en = """
Owned finite checked synchronous GPU readback with per-world views; no CPU physics or resident production Data.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::KinematicsWorld"

[apis.description]
zh = """
借用七项世界结果。 切片只读且布局明确。
"""
en = """
Read-only per-world slices for xpos/xquat/xmat/xipos/ximat/xanchor/xaxis; f32, wxyz and row-major.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::probe_com_position"

[apis.description]
zh = """
实现GPU质心子集。 保留完整U057计划。
"""
en = """
Implemented strict synchronous GPU subtree mass/COM, packed cinert and cdof; no full U057/G01 equivalence.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::ComPositionOutput"

[apis.description]
zh = """
持有四项同步宿主结果。 世界视图检查索引。
"""
en = """
Owned finite checked GPU readback with per-world slices and nbody/nv counts.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::ComPositionWorld"

[apis.description]
zh = """
借用质量、质心与惯量。 自由度采用空间向量。
"""
en = """
Read-only subtree_mass/subtree_com/cinert/cdof view; f32, spatial angular-then-linear order.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::RnePlan"

[apis.description]
zh = """
严格RNE加速度辅助。完整G02仍待验收。
"""
en = """
Implemented strict resident RNE qacc auxiliary. Native false-mode failures and full G02/NP acceptance remain open.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::RneData"

[apis.description]
zh = """
严格RNE加速度辅助。完整G02仍待验收。
"""
en = """
Implemented strict resident RNE qacc auxiliary. Native false-mode failures and full G02/NP acceptance remain open.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::RneSnapshot"

[apis.description]
zh = """
严格RNE加速度辅助。完整G02仍待验收。
"""
en = """
Implemented strict resident RNE qacc auxiliary. Native false-mode failures and full G02/NP acceptance remain open.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::RneWorld"

[apis.description]
zh = """
严格RNE加速度辅助。完整G02仍待验收。
"""
en = """
Implemented strict resident RNE qacc auxiliary. Native false-mode failures and full G02/NP acceptance remain open.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SmoothForcePlan"

[apis.description]
zh = """
U016四力合并辅助，三步f32舍入与休眠正零。独立输入，不计算空间外力或加速度；完整G03/NP未验收。
"""
en = """
Standalone U016 four-force f32 merge with step rounding and sleeping +0; no spatial-force accumulation, acceleration or complete G03/NP acceptance.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SmoothForceData"

[apis.description]
zh = """
U016四力合并辅助，三步f32舍入与休眠正零。独立输入，不计算空间外力或加速度；完整G03/NP未验收。
"""
en = """
Standalone U016 four-force f32 merge with step rounding and sleeping +0; no spatial-force accumulation, acceleration or complete G03/NP acceptance.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SmoothForceOutput"

[apis.description]
zh = """
U016四力合并辅助，三步f32舍入与休眠正零。独立输入，不计算空间外力或加速度；完整G03/NP未验收。
"""
en = """
Standalone U016 four-force f32 merge with step rounding and sleeping +0; no spatial-force accumulation, acceleration or complete G03/NP acceptance.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SmoothForceOptions"

[apis.description]
zh = """
U016四力合并辅助，三步f32舍入与休眠正零。独立输入，不计算空间外力或加速度；完整G03/NP未验收。
"""
en = """
Standalone U016 four-force f32 merge with step rounding and sleeping +0; no spatial-force accumulation, acceleration or complete G03/NP acceptance.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::SmoothForceField"

[apis.description]
zh = """
U016四力合并辅助，三步f32舍入与休眠正零。独立输入，不计算空间外力或加速度；完整G03/NP未验收。
"""
en = """
Standalone U016 four-force f32 merge with step rounding and sleeping +0; no spatial-force accumulation, acceleration or complete G03/NP acceptance.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::AppliedForcePlan"

[apis.description]
zh = """
U075独立空间外力投影，f32子树累加与事务add/replace。调用方提供几何，不执行生产者；完整U075/NP未验收。
"""
en = """
Standalone U075 f32 subtree spatial-force projection with transactional add/replace; caller-supplied geometry, no producers or full U075/NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::AppliedForceData"

[apis.description]
zh = """
U075独立空间外力投影，f32子树累加与事务add/replace。调用方提供几何，不执行生产者；完整U075/NP未验收。
"""
en = """
Standalone U075 f32 subtree spatial-force projection with transactional add/replace; caller-supplied geometry, no producers or full U075/NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::AppliedForceOutput"

[apis.description]
zh = """
U075独立空间外力投影，f32子树累加与事务add/replace。调用方提供几何，不执行生产者；完整U075/NP未验收。
"""
en = """
Standalone U075 f32 subtree spatial-force projection with transactional add/replace; caller-supplied geometry, no producers or full U075/NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::AppliedForceGeometry"

[apis.description]
zh = """
U075独立空间外力投影，f32子树累加与事务add/replace。调用方提供几何，不执行生产者；完整U075/NP未验收。
"""
en = """
Standalone U075 f32 subtree spatial-force projection with transactional add/replace; caller-supplied geometry, no producers or full U075/NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::KineticEnergyPlan"

[apis.description]
zh = """
默认32线程动能辅助。 调用方提供同状态配对字段。 完整上游与NP仍待验收。
"""
en = """
Strict default32 f32 kinetic consumer; caller assures paired model/state provenance, no complete upstream or NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::KineticEnergyData"

[apis.description]
zh = """
默认32线程动能辅助。 调用方提供同状态配对字段。 完整上游与NP仍待验收。
"""
en = """
Strict default32 f32 kinetic consumer; caller assures paired model/state provenance, no complete upstream or NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::KineticEnergyOutput"

[apis.description]
zh = """
默认32线程动能辅助。 调用方提供同状态配对字段。 完整上游与NP仍待验收。
"""
en = """
Strict default32 f32 kinetic consumer; caller assures paired model/state provenance, no complete upstream or NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::KineticEnergyVector"

[apis.description]
zh = """
默认32线程动能辅助。 调用方提供同状态配对字段。 完整上游与NP仍待验收。
"""
en = """
Strict default32 f32 kinetic consumer; caller assures paired model/state provenance, no complete upstream or NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::JointPassivePlan"

[apis.description]
zh = """
GPU关节阻尼与四类弹簧子集。五参数独立周期。完整被动力仍缺。
"""
en = """
GPU joint damping and all four joint spring subset with five independent parameter periods; not complete passive or NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::JointPassiveData"

[apis.description]
zh = """
GPU关节阻尼与四类弹簧子集。五参数独立周期。完整被动力仍缺。
"""
en = """
GPU joint damping and all four joint spring subset with five independent parameter periods; not complete passive or NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::JointPassiveOutput"

[apis.description]
zh = """
GPU关节阻尼与四类弹簧子集。五参数独立周期。完整被动力仍缺。
"""
en = """
GPU joint damping and all four joint spring subset with five independent parameter periods; not complete passive or NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::JointPassiveOptions"

[apis.description]
zh = """
GPU关节阻尼与四类弹簧子集。五参数独立周期。完整被动力仍缺。
"""
en = """
GPU joint damping and all four joint spring subset with five independent parameter periods; not complete passive or NP admission.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::physics::JointPassiveParameters"

[apis.description]
zh = """
GPU关节阻尼与四类弹簧子集。五参数独立周期。完整被动力仍缺。
"""
en = """
GPU joint damping and all four joint spring subset with five independent parameter periods; not complete passive or NP admission.
"""

[[deps]]
kind = "reference"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::physics::forward::forward"
to_api = "rpc:mjwarp_rs::model::data::Data"

[deps.label]
zh = "访问物理状态与工作区"
en = "State and workspaces"

[[deps]]
kind = "call"
to = "mjwarp.runtime.completion"
from_api = "rpc:mjwarp_rs::physics::forward::step"
to_api = "rpc:mjwarp_rs::runtime::submit"

[deps.label]
zh = "提交物理执行计划"
en = "Physics submission"

[[deps]]
kind = "reference"
to = "mjwarp.math"
from_api = "rpc:mjwarp_rs::physics::forward::forward"
to_api = "rpc:mjwarp_rs::math::rot_vec_quat"

[deps.label]
zh = "使用空间代数约定"
en = "Spatial algebra"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::physics::forward::forward"
to_api = "rpc:mjwarp_rs::diagnostics::PhysicsReport"

[deps.label]
zh = "返回物理有效性报告"
en = "Physics validity"

[[deps]]
kind = "call"
to = "mjwarp.model.core"
from_api = "rpc:mjwarp_rs::physics::HistoryLayout"
to_api = "rpc:mjwarp_rs::model::BatchLayout"

[deps.label]
zh = "基础辅助的真实依赖"
en = "Foundation helper"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::physics::HistoryLayout"
to_api = "rpc:mjwarp_rs::diagnostics::InputError"

[deps.label]
zh = "基础辅助的真实依赖"
en = "Foundation helper"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::probe_kinematics"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[deps.label]
zh = "提交同步内核"
en = "Synchronous kernel launch"

[[deps]]
kind = "reference"
to = "mjwarp.model.inertial"
from_api = "rpc:mjwarp_rs::physics::probe_kinematics"
to_api = "rpc:mjwarp_rs::model::InertialModelInput"

[deps.label]
zh = "读取刚体模型子集"
en = "Validated rigid model"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::physics::probe_kinematics"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[deps.label]
zh = "返回输入与设备错误"
en = "Input and device errors"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
from_api = "rpc:mjwarp_rs::physics::probe_kinematics"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[deps.label]
zh = "复制模型与状态"
en = "Model and state transfer"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::probe_com_position"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[deps.label]
zh = "提交同步内核"
en = "Synchronous kernel launch"

[[deps]]
kind = "reference"
to = "mjwarp.model.inertial"
from_api = "rpc:mjwarp_rs::physics::probe_com_position"
to_api = "rpc:mjwarp_rs::model::InertialModelInput"

[deps.label]
zh = "读取刚体模型子集"
en = "Validated rigid model"

[[deps]]
kind = "reference"
to = "mjwarp.diagnostics"
from_api = "rpc:mjwarp_rs::physics::probe_com_position"
to_api = "rpc:mjwarp_rs::diagnostics::TransferError"

[deps.label]
zh = "返回输入与设备错误"
en = "Input and device errors"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.core"
from_api = "rpc:mjwarp_rs::physics::probe_com_position"
to_api = "rpc:mjwarp_rs::runtime::TransferSession"

[deps.label]
zh = "复制模型与状态"
en = "Model and state transfer"

[[deps]]
kind = "reference"
to = "mjwarp.model.attached"
from_api = "rpc:mjwarp_rs::physics::RnePlan"
to_api = "rpc:mjwarp_rs::model::AttachedModelInput"

[[deps]]
kind = "reference"
to = "mjwarp.model.sleep"
from_api = "rpc:mjwarp_rs::physics::SmoothForcePlan"
to_api = "rpc:mjwarp_rs::model::SmoothForceInput"

[deps.label]
zh = "读取合并索引子集"
en = "Merge index subset"

[[deps]]
kind = "call"
to = "mjwarp.runtime.transfer.kernel"
from_api = "rpc:mjwarp_rs::physics::SmoothForcePlan"
to_api = "rpc:mjwarp_rs::runtime::transfer::kernel::SynchronousKernel"

[deps.label]
zh = "提交独立合并内核"
en = "Bounded merge kernel"
+++

## 2026-10-14 RNE加速度严格辅助

新增RnePlan/Data/Snapshot/World。
辅助复用既有空间速度底座。
qacc支持独立全量与世界写入。
合法写入废弃RNE结果。
qacc不废弃空间速度前置。
输入拒绝保留结果。
六分量按深度采用f32原子加。
加速度惯量调用修复首二项融合。
两个状态双flag共332项零位差。
诊断共享产品前置，不代表全链。
新集成最终两过一败。
开启加速度450144标量原生通过。
关闭加速度仍1042块原生超差。
原容差与全部覆盖保持。
不借用G01/G03红线。
新集成与阶段诊断各累计三次。
两类现已冻结，不改名重放。
旧偏置接口与源码保持。
旧精度清单不继承到新单元。
完整G02及NP01–04仍未验收。
本核心保持计划态及原UID。
证据：C:/Rust/mjwarp-rs/docs/rne-acceleration.md。

## 零四元数产物修复

完整G01的零分支返回末位一。
该分支对齐冻结Warp实际PTX。
四元数仍按wxyz顺序存储。
严格入口及正长度计算不变。
四世界七输出608项对照一致。
完整G01集成本轮未重跑。
完整精度与性能仍未验收。
详细证据见docs/t4-zero-quaternion.md。

## 本轮源码复审

已移除内部f64标量模板。
刚体、质心设备链固定f32。
宿主暂存也保持f32。
质量求解复用同型设备矩阵。
模块另导出空间速度辅助。
模块也导出RNE偏置力辅助。
完整G01保留现有公开路径。
旧目标命名空间仍属计划。
G02与其余完整阶段仍待实现。
历史双平台证据另见报告。
当前T4以本轮补测为准。
本轮不激活完整物理核心。

## 2026-10-14 常驻混合桥接增量

导出MixedResident Plan/Data，qpos→CRB/M→CSR→factor/solve。
原f32计算、ABI与编译门禁保持。
新host2过，GPU1过1败，private1过，组合3/3冻结。
数值12批原容差通过，状态测试修后未执行。
不激活完整G03/G02或正式compiler。
原UID/计划态/正式前置资源锁保持。
旧柔体面覆盖与容差不改，不重放满额目标。
证据：C:/Rust/mjwarp-rs/docs/resident-mixed-mass.md。

## 2026-10-14 U016四力合并辅助

SmoothForce Plan/Data/Output/Options/Field已落地，core仍planned。
source字段顺序field/world/dof，三步f32 RN不重排/提升。
SLEEP&&!ISLAND控制tree>=0&&awake==0写+0，静态/非零awake活动。
原256 ABI/compile/gate不变，整数映射在解引用前验证。
新host4过0.00秒，GPU2过2.73秒，combined2/3成功冻结。
18固定bit证书/72批1204320槽零位差；只算子诊断非完整NP。
休眠正零54481、活动负零63732、静态活动114400。
世界256四字段/awake原位型、非法写/foreign保留、快照及private fault/overflow/恢复通过。
93源预绑定不变，四回执两PTX实际source/hash吻合。
PTX各sub.rn1/add.rn2/FMA0/floatf640，同名相同源不冒充逐次JIT。
2463ms含JIT/诊断/传输，四Data容量2773732bytes非峰值/公平性能。
不实现xfrc_accumulate/qacc_smooth/DISCRETE/sleep solve或完整fwd_acceleration。
原UID/planned与正式前置资源保持，主in_progress不关闭。
旧mul_m与bridge/Small/SDK/Clippy/所有冻目标零重跑且缺口保留。
首patch英文label超30未写入，修为短label；原无关记录不覆盖。
证据C:/Rust/mjwarp-rs/docs/smooth-force.md。

## 2026-10-14 U075/G26空间外力累加

AppliedForce Plan/Data/Output/Geometry已落地于原core，仍planned。
显式parent/root/dofbody索引，parent先于child排循环，root父链/范围一致。
Ft force/torque与cdof angular/linear，jac反排，rootCOM偏移。
升序body/真实子树/±0 FT跳过/world0排除，无atomic。
Dot6正零顺序，dot3三项左结合，cross保留原C++ f32融合许可。
Delta/current/staged全f32，final RNadd或copy后全批成功swap。
重复add真累加，replace不读取旧NaNpayload；fault不部分改变currentbits。
原256 ABI/NVRTC12.8 compute70/fmadtrue/fastfalse/gate不改。
所有几何/体/DOF/root内层偏移64位，checked尺寸/metai32/worku32。
Whole geometry/ft strict finite（包括unused）不冒充上游全兼容门面。
新host2过0.00秒/201过滤，GPU2过5.12秒/280过滤无warning。
72批delta与qfrc各1204320槽严格零位差，合计2408640非单字段。
固定四body独立位型+f64虚功诊断，无Native/旧author/Python/candidate反馈。
FMA隔离预冻结absrel2e-7/两位型候选，仅未定上游编译行为诊断；实际+0，f64误差1.421085e-14。
完整冻结_apply_ft generatedentry缺失，不能用别的dot/cross模板放行NP。
新purpose combined2/3成功冻结，不补第三次，全部旧目标/producer/Clippy/SDK/作者0调用。
96源预绑定无漂移，3实际entry/1PTX同源同hash，非逐JIT记录。
PTX FMA8/mul7/sub6/add3，无floatf64/atomic；原源edb51d90/PTXd4224772。
4747ms含JIT/host诊断/传输，6Data float+1Plan int容量8119492bytes非peak/fair。
只扩physicscore，不改model/runtime或immutable runningclaims，UID/core/rootplanned/主in_progress保持。
完整U075/G03/U016/qacc/DISCRETE/NP和旧deviceBorrow/原源覆盖/失败生命周期仍缺。
证据C:/Rust/mjwarp-rs/docs/applied-force.md。


## 2026-10-14 默认32动能消费者

U047/G23/G26新增严格f32辅助，P4能量原型不冒充P7完整sensor调度。
qvel/Mqvel声明同model/state，一次成对上传，world256保留未选位型。
外部标签不认证Mqvel=M(state)qvel；完整CRB→M→Mqvel设备接线仍缺。
Warp1.15默认32，register lane+reg*32，首项初始化，无效尾槽不加；shuffle16/8/4/2/1。
NV1..1024受限支持，GPU数值只测到513，1024只有宿主边界。
势能位型复制，状态只标动能；严格有限势能边界不同于完整上游入口。
源/保护槽/溢出不部分发布，完整暂存有限后交换，快照独占标签与结果。
新host2过0.00秒/203过滤，CUDA新consumer/private/ABI3过1.81秒/284过滤，无编译警告。
新combined2/3成功冻结，第三次不调用，旧producer/测试/作者/SDK/Clippy0调用。
48批4160世界；kinetic与potential各4160槽零位差，手写证书另12世界。
独立正对角质量与dyadic速度f64诊断；nv31/33树序列差、负零1/7/65与minsubnormal513。
新32ABI核对32lane与257world块、四会话和21拒绝；旧内核方法只加mod声明。
17实际entry/9唯一CU/PTX哈希一致，非逐JIT；nv33/65各FMA1，其余产品nv0，各shuffle5/ballot1，无f64/atomic。
完整冻结energy_vel generatedentry仍缺，不借其他模板放行NP。
458预规范化身份零漂移；其中20旧MJB的UTF8模式不是raw身份，保留原清单。
另做测试后20MJB原始基线Gitblob/磁盘核对全同，不冒充预跑快照；consumer不读MJB。
449旧Git过滤blob不变，另2文件只加module/reexport，旧容差/coverage/Cargo/gates保持。
1414ms含JIT/传输/host诊断，自有四float+一int容量1060000bytes，非peak/fair。
完整U047、NP01..NP04、Linux/release/T4、生产者借用与旧失败用途仍开放/冻结。
证据C:/Rust/mjwarp-rs/docs/kinetic-energy.md。


## 2026-10-14 常驻CRB/M直接借用消费者

新增MassMatrixMultiplyPlan/Data/View。计划借用实际MassMatrixPlan；只读context读取实际i32 parents检查CSR匹配，旧函数/CUDA不变。
每次update消费同原计划ready MassMatrixData原buffer，明确guard4+world*(10nb+nv²)+guard4，M无副本/复制/host反馈。
Sparse保留原追加顺序与下三角镜像地址，Dense静态nv列序；f32正零累加、融合许可与原门禁保持。
结果载荷D2D复制current到staged保留skip；全暂存有限/保护槽通过才交换，不部分发布。
View同时共享借用源与consumerData，Rust禁止视图期内qpos/M/vector写入；无外部整数标签或缓存M。
逐世界is_current仅本次计算为true，skip历史输出为false；fresh_snapshot拒绝任何skip，普通snapshot只作历史结果。
初次全skip、qpos新态全skip、vector改写部分skip与同plan另一source实例切换+skip已覆盖。
冷源/异计划/consumer跨token/非法向量与skip通过；错worlds和坏dense地址拒绝只有静态证据，不冒充GPU覆盖。
新host2过0.00秒/205过滤、GPU2过4.89秒/289过滤，combined2/3成功冻结不补第三次。
主72批210080槽，条件化于实际GPU f32 M的独立f64点积maxabs4.05125319957733154e-7，原abs/rel2e-5。
该参考只证明消费关系，不证明CRB/原生/冻结上游精度；旧CRB/mul_m/bridge/factor/SDK/作者/Clippy验收目的0。
实际旧生产者作为新连接依赖主19/private1共20次update，非20核启动，不声称producer0calls。
私有双格式102故障恢复，无部分发布；source guard/CRB/M/vector非有限与溢出、metadata静态范围拒绝。
主4065ms含JIT/host/传输/依赖；consumer5buffer219852bytes，不含producer，不是peak/fair。
19真实entry=12新consumer+7依赖，10唯一PTX；新Sparse各FMA5/Dense3/33/65，无f64/atomic。
225预身份原样封存，224同模式不变；新test名称大写M编译warning1，静态小写m改名正文相同，最终名版本未编译/执行，不重置冻结purpose。
补属性前217旧blob不变，4旧文件只新增声明/exports/只读context；属性文本另登记，产品源码没有后测修补。
原moduleUID/state/running claims/dependencies/resources保持，完整U073/G03/energy设备接线/NP、旧Sparse原源与Small/bridge失败继续开放冻结。
证据C:/Rust/mjwarp-rs/docs/borrowed-mass-multiply.md。


### 提交前完整祖先行修复

parents相等不能认证Compact/子集CSR来源。严格新packed新增完整祖先行拒绝，避免漏掉真实CRB耦合。
仅修改新消费者Rust构造与既有新host拒绝断言；六新CUDA源与全部旧body/tests/容差不变。
原host/GPU成功冻结2/3不再执行，原执行源与日志保留；final mainGPU body只改大小写测试名，另新host断言未运行。
实际cargo check --locked --features cuda-probe --lib成功2.46秒，只编译最终库，最终test版本仍未编译/执行。
该拒绝分支仅静态与库类型检查证据，不宣称动态已验收。旧预stage map c83e3e1属历史，最终map/HEAD另核。


## 2026-10-14 严格当前态设备动能接线

MassMatrixEnergyPlan/Data/View/Snapshot接通实际CRB→M→Mqvel→GPU pair→原scalar32。
能量计划借用实际乘积计划，双Arc身份/原matrix计划/ready/worlds/allcurrent全部检查，无外部状态标签。
能量View继续借用原乘积View与能量Data，合法qpos/vector/Mqvel可变写入在视图期内不成立；snapshot只作历史，势能仅显式透传不认证状态。
每world两次既有memcpy_dtod成对复制q/Mq，非零复制pair；M仍零复制，无正常物理Mqvel下载回传。
新原始source有限性/保护槽GPU验证，原CUDA字符串与32ABI逐字复用；完全暂存通过才swap，无部分发布。
新host2过0.00秒/207过滤（编译3.69），GPU2过4.58秒/293过滤（编译7.69），无warning，combined2/3成功freeze不补第三次。
72批6240 kinetic/potential世界各自零位差，420160配对槽精确，额外nv3双格式负零4世界，private56故障通过。
一热hot=(world+vector+pose)%nv逐world区别两向量；独立源Mjj正正规数指数减1作精确预期，不读候选Mqvel/energy。
不移植mul_m 2e-5作为能量容差；原U047固定输入与完整标准不变，一热只补接线；一般混合向量与M/原生/上游精度仍缺。
一个新GPU命令两新函数内source18+private1=19；Mqvel72+allskip2+partial2+negativezero2+private1=79，非独立test或核启动。
旧test selectors/Clippy/SDK/作者/cargo check/full0；共享旧tests仅编译不执行，原祖先CSR新拒绝断言不追认。
3762ms含JIT/transfer/host诊断/依赖，不含初次session；能量自有四float+一int138912bytes，不含M/Mqvel producer，非peak/fair。
28实际entry/16唯一PTX，source validator3+原scalar6+其他依赖19；nv33/65四份scalar源与PTX匹配旧U047原记录，完整冻结upstream生成entry仍缺。
228预绑定零漂移；补属性前217旧blob不变，5文件仅exports/privateconst/helpervisibility。执行源与当前源完全相同。
归档脚本首次counter matches与PS自动Matches冲突，只修静态计数重归档，无物理调用。
旧UID/state/main claims/deps/resources不变；完整energy_vel/sensor/G03/NP/一般向量、祖先拒绝动态验证、旧Sparse原源/失败及Linux/release/T4/fair性能仍开放。
证据C:/Rust/mjwarp-rs/docs/device-energy.md。


## 2026-10-14 关节被动力子集

GPU逐DOF阻尼覆盖FREE/BALL/SLIDE/HINGE；标量多项式弹簧仅SLIDE/HINGE。五参数独立周期、全物理f32、原逐项累加与融合许可。实际KinematicModelInput生成拓扑，Arc绑定Data；guard/meta/finite验证后暂存交换，无部分关节发布。关节和直接设备写原smooth Passive字段，joint_only命名且sleep关闭；三外来力显式输入，无物理宿主反馈。启用FREE/BALL非零旋转弹簧显式拒绝，完整被动力/生成上游算子/NP仍缺。

实际3/3冻结：host1 E0432零用例；host2两过0.00秒；GPU3两过2.47秒，无warning。48批三输出各45760零位差，51私有故障；48旧smooth依赖非独立测试。1888ms与130604bytes不是peak/fair。最终执行源51b1c516...不变。原UID/core planned/main in_progress/claims/deps/resources与旧容差覆盖保持。旧满额/成功/失败/Clippy/SDK/作者/check均未重放。完整U038/G04/NP与所有旧缺口保持。证据C:/Rust/mjwarp-rs/docs/joint-passive.md。
