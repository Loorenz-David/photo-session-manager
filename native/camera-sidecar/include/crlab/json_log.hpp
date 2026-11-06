// Structured JSON-lines event emitter for the native CLI.
//
// Emits the same envelope as camlab/events.py so both streams can be merged:
//   {"timestamp":"...Z","experiment":"...","source":"sdk","event":"...", ...}
// No third-party dependency: a tiny ordered field builder with escaping.
#pragma once

#include <cstdint>
#include <cstdio>
#include <mutex>
#include <optional>
#include <string>
#include <utility>
#include <vector>

namespace crlab {

std::string json_escape(const std::string& s);
// Reverses RFC 3986 percent-encoding (%XX -> byte). A malformed escape (a trailing "%", "%4",
// "%G1") is kept literally. Used for the app's `credentials` stdin line (R-I21).
std::string percent_decode(const std::string& s);
std::string iso_utc_now();      // 2026-11-05T08:00:00.123Z
int64_t monotonic_ms();         // steady clock, for durations

// Ordered set of JSON fields. Values are stored pre-encoded.
class Fields {
 public:
  Fields& str(const std::string& k, const std::string& v);
  Fields& num(const std::string& k, int64_t v);
  Fields& unum(const std::string& k, uint64_t v);
  Fields& dbl(const std::string& k, double v);
  Fields& boolean(const std::string& k, bool v);
  Fields& hex(const std::string& k, uint64_t v);  // "0x0000800A" (as string)
  Fields& raw(const std::string& k, const std::string& encodedJson);
  Fields& opt_str(const std::string& k, const std::optional<std::string>& v) {
    if (v) str(k, *v);
    return *this;
  }
  bool empty() const { return items_.empty(); }
  std::string encode_object() const;
  const std::vector<std::pair<std::string, std::string>>& items() const { return items_; }

 private:
  std::vector<std::pair<std::string, std::string>> items_;
};

// Abstract sink so tests can capture events.
class EventSink {
 public:
  virtual ~EventSink() = default;
  virtual void emit(const std::string& event, const Fields& fields) = 0;
};

// Writes JSON lines to stdout (and optionally a file). Thread-safe.
class JsonLog : public EventSink {
 public:
  JsonLog(std::string experiment, std::string source = "sdk");
  ~JsonLog() override;
  bool open_file(const std::string& path);  // append
  void set_default(const std::string& key, const std::string& value);  // e.g. camera_id
  void clear_default(const std::string& key);
  void emit(const std::string& event, const Fields& fields) override;
  void emit(const std::string& event) { emit(event, Fields{}); }

 private:
  std::string experiment_;
  std::string source_;
  std::vector<std::pair<std::string, std::string>> defaults_;
  FILE* file_ = nullptr;
  std::mutex mu_;
};

}  // namespace crlab
