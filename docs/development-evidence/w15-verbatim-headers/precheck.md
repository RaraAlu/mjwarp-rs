# W15 头文件原样复制 · 设计预检

任务：`2026-10-23-w15-verbatim-headers`（proposed）。
依据：`docs/development-evidence/layers-w15.md`、`docs/unified-launcher.md`、`docs/launcher-cache.md`。
本轮只做设计预检与范围重钉，不改产品代码、不复制头文件；实现另轮。

## 现状

- `src/runtime/launcher/artifact/nvrtc.rs` 对非空 `NvrtcSource.headers` 直接返回
  `LauncherError::Unsupported("内存头文件编译待W15")`。
- `NvrtcSource` 已有 `HeaderFile { name, content }` 与 `with_header`；`ArtifactKey` 已覆盖
  源码摘要、头文件逐项 sha256、配方、架构与 NVRTC 版本（`src/runtime/launcher/artifact.rs`）。
- cudarc 0.18.2 `CompileOptions` 提供 `include_paths: Vec<String>`（转成 `--include-path=`），
  没有内存头文件 API；因此编译期必须落到磁盘目录再传 include path。

## 复制来源与清单

- 来源：`target/research/warp`，tag `v1.15.0`，commit `ffa038b1adf3927a8e893d4536f1a7562f17d749`（已核对 clean）。
- 许可：Warp `LICENSE.md` 为 Apache-2.0，与仓内 `LICENSES/Apache-2.0.txt` 同源。
- 从 `builtin.h` + `deterministic.h` 递推 quoted include，得到 **36 个头文件**（含
  `nanovdb/PNanoVDB.h` 子目录），逐文件 bytes 与 SHA-256 见 `headers.json`。
  - 必需级与条件级（`tile*.h`、`volume.h`、`texture.h`、`noise.h`、`hashgrid.h`、`matnn.h` 等）
    被 `builtin.h` 无条件包含，因此实际复制集合以递推闭包为准。
  - 排除：`warp.h`、`aot.h`、`exports.h`、宿主 `*.cpp`（宿主 API/导出，不参与内核编译）。
- **重要发现**：wheel 安装的 warp 1.15.0 头文件与 clone tag 字节不同（示例 `builtin.h`
  clone 94374B / wheel 91555B，SHA 全不同）。W15 以 `layers-w15.md` 冻结的 clone tag 为唯一
  复制来源，**不混用 wheel 头文件**；冻结产物此前由 wheel 工具链生成，不代表 W15 复制来源。
- 外部头：`<cuda_fp16.h>`、`<cuda_runtime.h>`（另含 `vector_types.h` 等 SDK 传递依赖）来自
  CUDA SDK；标准库头由 NVRTC 内建提供。SDK 头不复制进仓，固定
  `target/toolchains/cuda-12.8.1/include` 并在产品侧按调用方传入 include path + 记录摘要
  （许可与再分发另行评审）。

## 实现方案（下一轮执行）

1. **目录**：`include/warp-headers/` 按来源相对路径逐字节复制 36 个头文件（只读，不许就地改）；
   `LICENSES/warp-1.15.0/LICENSE.md` 保存上游许可原文；`THIRD_PARTY_NOTICES.md` 增来源声明
   （项目、tag/commit、路径、逐文件摘要清单引用）。
2. **`NvrtcSource`**：
   - `HeaderFile` 名称校验放宽为允许相对子路径（`nanovdb/PNanoVDB.h`），仍拒绝绝对路径、
     `..`、反斜杠、NUL 与重名；`with_headers_from_dir(dir)` 读取 `*.h` 递归闭包并逐项入键。
   - 编译时把内存头文件按摘要安全落盘到唯一临时 include 目录（原子写、校验后使用、编译后清理），
     `CompileOptions.include_paths` 加该目录与调用方 external include paths；
     去掉「待W15」拒绝，补拒绝面测试（非法名字/重复/路径穿越/缺文件/摘要不符）。
   - `ArtifactKey` 扩展 `include_paths` 与外部 CUDA 头摘要（至少 `cuda_fp16.h`），保证 SDK 头漂移换键。
3. **测试（机器可判定）**：
   - 头文件清单校验：复制的 36 文件逐字节等于 `headers.json`（sha256/bytes），证明原 header 未被改写。
   - 许可落仓：`LICENSES/warp-1.15.0/LICENSE.md`、`THIRD_PARTY_NOTICES.md` 存在且含来源 commit。
   - 宿主单测：头文件名字校验、子路径、重复、临时目录落盘/失败回滚。
   - 定向 NVRTC/GPU（ignored，环境变量给 CUDA include 与证据目录）：编译一个
     `#include "builtin.h"` + `#include "deterministic.h"` 的最小内核，NVRTC 编译成功、
     PTX 非空、产物键随头文件内容变化。
4. **规范**：复制头文件会触发「新文件 ≤500 行」提交限制，需在 `.normify/config.toml`
   `[size].exclude` 增加 `include/warp-headers`，理由：上游字节原样复制，不可重排/拆分。
   该豁免仅覆盖复制目录，`src/` 新文件仍受 500 行限制。

## 边界

- 本任务不切产品内核：上游源码产物不得进入物理配方，未完成逐字段精度映射前只做编译能力与证据。
- 不改 `src/physics`、fixtures、容差、断言；不改冻结 PTX/手写移植两条既有来源。
- `cuda_fp16.h` 等 SDK 头不复制、不引入 Python 链；SDK 许可/再分发边界未做法律审阅，如实登记。
- 未跑 GPU 的项在证据中标记未执行；NVRTC 测试需 GPU 窗口，先报 QA。
