#include "crlab/fingerprint_policy.hpp"
#include "mini_test.hpp"
using namespace crlab;

void fingerprint_c1a() {
  FingerprintPolicy p;
  CHECK_EQ(p.arm("aa:bb", false), "");
  CHECK_EQ(p.decide(std::nullopt, "AA:BB", "aa:bb"), FingerprintDecision::Pin);
  CHECK_EQ(p.decide(std::nullopt, "AA:BB", "aa:bb"), FingerprintDecision::Unknown);
}
void fingerprint_c1b() {
  FingerprintPolicy p;
  CHECK_EQ(p.arm("aa:bb", false), "");
  CHECK_EQ(p.decide(std::nullopt, "cc:dd", "cc:dd"), FingerprintDecision::Unknown);
  CHECK_EQ(p.decide(std::nullopt, "aa:bb", "aa:bb"), FingerprintDecision::Unknown);
}
void fingerprint_c1c() {
  FingerprintPolicy p;
  CHECK_EQ(p.arm("cc:dd", true), "");
  CHECK_EQ(p.decide(std::string("X"), "cc:dd", "cc:dd"), FingerprintDecision::Replace);
  CHECK_EQ(p.arm("other", true), "");
  CHECK_EQ(p.decide(std::string("X"), "cc:dd", "cc:dd"), FingerprintDecision::Mismatch);
  CHECK_EQ(p.decide(std::string("X"), "other", "other"), FingerprintDecision::Mismatch);
}
void fingerprint_c1d() {
  FingerprintPolicy p;
  CHECK_EQ(p.arm("other", true), "");
  CHECK_EQ(p.decide(std::string("RAW"), "RAW", "raw"), FingerprintDecision::Match);
  CHECK_EQ(p.decide(std::nullopt, "other", "other"), FingerprintDecision::Pin);
  CHECK_EQ(p.decide(std::nullopt, "x", "x"), FingerprintDecision::Unknown);
  CHECK_EQ(p.decide(std::string("X"), "y", "y"), FingerprintDecision::Mismatch);
}
void fingerprint_c2a() {
  FingerprintPolicy p;
  CHECK_EQ(p.arm("", true), "fingerprint required");
  CHECK_EQ(p.decide(std::nullopt, "aa:bb", "aa:bb"), FingerprintDecision::Unknown);
}

