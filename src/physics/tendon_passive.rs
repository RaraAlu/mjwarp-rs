//! 肌腱弹簧与阻尼被动力。
//! 本模块不代表完整被动力阶段。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, ParameterBatch};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
use std::sync::Arc;

use super::{TendonVelocityPlan, TendonVelocityView};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TendonPassiveOptions {
    pub disable_spring: bool,
    pub disable_damper: bool,
}

#[derive(Clone, Debug)]
pub struct TendonPassiveParameters {
    pub stiffness: ParameterBatch,
    pub stiffness_poly: ParameterBatch,
    pub damping: ParameterBatch,
    pub damping_poly: ParameterBatch,
    pub length_spring: ParameterBatch,
}

impl TendonPassiveParameters {
    // 默认配置无设备路径；仅cuda-probe读取。
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    fn fields(&self) -> [&ParameterBatch; 5] {
        [
            &self.stiffness,
            &self.stiffness_poly,
            &self.damping,
            &self.damping_poly,
            &self.length_spring,
        ]
    }
}

// 默认配置无设备路径；仅cuda-probe读取。
#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
struct Spec {
    nv: usize,
    nt: usize,
    max_row: usize,
    offsets: [usize; 5],
    parameter_len: usize,
    parameters: TendonPassiveParameters,
}

// 默认配置无设备路径；仅cuda-probe读取。
#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
fn spec(
    context: (usize, usize, usize, usize, usize),
    parameters: TendonPassiveParameters,
) -> Result<Spec, InputError> {
    let (nv, nt, _nnz, _stride, max_row) = context;
    if nv == 0 || nt == 0 || max_row == 0 {
        return Err(InputError::InvalidDimension {
            field: "tendon_passive_dimensions",
        });
    }
    let widths = [nt, 2 * nt, nt, 2 * nt, 2 * nt];
    let mut offsets = [0; 5];
    let mut parameter_len = 0usize;
    for (index, field) in parameters.fields().iter().enumerate() {
        if field.row_elements() != widths[index] {
            return Err(InputError::LengthMismatch {
                field: "tendon_passive_parameter",
                expected: widths[index],
                actual: field.row_elements(),
            });
        }
        offsets[index] = parameter_len;
        parameter_len = parameter_len
            .checked_add(field.values().len())
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "tendon_passive_parameters",
            })?;
    }
    Ok(Spec {
        nv,
        nt,
        max_row,
        offsets,
        parameter_len,
        parameters,
    })
}

fn layout(worlds: usize, nv: usize) -> Result<BatchLayout, InputError> {
    if worlds == 0
        || nv == 0
        || worlds
            .checked_mul(nv)
            .and_then(|n| n.checked_mul(2))
            .is_none_or(|n| n > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "tendon_passive_work",
        });
    }
    BatchLayout::new(worlds, nv, 4)
}

pub struct TendonPassivePlan<'v, 'p> {
    velocity: &'v TendonVelocityPlan<'p>,
    spec: Arc<Spec>,
    options: TendonPassiveOptions,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    parameters: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 3],
}

pub struct TendonPassiveData {
    spec: Arc<Spec>,
    layout: BatchLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}

pub struct TendonPassiveView<'a, 'v, 'p> {
    plan: &'a TendonPassivePlan<'v, 'p>,
    velocity: &'a TendonVelocityView<'a, 'p>,
    data: &'a TendonPassiveData,
}

#[derive(Clone, Debug)]
pub struct TendonPassiveSnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}

impl TendonPassiveSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn spring(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        Ok(&self.values[4 + range.start..4 + range.end])
    }
    pub fn damper(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        let offset = 4 + self.layout.total_elements();
        Ok(&self.values[offset + range.start..offset + range.end])
    }
}

impl<'v, 'p> TendonPassivePlan<'v, 'p> {
    pub fn new(
        velocity: &'v TendonVelocityPlan<'p>,
        parameters: TendonPassiveParameters,
        options: TendonPassiveOptions,
    ) -> Result<Self, TransferError> {
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (velocity, parameters, options);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let (session, nv, nt, nnz, stride, max_row) = velocity.tendon_passive_context();
            let spec = spec((nv, nt, nnz, stride, max_row), parameters)?;
            layout(1, nv)?;
            let mut packed = vec![-131072.0; 4];
            for field in spec.parameters.fields() {
                packed.extend(field.values());
            }
            packed.extend([-131072.0; 4]);
            let periods = spec
                .parameters
                .fields()
                .map(|field| field.batches().to_string())
                .join(",");
            let offsets = spec.offsets.map(|offset| offset.to_string()).join(",");
            let code = format!(
                "#define TP_NV {nv}\n#define TP_NT {nt}\n#define TP_NNZ {nnz}\n#define TP_STRIDE {stride}\n#define TP_MAX_ROW {max_row}\n#define TP_PL {}\n#define TP_S {}\n#define TP_D {}\n__device__ __constant__ unsigned TP_BATCH[5]={{{periods}}};\n__device__ __constant__ unsigned TP_OFF[5]={{{offsets}}};\n{CUDA}",
                spec.parameter_len,
                u32::from(options.disable_spring),
                u32::from(options.disable_damper),
            );
            Ok(Self {
                velocity,
                spec: Arc::new(spec),
                options,
                parameters: session.upload(&packed)?,
                kernels: [
                    SynchronousKernel::compile(&session, &code, "tendon_passive_validate")?,
                    SynchronousKernel::compile(&session, &code, "tendon_passive_clear")?,
                    SynchronousKernel::compile(&session, &code, "tendon_passive_compute")?,
                ],
                session,
            })
        }
    }

    pub fn create_data(&self, worlds: usize) -> Result<TendonPassiveData, TransferError> {
        let layout = layout(worlds, self.spec.nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = layout;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count: usize, value: f32| {
                let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
                values.fill(-131072.0);
                values[4..4 + count].fill(value);
                self.session.upload(&values)
            };
            let count = 2 * layout.total_elements();
            Ok(TendonPassiveData {
                spec: self.spec.clone(),
                layout,
                ready: false,
                current: allocate(count, f32::NAN)?,
                staged: allocate(count, f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }

    pub fn options(&self) -> TendonPassiveOptions {
        self.options
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn shared_passive_context(&self) -> (TransferSession, usize, TendonPassiveOptions) {
        (self.session.clone(), self.spec.nv, self.options)
    }

    pub(super) fn accepts_shared_velocity(&self, velocity: &TendonVelocityView<'_, 'p>) -> bool {
        velocity.belongs_to(self.velocity)
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn accumulate_shared_passive(
        &self,
        velocity: &TendonVelocityView<'_, 'p>,
        target: &mut TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let input = velocity.tendon_passive_device()?;
        if !velocity.belongs_to(self.velocity)
            || input.worlds != worlds
            || target.len() != 2 * worlds * self.spec.nv + 8
            || status.len() != worlds + 8
        {
            return Err(TransferError::ModelMismatch);
        }
        let w = worlds as u32;
        // SAFETY: 速度视图锁定真实源。
        // 共享状态缓冲仅接收验证结果。
        unsafe {
            self.kernels[0].launch_tendon_passive(
                input.metadata,
                &self.parameters,
                input.source,
                input.velocity,
                status,
                [self.spec.nt as u32, self.spec.nv as u32, w, w],
            )?;
        }
        let checked = read(status, worlds, "passive_force_tendon_status")?;
        if let Some(index) = checked[4..4 + worlds].iter().position(|&v| v != 1.0) {
            return Err(InputError::InvalidTopology {
                field: "passive_force_tendon_source",
                index,
                reason: "guard_mapping_or_nonfinite",
            }
            .into());
        }
        let work = w
            .checked_mul(self.spec.nt as u32)
            .and_then(|n| n.checked_mul(self.spec.max_row as u32))
            .ok_or(InputError::Overflow {
                field: "passive_force_tendon_work",
            })?;
        // SAFETY: 每线程处理一个稀疏槽。
        // 目标先含关节贡献。
        unsafe {
            self.kernels[2].launch_tendon_passive(
                input.metadata,
                &self.parameters,
                input.source,
                input.velocity,
                target,
                [self.spec.nt as u32, self.spec.nv as u32, w, work],
            )
        }
    }

    pub fn update<'a>(
        &'a self,
        velocity: &'a TendonVelocityView<'a, 'p>,
        data: &'a mut TendonPassiveData,
    ) -> Result<TendonPassiveView<'a, 'v, 'p>, TransferError> {
        if !Arc::ptr_eq(&self.spec, &data.spec) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = velocity;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let input = velocity.tendon_passive_device()?;
            if !velocity.belongs_to(self.velocity) || input.worlds != data.worlds() {
                return Err(TransferError::ModelMismatch);
            }
            let worlds = data.worlds() as u32;
            let mut guards = [0.0; 8];
            data.current.read_range_into(0, &mut guards[..4])?;
            data.current
                .read_range_into(data.current.len() - 4, &mut guards[4..])?;
            crate::physics::check_device_guards(&guards)?;
            // SAFETY: 速度视图锁定真实源。
            // 常量限定全部读取跨度。
            unsafe {
                self.kernels[0].launch_tendon_passive(
                    input.metadata,
                    &self.parameters,
                    input.source,
                    input.velocity,
                    &mut data.status,
                    [self.spec.nt as u32, self.spec.nv as u32, worlds, worlds],
                )?;
            }
            let status = read(&data.status, data.worlds(), "tendon_passive_status")?;
            if let Some(index) = status[4..4 + data.worlds()]
                .iter()
                .position(|&value| value != 1.0)
            {
                return Err(InputError::InvalidTopology {
                    field: "tendon_passive_source",
                    index,
                    reason: "guard_mapping_or_nonfinite",
                }
                .into());
            }
            let slots = worlds * self.spec.nv as u32;
            // SAFETY: 清零线程各自独占结果槽。
            unsafe {
                self.kernels[1].launch_tendon_passive(
                    input.metadata,
                    &self.parameters,
                    input.source,
                    input.velocity,
                    &mut data.staged,
                    [0, 0, worlds, 2 * slots],
                )?;
            }
            let work = worlds
                .checked_mul(self.spec.nt as u32)
                .and_then(|n| n.checked_mul(self.spec.max_row as u32))
                .ok_or(InputError::Overflow {
                    field: "tendon_passive_work",
                })?;
            // SAFETY: 每线程处理一个稀疏槽。
            // 共享DOF严格使用设备原子加。
            unsafe {
                self.kernels[2].launch_tendon_passive(
                    input.metadata,
                    &self.parameters,
                    input.source,
                    input.velocity,
                    &mut data.staged,
                    [self.spec.nt as u32, self.spec.nv as u32, worlds, work],
                )?;
            }
            read(
                &data.staged,
                2 * data.layout.total_elements(),
                "tendon_passive_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            data.ready = true;
            Ok(TendonPassiveView {
                plan: self,
                velocity,
                data,
            })
        }
    }
}

impl TendonPassiveData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(super) fn corrupt_guard_for_test(
        &mut self,
        current: bool,
        index: usize,
        value: f32,
    ) -> Result<(), TransferError> {
        let buffer = if current {
            &mut self.current
        } else {
            &mut self.staged
        };
        buffer.write_range(index, &[value])
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(super) fn buffer_len_for_test(&self, current: bool) -> usize {
        if current {
            self.current.len()
        } else {
            self.staged.len()
        }
    }
}

impl TendonPassiveView<'_, '_, '_> {
    pub fn snapshot(&self) -> Result<TendonPassiveSnapshot, TransferError> {
        if !self.data.ready {
            return Err(TransferError::StageNotReady {
                stage: "tendon_passive",
            });
        }
        debug_assert!(self.velocity.belongs_to(self.plan.velocity));
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.velocity.tendon_passive_device()?;
            Ok(TendonPassiveSnapshot {
                layout: self.data.layout,
                values: read(
                    &self.data.current,
                    2 * self.data.layout.total_elements(),
                    "tendon_passive",
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
    let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut values)?;
    crate::physics::check_device_values(&values, field)?;
    Ok(values)
}

// 冻结来源：passive.py::_spring_damper_tendon_passive。
// 基线：71da24d，许可：Apache-2.0。
// 普通表达式保留上游融合许可。
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
__device__ float tp_poly(float linear,float a,float b,float x){float result=linear;result=result+a*x;result=result+(b*x)*x;return result;}
extern "C" __global__ void tendon_passive_validate(const int* meta,const float* p,const float* source,const float* velocity,float* status,unsigned nt,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=nt==TP_NT&&nv==TP_NV;
 for(unsigned k=0;k<4;++k)ok=ok&&meta[k]==-131072&&meta[4+2ull*TP_NT+TP_NNZ+k]==-131072&&p[k]==-131072.0f&&p[4+TP_PL+k]==-131072.0f&&source[k]==-131072.0f&&source[4+1ull*worlds*TP_STRIDE+k]==-131072.0f&&velocity[k]==-131072.0f&&velocity[4+1ull*worlds*TP_NT+k]==-131072.0f;
 for(unsigned i=0;i<TP_PL;++i)ok=ok&&isfinite(p[4+i]);
 for(unsigned t=0;t<TP_NT;++t){int start=meta[4+t],n=meta[4+TP_NT+t];if(start<0||n<0||start>TP_NNZ||n>TP_NNZ-start){ok=false;continue;}unsigned si=4+TP_OFF[0]+(w%TP_BATCH[0])*TP_NT+t,spi=4+TP_OFF[1]+2*((w%TP_BATCH[1])*TP_NT+t),di=4+TP_OFF[2]+(w%TP_BATCH[2])*TP_NT+t,dpi=4+TP_OFF[3]+2*((w%TP_BATCH[3])*TP_NT+t);bool has_s=!TP_S&&(p[si]!=0.0f||p[spi]!=0.0f||p[spi+1]!=0.0f),has_d=!TP_D&&(p[di]!=0.0f||p[dpi]!=0.0f||p[dpi+1]!=0.0f);if(n>0&&has_s)ok=ok&&isfinite(source[4+1ull*w*TP_STRIDE+t]);if(n>0&&has_d)ok=ok&&isfinite(velocity[4+1ull*w*TP_NT+t]);for(int i=0;i<n;++i){int c=meta[4+2ull*TP_NT+start+i];if(c<0||c>=TP_NV)ok=false;if(has_s||has_d)ok=ok&&isfinite(source[4+1ull*w*TP_STRIDE+TP_NT+start+i]);}}
 status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void tendon_passive_clear(const int* meta,const float* p,const float* source,const float* velocity,float* out,unsigned a,unsigned b,unsigned worlds,unsigned work){unsigned i=blockIdx.x*blockDim.x+threadIdx.x;if(i<work)out[4+i]=0.0f;}
extern "C" __global__ void tendon_passive_compute(const int* meta,const float* p,const float* source,const float* velocity,float* out,unsigned nt,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned k=tid%TP_MAX_ROW,q=tid/TP_MAX_ROW,t=q%TP_NT,w=q/TP_NT;unsigned si=4+TP_OFF[0]+(w%TP_BATCH[0])*TP_NT+t,spi=4+TP_OFF[1]+2*((w%TP_BATCH[1])*TP_NT+t),di=4+TP_OFF[2]+(w%TP_BATCH[2])*TP_NT+t,dpi=4+TP_OFF[3]+2*((w%TP_BATCH[3])*TP_NT+t);float s=p[si],sa=p[spi],sb=p[spi+1],d=p[di],da=p[dpi],db=p[dpi+1];bool has_s=!TP_S&&(s!=0.0f||sa!=0.0f||sb!=0.0f),has_d=!TP_D&&(d!=0.0f||da!=0.0f||db!=0.0f);if(!has_s&&!has_d)return;int n=meta[4+TP_NT+t];if(k>=(unsigned)n)return;int sparse=meta[4+t]+k,col=meta[4+2ull*TP_NT+sparse];float J=source[4+1ull*w*TP_STRIDE+TP_NT+sparse];
 if(has_s){unsigned li=4+TP_OFF[4]+2*((w%TP_BATCH[4])*TP_NT+t);float length=source[4+1ull*w*TP_STRIDE+t],lower=p[li],upper=p[li+1],x=length>upper?length-upper:(length<lower?length-lower:0.0f);float force=-x*tp_poly(s,sa,sb,x);atomicAdd(&out[4+1ull*w*TP_NV+col],J*force);}
 if(has_d){float v=velocity[4+1ull*w*TP_NT+t];float force=-v*tp_poly(d,da,db,fabsf(v));atomicAdd(&out[4+1ull*worlds*TP_NV+1ull*w*TP_NV+col],J*force);}
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn batch(period: usize, width: usize, value: f32) -> ParameterBatch {
        ParameterBatch::new(period, width, vec![value; period * width]).unwrap()
    }
    fn parameters(nt: usize, periods: [usize; 5]) -> TendonPassiveParameters {
        TendonPassiveParameters {
            stiffness: batch(periods[0], nt, 0.25),
            stiffness_poly: batch(periods[1], 2 * nt, 0.0),
            damping: batch(periods[2], nt, 0.125),
            damping_poly: batch(periods[3], 2 * nt, 0.0),
            length_spring: batch(periods[4], 2 * nt, 0.0),
        }
    }
    #[test]
    fn host_preserves_independent_parameter_periods() {
        let spec = spec((7, 3, 4, 25, 2), parameters(3, [2, 3, 5, 7, 11])).unwrap();
        assert_eq!(spec.offsets, [0, 6, 24, 39, 81]);
        assert_eq!(spec.parameter_len, 147);
        assert_eq!(
            spec.parameters.fields().map(ParameterBatch::batches),
            [2, 3, 5, 7, 11]
        );
        assert_eq!(layout(257, 7).unwrap().total_elements(), 1799);
    }
    #[test]
    fn host_rejects_bad_parameter_widths_and_work() {
        let mut parameters = parameters(3, [1; 5]);
        parameters.length_spring = batch(1, 5, 0.0);
        assert!(spec((7, 3, 4, 25, 2), parameters).is_err());
        assert!(layout(0, 7).is_err());
        assert!(layout(usize::MAX, 7).is_err());
    }
}
