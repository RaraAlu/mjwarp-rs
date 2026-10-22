# U038-3D 重力传播债务 · 首差定位（有界第 1 轮）

日期：2026-10-23。任务：`2026-10-23-u038-3d-gravity-propagation-debt`。
目标：定位全链 `qfrc_gravcomp` 8/77 超差的来源，判断能否单点修复到 1477 项 0 超差。
本轮只定位与取证据，未改产品源码、容差、断言。

## 方法

- 在定向 GPU 测试（`t4_gravity_3d_product_chain`）里增加环境变量开关
  `MJWARP_U038_DUMP_PRODUCT`：仅在设置时把产品八字段的原始 f32 位导出到
  `product-fields-raw.json`，不改变任何断言/容差/比较逻辑。
- 用 run6 导出（`target/u038-3d/run6/final/`）与冻结上游 `three-d-kinematics/output.json`
  逐字段逐索引比对，映射到 world/body/dof。

## 首差结果（product vs frozen upstream，按字段索引顺序）

| 字段 | 位差数 | 首个位差索引 | 映射（world, body/joint, component） |
| --- | ---: | ---: | --- |
| xpos | 4 | 39 | world2, body3(b3 hinge), x |
| xquat_wxyz | 7 | 128 | world6, body2(b2 ball), 分量0 |
| xmat | 17 | 288 | world2, body2, 元素0 |
| xipos | 4 | 39 | world2, body3, x |
| xanchor | 3 | 30 | world2, joint2(b3 hinge), x |
| xaxis | 4 | 55 | world3, joint3(b4 slide)? 索引55→world3,joint3, 分量1 |
| subtree_com | 9 | 32 | world2, body2, z（最大 11 ULP） |
| cdof | 55 | 153 | world2, dof 3（b1 free 的旋转自由度分量） |

关键定位：world2 的 body2（ball 关节）自身 `xpos/xquat` 与上游**逐位一致**，但以其为父的
body3 `xanchor`/`xpos` 出现 1–2 ULP 差异。因此首差发生在「用父姿态旋转子体/关节局部量」的
路径：`rotate(parent_quat, local)` 与随后 `multiply`/`normalize` 的组合，而不是 ball 关节
自身解析或重力算子。重力算子此前已验证 77/77 0 超差。

## 与上游生成代码的逐式对照

对照产物：`target/mjwarp-parallel/cache-quarantine/warp-1.15.0-double-version-2026-10-18/`
`wp_mujoco_warp._src.smooth_ca013d6/wp_mujoco_warp._src.smooth_ca013d6.cu`（生成 3D 参考所用同一缓存）。

| 产品 CUDA（`src/physics/mod.rs`） | 上游生成代码 | 结论 |
| --- | --- | --- |
| `rotate`: `2*(u·v)*u + (w²-u·u)*v + (2w)*(u×v)`，mul→mul→add 顺序 | `rot_vec_quat_0`: `mul(2.0, mul(dot,u))`、`sub(mul(s,s), dot)`、`mul(factor,vec)`、`add`、`mul(2.0,s)`→`cross`→`add` | 代数与逐项乘法/加法顺序一致（×2 为精确缩放，交换律成立） |
| `multiply`: real/ i / j / k 四分量按 `((a-b)-c)-d` 与 `((a+b)+c)-d` | `mul_quat_0` 生成代码同序 | 一致 |
| `normalize`: `sqrt(w²+x²+y²+z²)` 后 `1/n` 逐分量乘 | 头文件 `normalize(quat)`：`sqrt(dot)`、`inv_l=1/l`、逐分量乘 | 一致（分量顺序按存储槽对齐） |
| 关节/自由/球/铰链分支与写回顺序 | `_kinematics_branch` 生成代码逐行对应 | 一致 |

结论：产品与上游在**表达式层面代数一致**，差异不是公式写错；与 G03 矩阵链同样的
「NVRTC 对自然表达式做 FMA 收缩/融合的取舍与上游生成 PTX 不同」类别。G03 当初通过显式
`mul/add/sub/fma` 内联汇编固定算序解决；本任务的剩余差异需要同样的 **PTX 级算序合约转录**
（对 rotate/multiply/normalize 及 com/cdof 累加逐条固定收缩点），属于跨多个表达式的系统性对齐，
不是单点小改。

## 有界结论

- 首差已定位到「父姿态旋转变换 + 四元数组合」路径（world2 body3 anchor/xpos 为首个可观测点），
  排除了重力算子、参数映射（`dof_bodyid`/`body_isdofancestor`/`jnt_actgravcomp`）与参考矩阵问题
  （这些此前已分别验证/修复）。
- 单点修复不可达：差异来自编译器级 FMA 收缩选择，需要按上游 PTX 逐表达式固定算序（与 G03 同类
  但覆盖面更大），预估属于独立系统性任务，不宜在「两轮有界」内硬做。
- 未改容差、断言、排除清单；产品源码未改。
- 本轮之后建议：要么另立「三维运动学 PTX 算序对齐」任务（产品源码 + T4 repin），要么按用户裁决
  保留该 8 项为已知债务/比较口径问题。证据文件：
  `docs/development-evidence/u038-3d/product-fields-raw.json`、`docs/development-evidence/u038-3d/first-diff-round1.md`。
