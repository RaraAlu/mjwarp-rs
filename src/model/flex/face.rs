//! 线性壳体面映射校验。

use crate::diagnostics::InputError;

use super::*;

impl FlexFaceFields {
    pub fn nflexface(&self) -> usize {
        self.flex_face_map.len() / 2
    }
    pub(super) fn validate(&self, positions: &FlexPositionFields) -> Result<(), InputError> {
        let overflow = || InputError::Overflow {
            field: "flex_face_fields",
        };
        let mut expected = 0usize;
        for f in 0..positions.flex_interp.len() {
            if positions.flex_interp[f] == -1 {
                expected = expected
                    .checked_add(face_count(&positions.flex_cellnum[3 * f..3 * f + 3])?)
                    .ok_or_else(overflow)?;
            }
        }
        expected
            .checked_mul(10)
            .and_then(|n| n.checked_add(2))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        expected
            .checked_mul(31)
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        for (field, actual, required) in [
            ("flex_face_map", self.flex_face_map.len(), 2 * expected),
            ("flex_face", self.flex_face.len(), 9 * expected),
        ] {
            if actual != required {
                return Err(InputError::LengthMismatch {
                    field,
                    expected: required,
                    actual,
                });
            }
        }
        let mut face = 0;
        for f in 0..positions.flex_interp.len() {
            if positions.flex_interp[f] != -1 {
                continue;
            }
            let cells = &positions.flex_cellnum[3 * f..3 * f + 3];
            for local in 0..face_count(cells)? {
                if self.flex_face_map[2 * face..2 * face + 2] != [f as i32, local as i32] {
                    return Err(InputError::InvalidTopology {
                        field: "flex_face_map",
                        index: face,
                        reason: "noncanonical_face_mapping",
                    });
                }
                let (_, nodes) = face_nodes(cells, local);
                for (slot, expected) in nodes
                    .into_iter()
                    .map(|n| positions.flex_nodeadr[f] + n)
                    .chain([-1; 5])
                    .enumerate()
                {
                    if self.flex_face[9 * face + slot] != expected {
                        return Err(InputError::InvalidTopology {
                            field: "flex_face",
                            index: 9 * face + slot,
                            reason: "noncanonical_face_nodes",
                        });
                    }
                }
                face += 1;
            }
        }
        Ok(())
    }
}
