# G03 上游矩阵冻结参考：生成、天然校验与首差定位

任务：`2026-10-22-g03-upstream-matrix-authoring`（用户 2026-10-22 批准离线 authoring 例外）
日期：2026-10-22
源码基线：`02d2bcd`
结论：**上游 matrix 冻结参考已生成，天然校验 0/121 通过；首差定位到 kinematics 的 `xmat`（quat→matrix 的 FMA 收缩差异）。本任务只到计划，不实现。**

## 一、pinned 环境审计（`env-audit.json`）

| 项 | 结果 |
| --- | --- |
| venv | `target/mjwarp-parallel/venv`；`include-system-site-packages=false`（隔离） |
| Python | 3.12.13（uv 0.11.3 管理） |
| 包版本 | mujoco 3.15.0、warp-lang 1.15.0、numpy 2.5.4、mujoco-warp 3.15.0 |
| 上游源码 | 已安装 `mujoco_warp` 与 `target/mjwarp-parallel/upstream` **75/75 .py 文件 SHA256 一致** |
| 上游提交 | `71da24d956378a87a703b6e1442b13aec0c4ac29`，工作区 **clean** |
| warp.dll | pinned wheel RECORD 校验通过（sha256 `7c006eb3…671d6`） |
| authoring 缓存 | `target/g03-block-reference` 与冻结 manifest **35/35 cache_hashes 一致** |
| GPU/驱动 | RTX 4070 Ti SUPER / 596.36（Warp 运行时报告 CUDA Toolkit 12.9、Driver API 13.2） |

包装脚本 `scripts/authoring/author-g03-upstream-matrix.ps1` 在生成前逐项复核以上内容，
任一不一致直接拒绝运行（不降级放行）；本目录只有离线工具，产品构建/测试/cargo 门禁不引用它。

## 二、生成（一条命令）

```powershell
pwsh -NoProfile -File scripts/authoring/author-g03-upstream-matrix.ps1
```

- 方式与 `fixtures/mjwarp-mass-solve/reference.py` 相同：mixed-joints XML + `fixtures/mass-matrix`
  模型字段 + `fixtures/mass-solve` case4 qpos，8 世界，`kinematics → com_pos → crb`，
  然后按 `io.py` 的 gather tile `block_elemid` 采出 case4/world4 的 11 维稠密块
  （结构缺失项 `nC` 按 MathDx 的越界读 0 处理）；同时导出 `cdof/cinert/crb/subtree_com` 等中间量。
- 同一进程跑两次并逐字节断言一致：**PASS（deterministic_runs=2）**。
- 运行环境由脚本 `-B`、无网络；Python 只用于离线 authoring。

产物（`docs/development-evidence/g03-upstream-matrix/`，均**不进产品 fixtures**）：

| 产物 | 大小 | SHA256 |
| --- | ---: | --- |
| `matrix-world4.f32`（121 个 f32） | 484 B | `1361069ca91c19a42b3046c7170020ab17d1371323e9346643a205a90c997ef3` |
| `dense-blocks.f32`（8 世界 ×121） | 3872 B | `675b23178cd4a3de93bb254b437df6dca980dab20eafaa5262b70b1121a6f390` |
| `upstream-matrix.json`（矩阵+中间量+provenance） | 92029 B | `0d83b329678206f7bf8ae8b434eb02bfd5e41efa713c5ef1799c50008c2c679e` |
| `provenance.json` | 2616 B | `8d2af4fba38586f6eb375ec9a240c48648257168bc91a424e00c483616b3ccf1` |
| `env-audit.json` | 1279 B | `158687c08f501aead23662e3f7a4ac62ca9c0b3e604f845d41f9959f93787e3c` |

## 三、天然校验（硬门槛）：0/121 通过

```powershell
pwsh -NoProfile -File scripts/authoring/validate-g03-upstream-matrix.ps1
```

- 用生成的 11 维矩阵喂第一阶段已离线链接的 MathDx `potrf_11_11_1_89_32_1_1_5_x_x_0`
  （CUBIN sha256 `36e7f49e…2c765`，与第一阶段证据同哈希），与冻结上游 factor 逐位比较：
  **status=0、bit_differences=0/121、max_absolute=0、passed=true**。
- 结论：生成的矩阵就是当初产出冻结上游 factor 的那份上游矩阵（在 MathDx potrf 输入意义上逐位等价），
  可信度成立。结果见 `validation.json`（sha256 `929527aa…a2481`），oracle 原始输出 `potrf-world4.out`。

## 四、首差定位（对照上游中间量 vs 产品探针，8 世界）

工具：`tools/first_difference.rs`（离线、只读产品探针与冻结 evidence；结果 `first-difference.json`）

| 阶段 | 字段 | 比较数 | 位差数 | 首个差异 |
| --- | --- | ---: | ---: | --- |
| kinematics | xpos | 144 | 0 | — |
| kinematics | xquat | 192 | 0 | — |
| kinematics | **xmat** | 432 | **17** | world 3 / index 15（body1, entry6） |
| kinematics | xipos | 144 | 0 | — |
| kinematics | ximat | 432 | 0 | — |
| kinematics | xanchor | 96 | 0 | — |
| kinematics | xaxis | 96 | 0 | — |
| com_position | subtree_com | 144 | 0 | — |
| com_position | **cdof** | 528 | **18** | world 3 / index 20 |
| com_position | cinert | 480 | 0 | — |
| mass_matrix | crb | 480 | 0 | — |
| mass_matrix | **matrix_dense_block** | 968 | **132** | world 3 / index 3 |

**疑似阶段排序：kinematics.xmat → com_position.cdof → mass_matrix.matrix**。
`xpos/xquat/xipos/ximat/xanchor/xaxis/subtree_com/cinert/crb` 已逐位一致，说明这些公式已对齐；
差异从 `xmat` 开始，`cdof`/`matrix` 只是继承传播。

首个差异的根因证据（world3 body1，`xmat[6] = 2*(x*z - w*y)`）：

- 上游冻结值 bits = `0x3f0077f3`（0.5018302798271179）= **非收缩** `2*((x*z) - (w*y))`；
- 产品值 bits = `0x3f0077f4`（0.5018303394317627）= `2*fma(x, z, -(w*y))`；
- 上游 `math.quat_to_mat` 先把 `q00..q33` 乘积算成局部量，再做加减（无穷积-加相邻，不被 FMA 收缩）；
  产品 `KINEMATICS_CUDA` 的 `matrix()` 直接写内联表达式，允许编译器把 `x*z - w*y` 收缩成 FMA。

## 五、第二阶段（矩阵链对齐）计划：先报，不实现

1. **首差修复候选（最小改动面）**：把产品 `src/physics/mod.rs` 的 `KINEMATICS_CUDA` 中
   `matrix(Q q)` 改为与上游 `math.quat_to_mat` 相同的写法——先把 `q00,q01,q02,q03,q11,q12,q13,q22,q23,q33`
   算成局部 float，再组装 9 项；不新增 ABI、不改数据类型、不改 cuDNN/内核字符串的其它部分。
2. **验证顺序**：重跑 `tools/first_difference.rs` → 期望 `xmat` 0 差异；随后 `cdof`、`matrix` 归零；
   然后 `MathDx potrf(产品矩阵)` 应等于冻结上游 factor（0/121，与天然校验同一判据）；
   再接 `MathDx potrs(上游 factor, rhs)` = 冻结 solution 的既有 oracle 链路。
3. **回归面**：kinematics/G01 定向 GPU 测试 + `mass_solve_probe` 3 项准入 + 双模式门禁；
   产品源码变化会漂移 T4 源码身份 → 需登记并完成源码身份重钉（另立任务）。
4. **风险**：其余表达式（如 `xmat[0/4/8]` 的加和）也可能有收缩差异，需按首差循环归零；
   若 `xmat` 归零后 `cdof` 仍有差异，再单独定位 `_cdof` 的乘加顺序；不保证一轮完成。
5. **边界**：本任务只做证据与计划——不改容差、不改准入余量、不改测试断言、不改产品 fixtures；
   离线 authoring 工具与产物都不进入产品链；若后续要把矩阵参考升格进 `fixtures/`，另立任务评审。

## 六、复现与工具位置

- 生成/校验脚本：`scripts/authoring/author-g03-upstream-matrix.ps1`、
  `scripts/authoring/author-g03-upstream-matrix.py`、`scripts/authoring/validate-g03-upstream-matrix.ps1`
- 首差工具源码：`docs/development-evidence/g03-upstream-matrix/tools/first_difference.rs`，
  按 `docs/development-evidence/g03-parity-precheck/README.md` 的临时 cargo 包方式（path 依赖仓库根、
  `features=["cuda-probe"]`）编译运行；MathDx cubin/oracle 的复现见
  `docs/development-evidence/g03-mathdx-alignment/README.md`。
- 运行日志（本机、gitignored）：`target/authoring-g03.log`、`target/g03-upstream-matrix/`。

## 七、边界

- 离线 authoring，非产品链：产品构建/测试/cargo 门禁只跑 cargo，不引用本目录工具、不读取本目录产物；
- 未改 `src`、`tests`、`fixtures`、`native`、`include`、`Cargo.*`（unchanged 检查）；
- 未改容差/准入余量/测试断言；未重钉 T4；未跑 release/Linux；
- 不宣称 G03、精度契约或 NP 验收通过。
