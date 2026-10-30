# 证据 EOL 卫生（P1-P5）

任务：`2026-10-31-evidence-eol-hygiene`（Arch；离线，零 GPU、零窗口、零冻结）
来源：QA 对 attempt3 复核的 5 条需修（全文档/属性级）+ Chief 追加的两条（A1 manifest、A1 内核身份 EOL 污染）。

## 1. 做了什么

| 项 | 内容 |
| --- | --- |
| **P1** | `sdf-validate-window/README.md` 的「两次 attempt」表述全部改为三次（标题/开头/§2 表加 attempt3 列含 guards `8b34570e…`/aggregate `013babf4…`/2-9 exit；§4 guards-only 加 attempt3；文末后续改为「attempt3 已执行 = INVALID_ROUND，冻结作废」） |
| **P2** | `sdf-validate-window/receipt.json`：`total_artifacts` 10 → **35**（attempt1 5 + attempt2 5 + attempt3 22 + README/receipt/errata 3）、新增 `attempt3` 产物块、`target_copies_retained` 加 attempt3、`window_state` 改为三次 attempt 全 INVALID_ROUND |
| **P3** | `.gitattributes` 加 `docs/development-evidence/render-bvh/** -text -diff`（覆盖 QA 复核目录）+ `docs/development-evidence/g10-p5-mesh-cap20/** -text -diff`；`sphere-dot-a-order-fix`/`sphere-dot-a-order-revert` 的 `manifest.txt` 各 4 条（kernel-identity/no-run/src-diff/validate）按 **committed blob** 重算；8 个 CRLF 工作树副本 `rm` + `git checkout --` 归一到 blob 字节，最终 `i/lf w/lf`、`git status` 干净 |
| **P4** | `.gitattributes` 加 `*.cu text eol=lf`（放在证据 `-text` 规则之前，后者优先）；全仓 .cu 工作树归一（w/crlf **0**，含 `primitive_plane_shapes.cu` 的 169 对）；`MEGAKERNEL_SOURCE` 重测 **88,865 B / `778216b2…`** 不变；`.cu` blob 零改写 |
| **P5** | attempt3 的「6 实体首差」补限定：`render_kernel_flags` 的 **aa2 臂**首差 pixel **29448/(168,122)**、`max_ulp=91`（其余实体 28969/(169,120)、max 73），结论不变；同步进 README §3c 与 `attempts/attempt3/invalid-round-record.txt` |

## 2. A1 内核身份（QA 追加项，LF 口径）

现盘 A1 证据的内核身份被 EOL 污染（工作树 `primitive_plane_shapes.cu` 有 169 对 CRLF）。按 P4 归一后
`cargo clean -p mjwarp-rs` 强制重建重测，与 QA 独立复算一致：

| 项 | before（`307219fe^`） | after（`307219fe`） |
| --- | --- | --- |
| collision assembled source | **254,005 B / `9b7435ab…`** | **254,263 B / `12d50d68…`** |
| PTX | 3,883,740 B / 6 入口 | 3,883,769 B / 6 入口 |

- 旧数字（`255,840/28385801…` → `254,743/f32f0af2…`）**作废**，**冻结规划只用 LF 值**。
- A1 证据按此更新：`logs/nvrtc-identity.txt`、README §3、receipt `kernel_identity`、manifest（**10/10** 条）。
- 结论不变：collision 内核身份已变 ⇒ 后续 GPU 窗口必须新冻结、旧 guards 不复用。

## 3. 验证（`logs/verify-eol.txt`）

- `git check-attr text eol`：`.cu` → `set` / `lf`；证据文件 → `text: unset`；`sdf-validate-window` guards → `text: unset`。
- 全仓 tracked `.cu` = 849，`w/crlf = 0`。
- `MEGAKERNEL_SOURCE` = 88,865 B / `778216b2…`（与冻结口径一致）。
- fix/revert 的 8 个证据文件全部 `i/lf w/lf`；两个 manifest 与 committed blob 逐个对齐。
- `.gitattributes` diff 见 `materials/attributes-diff.patch`。

## 4. 边界

只改文档文字、`.gitattributes`、manifest 数字与 A1 证据的身份口径；不改产品代码/判据/容差/参考/fixtures；
不改写历史结论与提交历史；零 GPU、零窗口、零冻结。A1 冻结规划等本卡收口后按 LF 值重新定。
