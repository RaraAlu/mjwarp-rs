//! 常驻刚体模型的类型化上传。
//! 打包与校验仍在`resident.rs`；本文件只做入口适配。

use super::pack::PackedModel;
use crate::diagnostics::TransferError;
use crate::model::KinematicModelInput;
use crate::physics::rigid_kinematics::{RigidModel, RigidVariant};
use crate::runtime::TransferSession;

impl PackedModel {
    /// 刚体运动学入口在打包后先做类型化证明再上传。
    pub(super) fn upload_rigid(
        self,
        session: &TransferSession,
        equivalent: bool,
        kinematics: &KinematicModelInput,
        nmocap: usize,
    ) -> Result<RigidModel, TransferError> {
        let variant = if equivalent {
            RigidVariant::Equivalent
        } else {
            RigidVariant::Batch
        };
        RigidModel::upload(
            session,
            self.metadata,
            self.parameters,
            variant,
            [
                kinematics.nq() as u32,
                kinematics.nbody() as u32,
                kinematics.njnt() as u32,
                nmocap as u32,
            ],
        )
    }
}
