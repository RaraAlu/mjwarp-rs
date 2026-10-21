// 物理模块根拆分后的产品源码组合锚点。
// 顺序固定：模块根在前，随后按模块根 mod 声明顺序排列产品子模块。
// 组合口径是完整产品源码，不含 #[cfg(test)] 的 tests 子模块。
// 该表达式由测试用 include! 注入，语义锚点据此解析 CUDA 模板与路由。
concat!(
    include_str!("../../../src/physics/mod.rs"),
    "\n",
    include_str!("../../../src/physics/com_position.rs"),
    "\n",
    include_str!("../../../src/physics/device_checks.rs"),
    "\n",
    include_str!("../../../src/physics/history.rs"),
    "\n",
    include_str!("../../../src/physics/kinematics.rs"),
)
