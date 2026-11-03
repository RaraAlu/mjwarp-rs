// Offline authoring: compile scene-a with MuJoCo 3.12.0 (product version) and dump
// render-relevant model arrays to JSON.
//
// OFFLINE ONLY: not built or run by product targets/tests/gates. The product keeps
// MuJoCo 3.12; the reference pipeline uses 3.15. This tool proves that the committed
// 3.12 MJB loads with 3.12 and provides the 3.12 side of the field parity check.
//
// Usage:
//   compile_scene_312 <scene.xml> <scene-312.mjb> <model-312.json>
//
// Output JSON: {"schema":1,"tool":"compile_scene_312","mujoco_version":...,
//               "model_file":..., "loaded_mjb":true, "counts":{...},
//               "arrays":{name:{"kind":...,"count":N,"values":[...]},...}}
//
// Field list covers plan S0a render-relevant families: geom_*, cam_*, light_*,
// mat_*, tex_*, mesh_*, vis.headlight, vis.map.znear/zfar, stat.extent and the
// name tables (name_*adr + names bytes). Field names/kinds match model_parity.py.

#include <mujoco/mujoco.h>

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>

namespace {

FILE* g_out = nullptr;

void write_element_double(const double value) { std::fprintf(g_out, "%.17g", value); }
void write_element_float(const float value) { std::fprintf(g_out, "%.9g", static_cast<double>(value)); }
void write_element_int(const int value) { std::fprintf(g_out, "%d", value); }
void write_element_int64(const std::int64_t value) { std::fprintf(g_out, "%lld", static_cast<long long>(value)); }
void write_element_byte(const unsigned char value) { std::fprintf(g_out, "%u", static_cast<unsigned>(value)); }

void begin_array(const char* name, const char* kind, long long count) {
  if (count < 0) {
    std::fprintf(stderr, "negative count for %s\n", name);
    std::exit(3);
  }
  std::fprintf(g_out, "    \"%s\": {\"kind\": \"%s\", \"count\": %lld, \"values\": [", name, kind, count);
}

void end_array() { std::fprintf(g_out, "]},\n"); }

void dump_double_array(const char* name, const double* values, long long count) {
  begin_array(name, "float64", count);
  for (long long i = 0; i < count; ++i) {
    if (i) std::fputc(',', g_out);
    write_element_double(values[i]);
  }
  end_array();
}

void dump_float_array(const char* name, const float* values, long long count) {
  begin_array(name, "float32", count);
  for (long long i = 0; i < count; ++i) {
    if (i) std::fputc(',', g_out);
    write_element_float(values[i]);
  }
  end_array();
}

void dump_int_array(const char* name, const int* values, long long count) {
  begin_array(name, "int32", count);
  for (long long i = 0; i < count; ++i) {
    if (i) std::fputc(',', g_out);
    write_element_int(values[i]);
  }
  end_array();
}

void dump_int64_array(const char* name, const std::int64_t* values, long long count) {
  begin_array(name, "int64", count);
  for (long long i = 0; i < count; ++i) {
    if (i) std::fputc(',', g_out);
    write_element_int64(values[i]);
  }
  end_array();
}

void dump_byte_array(const char* name, const unsigned char* values, long long count) {
  begin_array(name, "byte", count);
  for (long long i = 0; i < count; ++i) {
    if (i) std::fputc(',', g_out);
    write_element_byte(values[i]);
  }
  end_array();
}

void dump_bool_array(const char* name, const bool* values, long long count) {
  begin_array(name, "byte", count);
  for (long long i = 0; i < count; ++i) {
    if (i) std::fputc(',', g_out);
    write_element_byte(values[i] ? 1 : 0);
  }
  end_array();
}

void write_json_string(FILE* out, const char* text) {
  std::fputc('"', out);
  for (const char* p = text; *p; ++p) {
    const unsigned char c = static_cast<unsigned char>(*p);
    if (c == '"' || c == '\\') {
      std::fputc('\\', out);
      std::fputc(c, out);
    } else if (c < 0x20) {
      std::fprintf(out, "\\u%04x", static_cast<unsigned>(c));
    } else {
      std::fputc(c, out);
    }
  }
  std::fputc('"', out);
}

void dump_counts(const mjModel* m) {
  std::fprintf(
      g_out,
      "  \"counts\": {\"nbody\": %lld, \"njnt\": %lld, \"ngeom\": %lld, \"nsite\": %lld, "
      "\"ncam\": %lld, \"nlight\": %lld, \"nmesh\": %lld, \"nmeshvert\": %lld, "
      "\"nmeshnormal\": %lld, \"nmeshtexcoord\": %lld, \"nmeshface\": %lld, "
      "\"nmeshgraph\": %lld, \"nmeshpoly\": %lld, \"nmeshpolyvert\": %lld, "
      "\"nmeshpolymap\": %lld, \"nmat\": %lld, \"ntex\": %lld, \"ntexdata\": %lld, "
      "\"nnames\": %lld, \"nbvh\": %lld, \"nbvhstatic\": %lld, \"nbvhdynamic\": %lld, "
      "\"nq\": %lld, \"nv\": %lld, \"nbuffer\": %lld, \"nkey\": %lld, "
      "\"nflex\": %lld, \"nflexvert\": %lld, \"nflexedge\": %lld, "
      "\"nflexelem\": %lld, \"nflexelemdata\": %lld, \"nflexshelldata\": %lld},\n",
      static_cast<long long>(m->nbody), static_cast<long long>(m->njnt),
      static_cast<long long>(m->ngeom), static_cast<long long>(m->nsite),
      static_cast<long long>(m->ncam), static_cast<long long>(m->nlight),
      static_cast<long long>(m->nmesh), static_cast<long long>(m->nmeshvert),
      static_cast<long long>(m->nmeshnormal), static_cast<long long>(m->nmeshtexcoord),
      static_cast<long long>(m->nmeshface), static_cast<long long>(m->nmeshgraph),
      static_cast<long long>(m->nmeshpoly), static_cast<long long>(m->nmeshpolyvert),
      static_cast<long long>(m->nmeshpolymap), static_cast<long long>(m->nmat),
      static_cast<long long>(m->ntex), static_cast<long long>(m->ntexdata),
      static_cast<long long>(m->nnames), static_cast<long long>(m->nbvh),
      static_cast<long long>(m->nbvhstatic), static_cast<long long>(m->nbvhdynamic),
      static_cast<long long>(m->nq), static_cast<long long>(m->nv),
      static_cast<long long>(m->nbuffer), static_cast<long long>(m->nkey),
      static_cast<long long>(m->nflex), static_cast<long long>(m->nflexvert),
      static_cast<long long>(m->nflexedge), static_cast<long long>(m->nflexelem),
      static_cast<long long>(m->nflexelemdata), static_cast<long long>(m->nflexshelldata));
}

void dump_arrays(const mjModel* m) {
  const long long ngeom = static_cast<long long>(m->ngeom);
  const long long ncam = static_cast<long long>(m->ncam);
  const long long nlight = static_cast<long long>(m->nlight);
  const long long nmesh = static_cast<long long>(m->nmesh);
  const long long nmat = static_cast<long long>(m->nmat);
  const long long ntex = static_cast<long long>(m->ntex);
  const long long nbody = static_cast<long long>(m->nbody);
  const long long njnt = static_cast<long long>(m->njnt);
  const long long nsite = static_cast<long long>(m->nsite);
  const long long nmeshvert = static_cast<long long>(m->nmeshvert);
  const long long nmeshnormal = static_cast<long long>(m->nmeshnormal);
  const long long nmeshtexcoord = static_cast<long long>(m->nmeshtexcoord);
  const long long nmeshface = static_cast<long long>(m->nmeshface);
  const long long nmeshgraph = static_cast<long long>(m->nmeshgraph);
  const long long nmeshpoly = static_cast<long long>(m->nmeshpoly);
  const long long nmeshpolyvert = static_cast<long long>(m->nmeshpolyvert);
  const long long nmeshpolymap = static_cast<long long>(m->nmeshpolymap);
  const long long ntexdata = static_cast<long long>(m->ntexdata);
  const long long nflex = static_cast<long long>(m->nflex);
  const long long nflexvert = static_cast<long long>(m->nflexvert);
  const long long nflexelemdata = static_cast<long long>(m->nflexelemdata);
  const long long nflexshelldata = static_cast<long long>(m->nflexshelldata);

  std::fprintf(g_out, "  \"arrays\": {\n");
  // geom_*
  dump_int_array("geom_type", m->geom_type, ngeom);
  dump_int_array("geom_contype", m->geom_contype, ngeom);
  dump_int_array("geom_conaffinity", m->geom_conaffinity, ngeom);
  dump_int_array("geom_condim", m->geom_condim, ngeom);
  dump_int_array("geom_bodyid", m->geom_bodyid, ngeom);
  dump_int_array("geom_dataid", m->geom_dataid, ngeom);
  dump_int_array("geom_matid", m->geom_matid, ngeom);
  dump_int_array("geom_group", m->geom_group, ngeom);
  dump_int_array("geom_priority", m->geom_priority, ngeom);
  dump_int_array("geom_plugin", m->geom_plugin, ngeom);
  dump_bool_array("geom_sameframe", reinterpret_cast<const bool*>(m->geom_sameframe), ngeom);
  dump_double_array("geom_solmix", m->geom_solmix, ngeom);
  dump_double_array("geom_solref", m->geom_solref, ngeom * mjNREF);
  dump_double_array("geom_solimp", m->geom_solimp, ngeom * mjNIMP);
  dump_double_array("geom_size", m->geom_size, ngeom * 3);
  dump_double_array("geom_aabb", m->geom_aabb, ngeom * 6);
  dump_double_array("geom_rbound", m->geom_rbound, ngeom);
  dump_double_array("geom_pos", m->geom_pos, ngeom * 3);
  dump_double_array("geom_quat", m->geom_quat, ngeom * 4);
  dump_double_array("geom_friction", m->geom_friction, ngeom * 3);
  dump_double_array("geom_margin", m->geom_margin, ngeom);
  dump_double_array("geom_gap", m->geom_gap, ngeom);
  dump_double_array("geom_surfacevel", m->geom_surfacevel, ngeom * 6);
  dump_double_array("geom_adhesion", m->geom_adhesion, ngeom);
  dump_double_array("geom_fluid", m->geom_fluid, ngeom * mjNFLUID);
  dump_float_array("geom_rgba", m->geom_rgba, ngeom * 4);
  // cam_*
  dump_int_array("cam_mode", m->cam_mode, ncam);
  dump_int_array("cam_bodyid", m->cam_bodyid, ncam);
  dump_int_array("cam_targetbodyid", m->cam_targetbodyid, ncam);
  dump_int_array("cam_projection", m->cam_projection, ncam);
  dump_int_array("cam_resolution", m->cam_resolution, ncam * 2);
  dump_int_array("cam_output", m->cam_output, ncam);
  dump_double_array("cam_pos", m->cam_pos, ncam * 3);
  dump_double_array("cam_quat", m->cam_quat, ncam * 4);
  dump_double_array("cam_poscom0", m->cam_poscom0, ncam * 3);
  dump_double_array("cam_pos0", m->cam_pos0, ncam * 3);
  dump_double_array("cam_mat0", m->cam_mat0, ncam * 9);
  dump_double_array("cam_fovy", m->cam_fovy, ncam);
  dump_double_array("cam_ipd", m->cam_ipd, ncam);
  dump_float_array("cam_sensorsize", m->cam_sensorsize, ncam * 2);
  dump_float_array("cam_intrinsic", m->cam_intrinsic, ncam * 4);
  // light_*
  dump_int_array("light_mode", m->light_mode, nlight);
  dump_int_array("light_bodyid", m->light_bodyid, nlight);
  dump_int_array("light_targetbodyid", m->light_targetbodyid, nlight);
  dump_int_array("light_type", m->light_type, nlight);
  dump_int_array("light_texid", m->light_texid, nlight);
  dump_bool_array("light_castshadow", reinterpret_cast<const bool*>(m->light_castshadow), nlight);
  dump_bool_array("light_active", reinterpret_cast<const bool*>(m->light_active), nlight);
  dump_float_array("light_bulbradius", m->light_bulbradius, nlight);
  dump_float_array("light_intensity", m->light_intensity, nlight);
  dump_float_array("light_range", m->light_range, nlight);
  dump_float_array("light_attenuation", m->light_attenuation, nlight * 3);
  dump_float_array("light_cutoff", m->light_cutoff, nlight);
  dump_float_array("light_softness", m->light_softness, nlight);
  dump_float_array("light_exponent", m->light_exponent, nlight);
  dump_float_array("light_ambient", m->light_ambient, nlight * 3);
  dump_float_array("light_diffuse", m->light_diffuse, nlight * 3);
  dump_float_array("light_specular", m->light_specular, nlight * 3);
  dump_double_array("light_pos", m->light_pos, nlight * 3);
  dump_double_array("light_dir", m->light_dir, nlight * 3);
  dump_double_array("light_poscom0", m->light_poscom0, nlight * 3);
  dump_double_array("light_pos0", m->light_pos0, nlight * 3);
  dump_double_array("light_dir0", m->light_dir0, nlight * 3);
  // mat_*
  dump_int_array("mat_texid", m->mat_texid, nmat * mjNTEXROLE);
  dump_bool_array("mat_texuniform", reinterpret_cast<const bool*>(m->mat_texuniform), nmat);
  dump_float_array("mat_texrepeat", m->mat_texrepeat, nmat * 2);
  dump_float_array("mat_emission", m->mat_emission, nmat);
  dump_float_array("mat_specular", m->mat_specular, nmat);
  dump_float_array("mat_shininess", m->mat_shininess, nmat);
  dump_float_array("mat_reflectance", m->mat_reflectance, nmat);
  dump_float_array("mat_metallic", m->mat_metallic, nmat);
  dump_float_array("mat_roughness", m->mat_roughness, nmat);
  dump_float_array("mat_rgba", m->mat_rgba, nmat * 4);
  // tex_*
  dump_int_array("tex_type", m->tex_type, ntex);
  dump_int_array("tex_colorspace", m->tex_colorspace, ntex);
  dump_int_array("tex_height", m->tex_height, ntex);
  dump_int_array("tex_width", m->tex_width, ntex);
  dump_int_array("tex_nchannel", m->tex_nchannel, ntex);
  dump_int64_array("tex_adr", m->tex_adr, ntex);
  dump_byte_array("tex_data", m->tex_data, ntexdata);
  dump_int_array("tex_pathadr", m->tex_pathadr, ntex);
  // mesh_*
  dump_int_array("mesh_vertadr", m->mesh_vertadr, nmesh);
  dump_int_array("mesh_vertnum", m->mesh_vertnum, nmesh);
  dump_int_array("mesh_faceadr", m->mesh_faceadr, nmesh);
  dump_int_array("mesh_facenum", m->mesh_facenum, nmesh);
  dump_int_array("mesh_bvhadr", m->mesh_bvhadr, nmesh);
  dump_int_array("mesh_bvhnum", m->mesh_bvhnum, nmesh);
  dump_int_array("mesh_octadr", m->mesh_octadr, nmesh);
  dump_int_array("mesh_octnum", m->mesh_octnum, nmesh);
  dump_int_array("mesh_normaladr", m->mesh_normaladr, nmesh);
  dump_int_array("mesh_normalnum", m->mesh_normalnum, nmesh);
  dump_int_array("mesh_texcoordadr", m->mesh_texcoordadr, nmesh);
  dump_int_array("mesh_texcoordnum", m->mesh_texcoordnum, nmesh);
  dump_int_array("mesh_graphadr", m->mesh_graphadr, nmesh);
  dump_int_array("mesh_extrema", m->mesh_extrema, nmesh * 27);
  dump_float_array("mesh_vert", m->mesh_vert, nmeshvert * 3);
  dump_float_array("mesh_normal", m->mesh_normal, nmeshnormal * 3);
  dump_float_array("mesh_texcoord", m->mesh_texcoord, nmeshtexcoord * 2);
  dump_int_array("mesh_face", m->mesh_face, nmeshface * 3);
  dump_int_array("mesh_facenormal", m->mesh_facenormal, nmeshface * 3);
  dump_int_array("mesh_facetexcoord", m->mesh_facetexcoord, nmeshface * 3);
  dump_int_array("mesh_graph", m->mesh_graph, nmeshgraph);
  dump_double_array("mesh_scale", m->mesh_scale, nmesh * 3);
  dump_double_array("mesh_pos", m->mesh_pos, nmesh * 3);
  dump_double_array("mesh_quat", m->mesh_quat, nmesh * 4);
  dump_int_array("mesh_pathadr", m->mesh_pathadr, nmesh);
  dump_int_array("mesh_polynum", m->mesh_polynum, nmesh);
  dump_int_array("mesh_polyadr", m->mesh_polyadr, nmesh);
  dump_double_array("mesh_polynormal", m->mesh_polynormal, nmeshpoly * 3);
  dump_int_array("mesh_polyvertadr", m->mesh_polyvertadr, nmeshpoly);
  dump_int_array("mesh_polyvertnum", m->mesh_polyvertnum, nmeshpoly);
  dump_int_array("mesh_polyvert", m->mesh_polyvert, nmeshpolyvert);
  dump_int_array("mesh_polymapadr", m->mesh_polymapadr, nmeshvert);
  dump_int_array("mesh_polymapnum", m->mesh_polymapnum, nmeshvert);
  dump_int_array("mesh_polymap", m->mesh_polymap, nmeshpolymap);
  // vis / stat
  dump_float_array("vis.headlight.ambient", m->vis.headlight.ambient, 3);
  dump_float_array("vis.headlight.diffuse", m->vis.headlight.diffuse, 3);
  dump_float_array("vis.headlight.specular", m->vis.headlight.specular, 3);
  dump_int_array("vis.headlight.active", &m->vis.headlight.active, 1);
  dump_float_array("vis.map.znear", &m->vis.map.znear, 1);
  dump_float_array("vis.map.zfar", &m->vis.map.zfar, 1);
  dump_double_array("stat.extent", &m->stat.extent, 1);
  dump_double_array("stat.center", m->stat.center, 3);
  // name tables
  dump_int_array("name_bodyadr", m->name_bodyadr, nbody);
  dump_int_array("name_jntadr", m->name_jntadr, njnt);
  dump_int_array("name_geomadr", m->name_geomadr, ngeom);
  dump_int_array("name_siteadr", m->name_siteadr, nsite);
  dump_int_array("name_camadr", m->name_camadr, ncam);
  dump_int_array("name_lightadr", m->name_lightadr, nlight);
  dump_int_array("name_meshadr", m->name_meshadr, nmesh);
  dump_int_array("name_texadr", m->name_texadr, ntex);
  dump_int_array("name_matadr", m->name_matadr, nmat);
  dump_byte_array("names", reinterpret_cast<const unsigned char*>(m->names), static_cast<long long>(m->nnames));
  // R06-S3b additive: render-relevant flex model arrays (scene-flex reference fixture).
  dump_int_array("flex_dim", m->flex_dim, nflex);
  dump_int_array("flex_vertadr", m->flex_vertadr, nflex);
  dump_int_array("flex_vertnum", m->flex_vertnum, nflex);
  dump_int_array("flex_edgeadr", m->flex_edgeadr, nflex);
  dump_int_array("flex_edgenum", m->flex_edgenum, nflex);
  dump_int_array("flex_elemadr", m->flex_elemadr, nflex);
  dump_int_array("flex_elemnum", m->flex_elemnum, nflex);
  dump_int_array("flex_elemdataadr", m->flex_elemdataadr, nflex);
  dump_int_array("flex_shellnum", m->flex_shellnum, nflex);
  dump_int_array("flex_shelldataadr", m->flex_shelldataadr, nflex);
  dump_int_array("flex_vertbodyid", m->flex_vertbodyid, nflexvert);
  dump_int_array("flex_elem", m->flex_elem, nflexelemdata);
  dump_int_array("flex_shell", m->flex_shell, nflexshelldata);
  dump_double_array("flex_radius", m->flex_radius, nflex);
  dump_double_array("flex_vert", m->flex_vert, nflexvert * 3);
  dump_double_array("flex_vert0", m->flex_vert0, nflexvert * 3);
  dump_float_array("flex_rgba", m->flex_rgba, nflex * 4);
  std::fprintf(g_out, "  \"_end\": 0}\n}");
}

}  // namespace

int main(int argc, char** argv) {
  if (argc != 4) {
    std::fprintf(stderr, "usage: %s <scene.xml> <scene-312.mjb> <model-312.json>\n", argv[0]);
    return 1;
  }
  char error[1000] = {0};
  mjModel* compiled = mj_loadXML(argv[1], nullptr, error, sizeof(error));
  if (!compiled) {
    std::fprintf(stderr, "mj_loadXML failed: %s\n", error);
    return 2;
  }
  mj_saveModel(compiled, argv[2], nullptr, 0);
  mj_deleteModel(compiled);
  mjModel* loaded = mj_loadModel(argv[2], nullptr);
  if (!loaded) {
    std::fprintf(stderr, "mj_loadModel failed: %s\n", argv[2]);
    return 3;
  }
  g_out = std::fopen(argv[3], "wb");
  if (!g_out) {
    std::fprintf(stderr, "cannot open output: %s\n", argv[3]);
    mj_deleteModel(loaded);
    return 4;
  }
  std::fprintf(
      g_out,
      "{\n  \"schema\": 1,\n  \"tool\": \"compile_scene_312\",\n"
      "  \"mujoco_version\": \"%s\",\n  \"mujoco_version_int\": %d,\n"
      "  \"source_xml\": ",
      mj_versionString(), mj_version());
  write_json_string(g_out, argv[1]);
  std::fprintf(g_out, ",\n  \"model_file\": ");
  write_json_string(g_out, argv[2]);
  std::fprintf(g_out, ",\n  \"loaded_mjb\": true,\n");
  dump_counts(loaded);
  dump_arrays(loaded);
  std::fclose(g_out);
  mj_deleteModel(loaded);
  std::printf("ok: %s -> %s, %s\n", argv[1], argv[2], argv[3]);
  return 0;
}
