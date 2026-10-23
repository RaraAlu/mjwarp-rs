# G01 rustdoc 私有项断链修复

任务：`2026-10-23-g01-rustdoc-private-link-fix`
发现：QA 在 `2026-10-23-g01-repin-final-acceptance` 的 debug/release 终验第 13 阶段 rustdoc 失败（两模式同一处），归因批次 2 G07 交付 `12151dc`。
基线：`0ab7841`（任务登记提交 `daed736`）
范围：一行文档文字；不改代码语义、断言、容差、排除清单、测试、门禁脚本。

## 根因

`BroadphaseModel::full_pairid` 的公开文档用方括号链接指向私有项：
`[`upper_tri_index`]` → `pub(crate) fn upper_tri_index`。
rustdoc 的 `private-intra-doc-links` lint 在 `RUSTDOCFLAGS=-D warnings` 下报错。

| 文件:行 | 修复前 | 修复后 |
| --- | --- | --- |
| `src/physics/collision.rs:124` | `` 见 [`upper_tri_index`]）`` | `` 见 `upper_tri_index`）`` |

修复前实测错误原文见 `pre-fix-error.txt`（退出码 101）。

## 验证

| 命令 | 修复前 | 修复后 |
| --- | --- | --- |
| `RUSTDOCFLAGS=-D warnings cargo +1.99.0-x86_64-pc-windows-msvc doc --locked --all-features --no-deps` | 退出 101，1 error（`private-intra-doc-links`，`src/physics/collision.rs:124:47`） | 退出 0，0 警告（`post-fix-rustdoc-all-features.log`） |
| 同口径 `--features native-model-probe,cuda-probe --no-deps` | 未单独跑（同一源码路径、同一 lint） | 退出 0，0 警告（`post-fix-rustdoc-features.log`） |
| 门禁 fmt/clippy/clippy-cuda/unit | — | 见 `.normify/runs/2026-10-23-g01-rustdoc-private-link-fix/` |

环境接线：`MJWARP_MUJOCO_ROOT=target\toolchains\mujoco-3.12.0\package`、`MJWARP_MUJOCO_DLL=...\bin\mujoco.dll`（仅 all-features 构建需要）；`RUSTDOCFLAGS=-D warnings`。

## 边界

- 只改一行文档文字（方括号链接改反引号），未改逻辑、签名、断言、容差、排除清单、测试或门禁脚本。
- 未新增 `allow` 豁免，不放宽任何 lint。
- 本任务只证明 rustdoc 断链已修；G01 debug/release 双模式是否全绿由 QA 在新 HEAD 复跑终验判定，不在此宣称。
- 修后 HEAD 变化：QA 原在 `0ab7841` 上启动的终验任务需按新 HEAD 重新登记/复跑。
- 平台边界：仅 Windows；release/Linux 本任务未跑。
