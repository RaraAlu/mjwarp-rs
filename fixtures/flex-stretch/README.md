# 柔体拉伸投影参考

本目录保留十二组静态状态。
MuJoCo版本采用3.12.0。
样本号固定为2601。
产品测试只读取静态文件。
产品测试不生成原生参考。

## 模型与布局

模型包含五组柔体。
刚性三角形保持静态。
线性格点提供八个节点。
节点用于检查顶点偏移。
直接网格包含四个三角形。
实体网格包含六个四面体。
线段提供一维停用分支。
模型合计47顶点与131边。
总单元数为61。
活动拉伸单元使用210系数。
每单元保留21系数跨度。
三角形拉伸只读取前六项。
本参考不覆盖24系数布局。

## 参考方法

工具编译本目录XML。
工具保存本目录MJB。
工具先将材料与静长舍入f32。
工具也将状态舍入f32。
其余参考计算使用f64。
工具调用原生运动学函数。
工具从原生顶点测量边长。
MuJoCo不提供缓存矩阵字段。
工具独立组装稠密坐标导数。
工具不复制GPU边块累加。

令D表示平方边长的导数。
令K表示对称材料矩阵。
令t表示K乘平方长度增量。
工具将负张力截断为零。
工具按以下公式组装矩阵：

```text
H = 0.5 D^T K D + sum(max(t_e, 0) s_e s_e^T ⊗ I_3)
```

工具再抽取顶点与有向边块。
顶点块采用六项上三角顺序。
边块采用九项行优先顺序。
矩阵采用冻结上游的投影。
压缩时它不等于普通能量海森。
本参考不证明原生弹性等价。

## 开发期作者工具

作者工具只依赖原生C++链。
产品测试不调用该工具。
开发者先配置MSVC与原生包。
然后执行以下命令：

```powershell
$pkg = Join-Path $PWD 'target/toolchains/mujoco-3.12.0/package'
$dir = Join-Path $PWD 'target/flex-stretch-reference'
New-Item -ItemType Directory -Force $dir | Out-Null
cl.exe /nologo /W4 /WX /MD /O2 /std:c++17 /EHsc "/I$pkg/include" fixtures/flex-stretch/reference.cpp "/Fo$dir/reference.obj" "/Fe$dir/reference.exe" /link "$pkg/lib/mujoco.lib"
$env:PATH = "$pkg/bin;$env:PATH"
& "$dir/reference.exe" fixtures/flex-stretch/stretch.xml fixtures/flex-stretch/stretch.json fixtures/flex-stretch/stretch.mjb
```

编译器会提示停用被动力。
本参考只采集几何和材料。
它不运行完整弹性阶段。
清单锁定XML、MJB、JSON与源码。
清单也锁定原生DLL哈希。
样本文本固定使用LF。
MJB保持二进制格式。
开发者重建后须更新清单。
数值容限保持不变。
