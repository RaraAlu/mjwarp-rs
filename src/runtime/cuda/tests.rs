//! 图参数替换单元测试。

use super::graph::replace_output;

#[test]
fn replaces_only_matching_buffer_arguments() {
    let mut args = [10, 20, 20];
    assert!(replace_output(&mut args, 20, 30));
    assert_eq!(args, [10, 30, 20]);
    assert!(!replace_output(&mut args, 99, 30));
    let mut downstream = [20, 40, 50];
    assert!(replace_output(&mut downstream, 20, 30));
    assert_eq!(downstream, [30, 40, 50]);
    let mut original = [10, 20, 50];
    assert!(replace_output(&mut original, 20, 20));
    assert_eq!(original, [10, 20, 50]);
}
