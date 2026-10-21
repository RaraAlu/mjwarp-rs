//! Small测试共用输入样例。

use crate as engine;
pub(super) mod input {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/small_mass_prefix.rs"
    ));
}
