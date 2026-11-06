// Minimal dependency-free test harness (keeps the build offline-capable).
#pragma once
#include <cstdio>
#include <functional>
#include <string>
#include <vector>

namespace mt {
struct Case {
  const char* name;
  std::function<void()> fn;
};
inline std::vector<Case>& registry() {
  static std::vector<Case> r;
  return r;
}
struct Reg {
  Reg(const char* n, std::function<void()> f) { registry().push_back({n, std::move(f)}); }
};
struct Failure {
  std::string msg;
};
}  // namespace mt

#define MT_CAT2(a, b) a##b
#define MT_CAT(a, b) MT_CAT2(a, b)
#define TEST_CASE(name)                                         \
  static void MT_CAT(mt_fn_, __LINE__)();                       \
  static mt::Reg MT_CAT(mt_reg_, __LINE__)(name, MT_CAT(mt_fn_, __LINE__)); \
  static void MT_CAT(mt_fn_, __LINE__)()

#define CHECK(cond)                                                                                   \
  do {                                                                                                \
    if (!(cond)) throw mt::Failure{std::string(__FILE__) + ":" + std::to_string(__LINE__) + ": CHECK(" #cond ")"}; \
  } while (0)

#define CHECK_EQ(a, b)                                                                                     \
  do {                                                                                                     \
    auto _va = (a);                                                                                        \
    auto _vb = (b);                                                                                        \
    if (!(_va == _vb))                                                                                     \
      throw mt::Failure{std::string(__FILE__) + ":" + std::to_string(__LINE__) + ": CHECK_EQ(" #a ", " #b ")"}; \
  } while (0)

inline int mt_run_all() {
  int failed = 0;
  for (auto& c : mt::registry()) {
    try {
      c.fn();
      std::printf("[ OK ] %s\n", c.name);
    } catch (const mt::Failure& f) {
      ++failed;
      std::printf("[FAIL] %s\n       %s\n", c.name, f.msg.c_str());
    } catch (const std::exception& e) {
      ++failed;
      std::printf("[FAIL] %s\n       exception: %s\n", c.name, e.what());
    }
  }
  std::printf("%zu tests, %d failed\n", mt::registry().size(), failed);
  return failed == 0 ? 0 : 1;
}
