// Transport manager tests against a scripted mock camera.
//
// These verify OUR state machine only. They say nothing about how the Sony
// SDK or the A7R V behave; that is what experiments 03/05/06 are for.
#include <map>
#include <string>
#include <vector>

#include "crlab/transport.hpp"
#include "mini_test.hpp"

using namespace crlab;
void fingerprint_c1a();
void fingerprint_c1b();
void fingerprint_c1c();
void fingerprint_c1d();
void fingerprint_c2a();

namespace {

struct CaptureSink : EventSink {
  struct Ev {
    std::string name;
    std::string json;
  };
  std::vector<Ev> events;
  void emit(const std::string& e, const Fields& f) override { events.push_back({e, f.encode_object()}); }
  int count(const std::string& e) const {
    int n = 0;
    for (auto& x : events) n += (x.name == e);
    return n;
  }
  bool has_field(const std::string& e, const std::string& frag) const {
    for (auto& x : events)
      if (x.name == e && x.json.find(frag) != std::string::npos) return true;
    return false;
  }
};

struct FakeClock {
  int64_t now = 0;
  void advance(int64_t ms) { now += ms; }
};

struct MockCamera : CameraAdapter {
  FakeClock& clock;
  bool usb_visible = false;
  bool wifi_visible = true;
  int fail_usb_connects = 0;   // next N USB connects fail
  int fail_wifi_connects = 0;
  bool fatal_wifi = false;
  int64_t discover_cost_ms = 100;
  int64_t connect_cost_ms = 500;
  int active_sessions = 0;
  int max_sessions = 0;
  uint64_t next_session = 1;
  uint64_t current_session = 0;
  std::vector<std::string> calls;

  explicit MockCamera(FakeClock& c) : clock(c) {}

  std::vector<CameraEndpoint> discover(Transport t) override {
    calls.push_back(std::string("discover:") + to_string(t));
    clock.advance(discover_cost_ms);
    last_saw_usb = usb_visible;
    if (t == Transport::Usb && usb_visible) return {{"SN123", "ILCE-7RM5", Transport::Usb, "usb:1"}};
    if (t == Transport::Wifi && wifi_visible) return {{"SN123", "ILCE-7RM5", Transport::Wifi, "192.168.1.50"}};
    return {};
  }
  ConnectResult connect(const CameraEndpoint& ep) override {
    calls.push_back(std::string("connect:") + to_string(ep.transport));
    clock.advance(connect_cost_ms);
    ConnectResult r;
    if (ep.transport == Transport::Usb && fail_usb_connects > 0) {
      --fail_usb_connects;
      r.sdk_error = 0x8203;  // illustrative code, not a claim about real SDK values
      r.message = "mock usb failure";
      return r;
    }
    if (ep.transport == Transport::Wifi && (fail_wifi_connects > 0 || fatal_wifi)) {
      if (fail_wifi_connects > 0) --fail_wifi_connects;
      r.fatal = fatal_wifi;
      r.message = fatal_wifi ? "mock auth rejected" : "mock wifi failure";
      return r;
    }
    ++active_sessions;
    max_sessions = std::max(max_sessions, active_sessions);
    current_session = next_session++;
    r.ok = true;
    r.session_id = current_session;
    return r;
  }
  void disconnect() override {
    calls.push_back("disconnect");
    if (active_sessions > 0) --active_sessions;
  }
  bool read_state(Fields& out) override {
    out.str("battery", "80%");
    return true;
  }
  bool last_saw_usb = false;
  bool last_discovery_saw(Transport t) const override { return t == Transport::Usb && last_saw_usb; }
};

struct Rig {
  FakeClock clock;
  CaptureSink log;
  MockCamera cam{clock};
  TransportConfig cfg;
  std::unique_ptr<TransportManager> tm;
  Rig() {
    cfg.usb_settle_ms = 1000;
    cfg.min_dwell_ms = 2000;
    cfg.after_disconnect_delay_ms = 500;
    cfg.backoff_initial_ms = 1000;
    cfg.backoff_max_ms = 8000;
    tm = std::make_unique<TransportManager>(cam, log, cfg, [this] { return clock.now; });
  }
  // Run the loop for `ms` of simulated time in 100 ms ticks.
  void run(int64_t ms) {
    int64_t end = clock.now + ms;
    while (clock.now < end) {
      tm->step();
      clock.advance(100);
    }
  }
  void plug_usb() {
    cam.usb_visible = true;
    tm->on_usb_presence(true, "test");
  }
  void unplug_usb() {
    cam.usb_visible = false;
    tm->on_usb_presence(false, "test");
  }
};

}  // namespace

TEST_CASE("starts on wifi when no usb present") {
  Rig r;
  r.run(2000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK_EQ(r.log.count("connection.ready"), 1);
  CHECK_EQ(r.cam.max_sessions, 1);
}

TEST_CASE("prefers usb at startup when present") {
  Rig r;
  r.plug_usb();
  r.run(3000);
  CHECK(r.tm->state() == State::ConnectedUsb);
  CHECK_EQ(r.cam.calls.front(), std::string("discover:usb"));
}

TEST_CASE("usb insertion switches wifi -> usb via controlled disconnect") {
  Rig r;
  r.run(3000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.plug_usb();
  r.run(4000);
  CHECK(r.tm->state() == State::ConnectedUsb);
  CHECK(r.log.has_field("session.closed", "\"reason\":\"switching wifi -> usb\""));
  CHECK(r.log.has_field("transport.state", "\"to\":\"SWITCHING\""));
  CHECK_EQ(r.cam.max_sessions, 1);  // never two sessions at once
  CHECK_EQ(r.cam.active_sessions, 1);
}

TEST_CASE("usb removal falls back to wifi and reports recovery latency") {
  Rig r;
  r.plug_usb();
  r.run(3000);
  CHECK(r.tm->state() == State::ConnectedUsb);
  r.unplug_usb();
  r.run(3000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK(r.log.has_field("session.lost", "usb device removed"));
  CHECK(r.log.has_field("connection.ready", "recovery_ms"));
  CHECK_EQ(r.cam.active_sessions, 1);
}

TEST_CASE("switch is deferred while camera is busy and resumes at safe point") {
  Rig r;
  r.run(3000);
  r.tm->set_busy(BusyReason::PixelShift);
  r.plug_usb();
  r.run(10000);
  CHECK(r.tm->state() == State::ConnectedWifi);  // still on wifi: never interrupt pixel shift
  CHECK(r.log.count("switch.deferred") >= 1);
  CHECK(r.log.count("switch.deferred") <= 3);  // rate limited
  r.tm->set_busy(BusyReason::None);
  r.run(2000);
  CHECK(r.tm->state() == State::ConnectedUsb);
}

TEST_CASE("unexpected disconnect during capture is reported as uncertain") {
  Rig r;
  r.run(3000);
  r.tm->set_busy(BusyReason::Bracketing);
  r.run(200);
  r.tm->on_sdk_disconnected(0x8208, r.cam.current_session);
  r.run(3000);
  CHECK_EQ(r.log.count("capture.uncertain"), 1);
  CHECK(r.log.has_field("capture.uncertain", "bracketing"));
  CHECK(r.tm->state() == State::ConnectedWifi);  // recovered
}

TEST_CASE("stale disconnect callback from old session is ignored") {
  Rig r;
  r.run(3000);
  uint64_t old_session = r.cam.current_session;
  r.plug_usb();
  r.run(4000);
  CHECK(r.tm->state() == State::ConnectedUsb);
  // The SDK delivers OnDisconnected for the intentionally closed Wi-Fi session late.
  r.tm->on_sdk_disconnected(0, old_session);
  r.run(1000);
  CHECK(r.tm->state() == State::ConnectedUsb);
  CHECK_EQ(r.log.count("sdk.disconnect.ignored"), 1);
}

TEST_CASE("wifi connect failures use bounded exponential backoff") {
  Rig r;
  r.cam.wifi_visible = false;
  r.run(60000);
  // delays: 1s,2s,4s,8s,8s,... -> bounded at backoff_max
  CHECK_EQ(TransportManager::backoff_delay(r.cfg, 1), int64_t(1000));
  CHECK_EQ(TransportManager::backoff_delay(r.cfg, 3), int64_t(4000));
  CHECK_EQ(TransportManager::backoff_delay(r.cfg, 10), int64_t(8000));
  int discovers = 0;
  for (auto& c : r.cam.calls) discovers += (c == "discover:wifi");
  CHECK(discovers >= 6);
  CHECK(discovers <= 12);  // no tight reconnect loop
  CHECK(r.log.has_field("connect.retry_scheduled", "\"delay_ms\":8000"));
  CHECK(r.tm->state() == State::Disconnected);
  r.cam.wifi_visible = true;
  r.run(10000);
  CHECK(r.tm->state() == State::ConnectedWifi);
}

TEST_CASE("repeated usb connect failure falls back to wifi") {
  Rig r;
  r.cam.fail_usb_connects = 100;  // e.g. camera in Mass Storage mode
  r.plug_usb();
  r.run(15000);
  CHECK(r.log.count("usb.fallback_to_wifi") == 1);
  CHECK(r.tm->state() == State::ConnectedWifi);
  // not flapping back to usb immediately
  int usb_connects = 0;
  for (auto& c : r.cam.calls) usb_connects += (c == "connect:usb");
  CHECK_EQ(usb_connects, r.cfg.usb_failures_before_wifi_fallback);
}

TEST_CASE("replugging usb after fallback gives usb a fresh chance") {
  Rig r;
  r.cam.fail_usb_connects = 3;
  r.plug_usb();
  r.run(15000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.unplug_usb();
  r.run(500);
  r.plug_usb();
  r.run(6000);
  CHECK(r.tm->state() == State::ConnectedUsb);
}

TEST_CASE("fatal error enters ERROR and waits for external change") {
  Rig r;
  r.cam.fatal_wifi = true;
  r.run(20000);
  CHECK(r.tm->state() == State::Error);
  int connects = 0;
  for (auto& c : r.cam.calls) connects += (c == "connect:wifi");
  CHECK_EQ(connects, 1);  // no retry storm on an auth error
  r.plug_usb();
  r.run(4000);
  CHECK(r.tm->state() == State::ConnectedUsb);
}

TEST_CASE("usb flapping within settle window does not trigger switching") {
  Rig r;
  r.run(3000);
  for (int i = 0; i < 5; ++i) {
    r.plug_usb();
    r.run(200);
    r.unplug_usb();
    r.run(200);
  }
  r.run(2000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK_EQ(r.log.count("session.closed"), 0);
  CHECK_EQ(r.cam.max_sessions, 1);
}

TEST_CASE("delayed operations: slow connect still yields single session") {
  Rig r;
  r.cam.connect_cost_ms = 7000;  // connect blocks for 7 s
  r.run(3000);
  r.plug_usb();  // arrives while (simulated) blocked
  r.run(20000);
  CHECK(r.tm->state() == State::ConnectedUsb);
  CHECK_EQ(r.cam.max_sessions, 1);
}

TEST_CASE("stop closes the session") {
  Rig r;
  r.run(3000);
  r.tm->request_stop();
  CHECK(!r.tm->step());
  CHECK_EQ(r.cam.active_sessions, 0);
  CHECK(r.tm->state() == State::Disconnected);
}

TEST_CASE("camera found on usb by discovery switches to usb even without an IOKit event") {
  // Observed on hardware (Exp 05): plugging USB drops the Wi-Fi session from the camera side and
  // the camera moves to USB; IOKit reported nothing. The manager must follow discovery.
  Rig r;
  r.run(2000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.cam.usb_visible = true;    // cable in, but no on_usb_presence() call
  r.cam.wifi_visible = false;  // camera no longer serves Wi-Fi remote
  r.tm->on_sdk_disconnected(0x8207, r.cam.current_session);
  r.run(3000);
  CHECK(r.tm->state() == State::ConnectedUsb);
  CHECK_EQ(r.log.count("usb.detected_by_discovery"), 1);
  CHECK_EQ(r.cam.max_sessions, 1);
  // Cable out: the USB session drops; with no IOKit signal the manager must go back to Wi-Fi.
  r.cam.usb_visible = false;
  r.cam.wifi_visible = true;
  r.tm->on_sdk_disconnected(0x8207, r.cam.current_session);
  r.run(3000);
  CHECK(r.tm->state() == State::ConnectedWifi);
}

TEST_CASE("json log escapes and encodes fields") {
  Fields f;
  f.str("a", "q\"uote\n").num("n", -5).boolean("b", true).hex("e", 0x8203).dbl("d", 1.5);
  CHECK_EQ(f.encode_object(), std::string("{\"a\":\"q\\\"uote\\n\",\"n\":-5,\"b\":true,\"e\":\"0x00008203\",\"d\":1.500}"));
}

TEST_CASE("json escape keeps output valid for binary bytes") {
  std::string bin = "M";
  bin += static_cast<char>(0xD9);
  bin += static_cast<char>(0x02);
  CHECK_EQ(json_escape(bin), std::string("M\\u00d9\\u0002"));
  CHECK_EQ(json_escape("caf\xc3\xa9"), std::string("caf\xc3\xa9"));  // valid UTF-8 passes through
}

// ---- Session Manager additions (P45) ------------------------------------------------------------

namespace {
int count_calls(const MockCamera& cam, const std::string& call) {
  int n = 0;
  for (auto& c : cam.calls) n += (c == call);
  return n;
}
}  // namespace

TEST_CASE("usb-retry gives usb a fresh chance without a re-plug") {
  Rig r;
  r.cam.fail_usb_connects = 3;
  r.plug_usb();
  r.run(15000);
  CHECK_EQ(r.log.count("usb.fallback_to_wifi"), 1);
  CHECK_EQ(count_calls(r.cam, "connect:usb"), 3);
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.tm->usb_retry();
  r.run(5000);
  CHECK_EQ(r.log.count("usb.retry_requested"), 1);
  CHECK_EQ(count_calls(r.cam, "connect:usb"), 4);
  CHECK(r.tm->state() == State::ConnectedUsb);
  CHECK_EQ(r.cam.active_sessions, 1);

  // Control: without the retry, USB is never probed again (no periodic probing).
  Rig control;
  control.cam.fail_usb_connects = 3;
  control.plug_usb();
  control.run(15000);
  CHECK(control.tm->state() == State::ConnectedWifi);
  control.run(60000);
  CHECK_EQ(count_calls(control.cam, "connect:usb"), 3);
  CHECK(control.tm->state() == State::ConnectedWifi);
}

TEST_CASE("stop is deferred while busy, bounded by 10 s") {
  // Busy cleared before the bound: the stop happens at the safe point.
  Rig r;
  r.plug_usb();
  r.run(3000);
  CHECK(r.tm->state() == State::ConnectedUsb);
  r.tm->set_busy(BusyReason::FileTransfer);
  r.tm->request_stop();
  r.run(3000);
  CHECK(r.tm->state() == State::ConnectedUsb);
  CHECK_EQ(r.log.count("session.closed"), 0);
  CHECK_EQ(r.cam.active_sessions, 1);
  r.tm->set_busy(BusyReason::None);
  r.run(500);
  CHECK_EQ(r.log.count("session.closed"), 1);
  CHECK_EQ(r.cam.active_sessions, 0);
  CHECK(r.tm->state() == State::Disconnected);

  // Busy never cleared: the stop happens 10 s after the step that drained it (>=).
  Rig b;
  b.plug_usb();
  b.run(3000);
  CHECK(b.tm->state() == State::ConnectedUsb);
  b.tm->set_busy(BusyReason::FileTransfer);
  b.tm->request_stop();
  b.run(9900);
  CHECK(b.tm->state() == State::ConnectedUsb);
  CHECK_EQ(b.log.count("session.closed"), 0);
  b.run(600);
  CHECK_EQ(b.log.count("session.closed"), 1);
  CHECK_EQ(b.cam.active_sessions, 0);
  CHECK(b.tm->state() == State::Disconnected);
}

TEST_CASE("credentials percent-decoding reverses the app's encoding") {
  // Fingerprint command and policy use the same display/percent wire boundary.
  fingerprint_c1a();
  fingerprint_c1b();
  fingerprint_c1c();
  fingerprint_c1d();
  fingerprint_c2a();
  CHECK_EQ(percent_decode("u%20name"), std::string("u name"));
  CHECK_EQ(percent_decode("p%2Fw%3D%26%C3%A9"), std::string("p/w=&\xc3\xa9"));
  CHECK_EQ(percent_decode("abc%"), std::string("abc%"));
  CHECK_EQ(percent_decode("abc%4"), std::string("abc%4"));
  CHECK_EQ(percent_decode("abc%G1"), std::string("abc%G1"));
}

// ---- Session Manager additions (P46) ------------------------------------------------------------

TEST_CASE("reconnect reopens the wifi session once") {
  Rig r;
  r.run(2000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK_EQ(r.log.count("connection.ready"), 1);
  int wifi_connects = count_calls(r.cam, "connect:wifi");
  r.tm->reconnect();
  r.run(3000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK_EQ(r.log.count("session.closed"), 1);
  CHECK(r.log.has_field("session.closed", "\"reason\":\"reconnect requested\""));
  int wifi_ready = 0;
  for (auto& e : r.log.events)
    wifi_ready += (e.name == "connection.ready" && e.json.find("\"transport\":\"wifi\"") != std::string::npos);
  CHECK_EQ(r.log.count("connection.ready"), 2);
  CHECK_EQ(wifi_ready, 2);
  CHECK_EQ(count_calls(r.cam, "connect:wifi"), wifi_connects + 1);
  CHECK_EQ(count_calls(r.cam, "connect:usb"), 0);
  CHECK_EQ(count_calls(r.cam, "discover:usb"), 0);
  CHECK_EQ(r.cam.max_sessions, 1);
  CHECK_EQ(r.cam.active_sessions, 1);
}

int main() { return mt_run_all(); }
