# 线性壳体面参考

本目录保留静态面参考。
MuJoCo版本采用3.12.0。
随机样本号固定为2401。
参考包含十二组状态。
每组包含十六个面。
产品测试只读取这些文件。
产品测试不生成参考。

## 模型来源

模型复用[柔体位置样本](../flex-position/README.md)。
输入采用已有`flex-tree.mjb`。
作者工具不重新编译XML。
工具将两项插值改为负一。
工具不保存或修改原MJB。
两组格点均没有内部节点。
原生TFI不改变这些节点。
面数分别为十项与六项。
状态沿用原样本公式。
qpos与mocap先舍入至f32。
其余参考计算使用原生精度。

## 参考方法

工具调用原生`mj_kinematics`。
工具调用原生`mj_comPos`。
工具调用原生`mj_flex`。
顶点结果来自原生字段。
节点结果使用原生变换函数。
每面引用四个真实节点。
末五点严格填零。
循环枚举六侧与面内格点。
映射使用全局节点编号。

MuJoCo不提供面位姿字段。
面位置直接采集原生节点。
面切线采用对边差分平均。
法线采用原生叉乘函数。
工具形成三列变形矩阵。
工具求解矩阵的极分解。
工具调用`mju_eig3`分解。
它分解对称矩阵`F^T F`。
工具计算`R=F(F^T F)^(-1/2)`。
它使用`mju_mat2Quat`取旋转。
它将四元数改为xyzw顺序。
这个参考不复制GPU迭代。
它不证明完整原生阶段等价。
奇异矩阵没有本参考输出。
Rust解析测试另查退化状态。

## 文件与哈希

| 文件 | 用途 |
| --- | --- |
| `face-state.json` | 面映射、状态与面参考 |
| `reference.cpp` | 开发期原生作者工具 |
| `manifest.json` | 原生版本与五项哈希 |

清单锁定已有XML、MJB与JSON。
清单也锁定本目录参考与源码。
DLL哈希沿用原样本。

## 开发期再生成

请先加载MSVC环境。
以下命令只供参考作者使用。
产品脚本不调用这些命令。

```powershell
$pkg = Join-Path $PWD 'target/toolchains/mujoco-3.12.0/package'
$dir = Join-Path $PWD 'target/flex-face-verification'
New-Item -ItemType Directory -Force $dir | Out-Null
cl.exe /nologo /W4 /WX /MD /O2 /std:c++17 /EHsc "/I$pkg/include" fixtures/flex-face/reference.cpp "/Fo$dir/reference.obj" "/Fe$dir/reference.exe" /link "$pkg/lib/mujoco.lib"
$env:PATH = "$pkg/bin;$env:PATH"
& "$dir/reference.exe" fixtures/flex-position/flex-tree.mjb fixtures/flex-face/face-state.json
```

生成后须更新两项内容哈希。
请勿将作者工具加入产品链。
