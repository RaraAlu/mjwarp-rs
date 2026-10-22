// Offline device caller for the pinned 11x11 MathDx POTRF LTO (analysis only).
extern "C" __device__ void potrf_11_11_1_89_32_1_1_5_x_x_0(float*, int*);

extern "C" __global__ void mathdx11_factor(const float* matrix, float* upper, int* status) {
    __shared__ __align__(16) float tile[121];
    __shared__ int info[1];
    const unsigned world = blockIdx.x;
    const unsigned lane = threadIdx.x;
    for (unsigned i = lane; i < 121; i += 32) tile[i] = matrix[world * 121 + i];
    if (lane == 0) info[0] = 0;
    __syncthreads();
    potrf_11_11_1_89_32_1_1_5_x_x_0(tile, info);
    __syncthreads();
    for (unsigned i = lane; i < 121; i += 32) {
        if (i / 11 > i % 11) tile[i] = 0.0f;
    }
    __syncthreads();
    for (unsigned i = lane; i < 121; i += 32) upper[world * 121 + i] = tile[i];
    if (lane == 0) status[world] = info[0];
}
