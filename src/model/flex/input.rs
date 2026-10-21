//! 位置模型输入构造与可选字段接线。

use crate::diagnostics::InputError;
use crate::model::TendonModelInput;

use super::*;

impl FlexPositionModelInput {
    pub fn new(tendons: TendonModelInput, fields: FlexPositionFields) -> Result<Self, InputError> {
        let nb = tendons
            .spatial()
            .fixed()
            .camlight()
            .mocap()
            .attached()
            .rigid()
            .kinematics()
            .nbody();
        fields.validate(nb)?;
        Ok(Self {
            tendons,
            fields,
            edges: None,
            faces: None,
            hessian: None,
        })
    }
    /// 检查后启用边计算。
    /// 空行保留冻结上游语义。
    pub fn with_edges(mut self, fields: FlexEdgeFields) -> Result<Self, InputError> {
        let rigid = self
            .tendons
            .spatial()
            .fixed()
            .camlight()
            .mocap()
            .attached()
            .rigid();
        fields.validate(&self.fields, rigid)?;
        if let Some(hessian) = &self.hessian {
            hessian.validate(&self.fields, &fields)?;
        }
        self.edges = Some(fields);
        Ok(self)
    }
    pub fn edges(&self) -> Option<&FlexEdgeFields> {
        self.edges.as_ref()
    }
    /// 检查完整线性壳体面映射。
    pub fn with_faces(mut self, fields: FlexFaceFields) -> Result<Self, InputError> {
        fields.validate(&self.fields)?;
        self.faces = Some(fields);
        Ok(self)
    }
    pub fn faces(&self) -> Option<&FlexFaceFields> {
        self.faces.as_ref()
    }
    /// 派生冻结上游的线性壳体面。
    /// 复用既有节点编号与面校验。
    pub fn with_derived_faces(self) -> Result<Self, InputError> {
        let mut count = 0usize;
        for (f, &order) in self.fields.flex_interp.iter().enumerate() {
            if order == -1 {
                count = count
                    .checked_add(face_count(&self.fields.flex_cellnum[3 * f..3 * f + 3])?)
                    .filter(|&n| n <= i32::MAX as usize / 31)
                    .ok_or(InputError::Overflow {
                        field: "flex_face_fields",
                    })?;
            }
        }
        let mut faces = FlexFaceFields {
            flex_face_map: vec![0; 2 * count],
            flex_face: vec![-1; 9 * count],
        };
        let mut face = 0;
        for (f, &order) in self.fields.flex_interp.iter().enumerate() {
            if order != -1 {
                continue;
            }
            let cells = &self.fields.flex_cellnum[3 * f..3 * f + 3];
            for local in 0..face_count(cells)? {
                faces.flex_face_map[2 * face..2 * face + 2]
                    .copy_from_slice(&[f as i32, local as i32]);
                let (_, nodes) = face_nodes(cells, local);
                for (slot, node) in nodes.into_iter().enumerate() {
                    faces.flex_face[9 * face + slot] = self.fields.flex_nodeadr[f] + node;
                }
                face += 1;
            }
        }
        self.with_faces(faces)
    }
    /// 启用21系数拉伸矩阵。
    /// 调用方先启用边计算。
    pub fn with_hessian(mut self, fields: FlexHessianFields) -> Result<Self, InputError> {
        let edges = self.edges.as_ref().ok_or(InputError::InvalidDimension {
            field: "flex_hessian_requires_edges",
        })?;
        fields.validate(&self.fields, edges)?;
        self.hessian = Some(fields);
        Ok(self)
    }
    pub fn hessian(&self) -> Option<&FlexHessianFields> {
        self.hessian.as_ref()
    }
    pub fn tendons(&self) -> &TendonModelInput {
        &self.tendons
    }
    pub fn fields(&self) -> &FlexPositionFields {
        &self.fields
    }
    pub fn nflex(&self) -> usize {
        self.fields.flex_interp.len()
    }
    pub fn nflexnode(&self) -> usize {
        self.fields.flex_nodebodyid.len()
    }
    pub fn nflexvert(&self) -> usize {
        self.fields.flex_vertbodyid.len()
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        TendonModelInput,
        FlexPositionFields,
        Option<FlexEdgeFields>,
        Option<FlexFaceFields>,
        Option<FlexHessianFields>,
    ) {
        (
            self.tendons,
            self.fields,
            self.edges,
            self.faces,
            self.hessian,
        )
    }
}
