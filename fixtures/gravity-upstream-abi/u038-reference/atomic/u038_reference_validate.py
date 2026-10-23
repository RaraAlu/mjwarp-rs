"""Offline authoring: U038 reference independent f64 recompute validators (OFFLINE ONLY)."""

from __future__ import annotations

import numpy as np

import u038_reference_common as common
from u038_reference_scenarios import geom_semiaxes


def fluid_force_f64(config: dict, intermediates: dict) -> np.ndarray:
    worlds, nbody, ngeom = config["worlds"], config["nbody"], config["ngeom"]
    wind = np.asarray(config["wind"], dtype=np.float64)
    density = np.asarray(config["density"], dtype=np.float64)
    viscosity = np.asarray(config["viscosity"], dtype=np.float64)
    mass = np.asarray(config["body_mass"][0], dtype=np.float64)
    geom_size = np.asarray(config["geom_size_periods"], dtype=np.float64).reshape(-1, ngeom, 3)
    geom_fluid = np.asarray(config["geom_fluid"], dtype=np.float64).reshape(ngeom, 12)
    xipos = np.asarray(intermediates["xipos"], dtype=np.float64).reshape(worlds, nbody, 3)
    ximat = np.asarray(intermediates["ximat"], dtype=np.float64).reshape(worlds, nbody, 3, 3)
    geom_xpos = np.asarray(intermediates["geom_xpos"], dtype=np.float64).reshape(worlds, ngeom, 3)
    geom_xmat = np.asarray(intermediates["geom_xmat"], dtype=np.float64).reshape(worlds, ngeom, 3, 3)
    subtree = np.asarray(intermediates["subtree_com"], dtype=np.float64).reshape(worlds, nbody, 3)
    cvel = np.asarray(intermediates["cvel"], dtype=np.float64).reshape(worlds, nbody, 6)
    out = np.zeros((worlds, nbody, 6), dtype=np.float64)
    for world in range(worlds):
        w = wind[world % wind.shape[0]]
        size_row = geom_size[world % geom_size.shape[0]]
        rho = density[world % density.shape[0]]
        visc = viscosity[world % viscosity.shape[0]]
        for body in range(1, nbody):
            if mass[body] < common.MJ_MINVAL or not config["body_fluid_ellipsoid"][body]:
                continue
            rot = np.array(ximat[world, body], dtype=np.float64)
            rot_t = rot.T
            ang = np.array(cvel[world, body, :3], dtype=np.float64)
            lin = np.array(cvel[world, body, 3:], dtype=np.float64)
            root = config["body_rootid"][body]
            lin_com = lin - np.cross(xipos[world, body] - subtree[world, root], ang)
            force = np.zeros(3, dtype=np.float64)
            torque = np.zeros(3, dtype=np.float64)
            for g in range(config["body_geomnum"][body]):
                geomid = config["body_geomadr"][body] + g
                coef = geom_fluid[geomid, 0]
                if coef <= 0.0:
                    continue
                sa = geom_semiaxes(size_row[geomid], config["geom_type"][geomid])
                geom_rot = np.array(geom_xmat[world, geomid], dtype=np.float64)
                geom_rot_t = geom_rot.T
                geom_pos = np.array(geom_xpos[world, geomid], dtype=np.float64)
                lin_point = lin_com + np.cross(ang, geom_pos - xipos[world, body])
                l_ang = geom_rot_t @ ang
                l_lin = geom_rot_t @ lin_point
                if np.any(w):
                    l_lin = l_lin - geom_rot_t @ w
                lfrc_torque = np.zeros(3, dtype=np.float64)
                lfrc_force = np.zeros(3, dtype=np.float64)
                if rho > 0.0:
                    virtual_mass = geom_fluid[geomid, 6:9]
                    virtual_inertia = geom_fluid[geomid, 9:12]
                    virtual_lin_mom = rho * virtual_mass * l_lin
                    virtual_ang_mom = rho * virtual_inertia * l_ang
                    lfrc_force += np.cross(virtual_lin_mom, l_ang)
                    lfrc_torque += np.cross(virtual_lin_mom, l_lin) + np.cross(virtual_ang_mom, l_ang)
                magnus_coef = geom_fluid[geomid, 5]
                kutta_coef = geom_fluid[geomid, 4]
                blunt_drag_coef = geom_fluid[geomid, 1]
                slender_drag_coef = geom_fluid[geomid, 2]
                ang_drag_coef = geom_fluid[geomid, 3]
                volume = 4.0 / 3.0 * np.pi * sa[0] * sa[1] * sa[2]
                d_max = float(np.max(sa))
                d_min = float(np.min(sa))
                d_mid = float(sa[0] + sa[1] + sa[2] - d_max - d_min)
                a_max = np.pi * d_max * d_mid
                lin_speed = float(np.linalg.norm(l_lin))
                inv_speed = 1.0 / lin_speed if lin_speed > common.MJ_MINVAL else 0.0
                v = l_lin * inv_speed
                inv_dmax = 1.0 / d_max if d_max > common.MJ_MINVAL else 0.0
                s = sa * inv_dmax
                magnus_force = np.cross(l_ang, l_lin) * (magnus_coef * rho * volume)
                s12_sq = (s[1] * s[2]) ** 2
                s20_sq = (s[2] * s[0]) ** 2
                s01_sq = (s[0] * s[1]) ** 2
                proj_num = s12_sq * v[0] ** 2 + s20_sq * v[1] ** 2 + s01_sq * v[2] ** 2
                inv_proj_num = 1.0 / proj_num if proj_num > common.MJ_MINVAL else 0.0
                aa = s12_sq * inv_proj_num
                bb = s20_sq * inv_proj_num
                cc = s01_sq * inv_proj_num
                proj_denom = aa * aa * v[0] ** 2 + bb * bb * v[1] ** 2 + cc * cc * v[2] ** 2
                a_proj = np.pi * d_max * d_max * np.sqrt(proj_num * proj_denom)
                cos_alpha = 1.0 / proj_denom if proj_denom > common.MJ_MINVAL else 0.0
                norm = np.array([aa * v[0], bb * v[1], cc * v[2]], dtype=np.float64)
                kutta_force = np.zeros(3, dtype=np.float64)
                if rho > 0.0 and kutta_coef != 0.0 and lin_speed > common.MJ_MINVAL:
                    circ = np.cross(norm, l_lin) * (kutta_coef * rho * cos_alpha * a_proj)
                    kutta_force = np.cross(circ, l_lin)
                eq_sphere_d = 2.0 / 3.0 * (sa[0] + sa[1] + sa[2])
                lin_visc_force_coef = 3.0 * np.pi * eq_sphere_d
                lin_visc_torq_coef = np.pi * eq_sphere_d ** 3
                i_max = 8.0 / 15.0 * np.pi * d_mid * d_max ** 4
                moments = []
                for direction in range(3):
                    d0 = sa[direction]
                    d1, d2 = sa[(direction + 1) % 3], sa[(direction + 2) % 3]
                    moments.append(8.0 / 15.0 * np.pi * d0 * max(d1, d2) ** 4)
                mom_visc = np.array(
                    [
                        l_ang[i] * (ang_drag_coef * moments[i] + slender_drag_coef * (i_max - moments[i]))
                        for i in range(3)
                    ],
                    dtype=np.float64,
                )
                drag_lin_coef = visc * lin_visc_force_coef + rho * lin_speed * (
                    a_proj * blunt_drag_coef + slender_drag_coef * (a_max - a_proj)
                )
                drag_ang_coef = visc * lin_visc_torq_coef + rho * float(np.linalg.norm(mom_visc))
                lfrc_torque = lfrc_torque - drag_ang_coef * l_ang
                lfrc_force = lfrc_force + magnus_force + kutta_force - drag_lin_coef * l_lin
                lfrc_torque = lfrc_torque * coef
                lfrc_force = lfrc_force * coef
                torque += geom_rot @ lfrc_torque
                force += geom_rot @ lfrc_force
            out[world, body, :3] = force
            out[world, body, 3:] = torque
    return out


def gravity_force_f64(config: dict, intermediates: dict) -> np.ndarray:
    worlds, nbody, nv = config["worlds"], config["nbody"], config["nv"]
    gravity = np.asarray(config["gravity"], dtype=np.float64)
    mass = np.asarray(config["body_mass"][0], dtype=np.float64)
    gravcomp = np.asarray(config["body_gravcomp"], dtype=np.float64)
    ancestor = np.asarray(config["body_isdofancestor"], dtype=np.int64)
    xipos = np.asarray(intermediates["xipos"], dtype=np.float64).reshape(worlds, nbody, 3)
    subtree = np.asarray(intermediates["subtree_com"], dtype=np.float64).reshape(worlds, nbody, 3)
    cdof = np.asarray(intermediates["cdof"], dtype=np.float64).reshape(worlds, nv, 6)
    out = np.zeros((worlds, nv), dtype=np.float64)
    for world in range(worlds):
        g = gravity[world % gravity.shape[0]]
        for body in range(1, nbody):
            gc = gravcomp[world % gravcomp.shape[0], body]
            if gc == 0.0:
                continue
            force = -g * mass[body] * gc
            root = config["body_rootid"][body]
            offset = xipos[world, body] - subtree[world, root]
            for dof in range(nv):
                if not ancestor[body, dof]:
                    continue
                cd = cdof[world, dof]
                cdof_ang, cdof_lin = cd[:3], cd[3:]
                jacp = cdof_lin + np.cross(cdof_ang, offset)
                out[world, dof] += float(np.dot(jacp, force))
    return out


def report_variant_runs(calls):
    variants = []
    seen = []
    for run, values in enumerate(calls):
        for index, prior in enumerate(seen):
            if np.array_equal(values, prior):
                variants[index]["runs"].append(run)
                break
        else:
            seen.append(values)
            variants.append({"runs": [run], "bits": common.bits_of(values)})
    return variants
