#pragma once

#include <algorithm>
#include <array>
#include <chrono>
#include <condition_variable>
#include <cstdint>
#include <deque>
#include <mutex>
#include <memory>
#include <optional>
#include <string>
#include <vector>

namespace crlab {

class CameraSession;
class EventSink;

enum class PullKind { Ok, NotUpdated, Error, Unsupported };

struct PaceResult { bool forward = false; bool stop = false; int64_t next_ms = 0; };

class LivePacer {
 public:
  void reset(uint32_t epoch) {
    epoch_ = epoch; last_number_ = 0; last_hash_ = 0; last_size_ = 0;
    have_last_ = false; last_forwarded_ = -1; intervals_.clear(); frames_ = repeats_ = errors_ = 0;
  }
  PaceResult on_pull(PullKind kind, uint32_t frame_no, const std::vector<uint8_t>& jpeg,
                     uint32_t sdk_error, int64_t now_ms) {
    if (kind == PullKind::Unsupported) return {false, true, now_ms};
    if (kind == PullKind::NotUpdated) return {false, false, now_ms + 5};
    if (kind == PullKind::Error) {
      if (sdk_error) ++errors_;
      return {false, false, now_ms + 20};
    }
    uint64_t hash = 14695981039346656037ULL;
    for (uint8_t byte : jpeg) { hash ^= byte; hash *= 1099511628211ULL; }
    bool repeat = have_last_ && (frame_no ? frame_no == last_number_ :
                                 last_number_ == 0 && jpeg.size() == last_size_ && hash == last_hash_);
    if (repeat) { ++repeats_; return {false, false, now_ms + 5}; }
    if (last_forwarded_ >= 0) {
      intervals_.push_back(now_ms - last_forwarded_);
      if (intervals_.size() > 16) intervals_.pop_front();
    }
    last_forwarded_ = now_ms; last_number_ = frame_no; last_hash_ = hash;
    last_size_ = jpeg.size(); have_last_ = true; ++frames_;
    int64_t estimate = 40;
    if (!intervals_.empty()) {
      std::vector<int64_t> sorted(intervals_.begin(), intervals_.end());
      std::sort(sorted.begin(), sorted.end());
      estimate = sorted[(sorted.size() - 1) / 2];
    }
    estimate = std::max<int64_t>(20, std::min<int64_t>(200, estimate));
    return {true, false, now_ms + std::max<int64_t>(15, estimate - 10)};
  }
  uint64_t frames() const { return frames_; }
  uint64_t repeats() const { return repeats_; }
  uint64_t errors() const { return errors_; }
 private:
  uint32_t epoch_ = 0, last_number_ = 0;
  uint64_t last_hash_ = 0, frames_ = 0, repeats_ = 0, errors_ = 0;
  size_t last_size_ = 0;
  bool have_last_ = false;
  int64_t last_forwarded_ = -1;
  std::deque<int64_t> intervals_;
};

struct FrameRecord {
  uint32_t epoch = 0, frame_no = 0;
  uint16_t width = 0, height = 0;
  std::vector<uint8_t> jpeg;
  std::vector<uint8_t> encode() const {
    if (jpeg.empty() || jpeg.size() > 4194304) return {};
    std::vector<uint8_t> out = {'P','S','L','V',1,1,0,0};
    auto put32 = [&](uint32_t n) { for (int i=0;i<4;++i) out.push_back(static_cast<uint8_t>(n >> (8*i))); };
    auto put16 = [&](uint16_t n) { for (int i=0;i<2;++i) out.push_back(static_cast<uint8_t>(n >> (8*i))); };
    put32(epoch); put32(frame_no); put16(width); put16(height); put32(static_cast<uint32_t>(jpeg.size()));
    out.insert(out.end(), jpeg.begin(), jpeg.end());
    return out;
  }
  static std::pair<uint16_t,uint16_t> jpeg_size(const std::vector<uint8_t>& b) {
    if (b.size() < 4 || b[0] != 0xff || b[1] != 0xd8) return {0,0};
    size_t p=2;
    while (p+4 <= b.size()) {
      if (b[p++] != 0xff) return {0,0};
      uint8_t marker=b[p++];
      if (marker == 0xd9 || marker == 0xda) break;
      if (marker == 0xff || marker == 0x00 || (marker >= 0xd0 && marker <= 0xd7)) continue;
      uint16_t len=static_cast<uint16_t>((b[p]<<8)|b[p+1]);
      if (len < 2 || p+len > b.size()) return {0,0};
      if ((marker >= 0xc0 && marker <= 0xc3) && len >= 7) {
        return {static_cast<uint16_t>((b[p+5]<<8)|b[p+6]), static_cast<uint16_t>((b[p+3]<<8)|b[p+4])};
      }
      p += len;
    }
    return {0,0};
  }
};

template <class T> class LatestSlot {
 public:
  void put(T value) { { std::lock_guard<std::mutex> lk(mu_); item_ = std::move(value); } cv_.notify_one(); }
  std::optional<T> take(std::chrono::milliseconds timeout = std::chrono::milliseconds(0)) {
    std::unique_lock<std::mutex> lk(mu_);
    if (timeout.count() > 0) cv_.wait_for(lk, timeout, [&] { return item_.has_value() || closed_; });
    auto out=std::move(item_); item_.reset(); return out;
  }
  void close() { { std::lock_guard<std::mutex> lk(mu_); closed_=true; } cv_.notify_all(); }
 private:
  std::mutex mu_; std::condition_variable cv_; std::optional<T> item_; bool closed_=false;
};

struct FocusJob { int x=0, y=0; bool af=true; std::string cmd_id; };
class FocusQueue {
 public:
  std::optional<FocusJob> push(FocusJob j) {
    std::lock_guard<std::mutex> lk(mu_); auto old=std::move(waiting_); waiting_=std::move(j); cv_.notify_one(); return old;
  }
  std::optional<FocusJob> take(std::chrono::milliseconds timeout) {
    std::unique_lock<std::mutex> lk(mu_);
    cv_.wait_for(lk, timeout, [&] { return waiting_.has_value() || closed_; });
    auto out=std::move(waiting_); waiting_.reset(); return out;
  }
  void close() { { std::lock_guard<std::mutex> lk(mu_); closed_=true; } cv_.notify_all(); }
 private:
  std::mutex mu_; std::condition_variable cv_; std::optional<FocusJob> waiting_; bool closed_=false;
};

// Capture takes the next turn after a focus already in progress; a newly queued
// focus cannot overtake a waiting capture.
class ActionLock {
 public:
  void lock_capture() {
    std::unique_lock<std::mutex> lk(mu_); ++waiting_captures_;
    cv_.wait(lk,[&]{return !busy_;}); --waiting_captures_; busy_=true;
  }
  void lock_focus() {
    std::unique_lock<std::mutex> lk(mu_);
    cv_.wait(lk,[&]{return !busy_ && waiting_captures_==0;}); busy_=true;
  }
  void unlock() { {std::lock_guard<std::mutex> lk(mu_); busy_=false;} cv_.notify_all(); }
 private:
  std::mutex mu_; std::condition_variable cv_; bool busy_=false; unsigned waiting_captures_=0;
};

inline std::optional<FocusJob> parse_focus_point(const std::string& x, const std::string& y,
                                                bool af, const std::string& cmd_id) {
  if (x.empty() || y.empty()) return std::nullopt;
  auto number=[](const std::string& s) -> std::optional<int> {
    try { size_t end=0; int n=std::stoi(s,&end); if (end==s.size()) return n; } catch (...) {}
    return std::nullopt;
  };
  auto a=number(x), b=number(y);
  if (!a || !b || *a<0 || *a>639 || *b<0 || *b>479) return std::nullopt;
  return FocusJob{*a,*b,af,cmd_id};
}

class LiveRunner {
 public:
  LiveRunner(CameraSession& session, EventSink& log, ActionLock& action, int frame_fd);
  ~LiveRunner();
  LiveRunner(const LiveRunner&)=delete;
  LiveRunner& operator=(const LiveRunner&)=delete;
  void start(uint32_t epoch);
  void stop(const std::string& reason);
  void queue_focus(FocusJob job);
  bool running() const;
 private:
  struct Impl;
  std::unique_ptr<Impl> impl_;
};

} // namespace crlab
