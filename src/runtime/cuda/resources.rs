//! 移交真实CUDA所有者。
//! 探针不接收外部裸指针。

mod affine;
mod checks;
mod fence;
mod foreign;
mod probe;
#[cfg(test)]
mod tests;

pub(super) use affine::launch_affine_raw;
pub(super) use fence::CudaFence;
pub(super) use foreign::ForeignContext;
pub(in crate::runtime) use probe::run;
