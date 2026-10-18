//! 借用实际设备肌腱雅可比。
//! 独占速度；不交付完整G02。
use super::{KinematicsData, KinematicsPlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, TendonRows};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
use std::sync::Arc;

struct Spec {
    nv: usize,
    nt: usize,
    nnz: usize,
    stride: usize,
    meta: Vec<i32>,
}
fn spec(nv: usize, rows: &TendonRows) -> Result<Spec, InputError> {
    let nt = rows.ntendon();
    let nnz = rows.nnz();
    if nv == 0 || nt == 0 {
        return Err(InputError::InvalidDimension {
            field: "tendon_velocity_dimensions",
        });
    }
    let overflow = || InputError::Overflow {
        field: "tendon_velocity_metadata",
    };
    let len = nt
        .checked_mul(2)
        .and_then(|n| n.checked_add(nnz))
        .and_then(|n| n.checked_add(8))
        .filter(|&n| n <= i32::MAX as usize)
        .ok_or_else(overflow)?;
    let stride = rows
        .nwrap()
        .checked_mul(6)
        .and_then(|n| n.checked_add(nt))
        .and_then(|n| n.checked_add(nnz))
        .filter(|&n| n <= i32::MAX as usize)
        .ok_or_else(overflow)?;
    let mut meta = Vec::with_capacity(len);
    meta.extend([-131072; 4]);
    meta.extend(rows.rowadr());
    meta.extend(rows.rownnz());
    meta.extend(rows.colind());
    meta.extend([-131072; 4]);
    Ok(Spec {
        nv,
        nt,
        nnz,
        stride,
        meta,
    })
}
fn layout(worlds: usize, width: usize) -> Result<BatchLayout, InputError> {
    if worlds == 0
        || width == 0
        || worlds
            .checked_mul(width)
            .is_none_or(|n| n > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "tendon_velocity_work",
        });
    }
    let l = BatchLayout::new(worlds, width, 4)?;
    l.total_elements()
        .checked_add(8)
        .and_then(|n| n.checked_mul(4))
        .ok_or(InputError::Overflow {
            field: "tendon_velocity_guards",
        })?;
    Ok(l)
}

/// 计划借用实际位置生产者。
pub struct TendonVelocityPlan<'p> {
    source: &'p KinematicsPlan,
    spec: Arc<Spec>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    validate: SynchronousKernel,
    #[cfg(feature = "cuda-probe")]
    compute: SynchronousKernel,
}
/// qvel不借用柔体边状态。
pub struct TendonVelocityData {
    spec: Arc<Spec>,
    velocity: BatchLayout,
    output: BatchLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    qvel: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
/// 视图锁定实际源与速度状态。
pub struct TendonVelocityView<'a, 'p> {
    plan: &'a TendonVelocityPlan<'p>,
    source: &'a KinematicsData,
    data: &'a TendonVelocityData,
}
#[cfg(feature = "cuda-probe")]
pub(super) struct TendonVelocityDeviceView<'a> {
    pub metadata: &'a TransferBuffer<i32>,
    pub source: &'a TransferBuffer<f32>,
    pub velocity: &'a TransferBuffer<f32>,
    pub worlds: usize,
}
/// 独占历史值，不认证未来状态。
#[derive(Clone, Debug)]
pub struct TendonVelocitySnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}
impl TendonVelocitySnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
impl<'p> TendonVelocityPlan<'p> {
    #[cfg(feature = "cuda-probe")]
    pub(super) fn tendon_passive_context(
        &self,
    ) -> (TransferSession, usize, usize, usize, usize, usize) {
        let max_row = self.spec.meta[4 + self.spec.nt..4 + 2 * self.spec.nt]
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
            .max(1) as usize;
        (
            self.session.clone(),
            self.spec.nv,
            self.spec.nt,
            self.spec.nnz,
            self.spec.stride,
            max_row,
        )
    }
    pub fn new(source: &'p KinematicsPlan) -> Result<Self, TransferError> {
        let (session, nv, rows) = source.tendon_velocity_context()?;
        let s = spec(nv, &rows)?;
        layout(1, nv)?;
        layout(1, s.nt)?;
        debug_assert_eq!(s.meta.len(), 8 + 2 * s.nt + s.nnz);
        debug_assert!(s.stride >= s.nt + s.nnz);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, s);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let code = format!(
                "#define TV_NV {nv}\n#define TV_NT {}\n#define TV_NNZ {}\n#define TV_STRIDE {}\n{CUDA}",
                s.nt, s.nnz, s.stride
            );
            let metadata = session.upload(&s.meta)?;
            let validate = SynchronousKernel::compile(&session, &code, "tendon_velocity_validate")?;
            let compute = SynchronousKernel::compile(&session, &code, "tendon_velocity_compute")?;
            Ok(Self {
                source,
                spec: Arc::new(s),
                session,
                metadata,
                validate,
                compute,
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<TendonVelocityData, TransferError> {
        let velocity = layout(worlds, self.spec.nv)?;
        let output = layout(worlds, self.spec.nt)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (velocity, output);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let alloc = |n: usize, value| {
                let mut v = crate::runtime::host_staging::<f32>(n + 8)?;
                v.fill(-131072.0);
                v[4..4 + n].fill(value);
                self.session.upload(&v)
            };
            Ok(TendonVelocityData {
                spec: self.spec.clone(),
                velocity,
                output,
                ready: false,
                qvel: alloc(velocity.total_elements(), 0.0)?,
                current: alloc(output.total_elements(), f32::NAN)?,
                staged: alloc(output.total_elements(), f32::NAN)?,
                status: alloc(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update<'a>(
        &'a self,
        source: &'a KinematicsData,
        data: &'a mut TendonVelocityData,
    ) -> Result<TendonVelocityView<'a, 'p>, TransferError> {
        if !Arc::ptr_eq(&self.spec, &data.spec) {
            return Err(TransferError::ModelMismatch);
        }
        self.source.check_data(source)?;
        if source.worlds() != data.worlds() {
            return Err(TransferError::ModelMismatch);
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let j = self.source.tendon_velocity_source(source)?;
            data.ready = false;
            let mut g = [0.0; 8];
            data.current.read_range_into(0, &mut g[..4])?;
            data.current
                .read_range_into(data.current.len() - 4, &mut g[4..])?;
            crate::physics::check_device_guards(&g)?;
            let w = data.worlds() as u32;
            // SAFETY: 同计划拥有实际源。
            // 静态元数据限定读取跨度。
            // 验证先拒绝坏行列号。
            unsafe {
                self.validate.launch(
                    &self.metadata,
                    j,
                    &data.qvel,
                    &mut data.status,
                    [0, 0, w, w],
                )?;
            }
            let status = read(&data.status, data.worlds(), "tendon_velocity_status")?;
            if let Some(index) = status[4..4 + data.worlds()].iter().position(|&x| x != 1.0) {
                return Err(InputError::InvalidTopology {
                    field: "tendon_velocity_source",
                    index,
                    reason: "guard_mapping_or_nonfinite",
                }
                .into());
            }
            // SAFETY: 每线程只写一项。
            // 验证已覆盖源与全部索引。
            // 源和目标持有同一会话。
            unsafe {
                self.compute.launch(
                    &self.metadata,
                    j,
                    &data.qvel,
                    &mut data.staged,
                    [0, 0, w, w * self.spec.nt as u32],
                )?;
            }
            read(
                &data.staged,
                data.output.total_elements(),
                "tendon_velocity_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            data.ready = true;
            Ok(TendonVelocityView {
                plan: self,
                source,
                data,
            })
        }
    }
}
impl TendonVelocityData {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.velocity.worlds(), self.output.worlds());
        self.velocity.worlds()
    }
    pub fn write_qvel(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(None, values)
    }
    pub fn write_world_qvel(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        self.write(Some(world), values)
    }
    fn write(&mut self, world: Option<usize>, values: &[f32]) -> Result<(), TransferError> {
        let (offset, count) = if let Some(w) = world {
            let r = self.velocity.world_elements(w)?;
            (r.start, r.len())
        } else {
            (0, self.velocity.total_elements())
        };
        if values.len() != count {
            return Err(InputError::LengthMismatch {
                field: "tendon_velocity_qvel",
                expected: count,
                actual: values.len(),
            }
            .into());
        }
        if let Some(index) = values.iter().position(|v| !v.is_finite()) {
            return Err(InputError::NonFinite {
                field: "tendon_velocity_qvel",
                index,
            }
            .into());
        }
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.qvel.write_range(4 + offset, values)
        }
    }
}
impl TendonVelocityView<'_, '_> {
    pub(super) fn belongs_to(&self, plan: &TendonVelocityPlan<'_>) -> bool {
        std::ptr::eq(self.plan, plan)
    }
    pub(super) fn source_data(&self) -> &KinematicsData {
        self.source
    }
    pub(super) fn worlds(&self) -> usize {
        self.data.worlds()
    }
    #[cfg(feature = "cuda-probe")]
    pub(super) fn tendon_passive_device(
        &self,
    ) -> Result<TendonVelocityDeviceView<'_>, TransferError> {
        if !self.data.ready {
            return Err(TransferError::StageNotReady {
                stage: "tendon_velocity",
            });
        }
        self.plan.source.check_data(self.source)?;
        let source = self.plan.source.tendon_velocity_source(self.source)?;
        Ok(TendonVelocityDeviceView {
            metadata: &self.plan.metadata,
            source,
            velocity: &self.data.current,
            worlds: self.data.worlds(),
        })
    }
    pub fn snapshot(&self) -> Result<TendonVelocitySnapshot, TransferError> {
        if !self.data.ready {
            return Err(TransferError::StageNotReady {
                stage: "tendon_velocity",
            });
        }
        self.plan.source.check_data(self.source)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.plan.source.tendon_velocity_source(self.source)?;
            Ok(TendonVelocitySnapshot {
                layout: self.data.output,
                values: read(
                    &self.data.current,
                    self.data.output.total_elements(),
                    "tendon_velocity",
                )?,
            })
        }
    }
}
#[cfg(feature = "cuda-probe")]
fn read(
    buffer: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut v)?;
    crate::physics::check_device_values(&v, field)?;
    Ok(v)
}
// Adapted from frozen forward.py::_tendon_velocity (71da24d), Apache-2.0.
// Copyright 2025 The Newton Developers; LICENSES/Apache-2.0.txt.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void tendon_velocity_validate(const int* meta,const float* jac,const float* v,float* status,unsigned a,unsigned b,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=true;
 for(unsigned k=0;k<4;++k)ok=ok&&meta[k]==-131072&&meta[4+2ull*TV_NT+TV_NNZ+k]==-131072&&jac[k]==-131072.0f&&jac[4+1ull*worlds*TV_STRIDE+k]==-131072.0f&&v[k]==-131072.0f&&v[4+1ull*worlds*TV_NV+k]==-131072.0f;
 for(unsigned t=0;t<TV_NT;++t){int start=meta[4+t],n=meta[4+TV_NT+t];if(start<0||n<0||start>TV_NNZ||n>TV_NNZ-start){ok=false;continue;}for(int i=0;i<n;++i){int c=meta[4+2ull*TV_NT+start+i];if(c<0||c>=TV_NV)ok=false;}}
 for(unsigned i=0;i<TV_NNZ;++i)ok=ok&&isfinite(jac[4+1ull*w*TV_STRIDE+TV_NT+i]);
 for(unsigned i=0;i<TV_NV;++i)ok=ok&&isfinite(v[4+1ull*w*TV_NV+i]);status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void tendon_velocity_compute(const int* meta,const float* jac,const float* v,float* out,unsigned a,unsigned b,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/TV_NT,t=tid%TV_NT;int start=meta[4+t],n=meta[4+TV_NT+t];float velocity=0.0f;
 for(int i=0;i<n;++i){int sparse=start+i;float J=jac[4+1ull*w*TV_STRIDE+TV_NT+sparse];if(J!=0.0f){int col=meta[4+2ull*TV_NT+sparse];velocity+=J*v[4+1ull*w*TV_NV+col];}}
 out[4+tid]=velocity;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, InertialFields,
        InertialModelInput, KinematicFields, KinematicModelInput, MocapModelInput, TendonFields,
        TendonModelInput,
    };
    #[cfg(feature = "cuda-probe")]
    use crate::model::{CamLightParameters, KinematicsParameters, ParameterBatch};

    fn coefficient(j: usize) -> f32 {
        (j as i32 % 5 - 2) as f32 / 8.0
    }
    fn model(nv: usize) -> TendonModelInput {
        let k = KinematicModelInput::new(
            nv,
            KinematicFields {
                qpos0: vec![0.0; nv],
                body_parentid: vec![0; nv + 1],
                body_jntadr: std::iter::once(-1).chain(0..nv as i32).collect(),
                body_jntnum: std::iter::once(0)
                    .chain(std::iter::repeat_n(1, nv))
                    .collect(),
                body_pos: vec![0.0; 3 * (nv + 1)],
                body_quat: [1.0, 0.0, 0.0, 0.0].repeat(nv + 1),
                jnt_type: vec![2; nv],
                jnt_bodyid: (1..=nv as i32).collect(),
                jnt_qposadr: (0..nv as i32).collect(),
                jnt_dofadr: (0..nv as i32).collect(),
                jnt_pos: vec![0.0; 3 * nv],
                jnt_axis: [1.0, 0.0, 0.0].repeat(nv),
            },
        )
        .unwrap();
        let rigid = InertialModelInput::new(
            k,
            InertialFields {
                body_ipos: vec![0.0; 3 * (nv + 1)],
                body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(nv + 1),
                body_mass: std::iter::once(0.0)
                    .chain(std::iter::repeat_n(1.0, nv))
                    .collect(),
                body_inertia: std::iter::once([0.0; 3])
                    .chain(std::iter::repeat_n([1.0; 3], nv))
                    .flatten()
                    .collect(),
                dof_bodyid: (1..=nv as i32).collect(),
                dof_jntid: (0..nv as i32).collect(),
                dof_parentid: vec![-1; nv],
                dof_armature: vec![0.0; nv],
                dof_damping: vec![0.0; nv],
            },
        )
        .unwrap();
        let attached = AttachedModelInput::new(
            rigid,
            AttachedFields {
                site_bodyid: vec![1, 2, 0, 0],
                site_pos: vec![0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0],
                site_quat: [1.0, 0.0, 0.0, 0.0].repeat(4),
                ..Default::default()
            },
        )
        .unwrap();
        let cam = CamLightModelInput::new(
            MocapModelInput::new(attached, vec![-1; nv + 1]).unwrap(),
            CamLightFields::default(),
        )
        .unwrap();
        TendonModelInput::new(
            cam,
            TendonFields {
                tendon_adr: vec![0, 2, (2 + nv) as i32, (3 + nv) as i32, (4 + nv) as i32],
                tendon_num: vec![2, nv as i32, 1, 1, 2],
                wrap_type: [vec![3, 3], vec![1; nv + 2], vec![3, 3]].concat(),
                wrap_objid: [
                    vec![0, 1],
                    (0..nv as i32).collect(),
                    vec![1, nv as i32 - 1],
                    vec![2, 3],
                ]
                .concat(),
                wrap_prm: [
                    vec![0.0; 2],
                    (0..nv).map(coefficient).collect(),
                    vec![0.0, 1.0],
                    vec![0.0; 2],
                ]
                .concat(),
                ten_j_rowadr: vec![
                    0,
                    nv as i32,
                    2 * nv as i32,
                    2 * nv as i32 + 1,
                    2 * nv as i32 + 2,
                ],
                ten_j_rownnz: vec![nv as i32, nv as i32, 1, 1, 0],
                ten_j_colind: [
                    (0..nv as i32).collect::<Vec<_>>(),
                    (0..nv as i32).collect(),
                    vec![1, nv as i32 - 1],
                ]
                .concat(),
            },
        )
        .unwrap()
    }
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
    #[cfg(feature = "cuda-probe")]
    fn producer(session: &TransferSession, nv: usize) -> KinematicsPlan {
        KinematicsPlan::with_tendons(
            session,
            model(nv),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    }
    #[cfg(feature = "cuda-probe")]
    fn raw(b: &TransferBuffer<f32>) -> Vec<f32> {
        let mut v = vec![0.0; b.len()];
        b.read_range_into(0, &mut v).unwrap();
        v
    }
    #[cfg(feature = "cuda-probe")]
    fn bits(v: &[f32]) -> Vec<u32> {
        v.iter().map(|v| v.to_bits()).collect()
    }
    #[cfg(feature = "cuda-probe")]
    fn expected(v: &[f32]) -> Vec<f32> {
        let mut fixed = 0.0_f64;
        for (j, &x) in v.iter().enumerate() {
            fixed += coefficient(j) as f64 * x as f64;
        }
        let values = [
            0.0_f64 - v[0] as f64 + v[1] as f64,
            fixed,
            0.0,
            0.0 + v[v.len() - 1] as f64,
            0.0,
        ];
        values
            .into_iter()
            .map(|x| {
                assert_eq!(x as f32 as f64, x);
                x as f32
            })
            .collect()
    }
    #[cfg(feature = "cuda-probe")]
    fn passive_repeated(period: usize, width: usize, row: &[f32]) -> ParameterBatch {
        assert_eq!(width, row.len());
        ParameterBatch::new(period, width, row.repeat(period)).unwrap()
    }
    #[cfg(feature = "cuda-probe")]
    fn passive_parameters(periods: [usize; 5]) -> crate::physics::TendonPassiveParameters {
        let mut stiffness = vec![0.0; 5];
        stiffness[0] = 0.25;
        let mut stiffness_poly = vec![0.0; 10];
        stiffness_poly[0] = 0.125;
        stiffness_poly[1] = 0.0625;
        let mut damping = vec![0.0; 5];
        damping[0] = 0.125;
        let mut damping_poly = vec![0.0; 10];
        damping_poly[0] = 0.125;
        damping_poly[1] = 0.0625;
        let mut spring = vec![0.0; 10];
        spring[1] = 1.0;
        crate::physics::TendonPassiveParameters {
            stiffness: passive_repeated(periods[0], 5, &stiffness),
            stiffness_poly: passive_repeated(periods[1], 10, &stiffness_poly),
            damping: passive_repeated(periods[2], 5, &damping),
            damping_poly: passive_repeated(periods[3], 10, &damping_poly),
            length_spring: passive_repeated(periods[4], 10, &spring),
        }
    }
    #[cfg(feature = "cuda-probe")]
    fn varying_passive_parameters() -> crate::physics::TendonPassiveParameters {
        let scalar = |period: usize, base: f32| {
            let mut values = Vec::with_capacity(5 * period);
            for row in 0..period {
                let mut field = vec![0.0; 5];
                field[0] = base + row as f32 / 64.0;
                values.extend(field);
            }
            ParameterBatch::new(period, 5, values).unwrap()
        };
        let poly = |period: usize| {
            let mut values = Vec::with_capacity(10 * period);
            for row in 0..period {
                let mut field = vec![0.0; 10];
                field[0] = row as f32 / 64.0;
                values.extend(field);
            }
            ParameterBatch::new(period, 10, values).unwrap()
        };
        let mut lengths = Vec::with_capacity(110);
        for row in 0..11 {
            let mut field = vec![0.0; 10];
            field[1] = 1.0 + row as f32 / 32.0;
            lengths.extend(field);
        }
        crate::physics::TendonPassiveParameters {
            stiffness: scalar(2, 0.25),
            stiffness_poly: poly(3),
            damping: scalar(5, 0.125),
            damping_poly: poly(7),
            length_spring: ParameterBatch::new(11, 10, lengths).unwrap(),
        }
    }
    #[cfg(feature = "cuda-probe")]
    fn empty_row_passive_parameters() -> crate::physics::TendonPassiveParameters {
        let mut stiffness = vec![0.0; 5];
        stiffness[4] = 0.25;
        let mut damping = vec![0.0; 5];
        damping[4] = 0.125;
        crate::physics::TendonPassiveParameters {
            stiffness: ParameterBatch::new(1, 5, stiffness).unwrap(),
            stiffness_poly: ParameterBatch::new(1, 10, vec![0.0; 10]).unwrap(),
            damping: ParameterBatch::new(1, 5, damping).unwrap(),
            damping_poly: ParameterBatch::new(1, 10, vec![0.0; 10]).unwrap(),
            length_spring: ParameterBatch::new(1, 10, vec![0.0; 10]).unwrap(),
        }
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new actual J device consumer, not old tendon/G01 acceptance"]
    fn gpu_connects_real_mixed_jacobian_to_owned_velocities() {
        let session = TransferSession::new(0).unwrap();
        let timer = std::time::Instant::now();
        let (mut batches, mut slots, mut capacity) = (0, 0, 0);
        for nv in [3, 33, 65] {
            let source_plan = producer(&session, nv);
            let plan = TendonVelocityPlan::new(&source_plan).unwrap();
            for worlds in [1, 2, 257] {
                let mut source = source_plan.create_data(worlds).unwrap();
                let mut data = plan.create_data(worlds).unwrap();
                let mut last_snapshot = None;
                assert!(plan.update(&source, &mut data).is_err());
                for pose in 0..2 {
                    let q: Vec<f32> = (0..nv * worlds)
                        .map(|i| ((i / nv + pose) % 3) as f32 / 4.0)
                        .collect();
                    source.write_qpos(&q).unwrap();
                    assert!(plan.update(&source, &mut data).is_err());
                    source_plan.update(&mut source).unwrap();
                    let j = raw(source_plan.tendon_velocity_source(&source).unwrap());
                    for w in 0..worlds {
                        for i in 0..2 * nv + 2 {
                            let e = if i < nv {
                                match i {
                                    0 => -1.0,
                                    1 => 1.0,
                                    _ => 0.0,
                                }
                            } else if i < 2 * nv {
                                coefficient(i - nv)
                            } else if i == 2 * nv {
                                0.0
                            } else {
                                1.0
                            };
                            assert_eq!(j[4 + w * plan.spec.stride + 5 + i], e);
                        }
                    }
                    for vector in 0..2 {
                        let v: Vec<f32> = (0..nv * worlds)
                            .map(|i| {
                                if vector == 1 && i < nv {
                                    -0.0
                                } else {
                                    ((i + pose + vector) % 9) as f32 / 8.0 - 0.5
                                }
                            })
                            .collect();
                        data.write_qvel(&v).unwrap();
                        assert!(!data.ready);
                        let snapshot = plan.update(&source, &mut data).unwrap().snapshot().unwrap();
                        for w in 0..worlds {
                            assert_eq!(
                                bits(snapshot.world(w).unwrap()),
                                bits(&expected(&v[w * nv..(w + 1) * nv]))
                            );
                            slots += 5;
                        }
                        let state = bits(&raw(&data.qvel));
                        let result = bits(&raw(&data.current));
                        let mut bad = v.clone();
                        bad[0] = f32::NAN;
                        assert!(data.write_qvel(&bad).is_err());
                        bad[0] = f32::INFINITY;
                        assert!(data.write_qvel(&bad).is_err());
                        assert!(data.write_world_qvel(worlds, &v[..nv]).is_err());
                        assert!(data.write_qvel(&v[..nv - 1]).is_err());
                        assert!(data.ready);
                        assert_eq!(state, bits(&raw(&data.qvel)));
                        assert_eq!(result, bits(&raw(&data.current)));
                        assert_eq!(snapshot.worlds(), worlds);
                        batches += 1;
                        last_snapshot = Some(snapshot);
                    }
                }
                if nv == 3 && worlds == 2 {
                    let old = bits(&raw(&data.current));
                    let foreign_plan = producer(&session, nv);
                    let foreign = foreign_plan.create_data(worlds).unwrap();
                    assert!(matches!(
                        plan.update(&foreign, &mut data),
                        Err(TransferError::ModelMismatch)
                    ));
                    let wrong = source_plan.create_data(worlds + 1).unwrap();
                    assert!(matches!(
                        plan.update(&wrong, &mut data),
                        Err(TransferError::ModelMismatch)
                    ));
                    let other = TendonVelocityPlan::new(&source_plan).unwrap();
                    let mut other_data = other.create_data(worlds).unwrap();
                    assert!(matches!(
                        plan.update(&source, &mut other_data),
                        Err(TransferError::ModelMismatch)
                    ));
                    assert!(data.ready);
                    assert_eq!(old, bits(&raw(&data.current)));
                    let history = last_snapshot.as_ref().unwrap();
                    let historical_bits = bits(history.world(1).unwrap());
                    data.write_world_qvel(1, &[0.25; 3]).unwrap();
                    assert!(!data.ready);
                    let s = plan.update(&source, &mut data).unwrap().snapshot().unwrap();
                    assert_eq!(bits(s.world(1).unwrap()), bits(&expected(&[0.25; 3])));
                    assert_eq!(historical_bits, bits(history.world(1).unwrap()));
                }
                capacity = capacity.max(
                    4 * (plan.metadata.len()
                        + data.qvel.len()
                        + data.current.len()
                        + data.staged.len()
                        + data.status.len()),
                );
            }
        }
        assert_eq!((batches, slots, capacity), (36, 15600, 78856));
        println!(
            "tendon_velocity batches={batches} slots={slots} bit_difference=0 source_updates=18 extra_world_write_consumer=1 phase_ms={} owned_capacity_bytes={capacity}",
            timer.elapsed().as_millis()
        );
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new consumer guards/mapping/nonfinite/overflow and publication isolation"]
    fn gpu_rejects_corrupted_connection_without_partial_publication() {
        let session = TransferSession::new(0).unwrap();
        let source_plan = producer(&session, 3);
        let mut source = source_plan.create_data(2).unwrap();
        source_plan.update(&mut source).unwrap();
        let mut plan = TendonVelocityPlan::new(&source_plan).unwrap();
        let mut data = plan.create_data(2).unwrap();
        data.write_qvel(&[0.25; 6]).unwrap();
        plan.update(&source, &mut data).unwrap();
        let mut faults = 0;
        for kind in 0..5 {
            let n = match kind {
                0 => data.qvel.len(),
                1 => data.current.len(),
                2 => data.staged.len(),
                3 => data.status.len(),
                _ => source_plan.tendon_velocity_source(&source).unwrap().len(),
            };
            for index in [0, 1, 2, 3, n - 4, n - 3, n - 2, n - 1] {
                let before = bits(&raw(&data.current));
                let b = match kind {
                    0 => &mut data.qvel,
                    1 => &mut data.current,
                    2 => &mut data.staged,
                    3 => &mut data.status,
                    _ => source.tendon_velocity_source_for_test(),
                };
                let value = raw(b)[index];
                b.write_range(index, &[99.0]).unwrap();
                assert!(plan.update(&source, &mut data).is_err());
                assert!(!data.ready);
                let b = match kind {
                    0 => &mut data.qvel,
                    1 => &mut data.current,
                    2 => &mut data.staged,
                    3 => &mut data.status,
                    _ => source.tendon_velocity_source_for_test(),
                };
                b.write_range(index, &[value]).unwrap();
                assert_eq!(before, bits(&raw(&data.current)));
                plan.update(&source, &mut data).unwrap();
                faults += 1;
            }
        }
        let m = plan.metadata.len();
        for index in [0, 1, 2, 3, m - 4, m - 3, m - 2, m - 1, 4, 9, 14] {
            let before = bits(&raw(&data.current));
            let mut old = [0];
            plan.metadata.read_range_into(index, &mut old).unwrap();
            plan.metadata.write_range(index, &[i32::MAX]).unwrap();
            assert!(plan.update(&source, &mut data).is_err());
            assert_eq!(before, bits(&raw(&data.current)));
            plan.metadata.write_range(index, &old).unwrap();
            plan.update(&source, &mut data).unwrap();
            faults += 1;
        }
        for kind in 0..2 {
            let index = if kind == 0 { 4 } else { 9 };
            let before = bits(&raw(&data.current));
            let b = if kind == 0 {
                &mut data.qvel
            } else {
                source.tendon_velocity_source_for_test()
            };
            let old = raw(b)[index];
            b.write_range(index, &[f32::NAN]).unwrap();
            assert!(plan.update(&source, &mut data).is_err());
            assert_eq!(before, bits(&raw(&data.current)));
            let b = if kind == 0 {
                &mut data.qvel
            } else {
                source.tendon_velocity_source_for_test()
            };
            b.write_range(index, &[old]).unwrap();
            plan.update(&source, &mut data).unwrap();
            faults += 1;
        }
        let before = bits(&raw(&data.current));
        let old_j = raw(source_plan.tendon_velocity_source(&source).unwrap())[9];
        source
            .tendon_velocity_source_for_test()
            .write_range(9, &[f32::MAX])
            .unwrap();
        data.qvel.write_range(4, &[2.0]).unwrap();
        assert!(plan.update(&source, &mut data).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        source
            .tendon_velocity_source_for_test()
            .write_range(9, &[old_j])
            .unwrap();
        data.qvel.write_range(4, &[0.25]).unwrap();
        plan.update(&source, &mut data).unwrap();
        faults += 1;
        assert_eq!(faults, 54);
        println!(
            "tendon_velocity_private faults={faults} source_updates=1 no_partial_publication=true recovered=true"
        );
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new tendon passive consumer; does not rerun old tendon velocity acceptance"]
    fn gpu_tendon_passive_consumes_actual_length_jacobian_and_velocity() {
        use crate::physics::{TendonPassiveOptions, TendonPassivePlan};

        let session = TransferSession::new(0).unwrap();
        let source_plan = producer(&session, 3);
        let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
        let passive_plan = TendonPassivePlan::new(
            &velocity_plan,
            passive_parameters([2, 3, 5, 7, 11]),
            TendonPassiveOptions::default(),
        )
        .unwrap();
        let timer = std::time::Instant::now();
        let mut slots = 0;
        for worlds in [1, 2, 257] {
            let mut source = source_plan.create_data(worlds).unwrap();
            let mut velocity = velocity_plan.create_data(worlds).unwrap();
            let mut passive = passive_plan.create_data(worlds).unwrap();
            assert!(velocity_plan.update(&source, &mut velocity).is_err());
            source_plan.update(&mut source).unwrap();
            let mut qvel = vec![0.0; 3 * worlds];
            for row in qvel.chunks_mut(3) {
                row[0] = 0.25;
                row[1] = -0.25;
            }
            velocity.write_qvel(&qvel).unwrap();
            let velocity_view = velocity_plan.update(&source, &mut velocity).unwrap();
            let velocity_snapshot = velocity_view.snapshot().unwrap();
            for world in 0..worlds {
                assert_eq!(velocity_snapshot.world(world).unwrap()[0], -0.5);
            }
            let snapshot = passive_plan
                .update(&velocity_view, &mut passive)
                .unwrap()
                .snapshot()
                .unwrap();
            for world in 0..worlds {
                assert_eq!(
                    bits(snapshot.spring(world).unwrap()),
                    bits(&[0.4375, -0.4375, 0.0])
                );
                assert_eq!(
                    bits(snapshot.damper(world).unwrap()),
                    bits(&[-0.1015625, 0.1015625, 0.0])
                );
                slots += 6;
            }

            let _ = velocity_view;
            let before = bits(snapshot.spring(0).unwrap());
            let source_buffer = source.tendon_velocity_source_for_test();
            let old_length = raw(source_buffer)[4];
            source_buffer.write_range(4, &[f32::NAN]).unwrap();
            let view = velocity_plan.update(&source, &mut velocity).unwrap();
            assert!(passive_plan.update(&view, &mut passive).is_err());
            assert_eq!(before, bits(snapshot.spring(0).unwrap()));
            let _ = view;
            source
                .tendon_velocity_source_for_test()
                .write_range(4, &[old_length])
                .unwrap();
            let view = velocity_plan.update(&source, &mut velocity).unwrap();
            passive_plan.update(&view, &mut passive).unwrap();

            let mut wrong_worlds = passive_plan.create_data(worlds + 1).unwrap();
            assert!(matches!(
                passive_plan.update(&view, &mut wrong_worlds),
                Err(TransferError::ModelMismatch)
            ));

            passive.corrupt_guard_for_test(true, 0, 7.0).unwrap();
            assert!(passive_plan.update(&view, &mut passive).is_err());
            passive.corrupt_guard_for_test(true, 0, -131072.0).unwrap();
            passive_plan.update(&view, &mut passive).unwrap();
            let end = passive.buffer_len_for_test(false) - 1;
            passive.corrupt_guard_for_test(false, end, 7.0).unwrap();
            assert!(passive_plan.update(&view, &mut passive).is_err());
            passive
                .corrupt_guard_for_test(false, end, -131072.0)
                .unwrap();
            passive_plan.update(&view, &mut passive).unwrap();
        }

        let foreign_source_plan = producer(&session, 3);
        let foreign_velocity_plan = TendonVelocityPlan::new(&foreign_source_plan).unwrap();
        let mut foreign_source = foreign_source_plan.create_data(1).unwrap();
        foreign_source_plan.update(&mut foreign_source).unwrap();
        let mut foreign_velocity = foreign_velocity_plan.create_data(1).unwrap();
        let foreign_view = foreign_velocity_plan
            .update(&foreign_source, &mut foreign_velocity)
            .unwrap();
        let mut passive = passive_plan.create_data(1).unwrap();
        assert!(matches!(
            passive_plan.update(&foreign_view, &mut passive),
            Err(TransferError::ModelMismatch)
        ));
        assert_eq!(slots, 1560);
        println!(
            "tendon_passive worlds=260 slots={slots} bit_difference=0 atomic_overlap_unproven=true phase_ms={}",
            timer.elapsed().as_millis()
        );
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new conditional-read coverage for frozen disable and zero-coefficient semantics"]
    fn gpu_tendon_passive_skips_disabled_and_inactive_inputs() {
        use crate::physics::{TendonPassiveOptions, TendonPassivePlan};

        let session = TransferSession::new(0).unwrap();
        let source_plan = producer(&session, 3);
        let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
        let parameters = passive_parameters([2, 3, 5, 7, 11]);
        let active = TendonPassivePlan::new(
            &velocity_plan,
            parameters.clone(),
            TendonPassiveOptions::default(),
        )
        .unwrap();
        let disabled = TendonPassivePlan::new(
            &velocity_plan,
            parameters.clone(),
            TendonPassiveOptions {
                disable_spring: true,
                disable_damper: true,
            },
        )
        .unwrap();
        let damper = TendonPassivePlan::new(
            &velocity_plan,
            parameters.clone(),
            TendonPassiveOptions {
                disable_spring: true,
                disable_damper: false,
            },
        )
        .unwrap();
        let spring = TendonPassivePlan::new(
            &velocity_plan,
            parameters,
            TendonPassiveOptions {
                disable_spring: false,
                disable_damper: true,
            },
        )
        .unwrap();

        let mut source = source_plan.create_data(1).unwrap();
        source_plan.update(&mut source).unwrap();
        let mut velocity = velocity_plan.create_data(1).unwrap();
        velocity.write_qvel(&[0.25, -0.25, 0.0]).unwrap();
        let initial = velocity_plan.update(&source, &mut velocity).unwrap();
        let _ = initial;

        let source_buffer = source.tendon_velocity_source_for_test();
        let raw_source = raw(source_buffer);
        source_buffer.write_range(4, &[f32::NAN, f32::NAN]).unwrap();
        let view = velocity_plan.update(&source, &mut velocity).unwrap();
        let mut disabled_data = disabled.create_data(1).unwrap();
        let zero = disabled
            .update(&view, &mut disabled_data)
            .unwrap()
            .snapshot()
            .unwrap();
        assert_eq!(bits(zero.spring(0).unwrap()), bits(&[0.0; 3]));
        assert_eq!(bits(zero.damper(0).unwrap()), bits(&[0.0; 3]));

        let mut damper_data = damper.create_data(1).unwrap();
        let damper_snapshot = damper
            .update(&view, &mut damper_data)
            .unwrap()
            .snapshot()
            .unwrap();
        assert_eq!(
            bits(damper_snapshot.damper(0).unwrap()),
            bits(&[-0.1015625, 0.1015625, 0.0])
        );
        let mut active_data = active.create_data(1).unwrap();
        assert!(active.update(&view, &mut active_data).is_err());
        let _ = view;

        source
            .tendon_velocity_source_for_test()
            .write_range(4, &[raw_source[4]])
            .unwrap();
        let view = velocity_plan.update(&source, &mut velocity).unwrap();
        active.update(&view, &mut active_data).unwrap();
        let _ = view;
        source
            .tendon_velocity_source_for_test()
            .write_range(5, &[raw_source[5]])
            .unwrap();

        velocity.current.write_range(4, &[f32::NAN]).unwrap();
        let invalid_velocity = TendonVelocityView {
            plan: &velocity_plan,
            source: &source,
            data: &velocity,
        };
        let mut spring_data = spring.create_data(1).unwrap();
        spring.update(&invalid_velocity, &mut spring_data).unwrap();
        assert!(damper.update(&invalid_velocity, &mut damper_data).is_err());
        println!("tendon_passive_conditional disabled=2 inactive_length=1 disabled_velocity=1");
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new varying-period and empty-CSR branch coverage"]
    fn gpu_tendon_passive_maps_varying_periods_and_skips_empty_rows() {
        use crate::physics::{TendonPassiveOptions, TendonPassivePlan};

        let session = TransferSession::new(0).unwrap();
        let source_plan = producer(&session, 3);
        let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
        let varying = TendonPassivePlan::new(
            &velocity_plan,
            varying_passive_parameters(),
            TendonPassiveOptions::default(),
        )
        .unwrap();
        let empty = TendonPassivePlan::new(
            &velocity_plan,
            empty_row_passive_parameters(),
            TendonPassiveOptions::default(),
        )
        .unwrap();
        let worlds = 11;
        let mut source = source_plan.create_data(worlds).unwrap();
        source_plan.update(&mut source).unwrap();
        let mut velocity = velocity_plan.create_data(worlds).unwrap();
        let qvel = [0.25, -0.25, 0.0].repeat(worlds);
        velocity.write_qvel(&qvel).unwrap();
        let view = velocity_plan.update(&source, &mut velocity).unwrap();
        let mut varying_data = varying.create_data(worlds).unwrap();
        let snapshot = varying
            .update(&view, &mut varying_data)
            .unwrap()
            .snapshot()
            .unwrap();
        for world in 0..worlds {
            let x = 1.0_f32 - (world % 11) as f32 / 32.0;
            let stiffness = 0.25_f32 + (world % 2) as f32 / 64.0;
            let stiffness_poly = (world % 3) as f32 / 64.0;
            let spring_force = -x * (stiffness + stiffness_poly * x);
            let damping = 0.125_f32 + (world % 5) as f32 / 64.0;
            let damping_poly = (world % 7) as f32 / 64.0;
            let damper_force = 0.5_f32 * (damping + damping_poly * 0.5_f32);
            assert_eq!(
                bits(snapshot.spring(world).unwrap()),
                bits(&[-spring_force, spring_force, 0.0])
            );
            assert_eq!(
                bits(snapshot.damper(world).unwrap()),
                bits(&[-damper_force, damper_force, 0.0])
            );
        }
        let _ = view;

        source
            .tendon_velocity_source_for_test()
            .write_range(8, &[f32::NAN])
            .unwrap();
        let ready = velocity_plan.update(&source, &mut velocity).unwrap();
        let _ = ready;
        for world in 0..worlds {
            velocity
                .current
                .write_range(4 + world * 5 + 4, &[f32::NAN])
                .unwrap();
        }
        let invalid_empty_values = TendonVelocityView {
            plan: &velocity_plan,
            source: &source,
            data: &velocity,
        };
        let mut empty_data = empty.create_data(worlds).unwrap();
        let empty_snapshot = empty
            .update(&invalid_empty_values, &mut empty_data)
            .unwrap()
            .snapshot()
            .unwrap();
        for world in 0..worlds {
            assert_eq!(bits(empty_snapshot.spring(world).unwrap()), bits(&[0.0; 3]));
            assert_eq!(bits(empty_snapshot.damper(world).unwrap()), bits(&[0.0; 3]));
        }
        println!("tendon_passive_periods worlds=11 periods=2,3,5,7,11 empty_rows=11");
    }
}
