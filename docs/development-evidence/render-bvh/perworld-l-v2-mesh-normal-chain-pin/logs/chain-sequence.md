# offline NVRTC probe: mesh normal chain instruction sequences (product options, no GPU)

probe sources: logs/offline-probe-old.cu / logs/offline-probe-new.cu (functions extracted byte-identically from
HEAD render_math.cu + scene.cu::mat_vec and from the fixed worktree render_math.cu respectively), compiled with
pinned CUDA 12.8.1 NVRTC, --fmad=true --std=c++17 --gpu-architecture=compute_70 --device-as-default-execution-space.
Registers: probe params are (mat, tri, out); %rd5=mat pointer base, %rd4=tri/n_local pointer base.

## OLD chain (16 f32 arithmetic ops): mat_vec y-term rounded separately first, then ONE normalize
```text
ld.global.nc.f32 	%f14, [%rd5+4];
ld.global.nc.f32 	%f15, [%rd4+4];
mul.f32 	%f16, %f15, %f14;
ld.global.nc.f32 	%f17, [%rd5];
ld.global.nc.f32 	%f18, [%rd4];
fma.rn.f32 	%f19, %f17, %f18, %f16;
ld.global.nc.f32 	%f20, [%rd5+8];
ld.global.nc.f32 	%f21, [%rd4+8];
fma.rn.f32 	%f1, %f20, %f21, %f19;
ld.global.nc.f32 	%f22, [%rd5+16];
mul.f32 	%f23, %f15, %f22;
ld.global.nc.f32 	%f24, [%rd5+12];
fma.rn.f32 	%f25, %f24, %f18, %f23;
ld.global.nc.f32 	%f26, [%rd5+20];
fma.rn.f32 	%f2, %f26, %f21, %f25;
ld.global.nc.f32 	%f27, [%rd5+28];
mul.f32 	%f28, %f15, %f27;
ld.global.nc.f32 	%f29, [%rd5+24];
fma.rn.f32 	%f30, %f29, %f18, %f28;
ld.global.nc.f32 	%f31, [%rd5+32];
fma.rn.f32 	%f3, %f31, %f21, %f30;
mul.f32 	%f32, %f2, %f2;
fma.rn.f32 	%f33, %f1, %f1, %f32;
fma.rn.f32 	%f34, %f3, %f3, %f33;
sqrt.rn.f32 	%f4, %f34;
div.rn.f32 	%f37, %f3, %f4;
div.rn.f32 	%f36, %f2, %f4;
div.rn.f32 	%f35, %f1, %f4;
```

## NEW chain (23 f32 arithmetic ops = old chain + 7 for the restored local normalize)
```text
ld.global.nc.f32 	%f1, [%rd4];
ld.global.nc.f32 	%f2, [%rd4+4];
mul.f32 	%f24, %f2, %f2;
fma.rn.f32 	%f25, %f1, %f1, %f24;
ld.global.nc.f32 	%f3, [%rd4+8];
fma.rn.f32 	%f26, %f3, %f3, %f25;
sqrt.rn.f32 	%f4, %f26;
div.rn.f32 	%f50, %f3, %f4;
div.rn.f32 	%f49, %f2, %f4;
div.rn.f32 	%f48, %f1, %f4;
ld.global.nc.f32 	%f30, [%rd5];
mul.f32 	%f31, %f48, %f30;
ld.global.nc.f32 	%f32, [%rd5+4];
fma.rn.f32 	%f33, %f32, %f49, %f31;
ld.global.nc.f32 	%f34, [%rd5+8];
fma.rn.f32 	%f11, %f34, %f50, %f33;
ld.global.nc.f32 	%f35, [%rd5+12];
mul.f32 	%f36, %f48, %f35;
ld.global.nc.f32 	%f37, [%rd5+16];
fma.rn.f32 	%f38, %f37, %f49, %f36;
ld.global.nc.f32 	%f39, [%rd5+20];
fma.rn.f32 	%f12, %f39, %f50, %f38;
ld.global.nc.f32 	%f40, [%rd5+24];
mul.f32 	%f41, %f48, %f40;
ld.global.nc.f32 	%f42, [%rd5+28];
fma.rn.f32 	%f43, %f42, %f49, %f41;
ld.global.nc.f32 	%f44, [%rd5+32];
fma.rn.f32 	%f13, %f44, %f50, %f43;
mul.f32 	%f45, %f12, %f12;
fma.rn.f32 	%f46, %f11, %f11, %f45;
fma.rn.f32 	%f47, %f13, %f13, %f46;
sqrt.rn.f32 	%f14, %f47;
div.rn.f32 	%f53, %f13, %f14;
div.rn.f32 	%f52, %f12, %f14;
div.rn.f32 	%f51, %f11, %f14;
```

Interpretation (load offsets give the operand provenance):
- OLD: first `mul.f32` of each matrix row multiplies `[%rd5+4]` (m[1]) by `[%rd4+4]` (a.y)  => y term is the separately rounded term; then fma m[0]*a.x, fma m[2]*a.z; one normalize afterwards.
- NEW: first `mul.f32` of each row multiplies `[%rd5+0]` (m[0]) by `%f48` (= n_local.x after the local normalize) => x term is the separately rounded term; then fma m[1]*n_local.y, fma m[2]*n_local.z; a second normalize follows.
- NEW local normalize on the triangle normal: mul (y*y) -> fma (x*x) -> fma (z*z) -> sqrt.rn -> 3x div.rn (same plain normalize3_mesh body as before).
