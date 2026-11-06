#pragma once
#include <string>

namespace crlab {
// Hex SHA-256 of a file (CommonCrypto). Returns empty string on I/O error.
std::string sha256_file(const std::string& path);
long long file_size(const std::string& path);  // -1 on error
}  // namespace crlab
