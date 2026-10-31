# 参考冲突 RCA：scene-a（S7b1）vs scene-sdf（U3）两份精确参考互不兼容

状态：**只读 RCA**（离线，零 GPU 窗口、零冻结、未改仓）。日期 2026-10-31。
结论分级：**「两份参考互不兼容」= 机器证据支撑的定论；「成因 = 同一生成表达式被编译进不同形态 kernel 导致收缩不同」= 机器证据支持但未逐 site 定位到唯一指令点，故不作为唯一成因断言。**

## 1. 两份参考的 recipe/环境身份（都是上游 GPU 采集）

| 项 | scene-a（S7b1-kernel 参考） | scene-sdf（U3 采集） |
| --- | --- | --- |
| 文件 | `fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json` | `fixtures/render-bvh/scene-sdf/capture-sdf/sdf-mesh/{depth,seg,rgb}.npz` + `capture.json` |
| task / 日期 | `2026-10-25-render-bvh-s7b1-kernel`，2026-10-25T03:29:25Z | `2026-10-30-render-sdf-capture`（U3 窗口） |
| upstream revision | `71da24d956378a87a703b6e1442b13aec0c4ac29` | 同 |
| warp / mujoco / mujoco_warp / numpy | 1.15.0 / 3.15.0 / 3.15.0 / 2.5.4 | 同 |
| GPU / driver | RTX 4070 Ti SUPER / 596.36 | 同（`arch: 89`, cuda_toolkit 12.9, driver 13.2） |
| python | `target/mjwarp-parallel/venv/Scripts/python.exe` | 同（含 exe sha256 `feeec361…`） |
| `use_fast_math` | **false**（参考文件的 `recipe.compile_fast_math=false`） | **false**（`capture.json.use_fast_math=false`；`use_shadows=true`） |
| 采集脚本 | sha256 `29ae04aced55afa38192b9dc36e5ed7aa6501310c0b5f1324be5154de4b0bdb6` | sha256 `8064596876aeec3ee2fe1f63e83d7de87f898a8dbe9a1ef64f64026de8a77ba9`（`scripts/capture_render_sdf.py`） |
| warp kernel cache | `target/render-bvh/s7b1-fastmath-cache` | `target/render-bvh/sdf-capture-cache` |
| 生成 megakernel（缓存目录名） | `…render_megakernel_92d94e46_dde26ad` | `…render_megakernel_9a36ca11_73e6b6c`（另有 mixed 场景 `d73b0b4b_dc292bb`） |
| 生成 `.cu` | 426,960 B `7b8ccc82afe5b984787d7f178f2b32e614cf88ba9052920c88568fabda5cdebd` | 337,080 B `cb5281c7246f8f9b7b6afa99fa68b713463b6f2e6e9ba1a03ddfa4f24c204bb6` |
| `.sm89.ptx` | **637,336 B `ad21b93f94bbc6f42544e436104384d299d2130a78fc59b3dd24608876ca0988`**（= 参考文件 `kernel_cache.ptx_sha256`，可确认就是该参考的 PTX） | 428,462 B `6f57d91519dcfe94f2ef6a4ca3114028b921ba89d95a32e891c9caf57d2f8b91` |

⇒ **两份参考同 upstream、同 warp/mujoco 版本、同 GPU、同 `use_fast_math=false`**；差异只在采集脚本与场景/上下文。

## 2. 机器证据：生成源码相同、PTX 不同

用缓存里的 warp 产物直接比（脚本 `target/rca/ptx_sphere_compare.py`，输出 `target/rca/ptx-sphere-compare.txt`）：

- 球面二次式所在函数 `_ray_quad_0` 的**生成 CUDA 源码在两份采集里逐字节相同**
  （len 3,451，sha256 `c4a78a35e2629176…` / `c4a78a35e2629176…`）。
- `ray_sphere_0` 生成源码 len 2,625 相同，**唯一差异是 warp 源码映射注释行号**
  （场景 A：`ray.py:332`；scene-sdf：`ray.py:424`）⇒ 说明这是**不同调用点/不同 inlining 上下文的特化副本**，
  不是表达式本身不同。
- 两份 `.sm89.ptx` **不同**（637,336 B vs 428,462 B，sha 不同），即同一份生成 C++ 文本被
  编译进了不同形态的 kernel，SASS/PTX 层面的乘法/FMA 收缩不一样。

PTX 中 `det < MJ_MINVAL`（`0f26901D7D` = f32 1e-15）site 的一例（scene-a，行 951 附近；该 site 的二次式序列）：

```ptx
mul.f32     %f4322, %f8, %f4320;          // b: 先乘 1 号分量
fma.rn.f32  %f4323, %f7, %f4319, %f4322;  // 再 fma 0 号
fma.rn.f32  %f90,   %f9, %f4321, %f4323;  // 再 fma 2 号  -> b = (1,0,2)
mul.f32     %f4324, %f4320, %f4320;       // dd: 1 号先乘
fma.rn.f32  %f4325, %f4319, %f4319, %f4324;
fma.rn.f32  %f4326, %f4321, %f4321, %f4325;
sub.f32     %f4327, %f4326, %f4318;
mul.f32     %f4328, %f90, %f90;
mul.f32     %f4329, %f14, %f4327;
sub.f32     %f91,   %f4328, %f4329;       // det = b*b - a*c（该 site 未融合）
```

说明：PTX 在每份 kernel 里有数十个 `det<1e-15` site（不同的 primitive/参数特化），逐 site 与
`ray_sphere` 的唯一对应需要更深的 SSA 归属，本轮**未**把「哪个 site 就是球的 site」当作定论；
**收缩次序要求 (0,1,2)/(1,0,2) 的结论来自逐像素位级复现表（§3），那是硬证据。**

## 3. 逐像素位级复现表（同一产品实现无法同时满足）

| 数据 | 产品 `a=(0,1,2)`（gen1/gen3） | 产品 `a=(1,0,2)`（fix gen2） |
| --- | --- | --- |
| **scene-a**（S7b1 精确参考，120,000 像素；R05/R08/R09/S7b2 实体） | **逐位相等 120,000/120,000**（attempt1/2 的 8/9 回归全绿） | cam0 0 差异；**cam1 183 像素不符、max_ulp=73**（attempt3 6 个回归实体，首差 pixel 28969 / (169,120)） |
| **scene-sdf**（U3 精确参考，2 相机） | cam0 215/76,800、max_ulp=191；cam1 4/43,200、max_ulp=25（attempt1/2） | 诊断 44 个样本像素 **44/44 位级复现**（fix 片 RCA，`target/sphere-depth-rca.md`） |

⇒ 若把产品固定为 (0,1,2)：scene-a 位级全绿、scene-sdf 球轮廓超判据（191 ULP）。
⇒ 若固定为 (1,0,2)：scene-sdf 球轮廓位级、scene-a 球轮廓 73 ULP（>4）⇒ 6 个回归实体红。
**两份参考互不兼容**：不存在一个实现能同时位级匹配；73 ULP 也远大于任何 ≤4 ULP 判据。

## 4. 成因判断（分级）

- **已定论（机器证据）**：两份参考的生成 CUDA 源码中球二次式文本相同，但编译出的 PTX 不同，
  且两份 PTX 的 sha/bytes 与各自参考的 `kernel_cache` 记录一一对应 ⇒ **「同一表达式在两份采集里
  被编译成不同收缩」成立**，不是产品实现里某个算术写错。
- **支持但未定位到唯一指令点**：收缩差异发生在 nvrtc/ptxas 侧，由「不同的周围代码/调用点特化」驱动
  （`ray_sphere_0` 的源码映射注释分别指向 `ray.py:332` / `:424`，且两份 kernel 的 .cu 体积/形态不同）。
  要把某个具体 SASS 指令归属到球 site，需要逐 site 的 SSA 对应，本轮未做，故不写成唯一成因。
- 因此：**产品侧无法"按场景控制生成形态"**——产品只写一份 CUDA；决定收缩的是上游为每个场景生成的
  kernel 形态与编译上下文。上游自己跨场景也不保证同一收缩。

## 5. 候选处置（只出候选，不自行落地；均需用户签）

1. **改判据（推荐优先评估）**：U4 的 depth 判据在**球面轮廓像素**单列口径（例如：轮廓像素 = 参考 seg
   指向球且 |depth ULP| 落在掠射区间的像素，单列 ≤73 ULP 或排除位级要求；其余像素保持位级 + ≤4 ULP）。
   需要：新判据版本号 + 逐条受影响像素清单（可从 scene-sdf/attempt1/2 差异 + 几何掠射判据生成）+ 重新签。
2. **重采参考（改参考，需用户签）**：用同一上游环境重采 scene-sdf 参考——但**不能控制**其生成的 kernel 形态
   与 scene-a 相同（场景不同 ⇒ 特化不同），重采只会得到"另一份同样互不兼容"的参考；除非同时重采 scene-a
   并按"单场景内自洽"作为判据（等价于把判据从"位级对齐上游"退为"位级对齐本场景已签参考"）。
3. **维持 U4 未通过、SDF 不发布**：产品保守停留在 gen1（与全部历史参考位级一致），U4 球轮廓判据保持未通过。
4. 任何方案都**不改 ≤4 ULP 的既有判据、不改历史窗口证据**；U1b 重做继续暂缓（SDF 能力维持不可达）。

## 6. 复现材料（全部在 target/，不入库）

- `target/rca/ptx_sphere_compare.py` + `target/rca/ptx-sphere-compare.txt`（生成源码/PTX 身份与片段）
- `target/rca/ptx-det-context.txt`（两份 PTX 的 `det<1e-15` site 上下文）
- `target/rca/sphere_depth_rca*.py`（scene-sdf 44 像素 44/44 与 432 变体扫描）
- attempt3 证据：`docs/development-evidence/render-bvh/sdf-validate-window/attempts/attempt3/`（6 个回归实体
  的失败位置/max_ulp=73）与 `target/render-bvh/sdf-validate-window/attempt3/`（原始）

## 1b. 参考文件/采集清单的字节身份（机器计算，补 §1）

| 文件 | bytes | sha256 |
| --- | --- | --- |
| `fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json`（scene-a 精确参考本体） | 1,492,762 | `d6ab6dde58034df5c1bc69ab397026a8f7c3aaf58d29d25edb032d3818000dc7` |
| `fixtures/render-bvh/scene-sdf/capture-sdf/sdf-mesh/capture.json`（U3 采集清单） | 10,547 | `d45c9ef9560b9e4f18e01a92a46a513c3a50b419d3203e7d378abfc1c108e517` |
| `…/sdf-mesh/depth.npz` | 480,486 | `3415b796748091df64ca45845c305a43f9bba63ce0fc6962bd49d75b22dc7ee3` |
| `…/sdf-mesh/rgb.npz` | 1,440,478 | `7ca2abd663cd40bc8cbdda35f601f981256093519d7bbf02ba8dbd447a0ed3ad` |
| `…/sdf-mesh/seg.npz` | 960,478 | `afd5eacd771a16292cc9f39ec97be1276f3cd1abb4ab99dd6b1728de01c941e5` |

（scene-a 参考本体内 `source.s0b_depth_npz_sha256` = `0fa55821…`、`source.capture_script_sha256` = `29ae04ac…`、
`environment` 见 §1；其 `kernel_cache.ptx_sha256` = `ad21b93f…`/637,336 B，与本地缓存的 PTX 逐位一致。）