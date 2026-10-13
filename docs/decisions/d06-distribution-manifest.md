# D06组件与分发清单

日期：2026-10-14。
任务：`2026-10-11-roadmap-d06`。
本决议只确定分发边界。
它不批准发布或替权利人授权。
当前正式发行仍不可放行。

## 决议与适用范围

本轮沿用原任务与架构。
本轮只写本决议文件。
本轮不新增依赖或修改产品。
本轮不改精度、容差与覆盖。
本轮不推送、发布或部署。

正式包默认不捆绑外部工具链。
调用方提供受支持驱动与运行库。
捆绑任何第三方组件须另行核准。
静态链接也须纳入组件清单。
动态加载不免除分发义务。
源代码分发也须保留适用通知。
文件名与安装成功不证明授权。
许可声明不等于相容性审查。
本清单记录事实与待办。
发布负责人须核准实际发行包。

本轮不替根包选择许可。
根包未声明`license`或`license-file`。
根目录未见LICENSE授权文件。
现有`LICENSES/Apache-2.0.txt`仅存文本。
它匹配冻结MJWarp许可文本。
它不自动授权本项目全部源码。
此项阻止正式产品分发。

## 实际依赖路线

依据见[包配置](../../Cargo.toml)、
[锁定清单](../../Cargo.lock)、
[原生构建](../../build.rs)、
[双路编译](../../src/runtime/cubecl.rs)
及[原生加载](../../native/model_probe.cpp)。

| 路线 | 实际组件 | 当前行为 | 分发决议 |
| --- | --- | --- | --- |
| 默认功能 | 本crate | 默认不启用可选依赖 | 根包授权先行 |
| CUDA探针 | cudarc 0.18.2、serde 1.0.229、serde_json 1.0.151、sha2 0.10.9 | cudarc使用driver、nvrtc、dynamic-loading及cuda-12000 | 核实际静态成分与运行库 |
| CubeCL C++ | core/cpp 0.11.0-pre.4 | 固定Git提交，C++转NVRTC/PTX | 保留CubeCL与继承通知 |
| CubeCL LLVM | core/llvm 0.11.0-pre.4 | Pliron生成NVPTX | 单列宿主LLVM与libdevice |
| 原生模型 | cc 1.6.0及自有C++桥 | Windows MSVC或Linux GNU；桥静态编入 | 区分桥与MuJoCo动态库 |
| 原生动态库 | MuJoCo 3.12.0 | Windows LoadLibraryExW；Linux dlopen | 默认外部供应；捆绑须带通知 |
| 开发诊断 | 原生compile工具、接触作者工具、CUPTI、nvdisasm | 只作离线输入或诊断 | 不进入产品默认包 |
| 历史参考 | MJWarp、Warp、NumPy历史环境 | 产品测试读取静态文件 | 不增加Python构建运行链 |

CubeCL提交固定为：
`1f73b9f63de50a17398c1d5278e2a5f11612c7e1`。
Pliron提交固定为：
`81155d96b8797d149de1d88219c431a68a75ef33`。
Pliron包版本为0.18.0。
`llvm-sys`版本为231.0.0。
宿主打包器版本为23.1.0-3。
其源码声明LLVM 23.1.0。
本机存在该版本缓存目录。
本轮未核全宿主LLVM二进制身份。
本轮也未取得其完整许可包。
打包器的MIT/Apache声明不覆盖LLVM。
此项继续阻止LLVM路线发行。

## 原生与SDK组件

| 组件 | 已核来源与版本 | 已核许可证据 | 当前分发状态 |
| --- | --- | --- | --- |
| MuJoCo Windows | 官方3.12.0 ZIP；摘要见下表 | 原包LICENSE与THIRD_PARTY_NOTICES.txt | 不默认捆绑；必须保留两者 |
| MuJoCo Linux | prepare-linux-native.sh固定3.12.0与归档摘要 | 本机仅核脚本；本轮未重取Linux包 | 不从Windows推断Linux包已核 |
| CUDA NVRTC | 12.8.93；官方redist | 包内EULA及12.8.1附件A | 默认外部供应；逐文件核准 |
| CUDA cudart | 12.8.90；工具链已下载 | 同版EULA | 下载不证明产品实际链接 |
| CUDA CCCL | 12.8.90；编译头文件 | 包EULA之外还须核嵌入文件通知 | 尚无完整发行通知包 |
| CUDA nvcc | 12.8.93；开发编译工具 | 同版EULA | 不进入默认产品包 |
| NVVM/libdevice | 本地nvvm目录；libdevice.10.bc | EULA附件A列NVVM与libdevice | 名称不证明实际版本或授权完成 |
| CUPTI | 12.8.90；既有机器码诊断 | EULA附件A列CUPTI动态库 | 仅开发诊断；不整包携带 |
| nvdisasm | 12.8.90；既有反汇编诊断 | 包内EULA；附件A未列该工具名 | 不进入默认产品包 |
| NVIDIA驱动 | 用户安装的NVIDIA驱动 | 不等同NVRTC许可或CUDA工具包 | 不复制驱动或系统DLL |
| MSVC/Windows运行库 | 原生桥采用动态CRT；本机MSVC 14.44.35207 | 本轮未冻结运行库再分发许可及清单 | 默认由用户官方安装 |
| Linux C/C++运行库 | GNU路线及dl | 本轮未核实际发行依赖闭包 | Linux发行仍须实包核查 |
| LLVM宿主库 | tracel-llvm 23.1.0-3缓存 | 缺完整归档许可与通知证据 | 阻止该路线发行 |

MuJoCo依据见[官方3.12.0许可](https://github.com/google-deepmind/mujoco/blob/3.12.0/LICENSE)。
包内第三方通知另列多种组件。
不能只携带一份Apache文本。
本轮不分发simulate或模型资产包。

CUDA依据见[12.8.1许可](https://docs.nvidia.com/cuda/archive/12.8.1/eula/index.html)。
该协议仅列部分可分发组件。
其第1.1.2节仍要求附加条件。
附件A列NVRTC及builtins动态库。
实际Windows文件含版本后缀。
本机为`nvrtc64_120_0.dll`
与`nvrtc-builtins64_128.dll`。
附件A也列CUPTI、NVVM与libdevice。
这些条目不批准整个工具链打包。
本决议不代替逐文件合规审查。

| 已核文件 | SHA-256原始字节 |
| --- | --- |
| Cargo.toml | `d5f8857874b1196d8c26446e3281b60f15c1aa5ebaf26ecf5033091386258dcf` |
| Cargo.lock | `72af69f9b0816674118e8ca59e85f101aaebe9021a28fd5e9e9aae102ec25d90` |
| LICENSES/Apache-2.0.txt及冻结MJWarp LICENSE | `49bbe9114e49214df2ccc324cb3ac8d1d1aa1c3a0947f94c286765e86647b32e` |
| MuJoCo Windows官方ZIP | `ffe071c2747dd9513a1c59e7d2428bb678d887f9edec9eb9674b4288a248a8e9` |
| MuJoCo LICENSE | `3ddf9be5c28fe27dad143a5dc76eea25222ad1dd68934a047064e56ed2fa40c5` |
| MuJoCo THIRD_PARTY_NOTICES.txt | `b298d7577719dd1cefd5d666ea28e43f0406bd5502c7d802108012a31280bd90` |
| CUDA components.json | `2eeda3efe7d911eaf5da6667858397a71d109a3e86687e8ea4db95e92a205c4f` |
| 本机六份CUDA组件LICENSE | `e2c71babfd18a8e69542dd7e9ca018f9caa438094001a58e6bc4d8c999bf0d07` |

六份CUDA文本来自nvrtc、cudart、
cccl、nvcc、cupti与nvdisasm目录。
摘要相同不代表各文件授权相同。
Linux原生归档的脚本固定摘要为：
`a9367911e6d5eaeade17c2197304687421c1fc932cdf7bcd4cb8cfaf0374dcb2`。
本轮不把脚本常量当实际重载结果。

## 静态参考与资产

冻结MJWarp提交为：
`71da24d956378a87a703b6e1442b13aec0c4ac29`。
仓库许可声明为Apache-2.0。
它不授权任意第三方模型。
历史参考来源声明MuJoCo 3.15.0、
Warp 1.15.0与NumPy 2.5.4。
这不升级原生输入ABI 3.12.0。
这些版本来自历史来源清单。
本轮未执行其Python环境。

依据见[质量参考清单](../../fixtures/mjwarp-mass-solve/manifest.json)
与[绕行来源](../../fixtures/mjwarp-geom-tendon/provenance.json)。
绕行来源记录各包许可声明。
本轮未逐项核其历史wheel通知。
Warp生成产物须保留来源与通知。
本轮不把数值哈希当资产授权。

本轮核Git跟踪夹具共200文件。
下表包含源码、记录与数据。
它不是200个独立模型。
所有组默认仅作开发与测试材料。
正式发行须另核来源及权利链。
本轮不新增资产授权声明。

| 夹具组 | 跟踪文件数 | 发行门禁 |
| --- | ---: | --- |
| attached-kinematics | 14 | 原生参考与作者来源 |
| camlight | 6 | 原生参考与作者来源 |
| collision-reference-input | 10 | 自有recipe、原生转换与诊断；根包授权仍缺 |
| com-position | 10 | 原生参考与作者来源 |
| com-velocity | 8 | 原生参考与作者来源 |
| fixed-tendon | 6 | 原生参考与作者来源 |
| flex-edge | 4 | 原生参考与作者来源 |
| flex-face | 4 | 原生与独立诊断来源 |
| flex-position | 6 | 原生参考与作者来源 |
| flex-stretch | 6 | 原生与独立诊断来源 |
| g01 | 6 | 完整链参考不等于资产授权 |
| geom-tendon | 9 | 原生参考与作者来源 |
| kinematics | 9 | 原生参考与作者来源 |
| mass-matrix | 11 | 原生参考与作者来源 |
| mass-solve | 9 | 原生参考与作者来源 |
| mixed-tendon | 6 | 原生参考与作者来源 |
| mjwarp-geom-tendon | 6 | 上游来源、生成通知与历史依赖 |
| mjwarp-mass-solve | 13 | 上游来源、生成通知与历史依赖 |
| mocap-kinematics | 6 | 原生参考与作者来源 |
| native-probe | 13 | 模型、结构与原生转换来源 |
| precision | 20 | 源码派生、产物与来源通知 |
| rne-bias | 8 | 原生参考与作者来源 |
| spatial-tendon | 6 | 原生参考与作者来源 |
| tendon-wake | 4 | 原生参考与作者来源 |

外部mesh、纹理与字体须另建明细。
本轮不拷贝工具包model或assets。
XML解析未见外部file属性。
该检索不证明未来模型授权齐全。
调用方输入模型不等于项目可再分发。

## 发布硬门禁

1. 权利人明确根包许可与范围。
2. 核发行目标、功能与实际链接闭包。
3. 汇集第三方原文及适用通知。
4. 逐项处理文件级及组合许可。
5. 核LLVM原生包与运行库许可。
6. 核捆绑SDK文件与协议条件。
7. 核模型、纹理与生成资产来源。
8. 产出实际发行归档及逐文件摘要。
9. 通过P11安全、ABI与安装门禁。
10. 保留NP01至NP04全部要求。

元数据含两项MPL-2.0声明。
它们是buildid与option-ext。
本轮不把它们改写成MIT。
部分声明含AND及LLVM例外。
unicode-ident还声明Unicode许可。
webpki-roots另含数据许可声明。
本轮保留原始表达式。
旧式斜杠表达式也保持原样。
不能把所有许可简单归成双许可。
发行前须核目标可达性与义务。
本轮不判断这些声明已全部相容。

## 本轮核验与剩余证据

首次离线元数据查询失败。
Cargo缺critical-section 1.2.0缓存。
第二次联网查询成功。
Cargo仅补取既有锁定包。
本轮未新增依赖或改锁文件。
本轮成功后没有再次查询。
两次属于元数据读取，不是物理测试。

成功命令：

```powershell
cargo metadata --locked --all-features --format-version 1
```

完整结果包含266包。
其中根包1项、注册表250项、
Git包15项。
该结果包含非目标平台依赖。
它不是实际发行二进制闭包。
发行时须按目标与feature裁剪后核验。
本清单不宣称每包进入产品。

原始元数据文件为：
`target/d06-distribution/cargo-metadata.json`。
其SHA-256为：
`cd1e30a5ba0898b4a2dca7a2b53953c930621ca4dac5520b8763d6e7f86f3767`。
下载日志SHA-256为：
`f952c207342845668d31ef6bc353bed569325798b5d91b3f0ff7aa3659db3085`。

本轮核对包声明及许可文件摘要。
本轮未逐字法律审阅全部许可。
11项第三方包未检出顶层许可文件。
下表以“未检出”标识这些项。
检索仅覆盖包顶层与Git仓库根。
它不证明源码中没有许可说明。
这些项继续阻止完整通知包放行。

NP01：本轮不改任何物理类型。
NP02：本轮核真实依赖与工具路线。
本轮不新增内核编译证据。
NP03：本决议不作数值验收。
NP04：本决议不作性能验收。
本轮不运行GPU或Python。
本轮不重跑任何满额配置。
当前面设备位差仍未知。
2078只属于旧源码的失败。
原生诊断不替代GPU参考。
D06只关闭边界决议与清单工作。
P11分发准入与产品验收仍开放。

## 锁定依赖逐项清单

下表直接取成功元数据结果。
R代表注册表来源。
C代表上述CubeCL固定提交。
P代表上述Pliron固定提交。
L代表当前项目。
文本列记录路径及原始SHA-256。
R路径相对各包解包根。
C与P路径相对各Git仓库根。
“未检出”不等于无许可。
下表不替代分发所需完整原文。

| 包 | 版本 | 原始许可声明 | 来源 | 已见文本与SHA-256 |
| --- | --- | --- | --- | --- |
| addr2line | 0.25.1 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `e99d88d232bf57d70f0fb87f6b496d44b6653f99f8a63d250a54c61ea4bcde40` |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | R | `LICENSE-0BSD`: `861399f8c21c042b110517e76dc6b63a2b334276c8cf17412fc3c8908ca8dc17`<br>`LICENSE-APACHE`: `8ada45cd9f843acf64e4722ae262c622a2b3b3007c7310ef36ac1061a30f6adb`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| aho-corasick | 1.1.5 | Unlicense OR MIT | R | `COPYING`: `01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f`<br>`LICENSE-MIT`: `0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f` |
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `20fe7b00e904ed690e3b9fd6073784d3fc428141dbd10b81c01fd143d0797f58`<br>`LICENSE-MIT`: `36516aefdc84c5d5a1e7485425913a22dbda69eb1930c5e84d6ae4972b5194b9` |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `4da95ec4ecb65b738d470b7d762894ad9c97da93e6cbfb18b570fc2c96f4b871` |
| async-channel | 2.5.0 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `27995d58ad5c1145c1a8cd86244ce844886958a35eb2b78c6b772748669999ac` |
| awint | 0.19.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `5c71280e740d267af8fcd1928b7dba351a8fcbc8454b7674dfdb43e234df4e12` |
| awint_core | 0.19.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `5c71280e740d267af8fcd1928b7dba351a8fcbc8454b7674dfdb43e234df4e12` |
| awint_dag | 0.19.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `5c71280e740d267af8fcd1928b7dba351a8fcbc8454b7674dfdb43e234df4e12` |
| awint_ext | 0.19.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `5c71280e740d267af8fcd1928b7dba351a8fcbc8454b7674dfdb43e234df4e12` |
| awint_internals | 0.19.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `5c71280e740d267af8fcd1928b7dba351a8fcbc8454b7674dfdb43e234df4e12` |
| awint_macro_internals | 0.19.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `5c71280e740d267af8fcd1928b7dba351a8fcbc8454b7674dfdb43e234df4e12` |
| awint_macros | 0.19.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `5c71280e740d267af8fcd1928b7dba351a8fcbc8454b7674dfdb43e234df4e12` |
| backtrace | 0.3.76 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| base64 | 0.23.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `ab499c75a0f0da8e0fe83bf9ba3a27e5c39705310f07107c6c66b69958d2401c` |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5`<br>`LICENSE-MIT`: `d5c22aa3118d240e877ad41c5d9fa232f9c77d757d4aac0c2f943afc0a95e0ef` |
| buildid | 1.0.5 | MPL-2.0 | R | 未检出 |
| bumpalo | 3.20.3 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `65f94e99ddaf4f5d1782a6dae23f35d4293a9a01444a13135a6887017d353cee` |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `e3ba223bb1423f0aad8c3dfce0fe3148db48926d41e6fbc3afbbf5ff9e1c89cb`<br>`LICENSE-MIT`: `9df9ba60a11af705f2e451b53762686e615d86f76b169cf075c3237730dbd7e2`<br>`LICENSE-ZLIB`: `84b34dd7608f7fb9b17bd588a6bf392bf7de504e2716f024a77d89f1b145a151` |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `e3ba223bb1423f0aad8c3dfce0fe3148db48926d41e6fbc3afbbf5ff9e1c89cb`<br>`LICENSE-MIT`: `9df9ba60a11af705f2e451b53762686e615d86f76b169cf075c3237730dbd7e2`<br>`LICENSE-ZLIB`: `84b34dd7608f7fb9b17bd588a6bf392bf7de504e2716f024a77d89f1b145a151` |
| bytes | 1.12.1 | MIT | R | `LICENSE`: `45f522cacecb1023856e46df79ca625dfc550c94910078bd8aec6e02880b3d42` |
| cc | 1.6.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| cfg_aliases | 0.2.2 | MIT | R | `LICENSE`: `31b94860253d8ec7b4529f51901044d3b459d6292d996504a36b1bae3a36a812`<br>`NOTICES.md`: `1e2b7ade3fb228130408b9990cae6a7618eb314c75aa0b164bfe485d9d9756ee` |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| chacha20 | 0.10.2 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5`<br>`LICENSE-MIT`: `b8c6939380a400f53e11923d50fcc4dd2fa1ba8339fd9d04cda38a0251b6c9b0` |
| ciborium | 0.2.2 | Apache-2.0 | R | `LICENSE`: `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4` |
| ciborium-io | 0.2.2 | Apache-2.0 | R | `LICENSE`: `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4` |
| ciborium-ll | 0.2.2 | Apache-2.0 | R | `LICENSE`: `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4` |
| combine | 4.6.8 | MIT | R | `LICENSE`: `9bbc1b3dc4674a9ebb12c2a62f54fe3c08672539717f8d05828f684b6cc419a3` |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| const_fn | 0.4.12 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| constcat | 0.6.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4`<br>`LICENSE-MIT`: `f9c4c77baa3828004ee54b8a4f2db2e88ed44a6237a493965bf551fac0fcb62d` |
| convert_case | 0.10.0 | MIT | R | `LICENSE`: `aed7b1758e35afa0cd0fde059d61950747ca11cd0e5e169cb21c11608daed772` |
| convert_case | 0.11.0 | MIT | R | `LICENSE`: `aed7b1758e35afa0cd0fde059d61950747ca11cd0e5e169cb21c11608daed772` |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5`<br>`LICENSE-MIT`: `ae9baa7beea910273c2f384c2a6b721fb7bd02bda3436074a1072e4ee689f985` |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5`<br>`LICENSE-MIT`: `73b9dc2e79c7308998dd30296e073aefaefb944a68fb89aa412c23c0edcabcaa` |
| critical-section | 1.2.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `fee6bdac4b917351332567b9fd5013c2ef3c847dd113fd6216cf28e96637a157` |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `5734ed989dfca1f625b40281ee9f4530f91b2411ec01cb748223e7eb87e201ab` |
| crunchy | 0.2.4 | MIT | R | `LICENSE`: `3eab77734a181b2a2e8418af9004f960f715f04aac4d76a26cf774f63a7b13d1` |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5`<br>`LICENSE-MIT`: `3521672491a3479422d5fe1aca6645dd2984090f85da6e5205abfb18fb7a6897` |
| cubecl-common | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `crates/cubecl-common/LICENSE-APACHE`: `9cff568488e9d56aae60f162404acdb15d8533a0202cf01f6eb0843b264f20ff`<br>`crates/cubecl-common/LICENSE-MIT`: `94949dd83279f3ca3d0ce42ce0fc154d0ed817444454ee2925e903eec93949b6`<br>`LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-core | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `crates/cubecl-core/LICENSE-APACHE`: `9cff568488e9d56aae60f162404acdb15d8533a0202cf01f6eb0843b264f20ff`<br>`crates/cubecl-core/LICENSE-MIT`: `94949dd83279f3ca3d0ce42ce0fc154d0ed817444454ee2925e903eec93949b6`<br>`LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-cpp | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-environment | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-ir | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `crates/cubecl-ir/LICENSE-APACHE`: `9cff568488e9d56aae60f162404acdb15d8533a0202cf01f6eb0843b264f20ff`<br>`crates/cubecl-ir/LICENSE-MIT`: `94949dd83279f3ca3d0ce42ce0fc154d0ed817444454ee2925e903eec93949b6`<br>`LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-llvm | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `crates/cubecl-llvm/LICENSE-APACHE`: `9cff568488e9d56aae60f162404acdb15d8533a0202cf01f6eb0843b264f20ff`<br>`crates/cubecl-llvm/LICENSE-MIT`: `94949dd83279f3ca3d0ce42ce0fc154d0ed817444454ee2925e903eec93949b6`<br>`LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-macros | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `crates/cubecl-macros/LICENSE-APACHE`: `9cff568488e9d56aae60f162404acdb15d8533a0202cf01f6eb0843b264f20ff`<br>`crates/cubecl-macros/LICENSE-MIT`: `94949dd83279f3ca3d0ce42ce0fc154d0ed817444454ee2925e903eec93949b6`<br>`LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-macros-internal | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `crates/cubecl-macros-internal/LICENSE-APACHE`: `9cff568488e9d56aae60f162404acdb15d8533a0202cf01f6eb0843b264f20ff`<br>`crates/cubecl-macros-internal/LICENSE-MIT`: `94949dd83279f3ca3d0ce42ce0fc154d0ed817444454ee2925e903eec93949b6`<br>`LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-monitoring | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-opt | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-runtime | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `crates/cubecl-runtime/LICENSE-APACHE`: `9cff568488e9d56aae60f162404acdb15d8533a0202cf01f6eb0843b264f20ff`<br>`crates/cubecl-runtime/LICENSE-MIT`: `94949dd83279f3ca3d0ce42ce0fc154d0ed817444454ee2925e903eec93949b6`<br>`LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cubecl-zspace | 0.11.0-pre.4 | MIT OR Apache-2.0 | C | `LICENSE-APACHE`: `bdd26d197850a30393da963f01e97c78e91d69245089c543903f96a5dab95930`<br>`LICENSE-MIT`: `b3b77503e5498c5fb0a1a0b102d748d7baed9cae9f502086a3cfedad62d503a4`<br>`NOTICE.md`: `70a9ae1dc17b8efb603c3fdb5f7798041a5450754f524ee6c57c764e18427b13` |
| cudarc | 0.18.2 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| darling | 0.24.1 | MIT | R | `LICENSE`: `8ea93490d74a5a1b1af3ff71d786271b3f1e5f0bea79ac16e02ec533cef040d6` |
| darling_core | 0.24.1 | MIT | R | `LICENSE`: `8ea93490d74a5a1b1af3ff71d786271b3f1e5f0bea79ac16e02ec533cef040d6` |
| darling_macro | 0.24.1 | MIT | R | `LICENSE`: `8ea93490d74a5a1b1af3ff71d786271b3f1e5f0bea79ac16e02ec533cef040d6` |
| derive_more | 2.1.1 | MIT | R | `LICENSE`: `8a35369f3ca263b3c62fbb5032947e53b6bfebc6c8a4d1bb982de1c069f6fba5` |
| derive_more-impl | 2.1.1 | MIT | R | `LICENSE`: `8a35369f3ca263b3c62fbb5032947e53b6bfebc6c8a4d1bb982de1c069f6fba5` |
| derive-new | 0.7.0 | MIT | R | `LICENSE`: `e13217b08deef741d527bd01178f8c2c801122fe9c3221f61762e95d31eceec1` |
| digest | 0.10.7 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5`<br>`LICENSE-MIT`: `9e0dfd2dd4173a530e238cb6adb37aa78c34c6bc7444e0e10c1ab5d8881f63ba` |
| dirs | 6.0.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `d3174ad63e721d4c9dccb8ad4320848992d314369bc46319720b5802c9153fe9`<br>`LICENSE-MIT`: `6a2e0ade09a7d5f816f11566fee2b151b32235a7fad52b41d49cce96f833c1a9` |
| dirs-sys | 0.5.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `d3174ad63e721d4c9dccb8ad4320848992d314369bc46319720b5802c9153fe9`<br>`LICENSE-MIT`: `6a2e0ade09a7d5f816f11566fee2b151b32235a7fad52b41d49cce96f833c1a9` |
| document-features | 0.2.12 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `074e6e32c86a4c0ef8b3ed25b721ca23aca83df277cd88106ef7177c354615ff`<br>`LICENSE-MIT`: `aa893340d14b9844625be6a50ac644169a01b52f0211cbf81b09e1874c8cd81d` |
| downcast-rs | 2.0.2 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `8173d5c29b4f956d532781d2b86e4e30f83e6b7878dce18c919451d6ba707c90`<br>`LICENSE-MIT`: `fb252fecf98d9af8f1d9201c5860319a514f5a8a24700dc84fc6ea287a7a4958` |
| dyn-clone | 1.0.20 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| either | 1.18.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `7576269ea71f767b99297934c0b2367532690f8c4badc695edf8e04ab6a1e545` |
| embassy-futures | 0.1.2 | MIT OR Apache-2.0 | R | 未检出 |
| embassy-time | 0.5.1 | MIT OR Apache-2.0 | R | 未检出 |
| embassy-time-driver | 0.2.2 | MIT OR Apache-2.0 | R | 未检出 |
| embedded-hal | 0.2.7 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `520bc4942f199aad3dfe939c475d67ffbddfea482e41fc4fcad8829b71affc99` |
| embedded-hal | 1.0.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `520bc4942f199aad3dfe939c475d67ffbddfea482e41fc4fcad8829b71affc99` |
| embedded-hal-async | 1.0.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `b32b47118594bca5e21a37d64d9924bff856bf4f688c482b417cc5e885d6e7bf` |
| equivalent | 1.0.2 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `7365cc8878a1d7ce155a58c4ca09c3d7a6be413efa5334a80ea842912b669349` |
| errno | 0.3.14 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `8764a597675778ddfd4e25f81b08a05dbcf089ac05662df7613fe67f150e3aa2` |
| etcetera | 0.11.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| event-listener | 5.4.2 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| filetime | 0.2.29 | MIT/Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| find-msvc-tools | 0.1.14 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| float-ord | 0.3.2 | MIT / Apache-2.0 | R | `LICENSE-APACHE`: `c6596eb7be8581c18be736c846fb9173b69eccf6ef94c5135893ec56bd92ba08`<br>`LICENSE-MIT`: `0a371b88c0f6a8766cddb6cd7b7dbe37c6645570888fb2b8619b16fd5983e884` |
| float4 | 0.2.0 | MIT | R | `LICENSE`: `b40bb09c7b3b6f697a8920cf9d2cb3ff8cfcf77a0fcb6d264a92842b43c4dcae` |
| float8 | 0.7.0 | MIT | R | `LICENSE`: `b40bb09c7b3b6f697a8920cf9d2cb3ff8cfcf77a0fcb6d264a92842b43c4dcae` |
| foldhash | 0.1.5 | Zlib | R | `LICENSE`: `b1181a40b2a7b25cf66fd01481713bc1005df082c53ef73e851e55071b102744` |
| foldhash | 0.2.0 | Zlib | R | `LICENSE`: `b1181a40b2a7b25cf66fd01481713bc1005df082c53ef73e851e55071b102744` |
| futures-core | 0.3.34 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `275c491d6d1160553c32fd6127061d7f9606c3ea25abfad6ca3f6ed088785427`<br>`LICENSE-MIT`: `6652c868f35dfe5e8ef636810a4e576b9d663f3a17fb0f5613ad73583e1b88fd` |
| futures-io | 0.3.34 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `275c491d6d1160553c32fd6127061d7f9606c3ea25abfad6ca3f6ed088785427`<br>`LICENSE-MIT`: `6652c868f35dfe5e8ef636810a4e576b9d663f3a17fb0f5613ad73583e1b88fd` |
| futures-lite | 2.6.1 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3`<br>`LICENSE-THIRD-PARTY`: `6226d0632e2e1a80c23597e964da9812ae193c535fe058154afb034e94167aa5` |
| futures-task | 0.3.34 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `275c491d6d1160553c32fd6127061d7f9606c3ea25abfad6ca3f6ed088785427`<br>`LICENSE-MIT`: `6652c868f35dfe5e8ef636810a4e576b9d663f3a17fb0f5613ad73583e1b88fd` |
| futures-util | 0.3.34 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `275c491d6d1160553c32fd6127061d7f9606c3ea25abfad6ca3f6ed088785427`<br>`LICENSE-MIT`: `6652c868f35dfe5e8ef636810a4e576b9d663f3a17fb0f5613ad73583e1b88fd` |
| generic-array | 0.14.7 | MIT | R | `LICENSE`: `c09aae9d3c77b531f56351a9947bc7446511d6b025b3255312d3e3442a9a7583` |
| getrandom | 0.2.17 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `aaff376532ea30a0cd5330b9502ad4a4c8bf769c539c87ffe78819d188a18ebf`<br>`LICENSE-MIT`: `42fa16951ce7f24b5a467a40e5b449a1d41e662f97ca779864f053f39e097737` |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `aaff376532ea30a0cd5330b9502ad4a4c8bf769c539c87ffe78819d188a18ebf`<br>`LICENSE-MIT`: `523a42c25d245dde9c015f882cec7f4555aad883382a6cf19b4b7d9b2cd5419b` |
| gimli | 0.32.3 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `7b63ecd5f1902af1b63729947373683c32745c16a10e8e6292e2e2dcd7e90ae0` |
| half | 2.7.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a6cba85bc92e0cff7a450b1d873c0eaa2e9fc96bf472df0247a26bec77bf3ff9`<br>`LICENSE-MIT`: `508a77d2e7b51d98adeed32648ad124b7b30241a8e70b2e72c99f92d8e5874d1` |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `ff8f68cb076caf8cefe7a6430d4ac086ce6af2ca8ce2c4e5a2004d4552ef52a2` |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `ff8f68cb076caf8cefe7a6430d4ac086ce6af2ca8ce2c4e5a2004d4552ef52a2` |
| hermit-abi | 0.5.3 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| hi_sparse_bitset | 0.10.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `ec1981d6135d49c18b8f9ed1626966c6fd168c5072eadbb4ad65dc746520e388`<br>`LICENSE-MIT`: `da91ce742c71b1cfecbe5256ca79dc2665b82f88533fc7facc3da26ea969d5d7` |
| hi_sparse_bitset | 0.9.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `ec1981d6135d49c18b8f9ed1626966c6fd168c5072eadbb4ad65dc746520e388`<br>`LICENSE-MIT`: `da91ce742c71b1cfecbe5256ca79dc2665b82f88533fc7facc3da26ea969d5d7` |
| http | 1.5.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `8bb1b50b0e5c9399ae33bd35fab2769010fa6c14e8860c729a52295d84896b7a`<br>`LICENSE-MIT`: `dc91f8200e4b2a1f9261035d4c18c33c246911a6c0f7b543d75347e61b249cff` |
| httparse | 1.10.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `391a5396cec6230bfabd4ef4eb2350eb895bc5efce377a2218f5702ed020d3e3` |
| ident_case | 1.0.1 | MIT/Apache-2.0 | R | `LICENSE`: `508a77d2e7b51d98adeed32648ad124b7b30241a8e70b2e72c99f92d8e5874d1` |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `ecc269ef87fd38a1d98e30bfac9ba964a9dbd9315c3770fed98d4d7cb5882055` |
| inflections | 1.1.1 | MIT | R | `LICENSE`: `fe5c9498374de7953275f83f35565b289f377ceab4a28d04c7da87723e02a372` |
| internment | 0.8.6 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `3528317b4cafbde89260bc5f4cd823867d891f827e91c33ded6546044709f988` |
| inventory | 0.3.24 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| itertools | 0.15.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `7576269ea71f767b99297934c0b2367532690f8c4badc695edf8e04ab6a1e545` |
| itoa | 1.0.18 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| jobserver | 0.1.35 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| js-sys | 0.3.106 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| lazy_static | 1.5.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| libc | 0.2.190 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `123a331b5dbf04c30097fa43b8f858bc85df671fe776de498d01f3d6b7c1f69e` |
| libloading | 0.8.8 | ISC | R | `LICENSE`: `b29f8b01452350c20dd1af16ef83b598fea3053578ccc1c7a0ef40e57be2620f` |
| liblzma | 0.4.8 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `a90e2da47bdc152cfb89e1fb09972143056c417ddbf86122babc83d0a8ad81d1` |
| liblzma-sys | 0.4.9 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `a90e2da47bdc152cfb89e1fb09972143056c417ddbf86122babc83d0a8ad81d1` |
| libm | 0.2.16 | MIT | R | `LICENSE.txt`: `3823dda7cf046602f4b4e77ec8e227863dc4736037cc85bb33d9f19febe16bb7` |
| libredox | 0.1.25 | MIT | R | `LICENSE`: `8d073a6a80d1ef2d2fc2bc6c809aa221161bf1906c76d4aeef55fd2b999d9413` |
| linkme | 0.3.37 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| linkme-impl | 0.3.37 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | R | `COPYRIGHT`: `3290ae0fbc9ddb77d2239121d710f0bb9d31b3b4744e6d97fe01e652b4c1870b`<br>`LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-Apache-2.0_WITH_LLVM-exception`: `268872b9816f90fd8e85db5a28d33f8150ebb8dd016653fb39ef1f94f2686bc5`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| litrs | 1.0.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `7dc1552e88f49132cb358b1b962fc5e79fa42d70bcbb88c526d33e45b8e98036` |
| llvm-sys | 231.0.0 | MIT | R | `LICENSE`: `e8b466671593a1337ac18888417709c4def3c213a0fc28ca4894fe0bf2f03f6c` |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `c9a75f18b9ab2927829a208fc6aa2cf4e63b8420887ba29cdb265d6619ae82d5` |
| log | 0.4.34 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| memchr | 2.8.3 | Unlicense OR MIT | R | `COPYING`: `01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f`<br>`LICENSE-MIT`: `0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f` |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | R | `LICENSE`: `4108245a1f2df9d4e94df8abed5b4ba0759bb2f9b40a6b939f1be141077ae50b`<br>`LICENSE-APACHE.md`: `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594`<br>`LICENSE-MIT.md`: `799e9ca9d179295ef372f25d3769cdda7d25bb2668add6a6a1e22d1e4c678b8d`<br>`LICENSE-ZLIB.md`: `0a54e647fe54104658b5e563c04c6f9edf251710e47bce692e0bd990a4ddaa39` |
| mjwarp-rs | 0.1.0 | 未声明 | L | 未检出 |
| mutants | 0.0.3 | MIT | R | 未检出 |
| nb | 0.1.3 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `035e70219855119df4273b3c5b97543ae82e0dd60c520416e759107c602f651b` |
| nb | 1.1.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `035e70219855119df4273b3c5b97543ae82e0dd60c520416e759107c602f651b` |
| nom | 8.0.0 | MIT | R | `LICENSE`: `4dbda04344456f09a7a588140455413a9ac59b6b26a1ef7cdf9c800c012d87f0` |
| num_cpus | 1.17.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `391a5396cec6230bfabd4ef4eb2350eb895bc5efce377a2218f5702ed020d3e3` |
| num-complex | 0.4.6 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| num-integer | 0.1.47 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| object | 0.37.3 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `0b74dfa0bcee5c420c6b7f67b4b2658f9ab8388c97b8e733975f2cecbdd668a6` |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| once_vec | 0.4.0 | Apache-2.0 | R | `LICENSE.md`: `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4` |
| oneshot | 0.2.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| option-ext | 0.2.0 | MPL-2.0 | R | `LICENSE.txt`: `66a3107d5ad6a058aab753eaac2047ccb2ed0e39465dd0fe5844da3e300d5172` |
| parking | 2.2.1 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3`<br>`LICENSE-THIRD-PARTY`: `9a6d7a3c1b8edcc706c4b6c37f8dae9a77f02c7f9051df603f975db136b55419` |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `c9a75f18b9ab2927829a208fc6aa2cf4e63b8420887ba29cdb265d6619ae82d5` |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `c9a75f18b9ab2927829a208fc6aa2cf4e63b8420887ba29cdb265d6619ae82d5` |
| paste | 1.0.15 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `b38f11f6096706e6de553dabe2a7ed142d59b6fa8c97e290c67496154745cdd5` |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| pliron | 0.18.0 | Apache-2.0 | P | `LICENSE.md`: `4f86ad10a7abb670af6ad2e7adafdb10e96d855beba933b36de2d763d85dc2e8`<br>`NOTICE`: `416f25d587308a3ae3070e95544fb12fe964e544cf7cffe82160cab948dc2fca` |
| pliron-derive | 0.18.0 | Apache-2.0 | P | `LICENSE.md`: `4f86ad10a7abb670af6ad2e7adafdb10e96d855beba933b36de2d763d85dc2e8`<br>`NOTICE`: `416f25d587308a3ae3070e95544fb12fe964e544cf7cffe82160cab948dc2fca`<br>`pliron-derive/LICENSE.md`: `4f86ad10a7abb670af6ad2e7adafdb10e96d855beba933b36de2d763d85dc2e8`<br>`pliron-derive/NOTICE`: `416f25d587308a3ae3070e95544fb12fe964e544cf7cffe82160cab948dc2fca` |
| pliron-llvm | 0.18.0 | Apache-2.0 | P | `LICENSE.md`: `4f86ad10a7abb670af6ad2e7adafdb10e96d855beba933b36de2d763d85dc2e8`<br>`NOTICE`: `416f25d587308a3ae3070e95544fb12fe964e544cf7cffe82160cab948dc2fca`<br>`pliron-llvm/LICENSE.md`: `4f86ad10a7abb670af6ad2e7adafdb10e96d855beba933b36de2d763d85dc2e8`<br>`pliron-llvm/NOTICE`: `416f25d587308a3ae3070e95544fb12fe964e544cf7cffe82160cab948dc2fca` |
| portable-atomic | 1.15.0 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| portable-atomic-util | 0.2.8 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `0d542e0c8804e39aa7f37eb00da5a762149dc682d7829451287e11b938e94594`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| prettyplease | 0.3.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| quote | 1.0.47 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | R | 未检出 |
| rand | 0.10.3 | MIT OR Apache-2.0 | R | `COPYRIGHT`: `90eb64f0279b0d9432accfa6023ff803bc4965212383697eee27a0f426d5f8d5`<br>`LICENSE-APACHE`: `35242e7a83f69875e6edeff02291e688c97caafe2f8902e4e19b49d3e78b4cab`<br>`LICENSE-MIT`: `209fbbe0ad52d9235e37badf9cadfe4dbdc87203179c0899e738b39ade42177b` |
| rand_core | 0.10.1 | MIT OR Apache-2.0 | R | `COPYRIGHT`: `92b81db30f7ab6d693e0af5661a40d273ca1947e891123ed823924192c10cbc7`<br>`LICENSE-APACHE`: `6df43f6f4b5d4587f3d8d71e45532c688fd168afa5fe89d571cb32fa09c4ef51`<br>`LICENSE-MIT`: `8b6e9feec03e7c9a5facb26855cecd31662bf989b636bcfe79521bdf8ac863f0` |
| recasting | 0.2.3 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `0fb9a918700c99457aa83a0ff99e14584cac5ece5cc9faa7c584dba17579ffe5` |
| redox_syscall | 0.5.18 | MIT | R | `LICENSE`: `efcfee7981ff72431fffb06925cad00a23dce079ed4354f61030ad5abdb78829` |
| redox_users | 0.5.3 | MIT | R | `LICENSE`: `e6a8ae2d796083783efc94b1e66271aa2929dc4dfb231d34239aa9c7db8396db` |
| regex | 1.13.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| regex-lite | 0.1.9 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `6485b8ed310d3f0340bf1ad1f47645069ce4069dcc6bb46c7d5c6faf41de1fdb` |
| ring | 0.17.14 | Apache-2.0 AND ISC | R | `LICENSE`: `b3d734001a94efff3579978d953391aa7115f877657d25eb54037a43875d078a`<br>`LICENSE-BoringSSL`: `005fc765ddc5115da796cca915baa9557abae13ff35e0a47c47affc56f6c414d`<br>`LICENSE-other-bits`: `f025ccfb7dfb6bdfedc75ca0f67acc69e6fb4998143d834f7c2f38a29989680f` |
| rustc_apfloat | 0.2.3+llvm-462a31f5a5ab | Apache-2.0 WITH LLVM-exception | R | `LICENSE-DETAILS.md`: `5998f303e26191363f591e04bdd0f829b2000afc843c67326a9b7efd66850416`<br>`LICENSE.txt`: `981f4155fbd55dcf13745e2ed508e6fa30aa90f9f668c4ef0f7686980e5d8521` |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `c9a75f18b9ab2927829a208fc6aa2cf4e63b8420887ba29cdb265d6619ae82d5` |
| rustc-demangle | 0.1.28 | MIT/Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `95bd3988beee069fa2848f648dab43cc6e0b2add2ad6bcb17360caf749802bcc`<br>`LICENSE-MIT`: `30fefc3a7d6a0041541858293bcbea2dde4caa4c0a5802f996a7f7e8c0085652` |
| rustix | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | R | `COPYRIGHT`: `377c2e7c53250cc5905c0b0532d35973392af16ffb9596a41d99d202cf3617c9`<br>`LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-Apache-2.0_WITH_LLVM-exception`: `268872b9816f90fd8e85db5a28d33f8150ebb8dd016653fb39ef1f94f2686bc5`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| rustls | 0.23.45 | Apache-2.0 OR ISC OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-ISC`: `7cfafc877eccc46c0e346ccbaa5c51bb6b894d2b818e617d970211e232785ad4`<br>`LICENSE-MIT`: `709e3175b4212f7b13aa93971c9f62ff8c69ec45ad8c6532a7e0c41d7a7d6f8c` |
| rustls-pki-types | 1.15.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `45fd05c4865e7c350b98ad7ac50e1b15462d49af4a91e9b0c9dd933dc9a69742`<br>`LICENSE-MIT`: `9117d922e667125508dde62b02c1f57ed22f5ad21eb536aa2e2d99e1c796e639` |
| rustls-webpki | 0.103.15 | ISC | R | `LICENSE`: `5b698ca13897be3afdb7174256fa1574f8c6892b8bea1a66dd6469d3fe27885a` |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| safe_arch | 1.2.0 | Zlib OR Apache-2.0 OR MIT | R | `LICENSE-APACHE.md`: `e3ba223bb1423f0aad8c3dfce0fe3148db48926d41e6fbc3afbbf5ff9e1c89cb`<br>`LICENSE-MIT.md`: `e57011537d230b14e790f6666dc00816f7b371ebbd7da8a12491e51086fec278`<br>`LICENSE-ZLIB.md`: `c43b9a9b1387ed53d2c49263838261129a010e280e3a174a792242c3e2c98db9` |
| same-file | 1.0.6 | Unlicense/MIT | R | `COPYING`: `01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f`<br>`LICENSE-MIT`: `cb3c929a05e6cbc9de9ab06a4c57eeb60ca8c724bef6c138c87d3a577e27aa14` |
| sanitize-filename | 0.6.0 | MIT | R | `LICENSE`: `f3760c9773984365cc37ca7eccf7a743a60baf8859f09fed24ad588c81bfbaf3` |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `fb77f0a9c53e473abe5103c8632ef9f0f2874d4fb3f17cb2d8c661aab9cee9d7` |
| semver | 1.0.28 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| serde | 1.0.229 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| serde_bytes | 0.11.19 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| serde_spanned | 1.1.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `c6596eb7be8581c18be736c846fb9173b69eccf6ef94c5135893ec56bd92ba08`<br>`LICENSE-MIT`: `6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6` |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a9040321c3712d8fd0b09cf52b17445de04a23a10165049ae187cd39e5c86be5`<br>`LICENSE-MIT`: `b4eb00df6e2a4d22518fcaa6a2b4646f249b3a3c9814509b22bd2091f1392ff1` |
| shlex | 2.0.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `553fffcd9b1cb158bc3e9edc35da85ca5c3b3d7d2e61c883ebcfa8a65814b583`<br>`LICENSE-MIT`: `4455bf75a91154108304cb283e0fea9948c14f13e20d60887cf2552449dea3b1` |
| slab | 0.4.12 | MIT | R | `LICENSE`: `8ce0830173fdac609dfb4ea603fdc002c2f4af0dc9b1a005653f5da9cf534b18` |
| slotmap | 1.1.1 | Zlib | R | `LICENSE`: `6cec81441b2ab2b5090954c1fb5d0bcca81b86487b5849061fa8952d20ef67d4` |
| smallvec | 1.16.2 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `0b28172679e0009b655da42797c03fd163a3379d5cfa67ba1f1655e974a2a1a9` |
| spin | 0.12.3 | MIT | R | `LICENSE`: `6ac8711fb340c62ce0a4ecd463342d3fa0e8e70de697c863a2e1c0c53006003c` |
| strsim | 0.11.1 | MIT | R | `LICENSE`: `1e697ce8d21401fbf1bddd9b5c3fd4c4c79ae1e3bdf51f81761c85e11d5a89cd` |
| subtle | 2.6.1 | BSD-3-Clause | R | `LICENSE`: `d1fc1bc0d155df60b2e7705b6b2ae02a05c96f948e1cec6e2fb86360b09f346b` |
| syn | 2.0.119 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| syn | 3.0.6 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| tar | 0.4.46 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `8ca6b96cea9e67c6c5c63f452c31bd396db8bd2406231fdea5d48ef462b48077` |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| tokio | 1.53.2 | MIT | R | `LICENSE`: `253cd04c6714889df2d32f3f64d669179a1c95c76ac43c40882c52eb06bc3552` |
| toml | 1.1.6+spec-1.1.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `c6596eb7be8581c18be736c846fb9173b69eccf6ef94c5135893ec56bd92ba08`<br>`LICENSE-MIT`: `6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6` |
| toml_datetime | 1.1.1+spec-1.1.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `c6596eb7be8581c18be736c846fb9173b69eccf6ef94c5135893ec56bd92ba08`<br>`LICENSE-MIT`: `6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6` |
| toml_parser | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `c6596eb7be8581c18be736c846fb9173b69eccf6ef94c5135893ec56bd92ba08`<br>`LICENSE-MIT`: `6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6` |
| toml_writer | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `c6596eb7be8581c18be736c846fb9173b69eccf6ef94c5135893ec56bd92ba08`<br>`LICENSE-MIT`: `6efb0476a1cc085077ed49357026d8c173bf33017278ef440f222fb9cbcb66e6` |
| tracel-llvm-bundler | 23.1.0-3 | MIT OR Apache-2.0 | R | 未检出 |
| triple_arena | 0.15.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `5c71280e740d267af8fcd1928b7dba351a8fcbc8454b7674dfdb43e234df4e12` |
| tynm | 0.2.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`<br>`LICENSE-MIT`: `b27e3b6898d6a0716b26f2c4be76d03a634581c5ba9bb6fab5433c0117b84785` |
| typenum | 1.20.1 | MIT OR Apache-2.0 | R | `LICENSE`: `db11fec9946737df39ca3898d9cd8c10ec6f6c3a884a6802b0ad0b81b4e8f23a`<br>`LICENSE-APACHE`: `516b24e051bf5630880ebbd55c40a25ce9552ebaf8970a53e8976eb70e522406`<br>`LICENSE-MIT`: `a825bd853ab71619a4923d7b4311221427848070ff44d990da39b0b274c1683f` |
| unicode-ident | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3`<br>`LICENSE-UNICODE`: `f7db81051789b729fea528a63ec4c938fdcb93d9d61d97dc8cc2e9df6d47f2a1` |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | R | `COPYRIGHT`: `23860c2a7b5d96b21569afedf033469bab9fe14a1b24a35068b8641c578ce24d`<br>`LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `7b63ecd5f1902af1b63729947373683c32745c16a10e8e6292e2e2dcd7e90ae0` |
| unicode-xid | 0.2.6 | MIT OR Apache-2.0 | R | `COPYRIGHT`: `23860c2a7b5d96b21569afedf033469bab9fe14a1b24a35068b8641c578ce24d`<br>`LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `7b63ecd5f1902af1b63729947373683c32745c16a10e8e6292e2e2dcd7e90ae0` |
| unsynn | 0.3.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| untrusted | 0.9.0 | ISC | R | `LICENSE.txt`: `7abd9b6960dcf7d4d0a48606a5b71bfe37d472db68d70637f3a58a56785f1621` |
| ureq | 3.4.2 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4`<br>`LICENSE-MIT`: `7e5886959f67f8c75063a9d55cd3cb08c5a8cbea9a944fa1df2d4ae038f53721` |
| ureq-proto | 0.6.4 | MIT OR Apache-2.0 | R | `LICENSE-APACHE.txt`: `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`<br>`LICENSE-MIT.txt`: `5b460f37be48ac4cf2181596181cc6cae432fc2caae05944873f380730829a4b` |
| utf8-chars | 3.0.7 | MIT OR Apache-2.0 | R | 未检出 |
| utf8-zero | 0.8.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| variadics_please | 2.0.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a6cba85bc92e0cff7a450b1d873c0eaa2e9fc96bf472df0247a26bec77bf3ff9`<br>`LICENSE-MIT`: `508a77d2e7b51d98adeed32648ad124b7b30241a8e70b2e72c99f92d8e5874d1` |
| version_check | 0.9.5 | MIT/Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `b7e650f3fce5c53249d1cdc608b54df156a97edd636cf9d23498d0cfe7aec63e` |
| void | 1.0.2 | MIT | R | 未检出 |
| walkdir | 2.5.0 | Unlicense/MIT | R | `COPYING`: `01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f`<br>`LICENSE-MIT`: `0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f` |
| wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-Apache-2.0_WITH_LLVM-exception`: `268872b9816f90fd8e85db5a28d33f8150ebb8dd016653fb39ef1f94f2686bc5`<br>`LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
| wasm-bindgen | 0.2.129 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| wasm-bindgen-futures | 0.4.79 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| wasm-bindgen-macro | 0.2.129 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| wasm-bindgen-macro-support | 0.2.129 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| wasm-bindgen-shared | 0.2.129 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397` |
| web-time | 1.1.0 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `54a744942eb3fa63a74d8d359989ed6962c04f3f10ff20cfa08ff8aa7efbe4e6`<br>`LICENSE-MIT`: `f428305bbf2e70fbd807af0f3c7ac1b7acfb95311640f451e9d2e48db6d7e96b` |
| webpki-roots | 1.0.9 | CDLA-Permissive-2.0 | R | `LICENSE`: `e271993808fec50ab29350b39539cdec611a9103f827e0aa26d61da70e2d33f8` |
| wide | 1.7.1 | Zlib OR Apache-2.0 OR MIT | R | `LICENSE-APACHE.txt`: `d64ac6ba60c7352115244dcfcb46c24f934603dd92e64835beafe7631c485abc`<br>`LICENSE-MIT.txt`: `69c62cda6938d510467c134dc0b5c83fb9c8ca847b9cd9d9d2fe9ab427c82100`<br>`LICENSE-ZLIB.txt`: `3c6921f3dfec0bed0cfa2d632616edd7380a875c8ad76c7b23dc59892259f62c` |
| winapi | 0.3.9 | MIT/Apache-2.0 | R | `LICENSE-APACHE`: `b40930bbcf80744c86c46a12bc9da056641d722716c378f5659b9e555ef833e1`<br>`LICENSE-MIT`: `ce7bc3499fee93d5022ef430d5e4201e79a6d9154f3974e42f41349f0569e09b` |
| winapi-i686-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 | R | 未检出 |
| winapi-util | 0.1.11 | Unlicense OR MIT | R | `COPYING`: `01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f`<br>`LICENSE-MIT`: `cb3c929a05e6cbc9de9ab06a4c57eeb60ca8c724bef6c138c87d3a577e27aa14` |
| winapi-x86_64-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 | R | 未检出 |
| windows_aarch64_gnullvm | 0.52.6 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_aarch64_gnullvm | 0.53.1 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_aarch64_msvc | 0.52.6 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_aarch64_msvc | 0.53.1 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_i686_gnu | 0.52.6 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_i686_gnu | 0.53.1 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_i686_gnullvm | 0.52.6 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_i686_gnullvm | 0.53.1 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_i686_msvc | 0.52.6 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_i686_msvc | 0.53.1 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_x86_64_gnu | 0.52.6 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_x86_64_gnu | 0.53.1 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_x86_64_gnullvm | 0.52.6 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_x86_64_gnullvm | 0.53.1 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows_x86_64_msvc | 0.53.1 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows-sys | 0.52.0 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows-targets | 0.52.6 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| windows-targets | 0.53.5 | MIT OR Apache-2.0 | R | `license-apache-2.0`: `c16f8dcf1a368b83be78d826ea23de4079fe1b4469a0ab9ee20563f37ff3d44b`<br>`license-mit`: `c2cfccb812fe482101a8f04597dfc5a9991a6b2748266c47ac91b6a5aae15383` |
| winnow | 1.0.4 | MIT | R | `LICENSE-MIT`: `cb5aedb296c5246d1f22e9099f925a65146f9f0d6b4eebba97fd27a6cdbbab2d` |
| xattr | 1.6.1 | MIT OR Apache-2.0 | R | `LICENSE-APACHE`: `a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2`<br>`LICENSE-MIT`: `8b427f5bc501764575e52ba4f9d95673cf8f6d80a86d0d06599852e1a9a20a36` |
| xxhash-rust | 0.8.19 | BSL-1.0 | R | `LICENSE`: `c9bff75738922193e67fa726fa225535870d2aa1059f91452c411736284ad566` |
| zerocopy | 0.8.60 | BSD-2-Clause OR Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `9d185ac6703c4b0453974c0d85e9eee43e6941009296bb1f5eb0b54e2329e9f3`<br>`LICENSE-BSD`: `83c1763356e822adde0a2cae748d938a73fdc263849ccff6b27776dff213bd32`<br>`LICENSE-MIT`: `1a2f5c12ddc934d58956aa5dbdd3255fe55fd957633ab7d0d39e4f0daa73f7df` |
| zerocopy-derive | 0.8.60 | BSD-2-Clause OR Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `9d185ac6703c4b0453974c0d85e9eee43e6941009296bb1f5eb0b54e2329e9f3`<br>`LICENSE-BSD`: `83c1763356e822adde0a2cae748d938a73fdc263849ccff6b27776dff213bd32`<br>`LICENSE-MIT`: `1a2f5c12ddc934d58956aa5dbdd3255fe55fd957633ab7d0d39e4f0daa73f7df` |
| zeroize | 1.9.0 | Apache-2.0 OR MIT | R | `LICENSE-APACHE`: `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`<br>`LICENSE-MIT`: `8c7516d4b27b1e495be5e38b612298b63de48d05f49cdac94f70f3cd70f8864b` |
| zmij | 1.0.23 | MIT | R | `LICENSE-MIT`: `23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3` |
