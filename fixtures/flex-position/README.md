# 柔体位置原生参考

日期：2026-10-07。
本目录保存静态参考。
产品测试只读取这些文件。
产品测试不启动生成器。
构建与测试不引入Python。

## 样本内容

| 项目 | 数量 |
| --- | --- |
| MuJoCo版本 | 3.12.0 |
| 确定性状态 | 12 |
| 柔体 | 4 |
| 节点 | 20 |
| 顶点 | 80 |
| 关节 | 63 |
| 自由度 | 65 |

样本覆盖直接与线性插值。
样本覆盖居中与局部偏移。
线性网格含两个x方向单元。
样本含关节与mocap运动。
坐标包含网格端点与内部点。
编译器留下微小越界坐标。
GPU沿用上游钳制规则。

参考工具先编译源XML。
工具再改写部分节点偏移。
它将首个线性柔体设为非居中。
它改写首尾节点的局部坐标。
MJB保存改写后的模型。
JSON保存相同模型字段。
XML需经参考工具才能复现。
位置参考不要求弹性力。
编译器因此提示四条警告。
这些警告不阻止模型编译。

顶点参考来自原生柔体函数。
原生Data不公开节点位置。
工具调用原生矩阵变换函数。
工具据此保存节点参考。
输入状态先舍入至f32。
原生函数随后采用其原始精度。
绝对与相对容限均为`2e-5`。

## 独立生成

开发者手动编译C++工具。
产品脚本不执行此流程。
请先配置MSVC与MuJoCo目录。

```powershell
New-Item -ItemType Directory -Force target/flex-position-verification | Out-Null
$env:PATH = "$env:MJWARP_MUJOCO_ROOT/bin;$env:PATH"
cl.exe /nologo /W4 /WX /MD /O2 /std:c++17 /EHsc `
  /I"$env:MJWARP_MUJOCO_ROOT/include" fixtures/flex-position/reference.cpp `
  /Fotarget/flex-position-verification/reference.obj `
  /Fetarget/flex-position-verification/reference.exe `
  /link "$env:MJWARP_MUJOCO_ROOT/lib/mujoco.lib"
target/flex-position-verification/reference.exe `
  fixtures/flex-position/flex-tree.xml `
  fixtures/flex-position/flex-tree.json `
  fixtures/flex-position/flex-tree.mjb
```

生成后核对版本与哈希。
`manifest.json`锁定四个文件。
它另锁定原生DLL哈希。
本参考不验收完整G01。
本参考不验收完整G22。
