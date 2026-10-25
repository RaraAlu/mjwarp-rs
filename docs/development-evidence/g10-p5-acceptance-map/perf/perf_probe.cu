// G6-2 性能探针主入口 · 冻结 G5 探针源第 427-743 行（main）+ 插入的 `--perf` 计时分支。
// 来源：docs/development-evidence/g10-p5-gpu-kernels/gpu-window/probe.cu（sha256=4ad47bce54fa8f3c257f851fae12c8950d3d9d35d4d5ad3c71ef31daf784d6e2）。
// 正常模式（`perf_probe <case_dir> [repeats]`）与冻结探针完全一致；`--perf` 只做计量。

#include "perf_probe_common.cu"
#include "perf_probe_timing.cu"

int main(int argc, char** argv) {
  if (argc < 2) {
    std::fprintf(stderr, "usage: probe <case_dir> [repeats]\n");
    return 2;
  }
  const std::string directory = argv[1];
  const int repeats = argc > 2 ? std::atoi(argv[2]) : 2;
  const CaseMeta meta = read_meta(directory);

  DeviceArrays host;
  host.geom_xpos = load_bin<float>(directory, "geom_xpos");
  host.geom_xmat = load_bin<float>(directory, "geom_xmat");
  host.geom_size = load_bin<float>(directory, "geom_size");
  host.geom_margin = load_bin<float>(directory, "geom_margin");
  host.geom_gap = load_bin<float>(directory, "geom_gap");
  host.geom_rbound = load_bin<float>(directory, "geom_rbound");
  host.geom_aabb = load_bin<float>(directory, "geom_aabb");
  host.geom_type = load_bin<int>(directory, "geom_type");
  host.geom_dataid = load_bin<int>(directory, "geom_dataid");
  host.mesh_vertadr = load_bin<int>(directory, "mesh_vertadr");
  host.mesh_vertnum = load_bin<int>(directory, "mesh_vertnum");
  host.mesh_graphadr = load_bin<int>(directory, "mesh_graphadr");
  host.mesh_octadr = load_bin<int>(directory, "mesh_octadr");
  host.mesh_vert = load_bin<float>(directory, "mesh_vert");
  host.mesh_graph = load_bin<int>(directory, "mesh_graph");
  host.hfield_size = load_bin<float>(directory, "hfield_size");
  host.hfield_nrow = load_bin<int>(directory, "hfield_nrow");
  host.hfield_ncol = load_bin<int>(directory, "hfield_ncol");
  host.hfield_adr = load_bin<int>(directory, "hfield_adr");
  host.hfield_data = load_bin<float>(directory, "hfield_data");
  host.oct_aabb = load_bin<float>(directory, "oct_aabb");
  host.oct_child = load_bin<int>(directory, "oct_child");
  host.oct_coeff = load_bin<float>(directory, "oct_coeff");

  ProbeArgs args{};
  args.geom_xpos = to_device(host.geom_xpos);
  args.geom_xmat = to_device(host.geom_xmat);
  args.geom_size = to_device(host.geom_size);
  args.geom_margin = to_device(host.geom_margin);
  args.geom_gap = to_device(host.geom_gap);
  args.geom_rbound = to_device(host.geom_rbound);
  args.geom_aabb = to_device(host.geom_aabb);
  args.geom_type = to_device(host.geom_type);
  args.geom_dataid = to_device(host.geom_dataid);
  args.mesh_vertadr = to_device(host.mesh_vertadr);
  args.mesh_vertnum = to_device(host.mesh_vertnum);
  args.mesh_graphadr = to_device(host.mesh_graphadr);
  args.mesh_octadr = to_device(host.mesh_octadr);
  args.mesh_vert = to_device(host.mesh_vert);
  args.mesh_graph = to_device(host.mesh_graph);
  args.hfield_size = to_device(host.hfield_size);
  args.hfield_nrow = to_device(host.hfield_nrow);
  args.hfield_ncol = to_device(host.hfield_ncol);
  args.hfield_adr = to_device(host.hfield_adr);
  args.hfield_data = to_device(host.hfield_data);
  args.oct_aabb = to_device(host.oct_aabb);
  args.oct_child = to_device(host.oct_child);
  args.oct_coeff = to_device(host.oct_coeff);

  args.nworld = meta.i32("nworld");
  args.ngeom = meta.i32("ngeom");
  args.g1 = meta.i32("g1");
  args.g2 = meta.i32("g2");
  args.type1 = meta.i32("type1");
  args.type2 = meta.i32("type2");
  args.geomgeomid = 0;
  args.naccdmax = meta.i32("naccdmax");
  args.naconmax = meta.i32("naconmax");
  args.efc_width = meta.i32("efc_width");
  args.hfield_id = meta.i32("hfield_id");
  args.oct_root = meta.i32("oct_root");
  args.noct = meta.i32("noct");
  args.tolerance = meta.f32("tolerance");
  args.margin = meta.f32("margin");
  args.gap = meta.f32("gap");
  args.gjk_iterations = meta.i32("gjk_iterations");
  args.epa_iterations = meta.i32("epa_iterations");
  args.condim = meta.i32("condim");
  args.pairid0 = meta.i32("pairid0");
  args.pairid1 = meta.i32("pairid1");
  const auto friction = meta.f32s("friction");
  const auto solref = meta.f32s("solref");
  const auto solreffriction = meta.f32s("solreffriction");
  const auto solimp = meta.f32s("solimp");
  for (int index = 0; index < 5; ++index) {
    args.friction[index] = friction[index];
    args.solimp[index] = solimp[index];
  }
  for (int index = 0; index < 2; ++index) {
    args.solref[index] = solref[index];
    args.solreffriction[index] = solreffriction[index];
  }
  args.adhesion = meta.f32("adhesion");
  args.sdf_initpoints = meta.i32("sdf_initpoints");
  args.sdf_iterations = meta.i32("sdf_iterations");

  const int naconmax = args.naconmax;
  const int efc_width = args.efc_width;
  const int nworld = args.nworld;

  // 输出缓冲（设备端）。
  float* d_dist = nullptr;
  v3* d_pos = nullptr;
  float* d_frame = nullptr;
  float* d_includemargin = nullptr;
  float* d_friction = nullptr;
  float* d_solref = nullptr;
  float* d_solreffriction = nullptr;
  float* d_solimp = nullptr;
  float* d_adhesion = nullptr;
  int* d_dim = nullptr;
  int* d_geom = nullptr;
  int* d_efc = nullptr;
  int* d_worldid = nullptr;
  int* d_type = nullptr;
  int* d_geomcollisionid = nullptr;
  int* d_nacon = nullptr;
  int* d_nccd = nullptr;
  unsigned* d_overflow = nullptr;
  PROBE_CHECK(cudaMalloc(&d_dist, naconmax * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_pos, naconmax * sizeof(v3)));
  PROBE_CHECK(cudaMalloc(&d_frame, naconmax * 9 * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_includemargin, naconmax * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_friction, naconmax * 5 * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_solref, naconmax * 2 * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_solreffriction, naconmax * 2 * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_solimp, naconmax * 5 * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_adhesion, naconmax * sizeof(float)));
  PROBE_CHECK(cudaMalloc(&d_dim, naconmax * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_geom, naconmax * 2 * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_efc, naconmax * efc_width * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_worldid, naconmax * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_type, naconmax * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_geomcollisionid, naconmax * sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_nacon, sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_nccd, sizeof(int)));
  PROBE_CHECK(cudaMalloc(&d_overflow, nworld * sizeof(unsigned)));

  d_contact_out out{};
  out.dist = d_dist;
  out.pos = d_pos;
  out.frame = d_frame;
  out.includemargin = d_includemargin;
  out.friction = d_friction;
  out.solref = d_solref;
  out.solreffriction = d_solreffriction;
  out.solimp = d_solimp;
  out.dim = d_dim;
  out.geom = d_geom;
  out.efc_address = d_efc;
  out.worldid = d_worldid;
  out.type = d_type;
  out.geomcollisionid = d_geomcollisionid;
  out.adhesion = d_adhesion;
  out.nacon = d_nacon;
  out.efc_address_width = efc_width;
  args.out = out;
  args.nacon = d_nacon;
  args.nccd = d_nccd;
  args.overflow = d_overflow;

  cudaDeviceProp properties{};
  PROBE_CHECK(cudaGetDeviceProperties(&properties, 0));
  size_t free_bytes = 0;
  size_t total_bytes = 0;
  PROBE_CHECK(cudaMemGetInfo(&free_bytes, &total_bytes));
  std::printf("device=%s sm=%d.%d cudart=%d free=%zuMiB total=%zuMiB\n", properties.name,
              properties.major, properties.minor, CUDART_VERSION, free_bytes >> 20,
              total_bytes >> 20);

  const std::string kind = meta.text("kind");
  void (*kernel)(ProbeArgs) = kernel_contact;
  if (kind == "hfield") kernel = kernel_hfield;
  if (kind == "sdf") kernel = kernel_sdf;

  // G6-2 perf 模式：`perf_probe <case_dir> --perf [warmup] [reps]`，只计量、不写 out1/out2。
  if (argc > 2 && std::string(argv[2]) == "--perf") {
    const int warmup = argc > 3 ? std::atoi(argv[3]) : 5;
    const int reps = argc > 4 ? std::atoi(argv[4]) : 50;
    return run_perf_mode(directory, host, args, kernel, meta, properties, warmup, reps);
  }

  std::vector<double> times;
  std::vector<Outputs> runs;
  for (int repeat = 0; repeat < repeats; ++repeat) {
    PROBE_CHECK(cudaMemset(d_nacon, 0, sizeof(int)));
    PROBE_CHECK(cudaMemset(d_nccd, 0, sizeof(int)));
    PROBE_CHECK(cudaMemset(d_overflow, 0, nworld * sizeof(unsigned)));
    PROBE_CHECK(cudaMemset(d_efc, 0xff, naconmax * efc_width * sizeof(int)));
    const auto started = std::chrono::steady_clock::now();
    kernel<<<nworld, 1>>>(args);
    PROBE_CHECK(cudaGetLastError());
    PROBE_CHECK(cudaDeviceSynchronize());
    const auto elapsed =
        std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - started)
            .count();
    times.push_back(elapsed);

    Outputs run;
    std::vector<int> nacon_counter(1, 0);
    from_device(d_nacon, nacon_counter);
    run.nacon_counter = nacon_counter;
    run.nacon = nacon_counter[0];
    const int rows = naconmax;
    run.dist.resize(rows);
    run.pos.resize(rows * 3);
    run.frame.resize(rows * 9);
    run.includemargin.resize(rows);
    run.friction.resize(rows * 5);
    run.solref.resize(rows * 2);
    run.solreffriction.resize(rows * 2);
    run.solimp.resize(rows * 5);
    run.adhesion.resize(rows);
    run.dim.resize(rows);
    run.geom.resize(rows * 2);
    run.efc_address.resize(rows * efc_width);
    run.worldid.resize(rows);
    run.type.resize(rows);
    run.geomcollisionid.resize(rows);
    run.overflow.resize(nworld);
    run.nccd.resize(1);
    from_device(d_dist, run.dist);
    from_device(reinterpret_cast<const float*>(d_pos), run.pos);
    from_device(d_frame, run.frame);
    from_device(d_includemargin, run.includemargin);
    from_device(d_friction, run.friction);
    from_device(d_solref, run.solref);
    from_device(d_solreffriction, run.solreffriction);
    from_device(d_solimp, run.solimp);
    from_device(d_adhesion, run.adhesion);
    from_device(d_dim, run.dim);
    from_device(d_geom, run.geom);
    from_device(d_efc, run.efc_address);
    from_device(d_worldid, run.worldid);
    from_device(d_type, run.type);
    from_device(d_geomcollisionid, run.geomcollisionid);
    from_device(d_overflow, run.overflow);
    from_device(d_nccd, run.nccd);
    runs.push_back(run);
  }

  // 只落有意义的行（nacon 行）；同时落 nacon/overflow/nccd 汇总。
  const int nacon_rows = runs.front().nacon;
  for (size_t repeat = 0; repeat < runs.size(); ++repeat) {
    const std::string out_dir = directory + "/out" + std::to_string(repeat + 1);
    std::system(("mkdir \"" + out_dir + "\" 2>nul").c_str());
    const Outputs& run = runs[repeat];
    std::vector<float> dist(run.dist.begin(), run.dist.begin() + nacon_rows);
    std::vector<float> pos(run.pos.begin(), run.pos.begin() + nacon_rows * 3);
    std::vector<float> frame(run.frame.begin(), run.frame.begin() + nacon_rows * 9);
    std::vector<float> includemargin(run.includemargin.begin(),
                                    run.includemargin.begin() + nacon_rows);
    std::vector<float> friction(run.friction.begin(), run.friction.begin() + nacon_rows * 5);
    std::vector<float> solref(run.solref.begin(), run.solref.begin() + nacon_rows * 2);
    std::vector<float> solreffriction(run.solreffriction.begin(),
                                      run.solreffriction.begin() + nacon_rows * 2);
    std::vector<float> solimp(run.solimp.begin(), run.solimp.begin() + nacon_rows * 5);
    std::vector<float> adhesion(run.adhesion.begin(), run.adhesion.begin() + nacon_rows);
    std::vector<int> dim(run.dim.begin(), run.dim.begin() + nacon_rows);
    std::vector<int> geom(run.geom.begin(), run.geom.begin() + nacon_rows * 2);
    std::vector<int> efc(run.efc_address.begin(), run.efc_address.begin() + nacon_rows * efc_width);
    std::vector<int> worldid(run.worldid.begin(), run.worldid.begin() + nacon_rows);
    std::vector<int> type(run.type.begin(), run.type.begin() + nacon_rows);
    std::vector<int> geomcollisionid(run.geomcollisionid.begin(),
                                     run.geomcollisionid.begin() + nacon_rows);
    write_bin(out_dir, "dist", dist);
    write_bin(out_dir, "pos", pos);
    write_bin(out_dir, "frame", frame);
    write_bin(out_dir, "includemargin", includemargin);
    write_bin(out_dir, "friction", friction);
    write_bin(out_dir, "solref", solref);
    write_bin(out_dir, "solreffriction", solreffriction);
    write_bin(out_dir, "solimp", solimp);
    write_bin(out_dir, "adhesion", adhesion);
    write_bin(out_dir, "dim", dim);
    write_bin(out_dir, "geom", geom);
    write_bin(out_dir, "efc_address", efc);
    write_bin(out_dir, "worldid", worldid);
    write_bin(out_dir, "type", type);
    write_bin(out_dir, "geomcollisionid", geomcollisionid);
    write_bin(out_dir, "overflow", run.overflow);
    write_bin(out_dir, "nccd", run.nccd);
    write_bin(out_dir, "nacon", run.nacon_counter);
    // efc 宽度记录，Rust 侧据此解析 efc_address。
    std::ofstream efc_meta(out_dir + "/efc_width.txt");
    efc_meta << efc_width << "\n";
  }

  const bool raw_equal = runs.size() < 2 || [&]() {
    const Outputs& a = runs[0];
    const Outputs& b = runs[1];
    return a.nacon == b.nacon && a.dist == b.dist && a.pos == b.pos && a.frame == b.frame &&
           a.dim == b.dim && a.geom == b.geom && a.worldid == b.worldid && a.type == b.type &&
           a.geomcollisionid == b.geomcollisionid && a.overflow == b.overflow && a.nccd == b.nccd;
  }();
  std::printf("case=%s kind=%s nacon=%d overflow=", meta.text("case").c_str(), kind.c_str(),
              runs.front().nacon);
  for (unsigned value : runs.front().overflow) std::printf("0x%x,", value);
  std::printf(" nccd=%d raw_equal=%s times=", runs.front().nccd[0], raw_equal ? "true" : "false");
  for (double time : times) std::printf("%.3fms,", time);
  std::printf("\n");

  std::ofstream report(directory + "/probe-report.txt");
  report << "case " << meta.text("case") << "\n";
  report << "kind " << kind << "\n";
  report << "device " << properties.name << "\n";
  report << "compute_capability " << properties.major << "." << properties.minor << "\n";
  report << "cudart " << CUDART_VERSION << "\n";
  report << "nacon " << runs.front().nacon << "\n";
  report << "nccd " << runs.front().nccd[0] << "\n";
  for (size_t index = 0; index < runs.front().overflow.size(); ++index) {
    report << "overflow_world" << index << " " << runs.front().overflow[index] << "\n";
  }
  report << "two_runs_raw_bitwise_identical " << (raw_equal ? "true" : "false") << "\n";
  for (size_t index = 0; index < times.size(); ++index) {
    report << "run" << (index + 1) << "_ms " << times[index] << "\n";
  }
  report << "free_mib_before " << (free_bytes >> 20) << "\n";
  size_t free_after = 0;
  size_t total_after = 0;
  PROBE_CHECK(cudaMemGetInfo(&free_after, &total_after));
  report << "free_mib_after " << (free_after >> 20) << "\n";
  return 0;
}
