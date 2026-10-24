"""S4 authoring support: frozen-upstream texture conversion and LINEAR sample capture.

OFFLINE ONLY: imported by `capture_s4_textures.py`, never by product builds/tests/gates.
Everything here calls the frozen upstream (MJWarp 3.15) code paths:
  - `render_util._convert_texture_data` for the u8 -> f32/255 RGBA conversion;
  - `render_util.create_warp_texture` + `wp.texture_sample` for the hardware LINEAR
    sampling reference (the same path the render kernel uses);
  - the same conversion/output on the Warp `cpu` device purely as a documented
    hardware-vs-software deviation cross-check (not the product acceptance reference).
"""

from __future__ import annotations

import numpy as np
import warp as wp

from mujoco_warp._src import render_util

# Fixed uv grid: a 32x32 lattice over [-0.125, 1.125] (out-of-range -> CLAMP) plus a
# fixed boundary/edge/texel-center list. All values are exactly representable f32.
GRID = 32
SPAN = 1.25
ORIGIN = -0.125
SPECIAL_UVS = np.asarray(
    [
        [0.0, 0.0],
        [1.0, 1.0],
        [0.5, 0.5],
        [0.0, 1.0],
        [1.0, 0.0],
        [-0.25, 1.25],
        [1.25, -0.25],
        [1.0 / 64.0, 1.0 / 64.0],
        [63.5 / 64.0, 63.5 / 64.0],
        [0.5 / 64.0, 0.5 / 64.0],
        [0.0078125, 0.9921875],
        [0.999, 0.001],
        [0.25, 0.75],
        [0.75, 0.25],
        [0.5, 0.0],
        [0.0, 0.5],
    ],
    dtype=np.float32,
)


def uv_grid() -> np.ndarray:
    axes = np.asarray(
        [ORIGIN + SPAN * (index + 0.5) / GRID for index in range(GRID)], dtype=np.float32
    )
    grid = np.stack(np.meshgrid(axes, axes, indexing="xy"), axis=-1).reshape(-1, 2)
    return np.ascontiguousarray(np.concatenate([grid, SPECIAL_UVS], axis=0), dtype=np.float32)


def bits(values) -> list[int]:
    return np.ascontiguousarray(values, dtype=np.float32).view(np.uint32).reshape(-1).tolist()


def u8s(values) -> list[int]:
    return np.asarray(values, dtype=np.uint8).reshape(-1).tolist()


def ints(values) -> list[int]:
    return np.asarray(values, dtype=np.int64).reshape(-1).tolist()


def texture_metadata(mjm) -> list[dict]:
    return [
        {
            "tex_id": tex_id,
            "width": int(mjm.tex_width[tex_id]),
            "height": int(mjm.tex_height[tex_id]),
            "nchannel": int(mjm.tex_nchannel[tex_id]),
            "type": int(mjm.tex_type[tex_id]),
            "adr": int(mjm.tex_adr[tex_id]),
        }
        for tex_id in range(int(mjm.ntex))
    ]


def texture_bytes(mjm, meta: dict) -> np.ndarray:
    width = meta["width"]
    height = meta["height"]
    channels = meta["nchannel"]
    adr = meta["adr"]
    return np.asarray(mjm.tex_data[adr : adr + width * height * channels], dtype=np.uint8)


def convert_reference(mjm, tex_id: int, device: str) -> tuple[list[int], list[int]]:
    """Run the frozen upstream `_convert_texture_data` for one MjModel texture."""
    meta = texture_metadata(mjm)[tex_id]
    data = texture_bytes(mjm, meta)
    converted = wp.zeros((meta["height"], meta["width"], 4), dtype=float, device=device)
    wp.launch(
        render_util._convert_texture_data,
        dim=(meta["width"], meta["height"]),
        inputs=[meta["width"], 0, meta["nchannel"], wp.array(data, dtype=wp.uint8, device=device)],
        outputs=[converted],
        device=device,
    )
    wp.synchronize()
    return u8s(data), bits(np.asarray(converted.numpy(), dtype=np.float32))


def convert_synthetic(width: int, height: int, channels: int, device: str) -> dict:
    """Frozen upstream conversion for a synthetic u8 texture (nullable channel padding)."""
    count = width * height * channels
    data = np.asarray([(index * 37 + 11) % 256 for index in range(count)], dtype=np.uint8)
    converted = wp.zeros((height, width, 4), dtype=float, device=device)
    wp.launch(
        render_util._convert_texture_data,
        dim=(width, height),
        inputs=[width, 0, channels, wp.array(data, dtype=wp.uint8, device=device)],
        outputs=[converted],
        device=device,
    )
    wp.synchronize()
    return {
        "name": f"nc{channels}",
        "width": width,
        "height": height,
        "nchannel": channels,
        "data_bits": u8s(data),
        "converted_bits": bits(np.asarray(converted.numpy(), dtype=np.float32)),
    }


@wp.kernel
def _sample_kernel(
    tex: wp.Texture2D,
    uv: wp.array[wp.vec2],
    out: wp.array[wp.vec4],
):
    tid = wp.tid()
    out[tid] = wp.texture_sample(tex, uv[tid], dtype=wp.vec4)


def sample_reference(converted_bits: list[int], width: int, height: int, device: str):
    """Sample one converted texture at the fixed uv grid with Warp's LINEAR path."""
    values = (
        np.asarray(converted_bits, dtype=np.uint32)
        .reshape(height, width, 4)
        .view(np.float32)
        .copy()
    )
    texture = wp.Texture2D(
        np.ascontiguousarray(values, dtype=np.float32),
        filter_mode=wp.TextureFilterMode.LINEAR,
        device=device,
    )
    uvs = uv_grid()
    uv_array = wp.array(uvs, dtype=wp.vec2, device=device)
    sampled = wp.zeros(len(uvs), dtype=wp.vec4, device=device)
    wp.launch(_sample_kernel, dim=len(uvs), inputs=[texture, uv_array, sampled], device=device)
    wp.synchronize()
    return bits(np.asarray(sampled.numpy(), dtype=np.float32))


def capture_registry_semantics(mjm) -> dict:
    """Upstream `use_textures` toggle: registry length 0 (off) vs ntex (on)."""
    off = render_util.create_render_context(mjm, nworld=1, cam_res=(8, 8), use_textures=False)
    on = render_util.create_render_context(mjm, nworld=1, cam_res=(8, 8), use_textures=True)
    result = {
        "ntex": int(mjm.ntex),
        "registry_off": len(off.textures_registry),
        "registry_on": len(on.textures_registry),
        "use_textures_off": bool(off.use_textures),
        "use_textures_on": bool(on.use_textures),
    }
    del off, on
    return result


def capture_zero_size_rejection() -> dict:
    """Upstream rejects a zero-sized texture at `wp.Texture2D` construction time."""
    try:
        wp.Texture2D(np.zeros((0, 8, 4), dtype=np.float32), device="cuda:0")
    except Exception as error:  # noqa: BLE001 - capture the exact upstream exception
        return {
            "constructible": False,
            "exception": type(error).__name__,
            "message": str(error).splitlines()[0][:200],
        }
    return {"constructible": True}
