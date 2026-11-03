// GPU camera-space ray builder (render/BVH S5).
//
// ABI (SynchronousKernel four-buffer launch):
//   extern "C" __global__ void build_rays(const int* meta, const float* params,
//       const float* state, float* out, unsigned d0, unsigned d1, unsigned d2, unsigned d3);
//
// meta:   [ncam, samples(spp^2), spp, total_rays,
//          cam_pixel_offset(ncam), width(ncam), height(ncam), projection(ncam)]
// params: [znear, per camera: fovy, sensorsize(2), intrinsic(4)]   (7 floats/camera)
// out:    [dir.x,dir.y,dir.z, offset.x,offset.y,offset.z] per ray,
//         order [sample][camera][row][column], x fastest.
//
// Semantics are a direct f32 translation of the frozen upstream
// `mujoco_warp/_src/render_util.py::compute_ray` (71da24d) and of the
// `_build_rays` launch layout in `create_render_context`:
//   - perspective: intrinsic branch when sensorsize[1] != 0, else fovy branch;
//   - orthographic: direction (0,0,-1), offset from camera-space sensor rectangle;
//   - sub-pixel sample offsets: px = xid*spp + sample%spp, py = yid*spp + sample/spp,
//     image dimensions multiplied by spp (2x2 supersampling).
// Camera-space only: world transform (`cam_xmat @ dir`) is S7b, not this slice.

struct V3 {
    float x, y, z;
};

__device__ __forceinline__ V3 v3(float x, float y, float z) { V3 r; r.x = x; r.y = y; r.z = z; return r; }

// warp vec.h normalize(vec3) as contracted by the frozen reference compiler: the
// component-sum is emitted as fma(z, z, fma(y, y, x*x)); the explicit fmaf chain
// keeps that rounding independent of the local NVRTC contraction choices.
__device__ __forceinline__ V3 normalize3(V3 a) {
    float length = sqrtf(fmaf(a.z, a.z, fmaf(a.y, a.y, a.x * a.x)));
    if (length > 0.0f) {
        return v3(a.x / length, a.y / length, a.z / length);
    }
    return v3(0.0f, 0.0f, 0.0f);
}

// f32(pi / 180.0) exactly as Warp folds `wp.static(wp.pi / 180.0)`.
#define DEG_TO_RAD 0.017453292519943295f

// compute_ray(projection, fovy, sensorsize, intrinsic, img_w, img_h, px, py, znear).
__device__ __forceinline__ void compute_ray(int projection, float fovy, float sensor_size_x,
                                            float sensor_size_y, const float* intrinsic, int img_w,
                                            int img_h, int px, int py, float znear, V3* direction,
                                            V3* offset) {
    float inv_img_h = 1.0f / (float)img_h;
    if (projection == 1) {
        // ORTHOGRAPHIC: direction always forward; offset from the sensor rectangle.
        *direction = v3(0.0f, 0.0f, -1.0f);
        float aspect = (float)img_w * inv_img_h;
        float sensor_h = fovy;
        float sensor_w = sensor_h * aspect;
        float left = -0.5f * sensor_w;
        float top = 0.5f * sensor_h;
        float bottom = -top;
        float u = ((float)px + 0.5f) / (float)img_w;
        float v = ((float)py + 0.5f) * inv_img_h;
        float x = fmaf(sensor_w, u, left);
        float y = fmaf(bottom - top, v, top);
        *offset = v3(x, y, 0.0f);
        return;
    }

    // PERSPECTIVE.
    float aspect = (float)img_w * inv_img_h;
    float sensor_h = sensor_size_y;
    float left;
    float right;
    float top;
    float bottom;
    if (sensor_h != 0.0f) {
        float fx = intrinsic[0];
        float fy = intrinsic[1];
        float cx = intrinsic[2];
        float cy = intrinsic[3];
        float sensor_w = sensor_size_x;
        float target_aspect = aspect;
        float sensor_aspect = sensor_w / sensor_h;
        if (target_aspect > sensor_aspect) {
            sensor_h = sensor_w / target_aspect;
        } else if (target_aspect < sensor_aspect) {
            sensor_w = sensor_h * target_aspect;
        }
        float inv_fx_znear = znear / fx;
        float inv_fy_znear = znear / fy;
        float half_sensor_w = 0.5f * sensor_w;
        float half_sensor_h = 0.5f * sensor_h;
        left = -inv_fx_znear * (half_sensor_w - cx);
        right = inv_fx_znear * (half_sensor_w + cx);
        top = inv_fy_znear * (half_sensor_h - cy);
        bottom = -inv_fy_znear * (half_sensor_h + cy);
    } else {
        float fovy_rad = fovy * DEG_TO_RAD;
        float half_height = znear * tanf(0.5f * fovy_rad);
        float half_width = half_height * aspect;
        left = -half_width;
        right = half_width;
        top = half_height;
        bottom = -half_height;
    }
    float u = ((float)px + 0.5f) / (float)img_w;
    float v = ((float)py + 0.5f) * inv_img_h;
    float x = fmaf(right - left, u, left);
    float y = fmaf(bottom - top, v, top);
    *direction = normalize3(v3(x, y, -znear));
    *offset = v3(0.0f, 0.0f, 0.0f);
}

extern "C" __global__ void build_rays(const int* meta, const float* params, const float* state,
                                      float* out, unsigned d0, unsigned d1, unsigned d2,
                                      unsigned d3) {
    (void)state;
    (void)d0;
    (void)d1;
    (void)d2;
    (void)d3;

    const int ncam = meta[0];
    const int samples = meta[1];
    const int spp = meta[2];
    const int total = meta[3];
    const int* offsets = meta + 4;
    const int* widths = offsets + ncam;
    const int* heights = widths + ncam;
    const int* projections = heights + ncam;

    unsigned index = blockDim.x * blockIdx.x + threadIdx.x;
    if ((int)index >= samples * total) {
        return;
    }

    int sample = (int)index / total;
    int remainder = (int)index - sample * total;
    int camera = 0;
    while (camera + 1 < ncam && remainder >= offsets[camera] + widths[camera] * heights[camera]) {
        camera++;
    }
    int local = remainder - offsets[camera];
    int width = widths[camera];
    int height = heights[camera];
    int xid = local % width;
    int yid = local / width;

    int px = xid * spp + sample % spp;
    int py = yid * spp + sample / spp;
    int img_w = width * spp;
    int img_h = height * spp;

    float znear = params[0];
    const float* camera_params = params + 1 + 7 * camera;
    float fovy = camera_params[0];
    float sensor_size_x = camera_params[1];
    float sensor_size_y = camera_params[2];
    const float* intrinsic = camera_params + 3;

    V3 direction;
    V3 offset;
    compute_ray(projections[camera], fovy, sensor_size_x, sensor_size_y, intrinsic, img_w, img_h, px,
                py, znear, &direction, &offset);

    float* target = out + 6 * (long)index;
    target[0] = direction.x;
    target[1] = direction.y;
    target[2] = direction.z;
    target[3] = offset.x;
    target[4] = offset.y;
    target[5] = offset.z;
}
