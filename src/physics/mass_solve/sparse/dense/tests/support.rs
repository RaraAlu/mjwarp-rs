//! 稠密测试共用输入样例。

use crate as engine;
pub(super) mod input {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/sparse_mass_dense.rs"
    ));
}
