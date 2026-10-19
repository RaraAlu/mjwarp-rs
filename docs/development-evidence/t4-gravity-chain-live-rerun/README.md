# 重力非零链实测复验

任务：`2026-10-19-t4-gravity-chain-live-rerun`。
前置：`2026-10-19-t4-resident-source-drift`。
基线提交：`be6ccf756cdf08b1d27264bcdba898f46515c8cf`。
平台：Rara，Windows x64 MSVC。
显卡：RTX 4070 Ti SUPER。
驱动：596.36，显存16376 MiB。
资源锁：`gpu:windows-t4`。

## 结论

新证据绑定新包摘要。
摘要为`c4752d60…d199`。
比较共1134项。
位差为零。
超容差项为零。
最大绝对误差为零。
原2e-6容差未变。

## 执行

命令如下：

```text
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --test t4_gravity_nonzero_product_chain -- --ignored --test-threads=1
```

`CUDA_PATH`指向CUDA 12.8.1。
证据目录指向`gpu-4`。
脚本先编译，再计时。

| 运行 | PTX捕获 | 结果 | libtest | 整体 |
| --- | --- | --- | --- | --- |
| run-01 | 未设置 | 1过0败 | 1.14 s | 1439.5 ms |
| run-02 | 已设置 | 1过0败 | 0.86 s | 1038.9 ms |

run-01遗漏PTX捕获。
gpu-3设置了`MJWARP_PHYSICS_PTX_DIR`。
run-02补设该变量。
run-01产物原样移入本目录。
两次数值文件逐字节一致。
`gpu-4`采用run-02产物。

## 显存

脚本以50 ms采样。
空闲占用4838 MiB。
峰值占用5056 MiB。
结束后回到4838 MiB。
峰值增量约218 MiB。
桌面程序共用此卡。
WDDM不给单进程显存。
因此增量只作参考。
本次不作性能验收。

## 与gpu-3比较

两侧共30个文件。
25个文件逐字节一致。
其余5个只差包摘要。
替换旧摘要后逐字节一致。
八个入口的CUDA一致。
PTX与编译回执也一致。
见[比较日志](comparison/gpu3-vs-gpu4.log)。

## 未验收项

本次未改源码与测试。
本次未跑满额回归。
本次未跑release构建。
本次未跑Linux验证。
NP01至NP04仍未验收。
完整三维状态仍未覆盖。
产品测试没有启动Python。
