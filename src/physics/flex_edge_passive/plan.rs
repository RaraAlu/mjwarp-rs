//! 柔体边被动力计划与更新。

#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
use super::*;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;
#[cfg(feature = "cuda-probe")]
use crate::runtime::SynchronousKernel;

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
                options,
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

    pub fn options(&self) -> FlexEdgePassiveOptions {
        self.options
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn shared_passive_context(
        &self,
    ) -> (TransferSession, usize, FlexEdgePassiveOptions) {
        (self.session.clone(), self.spec.nv, self.options)
    }

    // 默认配置无设备路径；仅cuda-probe读取。
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    pub(in crate::physics) fn shared_sleep_context(&self) -> Option<(usize, &[i32], bool, bool)> {
        self.source.passive_sleep_context()
    }

    pub(in crate::physics) fn shared_source_plan(&self) -> &'p KinematicsPlan {
        self.source
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn shared_sleep_source<'a>(
        &self,
        source: &'a KinematicsData,
    ) -> Result<&'a TransferBuffer<i32>, TransferError> {
        self.source.passive_sleep_source(source)
    }

    pub(in crate::physics) fn accepts_shared_source(&self, source: &KinematicsData) -> bool {
        self.source.check_data(source).is_ok()
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn matches_shared_model(
        &self,
        model: &crate::model::KinematicModelInput,
    ) -> bool {
        self.source.shared_kinematic_model() == model
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn accumulate_shared_passive(
        &self,
        source: &KinematicsData,
        target: &mut TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let input = self.source.flex_edge_passive_source(source)?;
        if source.worlds() != worlds
            || target.len() != 2 * worlds * self.spec.nv + 8
            || status.len() != worlds + 8
        {
            return Err(TransferError::ModelMismatch);
        }
        let w = worlds as u32;
        // SAFETY: 计划锁定常驻边源。
        // 共享状态缓冲仅接收验证结果。
        unsafe {
            self.kernels[0].launch(
                &self.metadata,
                &self.parameters,
                input,
                status,
                [self.spec.ne as u32, self.spec.nv as u32, w, w],
            )?;
        }
        let checked = read(status, worlds, "passive_force_flex_edge_status")?;
        if let Some(index) = checked[4..4 + worlds].iter().position(|&v| v != 1.0) {
            return Err(InputError::InvalidTopology {
                field: "passive_force_flex_edge_source",
                index,
                reason: "guard_mapping_or_nonfinite",
            }
            .into());
        }
        let work = w
            .checked_mul(self.spec.ne as u32)
            .ok_or(InputError::Overflow {
                field: "passive_force_flex_edge_work",
            })?;
        // SAFETY: 每线程处理一条边。
        // 目标已含关节和肌腱贡献。
        unsafe {
            self.kernels[2].launch(
                &self.metadata,
                &self.parameters,
                input,
                target,
                [self.spec.ne as u32, self.spec.nv as u32, w, work],
            )
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
