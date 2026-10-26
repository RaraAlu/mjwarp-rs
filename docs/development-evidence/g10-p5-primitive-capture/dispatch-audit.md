# B0 dispatch 审计（纯 CPU，2026-10-26）

任务：`2026-10-26-g10-p5-primitive-capture`；pinned frozen 源：
`docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/`。
审计对象：`collision_driver.py::MJ_COLLISION_TABLE`（L47-81）、`_narrowphase` 的 NATIVECCD 覆盖
（L908-915）、`collision_primitive.py::_PRIMITIVE_COLLISIONS`（L1335-1349）与 `primitive_narrowphase`
分派（L1521-1548）。**只读 pinned python 源，未运行 GPU/未改任何文件。**

## 1. 原始 dump（逐行，行号即 frozen 文件行号）

`collision_driver.py`：

```
47: MJ_COLLISION_TABLE = {
48:   (GeomType.PLANE, GeomType.SPHERE): CollisionType.PRIMITIVE,
49:   (GeomType.PLANE, GeomType.CAPSULE): CollisionType.PRIMITIVE,
50:   (GeomType.PLANE, GeomType.ELLIPSOID): CollisionType.PRIMITIVE,
51:   (GeomType.PLANE, GeomType.CYLINDER): CollisionType.PRIMITIVE,
52:   (GeomType.PLANE, GeomType.BOX): CollisionType.PRIMITIVE,
53:   (GeomType.PLANE, GeomType.MESH): CollisionType.PRIMITIVE,
60:   (GeomType.SPHERE, GeomType.SPHERE): CollisionType.PRIMITIVE,
61:   (GeomType.SPHERE, GeomType.CAPSULE): CollisionType.PRIMITIVE,
63:   (GeomType.SPHERE, GeomType.CYLINDER): CollisionType.PRIMITIVE,
66:   (GeomType.CAPSULE, GeomType.CAPSULE): CollisionType.PRIMITIVE,
78:   (GeomType.BOX, GeomType.BOX): CollisionType.CONVEX,  # overwritten by NATIVECCD disable flag
908: def _narrowphase(m: Model, d: Data, ctx: CollisionContext):
909:   collision_table = MJ_COLLISION_TABLE
910:   if m.opt.disableflags & DisableBit.NATIVECCD:
911:     collision_table = collision_table.copy()
912:     collision_table[(GeomType.BOX, GeomType.BOX)] = CollisionType.PRIMITIVE
913:
914:   convex_pairs = [key for key, value in collision_table.items() if value == CollisionType.CONVEX]
915:   primitive_pairs = [key for key, value in collision_table.items() if value == CollisionType.PRIMITIVE]
```

`collision_primitive.py`：

```
1334: # Map of supported primitive collision functions
1335: _PRIMITIVE_COLLISIONS = {
1336:   (GeomType.PLANE, GeomType.SPHERE): plane_sphere_wrapper,
1337:   (GeomType.PLANE, GeomType.CAPSULE): plane_capsule_wrapper,
1338:   (GeomType.PLANE, GeomType.ELLIPSOID): plane_ellipsoid_wrapper,
1339:   (GeomType.PLANE, GeomType.CYLINDER): plane_cylinder_wrapper,
1340:   (GeomType.PLANE, GeomType.BOX): plane_box_wrapper,
1341:   (GeomType.PLANE, GeomType.MESH): plane_convex_wrapper,
1342:   (GeomType.SPHERE, GeomType.SPHERE): sphere_sphere_wrapper,
1343:   (GeomType.SPHERE, GeomType.CAPSULE): sphere_capsule_wrapper,
1344:   (GeomType.SPHERE, GeomType.CYLINDER): sphere_cylinder_wrapper,
1345:   (GeomType.SPHERE, GeomType.BOX): sphere_box_wrapper,
1346:   (GeomType.CAPSULE, GeomType.CAPSULE): capsule_capsule_wrapper,
1347:   (GeomType.CAPSULE, GeomType.BOX): capsule_box_wrapper,
1348:   (GeomType.BOX, GeomType.BOX): box_box_wrapper,
1349: }
1539:   for types, func in _PRIMITIVE_COLLISIONS.items():
1543:     if m.geom_pair_type_count[idx] and types not in _PRIMITIVE_COLLISION_TYPES:
```

## 2. 逐组合对照（`nativeccd="disable"` = `DisableBit.NATIVECCD` 置位）

| # | 组合 | table 行/覆盖规则 | dispatch 结果 | wrapper 函数名 |
| --- | --- | --- | --- | --- |
| 1 | PLANE-SPHERE | L48 `PRIMITIVE` | PRIMITIVE | `plane_sphere_wrapper` |
| 2 | PLANE-CAPSULE | L49 `PRIMITIVE` | PRIMITIVE | `plane_capsule_wrapper` |
| 3 | PLANE-ELLIPSOID | L50 `PRIMITIVE` | PRIMITIVE | `plane_ellipsoid_wrapper` |
| 4 | PLANE-CYLINDER | L51 `PRIMITIVE` | PRIMITIVE | `plane_cylinder_wrapper` |
| 5 | PLANE-BOX | L52 `PRIMITIVE` | PRIMITIVE | `plane_box_wrapper` |
| 6 | PLANE-MESH | L53 `PRIMITIVE` | PRIMITIVE | `plane_convex_wrapper` |
| 7 | SPHERE-SPHERE | L60 `PRIMITIVE` | PRIMITIVE | `sphere_sphere_wrapper` |
| 8 | SPHERE-CAPSULE | L61 `PRIMITIVE` | PRIMITIVE | `sphere_capsule_wrapper` |
| 9 | SPHERE-CYLINDER | L63 `PRIMITIVE` | PRIMITIVE | `sphere_cylinder_wrapper` |
| 10 | CAPSULE-CAPSULE | L66 `PRIMITIVE` | PRIMITIVE | `capsule_capsule_wrapper` |
| (参) | BOX-BOX | L78 `CONVEX` → L910-912 覆盖为 `PRIMITIVE` | PRIMITIVE | `box_box_wrapper`（既有参考，不重捕） |

**结论：10/10 组合在 `nativeccd=disable` 下都路由到 `CollisionType.PRIMITIVE` 且各有明确 wrapper；
无需改动任何既有 verified fixture 的路由条件 → 审计通过，不触发停手条件。**

## 3. 附注（影响 B0/B1 边界）

- `plane-mesh` 走 `plane_convex_wrapper`（L838-874），内部用 `convex.vert[convex.vertadr + idx]`
  （L159/176/196/218/242/269）即 mesh 顶点 + `geom_dataid/vertadr`；**B0 捕获侧不需要新 mesh polygon ABI**。
  若 B1 设备实现发现需要 mesh polygon 额外数据 → 按红线停手报 Arch、记未覆盖。
- 本例只审计 pinned frozen 源；`installed-source/_src/` 同 revision，未使用。
