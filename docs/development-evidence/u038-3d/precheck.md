# U038-3D 设计预检

## 现有覆盖缺口

`fixtures/gravity-upstream-abi/nonzero-*` 的 1134 项零位差证据只覆盖平面铰链链：

- nq=nv=4、四个 hinge、nbranch=2、3 个 body 子链；
- 无 free/ball 关节，无三维体旋转，无四元数非零旋转输入；
- 因此不能代表 U038「完整三维状态」。

## 需要补的三维旋转/组合状态算例

新增独立 3D 场景（`three-d-kinematics`，不改动旧平面证据）：

| 项 | 取值 |
| --- | --- |
| bodies | 5（world + b1..b4） |
| 分支 | 2（b4 直接挂 world；b1→b2→b3 串链） |
| 关节类型 | free(b1) + ball(b2) + hinge(b3) + slide(b4) |
| nq / nv | 13 / 11 |
| 世界数 | 7，每世界 qpos 全非零 |
| 旋转覆盖 | 自由关节四元数非零、球关节四元数非零、四个 body 各带非零三维旋转 quat |
| 参数周期 | gravity=2、body_mass=3、body_gravcomp=5（与产品按世界取参口径一致） |
| 重力 | `[0.3, -9.81, 1.2]`（两个周期相同值） |

## 参考生成方式

- 用已批准离线 authoring 框架：pinned Python 3.12.13 + mujoco 3.15.0 + warp-lang 1.15.0 +
  numpy 2.5.4 + mujoco-warp 3.15.0，上游 `71da24d956378a87a703b6e1442b13aec0c4ac29` clean。
- 只读复用冻结 smooth CUDA/PTX 产物（cu/meta/ptx 哈希锁定），两次运行逐位一致。
- 生成上游 kinematics/com/cdof 输出（`three-d-kinematics/`），并派生模型包
  （`three-d-model-package.json`）与产品链包（`three-d-package.json`）。
- 重力参考不是单独数值表：产品测试用冻结上游 gravity 算子产物（`fixtures/gravity-upstream-abi/generated`）
  对同一模型/状态实时计算，再与产品同源链比较。

## 天然校验要求

- 模型包 `body_isdofancestor` 必须与 pinned 上游 `put_model` 的
  `body_isdofancestor` 预计算逐位一致（`natural-validation.json`，不一致即中止生成）。
- 上游两次运行逐位一致；输入/输出/receipt 哈希自洽。
- 保持既有 2e-6 容差与断言语义；产品源码不改（无需 T4 重钉）。

## 边界

- 不宣称完整状态空间、NP03/NP04、release/Linux/性能。
- 旧平面证据保持原样，新 3D 证据单列。
