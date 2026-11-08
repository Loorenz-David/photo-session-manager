#include "crlab/fingerprint_policy.hpp"

namespace crlab {
std::string FingerprintPolicy::arm(const std::string& display_fp, bool replace) {
  if (display_fp.empty()) return "fingerprint required";
  std::lock_guard<std::mutex> lock(mu_);
  expectation_ = FingerprintExpectation{display_fp, replace};
  return {};
}

FingerprintDecision FingerprintPolicy::decide(const std::optional<std::string>& pinned,
                                              const std::string& presented_raw,
                                              const std::string& presented_display, bool cli_accept) {
  std::lock_guard<std::mutex> lock(mu_);
  if (pinned && *pinned == presented_raw) return FingerprintDecision::Match;
  auto expected = std::move(expectation_);
  expectation_.reset();
  if (pinned) {
    return expected && expected->replace && expected->fp == presented_display
        ? FingerprintDecision::Replace : FingerprintDecision::Mismatch;
  }
  return (expected && expected->fp == presented_display) || cli_accept
      ? FingerprintDecision::Pin : FingerprintDecision::Unknown;
}
}
