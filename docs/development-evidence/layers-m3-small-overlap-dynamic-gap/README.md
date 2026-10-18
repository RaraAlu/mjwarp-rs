# M3 Small重叠动态证据

## 结论

本轮新增一项GPU负例。
本轮只改模块内测试。
产品逻辑保持不变。

测试使用真实GPU会话。
设备为本机T4通道。
实际型号如下：

```text
NVIDIA GeForce RTX 4070 Ti SUPER
```

测试复制完整Small三元组。
第二组完整复制第一组。
两组范围完全重复。
两组地址完全相同。
两组标记保持合法。

常规元数据校验通过。
融合元数据精确拒绝。
内部原因为以下内容：

```text
混合融合Small分区冲突
```

安全绑定返回以下错误：

```text
field = mixed_solve_fused_metadata
index = 0
reason = invalid_or_stale_proof
```

## 安全边界

拒绝发生在宿主校验。
候选没有上传GPU。
内核没有读取候选。
测试没有绕过元数据证明。

原元数据绑定保持不变。
原元数据位保持不变。
原RHS位保持不变。
原因子位保持不变。
原输出位保持不变。
原状态位保持不变。
两项发布状态保持不变。

测试随后复用原计划。
同一融合路径恢复成功。
恢复结果满足原容差。

## 正式运行

正式命令如下：

```text
cargo test --locked --features cuda-probe --lib physics::mass_solve::sparse::mixed::solve::tests::gpu_mixed_solve_fused_rejects_in_bounds_small_overlap_without_state_change -- --exact --ignored --nocapture --test-threads=1
```

结果如下：

```text
1 passed
0 failed
393 filtered out
```

正式进程耗时7026毫秒。
测试框架耗时1.06秒。

测试只创建一个计划。
捕获包含十四个入口。
捕获包含四十二个文件。
每个入口各有三件套。
本轮没有同名覆盖。

四十二文件保持一致。
比较基线是前次新捕获。
该比较不恢复旧原件。

原始归档含六十成员。
提取后逐项保持一致。

## 历史边界

历史第二次原件未恢复。
原第十一条仍不充分。
旧任务和旧证据不变。

本轮只补当前动态缺口。
它不改写原任务历史。

## 父级关系

本证据支持重叠故障项。
它只覆盖融合Small路径。
它不转授其他入口结论。

父级M3仍未完成。
父级仍需逐入口审查。
父级四条验收保持不变。

## 未验收项

- Linux没有运行。
- Release没有运行。
- 集成测试没有运行。
- 性能没有验收。
- 显存峰值没有测量。
- NP01至NP04未验收。
