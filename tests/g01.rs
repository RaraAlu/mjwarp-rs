#![cfg(all(feature = "cuda-probe", feature = "native-model-probe"))]
//! G01完整链验收。参考采用离线原生计算。

#[path = "g01/frames.rs"]
mod frames;
#[path = "g01/full_chain.rs"]
mod full_chain;
#[path = "g01/host.rs"]
mod host;
#[path = "g01/parameters.rs"]
mod parameters;
#[path = "g01/preflight.rs"]
mod preflight;
#[path = "g01/stages.rs"]
mod stages;
#[path = "g01/support.rs"]
mod support;
#[path = "support/two_reference.rs"]
mod two_reference;
#[path = "g01/wake.rs"]
mod wake;
