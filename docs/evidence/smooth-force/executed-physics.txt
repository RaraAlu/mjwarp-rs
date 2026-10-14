//! 四项广义力的常驻合并辅助。
//! 不累加空间外力，不计算平滑加速度。
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, SmoothForceInput};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SmoothForceOptions {
    pub sleep_enabled: bool,
    pub island_disabled: bool,
}
impl SmoothForceOptions {
    pub fn enable_sleep(self) -> bool {
        self.sleep_enabled && !self.island_disabled
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmoothForceField {
    Passive,
    Bias,
    Actuator,
    Applied,
}
impl SmoothForceField {
    fn index(self) -> usize {
        match self {
            Self::Passive => 0,
            Self::Bias => 1,
            Self::Actuator => 2,
            Self::Applied => 3,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Passive => "qfrc_passive",
            Self::Bias => "qfrc_bias",
            Self::Actuator => "qfrc_actuator",
            Self::Applied => "qfrc_applied",
        }
    }
}
#[derive(Clone, Copy)]
struct Layout {
    vector: BatchLayout,
    awake: BatchLayout,
    force_len: usize,
    meta_len: usize,
}
fn layout(worlds: usize, nbody: usize, nv: usize, ntree: usize) -> Result<Layout, InputError> {
    let overflow = || InputError::Overflow {
        field: "smooth_force_layout",
    };
    if worlds == 0
        || nbody == 0
        || nv == 0
        || worlds
            .checked_mul(nv)
            .is_none_or(|v| v > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "smooth_force_work",
        });
    }
    let vector = BatchLayout::new(worlds, nv, 4)?;
    let awake = BatchLayout::new(worlds, ntree, 4)?;
    let force_len = vector
        .total_elements()
        .checked_mul(4)
        .ok_or_else(overflow)?;
    let meta_len = nbody
        .checked_add(nv)
        .and_then(|v| v.checked_add(awake.total_elements()))
        .and_then(|v| v.checked_add(8))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or_else(overflow)?;
    for count in [force_len, vector.total_elements(), worlds] {
        BatchLayout::new(1, count.checked_add(8).ok_or_else(overflow)?, 4)?;
    }
    BatchLayout::new(1, meta_len, 4)?;
    Ok(Layout {
        vector,
        awake,
        force_len,
        meta_len,
    })
}
pub struct SmoothForcePlan {
    input: Arc<SmoothForceInput>,
    options: SmoothForceOptions,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 2],
}
pub struct SmoothForceData {
    input: Arc<SmoothForceInput>,
    layout: Layout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    forces: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SmoothForceOutput {
    layout: BatchLayout,
    values: Vec<f32>,
}
impl SmoothForceOutput {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
impl SmoothForcePlan {
    pub fn new(
        session: &TransferSession,
        input: SmoothForceInput,
        options: SmoothForceOptions,
    ) -> Result<Self, TransferError> {
        layout(1, input.nbody(), input.nv(), input.ntree())?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, options);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let source = format!(
                "#define SMOOTH_SLEEP {}\n{}",
                i32::from(options.enable_sleep()),
                CUDA
            );
            Ok(Self {
                input: Arc::new(input),
                options,
                session: session.clone(),
                kernels: [
                    SynchronousKernel::compile(session, &source, "smooth_force_validate")?,
                    SynchronousKernel::compile(session, &source, "smooth_force")?,
                ],
            })
        }
    }
    pub fn input(&self) -> &SmoothForceInput {
        &self.input
    }
    pub fn options(&self) -> SmoothForceOptions {
        self.options
    }
    pub fn create_data(&self, worlds: usize) -> Result<SmoothForceData, TransferError> {
        let l = layout(
            worlds,
            self.input.nbody(),
            self.input.nv(),
            self.input.ntree(),
        )?;
        // 整数守卫包围静态映射与逐世界树状态。
        let mut meta = crate::runtime::host_staging::<i32>(l.meta_len)?;
        meta.fill(-131072);
        let body_end = 4 + self.input.nbody();
        let awake_start = body_end + self.input.nv();
        meta[4..body_end].copy_from_slice(self.input.body_treeid());
        meta[body_end..awake_start].copy_from_slice(self.input.dof_bodyid());
        meta[awake_start..l.meta_len - 4].fill(1);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = meta;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count, initial| {
                let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
                v.fill(-131072.0);
                v[4..4 + count].fill(initial);
                self.session.upload(&v)
            };
            Ok(SmoothForceData {
                input: self.input.clone(),
                layout: l,
                ready: false,
                metadata: self.session.upload(&meta)?,
                forces: allocate(l.force_len, 0.0)?,
                output: allocate(l.vector.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut SmoothForceData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let nb = self.input.nbody() as u32;
            let n = self.input.nv() as u32;
            let nt = self.input.ntree() as u32;
            let worlds = data.worlds() as u32;
            // SAFETY: Checked same-plan guarded metadata owns nb+n+worlds*nt
            // i32 payload. Each validator writes a separate status/world and
            // bounds-checks every body/tree before dereferencing that mapping.
            unsafe {
                self.kernels[0].launch(
                    &data.metadata,
                    &data.forces,
                    &data.forces,
                    &mut data.status,
                    [nb, n, nt, worlds],
                )?;
            }
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "smooth_force_status")?;
            if let Some(index) = status[4..4 + data.worlds()].iter().position(|&v| v != 1.0) {
                return Err(InputError::InvalidTopology {
                    field: "smooth_force_source",
                    index,
                    reason: "nonfinite_guard_or_mapping",
                }
                .into());
            }
            // SAFETY: Validation proves all mappings and active force payloads.
            // Field-major input owns4*worlds*n f32s; one thread owns each result.
            // Sleeping threads write positive zero without reading any forces.
            unsafe {
                self.kernels[1].launch(
                    &data.metadata,
                    &data.forces,
                    &data.forces,
                    &mut data.output,
                    [nb, n, nt, data.layout.vector.total_elements() as u32],
                )?;
            }
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
fn check_force(values: &[f32], expected: usize, field: SmoothForceField) -> Result<(), InputError> {
    if values.len() != expected {
        return Err(InputError::LengthMismatch {
            field: field.name(),
            expected,
            actual: values.len(),
        });
    }
    if let Some(index) = values.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite {
            field: field.name(),
            index,
        });
    }
    Ok(())
}
impl SmoothForceData {
    pub fn worlds(&self) -> usize {
        self.layout.vector.worlds()
    }
    pub fn write_force(
        &mut self,
        field: SmoothForceField,
        values: &[f32],
    ) -> Result<(), TransferError> {
        self.write(field, 0, self.layout.vector.total_elements(), values)
    }
    pub fn write_world_force(
        &mut self,
        field: SmoothForceField,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.layout.vector.world_elements(world)?;
        self.write(field, r.start, r.len(), values)
    }
    fn write(
        &mut self,
        field: SmoothForceField,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        check_force(values, expected, field)?;
        let start = 4 + field.index() * self.layout.vector.total_elements() + offset;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = start;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.forces.write_range(start, values)
        }
    }
    pub fn write_awake(&mut self, values: &[i32]) -> Result<(), TransferError> {
        self.awake(0, self.layout.awake.total_elements(), values)
    }
    pub fn write_world_awake(&mut self, world: usize, values: &[i32]) -> Result<(), TransferError> {
        let r = self.layout.awake.world_elements(world)?;
        self.awake(r.start, r.len(), values)
    }
    fn awake(
        &mut self,
        offset: usize,
        expected: usize,
        values: &[i32],
    ) -> Result<(), TransferError> {
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "tree_awake",
                expected,
                actual: values.len(),
            }
            .into());
        }
        let start = 4 + self.input.nbody() + self.input.nv() + offset;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = start;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.metadata.write_range(start, values)
        }
    }
    #[cfg(feature = "cuda-probe")]
    fn checked_values(&self) -> Result<Vec<f32>, TransferError> {
        let mut v = crate::runtime::host_staging::<f32>(self.layout.vector.total_elements() + 8)?;
        self.output.read_range_into(0, &mut v)?;
        crate::physics::check_device_values(&v, "qfrc_smooth")?;
        Ok(v)
    }
    pub fn readback(&self) -> Result<SmoothForceOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "qfrc_smooth",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(SmoothForceOutput {
                layout: self.layout.vector,
                values: self.checked_values()?,
            })
        }
    }
}
// Frozen forward.py _qfrc_smooth (Apache-2.0): three separate f32 rounds.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void smooth_force_validate(const int* meta,const float* forces,
 const float* unused,float* status,unsigned nb,unsigned n,unsigned nt,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long total=1ull*worlds*n,mlen=1ull*nb+n+1ull*worlds*nt;bool valid=true;
 for(unsigned j=0;j<4;++j)valid=valid&&meta[j]==-131072&&meta[4+mlen+j]==-131072&&forces[j]==-131072.0f&&forces[4+4*total+j]==-131072.0f;
 const int* tree=meta+4;const int* body=tree+nb;const int* awake=body+n;
 for(unsigned b=0;b<nb;++b)if(tree[b]<-1||(tree[b]>=0&&(unsigned)tree[b]>=nt))valid=false;
 for(unsigned i=0;i<n;++i){int b=body[i];if(b<0||(unsigned)b>=nb){valid=false;continue;}
  int t=tree[b];if(t<-1||(t>=0&&(unsigned)t>=nt)){valid=false;continue;}
  #if SMOOTH_SLEEP
  if(t>=0&&awake[1ull*w*nt+t]==0)continue;
  #endif
  for(unsigned f=0;f<4;++f)valid=valid&&isfinite(forces[4+f*total+1ull*w*n+i]);
 }
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void smooth_force(const int* meta,const float* forces,
 const float* unused,float* output,unsigned nb,unsigned n,unsigned nt,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/n,i=tid%n;const int* tree=meta+4;const int* body=tree+nb;const int* awake=body+n;
 #if SMOOTH_SLEEP
 int t=tree[body[i]];if(t>=0&&awake[1ull*w*nt+t]==0){output[4+tid]=0.0f;return;}
 #endif
 const float* f=forces+4+tid;float acc=__fsub_rn(f[0],f[work]);
 acc=__fadd_rn(acc,f[2ull*work]);output[4+tid]=__fadd_rn(acc,f[3ull*work]);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    fn reference() -> serde_json::Value {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/smooth-force/reference.json"
        )))
        .unwrap()
    }
    fn row(v: &serde_json::Value, index: usize) -> ([f32; 4], u32) {
        let r = &v["rows"][index];
        let mut x = [0.0; 4];
        for (j, value) in x.iter_mut().enumerate() {
            *value = f32::from_bits(
                u32::from_str_radix(r["input_bits"][j].as_str().unwrap(), 16).unwrap(),
            );
        }
        let bits = u32::from_str_radix(r["output_bits"].as_str().unwrap(), 16).unwrap();
        (x, bits)
    }
    #[test]
    fn fixed_scalar_certificate_distinguishes_step_rounding_and_signed_zeros() {
        let v = reference();
        assert_eq!(v["rows"].as_array().unwrap().len(), 18);
        for i in 0..18 {
            let (x, bits) = row(&v, i);
            let first = x[0] - x[1];
            let second = first + x[2];
            let third = second + x[3];
            assert_eq!(third.to_bits(), bits, "case{i}");
        }
        let (x, bits) = row(&v, 2);
        assert_eq!(
            f64::from(x[0]) - f64::from(x[1]) + f64::from(x[2]) + f64::from(x[3]),
            0.0
        );
        assert_eq!(bits, 1.0f32.to_bits());
        assert_eq!(row(&v, 7).1, (-0.0f32).to_bits());
    }
    #[test]
    fn bounds_layout_and_force_lengths_without_gpu_allocation() {
        for (w, nb, n, nt) in [
            (0, 1, 1, 0),
            (1, 0, 1, 0),
            (1, 1, 0, 0),
            (usize::MAX, 1, 1, 1),
            (1, 1, 1, usize::MAX),
            (1, usize::MAX, 1, 0),
        ] {
            assert!(layout(w, nb, n, nt).is_err());
        }
        let l = layout(257, 273, 513, 129).unwrap();
        assert_eq!(l.force_len, 4 * 257 * 513);
        assert_eq!(l.meta_len, 273 + 513 + 257 * 129 + 8);
        assert!(layout(2, 2, 2, 0).is_ok());
        for f in [
            SmoothForceField::Passive,
            SmoothForceField::Bias,
            SmoothForceField::Actuator,
            SmoothForceField::Applied,
        ] {
            assert!(check_force(&[], 1, f).is_err());
            assert!(check_force(&[f32::NAN], 1, f).is_err());
        }
        for (sleep, island, enabled) in [
            (false, false, false),
            (false, true, false),
            (true, false, true),
            (true, true, false),
        ] {
            assert_eq!(
                SmoothForceOptions {
                    sleep_enabled: sleep,
                    island_disabled: island
                }
                .enable_sleep(),
                enabled
            );
        }
    }
    #[cfg(feature = "cuda-probe")]
    const FIELDS: [SmoothForceField; 4] = [
        SmoothForceField::Passive,
        SmoothForceField::Bias,
        SmoothForceField::Actuator,
        SmoothForceField::Applied,
    ];
    #[cfg(feature = "cuda-probe")]
    fn model(n: usize) -> SmoothForceInput {
        SmoothForceInput::new(
            129,
            (0..273)
                .map(|b| {
                    if b % 11 == 0 {
                        -1
                    } else {
                        (b * 7 % 129) as i32
                    }
                })
                .collect(),
            (0..n).map(|i| ((i * 17 + 1) % 273) as i32).collect(),
        )
        .unwrap()
    }
    #[cfg(feature = "cuda-probe")]
    fn raw_f32(buffer: &TransferBuffer<f32>) -> Vec<f32> {
        let mut v = vec![0.0; buffer.len()];
        buffer.read_range_into(0, &mut v).unwrap();
        v
    }
    #[cfg(feature = "cuda-probe")]
    fn raw_i32(buffer: &TransferBuffer<i32>) -> Vec<i32> {
        let mut v = vec![0; buffer.len()];
        buffer.read_range_into(0, &mut v).unwrap();
        v
    }
    #[cfg(feature = "cuda-probe")]
    fn bits(v: &[f32]) -> Vec<u32> {
        v.iter().map(|v| v.to_bits()).collect()
    }
    #[cfg(feature = "cuda-probe")]
    fn tokens(d: &SmoothForceData) -> (Vec<u32>, Vec<i32>, Vec<u32>) {
        (
            bits(&raw_f32(&d.forces)),
            raw_i32(&d.metadata),
            bits(&raw_f32(&d.output)),
        )
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new U016 four-force merge, independent scalar bits, sleep/island and owned snapshots"]
    fn gpu_merges_exact_f32_steps_and_zeroes_sleeping_dofs_in_all_option_combinations() {
        let v = reference();
        let session = TransferSession::new(0).unwrap();
        let start = std::time::Instant::now();
        let mut batches = 0usize;
        let mut slots = 0usize;
        let mut asleep = 0usize;
        let mut static_active = 0usize;
        let mut negative_zero = 0usize;
        let mut capacity = 0usize;
        let mut seen = [0usize; 18];
        for n in [1, 65, 513] {
            let input = model(n);
            for (sleep, island) in [(false, false), (false, true), (true, false), (true, true)] {
                let options = SmoothForceOptions {
                    sleep_enabled: sleep,
                    island_disabled: island,
                };
                let plan = SmoothForcePlan::new(&session, input.clone(), options).unwrap();
                assert_eq!(plan.options(), options);
                assert_eq!(plan.input(), &input);
                for sample in 0..2 {
                    for worlds in [1, 2, 257] {
                        let mut d = plan.create_data(worlds).unwrap();
                        assert!(d.readback().is_err());
                        let mut values: [Vec<f32>; 4] =
                            std::array::from_fn(|_| Vec::with_capacity(worlds * n));
                        for w in 0..worlds {
                            for i in 0..n {
                                let (x, _) = row(&v, (i + 3 * w + 5 * sample) % 18);
                                for f in 0..4 {
                                    values[f].push(x[f]);
                                }
                            }
                        }
                        let awake = (0..worlds * 129)
                            .map(|k| match (k % 129 + k / 129 + sample) % 5 {
                                0 => 0,
                                1 => -11,
                                2 => 2,
                                3 => 1,
                                _ => i32::MAX,
                            })
                            .collect::<Vec<_>>();
                        for (f, values) in FIELDS.iter().zip(&values) {
                            d.write_force(*f, values).unwrap();
                        }
                        d.write_awake(&awake).unwrap();
                        plan.update(&mut d).unwrap();
                        let out = d.readback().unwrap();
                        for w in 0..worlds {
                            for (i, &actual) in out.world(w).unwrap().iter().enumerate() {
                                let tree = input.body_treeid()[input.dof_bodyid()[i] as usize];
                                let index = (i + 3 * w + 5 * sample) % 18;
                                let sleeping = options.enable_sleep()
                                    && tree >= 0
                                    && awake[w * 129 + tree as usize] == 0;
                                let expected = if sleeping {
                                    asleep += 1;
                                    0
                                } else {
                                    seen[index] += 1;
                                    if tree < 0 {
                                        static_active += 1;
                                    }
                                    row(&v, index).1
                                };
                                assert_eq!(
                                    actual.to_bits(),
                                    expected,
                                    "n{n}/w{w}/i{i}/sample{sample}/{options:?}"
                                );
                                if !sleeping && expected == 0x80000000 {
                                    negative_zero += 1;
                                }
                                slots += 1;
                            }
                        }
                        assert!(out.world(worlds).is_err());
                        capacity = capacity.max(
                            4 * (d.forces.len()
                                + d.output.len()
                                + d.status.len()
                                + d.metadata.len()),
                        );
                        batches += 1;
                    }
                }
            }
        }
        assert_eq!((batches, slots), (72, 1204320));
        assert!(asleep > 0 && static_active > 0 && negative_zero > 0);
        assert!(seen.iter().all(|&c| c > 0));
        // New field/awake lifecycle only; no qpos, mass or RHS producer exists.
        let input = model(65);
        let options = SmoothForceOptions {
            sleep_enabled: true,
            island_disabled: false,
        };
        let plan = SmoothForcePlan::new(&session, input.clone(), options).unwrap();
        let other = SmoothForcePlan::new(&session, input.clone(), options).unwrap();
        let mut d = plan.create_data(257).unwrap();
        for (f, x) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
            d.write_force(*f, &vec![x; 257 * 65]).unwrap();
        }
        plan.update(&mut d).unwrap();
        let unchanged = tokens(&d);
        assert!(other.update(&mut d).is_err());
        assert_eq!(tokens(&d), unchanged);
        assert!(d.readback().is_ok());
        for f in FIELDS {
            for (w, length) in [(257, 65), (0, 64)] {
                assert!(d.write_world_force(f, w, &vec![1.0; length]).is_err());
                assert_eq!(tokens(&d), unchanged);
                assert!(d.readback().is_ok());
            }
            let mut bad = vec![1.0; 65];
            bad[3] = f32::NAN;
            assert!(d.write_world_force(f, 0, &bad).is_err());
            assert!(d.write_force(f, &[]).is_err());
            assert_eq!(tokens(&d), unchanged);
            assert!(d.readback().is_ok());
        }
        assert!(d.write_world_awake(257, &[1; 129]).is_err());
        assert!(d.write_world_awake(0, &[1; 128]).is_err());
        assert!(d.write_awake(&[]).is_err());
        assert_eq!(tokens(&d), unchanged);
        assert!(d.readback().is_ok());
        for (j, f) in FIELDS.iter().enumerate() {
            let before = raw_f32(&d.forces);
            let values = vec![[7.0, 1.0, 6.0, 8.0][j]; 65];
            let a = 4 + j * 257 * 65 + 256 * 65;
            d.write_world_force(*f, 256, &values).unwrap();
            let after = raw_f32(&d.forces);
            for i in (0..a).chain(a + 65..before.len()) {
                assert_eq!(before[i].to_bits(), after[i].to_bits());
            }
            assert_eq!(bits(&after[a..a + 65]), bits(&values));
            assert!(d.readback().is_err());
            plan.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            for w in 0..257 {
                let expected = if w == 256 {
                    [12.0f32, 11.0, 13.0, 14.0][j]
                } else {
                    10.0
                };
                assert!(
                    out.world(w)
                        .unwrap()
                        .iter()
                        .all(|v| v.to_bits() == expected.to_bits())
                );
            }
            d.write_world_force(*f, 256, &vec![[5.0, 2.0, 3.0, 4.0][j]; 65])
                .unwrap();
            plan.update(&mut d).unwrap();
        }
        let before = raw_i32(&d.metadata);
        let a = 4 + 273 + 65 + 256 * 129;
        d.write_world_awake(256, &[0; 129]).unwrap();
        let after = raw_i32(&d.metadata);
        for i in (0..a).chain(a + 129..before.len()) {
            assert_eq!(before[i], after[i]);
        }
        assert!(after[a..a + 129].iter().all(|&v| v == 0));
        assert!(d.readback().is_err());
        plan.update(&mut d).unwrap();
        let sleeping = d.readback().unwrap();
        for w in 0..257 {
            for (i, &value) in sleeping.world(w).unwrap().iter().enumerate() {
                let tree = input.body_treeid()[input.dof_bodyid()[i] as usize];
                let expected = if w == 256 && tree >= 0 { 0.0f32 } else { 10.0 };
                assert_eq!(value.to_bits(), expected.to_bits());
            }
        }
        d.write_world_awake(256, &[-11; 129]).unwrap();
        plan.update(&mut d).unwrap();
        let snapshot = d.readback().unwrap();
        drop(d);
        drop(plan);
        drop(other);
        drop(session);
        assert!(
            snapshot
                .world(256)
                .unwrap()
                .iter()
                .all(|v| v.to_bits() == 10.0f32.to_bits())
        );
        // ntree0 has no awake payload; static DOFs still preserve active -0.
        let session = TransferSession::new(0).unwrap();
        let plan = SmoothForcePlan::new(
            &session,
            SmoothForceInput::new(0, vec![-1, -1], vec![1, 0]).unwrap(),
            options,
        )
        .unwrap();
        let mut d = plan.create_data(2).unwrap();
        let (x, _) = row(&v, 7);
        for (f, x) in FIELDS.iter().zip(x) {
            d.write_force(*f, &[x; 4]).unwrap();
        }
        d.write_world_awake(1, &[]).unwrap();
        plan.update(&mut d).unwrap();
        assert!(
            d.readback()
                .unwrap()
                .world(1)
                .unwrap()
                .iter()
                .all(|v| v.to_bits() == 0x80000000)
        );
        let report = serde_json::json!({"batches":batches,"slots":slots,"bit_differences":0,"asleep_positive_zero_slots":asleep,"static_active_slots":static_active,"active_negative_zero_slots":negative_zero,"active_case_coverage":seen,"phase_ms":start.elapsed().as_millis(),"owned_Data_max_bytes":capacity,"full_fwd_acceleration_NP":false});
        std::fs::write(
            std::env::var_os("MJWARP_SMOOTH_FORCE_REPORT").unwrap(),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("{report}");
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new smooth-force finite/overflow, mapping and full guarded-buffer rejection"]
    fn gpu_rejects_guard_mapping_and_active_overflow_but_overwrites_sleeping_output_with_zero() {
        let session = TransferSession::new(0).unwrap();
        let input = SmoothForceInput::new(2, vec![-1, 0, 1], vec![1, 2, 0]).unwrap();
        let options = SmoothForceOptions {
            sleep_enabled: true,
            island_disabled: false,
        };
        let plan = SmoothForcePlan::new(&session, input, options).unwrap();
        let mut d = plan.create_data(2).unwrap();
        for (f, x) in FIELDS.iter().zip([5.0, 2.0, 3.0, 4.0]) {
            d.write_force(*f, &[x; 6]).unwrap();
        }
        plan.update(&mut d).unwrap();
        let good = d.readback().unwrap();
        for (kind, offset, bad) in [
            (0, 0, 0.0),
            (0, 28, 0.0),
            (0, 4, f32::NAN),
            (0, 4 + 18 + 2, f32::INFINITY),
            (1, 0, 0.0),
            (1, 10, 0.0),
            (2, 0, 0.0),
            (2, 6, 0.0),
        ] {
            let buffer = match kind {
                0 => &mut d.forces,
                1 => &mut d.output,
                _ => &mut d.status,
            };
            let mut old = [0.0];
            buffer.read_range_into(offset, &mut old).unwrap();
            buffer.write_range(offset, &[bad]).unwrap();
            assert!(plan.update(&mut d).is_err());
            assert!(d.readback().is_err());
            let buffer = match kind {
                0 => &mut d.forces,
                1 => &mut d.output,
                _ => &mut d.status,
            };
            buffer.write_range(offset, &old).unwrap();
            plan.update(&mut d).unwrap();
            assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
        }
        for (offset, bad) in [(0, 0), (14, 0), (5, -2), (5, 2), (7, -1), (7, 3)] {
            let mut old = [0];
            d.metadata.read_range_into(offset, &mut old).unwrap();
            d.metadata.write_range(offset, &[bad]).unwrap();
            assert!(plan.update(&mut d).is_err());
            assert!(d.readback().is_err());
            d.metadata.write_range(offset, &old).unwrap();
            plan.update(&mut d).unwrap();
            assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
        }
        d.write_world_awake(0, &[0, 1]).unwrap();
        d.forces.write_range(4, &[f32::NAN]).unwrap();
        plan.update(&mut d).unwrap();
        let asleep = d.readback().unwrap();
        assert_eq!(asleep.world(0).unwrap()[0].to_bits(), 0);
        assert_eq!(asleep.world(0).unwrap()[1], 10.0);
        assert_eq!(asleep.world(0).unwrap()[2], 10.0);
        d.write_world_awake(0, &[2, -11]).unwrap();
        assert!(plan.update(&mut d).is_err());
        assert!(d.readback().is_err());
        d.forces.write_range(4, &[5.0]).unwrap();
        plan.update(&mut d).unwrap();
        d.write_world_force(SmoothForceField::Passive, 0, &[f32::MAX, 5.0, 5.0])
            .unwrap();
        d.write_world_force(SmoothForceField::Bias, 0, &[-f32::MAX, 2.0, 2.0])
            .unwrap();
        assert!(plan.update(&mut d).is_err());
        assert!(d.readback().is_err());
        d.write_world_awake(0, &[0, 1]).unwrap();
        plan.update(&mut d).unwrap();
        assert_eq!(d.readback().unwrap().world(0).unwrap()[0].to_bits(), 0);
        d.write_world_force(SmoothForceField::Passive, 0, &[5.0; 3])
            .unwrap();
        d.write_world_force(SmoothForceField::Bias, 0, &[2.0; 3])
            .unwrap();
        d.write_world_awake(0, &[1, 1]).unwrap();
        plan.update(&mut d).unwrap();
        assert_eq!(bits(&d.readback().unwrap().values), bits(&good.values));
    }
}
