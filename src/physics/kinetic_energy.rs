//! 冻结默认32线程动能归约子集。
//! 调用方保证Mqvel与qvel来自同一模型和状态。
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::BatchLayout;
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;

#[derive(Clone, Debug)]
struct Spec {
    model_id: u64,
    nv: usize,
}
#[derive(Clone, Copy, Debug)]
struct Layout {
    pair: BatchLayout,
    energy: BatchLayout,
}
fn spec(model_id: u64, nv: usize) -> Result<Spec, InputError> {
    if model_id == 0 || !(1..=1024).contains(&nv) {
        return Err(InputError::InvalidDimension {
            field: "kinetic_energy_spec",
        });
    }
    Ok(Spec { model_id, nv })
}
fn layout(worlds: usize, nv: usize) -> Result<Layout, InputError> {
    if worlds == 0 || worlds > i32::MAX as usize || !(1..=1024).contains(&nv) {
        return Err(InputError::InvalidDimension {
            field: "kinetic_energy_layout",
        });
    }
    let pair = BatchLayout::new(worlds, 2 * nv, 4)?;
    let energy = BatchLayout::new(worlds, 2, 4)?;
    for count in [pair.total_elements(), energy.total_elements(), worlds] {
        count
            .checked_add(8)
            .and_then(|v| v.checked_mul(4))
            .ok_or(InputError::Overflow {
                field: "kinetic_energy_guards",
            })?;
    }
    Ok(Layout { pair, energy })
}
/// 标签声明来源关联，不认证外部矩阵或生产者。
#[derive(Clone, Copy, Debug)]
pub struct KineticEnergyVector<'a> {
    pub model_id: u64,
    pub state_id: u64,
    pub values: &'a [f32],
}
fn check(v: &[f32], expected: usize, field: &'static str) -> Result<(), InputError> {
    if v.len() != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual: v.len(),
        });
    }
    if let Some(index) = v.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite { field, index });
    }
    Ok(())
}
fn check_pair(
    s: &Spec,
    count: usize,
    q: KineticEnergyVector<'_>,
    m: KineticEnergyVector<'_>,
) -> Result<(), TransferError> {
    if q.model_id != s.model_id || m.model_id != s.model_id {
        return Err(TransferError::ModelMismatch);
    }
    if q.state_id != m.state_id {
        return Err(InputError::InvalidTopology {
            field: "kinetic_energy_pair",
            index: 0,
            reason: "state_mismatch",
        }
        .into());
    }
    check(q.values, count, "qvel")?;
    check(m.values, count, "Mqvel")?;
    Ok(())
}
pub struct KineticEnergyPlan {
    spec: Arc<Spec>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 2],
}
pub struct KineticEnergyData {
    spec: Arc<Spec>,
    layout: Layout,
    states: Vec<Option<u64>>,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    pair: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct KineticEnergyOutput {
    layout: BatchLayout,
    model_id: u64,
    states: Vec<u64>,
    values: Vec<f32>,
}
impl KineticEnergyOutput {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn model_id(&self) -> u64 {
        self.model_id
    }
    /// 返回[势能,动能]；状态标签仅描述动能输入。
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
    pub fn state_id(&self, world: usize) -> Result<u64, InputError> {
        self.layout.world_elements(world)?;
        Ok(self.states[world])
    }
}
impl KineticEnergyPlan {
    pub fn new(session: &TransferSession, model_id: u64, nv: usize) -> Result<Self, TransferError> {
        let s = spec(model_id, nv)?;
        layout(1, nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, s);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let source = format!("#define ENERGY_NV {nv}\n{CUDA}");
            Ok(Self {
                spec: Arc::new(s),
                session: session.clone(),
                metadata: session.upload(&[nv as i32])?,
                kernels: [
                    SynchronousKernel::compile(session, &source, "kinetic_energy_validate")?,
                    SynchronousKernel::compile(session, &source, "kinetic_energy_reduce32")?,
                ],
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<KineticEnergyData, TransferError> {
        let l = layout(worlds, self.spec.nv)?;
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
            Ok(KineticEnergyData {
                spec: self.spec.clone(),
                layout: l,
                states: vec![None; worlds],
                ready: false,
                pair: alloc(l.pair.total_elements(), 0.0)?,
                current,
                staged: alloc(l.energy.total_elements(), f32::NAN)?,
                status: alloc(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, d: &mut KineticEnergyData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.spec, &d.spec) {
            return Err(TransferError::ModelMismatch);
        }
        if d.states.iter().any(Option::is_none) {
            return Err(TransferError::StageNotReady {
                stage: "kinetic_energy_pair",
            });
        }
        d.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let dims = [self.spec.nv as u32, d.worlds() as u32, 0, d.worlds() as u32];
            // SAFETY: Exact guarded capacities and same-plan static NV bound
            // pair/current reads; one ordinary validator thread owns each world.
            unsafe {
                self.kernels[0].launch(&self.metadata, &d.pair, &d.current, &mut d.status, dims)?;
            }
            let status = read(&d.status, d.worlds(), "kinetic_energy_status")?;
            if let Some(index) = status[4..4 + d.worlds()].iter().position(|&v| v != 1.0) {
                return Err(InputError::InvalidTopology {
                    field: "kinetic_energy_source",
                    index,
                    reason: "nonfinite_guard_or_shape",
                }
                .into());
            }
            // SAFETY: Validated pairs are finite. Exactly32 lanes reach ballot;
            // each lane0 publishes its two staged slots, never current slots.
            unsafe {
                self.kernels[1].launch_kinetic_energy32(
                    &self.metadata,
                    &d.pair,
                    &d.current,
                    &mut d.staged,
                    dims,
                )?;
            }
            read(&d.staged, d.layout.energy.total_elements(), "energy_staged")?;
            std::mem::swap(&mut d.current, &mut d.staged);
            d.ready = true;
            Ok(())
        }
    }
}
impl KineticEnergyData {
    pub fn worlds(&self) -> usize {
        self.layout.energy.worlds()
    }
    pub fn write_pair(
        &mut self,
        q: KineticEnergyVector<'_>,
        m: KineticEnergyVector<'_>,
    ) -> Result<(), TransferError> {
        self.write(0, self.worlds(), q, m)
    }
    pub fn write_world_pair(
        &mut self,
        world: usize,
        q: KineticEnergyVector<'_>,
        m: KineticEnergyVector<'_>,
    ) -> Result<(), TransferError> {
        self.layout.pair.world_elements(world)?;
        self.write(world, 1, q, m)
    }
    fn write(
        &mut self,
        first: usize,
        worlds: usize,
        q: KineticEnergyVector<'_>,
        m: KineticEnergyVector<'_>,
    ) -> Result<(), TransferError> {
        check_pair(&self.spec, worlds * self.spec.nv, q, m)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = first;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let n = self.spec.nv;
            let mut packed = crate::runtime::host_staging::<f32>(2 * worlds * n)?;
            for w in 0..worlds {
                packed[2 * w * n..(2 * w + 1) * n].copy_from_slice(&q.values[w * n..(w + 1) * n]);
                packed[(2 * w + 1) * n..2 * (w + 1) * n]
                    .copy_from_slice(&m.values[w * n..(w + 1) * n]);
            }
            // 输入错误保持就绪；驱动错误先使旧快照失效。
            self.ready = false;
            self.pair.write_range(4 + 2 * first * n, &packed)?;
            self.states[first..first + worlds].fill(Some(q.state_id));
            Ok(())
        }
    }
    pub fn write_potential(&mut self, values: &[f32]) -> Result<(), TransferError> {
        check(values, self.worlds(), "energy_potential")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.ready = false;
            for (w, v) in values.iter().enumerate() {
                self.current.write_range(4 + 2 * w, &[*v])?;
            }
            Ok(())
        }
    }
    pub fn write_world_potential(&mut self, world: usize, value: f32) -> Result<(), TransferError> {
        self.layout.energy.world_elements(world)?;
        check(&[value], 1, "energy_potential")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.ready = false;
            self.current.write_range(4 + 2 * world, &[value])
        }
    }
    pub fn readback(&self) -> Result<KineticEnergyOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "kinetic_energy",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(KineticEnergyOutput {
                layout: self.layout.energy,
                model_id: self.spec.model_id,
                states: self.states.iter().map(|s| s.unwrap()).collect(),
                values: read(&self.current, self.layout.energy.total_elements(), "energy")?,
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
// Frozen sensor.py + Warp1.15 tile.h/tile_reduce.h (Apache-2.0).
// Preserve ordinary float multiplication/addition and compiler contraction
// permission. Padding is zero-initialized but never enters local reduction.
#[cfg(feature = "cuda-probe")]
pub(super) const CUDA: &str = r#"
extern "C" __global__ void kinetic_energy_validate(const int* meta,const float* pair,
 const float* current,float* status,unsigned n,unsigned worlds,unsigned unused,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 bool valid=meta[0]==ENERGY_NV&&n==ENERGY_NV;unsigned long long pl=2ull*n*worlds,el=2ull*worlds;
 for(unsigned j=0;j<4;++j)valid=valid&&pair[j]==-131072.0f&&pair[4+pl+j]==-131072.0f&&current[j]==-131072.0f&&current[4+el+j]==-131072.0f;
 for(unsigned j=0;j<2*n;++j)valid=valid&&isfinite(pair[4+2ull*n*w+j]);
 valid=valid&&isfinite(current[4+2ull*w]);status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void kinetic_energy_reduce32(const int* meta,const float* pair,
 const float* current,float* staged,unsigned n,unsigned worlds,unsigned unused,unsigned work){
 unsigned w=blockIdx.x,lane=threadIdx.x;const float* q=pair+4+2ull*ENERGY_NV*w;const float* m=q+ENERGY_NV;
 float products[(ENERGY_NV+31)/32];
 #pragma unroll
 for(unsigned r=0;r<(ENERGY_NV+31)/32;++r){unsigned i=lane+32*r;float a=0.0f,b=0.0f;if(i<ENERGY_NV){a=q[i];b=m[i];}products[r]=a*b;}
 float sum=products[0];bool active=lane<ENERGY_NV;
 #pragma unroll
 for(unsigned r=1;r<(ENERGY_NV+31)/32;++r){if(lane+32*r>=ENERGY_NV)break;sum=sum+products[r];}
 unsigned mask=__ballot_sync(0xffffffffu,active);
 if(active){
  #pragma unroll
  for(unsigned offset=16;offset>0;offset/=2){float other=__shfl_down_sync(mask,sum,offset,32);
   if(mask==0xffffffffu||(mask&(1u<<((lane+offset)%32))))sum=sum+other;}
  if(lane==0){staged[4+2ull*w]=current[4+2ull*w];staged[5+2ull*w]=0.5f*sum;}
 }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    fn tagged(values: &[f32], state_id: u64) -> KineticEnergyVector<'_> {
        KineticEnergyVector {
            model_id: 47,
            state_id,
            values,
        }
    }
    fn fixture() -> serde_json::Value {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/kinetic-energy/reference.json"
        )))
        .unwrap()
    }
    fn stress(v: &serde_json::Value) -> (Vec<f32>, Vec<f32>, u32) {
        let n = v["nv"].as_u64().unwrap() as usize;
        let expected = u32::from_str_radix(v["expected_bits"].as_str().unwrap(), 16).unwrap();
        let mut q = vec![1.0; n];
        let mut m = vec![0.0; n];
        match v["kind"].as_str().unwrap() {
            "cancel" | "cancel_tail" => {
                m[0] = 33554432.0;
                m[1] = 1.0;
                m[16] = -33554432.0;
                if n == 33 {
                    m[32] = -1.0;
                }
            }
            "negative_zero" => q.fill(-0.0),
            "subnormal" => m.fill(f32::from_bits(1)),
            _ => panic!("unknown certificate"),
        }
        (q, m, expected)
    }
    // Diagnostic only: fixed Warp lane/register/shuffle dependency graph.
    // This never computes a product output or supplies a device result.
    fn warp_certificate(q: &[f32], m: &[f32]) -> f32 {
        let mut lanes = [0.0f32; 32];
        let active = q.len().min(32);
        for (lane, sum) in lanes.iter_mut().enumerate().take(active) {
            *sum = q[lane] * m[lane];
            for i in (lane + 32..q.len()).step_by(32) {
                *sum += q[i] * m[i];
            }
        }
        let mask = if active == 32 {
            u32::MAX
        } else {
            (1u32 << active) - 1
        };
        for offset in [16, 8, 4, 2, 1] {
            let old = lanes;
            for lane in 0..active {
                if mask == u32::MAX || mask & (1 << ((lane + offset) % 32)) != 0 {
                    // Out-of-warp shuffles return the requesting lane's value.
                    lanes[lane] += old[if lane + offset < 32 {
                        lane + offset
                    } else {
                        lane
                    }];
                }
            }
        }
        0.5 * lanes[0]
    }
    #[test]
    fn host_rejects_shapes_tags_and_both_invalid_vectors_before_pairing() {
        assert!(spec(0, 1).is_err());
        assert!(spec(47, 0).is_err());
        assert!(spec(47, 1025).is_err());
        assert!(layout(0, 1).is_err());
        assert!(layout(i32::MAX as usize + 1, 1).is_err());
        for n in [1, 7, 31, 32, 33, 65, 129, 513, 1024] {
            let l = layout(257, n).unwrap();
            assert_eq!(l.pair.total_elements(), 257 * 2 * n);
            assert_eq!(l.energy.total_elements(), 514);
            assert!(l.pair.world_elements(257).is_err());
        }
        let s = spec(47, 2).unwrap();
        let q = [-0.0, 1.0];
        let m = [0.0, 2.0];
        check_pair(&s, 2, tagged(&q, 1), tagged(&m, 1)).unwrap();
        let wrong = KineticEnergyVector {
            model_id: 48,
            ..tagged(&m, 1)
        };
        assert!(check_pair(&s, 2, wrong, tagged(&m, 1)).is_err());
        assert!(check_pair(&s, 2, tagged(&q, 1), wrong).is_err());
        assert!(check_pair(&s, 2, tagged(&q, 1), tagged(&m, 2)).is_err());
        for bad in [
            &[][..],
            &[1.0][..],
            &[1.0, 2.0, 3.0][..],
            &[f32::NAN, 1.0][..],
            &[1.0, f32::INFINITY][..],
        ] {
            assert!(check_pair(&s, 2, tagged(bad, 1), tagged(&m, 1)).is_err());
            assert!(check_pair(&s, 2, tagged(&q, 1), tagged(bad, 1)).is_err());
        }
    }
    #[test]
    fn host_certifies_register_tail_shuffle_tree_and_signed_subnormal_bits() {
        let v = fixture();
        assert_eq!(v["block_dim"], 32);
        for c in v["cases"].as_array().unwrap() {
            let (q, m, expected) = stress(c);
            assert_eq!(warp_certificate(&q, &m).to_bits(), expected);
            if let Some(bits) = c["serial_bits"].as_str() {
                let serial = 0.5 * q.iter().zip(&m).fold(0.0f32, |s, (&q, &m)| s + q * m);
                assert_eq!(serial.to_bits(), u32::from_str_radix(bits, 16).unwrap());
                assert_ne!(serial.to_bits(), expected);
                let diagnostic = 0.5
                    * q.iter()
                        .zip(&m)
                        .map(|(&q, &m)| q as f64 * m as f64)
                        .sum::<f64>();
                assert_eq!(diagnostic, c["f64_energy"].as_f64().unwrap());
            }
        }
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
    fn scene(n: usize, worlds: usize, state: usize) -> (Vec<f32>, Vec<f32>, Vec<f64>, Vec<f32>) {
        let mut q = Vec::new();
        let mut m = Vec::new();
        let mut expected = Vec::new();
        let mut p = Vec::new();
        for w in 0..worlds {
            let mut energy = 0.0;
            for i in 0..n {
                // Independent positive diagonal model, dyadic inputs. Mqvel
                // derives from that known M; no old mass producer executes.
                let mass = ((i + state) % 7 + 1) as f64 / 4.0;
                let velocity = ((i + 3 * w + state) % 9) as f64 / 8.0 - 0.5;
                q.push(velocity as f32);
                m.push((mass * velocity) as f32);
                energy += 0.5 * mass * velocity * velocity;
            }
            assert_eq!(energy as f32 as f64, energy);
            expected.push(energy);
            p.push(if (w + state) % 7 == 0 {
                -0.0
            } else {
                ((w + state) % 17) as f32 / 4.0
            });
        }
        (q, m, expected, p)
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new U047 kinetic consumer, independent diagonal M, warp bits and paired state lifecycle"]
    fn gpu_reduces_exact_dyadic_and_fixed_warp_cases_with_paired_world_states() {
        let session = TransferSession::new(0).unwrap();
        let start = std::time::Instant::now();
        let mut batches = 0;
        let mut worlds_checked = 0;
        let mut capacity = 0;
        for n in [1, 7, 31, 32, 33, 65, 129, 513] {
            let plan = KineticEnergyPlan::new(&session, 47, n).unwrap();
            for worlds in [1, 2, 257] {
                let mut d = plan.create_data(worlds).unwrap();
                assert!(plan.update(&mut d).is_err());
                assert!(d.readback().is_err());
                capacity = capacity.max(
                    4 * (d.pair.len()
                        + d.current.len()
                        + d.staged.len()
                        + d.status.len()
                        + plan.metadata.len()),
                );
                for state in 0..2 {
                    let (q, m, expected, p) = scene(n, worlds, state);
                    d.write_pair(tagged(&q, state as u64), tagged(&m, state as u64))
                        .unwrap();
                    d.write_potential(&p).unwrap();
                    plan.update(&mut d).unwrap();
                    let out = d.readback().unwrap();
                    assert_eq!(out.model_id(), 47);
                    assert_eq!(out.worlds(), worlds);
                    assert!(out.world(worlds).is_err());
                    assert!(out.state_id(worlds).is_err());
                    for w in 0..worlds {
                        assert_eq!(out.world(w).unwrap()[0].to_bits(), p[w].to_bits());
                        assert_eq!(
                            out.world(w).unwrap()[1].to_bits(),
                            (expected[w] as f32).to_bits()
                        );
                        assert_eq!(out.state_id(w).unwrap(), state as u64);
                    }
                    batches += 1;
                    worlds_checked += worlds;
                }
                // Invalid tags/values and foreign plan preserve ready output,
                // both input fields and all state associations bit-for-bit.
                let before = bits(&raw(&d.pair));
                let result = bits(&raw(&d.current));
                let states = d.states.clone();
                let q = vec![1.0; n];
                let m = vec![2.0; n];
                assert!(
                    d.write_world_pair(worlds, tagged(&q, 8), tagged(&m, 8))
                        .is_err()
                );
                assert!(d.write_world_pair(0, tagged(&q, 8), tagged(&m, 9)).is_err());
                for bad in [
                    KineticEnergyVector {
                        model_id: 48,
                        ..tagged(&m, 8)
                    },
                    tagged(&[], 8),
                ] {
                    assert!(d.write_world_pair(0, tagged(&q, 8), bad).is_err());
                    assert!(d.write_world_pair(0, bad, tagged(&m, 8)).is_err());
                }
                let bad = vec![f32::NAN; n];
                assert!(
                    d.write_world_pair(0, tagged(&q, 8), tagged(&bad, 8))
                        .is_err()
                );
                assert!(d.write_potential(&vec![f32::INFINITY; worlds]).is_err());
                assert!(d.write_world_potential(0, f32::NAN).is_err());
                assert!(d.ready);
                assert_eq!(before, bits(&raw(&d.pair)));
                assert_eq!(result, bits(&raw(&d.current)));
                assert_eq!(states, d.states);
                if worlds == 257 {
                    d.write_world_pair(256, tagged(&q, 17), tagged(&m, 17))
                        .unwrap();
                    assert!(d.readback().is_err());
                    let after = bits(&raw(&d.pair));
                    let a = 4 + 512 * n;
                    assert_eq!(&before[..a], &after[..a]);
                    assert_eq!(&before[a + 2 * n..], &after[a + 2 * n..]);
                    assert_eq!(after[a..a + n], bits(&q));
                    assert_eq!(after[a + n..a + 2 * n], bits(&m));
                    assert_eq!(&d.states[..256], &states[..256]);
                    assert_eq!(d.states[256], Some(17));
                    d.write_world_potential(256, -0.0).unwrap();
                    let cur = bits(&raw(&d.current));
                    assert_eq!(&cur[..516], &result[..516]);
                    assert_eq!(&cur[517..], &result[517..]);
                    assert_eq!(cur[516], 0x80000000);
                    plan.update(&mut d).unwrap();
                    let out = d.readback().unwrap();
                    assert_eq!(out.state_id(256).unwrap(), 17);
                    assert_eq!(out.world(256).unwrap(), &[-0.0, n as f32]);
                }
            }
        }
        for c in fixture()["cases"].as_array().unwrap() {
            let (q, m, expected) = stress(c);
            let plan = KineticEnergyPlan::new(&session, 47, q.len()).unwrap();
            let mut d = plan.create_data(2).unwrap();
            for w in 0..2 {
                d.write_world_pair(w, tagged(&q, w as u64), tagged(&m, w as u64))
                    .unwrap();
            }
            d.write_potential(&[-0.0, 3.5]).unwrap();
            plan.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            for w in 0..2 {
                assert_eq!(out.world(w).unwrap()[1].to_bits(), expected);
            }
        }
        let plan = KineticEnergyPlan::new(&session, 47, 7).unwrap();
        let mut d = plan.create_data(2).unwrap();
        let q = vec![1.0; 14];
        let m = vec![2.0; 14];
        d.write_pair(tagged(&q, 55), tagged(&m, 55)).unwrap();
        plan.update(&mut d).unwrap();
        let foreign = KineticEnergyPlan::new(&session, 47, 7).unwrap();
        let before = bits(&raw(&d.current));
        assert!(foreign.update(&mut d).is_err());
        assert!(d.ready);
        assert_eq!(before, bits(&raw(&d.current)));
        let snapshot = d.readback().unwrap();
        drop(d);
        drop(foreign);
        drop(plan);
        drop(session);
        assert_eq!(snapshot.world(1).unwrap(), &[0.0, 7.0]);
        assert_eq!(snapshot.state_id(1).unwrap(), 55);
        assert_eq!((batches, worlds_checked), (48, 4160));
        assert_eq!(capacity, 1060000);
        println!(
            "kinetic_energy32 batches={batches} worlds={worlds_checked} kinetic_slots=4160 potential_slots=4160 fixed_bit_worlds=12 max_bit_difference=0 phase_ms={} owned_buffer_capacity_bytes={capacity}",
            start.elapsed().as_millis()
        );
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new kinetic consumer private guards/source/overflow transactional publication"]
    fn gpu_rejects_source_guards_and_overflow_without_partial_publication() {
        let session = TransferSession::new(0).unwrap();
        let mut plan = KineticEnergyPlan::new(&session, 47, 65).unwrap();
        let mut d = plan.create_data(257).unwrap();
        let (q, m, _, p) = scene(65, 257, 0);
        d.write_pair(tagged(&q, 1), tagged(&m, 1)).unwrap();
        d.write_potential(&p).unwrap();
        plan.update(&mut d).unwrap();
        for field in 0..4 {
            for edge in 0..8 {
                let b = match field {
                    0 => &mut d.pair,
                    1 => &mut d.current,
                    2 => &mut d.staged,
                    _ => &mut d.status,
                };
                let index = if edge < 4 { edge } else { b.len() - 8 + edge };
                b.write_range(index, &[99.0]).unwrap();
                let before = bits(&raw(&d.current));
                assert!(plan.update(&mut d).is_err());
                assert!(!d.ready);
                assert!(d.readback().is_err());
                assert_eq!(before, bits(&raw(&d.current)));
                let b = match field {
                    0 => &mut d.pair,
                    1 => &mut d.current,
                    2 => &mut d.staged,
                    _ => &mut d.status,
                };
                b.write_range(index, &[-131072.0]).unwrap();
                plan.update(&mut d).unwrap();
            }
        }
        plan.metadata.write_range(0, &[64]).unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.update(&mut d).is_err());
        assert_eq!(before, bits(&raw(&d.current)));
        plan.metadata.write_range(0, &[65]).unwrap();
        plan.update(&mut d).unwrap();
        for index in [4, 68, 69, 133, 4 + 512 * 65, 4 + 514 * 65 - 1] {
            for bad in [f32::NAN, f32::INFINITY] {
                let old = raw(&d.pair)[index];
                d.pair.write_range(index, &[bad]).unwrap();
                let before = bits(&raw(&d.current));
                assert!(plan.update(&mut d).is_err());
                assert_eq!(before, bits(&raw(&d.current)));
                d.pair.write_range(index, &[old]).unwrap();
                plan.update(&mut d).unwrap();
            }
        }
        let index = 516;
        let old = raw(&d.current)[index];
        d.current.write_range(index, &[f32::NAN]).unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.update(&mut d).is_err());
        assert_eq!(before, bits(&raw(&d.current)));
        d.current.write_range(index, &[old]).unwrap();
        // Old kinetic contents do not enter the frozen function at all.
        d.current.write_range(517, &[f32::NAN]).unwrap();
        plan.update(&mut d).unwrap();
        for (q, m) in [
            (vec![f32::MAX; 65], vec![2.0; 65]),
            (vec![1.0; 65], vec![f32::MAX; 65]),
        ] {
            d.write_world_pair(256, tagged(&q, 9), tagged(&m, 9))
                .unwrap();
            let before = bits(&raw(&d.current));
            assert!(plan.update(&mut d).is_err());
            assert_eq!(before, bits(&raw(&d.current)));
            assert!(d.readback().is_err());
            let (q, m, _, _) = scene(65, 1, 0);
            d.write_world_pair(256, tagged(&q, 10), tagged(&m, 10))
                .unwrap();
            plan.update(&mut d).unwrap();
        }
        let (q, m, _, _) = scene(65, 1, 0);
        let mut missing = plan.create_data(2).unwrap();
        missing
            .write_world_pair(1, tagged(&q, 0), tagged(&m, 0))
            .unwrap();
        assert!(plan.update(&mut missing).is_err());
        assert_eq!(missing.states, vec![None, Some(0)]);
        println!(
            "kinetic_energy_private guard_faults=32 metadata_faults=1 source_nonfinite_faults=12 potential_faults=1 overflow_faults=2 recovered=true"
        );
    }
}
