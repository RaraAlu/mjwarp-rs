# M1 dispatch 审计：mesh-box / mesh-cylinder 走哪条窄相路径

pinned 上游 revision `71da24d956378a87a703b6e1442b13aec0c4ac29`
（`docs/evidence/d04-complex-physics/authoring-runs/run-5/installed-source/_src/`）。
目标 flag 集：`nativeccd="disable"`（fixture 固定）、**MULTICCD 未禁用**（默认启用）。

## 结论

**mesh-box 与 mesh-cylinder 都走 CONVEX 的 convex CCD multicontact 分支（每对最多 4 个接触），
不走 primitive、也不走单接触回退。** 成立条件（fixture 已满足，CPU 预检实测）：

1. pair 在 `MJ_COLLISION_TABLE` 中为 `CollisionType.CONVEX`；
2. `use_multiccd = (disableflags & MULTICCD) == 0`（默认成立；fixture 只禁 NATIVECCD）；
3. 两侧 geom 类型 ∈ {BOX, MESH, CYLINDER} 且 **margin == 0**（fixture 未设 margin）；
4. mesh 侧 `mesh_polyadr >= 0` 且有 polygon 数据（CPU 预检：`polynum=6, polyadr=0,
   polyvertnum=4, polyvertadr=0, polymapnum=3, polymapadr=0`）。

mesh-mesh 同为 CONVEX、同一 multicontact 分支（`(MESH,MESH): CONVEX`），M1 只做预留结论、不给 case。

## 证据链（逐条行号）

1. **pair → CONVEX**
   - `collision_driver.py` L77：`(GeomType.CYLINDER, GeomType.MESH): CollisionType.CONVEX`
   - `collision_driver.py` L79：`(GeomType.BOX, GeomType.MESH): CollisionType.CONVEX`
   - `collision_driver.py` L80：`(GeomType.MESH, GeomType.MESH): CollisionType.CONVEX`
   - 对照：primitive 表（L48-70）只含 plane/sphere/capsule/box 组合，无 mesh。
2. **NATIVECCD disable 不影响目标组合**
   - `collision_driver.py` L908-920 `_narrowphase`：convex/primitive 分流，
     L919 `convex_narrowphase(...)`；L920 `primitive_narrowphase(...)`。
   - L910-912：`if disableflags & NATIVECCD: collision_table[(BOX,BOX)] = PRIMITIVE` ——
     只改 `(BOX,BOX)`，`(BOX,MESH)`/`(CYLINDER,MESH)` 保持 CONVEX。
3. **convex 窄相把 mesh 对交给带 multiccd 的 CCD kernel**
   - `collision_convex.py` L1236 `convex_narrowphase(...)`；
   - L1266-1270 统计 `nboxmesh/nmeshmesh/ncylmesh`；L1283-1289 因 cyl/mesh 组合扩大
     `npolygonmax/nmeshdegmax`；
   - L1275 `use_multiccd = (m.opt.disableflags & DisableBit.MULTICCD) == 0`；
   - L1432 把 `use_multiccd` 传给 kernel builder。
4. **ccd kernel 的 multicontact 静态门**
   - `collision_convex.py` L756 `ccd_kernel_builder(...)`，L761 `use_multiccd: bool` 参数；
   - L906-910：`(use_multiccd or (BOX,BOX)) and 两侧类型 ∈ {BOX,MESH,CYLINDER}` 才进入；
   - L911-919：mesh 侧若 `mesh_polyadr < 0` 则 `multiccd_idx = -1`（单接触回退）；
   - L922-944：`multicontact(...)` 产出最多 4 个 witness/dist；L946-951 0 接触时回退单 EPA witness；
   - L982-995：按 `ncollision` 逐条 `write_contact(id_=i, ...)` ⇒ 同一 pair 可写多行（nacon>1）。
5. **multicontact / EPA 侧条件**
   - `collision_gjk.py` L2216 `multicontact(...)`（`vec4` dists，≤4 接触，polygon clip）；
   - L2573 `epa_phase(...) -> (dist, ncontact, x1, x2, multiccd_idx)`；
   - L2669-2677：`margin != 0` 或任一侧类型不在 {BOX,MESH,CYLINDER} → `multiccd_idx = -1`
     （单接触）。
6. **功能验证（CPU 预检，见 `precheck-cpu.json`）**
   - `mesh-box`：MULTICCD on ⇒ nacon=4（geomcollisionid=[0,1,2,3]，dist 全 -0.010000002）；
     MULTICCD off ⇒ nacon=1。
   - `mesh-cylinder`：MULTICCD on ⇒ nacon=4（ids=[0,1,2,3]，dist 全 -0.010000005）；
     MULTICCD off ⇒ nacon=1。
   - 两者 mesh 均 `polygon_data_present=true`。

## 对 M1 的含义

- M1 捕获目标 = 上游 GPU 下 mesh-box / mesh-cylinder 的 multicontact 输出（nacon 预期 4，
  GPU 侧以实际捕获为准；CPU 预检只证 fixture 有效）。
- mesh-mesh 预留：同 CONVEX + multicontact 分支，M1 不给 case；M2/M3 若做 mesh-mesh 需另行
  捕获。
