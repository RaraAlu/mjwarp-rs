//! 原生CSR输入与冻结分块元数据。
//! 不生成物理矩阵或参考值。
use crate::diagnostics::InputError;
use std::collections::BTreeMap;

/// 原生下三角CSR。每行末项必须是对角。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SparseMassFields {
    pub dof_parentid: Vec<i32>,
    pub m_rownnz: Vec<i32>,
    pub m_rowadr: Vec<i32>,
    pub m_colind: Vec<i32>,
    pub tree_dofadr: Vec<i32>,
    pub tree_dofnum: Vec<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SparseMassRoute {
    Compact,
    SmallCholesky,
    TileCholesky,
    SparseLdl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SparseMassBlock {
    pub start: usize,
    pub size: usize,
    pub route: SparseMassRoute,
}

/// 保留冻结qLD块地址与分层顺序。
/// 稀疏大块只接受完整祖先CSR。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseMassInput {
    fields: SparseMassFields,
    blocks: Vec<SparseMassBlock>,
    qld_block_adr: Vec<i32>,
    qld_block_total: usize,
    updates: Vec<[i32; 3]>,
    level_offsets: Vec<i32>,
}

fn invalid(field: &'static str, index: usize, reason: &'static str) -> InputError {
    InputError::InvalidTopology {
        field,
        index,
        reason,
    }
}
fn length(field: &'static str, actual: usize, expected: usize) -> Result<(), InputError> {
    if actual != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual,
        });
    }
    Ok(())
}

impl SparseMassInput {
    pub fn new(fields: SparseMassFields) -> Result<Self, InputError> {
        let n = fields.dof_parentid.len();
        let nc = fields.m_colind.len();
        for size in [n, nc, fields.tree_dofadr.len()] {
            if size > i32::MAX as usize / 4 {
                return Err(InputError::Overflow {
                    field: "sparse_mass_metadata",
                });
            }
        }
        length("M_rownnz", fields.m_rownnz.len(), n)?;
        length("M_rowadr", fields.m_rowadr.len(), n)?;
        length(
            "tree_dofnum",
            fields.tree_dofnum.len(),
            fields.tree_dofadr.len(),
        )?;
        let mut next: usize = 0;
        for i in 0..n {
            let parent = fields.dof_parentid[i];
            if parent < -1 || parent >= i as i32 {
                return Err(invalid("dof_parentid", i, "parent_must_precede_child"));
            }
            let count = fields.m_rownnz[i];
            if count < 1 || fields.m_rowadr[i] != next as i32 {
                return Err(invalid("M_rowadr", i, "rows_must_partition_CSR"));
            }
            let end = next
                .checked_add(count as usize)
                .filter(|&v| v <= nc)
                .ok_or_else(|| invalid("M_rownnz", i, "row_out_of_range"))?;
            let row = &fields.m_colind[next..end];
            if row.last() != Some(&(i as i32)) || row.windows(2).any(|p| p[0] >= p[1]) {
                return Err(invalid(
                    "M_colind",
                    i,
                    "sorted_ancestors_then_diagonal_required",
                ));
            }
            let mut ancestor = i as i32;
            for &column in row.iter().rev() {
                while ancestor > column && ancestor >= 0 {
                    ancestor = fields.dof_parentid[ancestor as usize];
                }
                if column < 0 || ancestor != column {
                    return Err(invalid("M_colind", i, "column_is_not_ancestor"));
                }
            }
            next = end;
        }
        length("M_colind", nc, next)?;
        let mut blocks = Vec::new();
        let mut addresses = vec![-1; n];
        let mut packed = 0usize;
        let mut next_dof = 0;
        // Frozen io.m_block_layout (71da24d), Apache-2.0.
        // Copyright 2025 The Newton Developers; licenses/Apache-2.0.txt.
        for (&start, &size) in fields.tree_dofadr.iter().zip(&fields.tree_dofnum) {
            // 冻结_m_blocks按num过滤，不读取空树adr。
            if size == 0 {
                continue;
            }
            if start < 0 || size <= 0 || start as usize != next_dof {
                return Err(invalid(
                    "tree_dofadr",
                    next_dof,
                    "nonempty_trees_must_partition_dofs",
                ));
            }
            let start = start as usize;
            let size = size as usize;
            let end = start
                .checked_add(size)
                .filter(|&v| v <= n)
                .ok_or_else(|| invalid("tree_dofnum", start, "tree_out_of_range"))?;
            for i in start..end {
                let p = fields.dof_parentid[i];
                if (i == start && p != -1) || (i != start && p < start as i32) {
                    return Err(invalid("tree_dofadr", i, "tree_ancestry_mismatch"));
                }
            }
            let nnz = (fields.m_rowadr[end - 1] + fields.m_rownnz[end - 1] - fields.m_rowadr[start])
                as usize;
            let compact = nnz == size;
            let triangular = nnz == size * (size + 1) / 2;
            let route = if size <= 6 && (compact || triangular) {
                if compact {
                    SparseMassRoute::Compact
                } else {
                    SparseMassRoute::SmallCholesky
                }
            } else if size <= 64 {
                SparseMassRoute::TileCholesky
            } else {
                SparseMassRoute::SparseLdl
            };
            let address = match route {
                SparseMassRoute::Compact => -2,
                SparseMassRoute::SparseLdl => -1,
                _ => {
                    let address = packed;
                    packed = packed
                        .checked_add(size * size)
                        .filter(|&v| v <= i32::MAX as usize)
                        .ok_or(InputError::Overflow {
                            field: "qLD_block_total",
                        })?;
                    address as i32
                }
            };
            addresses[start..end].fill(address);
            blocks.push(SparseMassBlock { start, size, route });
            next_dof = end;
        }
        length("tree_dofnum", next_dof, n)?;
        let mut depth = vec![-1i32; n];
        let mut levels: BTreeMap<i32, Vec<[i32; 3]>> = BTreeMap::new();
        for k in 0..n {
            if fields.m_rownnz[k] == 1 {
                // 大块的压缩行可能破坏祖先前缀。
                if addresses[k] == -1 && fields.dof_parentid[k] >= 0 {
                    return Err(invalid("M_rownnz", k, "sparse_full_ancestor_rows_required"));
                }
                continue;
            }
            let parent = fields.dof_parentid[k];
            if parent < 0 {
                return Err(invalid("M_rownnz", k, "root_row_must_be_diagonal"));
            }
            depth[k] = depth[parent as usize] + 1;
            if addresses[k] != -1 {
                continue;
            }
            let mut i = parent;
            let mut address = fields.m_rowadr[k] + fields.m_rownnz[k] - 2;
            while i >= 0 {
                if address < fields.m_rowadr[k] || fields.m_colind[address as usize] != i {
                    return Err(invalid("M_colind", k, "sparse_full_ancestor_rows_required"));
                }
                let row_i = fields.m_rowadr[i as usize] as usize;
                let row_k = fields.m_rowadr[k] as usize;
                let count = fields.m_rownnz[i as usize] as usize;
                if row_k + count > address as usize + 1
                    || fields.m_colind[row_i..row_i + count]
                        != fields.m_colind[row_k..row_k + count]
                {
                    return Err(invalid("M_colind", k, "sparse_ancestor_prefix_mismatch"));
                }
                levels
                    .entry(depth[i as usize])
                    .or_default()
                    .push([i, k as i32, address]);
                i = fields.dof_parentid[i as usize];
                address -= 1;
            }
            if address != fields.m_rowadr[k] - 1 {
                return Err(invalid("M_rownnz", k, "sparse_full_ancestor_rows_required"));
            }
        }
        let mut updates = Vec::new();
        let mut offsets = vec![0];
        for layer in levels.into_values() {
            if updates
                .len()
                .checked_add(layer.len())
                .is_none_or(|v| v > i32::MAX as usize / 3)
            {
                return Err(InputError::Overflow {
                    field: "qLD_updates",
                });
            }
            updates.extend(layer);
            offsets.push(updates.len() as i32);
        }
        Ok(Self {
            fields,
            blocks,
            qld_block_adr: addresses,
            qld_block_total: packed,
            updates,
            level_offsets: offsets,
        })
    }
    pub fn nv(&self) -> usize {
        self.fields.dof_parentid.len()
    }
    pub fn nc(&self) -> usize {
        self.fields.m_colind.len()
    }
    pub fn fields(&self) -> &SparseMassFields {
        &self.fields
    }
    pub fn blocks(&self) -> &[SparseMassBlock] {
        &self.blocks
    }
    pub fn qld_block_adr(&self) -> &[i32] {
        &self.qld_block_adr
    }
    pub fn qld_block_total(&self) -> usize {
        self.qld_block_total
    }
    /// 零更新时保留冻结占位元组。
    pub fn qld_all_updates(&self) -> &[[i32; 3]] {
        if self.updates.is_empty() {
            &[[0, 0, 0]]
        } else {
            &self.updates
        }
    }
    pub fn qld_level_offsets(&self) -> &[i32] {
        &self.level_offsets
    }
    pub fn qld_updates(&self, level: usize) -> Result<&[[i32; 3]], InputError> {
        let levels = self.level_offsets.len() - 1;
        if level >= levels {
            return Err(InputError::InvalidIndex {
                field: "qLD_level",
                index: level,
                limit: levels,
            });
        }
        Ok(&self.updates
            [self.level_offsets[level] as usize..self.level_offsets[level + 1] as usize])
    }
}
