+++
uid = "1034f0ee"
id = "mjwarp.physics.collision"
tags = ["implemented-subset", "gpu", "collision", "windows"]
fingerprint = "924b6df12a2cd9e2704ccd3233c05287596ebb0f760d9bb899aa31fe0a81e8d7"

[name]
zh = "碰撞检测与窄相"
en = "Collision detection and narrow phase"

[description]
zh = """
碰撞检测与窄相。宿主侧宽相（nxn/SAP）、窄相分派、接触过滤与相对矩阵；设备侧 GJK/EPA 与多接触、SDF 窄相与八叉树/下降、hfield、primitive plane 组合，以及 G5 网格多边形通道。语义对齐冻结上游，代表实现子集，不等于完整碰撞验收。
"""
en = """
Collision detection and narrow phase. Host broad phase (nxn/SAP), narrow-phase dispatch, contact filtering and relative matrices; device GJK/EPA with multi-contact, SDF narrow phase with octree/descent, hfield, primitive-plane combinations and the G5 mesh polygon channel. Semantics follow the frozen upstream as an implemented subset; not full collision acceptance.
"""

[[source]]
path = "src/physics/collision.rs"

[[source]]
path = "src/physics/collision/cuda/boxbox.cu"

[[source]]
path = "src/physics/collision/cuda/boxbox_wrapper.cu"

[[source]]
path = "src/physics/collision/cuda/ccd.cu"

[[source]]
path = "src/physics/collision/cuda/common.cu"

[[source]]
path = "src/physics/collision/cuda/contact.cu"

[[source]]
path = "src/physics/collision/cuda/epa.cu"

[[source]]
path = "src/physics/collision/cuda/geom.cu"

[[source]]
path = "src/physics/collision/cuda/gjk_core.cu"

[[source]]
path = "src/physics/collision/cuda/hfield.cu"

[[source]]
path = "src/physics/collision/cuda/mod.rs"

[[source]]
path = "src/physics/collision/cuda/multicontact.cu"

[[source]]
path = "src/physics/collision/cuda/multicontact_clip.cu"

[[source]]
path = "src/physics/collision/cuda/multicontact_geom.cu"

[[source]]
path = "src/physics/collision/cuda/multicontact_mesh.cu"

[[source]]
path = "src/physics/collision/cuda/polytope.cu"

[[source]]
path = "src/physics/collision/cuda/polytope_build.cu"

[[source]]
path = "src/physics/collision/cuda/primitive.cu"

[[source]]
path = "src/physics/collision/cuda/primitive_plane_common.cu"

[[source]]
path = "src/physics/collision/cuda/primitive_plane_convex.cu"

[[source]]
path = "src/physics/collision/cuda/primitive_plane_shapes.cu"

[[source]]
path = "src/physics/collision/cuda/primitive_plane_wrapper.cu"

[[source]]
path = "src/physics/collision/cuda/primitive_sphere_capsule_shapes.cu"

[[source]]
path = "src/physics/collision/cuda/primitive_sphere_capsule_wrapper.cu"

[[source]]
path = "src/physics/collision/cuda/primitive_wrapper.cu"

[[source]]
path = "src/physics/collision/cuda/product.cu"

[[source]]
path = "src/physics/collision/cuda/sdf_math.cu"

[[source]]
path = "src/physics/collision/cuda/sdf_narrowphase.cu"

[[source]]
path = "src/physics/collision/cuda/sdf_octree.cu"

[[source]]
path = "src/physics/collision/cuda/simplex.cu"

[[source]]
path = "src/physics/collision/cuda/support.cu"

[[source]]
path = "src/physics/collision/filter.rs"

[[source]]
path = "src/physics/collision/gjk/case_compare.rs"

[[source]]
path = "src/physics/collision/gjk/case_data.rs"

[[source]]
path = "src/physics/collision/gjk/case_evidence.rs"

[[source]]
path = "src/physics/collision/gjk/ccd.rs"

[[source]]
path = "src/physics/collision/gjk/contact.rs"

[[source]]
path = "src/physics/collision/gjk/core.rs"

[[source]]
path = "src/physics/collision/gjk/epa.rs"

[[source]]
path = "src/physics/collision/gjk/math.rs"

[[source]]
path = "src/physics/collision/gjk/mod.rs"

[[source]]
path = "src/physics/collision/gjk/multicontact/clip.rs"

[[source]]
path = "src/physics/collision/gjk/multicontact/feature.rs"

[[source]]
path = "src/physics/collision/gjk/multicontact/mod.rs"

[[source]]
path = "src/physics/collision/gjk/multicontact/normals.rs"

[[source]]
path = "src/physics/collision/gjk/phase.rs"

[[source]]
path = "src/physics/collision/gjk/polytope.rs"

[[source]]
path = "src/physics/collision/gjk/polytope_build.rs"

[[source]]
path = "src/physics/collision/gjk/simplex.rs"

[[source]]
path = "src/physics/collision/gjk/support.rs"

[[source]]
path = "src/physics/collision/gjk/tests.rs"

[[source]]
path = "src/physics/collision/gjk/window_debug.rs"

[[source]]
path = "src/physics/collision/hfield/driver.rs"

[[source]]
path = "src/physics/collision/hfield/filter.rs"

[[source]]
path = "src/physics/collision/hfield/mod.rs"

[[source]]
path = "src/physics/collision/hfield/tests.rs"

[[source]]
path = "src/physics/collision/narrowphase.rs"

[[source]]
path = "src/physics/collision/narrowphase/primitive.rs"

[[source]]
path = "src/physics/collision/narrowphase/tests.rs"

[[source]]
path = "src/physics/collision/narrowphase/tests/plane_box.rs"

[[source]]
path = "src/physics/collision/narrowphase/validate.rs"

[[source]]
path = "src/physics/collision/nxn.rs"

[[source]]
path = "src/physics/collision/perf_report.rs"

[[source]]
path = "src/physics/collision/relmat.rs"

[[source]]
path = "src/physics/collision/sap.rs"

[[source]]
path = "src/physics/collision/sdf/descent.rs"

[[source]]
path = "src/physics/collision/sdf/mod.rs"

[[source]]
path = "src/physics/collision/sdf/narrowphase.rs"

[[source]]
path = "src/physics/collision/sdf/octree.rs"

[[source]]
path = "src/physics/collision/sdf/primitives.rs"

[[source]]
path = "src/physics/collision/sdf/tests.rs"

[[source]]
path = "src/physics/collision/tests.rs"

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
+++
