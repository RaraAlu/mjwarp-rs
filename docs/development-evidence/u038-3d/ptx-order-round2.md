# U038 三维运动学 PTX 算序对齐 · 两轮有界结果（未达 0 超差）

任务：`2026-10-23-u038-3d-ptx-order-alignment`。
方法依据：G03 矩阵链做法（把被 NVRTC 收缩的表达式改成上游生成代码的局部乘积写法/显式标量指令）。
本轮不改容差、断言、排除清单与 ABI；产品源码改动在收口时已回退，未做 T4 repin。

## 基线（run6，修改前）

- 全链 qfrc_gravcomp：**8/77 超差**，总位差 122，max 4.291534423828125e-6。
- 字段位差：xpos 4、xquat 7、xmat 17、xipos 4、xanchor 3、xaxis 4、subtree_com 9、cdof 55。

## 第 1 轮：局部乘积写法（照上游生成代码语句结构）

改动：把 `KINEMATICS_CUDA` 的 `normalize`/`multiply`/`rotate` 改成与 Warp 生成 C++ 相同的
straight-line 局部临时量（每步显式 mul/add/sub 顺序），保持代数语义不变。

结果（run7）：**8/77 超差不变**，总位差 122 → 123，max 不变（4.2915e-6）。
首差位置与 run6 相同（xpos/xipos/xanchor 最早在 world2 body3，body2 自身逐位一致）。
说明 NVRTC 对这两种写法给出同一收缩选择，局部写法本身不改变问题。

## 第 2 轮：显式 PTX 标量指令（禁止 FMA 收缩）

改动：把上述三个基元整体改用 `mul.f32/add.f32/sub.f32` 内联汇编标量（不产生任何 FMA），
排除 NVRTC 的融合选择。

结果（run8）：**8/77 超差不变**，总位差 122 → 147（变差），max 不变。
说明上游生成 PTX 并非「全不收缩」：它使用的是**混合收缩模式**（部分表达式融合、部分保持分离），
全显式或全自然两种极端都不能复现。

## 结论（有界两轮耗尽）

- 8/77 超差仍未归零；两种系统性写法都无法复现上游 FMA 收缩选择。
- 上游收缩模式需按表达式逐条从上游 PTX 反推（每条 mul/add 组合是否被融合），并在产品侧用
  显式 `fma.rn`/`mul`/`add` 序列逐项固定；这是一项需要 PTX 级对照工具与逐内核转录的独立系统性任务，
  不适合在「两轮有界」内继续硬试。
- 未改容差/断言/排除清单/ABI；产品源码已回退到修改前版本，无需 T4 repin；未跑 GPU 内核，仅
  定向测试+PTX 编译，GPU 已释放。
- 建议：由 Chief/用户裁决——（a）另立「逐表达式 PTX 收缩模式转录」系统性任务（需要产品 PTX
  dump 工具与上游 PTX 对照）；或（b）将 8 项登记为已知债务/比较口径问题。

## 证据

- run6（基线）：`target/u038-3d/run6/final/run.json` + `product-fields-raw.json`
- run7（第 1 轮）：`target/u038-3d/run7/final/run.json`
- run8（第 2 轮）：`target/u038-3d/run8/final/run.json`
- 首差证据：`docs/development-evidence/u038-3d/first-diff-round1.md`、`product-fields-raw.json`
