# 几何与site固定参考

日期：2026-10-07。
范围：G01附着位姿辅助子集。
参考采用MuJoCo 3.12.0。
它仍属于原生候选版本。
冻结Warp基线保持不变。

## 样本

| 样本 | 覆盖范围 |
| --- | --- |
| attached-tree | 四类关节、偏心旋转与多附着 |
| static-frames | 零自由度、世界体与静态子树 |
| sites-only | 无geom、ball关节与site |
| geoms-only | 无site、四类关节与geom |
| empty-frames | 无geom及site，保留刚体结果 |

前三组包含XML与可信MJB。
geoms-only复用原生输入MJB。
empty-frames复用刚体旋转MJB。
清单同时核对复用输入哈希。

每组固定八项状态。
随机种子固定为1789。
输入qpos先窄化为f32。
原生输出保持双精度。
输出包含四项附着位姿。
绝对与相对容限均为2e-5。
测试不运行原生运动学。
测试也不编译XML或启动Python。

## 独立制备

`reference.cpp`只供参考制备。
它调用官方`mj_kinematics`。
它不调用Rust或GPU实现。
产品构建与测试不执行它。

先进入MSVC x64开发环境。
以下命令不属于产品测试。

```powershell
$m = (Resolve-Path target/toolchains/mujoco-3.12.0/package).Path
$env:PATH = "$m/bin;" + $env:PATH
New-Item -ItemType Directory -Force target/attached-reference | Out-Null
cl /nologo /EHsc /W4 /WX /std:c++17 /I"$m/include" fixtures/attached-kinematics/reference.cpp /Fo:target/attached-reference/reference.obj /Fe:target/attached-reference/reference.exe /link /LIBPATH:"$m/lib" mujoco.lib
foreach ($name in @('attached-tree','static-frames','sites-only')) {
    target/attached-reference/reference.exe --xml "fixtures/attached-kinematics/$name.xml" "target/attached-reference/$name.json" "target/attached-reference/$name.mjb"
}
target/attached-reference/reference.exe --mjb fixtures/native-probe/mixed-joints.mjb target/attached-reference/geoms-only.json
target/attached-reference/reference.exe --mjb fixtures/kinematics/rotated-tree.mjb target/attached-reference/empty-frames.json
```

重复制备不得覆盖固定参考。
先核对全部JSON与MJB哈希。
修改样本时另行评审容限。

产品验收执行以下脚本：

```powershell
scripts/test-windows-attached-kinematics.ps1
scripts/test-windows-attached-kinematics.ps1 -Release
scripts/test-windows-attached-kinematics.ps1 -AllFeatures
scripts/test-windows-attached-kinematics.ps1 -AllFeatures -Release
```

数值证据见[探针报告](../../docs/windows-attached-kinematics-probe.md)。
