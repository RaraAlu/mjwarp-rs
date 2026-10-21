//! 十项独立参数周期与冻结公式对照。

use super::oracle::{oracle, shared};
use super::{compare, fixture, flattened, model, write_cases};
use mjwarp_rs::{
    model::{CamLightParameter as F, CamLightParameters, KinematicsParameters, ParameterBatch},
    physics::KinematicsPlan,
    runtime::TransferSession,
};

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn ten_independent_parameter_periods_match_frozen_formulas() {
    let v = fixture();
    let model = model(&v);
    let fields = model.fields().clone();
    let periods = [2, 3, 5, 7, 11, 3, 5, 7, 2, 11];
    let mut p = CamLightParameters::default();
    for (i, f) in F::ALL.into_iter().enumerate() {
        let base = shared(&fields, f);
        let mut values = vec![];
        for row in 0..periods[i] {
            let mut v = base.to_vec();
            if f == F::CamQuat {
                let a = row as f32 * 0.13;
                for q in v.as_chunks_mut::<4>().0 {
                    q.copy_from_slice(&[a.cos(), 0.0, 0.0, a.sin()]);
                }
            } else {
                for (j, v) in v.iter_mut().enumerate() {
                    *v += row as f32 * 0.17 + j as f32 * 0.003;
                }
            }
            values.extend(v);
        }
        p.set(
            f,
            ParameterBatch::new(periods[i], base.len(), values).unwrap(),
        );
    }
    let session = TransferSession::new(0).unwrap();
    let plan =
        KinematicsPlan::with_camlight(&session, model, KinematicsParameters::default(), p.clone())
            .unwrap();
    assert_eq!(plan.camlight_parameters(), &p);
    for worlds in [1, 2, 5, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        for round in 0..2 {
            write_cases(&mut data, &v, round);
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in 0..worlds {
                eprintln!(
                    "G01-camlight kind=parameters world={w} max_abs_error={:e}",
                    compare(&flattened(&out, w), &oracle(&out, w, &fields, &p))
                );
            }
        }
    }
}
