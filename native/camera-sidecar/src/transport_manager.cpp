#include <algorithm>
#include <climits>
#include <cmath>

#include "crlab/transport.hpp"

namespace crlab {

const char* to_string(Transport t) {
  switch (t) {
    case Transport::Usb: return "usb";
    case Transport::Wifi: return "wifi";
    default: return "none";
  }
}

const char* to_string(State s) {
  switch (s) {
    case State::Disconnected: return "DISCONNECTED";
    case State::Discovering: return "DISCOVERING";
    case State::ConnectingWifi: return "CONNECTING_WIFI";
    case State::ConnectedWifi: return "CONNECTED_WIFI";
    case State::ConnectingUsb: return "CONNECTING_USB";
    case State::ConnectedUsb: return "CONNECTED_USB";
    case State::Switching: return "SWITCHING";
    case State::Recovering: return "RECOVERING";
    case State::Error: return "ERROR";
  }
  return "?";
}

const char* to_string(BusyReason r) {
  switch (r) {
    case BusyReason::None: return "none";
    case BusyReason::Exposure: return "exposure";
    case BusyReason::Bracketing: return "bracketing";
    case BusyReason::PixelShift: return "pixel_shift";
    case BusyReason::FileTransfer: return "file_transfer";
    case BusyReason::Other: return "other";
  }
  return "?";
}

TransportManager::TransportManager(CameraAdapter& adapter, EventSink& log, TransportConfig cfg, ClockFn clock)
    : adapter_(adapter), log_(log), cfg_(cfg), clock_(std::move(clock)) {}

int64_t TransportManager::backoff_delay(const TransportConfig& cfg, int failures) {
  if (failures <= 0) return 0;
  double d = static_cast<double>(cfg.backoff_initial_ms) * std::pow(cfg.backoff_factor, failures - 1);
  return static_cast<int64_t>(std::min(d, static_cast<double>(cfg.backoff_max_ms)));
}

// ---------------------------------------------------------------- inputs

void TransportManager::on_usb_presence(bool present, const std::string& detail) {
  std::lock_guard<std::mutex> lk(mu_);
  inputs_.push_back({Input::UsbPresence, present, 0, 0, BusyReason::None, detail});
}

void TransportManager::on_sdk_disconnected(uint32_t sdk_error, uint64_t session_id) {
  std::lock_guard<std::mutex> lk(mu_);
  inputs_.push_back({Input::SdkDisconnected, false, sdk_error, session_id, BusyReason::None, ""});
}

void TransportManager::set_busy(BusyReason reason) {
  std::lock_guard<std::mutex> lk(mu_);
  inputs_.push_back({Input::Busy, false, 0, 0, reason, ""});
}

void TransportManager::request_stop() {
  std::lock_guard<std::mutex> lk(mu_);
  inputs_.push_back({Input::Stop, false, 0, 0, BusyReason::None, ""});
}

void TransportManager::usb_retry() {
  std::lock_guard<std::mutex> lk(mu_);
  inputs_.push_back({Input::UsbRetry, false, 0, 0, BusyReason::None, ""});
}

State TransportManager::state() const { return state_; }
Transport TransportManager::transport() const { return active_; }

// ---------------------------------------------------------------- helpers

void TransportManager::set_state(State s, const std::string& reason) {
  if (s == state_) return;
  log_.emit("transport.state", Fields()
                                   .str("from", to_string(state_))
                                   .str("to", to_string(s))
                                   .str("reason", reason)
                                   .str("transport", to_string(active_)));
  state_ = s;
}

Transport TransportManager::desired_transport() const {
  int64_t now = clock_();
  bool usb_ok = cfg_.usb_enabled && usb_present_ && now >= usb_retry_at_;
  if (usb_ok) return Transport::Usb;
  if (cfg_.wifi_enabled) return Transport::Wifi;
  return Transport::None;
}

void TransportManager::drain_inputs() {
  std::deque<Input> batch;
  {
    std::lock_guard<std::mutex> lk(mu_);
    batch.swap(inputs_);
  }
  for (const auto& in : batch) {
    int64_t now = clock_();
    switch (in.kind) {
      case Input::UsbPresence:
        if (in.flag == usb_present_) {
          usb_from_discovery_ = false;  // IOKit now confirms what discovery inferred
          break;
        }
        usb_present_ = in.flag;
        usb_from_discovery_ = false;
        usb_changed_at_ = now;
        log_.emit(in.flag ? "usb.arrived" : "usb.removed", Fields().str("detail", in.detail));
        if (in.flag) {
          // A fresh physical connection deserves a fresh chance.
          usb_failures_ = 0;
          usb_retry_at_ = 0;
          if (!session_open_) {
            failures_ = 0;
            next_attempt_ms_ = now;
          }
        }
        if (state_ == State::Error) {
          set_state(State::Disconnected, "usb presence changed; leaving ERROR");
          failures_ = 0;
          next_attempt_ms_ = now;
        }
        break;
      case Input::SdkDisconnected:
        if (!session_open_ || (in.session != 0 && in.session != session_id_)) {
          log_.emit("sdk.disconnect.ignored", Fields()
                                                  .hex("sdk_error", in.code)
                                                  .unum("callback_session", in.session)
                                                  .unum("current_session", session_id_)
                                                  .boolean("session_open", session_open_));
          break;
        }
        session_lost("sdk reported disconnection", in.code);
        break;
      case Input::Busy:
        if (in.busy != busy_) {
          busy_ = in.busy;
          busy_logged_at_ = -1;
          log_.emit("camera.busy", Fields().str("reason", to_string(busy_)).boolean("safe_point", busy_ == BusyReason::None));
        }
        break;
      case Input::Stop:
        if (!stop_) {
          stop_ = true;
          stop_requested_at_ = now;
        }
        break;
      case Input::UsbRetry:
        // Same fresh chance a physical re-plug gives (see UsbPresence above).
        usb_failures_ = 0;
        usb_retry_at_ = 0;
        failures_ = 0;
        next_attempt_ms_ = now;
        log_.emit("usb.retry_requested", Fields());
        break;
    }
  }
}

void TransportManager::schedule_retry(Transport t) {
  int64_t now = clock_();
  ++failures_;
  if (t == Transport::Usb) {
    ++usb_failures_;
    if (usb_failures_ >= cfg_.usb_failures_before_wifi_fallback && cfg_.wifi_enabled) {
      // Do not periodically retry USB: each retry would tear down a working
      // Wi-Fi session to probe a USB link that keeps failing (e.g. camera in
      // Mass Storage mode). USB gets a fresh chance on the next physical
      // re-plug (see drain_inputs).
      usb_retry_at_ = INT64_MAX;
      log_.emit("usb.fallback_to_wifi", Fields()
                                            .num("usb_failures", usb_failures_)
                                            .str("usb_retry", "on next USB re-plug"));
      failures_ = 0;
      next_attempt_ms_ = now;
      return;
    }
  }
  int64_t delay = backoff_delay(cfg_, failures_);
  next_attempt_ms_ = now + delay;
  log_.emit("connect.retry_scheduled",
            Fields().str("transport", to_string(t)).num("failures", failures_).num("delay_ms", delay));
}

void TransportManager::session_lost(const std::string& reason, uint32_t code) {
  int64_t now = clock_();
  Transport was = active_;
  Fields f;
  f.str("transport", to_string(was)).str("reason", reason).hex("sdk_error", code).str("busy", to_string(busy_));
  log_.emit("session.lost", f);
  if (busy_ != BusyReason::None) {
    // We cannot know whether the exposure/sequence/transfer completed.
    log_.emit("capture.uncertain", Fields()
                                       .str("transport", to_string(was))
                                       .str("operation", to_string(busy_))
                                       .str("reason", reason));
  }
  adapter_.disconnect();  // release invalid resources
  session_open_ = false;
  active_ = Transport::None;
  busy_ = BusyReason::None;
  if (was == Transport::Usb && usb_from_discovery_) {
    // No IOKit signal to tell us the cable is gone: assume it is and let discovery re-detect USB.
    usb_present_ = false;
    usb_from_discovery_ = false;
    usb_changed_at_ = now;
  }
  lost_at_ = now;
  failures_ = 0;
  next_attempt_ms_ = now + cfg_.after_disconnect_delay_ms;
  set_state(State::Recovering, reason);
}

void TransportManager::intentional_disconnect(const std::string& reason) {
  int64_t t0 = clock_();
  Transport was = active_;
  adapter_.disconnect();
  session_open_ = false;
  active_ = Transport::None;
  log_.emit("session.closed", Fields()
                                  .str("transport", to_string(was))
                                  .str("reason", reason)
                                  .num("duration_ms", clock_() - t0));
}

bool TransportManager::try_connect(Transport t) {
  if (session_open_) {  // structural guard: never two sessions
    log_.emit("connect.skipped", Fields().str("reason", "session already open"));
    return false;
  }
  bool recovering = (state_ == State::Recovering);
  set_state(State::Discovering, std::string("looking for camera over ") + to_string(t));
  int64_t t0 = clock_();
  auto eps = adapter_.discover(t);
  int64_t t1 = clock_();
  log_.emit("discovery.result", Fields()
                                    .str("transport", to_string(t))
                                    .num("count", static_cast<int64_t>(eps.size()))
                                    .num("duration_ms", t1 - t0)
                                    .boolean("success", !eps.empty()));
  if (eps.empty() && t == Transport::Wifi && cfg_.usb_enabled && !usb_present_ &&
      usb_retry_at_ != INT64_MAX && adapter_.last_discovery_saw(Transport::Usb)) {
    // Observed on the A7R V (Exp 05, 2026-11-05): plugging USB makes the camera drop its Wi-Fi
    // remote session by itself and move its endpoint to USB. If IOKit missed the arrival, the
    // camera would otherwise sit on USB while we keep looking on Wi-Fi. Discovery is evidence too.
    usb_present_ = true;
    usb_from_discovery_ = true;
    usb_changed_at_ = clock_() - cfg_.usb_settle_ms;  // already enumerated by the SDK: no settle
    failures_ = 0;
    next_attempt_ms_ = clock_();
    log_.emit("usb.detected_by_discovery",
              Fields().str("note", "camera listed on USB by SDK discovery; IOKit reported no arrival"));
    set_state(recovering ? State::Recovering : State::Disconnected, "camera found on usb");
    return false;
  }
  if (eps.empty()) {
    schedule_retry(t);
    set_state(recovering ? State::Recovering : State::Disconnected, "camera not found");
    return false;
  }
  // Prefer the camera we were connected to before.
  CameraEndpoint ep = eps.front();
  if (endpoint_) {
    for (const auto& e : eps)
      if (e.id == endpoint_->id) ep = e;
  }

  set_state(t == Transport::Usb ? State::ConnectingUsb : State::ConnectingWifi, "camera found");
  ConnectResult r = adapter_.connect(ep);
  int64_t t2 = clock_();
  Fields cf;
  cf.str("transport", to_string(t))
      .str("camera_model", ep.model)
      .str("camera_id", ep.id)
      .num("duration_ms", t2 - t1)
      .boolean("success", r.ok)
      .hex("sdk_error", r.sdk_error);
  if (!r.message.empty()) cf.str("message", r.message);
  log_.emit("connect.result", cf);

  if (!r.ok) {
    if (r.fatal) {
      set_state(State::Error, "fatal connection error: " + r.message);
      return false;
    }
    schedule_retry(t);
    set_state(recovering ? State::Recovering : State::Disconnected, "connect failed");
    return false;
  }

  session_open_ = true;
  session_id_ = r.session_id;
  active_ = t;
  endpoint_ = ep;
  failures_ = 0;
  if (t == Transport::Usb) usb_failures_ = 0;
  connected_at_ = t2;
  set_state(t == Transport::Usb ? State::ConnectedUsb : State::ConnectedWifi, "connected");

  Fields st;
  bool ok = adapter_.read_state(st);
  st.boolean("success", ok);
  log_.emit("camera.state", st);

  Fields ready;
  ready.str("transport", to_string(t)).num("connect_ms", t2 - t1).num("total_ms", clock_() - t0);
  if (lost_at_ >= 0) ready.num("recovery_ms", clock_() - lost_at_);
  ready.str("model", ep.model);
  if (!r.store_destination.empty()) ready.str("store_destination", r.store_destination);
  if (!r.store_value.empty()) ready.str("store_value", r.store_value);
  log_.emit("connection.ready", ready);
  lost_at_ = -1;
  idle_logged_ = false;
  return true;
}

// ---------------------------------------------------------------- driver

bool TransportManager::step() {
  drain_inputs();
  int64_t now = clock_();

  if (stop_) {
    // A voluntary stop waits for the safe point (busy None), at most stop_wait_ms measured from
    // the step that drained the Stop input. While it waits nothing else starts (no connect, no
    // switch). A session lost meanwhile ends the wait at once.
    bool deferred = session_open_ && busy_ != BusyReason::None && now - stop_requested_at_ < cfg_.stop_wait_ms;
    if (deferred && active_ == Transport::Usb && !usb_present_) {
      session_lost("usb device removed", 0);
      deferred = false;
    }
    if (deferred) return true;
    if (session_open_) {
      if (active_ == Transport::Usb) adapter_.before_voluntary_stop(Transport::Usb);
      intentional_disconnect("stop requested");
    }
    set_state(State::Disconnected, "stopped");
    return false;
  }

  switch (state_) {
    case State::ConnectedUsb:
    case State::ConnectedWifi: {
      Transport want = desired_transport();
      if (want == active_) return true;

      if (active_ == Transport::Usb && !usb_present_) {
        // Cable is gone: the USB session is invalid even if the SDK has not
        // reported it yet. Not an intentional switch -> recovery path.
        session_lost("usb device removed", 0);
        return true;
      }
      if (active_ == Transport::Wifi && want == Transport::Usb) {
        if (now - usb_changed_at_ < cfg_.usb_settle_ms) return true;  // let enumeration settle
        if (now - connected_at_ < cfg_.min_dwell_ms) return true;      // avoid flapping
        if (busy_ != BusyReason::None) {
          if (busy_logged_at_ < 0 || now - busy_logged_at_ >= cfg_.busy_log_interval_ms) {
            log_.emit("switch.deferred", Fields().str("to", "usb").str("busy", to_string(busy_)));
            busy_logged_at_ = now;
          }
          return true;
        }
        set_state(State::Switching, "usb available; preferring usb");
        intentional_disconnect("switching wifi -> usb");
        next_attempt_ms_ = now + cfg_.after_disconnect_delay_ms;
        return true;
      }
      return true;
    }

    case State::Disconnected:
    case State::Recovering:
    case State::Switching: {
      if (now < next_attempt_ms_) return true;
      Transport want = desired_transport();
      if (want == Transport::None) {
        if (!idle_logged_) {
          log_.emit("transport.idle", Fields().str("reason", "no enabled transport available"));
          idle_logged_ = true;
        }
        return true;
      }
      if (want == Transport::Usb && now - usb_changed_at_ < cfg_.usb_settle_ms) return true;
      try_connect(want);
      return true;
    }

    case State::Error:
      return true;  // waits for an external change (USB presence) or restart

    default:
      return true;  // Discovering / Connecting* are transient inside try_connect()
  }
}

}  // namespace crlab
