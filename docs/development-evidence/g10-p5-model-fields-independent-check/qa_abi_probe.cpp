// QA G1 independent ABI probe: call the product native G01 export contract directly
// (mjwarp_native_g01_field_info / mjwarp_native_copy_g01_field) on a 3.12 MJB and dump
// every field's raw little-endian bytes. Does not use the delivered Rust reader logic.
//
// The probe is compiled twice: against the pre-G1 commit and the reviewed G1 commit,
// with the same source file; the dump manifests let QA compare field bytes/kind/count.
#include <windows.h>

#include <cstdint>
#include <cstdlib>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

// Delivered native translation unit (the object under review is the field table/ABI;
// QA compiles it from the commit snapshot, but parses/outputs the bytes itself).
#include "native/model_probe_sparse.cpp"

namespace {

constexpr uint32_t kMaxKind = 5;

uint32_t scalar_bytes(uint32_t kind) {
  switch (kind) {
    case 1: return 4;  // int32
    case 2: return 8;  // double
    case 3: return 1;  // uint8
    case 4: return 4;  // float32
    case 5: return 8;  // int64
    default: return 0;
  }
}

std::vector<unsigned char> read_file(const wchar_t* path, bool* ok) {
  *ok = false;
  FILE* file = _wfopen(path, L"rb");
  if (!file) return {};
  _fseeki64(file, 0, SEEK_END);
  const long long size = _ftelli64(file);
  _fseeki64(file, 0, SEEK_SET);
  if (size < 0) {
    fclose(file);
    return {};
  }
  std::vector<unsigned char> data(static_cast<size_t>(size));
  const size_t got = fread(data.data(), 1, data.size(), file);
  fclose(file);
  *ok = got == data.size();
  return data;
}

bool write_file(const std::wstring& path, const void* data, size_t bytes) {
  FILE* file = _wfopen(path.c_str(), L"wb");
  if (!file) return false;
  const size_t wrote = bytes ? fwrite(data, 1, bytes, file) : 0;
  fclose(file);
  return wrote == bytes;
}

std::wstring join(const std::wstring& dir, const std::wstring& name) {
  if (dir.empty()) return name;
  const wchar_t last = dir.back();
  if (last == L'\\' || last == L'/') return dir + name;
  return dir + L"\\" + name;
}

}  // namespace

int wmain(int argc, wchar_t** argv) {
  if (argc < 4) {
    fwprintf(stderr, L"usage: %ls <mujoco_dll> <mjb> <out_dir> [max_id]\n", argv[0]);
    return 2;
  }
  const uint32_t max_id = argc >= 5 ? static_cast<uint32_t>(_wtoi(argv[4])) : 58;
  bool ok = false;
  const std::vector<unsigned char> mjb = read_file(argv[2], &ok);
  if (!ok) {
    fwprintf(stderr, L"cannot read mjb %ls\n", argv[2]);
    return 3;
  }
  CreateDirectoryW(argv[3], nullptr);
  mjwarp_native_owner* owner = nullptr;
  mjwarp_native_info info{};
  uint32_t detail = 0;
  const int32_t status = mjwarp_native_open(reinterpret_cast<const mjwarp_native_path_char*>(argv[1]), mjb.data(),
                                            static_cast<int32_t>(mjb.size()),
                                            &owner, &info, &detail);
  wprintf(L"open_status=%d detail=%u\n", status, detail);
  if (status != 0 || !owner) return 4;
  wprintf(L"native_info schema=%u version=%d nbody=%lld ngeom=%lld\n",
          info.schema, info.native_version,
          static_cast<long long>(info.nbody), static_cast<long long>(info.ngeom));

  FILE* manifest = _wfopen(join(argv[3], L"manifest.csv").c_str(), L"wb");
  if (!manifest) return 5;
  fprintf(manifest, "id,kind,count,bytes,present\n");
  uint32_t present = 0;
  for (uint32_t id = 0; id <= max_id; ++id) {
    mjwarp_g01_field_info field{};
    const int32_t info_status = mjwarp_native_g01_field_info(owner, id, &field);
    if (info_status != 0) {
      fprintf(manifest, "%u,0,0,0,0\n", id);
      continue;
    }
    const uint32_t width = scalar_bytes(field.kind);
    const uint64_t total = static_cast<uint64_t>(field.count) * width;
    std::vector<unsigned char> buffer(total ? total : 1, 0);
    const int32_t copy_status =
        mjwarp_native_copy_g01_field(owner, id, &field, buffer.data());
    if (copy_status != 0) {
      fprintf(manifest, "%u,%u,%lld,%llu,copy_fail_%d\n", id, field.kind,
              static_cast<long long>(field.count),
              static_cast<unsigned long long>(total), copy_status);
      continue;
    }
    wchar_t name[32];
    swprintf(name, 32, L"%u.bin", id);
    if (!write_file(join(argv[3], name), buffer.data(), static_cast<size_t>(total))) {
      fwprintf(stderr, L"write failed for id %u\n", id);
      return 6;
    }
    fprintf(manifest, "%u,%u,%lld,%llu,1\n", id, field.kind,
            static_cast<long long>(field.count),
            static_cast<unsigned long long>(total));
    ++present;
  }
  fclose(manifest);

  uint32_t neg_wrong_kind = 0;
  uint32_t neg_wrong_count = 0;
  uint32_t neg_null_target = 0;
  if (max_id >= 58) {
    mjwarp_g01_field_info field{};
    if (mjwarp_native_g01_field_info(owner, 57, &field) == 0) {
      unsigned char buffer[16] = {0};
      mjwarp_g01_field_info wrong_kind = field;
      wrong_kind.kind = 2;
      neg_wrong_kind = static_cast<uint32_t>(
          mjwarp_native_copy_g01_field(owner, 57, &wrong_kind, buffer) != 0);
      mjwarp_g01_field_info wrong_count = field;
      wrong_count.count = field.count + 1;
      neg_wrong_count = static_cast<uint32_t>(
          mjwarp_native_copy_g01_field(owner, 57, &wrong_count, buffer) != 0);
      neg_null_target = static_cast<uint32_t>(
          mjwarp_native_copy_g01_field(owner, 57, &field, nullptr) != 0);
    }
  }

  FILE* abi = _wfopen(join(argv[3], L"abi.txt").c_str(), L"wb");
  if (abi) {
    fprintf(abi, "field_info_size=%zu\n", sizeof(mjwarp_g01_field_info));
    fprintf(abi, "field_info_align=%zu\n", alignof(mjwarp_g01_field_info));
    fprintf(abi, "field_info_off_count=%zu\n", offsetof(mjwarp_g01_field_info, count));
    fprintf(abi, "native_info_size=%zu\n", sizeof(mjwarp_native_info));
    fprintf(abi, "native_info_align=%zu\n", alignof(mjwarp_native_info));
    fprintf(abi, "present_fields=%u\n", present);
    fprintf(abi, "neg_wrong_kind_rejected=%u\n", neg_wrong_kind);
    fprintf(abi, "neg_wrong_count_rejected=%u\n", neg_wrong_count);
    fprintf(abi, "neg_null_target_rejected=%u\n", neg_null_target);
    fclose(abi);
  }
  mjwarp_native_close(owner);
  wprintf(L"present_fields=%u max_id=%u out=%ls\n", present, max_id, argv[3]);
  return 0;
}
