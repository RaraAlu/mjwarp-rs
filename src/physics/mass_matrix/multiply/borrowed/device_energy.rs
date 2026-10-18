//! 严格当前态Mqvel视图接入默认32线程动能。
//! GPU内成对复制，不采用宿主物理回传。
use super::{MassMatrixMultiplyPlan, MassMatrixMultiplyView, Spec};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::BatchLayout;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
use std::sync::Arc;

struct EnergySpec {
    mass: Arc<Spec>,
}
#[derive(Clone, Copy)]
struct Layout {
    pair: BatchLayout,
    energy: BatchLayout,
}
fn layout(n: usize, worlds: usize) -> Result<Layout, InputError> {
    if !(1..=1024).contains(&n) || worlds == 0 || worlds > i32::MAX as usize {
        return Err(InputError::InvalidDimension {
            field: "device_energy_layout",
        });
    }
    let pair = BatchLayout::new(worlds, 2 * n, 4)?;
    let energy = BatchLayout::new(worlds, 2, 4)?;
    for count in [pair.total_elements(), energy.total_elements(), worlds] {
        count
            .checked_add(8)
            .and_then(|v| v.checked_mul(4))
            .ok_or(InputError::Overflow {
                field: "device_energy_guards",
            })?;
    }
    Ok(Layout { pair, energy })
}
fn binding(plan: &Arc<EnergySpec>, data: &Arc<EnergySpec>, mass: &Arc<Spec>) -> bool {
    Arc::ptr_eq(plan, data) && Arc::ptr_eq(&plan.mass, mass)
}
fn all_current(flags: &[bool]) -> Result<(), TransferError> {
    if flags.is_empty() || flags.iter().any(|&v| !v) {
        return Err(TransferError::StageNotReady {
            stage: "device_energy_all_current",
        });
    }
    Ok(())
}
pub struct MassMatrixEnergyPlan<'plan, 'matrix> {
    multiply: &'plan MassMatrixMultiplyPlan<'matrix>,
    spec: Arc<EnergySpec>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 3],
}
pub struct MassMatrixEnergyData {
    spec: Arc<EnergySpec>,
    layout: Layout,
    #[cfg(feature = "cuda-probe")]
    pair: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
/// 同时借用原乘积视图与能量数据，不认证未来状态。
/// 当前态证明仅覆盖动能；势能由调用方提供。
pub struct MassMatrixEnergyView<'energy, 'source> {
    input: &'energy MassMatrixMultiplyView<'source>,
    data: &'energy MassMatrixEnergyData,
}
#[derive(Clone, Debug)]
pub struct MassMatrixEnergySnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}
impl MassMatrixEnergySnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    /// 返回历史[势能,动能]，不携带伪造状态标签。
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
impl MassMatrixEnergyView<'_, '_> {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.input.worlds(), self.data.worlds());
        self.input.worlds()
    }
    pub fn snapshot(&self) -> Result<MassMatrixEnergySnapshot, TransferError> {
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(MassMatrixEnergySnapshot {
                layout: self.data.layout.energy,
                values: super::read(
                    &self.data.current,
                    self.data.layout.energy.total_elements(),
                    "device_energy",
                )?,
            })
        }
    }
}
impl<'plan, 'matrix> MassMatrixEnergyPlan<'plan, 'matrix> {
    pub fn new(multiply: &'plan MassMatrixMultiplyPlan<'matrix>) -> Result<Self, TransferError> {
        let n = multiply.spec.nv;
        layout(n, 1)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let session = multiply.session.clone();
            let source = format!("#define ENERGY_NV {n}\n{SOURCE_VALIDATE}");
            // Exactly the original scalar CUDA string and prefix, not a
            // rewritten reduction or a wider floating-point implementation.
            let scalar = format!(
                "#define ENERGY_NV {n}\n{}",
                crate::physics::kinetic_energy::CUDA
            );
            Ok(Self {
                multiply,
                spec: Arc::new(EnergySpec {
                    mass: multiply.spec.clone(),
                }),
                metadata: session.upload(&[n as i32])?,
                kernels: [
                    SynchronousKernel::compile(&session, &source, "device_energy_source_validate")?,
                    SynchronousKernel::compile(&session, &scalar, "kinetic_energy_validate")?,
                    SynchronousKernel::compile(&session, &scalar, "kinetic_energy_reduce32")?,
                ],
                session,
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<MassMatrixEnergyData, TransferError> {
        let l = layout(self.spec.mass.nv, worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = l;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let alloc = |count: usize, initial| {
                let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
                v.fill(-131072.0);
                v[4..4 + count].fill(initial);
                self.session.upload(&v)
            };
            let mut current = alloc(l.energy.total_elements(), f32::NAN)?;
            for w in 0..worlds {
                current.write_range(4 + 2 * w, &[0.0])?;
            }
            Ok(MassMatrixEnergyData {
                spec: self.spec.clone(),
                layout: l,
                pair: alloc(l.pair.total_elements(), 0.0)?,
                current,
                staged: alloc(l.energy.total_elements(), f32::NAN)?,
                status: alloc(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update<'energy, 'source>(
        &self,
        data: &'energy mut MassMatrixEnergyData,
        input: &'energy MassMatrixMultiplyView<'source>,
    ) -> Result<MassMatrixEnergyView<'energy, 'source>, TransferError> {
        if !binding(&self.spec, &data.spec, &input.data.spec) {
            return Err(TransferError::ModelMismatch);
        }
        self.multiply.matrix.check_data(input.source)?;
        if input.worlds() != data.worlds() || input.computed.len() != data.worlds() {
            return Err(InputError::InvalidDimension {
                field: "device_energy_worlds",
            }
            .into());
        }
        all_current(&input.computed)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            // Recheck the actual source ticket; historical solver or energy
            // snapshots never substitute for the live matrix/source borrow.
            input.source.matrix_input()?;
            let n = self.spec.mass.nv;
            let worlds = data.worlds();
            if input.data.vector.len() != worlds * n + 8
                || input.data.current.len() != worlds * n + 8
            {
                return Err(TransferError::ModelMismatch);
            }
            let dims = [n as u32, worlds as u32, 0, worlds as u32];
            // SAFETY: Actual parent-token identity and exact vector capacities
            // bind both readonly inputs to guarded worlds*nv allocations.
            unsafe {
                self.kernels[0].launch(
                    &self.metadata,
                    &input.data.vector,
                    &input.data.current,
                    &mut data.status,
                    dims,
                )?;
            }
            status(&data.status, worlds)?;
            // These are explicit GPU-to-GPU copies, not a zero-copy pair and
            // never a download/upload round trip of physical values.
            for w in 0..worlds {
                data.pair
                    .copy_range_from(4 + 2 * w * n, &input.data.vector, 4 + w * n, n)?;
                data.pair.copy_range_from(
                    4 + (2 * w + 1) * n,
                    &input.data.current,
                    4 + w * n,
                    n,
                )?;
            }
            // SAFETY: Metadata1, paired worlds*2nv, current worlds*2 and status
            // worlds have the original guarded scalar-validator capacities.
            unsafe {
                self.kernels[1].launch(
                    &self.metadata,
                    &data.pair,
                    &data.current,
                    &mut data.status,
                    dims,
                )?;
            }
            status(&data.status, worlds)?;
            // SAFETY: Original32 ABI checks all sessions and exact capacities;
            // all lanes reach ballot and only lane0 publishes each world.
            unsafe {
                self.kernels[2].launch_kinetic_energy32(
                    &self.metadata,
                    &data.pair,
                    &data.current,
                    &mut data.staged,
                    dims,
                )?;
            }
            super::read(
                &data.staged,
                data.layout.energy.total_elements(),
                "device_energy_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            Ok(MassMatrixEnergyView { input, data })
        }
    }
}
impl MassMatrixEnergyData {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.layout.pair.worlds(), self.layout.energy.worlds());
        self.layout.energy.worlds()
    }
    pub fn write_potential(&mut self, values: &[f32]) -> Result<(), TransferError> {
        super::super::checked(values, self.worlds(), "device_energy_potential")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            for (w, &v) in values.iter().enumerate() {
                self.current.write_range(4 + 2 * w, &[v])?;
            }
            Ok(())
        }
    }
    pub fn write_world_potential(&mut self, world: usize, value: f32) -> Result<(), TransferError> {
        self.layout.energy.world_elements(world)?;
        super::super::checked(&[value], 1, "device_energy_potential")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.current.write_range(4 + 2 * world, &[value])
        }
    }
}
#[cfg(feature = "cuda-probe")]
fn status(buffer: &TransferBuffer<f32>, worlds: usize) -> Result<(), TransferError> {
    let values = super::read(buffer, worlds, "device_energy_status")?;
    if let Some(index) = values[4..4 + worlds].iter().position(|&v| v != 1.0) {
        return Err(InputError::InvalidTopology {
            field: "device_energy_source",
            index,
            reason: "nonfinite_guard_or_shape",
        }
        .into());
    }
    Ok(())
}
#[cfg(feature = "cuda-probe")]
const SOURCE_VALIDATE: &str = r#"
extern "C" __global__ void device_energy_source_validate(const int* meta,const float* q,const float* mq,float* status,
 unsigned n,unsigned worlds,unsigned unused,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 bool valid=n==ENERGY_NV&&meta[0]==ENERGY_NV;
 for(unsigned j=0;j<4;++j)valid=valid&&q[j]==-131072.0f&&mq[j]==-131072.0f&&q[4+1ull*n*worlds+j]==-131072.0f&&mq[4+1ull*n*worlds+j]==-131072.0f;
 for(unsigned j=0;j<n;++j)valid=valid&&isfinite(q[4+1ull*w*n+j])&&isfinite(mq[4+1ull*w*n+j]);
 status[4+w]=valid?1.0f:0.0f;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_checks_pair_world_strides_and_original_scalar32_limits() {
        for n in [1, 3, 33, 65, 1024] {
            for w in [1, 2, 257] {
                let l = layout(n, w).unwrap();
                assert_eq!(l.pair.total_elements(), 2 * n * w);
                assert_eq!(l.energy.total_elements(), 2 * w);
                assert!(l.energy.world_elements(w).is_err());
            }
        }
        for (n, w) in [
            (0, 1),
            (1025, 1),
            (3, 0),
            (3, usize::MAX),
            (3, i32::MAX as usize + 1),
        ] {
            assert!(layout(n, w).is_err());
        }
    }
    #[test]
    fn host_requires_actual_owner_arcs_and_every_current_world() {
        let mass = Arc::new(Spec {
            nbody: 4,
            nv: 3,
            meta: vec![],
        });
        let plan = Arc::new(EnergySpec { mass: mass.clone() });
        let other = Arc::new(EnergySpec { mass: mass.clone() });
        let twin = Arc::new(Spec {
            nbody: 4,
            nv: 3,
            meta: vec![],
        });
        assert!(binding(&plan, &plan, &mass));
        assert!(!binding(&plan, &other, &mass));
        assert!(!binding(&plan, &plan, &twin));
        for flags in [
            &[][..],
            &[false][..],
            &[true, false][..],
            &[false, true][..],
        ] {
            assert!(all_current(flags).is_err());
        }
        all_current(&[true]).unwrap();
        all_current(&[true; 257]).unwrap();
    }
    #[cfg(feature = "cuda-probe")]
    fn raw(buffer: &TransferBuffer<f32>) -> Vec<f32> {
        let mut v = vec![0.0; buffer.len()];
        buffer.read_range_into(0, &mut v).unwrap();
        v
    }
    #[cfg(feature = "cuda-probe")]
    fn bits(values: &[f32]) -> Vec<u32> {
        values.iter().map(|v| v.to_bits()).collect()
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new device-only liveM->Mqvel->energy connection, independent onehot exact bits; no old selector replay"]
    fn gpu_links_live_matrix_energy_with_exact_one_hot_and_device_pair_bits() {
        use super::super::tests::{input, model};
        use crate::physics::{MassMatrixPlan, MassMultiplyFormat};
        let session = TransferSession::new(0).unwrap();
        let start = std::time::Instant::now();
        let mut sources = 0;
        let mut products = 0;
        let mut batches = 0;
        let mut worlds_checked = 0;
        let mut pair_slots = 0;
        let mut capacity = 0;
        let mut last = None;
        for n in [3, 33, 65] {
            let i = input(n, true);
            let producer = MassMatrixPlan::new(&session, model(&i)).unwrap();
            let multiply = [
                MassMatrixMultiplyPlan::new(&producer, i.clone(), MassMultiplyFormat::Sparse)
                    .unwrap(),
                MassMatrixMultiplyPlan::new(&producer, i, MassMultiplyFormat::Dense).unwrap(),
            ];
            let plans = [
                MassMatrixEnergyPlan::new(&multiply[0]).unwrap(),
                MassMatrixEnergyPlan::new(&multiply[1]).unwrap(),
            ];
            for worlds in [1, 2, 257] {
                let mut source = producer.create_data(worlds).unwrap();
                let mut md = [
                    multiply[0].create_data(worlds).unwrap(),
                    multiply[1].create_data(worlds).unwrap(),
                ];
                let mut ed = [
                    plans[0].create_data(worlds).unwrap(),
                    plans[1].create_data(worlds).unwrap(),
                ];
                for state in 0..2 {
                    let pose: Vec<f32> = (0..worlds * n)
                        .map(|j| ((j + state) % 9) as f32 * 0.07 - 0.28)
                        .collect();
                    source.write_qpos(&pose).unwrap();
                    producer.update(&mut source).unwrap();
                    sources += 1;
                    let matrix = source.readback().unwrap();
                    if n == 3 && worlds == 2 && state == 0 {
                        for f in 0..2 {
                            let view = multiply[f]
                                .update(&mut md[f], &source, Some(&[true, true]))
                                .unwrap();
                            products += 1;
                            let before = bits(&raw(&ed[f].current));
                            assert!(plans[f].update(&mut ed[f], &view).is_err());
                            assert_eq!(before, bits(&raw(&ed[f].current)));
                        }
                    }
                    for vector in 0..2 {
                        let mut q = vec![0.0; worlds * n];
                        let hot: Vec<usize> =
                            (0..worlds).map(|w| (w + vector + state) % n).collect();
                        if vector == 1 {
                            for (w, &h) in hot.iter().enumerate() {
                                assert_ne!(h, (w + state) % n);
                            }
                        }
                        for w in 0..worlds {
                            q[w * n + hot[w]] = 1.0;
                        }
                        let p: Vec<f32> = (0..worlds)
                            .map(|w| {
                                if (w + state) % 7 == 0 {
                                    -0.0
                                } else {
                                    ((w + vector + state) % 19) as f32 * 0.25
                                }
                            })
                            .collect();
                        for f in 0..2 {
                            md[f].write_vector(&q).unwrap();
                            let mv = multiply[f].update(&mut md[f], &source, None).unwrap();
                            products += 1;
                            ed[f].write_potential(&p).unwrap();
                            let energy = plans[f].update(&mut ed[f], &mv).unwrap();
                            assert_eq!(energy.worlds(), worlds);
                            let out = energy.snapshot().unwrap();
                            assert_eq!(out.worlds(), worlds);
                            assert!(out.world(worlds).is_err());
                            let packed = raw(&energy.data.pair);
                            let mq = raw(&mv.data.current);
                            let srcq = raw(&mv.data.vector);
                            for w in 0..worlds {
                                assert_eq!(
                                    bits(&packed[4 + 2 * w * n..4 + (2 * w + 1) * n]),
                                    bits(&srcq[4 + w * n..4 + (w + 1) * n])
                                );
                                assert_eq!(
                                    bits(&packed[4 + (2 * w + 1) * n..4 + 2 * (w + 1) * n]),
                                    bits(&mq[4 + w * n..4 + (w + 1) * n])
                                );
                                pair_slots += 2 * n;
                                let diagonal = matrix.world(w).unwrap().matrix[hot[w] * n + hot[w]];
                                let word = diagonal.to_bits();
                                assert!(
                                    diagonal > 0.0
                                        && diagonal.is_normal()
                                        && ((word >> 23) & 255) > 1
                                );
                                // Independent binary-exponent certificate for
                                // onehot q: half the source diagonal, no energy
                                // candidate or Mqvel feeds this expected word.
                                let expected = word - 0x0080_0000;
                                assert_eq!(out.world(w).unwrap()[1].to_bits(), expected);
                                assert_eq!(out.world(w).unwrap()[0].to_bits(), p[w].to_bits());
                            }
                            let _ = energy;
                            if n == 3 && worlds == 2 && state == 0 && vector == 0 {
                                let mut wrong = plans[f].create_data(1).unwrap();
                                assert!(plans[f].update(&mut wrong, &mv).is_err());
                                let foreign = MassMatrixEnergyPlan::new(&multiply[f]).unwrap();
                                let before = bits(&raw(&ed[f].current));
                                assert!(foreign.update(&mut ed[f], &mv).is_err());
                                assert_eq!(before, bits(&raw(&ed[f].current)));
                                if f == 1 {
                                    let before = bits(&raw(&ed[0].current));
                                    assert!(plans[0].update(&mut ed[0], &mv).is_err());
                                    assert_eq!(before, bits(&raw(&ed[0].current)));
                                }
                            }
                            capacity = capacity.max(
                                4 * (ed[f].pair.len()
                                    + ed[f].current.len()
                                    + ed[f].staged.len()
                                    + ed[f].status.len()
                                    + plans[f].metadata.len()),
                            );
                            last = Some(out);
                            batches += 1;
                            worlds_checked += worlds;
                        }
                    }
                }
                if n == 3 && worlds == 2 {
                    for f in 0..2 {
                        md[f].write_world_vector(1, &[0.25; 3]).unwrap();
                        let mv = multiply[f]
                            .update(&mut md[f], &source, Some(&[true, false]))
                            .unwrap();
                        products += 1;
                        let before = bits(&raw(&ed[f].current));
                        assert!(plans[f].update(&mut ed[f], &mv).is_err());
                        assert_eq!(before, bits(&raw(&ed[f].current)));
                        drop(mv);
                        md[f].write_vector(&[-0.0; 6]).unwrap();
                        let mv = multiply[f].update(&mut md[f], &source, None).unwrap();
                        products += 1;
                        let out = plans[f]
                            .update(&mut ed[f], &mv)
                            .unwrap()
                            .snapshot()
                            .unwrap();
                        for w in 0..2 {
                            assert_eq!(out.world(w).unwrap()[1].to_bits(), 0x80000000);
                        }
                    }
                }
            }
        }
        drop(session);
        assert!(
            last.unwrap()
                .world(256)
                .unwrap()
                .iter()
                .all(|v| v.is_finite())
        );
        assert_eq!(
            (batches, worlds_checked, sources, products),
            (72, 6240, 18, 78)
        );
        assert_eq!(pair_slots, 420160);
        assert_eq!(capacity, 138912);
        println!(
            "device_energy batches={batches} worlds={worlds_checked} bit_difference=0 pair_bit_slots={pair_slots} negative_zero_worlds=4 source_dependency_updates={sources} Mqvel_dependency_updates={products} phase_ms={} energy_owned_capacity_bytes={capacity} pair_D2D_copies_per_update=2W M_host_feedback=0",
            start.elapsed().as_millis()
        );
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new device-energy private raw-source/pair/output guards and finite/overflow transactional faults"]
    fn gpu_rejects_corrupted_device_inputs_and_stale_source_without_partial_energy() {
        use super::super::tests::{input, model};
        use crate::physics::{MassMatrixPlan, MassMultiplyFormat};
        let s = TransferSession::new(0).unwrap();
        let i = input(3, true);
        let producer = MassMatrixPlan::new(&s, model(&i)).unwrap();
        let mut source = producer.create_data(2).unwrap();
        producer.update(&mut source).unwrap();
        let multiply =
            MassMatrixMultiplyPlan::new(&producer, i, MassMultiplyFormat::Sparse).unwrap();
        let mut md = multiply.create_data(2).unwrap();
        md.write_vector(&[0.5; 6]).unwrap();
        drop(multiply.update(&mut md, &source, None).unwrap());
        let mut plan = MassMatrixEnergyPlan::new(&multiply).unwrap();
        let mut data = plan.create_data(2).unwrap();
        // Test-only forged tickets let fault injection emulate corrupted GPU
        // buffers without rerunning a frozen producer or claiming that public
        // callers can construct/forge the sealed current-view provenance.
        fn ticket<'a>(
            source: &'a crate::physics::MassMatrixData,
            data: &'a super::super::MassMatrixMultiplyData,
        ) -> MassMatrixMultiplyView<'a> {
            MassMatrixMultiplyView {
                source,
                data,
                computed: vec![true; 2],
            }
        }
        let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
        let mut faults = 0;
        for field in 0..4 {
            for edge in 0..8 {
                let b = match field {
                    0 => &mut data.pair,
                    1 => &mut data.current,
                    2 => &mut data.staged,
                    _ => &mut data.status,
                };
                let index = if edge < 4 { edge } else { b.len() - 8 + edge };
                b.write_range(index, &[99.0]).unwrap();
                let before = bits(&raw(&data.current));
                assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
                assert_eq!(before, bits(&raw(&data.current)));
                let b = match field {
                    0 => &mut data.pair,
                    1 => &mut data.current,
                    2 => &mut data.staged,
                    _ => &mut data.status,
                };
                b.write_range(index, &[-131072.0]).unwrap();
                let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
                faults += 1;
            }
        }
        for field in 0..2 {
            for edge in 0..8 {
                let b = if field == 0 {
                    &mut md.vector
                } else {
                    &mut md.current
                };
                let index = if edge < 4 { edge } else { b.len() - 8 + edge };
                b.write_range(index, &[99.0]).unwrap();
                let before = bits(&raw(&data.current));
                assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
                assert_eq!(before, bits(&raw(&data.current)));
                let b = if field == 0 {
                    &mut md.vector
                } else {
                    &mut md.current
                };
                b.write_range(index, &[-131072.0]).unwrap();
                let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
                faults += 1;
            }
        }
        for (field, index, bad) in [
            (0, 4, f32::NAN),
            (0, 9, f32::INFINITY),
            (1, 4, f32::NAN),
            (1, 9, f32::INFINITY),
        ] {
            let b = if field == 0 {
                &mut md.vector
            } else {
                &mut md.current
            };
            let old = raw(b)[index];
            b.write_range(index, &[bad]).unwrap();
            let before = bits(&raw(&data.current));
            assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
            assert_eq!(before, bits(&raw(&data.current)));
            let b = if field == 0 {
                &mut md.vector
            } else {
                &mut md.current
            };
            b.write_range(index, &[old]).unwrap();
            let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
            faults += 1;
        }
        plan.metadata.write_range(0, &[0]).unwrap();
        let before = bits(&raw(&data.current));
        assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        plan.metadata.write_range(0, &[3]).unwrap();
        let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
        faults += 1;
        let old = raw(&data.current)[4];
        data.current.write_range(4, &[f32::NAN]).unwrap();
        let before = bits(&raw(&data.current));
        assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        data.current.write_range(4, &[old]).unwrap();
        faults += 1;
        data.current.write_range(5, &[f32::NAN]).unwrap();
        data.pair.write_range(4, &[f32::NAN]).unwrap();
        let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
        let q = raw(&md.vector);
        let mq = raw(&md.current);
        md.vector.write_range(7, &[f32::MAX]).unwrap();
        md.current.write_range(7, &[f32::MAX]).unwrap();
        let before = bits(&raw(&data.current));
        assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        md.vector.write_range(7, &q[7..8]).unwrap();
        md.current.write_range(7, &mq[7..8]).unwrap();
        let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
        faults += 1;
        let before = bits(&raw(&data.current));
        assert!(data.write_potential(&[f32::NAN; 2]).is_err());
        assert!(data.write_world_potential(2, 1.0).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        source.write_qpos(&[0.0; 6]).unwrap();
        let stale = MassMatrixMultiplyView {
            source: &source,
            data: &md,
            computed: vec![true; 2],
        };
        assert!(plan.update(&mut data, &stale).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        faults += 1;
        assert_eq!(faults, 56);
        println!(
            "device_energy_private faults={faults} source_dependency_updates=1 Mqvel_dependency_updates=1 no_partial_publication=true cold_source_rejected=true"
        );
    }
}
