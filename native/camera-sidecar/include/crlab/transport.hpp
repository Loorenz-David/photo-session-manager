// Transport manager: automatic USB-C <-> Wi-Fi switching for one camera.
//
// Pure decision logic. It knows nothing about the Sony SDK; it drives a
// CameraAdapter (real: SdkCameraAdapter, tests: MockCameraAdapter).
//
// Model: every transport change is a controlled disconnect + fresh connect.
// No attempt is made to migrate an SDK session between transports (Sony does
// not document such a capability).
//
// Threading: external inputs (USB presence, SDK disconnect callbacks, busy
// state) may arrive from any thread and are queued. All adapter calls happen
// on the thread that calls step(), so at most one SDK session operation is in
// flight at any time and duplicate sessions are structurally impossible.
#pragma once

#include <cstdint>
#include <deque>
#include <functional>
#include <mutex>
#include <optional>
#include <string>
#include <vector>

#include "crlab/json_log.hpp"

namespace crlab {

enum class Transport { None, Usb, Wifi };
enum class State {
  Disconnected,
  Discovering,
  ConnectingWifi,
  ConnectedWifi,
  ConnectingUsb,
  ConnectedUsb,
  Switching,
  Recovering,
  Error,
};

const char* to_string(Transport t);
const char* to_string(State s);

struct CameraEndpoint {
  std::string id;        // stable identifier (e.g. USB serial / MAC / SDK id)
  std::string model;     // e.g. "ILCE-7RM5"
  Transport transport = Transport::None;
  std::string address;   // IP for Wi-Fi, USB location/serial for USB
};

struct ConnectResult {
  bool ok = false;
  uint64_t session_id = 0;     // adapter-assigned; echoed by disconnect callbacks
  bool fatal = false;          // e.g. authentication rejected: retrying will not help
  uint32_t sdk_error = 0;      // CrError_* code, 0 if none
  std::string message;
  // Connect sequence (R-I19): the store destination written for this link and read back.
  // Empty = not attempted (the mock camera, the stub).
  std::string store_destination;  // "verified" | "failed"
  std::string store_value;        // value read back, "0x%08X"
  // A USB connect failed because another process holds the camera's USB interface
  // (IOKit `UsbExclusiveOwner`, e.g. ptpcamerad). Retrying before it lets go cannot succeed.
  bool held = false;
};

// Why the camera is not at a safe point for an intentional disconnect.
enum class BusyReason { None, Exposure, Bracketing, PixelShift, FileTransfer, Other };
const char* to_string(BusyReason r);

// Whether `owners` (IOKit `UsbExclusiveOwner` values, "pid 95408, ptpcamerad") name a process
// other than `self_pid` (P56 ruling 2: an interface this helper holds is not held "by another").
bool usb_held_by_other(const std::vector<std::string>& owners, int self_pid);

class CameraAdapter {
 public:
  virtual ~CameraAdapter() = default;
  // Blocking discovery restricted to one transport.
  virtual std::vector<CameraEndpoint> discover(Transport t) = 0;
  // Blocking connect; returns when connected, failed, or timed out.
  virtual ConnectResult connect(const CameraEndpoint& ep) = 0;
  // Blocking, idempotent. Must release all SDK resources of the session.
  virtual void disconnect() = 0;
  // Read the actual camera state after connecting; fields are adapter-defined.
  virtual bool read_state(Fields& out) = 0;
  // Whether the most recent discover() call saw the camera on transport `t`
  // (discovery enumerates all transports even when asked for one).
  virtual bool last_discovery_saw(Transport /*t*/) const { return false; }
  // Called once on a voluntary stop (quit / end of stdin), after the safe-point wait and before
  // the session is closed, only for an open session on `t` == Usb (R-I19). Never called when a
  // session is lost.
  virtual void before_voluntary_stop(Transport /*t*/) {}
};

struct TransportConfig {
  bool wifi_enabled = true;
  bool usb_enabled = true;
  // The claim window after a USB arrival (MC-31 as amended by P56 ruling 1): the first USB
  // attempt starts at once and failed attempts repeat every usb_claim_retry_ms until this long
  // after the arrival; they do not count towards usb_failures_before_wifi_fallback.
  int64_t usb_settle_ms = 2000;
  int64_t usb_claim_retry_ms = 200;
  int64_t backoff_initial_ms = 1000;
  int64_t backoff_max_ms = 30000;
  double backoff_factor = 2.0;
  int64_t min_dwell_ms = 3000;       // minimum time on a transport before an intentional switch
  int usb_failures_before_wifi_fallback = 3;
  int64_t busy_log_interval_ms = 5000;
  int64_t after_disconnect_delay_ms = 1000;  // let the camera release the old session
  int64_t stop_wait_ms = 10000;  // a voluntary stop waits this long for a safe point (R-I20/R-I22)
};

class TransportManager {
 public:
  using ClockFn = std::function<int64_t()>;  // monotonic milliseconds

  TransportManager(CameraAdapter& adapter, EventSink& log, TransportConfig cfg = {}, ClockFn clock = monotonic_ms);

  // --- inputs (thread-safe) -------------------------------------------
  void on_usb_presence(bool present, const std::string& detail = "");
  // session_id 0 = "current session". Callbacks from an older session (e.g. the
  // OnDisconnected that follows an intentional disconnect) are ignored.
  void on_sdk_disconnected(uint32_t sdk_error, uint64_t session_id = 0);
  void set_busy(BusyReason reason);  // BusyReason::None == safe point
  // A voluntary stop. While the camera is busy it waits for the safe point, at most
  // cfg.stop_wait_ms from the step that takes the request; meanwhile no connect or switch starts.
  void request_stop();
  // `usb-retry` (R-I23): clears the three-failure USB fallback without a re-plug.
  void usb_retry();
  // `reconnect` (R-I29, Session Manager P46): closes the current Wi-Fi session and retries Wi-Fi
  // at once, at the next safe point. Ignored unless a Wi-Fi session is open.
  void reconnect();
  // Whether another process holds the camera's USB interface (IOKit `UsbExclusiveOwner` names a
  // process other than this helper), polled by the caller (P56 ruling 2). A change from held to
  // not held while USB waits for its holder (after a fallback to Wi-Fi) opens a new claim window.
  void on_usb_holder(bool held_by_other);

  // --- driver ----------------------------------------------------------
  // Process queued inputs and take at most one blocking action. Call in a
  // loop (e.g. every 100 ms). Returns false once stopped.
  bool step();

  State state() const;
  Transport transport() const;
  int consecutive_failures() const { return failures_; }
  int64_t next_attempt_at() const { return next_attempt_ms_; }
  std::optional<CameraEndpoint> endpoint() const { return endpoint_; }
  static int64_t backoff_delay(const TransportConfig& cfg, int failures);

 private:
  struct Input {
    enum Kind { UsbPresence, SdkDisconnected, Busy, Stop, UsbRetry, Reconnect, UsbHolder } kind;
    bool flag = false;
    uint32_t code = 0;
    uint64_t session = 0;
    BusyReason busy = BusyReason::None;
    std::string detail;
  };

  void drain_inputs();
  void set_state(State s, const std::string& reason);
  Transport desired_transport() const;
  bool try_connect(Transport t);
  void session_lost(const std::string& reason, uint32_t code);
  void intentional_disconnect(const std::string& reason);
  void schedule_retry(Transport t, bool held = false);
  void park_usb(const std::string& why);
  bool usb_parked() const { return usb_retry_at_ == INT64_MAX; }
  void arm_claim();

  CameraAdapter& adapter_;
  EventSink& log_;
  TransportConfig cfg_;
  ClockFn clock_;

  mutable std::mutex mu_;
  std::deque<Input> inputs_;

  State state_ = State::Disconnected;
  Transport active_ = Transport::None;
  bool session_open_ = false;
  bool usb_present_ = false;
  bool usb_from_discovery_ = false;  // usb_present_ inferred from SDK discovery, not IOKit
  int64_t usb_changed_at_ = 0;
  BusyReason busy_ = BusyReason::None;
  int64_t busy_logged_at_ = -1;
  bool stop_ = false;
  int64_t stop_requested_at_ = 0;  // when the step drained the Stop input
  int failures_ = 0;
  int usb_failures_ = 0;
  int64_t next_attempt_ms_ = 0;
  int64_t connected_at_ = 0;
  int64_t lost_at_ = -1;
  int64_t usb_retry_at_ = 0;     // after USB fallback: INT64_MAX until re-plug, holder release or usb-retry
  bool claim_pending_ = false;   // a USB arrival (or release) waits for its claim window
  int64_t claim_until_ = -1;     // end of the claim window in progress, -1 = none
  int claim_attempts_ = 0;       // USB attempts in the claim window (never counted as failures)
  bool usb_held_ = false;        // another process holds the USB interface (last report)
  Transport last_failed_ = Transport::None;  // the transport of the last failed attempt
  uint64_t session_id_ = 0;
  bool idle_logged_ = false;  // when the last session was lost (for recovery latency)
  bool reconnect_requested_ = false;  // `reconnect` taken on an open Wi-Fi session
  std::optional<CameraEndpoint> endpoint_;
};

}  // namespace crlab
