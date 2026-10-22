//! 肌腱速度计划构造、写入与更新入口。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::tendon_velocity_validate_launch_error;
use super::*;

impl<'p> TendonVelocityPlan<'p> {
    /// 被动力计划构造时复制冻结元数据宿主镜像。
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn passive_metadata(&self) -> &[i32] {
        self.metadata.passive_metadata()
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn tendon_passive_context(
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
            let static_dims = TendonVelocityStaticDims::new(nv, s.nt, s.nnz, s.stride)
                .map_err(tendon_velocity_validate_launch_error)?;
            let metadata =
                TendonVelocityMetadataBuffer::upload(&session, s.meta.clone(), static_dims)?;
            let validate = TendonVelocityValidateKernel::compile(&session, &code)?;
            let compute = TendonVelocityComputeKernel::compile(&session, &code)?;
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
            // 类型包装核对证明、精确跨度、同会话、别名与256线程网格。
            // 验证先拒绝坏行列号。
            self.validate
                .launch(&self.metadata, j, &data.qvel, &mut data.status, w)?;
            let status = read(&data.status, data.worlds(), "tendon_velocity_status")?;
            if let Some(index) = status[4..4 + data.worlds()].iter().position(|&x| x != 1.0) {
                return Err(InputError::InvalidTopology {
                    field: "tendon_velocity_source",
                    index,
                    reason: "guard_mapping_or_nonfinite",
                }
                .into());
            }
            // 类型包装核对证明、跨度、会话、别名与网格；每线程只写一项。
            self.compute
                .launch(&self.metadata, j, &data.qvel, &mut data.staged, w)?;
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
