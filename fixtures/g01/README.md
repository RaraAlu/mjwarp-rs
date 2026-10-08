# G01完整链参考

## 来源与边界

参考采用MuJoCo 3.12.0。
作者工具采用原生C++。
作者工具只在开发时运行。
产品测试不启动作者工具。
产品测试只读取冻结参考。
测试不调用CPU物理阶段。
测试不启动Python。

冻结GPU语义使用以下提交：

```text
google-deepmind/mujoco_warp
71da24d956378a87a703b6e1442b13aec0c4ac29
```

`manifest.json`锁定四项摘要。
它也锁定原生DLL摘要。
MuJoCo采用Apache-2.0许可。
本仓库保留既有许可清单。
本样本没有外部网格资产。

## 样本组成

| 文件 | 用途 |
| --- | --- |
| `scene.xml` | 原始场景与四类关节 |
| `scene.mjb` | 作者工具保存的编译模型 |
| `reference.cpp` | 离线原生参考作者 |
| `reference.json` | 78项模型字段与16组状态 |
| `manifest.json` | 文件、DLL与上游身份 |

模型覆盖直接与线性柔体。
模型包含实体与线性壳体。
相机与光源各有八个对象。
它们覆盖五类跟随模式。
负目标编号触发固定回退。
模型包含十条混合肌腱。
路径覆盖滑轮与球柱绕行。
模型包含mocap与静态几何。
限位肌腱提供唤醒输入。

作者工具另做三项修改。
它设置壳体插值值为-1。
它修改壳体局部节点偏移。
它设置负目标的跟随模式。
因此XML不等同最终MJB。
作者工具也启用直接边阻尼。
`mj_setConst`建立稀疏行。
作者随后保存编译模型。

## 数值参考

作者使用16组确定性输入。
JSON保留种子标记2501。
状态公式不调用随机数。
作者先把状态舍入至f32。
原生阶段随后采用f64计算。
输入含有限非单位四元数。

作者调用以下原生阶段：

```text
mj_kinematics -> mj_comPos -> mj_camlight -> mj_flex -> mj_tendon
```

原生库跳过插值边长度。
作者用原生顶点测量这些边。
`mju_sub3`与`mju_norm3`完成测量。
原始边长度另存独立字段。
直接边雅可比来自原生库。
边速度采用原生稀疏行乘qvel。
插值边的原生稀疏行为空。

节点位置使用原生矩阵运算。
壳体面采用独立极分解参考。
`mju_eig3`提供对称特征分解。
GPU实现使用另一条SVD路线。
面四元数采用xyzw顺序。
比较允许等价的整体符号。
睡眠测试另用独立树状态公式。
原生输出不提供Warp树状态。

## 作者命令

先加载MSVC x64开发环境。
再执行以下开发命令：

```powershell
$root = 'C:\Rust\mjwarp-rs'
$package = Join-Path $root 'target\toolchains\mujoco-3.12.0\package'
$out = Join-Path $root 'target\g01-verification'
New-Item -ItemType Directory -Force $out | Out-Null
cl.exe /nologo /std:c++17 /EHsc /W4 /WX "/I$package\include" `
    "$root\fixtures\g01\reference.cpp" "/Fo$out\reference.obj" "/Fe$out\reference.exe" `
    /link "/LIBPATH:$package\lib" mujoco.lib
$env:PATH = "$package\bin;$env:PATH"
& "$out\reference.exe" "$root\fixtures\g01\scene.xml" `
    "$root\fixtures\g01\reference.json" "$root\fixtures\g01\scene.mjb"
```

MuJoCo报告四条柔体提示。
它们提示缺少等式与被动力。
本参考只验证位置阶段。
作者命令不进入验收脚本。
重新生成后须更新摘要。
验收不能自动重生成参考。
