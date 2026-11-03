//! R10-S1：多 world 渲染状态与布局（宿主侧，无 GPU、无窗口）。
//!
//! 冻结上游 `render_util.create_render_context`（`71da24d`）的状态带前导 `nworld` 维：
//! 相机位姿 `cam_xpos[nworld, ncam, 3]` / `cam_xmat[nworld, ncam, 9]`，输出缓冲形状
//! `[nworld, pixels]`（world-major）。本模块把宿主侧状态从单 world 泛化：**同一个模型**复制到
//! `nworld` 个 world（多 state），world 维只增加状态副本、**不解析第二个模型**；每 world 的
//! 输出区段由既有 [`OutputLayout`] 按 `nworld` 乘法展开、互不重叠。
//!
//! 文件拆分（R10-S1 观察收口，行为不变）：[`errors`]（错误与共享校验助手）、
//! [`state`]（相机位姿/分辨率表/状态容器）、[`layout`]（输出区段）。
//!
//! world 越界/未激活/状态缺失/相机数或分辨率与 `nworld` 不匹配/`nworld == 0` 都在读写前返回
//! [`WorldError`]，不 panic；极大 `nworld` 走 `try_reserve` 返回 `CountOverflow`。内核/设备源
//! 不变（R10-S2 才接 GPU）。

mod errors;
pub mod layout;
pub mod scene_poses;
pub mod state;

pub use errors::WorldError;
pub use layout::WorldLayout;
pub use scene_poses::{ScenePose, WorldScenePoses};
pub use state::{CameraPose, WorldCameraPoses, WorldResolutions, WorldStates};
