//! 设备打包第二段。

#[cfg(feature = "cuda-probe")]
use super::pack::PackedModel;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn pack_fixed_tendon(
    model: &AttachedModelInput,
    fields: &FixedTendonFields,
) -> Result<PackedModel, TransferError> {
    let nt = fields.tendon_adr.len();
    let nw = fields.wrap_objid.len();
    let length = packed_length([1, nt, nt, nw, nw].into_iter())?;
    let mut metadata = crate::runtime::host_staging::<i32>(length)?;
    let mut parameters = crate::runtime::host_staging::<f32>(nw.max(1))?;
    metadata[0] = nw as i32;
    metadata[1..1 + nt].copy_from_slice(&fields.tendon_adr);
    metadata[1 + nt..1 + 2 * nt].copy_from_slice(&fields.tendon_num);
    parameters[..nw].copy_from_slice(&fields.wrap_prm);
    let k = model.rigid().kinematics().fields();
    for tendon in 0..nt {
        let row = fields.ten_j_rowadr[tendon] as usize;
        let columns = &fields.ten_j_colind[row..row + fields.ten_j_rownnz[tendon] as usize];
        let start = fields.tendon_adr[tendon] as usize;
        for i in start..start + fields.tendon_num[tendon] as usize {
            let joint = fields.wrap_objid[i] as usize;
            metadata[1 + 2 * nt + i] = k.jnt_qposadr[joint];
            // Model validation proved each reference exists in this sorted row.
            let slot = columns.binary_search(&k.jnt_dofadr[joint]).map_err(|_| {
                InputError::InvalidTopology {
                    field: "ten_J_colind",
                    index: row,
                    reason: "missing_tendon_dof_column",
                }
            })?;
            metadata[1 + 2 * nt + nw + i] = (row + slot) as i32;
        }
    }
    Ok(PackedModel {
        metadata,
        parameters,
    })
}

#[cfg(feature = "cuda-probe")]
pub(super) fn packed_length(mut lengths: impl Iterator<Item = usize>) -> Result<usize, InputError> {
    lengths.try_fold(0usize, |sum, len| {
        sum.checked_add(len)
            .filter(|&end| end <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "resident_parameter_pack",
            })
    })
}

#[cfg(feature = "cuda-probe")]
pub(super) fn pack_spatial_tendon(
    model: &AttachedModelInput,
    fields: &SpatialTendonFields,
    geometry: Option<&SpatialTendonGeometry>,
) -> Result<PackedModel, TransferError> {
    let k = model.rigid().kinematics();
    let f = k.fields();
    let nb = k.nbody();
    let nt = fields.tendon_adr.len();
    let nw = fields.wrap_type.len();
    let length = packed_length(
        [
            9,
            nb,
            nb,
            nb,
            nb,
            nt,
            nt,
            nt,
            nt,
            nw,
            nw,
            nw,
            nw,
            fields.ten_j_colind.len(),
        ]
        .into_iter(),
    )?;
    let mut metadata = crate::runtime::host_staging::<i32>(length)?;
    let size_count = geometry.map_or(0, |g| g.geom_size.values().len());
    let param_count = packed_length([nw, size_count].into_iter())?;
    let mut parameters = crate::runtime::host_staging::<f32>(param_count.max(1))?;
    let layout = AttachedLayout::new(1, model.ngeom(), model.nsite())?;
    metadata[..9].copy_from_slice(&[
        nt as i32,
        nw as i32,
        fields.ten_j_colind.len() as i32,
        nb as i32,
        layout.output.elements_per_world() as i32,
        (12 * model.ngeom()) as i32,
        model.ngeom() as i32,
        geometry.map_or(1, |g| g.geom_size.batches()) as i32,
        nw as i32,
    ]);
    metadata[9..9 + nb].copy_from_slice(&f.body_parentid);
    for body in 1..nb {
        let parent = f.body_parentid[body] as usize;
        metadata[9 + nb + body] = if parent == 0 {
            body as i32
        } else {
            metadata[9 + nb + parent]
        };
        let count = f.body_jntnum[body] as usize;
        if count > 0 {
            let start = f.body_jntadr[body] as usize;
            metadata[9 + 2 * nb + body] = f.jnt_dofadr[start];
            for joint in start..start + count {
                metadata[9 + 3 * nb + body] +=
                    crate::model::topology::JointType::try_from(f.jnt_type[joint])?.dof_width()
                        as i32;
            }
        }
    }
    let mut cursor = 9 + 4 * nb;
    for values in [
        &fields.tendon_adr,
        &fields.tendon_num,
        &fields.ten_j_rowadr,
        &fields.ten_j_rownnz,
        &fields.wrap_type,
        &fields.wrap_objid,
    ] {
        metadata[cursor..cursor + values.len()].copy_from_slice(values);
        cursor += values.len();
    }
    for (i, (&kind, &id)) in fields.wrap_type.iter().zip(&fields.wrap_objid).enumerate() {
        metadata[cursor + i] = match kind {
            3 => model.fields().site_bodyid[id as usize],
            4 | 5 => model.fields().geom_bodyid[id as usize],
            _ => -1,
        };
    }
    cursor += nw;
    for (i, &kind) in fields.wrap_type.iter().enumerate() {
        metadata[cursor + i] = if matches!(kind, 4 | 5) && fields.wrap_prm[i].round() >= 0.0 {
            fields.wrap_prm[i].round() as i32
        } else {
            -1
        };
    }
    cursor += nw;
    metadata[cursor..].copy_from_slice(&fields.ten_j_colind);
    for tendon in 0..nt {
        let mut scale = 1.0;
        let start = fields.tendon_adr[tendon] as usize;
        for (i, value) in parameters
            .iter_mut()
            .enumerate()
            .skip(start)
            .take(fields.tendon_num[tendon] as usize)
        {
            if fields.wrap_type[i] == 2 {
                scale = 1.0 / fields.wrap_prm[i];
            }
            *value = scale;
        }
    }
    if let Some(g) = geometry {
        parameters[nw..nw + size_count].copy_from_slice(g.geom_size.values());
    }
    Ok(PackedModel {
        metadata,
        parameters,
    })
}

#[cfg(feature = "cuda-probe")]
pub(super) fn pack_model(
    model: &AttachedModelInput,
    parameters: &KinematicsParameters,
    topology: &[&[i32]],
    fields: &[KinematicsParameter],
) -> Result<PackedModel, TransferError> {
    let metadata_len = packed_length(
        topology
            .iter()
            .map(|t| t.len())
            .chain(std::iter::once(2 * fields.len())),
    )?;
    let parameters_len = packed_length(fields.iter().map(|&f| parameters.values(f, model).len()))?;
    let mut metadata = crate::runtime::host_staging::<i32>(metadata_len)?;
    let mut values = crate::runtime::host_staging::<f32>(parameters_len.max(1))?;
    let mut cursor = 0;
    for t in topology {
        metadata[cursor..cursor + t.len()].copy_from_slice(t);
        cursor += t.len();
    }
    let mut offset = 0;
    for &f in fields {
        let v = parameters.values(f, model);
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

#[cfg(feature = "cuda-probe")]
pub(super) const FIELD_BATCH_CUDA: &str = r#"
#define MJWARP_FIELD_BATCHES
__device__ const float* field_parameter(const float* p,const int* desc,unsigned world,unsigned width) {
  return p+desc[0]+(unsigned long long)(world % (unsigned)desc[1])*width;
}
"#;
