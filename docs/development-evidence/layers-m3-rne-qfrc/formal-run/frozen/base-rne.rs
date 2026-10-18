//! 常驻RNE加速度辅助。
//! 不替代完整G02或等价调度。

use super::{ComVelocityData, ComVelocityPlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

/// 复用空间速度底座的严格辅助。
/// 共享重力；不转换完整Option。
pub struct RnePlan {
    velocity: ComVelocityPlan,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    depth: usize,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 4],
}

/// 独占qacc与RNE工作区。
pub struct RneData {
    velocity: ComVelocityData,
    layout: RneLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    flg_acc: bool,
    #[cfg(feature = "cuda-probe")]
    qacc: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}

#[derive(Clone, Debug)]
pub struct RneSnapshot {
    layout: RneLayout,
    flg_acc: bool,
    values: Vec<f32>,
}

/// 空间向量采用角量在前。
#[derive(Clone, Copy, Debug)]
pub struct RneWorld<'a> {
    pub cacc: &'a [f32],
    pub cfrc_int: &'a [f32],
    /// 跟随上游字段名。
    /// flg_acc=true时包含加速度。
    pub qfrc_bias: &'a [f32],
}

#[derive(Clone, Copy, Debug)]
struct RneLayout {
    output: BatchLayout,
    qacc: BatchLayout,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
}

impl RneLayout {
    fn new(worlds: usize, nbody: usize, nv: usize) -> Result<Self, InputError> {
        let overflow = || InputError::Overflow {
            field: "rne_layout",
        };
        if worlds == 0 || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "rne_dimensions",
            });
        }
        worlds
            .checked_mul(nbody)
            .filter(|&v| v <= (u32::MAX - 255) as usize)
            .ok_or_else(overflow)?;
        let width = nbody
            .checked_mul(12)
            .and_then(|n| n.checked_add(nv))
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let metadata = nbody
            .checked_mul(4)
            .and_then(|n| n.checked_add(nv))
            .and_then(|n| n.checked_add(5))
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, metadata, 4)?;
        let output = BatchLayout::new(worlds, width, 4)?;
        let qacc = BatchLayout::new(worlds, nv, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        let qa = qacc.total_elements().checked_add(8).ok_or_else(overflow)?;
        BatchLayout::new(1, qa, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = guarded;
        Ok(Self {
            output,
            qacc,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

impl RnePlan {
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        gravity: [f32; 3],
    ) -> Result<Self, TransferError> {
        check_values("rne_gravity", 3, &gravity)?;
        let k = model.rigid().kinematics();
        let (nbody, nv) = (k.nbody(), k.nv());
        RneLayout::new(1, nbody, nv)?;
        #[cfg(feature = "cuda-probe")]
        let (metadata, depth) = {
            let f = k.fields();
            let mut m = crate::runtime::host_staging::<i32>(5 + 4 * nbody + nv)?;
            m[0] = nbody as i32;
            m[1] = nv as i32;
            for i in 0..3 {
                m[2 + i] = gravity[i].to_bits() as i32;
            }
            m[5..5 + nbody].copy_from_slice(&f.body_parentid);
            for b in 1..nbody {
                let start = f.body_jntadr[b] as usize;
                let count = f.body_jntnum[b] as usize;
                if count != 0 {
                    m[5 + 2 * nbody + b] = f.jnt_dofadr[start];
                    m[5 + nbody + b] = f.jnt_type[start..start + count]
                        .iter()
                        .map(|&t| match t {
                            0 => 6,
                            1 => 3,
                            _ => 1,
                        })
                        .sum();
                }
                m[5 + 3 * nbody + nv + b] = m[5 + 3 * nbody + nv + f.body_parentid[b] as usize] + 1;
            }
            m[5 + 3 * nbody..5 + 3 * nbody + nv]
                .copy_from_slice(&model.rigid().fields().dof_bodyid);
            let depth = *m[5 + 3 * nbody + nv..].iter().max().unwrap() as usize;
            (session.upload(&m)?, depth)
        };
        let velocity = ComVelocityPlan::new(session, model)?;
        #[cfg(feature = "cuda-probe")]
        let kernels = [
            SynchronousKernel::compile(session, RNE_CUDA, "rne_cacc")?,
            SynchronousKernel::compile(session, RNE_CUDA, "rne_cfrc")?,
            SynchronousKernel::compile(session, RNE_CUDA, "rne_backward")?,
            SynchronousKernel::compile(session, RNE_CUDA, "rne_qfrc")?,
        ];
        Ok(Self {
            velocity,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            depth,
            #[cfg(feature = "cuda-probe")]
            session: session.clone(),
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            kernels,
        })
    }

    pub fn device(&self) -> usize {
        self.velocity.device()
    }

    pub fn create_data(&self, worlds: usize) -> Result<RneData, TransferError> {
        let layout = RneLayout::new(worlds, self.nbody, self.nv)?;
        let velocity = self.velocity.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, velocity);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut qacc = crate::runtime::host_staging::<f32>(layout.qacc.total_elements() + 8)?;
            qacc.fill(-131072.0);
            qacc[4..layout.qacc.total_elements() + 4].fill(0.0);
            let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
            values.fill(-131072.0);
            values[4..layout.guarded - 4].fill(f32::NAN);
            Ok(RneData {
                velocity,
                layout,
                ready: false,
                flg_acc: false,
                qacc: self.session.upload(&qacc)?,
                output: self.session.upload(&values)?,
            })
        }
    }

    pub fn update(&self, data: &mut RneData, flg_acc: bool) -> Result<(), TransferError> {
        self.update_velocity(data)?;
        self.update_rne(data, flg_acc)
    }

    pub fn update_velocity(&self, data: &mut RneData) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        self.velocity.update(&mut data.velocity)
    }

    /// 只消费已就绪的前置结果。
    /// 本严格辅助不隐式补算前置。
    pub fn update_rne(&self, data: &mut RneData, flg_acc: bool) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = flg_acc;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let inputs = data.velocity.rne_inputs()?;
            let worlds = data.worlds() as u32;
            // SAFETY: Validated parent/DOF maps bound every index. Inputs have
            // guarded f32 strides nv, nv, 14*nbody+6*nv, 6*nbody+6*nv.
            // The disjoint output stride is 12*nbody+nv with four guards.
            // The private flag ABI is u32; all launches synchronize one session.
            unsafe {
                self.kernels[0].launch_rne_cacc(
                    &self.metadata,
                    (inputs.qvel, &data.qacc, inputs.com, inputs.velocity),
                    &mut data.output,
                    flg_acc,
                    worlds,
                )?;
                self.kernels[1].launch(
                    &self.metadata,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                    [0, 0, 0, worlds],
                )?;
                // Depth barriers separate atomic reads from descendant writes.
                // Same-depth bodies never have a parent at that depth.
                // Checked worlds*nbody bounds the work-item ABI.
                for depth in (1..=self.depth).rev() {
                    self.kernels[2].launch(
                        &self.metadata,
                        inputs.com,
                        inputs.velocity,
                        &mut data.output,
                        [
                            self.nbody as u32,
                            depth as u32,
                            0,
                            worlds * self.nbody as u32,
                        ],
                    )?;
                }
                self.kernels[3].launch(
                    &self.metadata,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                    [0, 0, 0, worlds],
                )?;
            }
            data.flg_acc = flg_acc;
            data.ready = true;
            Ok(())
        }
    }
}

fn check_values(field: &'static str, expected: usize, values: &[f32]) -> Result<(), InputError> {
    if values.len() != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual: values.len(),
        });
    }
    if let Some(index) = values.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite { field, index });
    }
    Ok(())
}

impl RneData {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }

    fn after_write(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }

    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qpos(values);
        self.after_write(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qpos(world, values);
        self.after_write(result)
    }
    pub fn write_qvel(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qvel(values);
        self.after_write(result)
    }
    pub fn write_world_qvel(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qvel(world, values);
        self.after_write(result)
    }
    pub fn write_qacc(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write_checked_qacc(0, self.layout.qacc.total_elements(), values)
    }
    pub fn write_world_qacc(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let range = self.layout.qacc.world_elements(world)?;
        self.write_checked_qacc(range.start, range.len(), values)
    }
    fn write_checked_qacc(
        &mut self,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        check_values("rne_qacc", expected, values)?;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.qacc.write_range(4 + offset, values)
    }
    pub fn readback(&self) -> Result<RneSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady { stage: "rne" });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.velocity.readback()?;
            let mut qacc =
                crate::runtime::host_staging::<f32>(self.layout.qacc.total_elements() + 8)?;
            self.qacc.read_range_into(0, &mut qacc)?;
            super::check_device_values(&qacc, "rne_qacc")?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            super::check_device_values(&values, "rne_output")?;
            Ok(RneSnapshot {
                layout: self.layout,
                flg_acc: self.flg_acc,
                values,
            })
        }
    }
}

impl RneSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn includes_acceleration(&self) -> bool {
        self.flg_acc
    }
    pub fn world(&self, world: usize) -> Result<RneWorld<'_>, InputError> {
        let range = self.layout.output.world_elements(world)?;
        let v = &self.values[4 + range.start..4 + range.end];
        Ok(RneWorld {
            cacc: &v[..6 * self.nbody()],
            cfrc_int: &v[6 * self.nbody()..12 * self.nbody()],
            qfrc_bias: &v[12 * self.nbody()..],
        })
    }
}

// Adapted from frozen smooth.py RNE and math.py spatial algebra.
// Copyright 2025 The Newton Developers. Apache-2.0.
// See licenses/Apache-2.0.txt for the license and warranty disclaimer.
#[cfg(feature = "cuda-probe")]
const RNE_CUDA: &str = include_str!("rne.cu");

#[cfg(all(test, feature = "cuda-probe"))]
#[path = "../../tests/support/rne_acceleration_frozen.rs"]
mod frozen;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_acceleration_layout_and_input_boundaries() {
        for (w, b, v) in [
            (0, 1, 0),
            (1, 0, 0),
            (usize::MAX, 1, 0),
            (1, usize::MAX, 0),
            (1, 1, usize::MAX),
            (u32::MAX as usize, 2, 0),
        ] {
            assert!(RneLayout::new(w, b, v).is_err());
        }
        let layout = RneLayout::new(513, 1, 0).unwrap();
        assert_eq!(layout.output.total_elements(), 513 * 12);
        assert_eq!(layout.qacc.total_elements(), 0);
        assert!(check_values("test", 2, &[0.0]).is_err());
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(check_values("test", 1, &[value]).is_err());
        }
        assert!(check_values("test", 0, &[]).is_ok());
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn checks_acceleration_guards_and_unpublished_nonfinite_workspaces() {
        let model = super::super::tests::test_model(Some(3));
        let model =
            AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap();
        let session = TransferSession::new(0).unwrap();
        let plan = RnePlan::new(&session, model, [0.0; 3]).unwrap();
        let mut data = plan.create_data(2).unwrap();
        data.write_qacc(&[0.25, -0.5]).unwrap();
        plan.update(&mut data, true).unwrap();
        for index in [0, data.layout.guarded - 1] {
            data.output.write_range(index, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            data.output.write_range(index, &[-131072.0]).unwrap();
        }
        for index in [4, 4 + 6 * data.layout.nbody, 4 + 12 * data.layout.nbody] {
            data.output.write_range(index, &[f32::NAN]).unwrap();
            assert!(data.readback().is_err());
            plan.update_rne(&mut data, true).unwrap();
        }
        for index in [0, data.layout.qacc.total_elements() + 7] {
            data.qacc.write_range(index, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            data.qacc.write_range(index, &[-131072.0]).unwrap();
        }
        data.qacc.write_range(4, &[f32::NAN]).unwrap();
        assert!(data.readback().is_err());
        data.write_world_qacc(0, &[0.25]).unwrap();
        plan.update_rne(&mut data, true).unwrap();
        data.readback().unwrap();
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and pinned frozen smooth PTX"]
    fn diagnoses_new_qacc_stage_against_frozen_ptx_on_identical_device_inputs() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let base: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("fixtures/rne-bias/mixed-joints.json")).unwrap(),
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("fixtures/rne-acceleration/mixed-joints.json"))
                .unwrap(),
        )
        .unwrap();
        let model = crate::physics::rne::tests::reference_model(&base);
        let session = TransferSession::new(0).unwrap();
        let mut rows = Vec::new();
        for (state, g, gravity) in [(3, 0, [0.0, 0.0, -9.81]), (7, 2, [1.25, -2.0, 3.5])] {
            let plan = RnePlan::new(&session, model.clone(), gravity).unwrap();
            let mut data = plan.create_data(1).unwrap();
            let values = |key: &str| {
                v["cases"][state][key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| n.as_f64().unwrap() as f32)
                    .collect::<Vec<_>>()
            };
            data.write_qpos(&values("qpos")).unwrap();
            data.write_qvel(&values("qvel")).unwrap();
            data.write_qacc(&values("qacc")).unwrap();
            plan.update_velocity(&mut data).unwrap();
            for flag in [false, true] {
                plan.update_rne(&mut data, flag).unwrap();
                let candidate = data.readback().unwrap();
                let inputs = data.velocity.rne_inputs().unwrap();
                let mut m = vec![0; 5 + 4 * plan.nbody + plan.nv];
                plan.metadata.read_range_into(0, &mut m).unwrap();
                let mut qvel = vec![0.0; plan.nv];
                let mut qacc = vec![0.0; plan.nv];
                let mut com = vec![0.0; 14 * plan.nbody + 6 * plan.nv];
                let mut velocity = vec![0.0; 6 * plan.nbody + 6 * plan.nv];
                inputs.qvel.read_range_into(4, &mut qvel).unwrap();
                data.qacc.read_range_into(4, &mut qacc).unwrap();
                inputs.com.read_range_into(4, &mut com).unwrap();
                inputs.velocity.read_range_into(4, &mut velocity).unwrap();
                let frozen = frozen::run(&m, &qvel, &qacc, &com, &velocity, flag);
                let actual = &candidate.values[4..candidate.values.len() - 4];
                let differences: Vec<_> = actual.iter().zip(&frozen).enumerate().filter(|(_, (a, b))| a.to_bits() != b.to_bits())
                .map(|(i, (&a, &b))| serde_json::json!({"index":i,"candidate":a,"frozen_stage":b,"candidate_bits":a.to_bits(),"frozen_bits":b.to_bits()})).collect();
                let native = &v["cases"][state]["qfrc"][usize::from(flag)][g];
                rows.push(serde_json::json!({"state":state,"gravity":g,"flag":flag,"differences":differences,"candidate_qfrc":candidate.world(0).unwrap().qfrc_bias,
                "frozen_stage_qfrc":&frozen[12 * plan.nbody..],"native_qfrc":native}));
            }
        }
        let result = serde_json::json!({"scope":"new qacc first-difference; frozen RNE consumes identical product-prefix f32 inputs; not full upstream pipeline or admission", "model":"mixed-joints","rows":rows});
        std::fs::write(
            root.join("target/rne-acceleration-validation/frozen-first-difference.json"),
            serde_json::to_string_pretty(&result).unwrap(),
        )
        .unwrap();
        eprintln!("new qacc frozen stage diagnostic: {}", result);
        assert!(
            rows.iter()
                .all(|row| row["differences"].as_array().unwrap().is_empty()),
            "local RNE stage bit differences remain"
        );
    }

    #[cfg(feature = "cuda-probe")]
    fn reference_model(v: &serde_json::Value) -> AttachedModelInput {
        use crate::model::{
            InertialFields, InertialModelInput, KinematicFields, KinematicModelInput,
        };
        let m = &v["model"];
        let f = |key: &str| {
            m[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect()
        };
        let i = |key: &str| {
            m[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
                .collect()
        };
        let k = KinematicModelInput::new(
            v["nv"].as_u64().unwrap() as usize,
            KinematicFields {
                qpos0: f("qpos0"),
                body_parentid: i("body_parentid"),
                body_jntadr: i("body_jntadr"),
                body_jntnum: i("body_jntnum"),
                body_pos: f("body_pos"),
                body_quat: f("body_quat"),
                jnt_type: i("jnt_type"),
                jnt_bodyid: i("jnt_bodyid"),
                jnt_qposadr: i("jnt_qposadr"),
                jnt_dofadr: i("jnt_dofadr"),
                jnt_pos: f("jnt_pos"),
                jnt_axis: f("jnt_axis"),
            },
        )
        .unwrap();
        let model = InertialModelInput::new(
            k,
            InertialFields {
                body_ipos: f("body_ipos"),
                body_iquat: f("body_iquat"),
                body_mass: f("body_mass"),
                body_inertia: f("body_inertia"),
                dof_bodyid: i("dof_bodyid"),
                dof_jntid: i("dof_jntid"),
                dof_parentid: i("dof_parentid"),
                dof_armature: f("dof_armature"),
                dof_damping: f("dof_damping"),
            },
        )
        .unwrap();
        AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap()
    }
}
