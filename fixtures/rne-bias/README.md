# RNE偏置力静态参考

本目录覆盖G02刚体偏置力。
它不代表完整G02验收。
参考采用MuJoCo 3.12.0。
本批复用五组冻结MJB。
生成器只用于离线样本制作。
产品测试只读取静态JSON。
产品测试不运行原生物理。
产品测试不启动Python。

每个模型包含八组固定状态。
种子固定为1789。
输入qpos与qvel先量化至f32。
每组状态包含三种共享重力。
原生输出保持double精度。
参考覆盖四类关节与静态祖先。
零自由度与零质量均有样本。
自由关节另测纯平移与纯旋转。
manifest冻结生成器与模型摘要。
它也冻结五个JSON摘要。

```text
gravity = [0, 0, -9.8100004196166992]
gravity = [0, 0, 0]
gravity = [1.25, -2, 3.5]
abs(actual-reference) <= 2e-5 + 2e-5*abs(reference)
```

GPU验收前已固定上述容限。
生成器不调用Rust候选内核。
生成器只调用以下原生阶段：

```text
mj_kinematics -> mj_comPos -> mj_comVel -> mj_rne(flg_acc=0)
```

离线制作示例：

```bash
g++ -std=c++17 -O2 -Wall -Wextra -Werror \
  -I target/toolchains/mujoco-3.12.0/linux-package/include \
  fixtures/rne-bias/reference.cpp \
  target/toolchains/mujoco-3.12.0/linux-package/lib/libmujoco.so.3.12.0 \
  -Wl,-rpath,"$PWD/target/toolchains/mujoco-3.12.0/linux-package/lib" \
  -o target/rne-bias-reference
target/rne-bias-reference fixtures/native-probe/mixed-joints.mjb \
  fixtures/rne-bias/mixed-joints.json
```

不要将此命令接入产品测试。
复核结果见[RNE报告](../../docs/linux-resident-rne-bias.md)。
