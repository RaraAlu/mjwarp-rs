// 质量求解模块根拆分后的源码组合锚点。
// 顺序固定：模块根、内核常量子模块、发射点子模块。
// 该锚点服务内核模板与其发射路由的文本解析；产品身份哈希另行处理。
concat!(
    include_str!("../../../src/physics/mass_solve.rs"),
    "\n",
    include_str!("../../../src/physics/mass_solve/cuda.rs"),
    "\n",
    include_str!("../../../src/physics/mass_solve/kernel.rs"),
)
