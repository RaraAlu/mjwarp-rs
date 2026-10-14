//! 动能消费者独占32线程ABI。
use super::{Arc, LaunchConfig, PushKernelArg, SynchronousKernel, TransferBuffer};
use crate::diagnostics::{InputError, TransferError};

impl SynchronousKernel {
    /// # Safety
    /// Caller proves static NV agrees with metadata and the compiled kernel.
    /// Each world owns one warp; all 32 lanes reach the ballot before branching.
    pub(crate) unsafe fn launch_kinetic_energy32(
        &self,
        metadata: &TransferBuffer<i32>,
        pair: &TransferBuffer<f32>,
        current: &TransferBuffer<f32>,
        staged: &mut TransferBuffer<f32>,
        dimensions: [u32; 4],
    ) -> Result<(), TransferError> {
        for same in [
            Arc::ptr_eq(&self.session.inner, &metadata.session.inner),
            Arc::ptr_eq(&self.session.inner, &pair.session.inner),
            Arc::ptr_eq(&self.session.inner, &current.session.inner),
            Arc::ptr_eq(&self.session.inner, &staged.session.inner),
        ] {
            if !same {
                return Err(TransferError::SessionMismatch);
            }
        }
        let [n, worlds, reserved, work] = dimensions.map(|v| v as usize);
        let invalid = || InputError::InvalidDimension {
            field: "kinetic_energy32_abi",
        };
        if !(1..=1024).contains(&n)
            || worlds == 0
            || worlds > i32::MAX as usize
            || reserved != 0
            || work != worlds
        {
            return Err(invalid().into());
        }
        let pair_len = worlds
            .checked_mul(2 * n)
            .and_then(|v| v.checked_add(8))
            .ok_or_else(invalid)?;
        let energy_len = worlds
            .checked_mul(2)
            .and_then(|v| v.checked_add(8))
            .ok_or_else(invalid)?;
        if metadata.len() != 1
            || pair.len() != pair_len
            || current.len() != energy_len
            || staged.len() != energy_len
        {
            return Err(invalid().into());
        }
        let (Some(m), Some(p), Some(c), Some(s)) = (
            metadata.allocation.as_ref(),
            pair.allocation.as_ref(),
            current.allocation.as_ref(),
            staged.allocation.as_mut(),
        ) else {
            return Err(invalid().into());
        };
        self.session.run("kinetic-energy-32-launch", || {
            let mut args = self
                .session
                .inner
                .stream
                .launch_builder(self.function.as_ref().unwrap());
            args.arg(m).arg(p).arg(c).arg(s);
            for value in &dimensions {
                args.arg(value);
            }
            // SAFETY: Four same-session allocations match the checked private ABI.
            // run serializes the stream and synchronizes before returning.
            unsafe {
                args.launch(LaunchConfig {
                    grid_dim: (worlds as u32, 1, 1),
                    block_dim: (32, 1, 1),
                    shared_mem_bytes: 0,
                })
            }?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::TransferSession;
    #[test]
    #[ignore = "new32-thread four-buffer ABI, lane/world geometry, capacities and foreign sessions"]
    fn gpu_launches_one_warp_per_world_and_rejects_each_foreign_or_malformed_buffer() {
        let s = TransferSession::new(0).unwrap();
        let other = TransferSession::new(0).unwrap();
        let source = r#"extern "C" __global__ void kinetic_energy32_abi_probe(const int* meta,const float* pair,const float* current,float* staged,unsigned n,unsigned worlds,unsigned unused,unsigned work){
         float value=(float)(threadIdx.x+1);for(unsigned o=16;o>0;o/=2)value+=__shfl_down_sync(0xffffffffu,value,o,32);
         if(threadIdx.x==0){unsigned w=blockIdx.x;staged[4+2ull*w]=current[4+2ull*w];staged[5+2ull*w]=value+pair[4+2ull*n*w]+meta[0]+w;}}"#;
        let k = SynchronousKernel::compile(&s, source, "kinetic_energy32_abi_probe").unwrap();
        let n = 65;
        let worlds = 257;
        let mut pv = vec![-131072.0; worlds * 2 * n + 8];
        let mut cv = vec![-131072.0; 2 * worlds + 8];
        for w in 0..worlds {
            pv[4 + 2 * n * w] = w as f32 / 4.0;
            cv[4 + 2 * w] = w as f32 / 8.0;
        }
        let m = s.upload(&[65i32]).unwrap();
        let p = s.upload(&pv).unwrap();
        let c = s.upload(&cv).unwrap();
        let mut out = s.upload(&cv).unwrap();
        let dims = [65, 257, 0, 257];
        // SAFETY: Trusted probe owns one warp per world and exactly matches
        // the checked four-buffer ABI; guarded offsets stay in each allocation.
        unsafe {
            k.launch_kinetic_energy32(&m, &p, &c, &mut out, dims)
                .unwrap();
        }
        let mut result = vec![0.0; out.len()];
        out.read_range_into(0, &mut result).unwrap();
        for w in 0..worlds {
            assert_eq!(result[4 + 2 * w].to_bits(), cv[4 + 2 * w].to_bits());
            assert_eq!(result[5 + 2 * w], 593.0 + 1.25 * w as f32);
        }
        assert_eq!(&result[..4], &cv[..4]);
        assert_eq!(&result[result.len() - 4..], &cv[cv.len() - 4..]);
        let mut rejects = 0;
        for len in [0, 2] {
            let bad = s.upload(&vec![0i32; len]).unwrap();
            // SAFETY: Invalid capacity must reject before any launch.
            assert!(unsafe { k.launch_kinetic_energy32(&bad, &p, &c, &mut out, dims) }.is_err());
            rejects += 1;
        }
        for len in [0, p.len() - 1, p.len() + 1] {
            let bad = s.upload(&vec![0.0; len]).unwrap();
            // SAFETY: Invalid capacity must reject before any launch.
            assert!(unsafe { k.launch_kinetic_energy32(&m, &bad, &c, &mut out, dims) }.is_err());
            rejects += 1;
        }
        for len in [0, c.len() - 1, c.len() + 1] {
            let mut bad = s.upload(&vec![0.0; len]).unwrap();
            // SAFETY: Both invalid capacity checks reject before launch.
            assert!(unsafe { k.launch_kinetic_energy32(&m, &p, &bad, &mut out, dims) }.is_err());
            // SAFETY: As above, with the invalid buffer in the output slot.
            assert!(unsafe { k.launch_kinetic_energy32(&m, &p, &c, &mut bad, dims) }.is_err());
            rejects += 2;
        }
        let fm = other.upload(&[65i32]).unwrap();
        let fp = other.upload(&pv).unwrap();
        let fc = other.upload(&cv).unwrap();
        let mut fo = other.upload(&cv).unwrap();
        // SAFETY: Foreign sessions must reject before allocating launch args.
        assert!(matches!(
            unsafe { k.launch_kinetic_energy32(&fm, &p, &c, &mut out, dims) },
            Err(TransferError::SessionMismatch)
        ));
        // SAFETY: As above, for the pair slot.
        assert!(matches!(
            unsafe { k.launch_kinetic_energy32(&m, &fp, &c, &mut out, dims) },
            Err(TransferError::SessionMismatch)
        ));
        // SAFETY: As above, for the current slot.
        assert!(matches!(
            unsafe { k.launch_kinetic_energy32(&m, &p, &fc, &mut out, dims) },
            Err(TransferError::SessionMismatch)
        ));
        // SAFETY: As above, for the staged slot.
        assert!(matches!(
            unsafe { k.launch_kinetic_energy32(&m, &p, &c, &mut fo, dims) },
            Err(TransferError::SessionMismatch)
        ));
        rejects += 4;
        for bad in [
            [0, 257, 0, 257],
            [1025, 257, 0, 257],
            [65, 0, 0, 0],
            [65, u32::MAX, 0, u32::MAX],
            [65, 257, 1, 257],
            [65, 257, 0, 256],
        ] {
            // SAFETY: Invalid dimensions must reject before launch.
            assert!(unsafe { k.launch_kinetic_energy32(&m, &p, &c, &mut out, bad) }.is_err());
            rejects += 1;
        }
        let mut final_bits = vec![0.0; out.len()];
        out.read_range_into(0, &mut final_bits).unwrap();
        assert_eq!(result, final_bits);
        assert_eq!(rejects, 21);
        println!(
            "kinetic_energy32_abi worlds=257 lanes=32 warp_sum=528 rejected=21 current_and_guards_preserved=true"
        );
    }
}
