# 柔体边静态原生参考

日期：2026-10-08。
原生版本：MuJoCo 3.12.0。
固定种子：2301。
样本数：12。

本目录不加入产品生成链。
Rust测试只读取静态文件。
开发者使用C++生成参考。
产品测试不启动Python。

## 模型与字段

本目录复用柔体位置模型。
基础模型见[原样本](../flex-position/README.md)。
基础MJB保留节点偏移覆盖。
原XML需先执行其作者程序。
本目录不复制XML与MJB。
清单同时固定三项基础哈希。

| 项目 | 数量 |
| --- | --- |
| 柔体 | 4 |
| 节点 | 20 |
| 顶点 | 80 |
| 边 | 284 |
| 稀疏池 | 10 |
| 活跃直接边 | 6 |
| 插值边 | 278 |
| 参考状态 | 12 |

直接边覆盖世界与mocap体。
它们覆盖铰链、滑动与球关节。
插值边覆盖两种节点网格。
qvel采用固定正弦样本。
作者程序先将物理输入转为f32。
原生计算仍使用自身mjtNum。

## 原生计算与差异

作者程序读取冻结MJB。
它先保存未启用边力的稀疏行。
`inactive_model`保留零行元数据。
原生池仍预留十个槽。

程序再启用直接边阻尼。
程序调用`mj_setConst`。
该调用重建原生稀疏行。
程序不修改基础MJB。
它不提供产品弹性或阻尼。

程序依次调用原生入口：

```text
mj_kinematics
mj_comPos
mj_flex
```

直接边雅可比来自`mj_flex`。
速度采用原生稀疏行点积。
原生`mj_flex`跳过插值边长。
原生顶点仍提供几何参考。
程序用原生向量函数测距。
`flexedge_length`采用这个结果。
`native_flexedge_length`保留原值。
测试不混淆这两类结果。
插值边的稀疏行保持空行。

冻结Warp会计算全部边长。
本子集沿用这个分支。
本子集每轮清零整个稀疏池。
它不保持外部写入的脏值。
这些证据不证明完整flex等价。

## 开发者生成命令

先准备MSVC与MuJoCo目录。
下列命令不属于产品测试。

```powershell
$pkg = Join-Path $PWD 'target/toolchains/mujoco-3.12.0/package'
$dir = Join-Path $PWD 'target/flex-edge-verification'
New-Item -ItemType Directory -Force $dir | Out-Null
cl.exe /nologo /W4 /WX /MD /O2 /std:c++17 /EHsc "/I$pkg/include" `
    fixtures/flex-edge/reference.cpp "/Fo$dir/reference.obj" `
    "/Fe$dir/reference.exe" /link "$pkg/lib/mujoco.lib"
$env:PATH = "$pkg/bin;$env:PATH"
& "$dir/reference.exe" fixtures/flex-position/flex-tree.mjb `
    fixtures/flex-edge/edge-state.json
```

生成后重新计算清单哈希。
不要让产品测试更新参考。

[冻结边源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L347-L414)。
[原生柔体计算](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/engine/engine_core_smooth.c)。
[原生常量初始化](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/engine/engine_setconst.c)。
