//! 同步内核测试共用源码。

pub(super) const SOURCE: &str = r#"
extern "C" __global__ void adapter_probe(const int* m,const float* p,const float* s,
    float* o,unsigned a,unsigned b,unsigned c,unsigned n) {
  unsigned i=blockIdx.x*blockDim.x+threadIdx.x;
  if(i<n) o[i]=float(m[0])+p[0]+s[i]+float(a+b+c);
}
"#;
