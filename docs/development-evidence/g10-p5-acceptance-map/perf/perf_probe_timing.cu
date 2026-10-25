// G6-2 性能探针 · 计时模块（authoring-only，不进产品构建）。
//
// 与 `perf_probe_common.cu`（冻结 G5 探针源 1-426 行）和 `perf_probe.cu`（427-743 行 + `--perf`
// 分支）同一翻译单元；只用冻结探针定义的结构体与辅助函数，不复制内核、不改内核语义。
//
// 两栏口径（证据 README 与 receipt 同此）：
//   1. kernel-only：cudaEventElapsedTime 只包 `kernel<<<nworld,1>>>(args)` 一次发射（清零在
//      start 事件之前、同一流，由流序保证不计入）；warmup 5 + 50 次。
//   2. end-to-end：host `steady_clock` 包「几何位姿 H2D → 输出清零 → kernel → 全部输出 D2H →
//      cudaDeviceSynchronize」一次完整调用；warmup 5 + 50 次。
// 计时只作性能证据，不进精度判据、不跨硬件外推。

#include <algorithm>
#include <cmath>

namespace {

constexpr int kPerfDefaultWarmup = 5;
constexpr int kPerfDefaultReps = 50;

double perf_median(std::vector<double> values) {
  if (values.empty()) return 0.0;
  std::sort(values.begin(), values.end());
  const size_t middle = values.size() / 2;
  if (values.size() % 2 == 1) return values[middle];
  return 0.5 * (values[middle - 1] + values[middle]);
}

void perf_write_numbers(std::ofstream& out, const std::vector<double>& values) {
  out << "[";
  for (size_t index = 0; index < values.size(); ++index) {
    if (index != 0) out << ", ";
    out << static_cast<long long>(std::llround(values[index]));
  }
  out << "]";
}

void perf_write_overflow(std::ofstream& out, const std::vector<unsigned>& values) {
  out << "[";
  for (size_t index = 0; index < values.size(); ++index) {
    if (index != 0) out << ", ";
    out << values[index];
  }
  out << "]";
}

// 返回 0 成功；非 0 为探针语义错误码（窗口脚本按退出码判失败）。
int run_perf_mode(const std::string& directory, const DeviceArrays& host, ProbeArgs& args,
                  void (*kernel)(ProbeArgs), const CaseMeta& meta,
                  const cudaDeviceProp& properties, int warmup, int reps) {
  if (warmup < 0) warmup = 0;
  if (reps <= 0) {
    std::fprintf(stderr, "perf: reps 必须为正\n");
    return 2;
  }
  const int nworld = args.nworld;
  const int naconmax = args.naconmax;
  const int efc_width = args.efc_width;

  size_t free_before = 0;
  size_t total_bytes = 0;
  PROBE_CHECK(cudaMemGetInfo(&free_before, &total_bytes));

  auto clear_outputs = [&]() {
    PROBE_CHECK(cudaMemset(args.nacon, 0, sizeof(int)));
    PROBE_CHECK(cudaMemset(args.nccd, 0, sizeof(int)));
    PROBE_CHECK(cudaMemset(args.overflow, 0, static_cast<size_t>(nworld) * sizeof(unsigned)));
    PROBE_CHECK(cudaMemset(args.out.efc_address, 0xff,
                           static_cast<size_t>(naconmax) * efc_width * sizeof(int)));
  };
  auto launch = [&]() {
    kernel<<<nworld, 1>>>(args);
    PROBE_CHECK(cudaGetLastError());
  };

  // 第一栏：kernel-only（CUDA events）。
  for (int index = 0; index < warmup; ++index) {
    clear_outputs();
    launch();
    PROBE_CHECK(cudaDeviceSynchronize());
  }
  std::vector<double> kernel_ns;
  kernel_ns.reserve(static_cast<size_t>(reps));
  cudaEvent_t start = nullptr;
  cudaEvent_t stop = nullptr;
  PROBE_CHECK(cudaEventCreate(&start));
  PROBE_CHECK(cudaEventCreate(&stop));
  for (int index = 0; index < reps; ++index) {
    clear_outputs();
    PROBE_CHECK(cudaEventRecord(start));
    launch();
    PROBE_CHECK(cudaEventRecord(stop));
    PROBE_CHECK(cudaEventSynchronize(stop));
    float milliseconds = 0.0F;
    PROBE_CHECK(cudaEventElapsedTime(&milliseconds, start, stop));
    kernel_ns.push_back(static_cast<double>(milliseconds) * 1.0e6);
  }
  PROBE_CHECK(cudaEventDestroy(start));
  PROBE_CHECK(cudaEventDestroy(stop));

  // 第二栏：end-to-end（含几何位姿 H2D 与全部输出 D2H）。
  std::vector<float> h_dist(naconmax);
  std::vector<float> h_pos(static_cast<size_t>(naconmax) * 3);
  std::vector<float> h_frame(static_cast<size_t>(naconmax) * 9);
  std::vector<float> h_includemargin(naconmax);
  std::vector<float> h_friction(static_cast<size_t>(naconmax) * 5);
  std::vector<float> h_solref(static_cast<size_t>(naconmax) * 2);
  std::vector<float> h_solreffriction(static_cast<size_t>(naconmax) * 2);
  std::vector<float> h_solimp(static_cast<size_t>(naconmax) * 5);
  std::vector<float> h_adhesion(naconmax);
  std::vector<int> h_dim(naconmax);
  std::vector<int> h_geom(static_cast<size_t>(naconmax) * 2);
  std::vector<int> h_efc(static_cast<size_t>(naconmax) * efc_width);
  std::vector<int> h_worldid(naconmax);
  std::vector<int> h_type(naconmax);
  std::vector<int> h_geomcollisionid(naconmax);
  std::vector<unsigned> h_overflow(nworld);
  std::vector<int> h_nacon(1);
  std::vector<int> h_nccd(1);

  auto e2e_once = [&]() {
    // 产品每步会更新的几何位姿：按端到端口径计入 H2D。
    PROBE_CHECK(cudaMemcpy(const_cast<float*>(args.geom_xpos), host.geom_xpos.data(),
                           host.geom_xpos.size() * sizeof(float), cudaMemcpyHostToDevice));
    PROBE_CHECK(cudaMemcpy(const_cast<float*>(args.geom_xmat), host.geom_xmat.data(),
                           host.geom_xmat.size() * sizeof(float), cudaMemcpyHostToDevice));
    clear_outputs();
    launch();
    PROBE_CHECK(cudaDeviceSynchronize());
    from_device(args.out.dist, h_dist);
    from_device(reinterpret_cast<const float*>(args.out.pos), h_pos);
    from_device(args.out.frame, h_frame);
    from_device(args.out.includemargin, h_includemargin);
    from_device(args.out.friction, h_friction);
    from_device(args.out.solref, h_solref);
    from_device(args.out.solreffriction, h_solreffriction);
    from_device(args.out.solimp, h_solimp);
    from_device(args.out.adhesion, h_adhesion);
    from_device(args.out.dim, h_dim);
    from_device(args.out.geom, h_geom);
    from_device(args.out.efc_address, h_efc);
    from_device(args.out.worldid, h_worldid);
    from_device(args.out.type, h_type);
    from_device(args.out.geomcollisionid, h_geomcollisionid);
    from_device(args.nacon, h_nacon);
    from_device(args.nccd, h_nccd);
    from_device(args.overflow, h_overflow);
  };

  for (int index = 0; index < warmup; ++index) e2e_once();
  std::vector<double> e2e_ns;
  e2e_ns.reserve(static_cast<size_t>(reps));
  for (int index = 0; index < reps; ++index) {
    const auto started = std::chrono::steady_clock::now();
    e2e_once();
    const auto elapsed = std::chrono::steady_clock::now() - started;
    e2e_ns.push_back(
        static_cast<double>(std::chrono::duration_cast<std::chrono::nanoseconds>(elapsed).count()));
  }

  // 最后一轮 e2e 的输出计数留作证据（与 G5 已验收 nacon/nccd/overflow 对照用，不作本片判据）。
  const int nacon = h_nacon.empty() ? 0 : h_nacon[0];
  const int nccd = h_nccd.empty() ? 0 : h_nccd[0];
  size_t free_after = 0;
  size_t total_after = 0;
  PROBE_CHECK(cudaMemGetInfo(&free_after, &total_after));

  const double kernel_median = perf_median(kernel_ns);
  const double e2e_median = perf_median(e2e_ns);

  std::ofstream json(directory + "/perf-report.json");
  if (!json) {
    std::fprintf(stderr, "perf: 无法写 %s/perf-report.json\n", directory.c_str());
    return 3;
  }
  json << "{\n";
  json << "  \"schema\": 1,\n";
  json << "  \"kind\": \"device-perf\",\n";
  json << "  \"case\": \"" << meta.text("case") << "\",\n";
  json << "  \"kernel_kind\": \"" << meta.text("kind") << "\",\n";
  json << "  \"device\": \"" << properties.name << "\",\n";
  json << "  \"compute_capability\": \"" << properties.major << "." << properties.minor << "\",\n";
  json << "  \"cudart_version\": " << CUDART_VERSION << ",\n";
  json << "  \"nworld\": " << nworld << ",\n";
  json << "  \"naconmax\": " << naconmax << ",\n";
  json << "  \"efc_width\": " << efc_width << ",\n";
  json << "  \"warmup\": " << warmup << ",\n";
  json << "  \"reps\": " << reps << ",\n";
  json << "  \"kernel_only_method\": \"cudaEventElapsedTime around one kernel launch; "
          "output clears before start event on same stream\",\n";
  json << "  \"end_to_end_method\": \"host steady_clock: geom_xpos/xmat H2D + output clears + "
          "kernel + all output D2H + sync\",\n";
  json << "  \"kernel_ns\": ";
  perf_write_numbers(json, kernel_ns);
  json << ",\n";
  json << "  \"e2e_ns\": ";
  perf_write_numbers(json, e2e_ns);
  json << ",\n";
  json << "  \"kernel_median_ns\": " << static_cast<long long>(std::llround(kernel_median)) << ",\n";
  json << "  \"e2e_median_ns\": " << static_cast<long long>(std::llround(e2e_median)) << ",\n";
  json << "  \"nacon\": " << nacon << ",\n";
  json << "  \"nccd\": " << nccd << ",\n";
  json << "  \"overflow\": ";
  perf_write_overflow(json, h_overflow);
  json << ",\n";
  json << "  \"free_mib_before\": " << (free_before >> 20) << ",\n";
  json << "  \"free_mib_after\": " << (free_after >> 20) << ",\n";
  json << "  \"total_mib\": " << (total_bytes >> 20) << ",\n";
  json << "  \"note\": \"G6-2 device perf; kernel-only and end-to-end columns; performance is "
          "not an accuracy criterion and is not extrapolated across hardware\"\n";
  json << "}\n";
  json.close();

  std::printf(
      "perf case=%s kind=%s warmup=%d reps=%d kernel_median_ms=%.6f e2e_median_ms=%.6f "
      "nacon=%d nccd=%d free_before_mib=%zu free_after_mib=%zu\n",
      meta.text("case").c_str(), meta.text("kind").c_str(), warmup, reps, kernel_median / 1.0e6,
      e2e_median / 1.0e6, nacon, nccd, free_before >> 20, free_after >> 20);
  return 0;
}

}  // namespace
