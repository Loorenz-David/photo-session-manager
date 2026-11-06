#include <CommonCrypto/CommonDigest.h>
#include <sys/stat.h>

#include <cstdio>
#include <vector>

#include "crlab/sha256.hpp"

namespace crlab {

std::string sha256_file(const std::string& path) {
  FILE* f = std::fopen(path.c_str(), "rb");
  if (!f) return {};
  CC_SHA256_CTX ctx;
  CC_SHA256_Init(&ctx);
  std::vector<unsigned char> buf(4 * 1024 * 1024);
  size_t n;
  while ((n = std::fread(buf.data(), 1, buf.size(), f)) > 0) CC_SHA256_Update(&ctx, buf.data(), static_cast<CC_LONG>(n));
  bool err = std::ferror(f);
  std::fclose(f);
  if (err) return {};
  unsigned char md[CC_SHA256_DIGEST_LENGTH];
  CC_SHA256_Final(md, &ctx);
  static const char* hx = "0123456789abcdef";
  std::string out;
  for (unsigned char c : md) {
    out += hx[c >> 4];
    out += hx[c & 15];
  }
  return out;
}

long long file_size(const std::string& path) {
  struct stat st{};
  if (::stat(path.c_str(), &st) != 0) return -1;
  return static_cast<long long>(st.st_size);
}

}  // namespace crlab
