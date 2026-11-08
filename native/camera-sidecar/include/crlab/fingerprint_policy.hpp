#pragma once
#include <mutex>
#include <optional>
#include <string>

namespace crlab {
struct FingerprintExpectation { std::string fp; bool replace; };
enum class FingerprintDecision { Match, Pin, Replace, Mismatch, Unknown };

class FingerprintPolicy {
 public:
  // Empty result means accepted; otherwise the command rejection reason.
  std::string arm(const std::string& display_fp, bool replace);
  FingerprintDecision decide(const std::optional<std::string>& pinned,
                             const std::string& presented_raw, const std::string& presented_display,
                             bool cli_accept = false);
 private:
  std::mutex mu_;
  std::optional<FingerprintExpectation> expectation_;
};
}
