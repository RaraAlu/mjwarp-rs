# 相机与光源静态参考

本目录固定G01辅助样本。
原生候选版本为3.12.0。
冻结上游提交见manifest。
产品测试只读取静态文件。
产品测试不编译XML。
产品测试不执行参考生成器。
构建与运行不使用Python。

## 内容

| 文件 | 用途 |
| --- | --- |
| `mixed-tree.xml` | 保存参考模型源 |
| `mixed-tree.mjb` | 保存编译模型 |
| `mixed-tree.json` | 保存字段与八组结果 |
| `reference.cpp` | 提供开发期原生生成器 |
| `manifest.json` | 固定SHA256与原生版本 |

样本包含6个体与2个自由度。
样本包含1个mocap体。
相机与光源各有8项。
样本覆盖全部五种模式。
样本包含世界体附着。
关节与mocap改变跟踪目标。
固定子体改变子树质心。
初始常量来自已编译模型。

生成器先编译固定回退项。
随后改写第6与第7项模式。
两项分别采用负目标-2与-1。
两项分别采用模式3与4。
XML因此不同于最终MJB。
其余字段保留编译结果。
生成器保存修改后的MJB。

固定种子为1789。
生成器将状态先转成f32。
原生计算保留f64结果。
绝对与相对容限均为2e-5。
Rust测试不放宽这些容限。
退化目标另采用冻结公式。
原生退化回退不能代替Warp。

## 开发期生成

在MSVC开发环境运行：

```powershell
cl.exe /nologo /std:c++17 /EHsc /O2 `
  /Itarget/toolchains/mujoco-3.12.0/package/include `
  fixtures/camlight/reference.cpp `
  /Fetarget/camlight-reference/reference.exe `
  /Fotarget/camlight-reference/reference.obj `
  /link /LIBPATH:target/toolchains/mujoco-3.12.0/package/lib mujoco.lib
target/camlight-reference/reference.exe `
  fixtures/camlight/mixed-tree.xml `
  fixtures/camlight/mixed-tree.json `
  fixtures/camlight/mixed-tree.mjb
```

先创建目标目录。
先将原生bin目录加入PATH。
生成后更新manifest哈希。
不要将生成器接入产品测试。
