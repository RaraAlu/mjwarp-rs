//! 柔体边弹簧与阻尼被动力。
//! 本模块不代表完整被动力阶段。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, FlexEdgeFields};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
use std::sync::Arc;

use super::{KinematicsData, KinematicsPlan};

#[derive(Clone, Copy, Debug, Default)]
pub struct FlexEdgePassiveOptions {
    pub disable_spring: bool,
    pub disable_damper: bool,
}

/// 对应冻结上游的四项模型字段。
/// 刚度与阻尼按柔体编号索引。
#[derive(Clone, Debug)]
pub struct FlexEdgePassiveInput {
    pub length0: Vec<f32>,
    pub stiffness: Vec<f32>,
    pub damping: Vec<f32>,
    pub rigid: Vec<bool>,
}

struct Spec {
    nv: usize,
    ne: usize,
    nf: usize,
    nnz: usize,
    stride: usize,
    fields: Arc<FlexEdgeFields>,
    input: FlexEdgePassiveInput,
    flexid: Vec<i32>,
}

fn spec(
    nv: usize,
    fields: Arc<FlexEdgeFields>,
    input: FlexEdgePassiveInput,
) -> Result<Spec, InputError> {
    let ne = fields.nflexedge();
    let nf = fields.flex_edgeadr.len();
    for (field, actual, expected) in [
        ("flexedge_length0", input.length0.len(), ne),
        ("flex_edgestiffness", input.stiffness.len(), nf),
        ("flex_edgedamping", input.damping.len(), nf),
        ("flexedge_rigid", input.rigid.len(), ne),
    ] {
        if actual != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    if nv == 0 || ne == 0 || nf == 0 {
        return Err(InputError::InvalidDimension {
            field: "flex_edge_passive_dimensions",
        });
    }
    let nnz = fields.nnz();
    let stride = ne
        .checked_mul(2)
        .and_then(|n| n.checked_add(nnz))
        .filter(|&n| n <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "flex_edge_passive_stride",
        })?;
    let mut flexid = vec![-1; ne];
    for f in 0..nf {
        let start = fields.flex_edgeadr[f] as usize;
        let count = fields.flex_edgenum[f] as usize;
        flexid[start..start + count].fill(f as i32);
    }
    if flexid.iter().any(|&f| f < 0) {
        return Err(InputError::InvalidTopology {
            field: "flex_edgeflexid",
            index: 0,
            reason: "unclaimed_edge",
        });
    }
    Ok(Spec {
        nv,
        ne,
        nf,
        nnz,
        stride,
        fields,
        input,
        flexid,
    })
}

fn layout(worlds: usize, nv: usize) -> Result<BatchLayout, InputError> {
    if worlds == 0
        || worlds
            .checked_mul(nv)
            .and_then(|n| n.checked_mul(2))
            .is_none_or(|n| n > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "flex_edge_passive_work",
        });
    }
    BatchLayout::new(worlds, nv, 4)
}

pub struct FlexEdgePassivePlan<'p> {
    source: &'p KinematicsPlan,
    spec: Arc<Spec>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    parameters: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 3],
}

pub struct FlexEdgePassiveData {
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

pub struct FlexEdgePassiveView<'a, 'p> {
    plan: &'a FlexEdgePassivePlan<'p>,
    source: &'a KinematicsData,
    data: &'a FlexEdgePassiveData,
}

#[derive(Clone, Debug)]
pub struct FlexEdgePassiveSnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}

impl FlexEdgePassiveSnapshot {
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

impl<'p> FlexEdgePassivePlan<'p> {
    pub fn new(
        source: &'p KinematicsPlan,
        input: FlexEdgePassiveInput,
        options: FlexEdgePassiveOptions,
    ) -> Result<Self, TransferError> {
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (source, input, options);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let (session, nv, fields) = source.flex_edge_passive_context()?;
            let spec = spec(nv, fields, input)?;
            layout(1, nv)?;
            let mut metadata = vec![-131072; 4];
            metadata.extend(&spec.fields.flexedge_j_rowadr);
            metadata.extend(&spec.fields.flexedge_j_rownnz);
            metadata.extend(&spec.fields.flexedge_j_colind);
            metadata.extend(&spec.flexid);
            metadata.extend(spec.input.rigid.iter().map(|&v| i32::from(v)));
            metadata.extend([-131072; 4]);
            let mut parameters = vec![-131072.0; 4];
            parameters.extend(&spec.input.length0);
            parameters.extend(&spec.input.stiffness);
            parameters.extend(&spec.input.damping);
            parameters.extend([-131072.0; 4]);
            let code = format!(
                "#define FP_NV {nv}\n#define FP_NE {}\n#define FP_NF {}\n#define FP_NNZ {}\n#define FP_STRIDE {}\n#define FP_S {}\n#define FP_D {}\n{CUDA}",
                spec.ne,
                spec.nf,
                spec.nnz,
                spec.stride,
                u32::from(options.disable_spring),
                u32::from(options.disable_damper),
            );
            Ok(Self {
                source,
                spec: Arc::new(spec),
                metadata: session.upload(&metadata)?,
                parameters: session.upload(&parameters)?,
                kernels: [
                    SynchronousKernel::compile(&session, &code, "flex_edge_passive_validate")?,
                    SynchronousKernel::compile(&session, &code, "flex_edge_passive_clear")?,
                    SynchronousKernel::compile(&session, &code, "flex_edge_passive_compute")?,
                ],
                session,
            })
        }
    }

    pub fn create_data(&self, worlds: usize) -> Result<FlexEdgePassiveData, TransferError> {
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
            Ok(FlexEdgePassiveData {
                spec: self.spec.clone(),
                layout,
                ready: false,
                current: allocate(count, f32::NAN)?,
                staged: allocate(count, f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }

    pub fn update<'a>(
        &'a self,
        source: &'a KinematicsData,
        data: &'a mut FlexEdgePassiveData,
    ) -> Result<FlexEdgePassiveView<'a, 'p>, TransferError> {
        if !Arc::ptr_eq(&self.spec, &data.spec) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = source;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let input = self.source.flex_edge_passive_source(source)?;
            if source.worlds() != data.worlds() {
                return Err(TransferError::ModelMismatch);
            }
            let worlds = data.worlds() as u32;
            let mut guards = [0.0; 8];
            data.current.read_range_into(0, &mut guards[..4])?;
            data.current
                .read_range_into(data.current.len() - 4, &mut guards[4..])?;
            crate::physics::check_device_guards(&guards)?;
            // SAFETY: 计划锁定真实常驻边源。
            // 静态元数据限定全部跨度。
            unsafe {
                self.kernels[0].launch(
                    &self.metadata,
                    &self.parameters,
                    input,
                    &mut data.status,
                    [self.spec.ne as u32, self.spec.nv as u32, worlds, worlds],
                )?;
            }
            let status = read(&data.status, data.worlds(), "flex_edge_passive_status")?;
            if let Some(index) = status[4..4 + data.worlds()]
                .iter()
                .position(|&value| value != 1.0)
            {
                return Err(InputError::InvalidTopology {
                    field: "flex_edge_passive_source",
                    index,
                    reason: "guard_mapping_or_nonfinite",
                }
                .into());
            }
            let slots = worlds * self.spec.nv as u32;
            // SAFETY: 每线程只清零一个结果槽。
            unsafe {
                self.kernels[1].launch(
                    &self.metadata,
                    &self.parameters,
                    input,
                    &mut data.staged,
                    [0, 0, worlds, 2 * slots],
                )?;
            }
            let work = worlds
                .checked_mul(self.spec.ne as u32)
                .ok_or(InputError::Overflow {
                    field: "flex_edge_passive_work",
                })?;
            // SAFETY: 每线程处理一条边。
            // 共享自由度使用设备原子加。
            unsafe {
                self.kernels[2].launch(
                    &self.metadata,
                    &self.parameters,
                    input,
                    &mut data.staged,
                    [self.spec.ne as u32, self.spec.nv as u32, worlds, work],
                )?;
            }
            read(
                &data.staged,
                2 * data.layout.total_elements(),
                "flex_edge_passive_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            data.ready = true;
            Ok(FlexEdgePassiveView {
                plan: self,
                source,
                data,
            })
        }
    }
}

impl FlexEdgePassiveData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
}

impl FlexEdgePassiveView<'_, '_> {
    pub fn snapshot(&self) -> Result<FlexEdgePassiveSnapshot, TransferError> {
        if !self.data.ready {
            return Err(TransferError::StageNotReady {
                stage: "flex_edge_passive",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.plan.source.flex_edge_passive_source(self.source)?;
            Ok(FlexEdgePassiveSnapshot {
                layout: self.data.layout,
                values: read(
                    &self.data.current,
                    2 * self.data.layout.total_elements(),
                    "flex_edge_passive",
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

// 冻结来源：passive.py::_spring_damper_flexedge_passive。
// 基线：71da24d，许可：Apache-2.0。
// 普通表达式保留上游融合许可。
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void flex_edge_passive_validate(const int* m,const float* p,const float* source,float* status,unsigned ne,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=ne==FP_NE&&nv==FP_NV;
 const unsigned mend=4+4*FP_NE+FP_NNZ,pend=4+FP_NE+2*FP_NF;const unsigned long long send=4ull+(unsigned long long)worlds*FP_STRIDE;
 for(unsigned k=0;k<4;++k)ok=ok&&m[k]==-131072&&m[mend+k]==-131072&&p[k]==-131072.0f&&p[pend+k]==-131072.0f&&source[k]==-131072.0f&&source[send+k]==-131072.0f;
 const int* rowadr=m+4;const int* rownnz=rowadr+FP_NE;const int* colind=rownnz+FP_NE;const int* flexid=colind+FP_NNZ;const int* rigid=flexid+FP_NE;
 const float* length0=p+4;const float* stiffness=length0+FP_NE;const float* damping=stiffness+FP_NF;const float* edge=source+4+(unsigned long long)w*FP_STRIDE;
 for(unsigned e=0;e<FP_NE;++e){int adr=rowadr[e],n=rownnz[e],f=flexid[e];if(adr<0||n<0||adr>FP_NNZ||n>FP_NNZ-adr||f<0||f>=FP_NF){ok=false;continue;}if(rigid[e]!=0&&rigid[e]!=1)ok=false;if(rigid[e])continue;
  float s=0.0f,d=0.0f;if(!FP_S){s=stiffness[f];ok=ok&&isfinite(s);}if(!FP_D){d=damping[f];ok=ok&&isfinite(d);}if(s==0.0f&&d==0.0f)continue;if(n==0)continue;
  float fs=0.0f,fd=0.0f;if(s!=0.0f){float r=length0[e],x=edge[e];ok=ok&&isfinite(r)&&isfinite(x);fs=s*(r-x);ok=ok&&isfinite(fs);}if(d!=0.0f){float v=edge[FP_NE+e];ok=ok&&isfinite(v);fd=-d*v;ok=ok&&isfinite(fd);}if(fs==0.0f&&fd==0.0f)continue;
  for(int i=0;i<n;++i){int c=colind[adr+i];if(c<0||c>=FP_NV)ok=false;ok=ok&&isfinite(edge[2*FP_NE+adr+i]);}
 }
 status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void flex_edge_passive_clear(const int* m,const float* p,const float* source,float* out,unsigned a,unsigned b,unsigned worlds,unsigned work){unsigned i=blockIdx.x*blockDim.x+threadIdx.x;if(i<work)out[4+i]=0.0f;}
extern "C" __global__ void flex_edge_passive_compute(const int* m,const float* p,const float* source,float* out,unsigned ne,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned e=tid%FP_NE,w=tid/FP_NE;const int* rowadr=m+4;const int* rownnz=rowadr+FP_NE;const int* colind=rownnz+FP_NE;const int* flexid=colind+FP_NNZ;const int* rigid=flexid+FP_NE;if(rigid[e])return;int f=flexid[e];const float* length0=p+4;const float* stiffness=length0+FP_NE;const float* damping=stiffness+FP_NF;float s=FP_S?0.0f:stiffness[f],d=FP_D?0.0f:damping[f];if(s==0.0f&&d==0.0f)return;int adr=rowadr[e],n=rownnz[e];if(n==0)return;const float* edge=source+4+(unsigned long long)w*FP_STRIDE;float fs=0.0f,fd=0.0f;if(s!=0.0f)fs=s*(length0[e]-edge[e]);if(d!=0.0f)fd=-d*edge[FP_NE+e];if(fs==0.0f&&fd==0.0f)return;for(int i=0;i<n;++i){int c=colind[adr+i];float j=edge[2*FP_NE+adr+i];if(fs!=0.0f)atomicAdd(&out[4+(unsigned long long)w*FP_NV+c],j*fs);if(fd!=0.0f)atomicAdd(&out[4+(unsigned long long)worlds*FP_NV+(unsigned long long)w*FP_NV+c],j*fd);}
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn fields() -> Arc<FlexEdgeFields> {
        Arc::new(FlexEdgeFields {
            flex_edgeadr: vec![0, 2],
            flex_edgenum: vec![2, 1],
            flex_edge: vec![0, 1, 0, 1, 0, 1],
            flexedge_j_rowadr: vec![0, 2, 0],
            flexedge_j_rownnz: vec![2, 1, 0],
            flexedge_j_colind: vec![0, 2, 1],
        })
    }

    #[test]
    fn host_checks_native_dimensions_and_derives_flex_ids() {
        let input = FlexEdgePassiveInput {
            length0: vec![1.0, 2.0, f32::NAN],
            stiffness: vec![3.0, 4.0],
            damping: vec![5.0, 6.0],
            rigid: vec![false, true, false],
        };
        let s = spec(3, fields(), input.clone()).unwrap();
        assert_eq!(s.flexid, [0, 0, 1]);
        assert_eq!((s.ne, s.nf, s.nnz, s.stride), (3, 2, 3, 9));
        for field in 0..4 {
            let mut bad = input.clone();
            match field {
                0 => {
                    bad.length0.pop();
                }
                1 => {
                    bad.stiffness.pop();
                }
                2 => {
                    bad.damping.pop();
                }
                _ => {
                    bad.rigid.pop();
                }
            }
            assert!(spec(3, fields(), bad).is_err());
        }
        assert!(layout(0, 3).is_err());
        assert!(layout(usize::MAX, 3).is_err());
    }
}
