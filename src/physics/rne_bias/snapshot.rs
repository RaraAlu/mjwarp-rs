//! 偏置力快照与世界视图。

use super::*;

impl RneBiasSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<RneBiasWorld<'_>, InputError> {
        self.layout.output.world_elements(world)?;
        let start = world * self.nv();
        Ok(RneBiasWorld {
            qfrc_bias: &self.values[start..start + self.nv()],
        })
    }
}

// Adapted from frozen smooth.py RNE and math.py spatial inertia/force algebra.
// Copyright 2025 The Newton Developers.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
