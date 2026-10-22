// Offline device caller for the pinned 11x11 MathDx POTRS LTO (analysis only).
// Follows Warp's tile_cholesky_solve MathDx branch: fun_forward(U_ptr, x_ptr)
// with U the upper factor and x the RHS vector, both in shared memory.
extern "C" __device__ void potrs_11_11_1_89_64_1_1_5_x_x_0(float*, float*);

extern "C" __global__ void mathdx11_solve(const float* upper, const float* rhs, float* x) {
    __shared__ __align__(16) float tile[121];
    __shared__ __align__(16) float vector[11];
    __shared__ int info[1];
    const unsigned world = blockIdx.x;
    const unsigned lane = threadIdx.x;
    for (unsigned i = lane; i < 121; i += 64) tile[i] = upper[world * 121 + i];
    for (unsigned i = lane; i < 11; i += 64) vector[i] = rhs[world * 11 + i];
    if (lane == 0) info[0] = 0;
    __syncthreads();
    potrs_11_11_1_89_64_1_1_5_x_x_0(tile, vector);
    __syncthreads();
    for (unsigned i = lane; i < 11; i += 64) x[world * 11 + i] = vector[i];
    (void)info;
}
