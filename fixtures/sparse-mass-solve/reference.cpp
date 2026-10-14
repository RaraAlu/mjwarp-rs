// Offline independent LU reference. Never linked into the product.
#include <algorithm>
#include <cmath>
#include <cstdint>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>
#include <vector>

template <class T> T read(std::ifstream& in) {
  T value{};
  if (!in.read(reinterpret_cast<char*>(&value), sizeof(value)))
    throw std::runtime_error("truncated input");
  return value;
}
std::vector<int32_t> ints(std::ifstream& in, int n) {
  std::vector<int32_t> result(n);
  for (auto& value : result) value = read<int32_t>(in);
  return result;
}
template <class T> void array(std::ostream& out, const std::vector<T>& values) {
  out << '[';
  for (size_t i = 0; i < values.size(); ++i) {
    if (i) out << ',';
    out << values[i];
  }
  out << ']';
}
int main(int argc, char** argv) try {
  static_assert(sizeof(float) == 4 && sizeof(double) == 8);
  static_assert(std::numeric_limits<float>::is_iec559);
  if (argc != 3) throw std::runtime_error("input.bin output.json required");
  std::ifstream in(argv[1], std::ios::binary);
  char magic[8];
  if (!in.read(magic, 8) || std::string(magic, 8) != "SMREF001")
    throw std::runtime_error("bad schema");
  const int models = read<int32_t>(in);
  if (models != 3) throw std::runtime_error("expected three frozen models");
  std::ofstream out(argv[2], std::ios::binary);
  out << std::setprecision(17) << "{\"schema\":1,\"models\":[";
  for (int model = 0; model < models; ++model) {
    const int length = read<int32_t>(in);
    if (length < 1 || length > 100) throw std::runtime_error("name length");
    std::string name(length, ' ');
    if (!in.read(name.data(), length) || name.find_first_not_of("abcdefghijklmnopqrstuvwxyz0123456789-") != std::string::npos)
      throw std::runtime_error("name");
    const int n = read<int32_t>(in), nc = read<int32_t>(in), cases = read<int32_t>(in);
    if (n < 65 || n > 100 || nc < n || nc > n * n || cases != 2)
      throw std::runtime_error("dimensions");
    const auto nnz = ints(in, n), adr = ints(in, n), col = ints(in, nc);
    int end = 0;
    for (int i = 0; i < n; ++i) {
      if (adr[i] != end || nnz[i] < 1 || nnz[i] > i + 1 || end + nnz[i] > nc)
        throw std::runtime_error("CSR row");
      int previous = -1;
      for (int j = end; j < end + nnz[i]; ++j) {
        if (col[j] <= previous || col[j] > i) throw std::runtime_error("CSR column");
        previous = col[j];
      }
      if (previous != i) throw std::runtime_error("CSR diagonal");
      end += nnz[i];
    }
    if (end != nc) throw std::runtime_error("CSR extent");
    if (model) out << ',';
    out << "{\"name\":\"" << name << "\",\"nv\":" << n << ",\"cases\":[";
    for (int sample = 0; sample < cases; ++sample) {
      std::vector<double> a(n * n, 0.0);
      for (int i = 0; i < n; ++i) for (int j = adr[i]; j < adr[i] + nnz[i]; ++j) {
        const float value = read<float>(in);
        if (!std::isfinite(value)) throw std::runtime_error("nonfinite matrix");
        a[i * n + col[j]] = a[col[j] * n + i] = value;
      }
      auto lu = a;
      std::vector<int> pivots(n);
      for (int k = 0; k < n; ++k) {
        int pivot = k;
        for (int i = k + 1; i < n; ++i)
          if (std::abs(lu[i * n + k]) > std::abs(lu[pivot * n + k])) pivot = i;
        if (!(std::abs(lu[pivot * n + k]) > 1e-30)) throw std::runtime_error("singular matrix");
        pivots[k] = pivot;
        for (int j = 0; j < n; ++j) std::swap(lu[k * n + j], lu[pivot * n + j]);
        for (int i = k + 1; i < n; ++i) {
          lu[i * n + k] /= lu[k * n + k];
          for (int j = k + 1; j < n; ++j) lu[i * n + j] -= lu[i * n + k] * lu[k * n + j];
        }
      }
      std::vector<float> rhs(3 * n, 0.0f);
      uint32_t seed = 9701;
      for (int i = 0; i < n; ++i) {
        rhs[n + i] = static_cast<float>((i % 2 ? -1.0 : 1.0) * (i + 1) * 0.125);
        seed = 1664525u * seed + 1013904223u;
        rhs[2 * n + i] = static_cast<float>((static_cast<int32_t>(seed >> 8) - 8388608) / 8388608.0);
      }
      std::vector<double> solution(rhs.begin(), rhs.end());
      double max_residual = 0.0, norm = 0.0;
      for (int i = 0; i < n; ++i) {
        double row = 0.0;
        for (int j = 0; j < n; ++j) row += std::abs(a[i * n + j]);
        norm = std::max(norm, row);
      }
      for (int r = 0; r < 3; ++r) {
        double* x = solution.data() + r * n;
        for (int k = 0; k < n; ++k) std::swap(x[k], x[pivots[k]]);
        for (int i = 0; i < n; ++i) for (int j = 0; j < i; ++j) x[i] -= lu[i * n + j] * x[j];
        for (int i = n - 1; i >= 0; --i) {
          for (int j = i + 1; j < n; ++j) x[i] -= lu[i * n + j] * x[j];
          x[i] /= lu[i * n + i];
        }
        double residual = 0.0, xn = 0.0, bn = 0.0;
        for (int i = 0; i < n; ++i) {
          double ax = 0.0;
          for (int j = 0; j < n; ++j) ax += a[i * n + j] * x[j];
          residual = std::max(residual, std::abs(ax - rhs[r * n + i]));
          xn = std::max(xn, std::abs(x[i]));
          bn = std::max(bn, std::abs(static_cast<double>(rhs[r * n + i])));
        }
        max_residual = std::max(max_residual, residual / std::max(norm * xn + bn, 1e-30));
      }
      if (!(max_residual <= 1e-12)) throw std::runtime_error("reference residual");
      if (sample) out << ',';
      out << "{\"rhs\":"; array(out, rhs);
      out << ",\"solution\":"; array(out, solution);
      out << ",\"reference_residual\":" << max_residual << '}';
    }
    out << "]}";
  }
  if (in.peek() != std::char_traits<char>::eof()) throw std::runtime_error("trailing input");
  out << "]}\n";
  if (!out.good()) throw std::runtime_error("output failed");
  std::cout << "independent LU: 3 models, 6 matrices, 18 RHS columns\n";
  return 0;
} catch (const std::exception& error) {
  std::cerr << error.what() << '\n'; return 1;
}
