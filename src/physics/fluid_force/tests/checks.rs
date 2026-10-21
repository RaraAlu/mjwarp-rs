//! 流体清零工作边界测试。

use super::*;

#[test]
fn checks_fluid_clear_work_bounds() {
    assert_eq!(clear_work(1).unwrap(), 1);
    assert_eq!(
        clear_work((u32::MAX - 255) as usize).unwrap(),
        u32::MAX - 255
    );
    for value in [0, (u32::MAX - 254) as usize, usize::MAX] {
        assert!(matches!(
            clear_work(value),
            Err(InputError::Overflow {
                field: "fluid_force_clear_work"
            })
        ));
    }
}
