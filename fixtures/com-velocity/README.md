# 空间速度静态参考

本目录覆盖G02刚体速度子集。
它不代表完整G02验收。
参考采用MuJoCo 3.12.0。
本批复用既有冻结MJB。
生成器只用于离线样本制作。
产品测试只读取静态JSON。
产品测试不运行原生物理。
产品测试不启动Python。

每个模型包含八组固定状态。
种子固定为1789。
输入qpos与qvel先量化至f32。
原生输出保持double精度。
参考覆盖四类关节与静态祖先。
零自由度与零质量均有样本。
自由关节另测纯平移与纯旋转。
manifest冻结生成器与模型摘要。
它也冻结五个参考JSON摘要。

```text
abs(actual-reference) <= 2e-5 + 2e-5*abs(reference)
```

GPU验收前已固定上述容限。
生成器不调用Rust候选内核。
生成器只调用以下原生阶段：

```text
mj_kinematics -> mj_comPos -> mj_comVel
```

离线制作示例：

```bash
g++ -std=c++17 -O2 -Wall -Wextra -Werror \
  -I target/toolchains/mujoco-3.12.0/linux-package/include \
  fixtures/com-velocity/reference.cpp \
  target/toolchains/mujoco-3.12.0/linux-package/lib/libmujoco.so.3.12.0 \
  -Wl,-rpath,"$PWD/target/toolchains/mujoco-3.12.0/linux-package/lib" \
  -o target/com-velocity-reference
target/com-velocity-reference fixtures/native-probe/mixed-joints.mjb \
  fixtures/com-velocity/mixed-joints.json
```

不要将此命令接入产品测试。
复核结果见[速度报告](../../docs/linux-resident-com-velocity.md)。
