#include "crlab/json_log.hpp"

#include <chrono>
#include <cinttypes>
#include <cmath>
#include <ctime>

namespace crlab {

namespace {
// Length of a valid UTF-8 sequence starting at s[i], or 0 if invalid.
size_t utf8_len(const std::string& s, size_t i) {
  unsigned char c = static_cast<unsigned char>(s[i]);
  size_t n = c >= 0xF0 && c <= 0xF4 ? 4 : c >= 0xE0 ? 3 : c >= 0xC2 && c <= 0xDF ? 2 : 0;
  if (n == 0 || i + n > s.size()) return 0;
  for (size_t k = 1; k < n; ++k)
    if ((static_cast<unsigned char>(s[i + k]) & 0xC0) != 0x80) return 0;
  return n;
}
}  // namespace

std::string json_escape(const std::string& s) {
  std::string out;
  out.reserve(s.size() + 2);
  for (size_t i = 0; i < s.size(); ++i) {
    unsigned char c = static_cast<unsigned char>(s[i]);
    if (c >= 0x80) {
      size_t n = utf8_len(s, i);
      if (n) {
        out.append(s, i, n);
        i += n - 1;
      } else {  // raw byte that is not valid UTF-8: keep it visible, keep JSON valid
        char buf[8];
        std::snprintf(buf, sizeof buf, "\\u%04x", c);
        out += buf;
      }
      continue;
    }
    switch (c) {
      case '"': out += "\\\""; break;
      case '\\': out += "\\\\"; break;
      case '\n': out += "\\n"; break;
      case '\r': out += "\\r"; break;
      case '\t': out += "\\t"; break;
      default:
        if (c < 0x20) {
          char buf[8];
          std::snprintf(buf, sizeof buf, "\\u%04x", c);
          out += buf;
        } else {
          out += static_cast<char>(c);
        }
    }
  }
  return out;
}

namespace {
int hex_digit(char c) {
  if (c >= '0' && c <= '9') return c - '0';
  if (c >= 'a' && c <= 'f') return c - 'a' + 10;
  if (c >= 'A' && c <= 'F') return c - 'A' + 10;
  return -1;
}
}  // namespace

std::string percent_decode(const std::string& s) {
  std::string out;
  out.reserve(s.size());
  for (size_t i = 0; i < s.size(); ++i) {
    if (s[i] == '%' && i + 2 < s.size()) {  // both hex digits exist: no read past the end
      int hi = hex_digit(s[i + 1]);
      int lo = hex_digit(s[i + 2]);
      if (hi >= 0 && lo >= 0) {
        out += static_cast<char>((hi << 4) | lo);
        i += 2;
        continue;
      }
    }
    out += s[i];
  }
  return out;
}

std::string iso_utc_now() {
  using namespace std::chrono;
  auto now = system_clock::now();
  auto secs = time_point_cast<seconds>(now);
  auto ms = duration_cast<milliseconds>(now - secs).count();
  std::time_t t = system_clock::to_time_t(secs);
  std::tm tm{};
  gmtime_r(&t, &tm);
  char buf[40];
  std::snprintf(buf, sizeof buf, "%04d-%02d-%02dT%02d:%02d:%02d.%03dZ", tm.tm_year + 1900, tm.tm_mon + 1,
                tm.tm_mday, tm.tm_hour, tm.tm_min, tm.tm_sec, static_cast<int>(ms));
  return buf;
}

int64_t monotonic_ms() {
  using namespace std::chrono;
  return duration_cast<milliseconds>(steady_clock::now().time_since_epoch()).count();
}

Fields& Fields::str(const std::string& k, const std::string& v) {
  items_.emplace_back(k, "\"" + json_escape(v) + "\"");
  return *this;
}
Fields& Fields::num(const std::string& k, int64_t v) {
  items_.emplace_back(k, std::to_string(v));
  return *this;
}
Fields& Fields::unum(const std::string& k, uint64_t v) {
  items_.emplace_back(k, std::to_string(v));
  return *this;
}
Fields& Fields::dbl(const std::string& k, double v) {
  if (!std::isfinite(v)) {
    items_.emplace_back(k, "null");
    return *this;
  }
  char buf[64];
  std::snprintf(buf, sizeof buf, "%.3f", v);
  items_.emplace_back(k, buf);
  return *this;
}
Fields& Fields::boolean(const std::string& k, bool v) {
  items_.emplace_back(k, v ? "true" : "false");
  return *this;
}
Fields& Fields::hex(const std::string& k, uint64_t v) {
  char buf[32];
  std::snprintf(buf, sizeof buf, "\"0x%08" PRIX64 "\"", v);
  items_.emplace_back(k, buf);
  return *this;
}
Fields& Fields::raw(const std::string& k, const std::string& encodedJson) {
  items_.emplace_back(k, encodedJson);
  return *this;
}

std::string Fields::encode_object() const {
  std::string out = "{";
  bool first = true;
  for (const auto& [k, v] : items_) {
    if (!first) out += ",";
    first = false;
    out += "\"" + json_escape(k) + "\":" + v;
  }
  return out + "}";
}

JsonLog::JsonLog(std::string experiment, std::string source)
    : experiment_(std::move(experiment)), source_(std::move(source)) {}

JsonLog::~JsonLog() {
  if (file_) std::fclose(file_);
}

bool JsonLog::open_file(const std::string& path) {
  std::lock_guard<std::mutex> lk(mu_);
  if (file_) std::fclose(file_);
  file_ = std::fopen(path.c_str(), "a");
  return file_ != nullptr;
}

void JsonLog::set_default(const std::string& key, const std::string& value) {
  std::lock_guard<std::mutex> lk(mu_);
  for (auto& kv : defaults_) {
    if (kv.first == key) {
      kv.second = value;
      return;
    }
  }
  defaults_.emplace_back(key, value);
}

void JsonLog::clear_default(const std::string& key) {
  std::lock_guard<std::mutex> lk(mu_);
  for (auto it = defaults_.begin(); it != defaults_.end(); ++it) {
    if (it->first == key) {
      defaults_.erase(it);
      return;
    }
  }
}

void JsonLog::emit(const std::string& event, const Fields& fields) {
  std::lock_guard<std::mutex> lk(mu_);
  std::string line = "{\"timestamp\":\"" + iso_utc_now() + "\",\"experiment\":\"" + json_escape(experiment_) +
                     "\",\"source\":\"" + json_escape(source_) + "\",\"event\":\"" + json_escape(event) + "\"";
  for (const auto& [k, v] : defaults_) {
    bool overridden = false;
    for (const auto& item : fields.items())
      if (item.first == k) overridden = true;
    if (!overridden) line += ",\"" + json_escape(k) + "\":\"" + json_escape(v) + "\"";
  }
  for (const auto& [k, v] : fields.items()) line += ",\"" + json_escape(k) + "\":" + v;
  line += "}\n";
  std::fputs(line.c_str(), stdout);
  std::fflush(stdout);
  if (file_) {
    std::fputs(line.c_str(), file_);
    std::fflush(file_);
  }
}

}  // namespace crlab
