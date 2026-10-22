//! 设备打包第一段。

#[cfg(feature = "cuda-probe")]
use super::pack_batches::packed_length;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) const RIGID_PARAMETERS: [KinematicsParameter; 7] = [
    KinematicsParameter::Qpos0,
    KinematicsParameter::BodyPos,
    KinematicsParameter::BodyQuat,
    KinematicsParameter::BodyIpos,
    KinematicsParameter::BodyIquat,
    KinematicsParameter::JointPos,
    KinematicsParameter::JointAxis,
];
#[cfg(feature = "cuda-probe")]
pub(super) const COM_PARAMETERS: [KinematicsParameter; 2] = [
    KinematicsParameter::BodyMass,
    KinematicsParameter::BodyInertia,
];
#[cfg(feature = "cuda-probe")]
pub(super) const ATTACHED_PARAMETERS: [KinematicsParameter; 4] = [
    KinematicsParameter::GeomPos,
    KinematicsParameter::GeomQuat,
    KinematicsParameter::SitePos,
    KinematicsParameter::SiteQuat,
];

#[cfg(feature = "cuda-probe")]
pub(super) struct PackedModel {
    pub(super) metadata: Vec<i32>,
    pub(super) parameters: Vec<f32>,
}

#[cfg(feature = "cuda-probe")]
pub(super) fn pack_flex_hessian(
    positions: &FlexPositionFields,
    edges: &FlexEdgeFields,
    fields: &FlexHessianFields,
    position_stride: usize,
    edge_stride: usize,
) -> Result<PackedModel, TransferError> {
    let nf = positions.flex_interp.len();
    let elem_offset = 9 + 11 * nf;
    let elemedge_offset = elem_offset + fields.flex_elem.len();
    let edge_offset = elemedge_offset + fields.flex_elemedge.len();
    let mut metadata = crate::runtime::host_staging::<i32>(edge_offset + edges.flex_edge.len())?;
    metadata[..9].copy_from_slice(&[
        nf as i32,
        positions.flex_vertbodyid.len() as i32,
        edges.nflexedge() as i32,
        position_stride as i32,
        edge_stride as i32,
        elem_offset as i32,
        elemedge_offset as i32,
        edge_offset as i32,
        (3 * positions.flex_nodebodyid.len()) as i32,
    ]);
    for f in 0..nf {
        metadata[9 + 11 * f..20 + 11 * f].copy_from_slice(&[
            fields.flex_dim[f],
            i32::from(fields.active(positions, f)),
            positions.flex_vertadr[f],
            positions.flex_vertnum[f],
            edges.flex_edgeadr[f],
            edges.flex_edgenum[f],
            fields.flex_elemadr[f],
            fields.flex_elemnum[f],
            fields.flex_elemdataadr[f],
            fields.flex_elemedgeadr[f],
            fields.flex_stiffnessadr[f],
        ]);
    }
    metadata[elem_offset..elemedge_offset].copy_from_slice(&fields.flex_elem);
    metadata[elemedge_offset..edge_offset].copy_from_slice(&fields.flex_elemedge);
    metadata[edge_offset..].copy_from_slice(&edges.flex_edge);
    let mut parameters = crate::runtime::host_staging::<f32>(
        (fields.flexedge_length0.len() + fields.flex_stiffness.len()).max(1),
    )?;
    let n = fields.flexedge_length0.len();
    parameters[..n].copy_from_slice(&fields.flexedge_length0);
    parameters[n..n + fields.flex_stiffness.len()].copy_from_slice(&fields.flex_stiffness);
    Ok(PackedModel {
        metadata,
        parameters,
    })
}

#[cfg(feature = "cuda-probe")]
pub(super) fn pack_flex_faces(
    positions: &FlexPositionFields,
    faces: &FlexFaceFields,
    stride: usize,
) -> Result<PackedModel, TransferError> {
    let length = packed_length([2, faces.nflexface() * 10].into_iter())?;
    let mut metadata = crate::runtime::host_staging::<i32>(length)?;
    metadata[..2].copy_from_slice(&[faces.nflexface() as i32, stride as i32]);
    for face in 0..faces.nflexface() {
        let flex = faces.flex_face_map[2 * face] as usize;
        let local = faces.flex_face_map[2 * face + 1] as usize;
        let axis =
            crate::model::flex::face_nodes(&positions.flex_cellnum[3 * flex..3 * flex + 3], local)
                .0;
        metadata[2 + 10 * face] = axis;
        metadata[3 + 10 * face..12 + 10 * face]
            .copy_from_slice(&faces.flex_face[9 * face..9 * face + 9]);
    }
    Ok(PackedModel {
        metadata,
        parameters: vec![0.0],
    })
}

#[cfg(feature = "cuda-probe")]
pub(super) fn pack_sleep(
    info: &TendonWakeInfo,
    rows: &TendonRows,
) -> Result<PackedModel, TransferError> {
    let f = &info.fields;
    let l = TendonLayout::new(1, rows.ntendon(), rows.nnz(), rows.nwrap())?;
    let mut metadata = vec![
        info.ntree as i32,
        rows.ntendon() as i32,
        info.wrap_treeid.len() as i32,
        f.tendon_range.batches() as i32,
        f.tendon_margin.batches() as i32,
        l.output.elements_per_world() as i32,
        i32::from(f.sleep_enabled && !f.island_disabled),
    ];
    metadata.extend(&info.tendon_adr);
    metadata.extend(&info.tendon_num);
    metadata.extend(f.tendon_limited.iter().copied().map(i32::from));
    metadata.extend(&info.wrap_treeid);
    let mut parameters = f.tendon_range.values().to_vec();
    parameters.extend(f.tendon_margin.values());
    if parameters.is_empty() {
        parameters.push(0.0);
    }
    Ok(PackedModel {
        metadata,
        parameters,
    })
}

#[cfg(feature = "cuda-probe")]
pub(super) fn pack_flex_edges(
    model: &AttachedModelInput,
    f: &FlexPositionFields,
    e: &FlexEdgeFields,
    com_stride: usize,
    flex_stride: usize,
) -> Result<Vec<i32>, TransferError> {
    let rigid = model.rigid();
    let k = rigid.kinematics();
    let nb = k.nbody();
    let ne = e.nflexedge();
    let nnz = e.nnz();
    let nvert = f.flex_vertbodyid.len();
    let length = packed_length([7, 4 * ne, nnz, nvert, nb, nb, k.nv()].into_iter())?;
    let mut m = crate::runtime::host_staging::<i32>(length)?;
    m[..7].copy_from_slice(&[
        nb as i32,
        k.nv() as i32,
        ne as i32,
        nnz as i32,
        com_stride as i32,
        flex_stride as i32,
        nvert as i32,
    ]);
    for flex in 0..f.flex_interp.len() {
        let start = e.flex_edgeadr[flex] as usize;
        let end = start + e.flex_edgenum[flex] as usize;
        for edge in start..end {
            m[7 + 4 * edge..7 + 4 * edge + 4].copy_from_slice(&[
                f.flex_vertadr[flex] + e.flex_edge[2 * edge],
                f.flex_vertadr[flex] + e.flex_edge[2 * edge + 1],
                e.flexedge_j_rowadr[edge],
                e.flexedge_j_rownnz[edge],
            ]);
        }
    }
    let mut cursor = 7 + 4 * ne;
    for values in [
        &e.flexedge_j_colind,
        &f.flex_vertbodyid,
        &k.fields().body_parentid,
    ] {
        m[cursor..cursor + values.len()].copy_from_slice(values);
        cursor += values.len();
    }
    for body in 1..nb {
        let parent = k.fields().body_parentid[body] as usize;
        m[cursor + body] = if parent == 0 {
            body as i32
        } else {
            m[cursor + parent]
        };
    }
    cursor += nb;
    m[cursor..].copy_from_slice(&rigid.fields().dof_bodyid);
    Ok(m)
}

#[cfg(feature = "cuda-probe")]
pub(super) fn pack_flex(
    f: &FlexPositionFields,
    rigid_stride: usize,
    nb: usize,
) -> Result<PackedModel, TransferError> {
    let (nf, nn, nv) = (
        f.flex_interp.len(),
        f.flex_nodebodyid.len(),
        f.flex_vertbodyid.len(),
    );
    let mut metadata = crate::runtime::host_staging::<i32>(5 + 9 * nf + nn + nv)?;
    metadata[..5].copy_from_slice(&[
        nf as i32,
        nn as i32,
        nv as i32,
        nb as i32,
        rigid_stride as i32,
    ]);
    for i in 0..nf {
        metadata[5 + 9 * i..5 + 9 * i + 9].copy_from_slice(&[
            f.flex_interp[i],
            f.flex_cellnum[3 * i],
            f.flex_cellnum[3 * i + 1],
            f.flex_cellnum[3 * i + 2],
            f.flex_nodeadr[i],
            f.flex_nodenum[i],
            f.flex_vertadr[i],
            f.flex_vertnum[i],
            i32::from(f.flex_centered[i]),
        ]);
    }
    metadata[5 + 9 * nf..5 + 9 * nf + nn].copy_from_slice(&f.flex_nodebodyid);
    metadata[5 + 9 * nf + nn..].copy_from_slice(&f.flex_vertbodyid);
    let length = (6 * nv + 3 * nn).max(1);
    let mut parameters = crate::runtime::host_staging::<f32>(length)?;
    parameters[..3 * nv].copy_from_slice(&f.flex_vert);
    parameters[3 * nv..6 * nv].copy_from_slice(&f.flex_vert0);
    parameters[6 * nv..6 * nv + 3 * nn].copy_from_slice(&f.flex_node);
    Ok(PackedModel {
        metadata,
        parameters,
    })
}

#[cfg(feature = "cuda-probe")]
pub(super) fn pack_tendon(
    rows: &TendonRows,
    fixed: &FixedTendonRows,
    spatial: &SpatialTendonRows,
) -> Result<PackedModel, TransferError> {
    let length = packed_length([8, rows.ntendon() * 5].into_iter())?;
    let mut metadata = crate::runtime::host_staging::<i32>(length)?;
    metadata[..8].copy_from_slice(&[
        rows.ntendon() as i32,
        rows.nnz() as i32,
        rows.nwrap() as i32,
        fixed.ntendon() as i32,
        fixed.nnz() as i32,
        spatial.ntendon() as i32,
        spatial.nnz() as i32,
        spatial.nwrap() as i32,
    ]);
    for t in 0..rows.ntendon() {
        let (kind, local, offset) = match rows.subset(t)? {
            TendonSubset::Fixed(local) => (0, local, fixed.rowadr()[local]),
            TendonSubset::Spatial(local) => (1, local, spatial.rowadr()[local]),
        };
        metadata[8 + 5 * t..13 + 5 * t].copy_from_slice(&[
            kind,
            local as i32,
            offset,
            rows.rowadr()[t],
            rows.rownnz()[t],
        ]);
    }
    Ok(PackedModel {
        metadata,
        parameters: vec![0.0],
    })
}

#[cfg(feature = "cuda-probe")]
pub(super) fn pack_camlight(
    fields: &CamLightFields,
    parameters: &CamLightParameters,
    rigid_stride: usize,
    com_stride: usize,
) -> Result<PackedModel, TransferError> {
    let header = [rigid_stride as i32, com_stride as i32];
    let topology: [&[i32]; 7] = [
        &header,
        &fields.cam_mode,
        &fields.cam_bodyid,
        &fields.cam_targetbodyid,
        &fields.light_mode,
        &fields.light_bodyid,
        &fields.light_targetbodyid,
    ];
    let metadata_len = packed_length(topology.iter().map(|t| t.len()).chain(std::iter::once(20)))?;
    let parameters_len = packed_length(
        CamLightParameter::ALL
            .iter()
            .map(|&f| parameters.values(f, fields).len()),
    )?;
    let mut metadata = crate::runtime::host_staging::<i32>(metadata_len)?;
    let mut values = crate::runtime::host_staging::<f32>(parameters_len.max(1))?;
    let mut cursor = 0;
    for t in topology {
        metadata[cursor..cursor + t.len()].copy_from_slice(t);
        cursor += t.len();
    }
    let mut offset = 0;
    for f in CamLightParameter::ALL {
        let v = parameters.values(f, fields);
        metadata[cursor] = offset as i32;
        metadata[cursor + 1] = parameters.batches(f) as i32;
        values[offset..offset + v.len()].copy_from_slice(v);
        offset += v.len();
        cursor += 2;
    }
    Ok(PackedModel {
        metadata,
        parameters: values,
    })
}
