//! GPU肌腱唤醒与轻量树刷新。
//! 不维护身体或自由度活动表。

use crate::{diagnostics::InputError, model::BatchLayout};

#[derive(Clone, Copy, Debug)]
pub struct SleepTreeWorld<'a> {
    pub tree_asleep: &'a [i32],
    pub tree_awake: &'a [i32],
    pub ntree_awake: i32,
    pub nbody_awake: i32,
    pub nv_awake: i32,
}
#[derive(Clone, Debug)]
pub struct SleepTreeOutput {
    pub(super) layout: SleepTreeLayout,
    pub(super) values: Vec<i32>,
}
impl SleepTreeOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn ntree(&self) -> usize {
        self.layout.ntree
    }
    pub fn world(&self, world: usize) -> Result<SleepTreeWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[4 + r.start..4 + r.end];
        let n = self.ntree();
        Ok(SleepTreeWorld {
            tree_asleep: &v[..n],
            tree_awake: &v[n..2 * n],
            ntree_awake: v[2 * n],
            nbody_awake: v[2 * n + 1],
            nv_awake: v[2 * n + 2],
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) struct SleepTreeLayout {
    pub(super) output: BatchLayout,
    pub(super) ntree: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded: usize,
}
impl SleepTreeLayout {
    pub(super) fn new(worlds: usize, ntree: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize {
            return Err(InputError::InvalidDimension {
                field: "sleep_tree_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "sleep_tree_layout",
        };
        let width = ntree
            .checked_mul(2)
            .and_then(|n| n.checked_add(3))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let output = BatchLayout::new(worlds, width, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = guarded;
        Ok(Self {
            output,
            ntree,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

// Frozen sleep.py: _wake_tree, _wake_tendon_kernel, update_sleep_trees.
// Apache-2.0. One thread owns a world and visits global tendon order.
// Input awake flags stay unchanged until the final lightweight refresh.
#[cfg(feature = "cuda-probe")]
pub(super) const TENDON_WAKE_CUDA: &str = r#"
extern "C" __global__ void sleep_copy(const int* m,const float* params,
    const int* state,int* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  unsigned long long width=2ull*m[0]+3,offset=4+(unsigned long long)w*width;
  for(unsigned long long i=0;i<width;++i) result[offset+i]=state[offset+i];
}
__device__ void tendon_wake_tree(int* asleep,int ntree,int tree,int wakeval) {
  int old=asleep[tree];
  if(old<0) { if(wakeval<old) asleep[tree]=wakeval;return; }
  int current=tree;
  for(int step=0;step<=ntree;++step) {
    int next=asleep[current];if(next<0||next>=ntree) break;
    asleep[current]=wakeval;current=next;if(current==tree) break;
  }
}
extern "C" __global__ void tendon_wake(const int* m,const float* params,
    const float* lengths,int* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int n=m[0],nt=m[1];if(!m[6]||nt==0) return;
  int* asleep=result+4+(unsigned long long)w*(2ull*n+3);int* awake=asleep+n;
  const int* adr=m+7;const int* num=adr+nt;const int* limited=num+nt;const int* trees=limited+nt;
  const float* range=params+(unsigned long long)(w%m[3])*2*nt;
  const float* margin=params+2ull*nt*m[3]+(unsigned long long)(w%m[4])*nt;
  const float* length=lengths+4+(unsigned long long)w*m[5];
  for(int t=0;t<nt;++t) {
    int any_awake=0,wakeval=-11;
    for(int i=0;i<num[t];++i) {
      int tree=trees[adr[t]+i];if(tree>=0&&awake[tree]==1) {
        any_awake=1;if(asleep[tree]<wakeval) wakeval=asleep[tree];
      }
    }
    if(any_awake&&limited[t]&&
       (length[t]-range[2*t]<margin[t]||range[2*t+1]-length[t]<margin[t])) {
      for(int i=0;i<num[t];++i) {
        int tree=trees[adr[t]+i];if(tree>=0&&awake[tree]==0) tendon_wake_tree(asleep,n,tree,wakeval);
      }
    }
  }
  int count=0;for(int i=0;i<n;++i) { awake[i]=asleep[i]<0;count+=awake[i]; }
  awake[n]=count;awake[n+1]=0;awake[n+2]=0;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_overflow_and_keeps_zero_tree_counters() {
        let l = SleepTreeLayout::new(5, 0).unwrap();
        assert_eq!(l.output.total_elements(), 15);
        for (w, n) in [(0, 0), (u32::MAX as usize, 0), (1, usize::MAX)] {
            assert!(SleepTreeLayout::new(w, n).is_err());
        }
    }
}
