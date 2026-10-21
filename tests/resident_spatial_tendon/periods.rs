//! 独立参数周期与稀疏力矩测试。

use mjwarp_rs::model::{KinematicsParameter as F, ParameterBatch};

use super::*;

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn independent_field_periods_feed_site_paths_and_sparse_moments() {
    let v = fixture();
    let m = model(&v);
    let s = TransferSession::new(0).unwrap();
    let mut parameters = KinematicsParameters::default();
    let mut site = Vec::new();
    for row in 0..3 {
        let mut values = floats(&v["model"], "site_pos");
        for chunk in values.as_chunks_mut::<3>().0 {
            chunk[0] += row as f32 * 0.125;
        }
        site.extend(values);
    }
    parameters.set(F::SitePos, ParameterBatch::new(3, 27, site).unwrap());
    let mut mass = floats(&v["model"], "body_mass");
    mass.extend(vec![0.0; 7]);
    parameters.set(F::BodyMass, ParameterBatch::new(2, 7, mass).unwrap());
    let mut ipos = Vec::new();
    for row in 0..5 {
        let mut values = floats(&v["model"], "body_ipos");
        for chunk in values[3..].as_chunks_mut::<3>().0 {
            chunk[1] += row as f32 * 0.0625;
        }
        ipos.extend(values);
    }
    parameters.set(F::BodyIpos, ParameterBatch::new(5, 21, ipos).unwrap());
    let a = KinematicsPlan::with_spatial_tendons(
        &s,
        m.clone(),
        parameters,
        CamLightParameters::default(),
    )
    .unwrap();
    for worlds in [1, 2, 5, 513] {
        let mut data = a.create_data(worlds).unwrap();
        for round in 0..2 {
            if round > 0 {
                for w in 0..worlds {
                    write_case(&mut data, w, &v["cases"][(w + 1) % 8]);
                }
            }
            a.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in 0..worlds {
                eprintln!(
                    "G01-spatial-tendon kind=parameters world={w} max_abs_error={:e}",
                    compare(&flattened(&out, w), &oracle(&out, &m, w))
                );
                if w % 2 == 1 {
                    assert_eq!(out.com().world(w).unwrap().subtree_mass, &[0.0; 7]);
                }
            }
        }
    }
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn sparse_moments_match_length_derivatives_and_shared_column_pruning() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let m = model(&v);
    let mut f = m.fields().clone();
    f.ten_j_colind.drain(34..40);
    f.ten_j_rownnz[3] -= 6;
    f.ten_j_rowadr[4] -= 6;
    // Add a legal zero column to the static path.
    f.ten_j_colind.push(0);
    f.ten_j_rownnz[4] = 1;
    let m = SpatialTendonModelInput::new(m.fixed().clone(), f).unwrap();
    let a = KinematicsPlan::with_spatial_tendons(
        &s,
        m.clone(),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = a.create_data(513).unwrap();
    write_case(&mut data, 512, &v["cases"][2]);
    a.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    compare(&flattened(&out, 512), &oracle(&out, &m, 512));
    for (qadr, dof) in [(11, 9), (12, 10), (13, 11)] {
        let mut q = floats(&v["cases"][2], "qpos");
        let old = q[qadr];
        q[qadr] += 0.001;
        data.write_world_qpos(512, &q).unwrap();
        a.update(&mut data).unwrap();
        let next = data.readback().unwrap();
        for t in 0..5 {
            let original = out.spatial_tendon().world(512).unwrap();
            let next = next.spatial_tendon().world(512).unwrap();
            let derivative = (f64::from(next.ten_length[t]) - f64::from(original.ten_length[t]))
                / (f64::from(q[qadr]) - f64::from(old));
            let moment = m
                .rows()
                .row(t)
                .unwrap()
                .binary_search(&dof)
                .map(|slot| original.ten_jacobian[m.rows().rowadr()[t] as usize + slot])
                .unwrap_or(0.0);
            assert!(
                (derivative - f64::from(moment)).abs() < 0.003,
                "t={t} dof={dof} derivative={derivative} moment={moment}"
            );
        }
    }
    assert_eq!(
        *out.spatial_tendon()
            .world(512)
            .unwrap()
            .ten_jacobian
            .last()
            .unwrap(),
        0.0
    );
}
