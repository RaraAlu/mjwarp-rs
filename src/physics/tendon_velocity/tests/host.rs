//! 肌腱速度主机行序与边界测试。

use super::tests_support::*;
use super::*;

#[test]
fn host_keeps_native_row_order_empty_rows_and_source_stride() {
    for nv in [3, 33, 65] {
        let m = model(nv);
        let s = spec(nv, m.rows()).unwrap();
        assert_eq!((s.nt, s.nnz, s.stride), (5, 2 * nv + 2, 8 * nv + 43));
        assert_eq!(&s.meta[..4], &[-131072; 4]);
        assert_eq!(&s.meta[4..9], m.rows().rowadr());
        assert_eq!(&s.meta[9..14], m.rows().rownnz());
        assert_eq!(&s.meta[14..14 + s.nnz], m.rows().colind());
        assert!(m.rows().row(4).unwrap().is_empty());
        for w in [1, 2, 257] {
            assert_eq!(layout(w, nv).unwrap().total_elements(), w * nv);
        }
    }
}
#[test]
fn host_rejects_bad_work_and_bounds_historical_views() {
    for (w, n) in [(0, 3), (1, 0), (usize::MAX, 3), (2, u32::MAX as usize)] {
        assert!(layout(w, n).is_err());
    }
    assert!(spec(0, model(3).rows()).is_err());
    assert!(spec(3, &TendonRows::default()).is_err());
    let l = layout(2, 5).unwrap();
    let s = TendonVelocitySnapshot {
        layout: l,
        values: (0..18).map(|x| x as f32).collect(),
    };
    assert_eq!(s.world(0).unwrap(), [4.0, 5.0, 6.0, 7.0, 8.0]);
    assert_eq!(s.world(1).unwrap(), [9.0, 10.0, 11.0, 12.0, 13.0]);
    assert!(s.world(2).is_err());
}
