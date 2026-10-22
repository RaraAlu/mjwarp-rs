# W15 落地记录：Warp 头文件原样复制

任务：`2026-10-23-w15-verbatim-headers`。
路线依据：`docs/development-evidence/layers-w15.md`（上游源码与头文件原样复制）。
本轮落地编译前置能力，**不切换产品内核**。

## 复制资产

- 来源：`target/research/warp`，tag `v1.15.0`，提交 `ffa038b1adf3927a8e893d4536f1a7562f17d749`（clean）。
- 目录：`include/warp-headers/`，逐字节复制 **36 个头文件**（1,520,709 字节），含 `nanovdb/PNanoVDB.h` 子目录。
- 闭包根：`builtin.h` + `deterministic.h` 的 quoted include 递归闭包；排除 `warp.h`、`aot.h`、`exports.h` 与宿主 `*.cpp`。
- 逐文件 bytes/SHA-256：`headers.json`（本目录），也写入 `THIRD_PARTY_NOTICES.md` 的来源声明。
- 许可：`LICENSES/warp-1.15.0/LICENSE.md`（Warp `LICENSE.md` 原样复制，sha256 `6d1d968f…`，Apache-2.0）。
- **来源唯一性**：wheel 安装的 warp 1.15.0 头文件与 clone tag 字节不同（`headers.json.wheel_vs_clone`，2026-10-23 已补全 36/36 全量对比，36 个全部不同），
  W15 只用 clone tag，不混用 wheel 头文件。
- 外部 CUDA SDK 头不复制进仓：`<cuda_fp16.h>`、`<cuda_runtime.h>` 与 `vector_types.h` 等由
  `target/toolchains/cuda-12.8.1/include` 提供，调用方通过 include path 传入并登记摘要（见下）。

## 实现

- `src/runtime/launcher/artifact.rs`
  - `HeaderFile` 名称支持相对子路径（`nanovdb/PNanoVDB.h`），拒绝绝对路径、`..`、反斜杠、空段与 NUL。
  - `NvrtcSource` 新增 `with_headers_from_dir`（递归读取 `*.h`，按路径排序，逐项参与产物键）、
    `with_include_path`（外部 include 目录）、`with_external_header`（外部头文件 SHA-256 进键）。
  - `ArtifactKey` 增加 `include_paths` 与 `external_headers` 字段，`KEY_SCHEMA` 升为 2；
    源码摘要、头文件逐项摘要、配方、架构、NVRTC 版本、include 路径与 SDK 头摘要共同决定键。
- `src/runtime/launcher/artifact/nvrtc.rs`
  - 移除「内存头文件编译待W15」拒绝；编译前把内存头文件按相对路径落到唯一临时目录
    （`%TEMP%/mjwarp-launcher-headers/<pid>-<id>`），编译后清理，不修改仓内头文件。
  - `CompileOptions.include_paths` = 临时目录 + 调用方 include paths；
    头文件编译追加 `--device-as-default-execution-space`（Warp 头树含未标注执行空间的函数，NVRTC 必需）。
  - NVRTC 版本的 fmad/fast-math/std/arch 仍只取自配方，未放宽既有编译选项。
- `.normify/config.toml`：`[size].exclude` 增加 `include/warp-headers`（原样复制、不可重排/拆分）。

## 验证

| 项 | 命令 | 结果 |
| --- | --- | --- |
| 头文件未改写 | `cargo test --locked --features cuda-probe --test launcher-warp-headers copied_headers_match_the_frozen_manifest` | 通过：36 文件 bytes/SHA-256 与 manifest 全等，目录内无多余 `.h`；许可与来源声明存在 |
| 目录加载 | 同文件 `headers_load_from_the_copied_directory_with_relative_paths` | 通过：36 项、含 `nanovdb/PNanoVDB.h`、顺序稳定 |
| NVRTC 编译 | 先把 `target/toolchains/cuda-12.8.1/bin` 加入 `PATH`（NVRTC DLL），再 `$env:MJWARP_CUDA_INCLUDE=target/toolchains/cuda-12.8.1/include`，运行 `cargo test --locked --features cuda-probe --test launcher-warp-headers -- --ignored` | 通过：`builtin.h`+`deterministic.h` 最小内核编译出 PTX（含入口名），include 路径生效 |
| 产物键 | 同上测试内断言 | 通过：同源第二次编译命中缓存（compiles 不变）；追加头文件后键变化并重编译 |
| 宿主单测 | `cargo test --locked --features cuda-probe --lib runtime::launcher::artifact` | 14 过 0 败（含非法名字、路径穿越、重复、临时目录落盘、键覆盖 include paths/外部头） |
| 门禁 | `normify gate 2026-10-23-w15-verbatim-headers` | fmt/clippy/clippy-cuda/unit 见 `.normify/runs/` |

NVRTC 编译不启动 GPU 内核；本轮未跑 GPU 计算、未改物理路径。

## 边界（未验收）

- 产品内核来源未切换：上游源码产物在逐字段精度映射完成前不得进入物理配方（`layers-w15.md` 约束保持）。
- CUDA SDK 头不复制进仓；CUDA Toolkit 许可与再分发边界未做法律审阅，如实登记。
- 头文件树整树参与键（36 项），未做按需裁剪；`tile*.h` 属条件级但被 `builtin.h` 无条件包含，因此本轮全部复制。
- 未实现上游源码编译复现（需要具体内核与精度映射任务）；未跑 Linux；未做性能测量。
- 本任务不新增 Cargo 依赖，不引入 Python 链。
