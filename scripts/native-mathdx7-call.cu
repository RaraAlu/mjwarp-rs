// Offline device caller for the pinned native POTRF LTO, not a product kernel.
// The shared row-major pointer, cooperative call, barriers and lower clearing
// follow Warp's public tile_cholesky_inplace_impl MathDx branch.
extern "C" __device__ void potrf_7_7_1_89_32_1_1_5_x_x_0(float*, int*);

extern "C" __global__ void mathdx7_factor(const float* matrix, float* upper, int* status) {
    __shared__ __align__(16) float tile[49];
    __shared__ int info[1];
    const unsigned world = blockIdx.x;
    const unsigned lane = threadIdx.x;
    for (unsigned i = lane; i < 49; i += 32) tile[i] = matrix[world * 49 + i];
    if (lane == 0) info[0] = 0;
    __syncthreads();
    potrf_7_7_1_89_32_1_1_5_x_x_0(tile, info);
    __syncthreads();
    for (unsigned i = lane; i < 49; i += 32) {
        if (i / 7 > i % 7) tile[i] = 0.0f;
    }
    __syncthreads();
    for (unsigned i = lane; i < 49; i += 32) upper[world * 49 + i] = tile[i];
    if (lane == 0) status[world] = info[0];
}
