# impact 记录

## 模块级 impact（本片采用口径，命令 exit 0）

```text
$ normify impact mjwarp.render
mjwarp.render（active，源码39，API11）GPU批量渲染
  依赖：mjwarp.model.core，mjwarp.runtime.completion，mjwarp.math，mjwarp.diagnostics，mjwarp.model.core，mjwarp.diagnostics
  约束规则：core-no-deprecated-target，boundary-render
  未关闭任务：2026-10-11-roadmap-m03-p8(proposed)，…（历史任务，与本片无关）
```

## 文件级 impact 不适用（既有治理缺口，非门禁）

```text
$ normify impact src/render/kernel/cast.rs
没有模块对应：src/render/kernel/cast.rs        # 退出码 1 —— N/A（未登记，非门禁）
```

- `src/render/kernel/` 下 6 个 `.rs` 未登记为模块源（`cast.rs`、`megakernel.rs`、
  `megakernel_uploads.rs`、`megakernel_aa.rs`、`megakernel_arrays.rs` 等），
  属既有缺口，已由 backlog **`2026-10-28-arch-volume-module-sources`** 覆盖。
- 本片**不登记模块源**：登记会改架构登记，需要另立片并跑 `normify validate` 验证，超出本卡 scope。
- 替代口径 = **模块级 impact（`mjwarp.render`，exit 0）+ 任务 scope 校验（卡面 files 只有
  `src/render/kernel/cast.rs` 与本证据目录）+ `normify validate --fingerprints` 错误 0**。
- `materials/gates.txt` 里的所有门禁行均为 exit 0；上面这条失败命令已显式标注
  `N/A（未登记，非门禁）`，不计入门禁结果。
