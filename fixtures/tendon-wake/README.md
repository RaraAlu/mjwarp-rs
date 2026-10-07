# 肌腱唤醒静态参考

本目录覆盖G01局部副作用。
原生版本固定为3.12.0。
上游提交固定如下。

```text
71da24d956378a87a703b6e1442b13aec0c4ac29
```

## 样本组成

本样本复用[混合模型](../mixed-tendon/README.md)。
它保留三个动态树。
它保留十四条全局肌腱。
固定、绕行与滑轮连接两棵树。
范围周期采用三。
边距周期采用二。
六类状态含单树与多树循环。
四轮生成二十四组原生参考。
清单锁定参考、模型与工具。
产品测试只读取静态文件。
产品链不调用参考工具。
产品链不启动Python。

## 参考差异

工具调用原生运动学入口。
该入口执行原生肌腱唤醒。
工具调用原生树更新辅助。
该辅助导出符号固定版本。
工具不复制CUDA唤醒算法。
原生会过滤睡眠肌腱。
工具先记录全部唤醒时的长度。
随后记录过滤长度与树状态。

| JSON字段 | 来源 |
| --- | --- |
| `ten_length` | 全部树唤醒时的原生长度 |
| `native_filtered_ten_length` | 睡眠过滤后的原生长度 |
| `tree_asleep` | 原生唤醒后的循环与计数 |
| `tree_awake` | 原生树活动标记 |
| `native_nbody_awake` | 原生完整身体计数 |
| `native_nv_awake` | 原生完整自由度计数 |

GPU辅助仍重算全部肌腱。
测试单独核对全部长度。
测试逐项核对整数树状态。
冻结Warp采用轻量树刷新。
它清零身体与自由度计数。
测试不把原生完整计数当真值。
冻结Warp先取完全唤醒值。
该值为负十一。
原生直接沿用源树计数。
原生参考仅使用负十一及更小值。
解析测试另查负一源树。

## 开发者生成

先配置原生包路径。
再初始化MSVC命令环境。
以下命令只供开发者使用。

```powershell
$pkg = $env:MJWARP_MUJOCO_ROOT
cl.exe /nologo /W4 /WX /MD /O2 /std:c++17 /EHsc `
  "/I$pkg/include" fixtures/tendon-wake/reference.cpp `
  /Fotarget/tendon-wake-verification/reference.obj `
  /Fetarget/tendon-wake-verification/reference.exe `
  /link "$pkg/lib/mujoco.lib"
$env:PATH = "$pkg/bin;$env:PATH"
target/tendon-wake-verification/reference.exe `
  fixtures/mixed-tendon/mixed-tree.mjb `
  fixtures/tendon-wake/tree-state.json
```

首次生成前创建输出目录。
更新参考后重新核对清单哈希。
不得在产品测试中执行此流程。

[冻结唤醒源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/sleep.py)。
[原生运动学入口](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/engine/engine_forward.c#L119-L128)。
[原生唤醒源码](https://github.com/google-deepmind/mujoco/blob/3.12.0/src/engine/engine_sleep.c)。
