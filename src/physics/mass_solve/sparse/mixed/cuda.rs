//! 混合因子合并内核源。

#[cfg(feature = "cuda-probe")]
pub(super) const CUDA: &str = r#"
extern "C" __global__ void mixed_factor_merge(const int* metadata,const float* small,
 const float* sparse,float* output,unsigned prefix,unsigned nc,unsigned nv,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned stride=prefix+nc+nv,w=tid/stride,i=tid%stride;
 output[4+tid]=i<prefix?small[4+(unsigned long long)w*prefix+i]:
     sparse[4+(unsigned long long)w*(nc+nv)+i-prefix];
}
"#;
