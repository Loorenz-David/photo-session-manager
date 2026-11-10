// Transport manager tests against a scripted camera, plus a stub-backed adapter check.
//
// These verify OUR state machine only. They say nothing about how the Sony
// SDK or the A7R V behave; that is what experiments 03/05/06 are for.
#include <map>
#include <string>
#include <vector>

#include "crlab/transport.hpp"
#include "crlab/camera.hpp"
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
  bool usb_endpoint_available = true;
  bool wifi_visible = true;
  bool cached_wifi = false;
  bool cached_wifi_fails = false;
  // P63: the camera's Wi-Fi remote answers only from this clock time (after an unplug it comes
  // back a few seconds later); a direct connect before it fails after direct_fail_cost_ms.
  int64_t wifi_ready_at = 0;
  int64_t direct_fail_cost_ms = 300;
  bool learn_endpoint = false;  // a Wi-Fi connect through discovery caches the endpoint (as the SDK adapter)
  bool cached_wifi_stale = false;  // the cached address is no longer the camera's
  int fail_usb_connects = 0;   // next N USB connects fail
  bool usb_held = false;       // failing USB connects report the interface held by another process
  int64_t usb_fail_cost_ms = -1;  // cost of a failing USB connect (-1: connect_cost_ms)
  int fail_wifi_connects = 0;
  bool fatal_wifi = false;
  int64_t discover_cost_ms = 100;
  int64_t connect_cost_ms = 500;
  int active_sessions = 0;
  int max_sessions = 0;
  uint64_t next_session = 1;
  uint64_t current_session = 0;
  std::vector<std::string> calls;
  std::vector<std::pair<std::string, int64_t>> timed;  // every call with the clock when it started

  explicit MockCamera(FakeClock& c) : clock(c) {}

  std::vector<CameraEndpoint> discover(Transport t) override {
    if (t == Transport::Wifi && cached_wifi) {
      calls.push_back("cached:wifi");
      timed.push_back({calls.back(), clock.now});
      return {{"SN123", "ILCE-7RM5", Transport::Wifi, "direct-ip:192.168.1.50"}};
    }
    calls.push_back(std::string("discover:") + to_string(t));
    timed.push_back({calls.back(), clock.now});
    clock.advance(discover_cost_ms);
    last_saw_usb = usb_visible;
    if (t == Transport::Usb && usb_visible && usb_endpoint_available) return {{"SN123", "ILCE-7RM5", Transport::Usb, "usb:1"}};
    if (t == Transport::Wifi && wifi_visible && clock.now - discover_cost_ms >= wifi_ready_at)
      return {{"SN123", "ILCE-7RM5", Transport::Wifi, "192.168.1.50"}};
    return {};
  }
  ConnectResult connect(const CameraEndpoint& ep) override {
    calls.push_back(std::string("connect:") + to_string(ep.transport));
    timed.push_back({calls.back(), clock.now});
    ConnectResult r;
    if (ep.address.rfind("direct-ip:", 0) == 0 && cached_wifi_fails) {
      r.fatal = true;
      r.message = "cached endpoint failed";
      return r;
    }
    if (ep.address.rfind("direct-ip:", 0) == 0 && (cached_wifi_stale || clock.now < wifi_ready_at)) {
      clock.advance(direct_fail_cost_ms);
      r.message = "camera not answering at the cached address";
      return r;
    }
    if (ep.transport == Transport::Usb && fail_usb_connects > 0) {
      clock.advance(usb_fail_cost_ms >= 0 ? usb_fail_cost_ms : connect_cost_ms);
      --fail_usb_connects;
      r.sdk_error = 0x8203;  // illustrative code, not a claim about real SDK values
      r.message = "mock usb failure";
      r.held = usb_held;
      return r;
    }
    clock.advance(connect_cost_ms);
    if (ep.transport == Transport::Wifi && (fail_wifi_connects > 0 || fatal_wifi)) {
      if (fail_wifi_connects > 0) --fail_wifi_connects;
      r.fatal = fatal_wifi;
      r.message = fatal_wifi ? "mock auth rejected" : "mock wifi failure";
      return r;
    }
    if (learn_endpoint && ep.transport == Transport::Wifi) cached_wifi = true;
    ++active_sessions;
    max_sessions = std::max(max_sessions, active_sessions);
    current_session = next_session++;
    r.ok = true;
    r.session_id = current_session;
    return r;
  }
  void disconnect() override {
    calls.push_back("disconnect");
    timed.push_back({calls.back(), clock.now});
    if (active_sessions > 0) --active_sessions;
  }
  bool read_state(Fields& out) override {
    out.str("battery", "80%");
    return true;
  }
  bool last_saw_usb = false;
  void forget_cached_endpoint() override { cached_wifi = false; calls.push_back("forget:cached"); }
  bool last_discovery_saw(Transport t) const override { return t == Transport::Usb && last_saw_usb; }
};

struct Rig {
  FakeClock clock;
  CaptureSink log;
  MockCamera cam{clock};
  TransportConfig cfg;
  std::unique_ptr<TransportManager> tm;
  // production = the sidecar's `auto` timings (TransportConfig defaults, 2.5 s claim window).
  explicit Rig(bool production = false) {
    if (production) {
      cfg.usb_settle_ms = 2500;
    } else {
      cfg.usb_settle_ms = 1000;
      cfg.min_dwell_ms = 2000;
      cfg.after_disconnect_delay_ms = 500;
      cfg.backoff_initial_ms = 1000;
      cfg.backoff_max_ms = 8000;
    }
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
  // not flapping back to usb afterwards
  int usb_connects = 0;
  for (auto& c : r.cam.calls) usb_connects += (c == "connect:usb");
  r.run(60000);
  int later = 0;
  for (auto& c : r.cam.calls) later += (c == "connect:usb");
  CHECK_EQ(later, usb_connects);
  CHECK(r.tm->state() == State::ConnectedWifi);
}

TEST_CASE("P54 C3(a) busy USB claims fall back to Wi-Fi then return at a safe point") {
  Rig r;
  r.cam.fail_usb_connects = 100;  // holder rejects every connect
  r.plug_usb();
  r.run(10000);
  CHECK_EQ(r.log.count("usb.fallback_to_wifi"), 1);
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.unplug_usb();
  r.run(500);
  r.cam.fail_usb_connects = 0;  // holder quit
  r.plug_usb();
  r.tm->set_busy(BusyReason::FileTransfer);
  r.run(6000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.tm->set_busy(BusyReason::None);
  r.run(6000);
  CHECK(r.tm->state() == State::ConnectedUsb);

  // A claimed interface can also disappear from SDK enumeration entirely.
  Rig hidden;
  hidden.cam.usb_endpoint_available = false;
  hidden.plug_usb();
  hidden.run(10000);
  CHECK_EQ(hidden.log.count("usb.fallback_to_wifi"), 1);
  CHECK(hidden.tm->state() == State::ConnectedWifi);
  hidden.unplug_usb();
  hidden.run(500);
  hidden.cam.usb_endpoint_available = true;
  hidden.plug_usb();
  hidden.run(6000);
  CHECK(hidden.tm->state() == State::ConnectedUsb);

  // The general retry backoff may be longer; USB still reaches Wi-Fi within ten seconds.
  Rig slow;
  slow.cfg.backoff_initial_ms = 10000;
  slow.cfg.backoff_max_ms = 30000;
  slow.tm = std::make_unique<TransportManager>(slow.cam, slow.log, slow.cfg, [&slow] { return slow.clock.now; });
  slow.cam.fail_usb_connects = 100;
  slow.plug_usb();
  slow.run(10000);
  CHECK(slow.tm->state() == State::ConnectedWifi);
  CHECK_EQ(slow.log.count("usb.fallback_to_wifi"), 1);
}

TEST_CASE("replugging usb after fallback gives usb a fresh chance") {
  Rig r;
  r.cam.fail_usb_connects = 100;
  r.plug_usb();
  r.run(15000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK_EQ(r.log.count("usb.fallback_to_wifi"), 1);
  r.cam.fail_usb_connects = 0;
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
  // P56 ruling 1: every plug-in claims the cable at once (no settle wait), so Wi-Fi may be closed
  // while the cable flaps; the manager still ends on Wi-Fi and never holds two sessions.
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK_EQ(r.cam.max_sessions, 1);
  CHECK_EQ(r.cam.active_sessions, 1);
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
  r.cam.fail_usb_connects = 100;
  r.plug_usb();
  r.run(15000);
  CHECK_EQ(r.log.count("usb.fallback_to_wifi"), 1);
  int before = count_calls(r.cam, "connect:usb");
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.cam.fail_usb_connects = 0;
  r.tm->usb_retry();
  r.run(5000);
  CHECK_EQ(r.log.count("usb.retry_requested"), 1);
  CHECK_EQ(count_calls(r.cam, "connect:usb"), before + 1);
  CHECK(r.tm->state() == State::ConnectedUsb);
  CHECK_EQ(r.cam.active_sessions, 1);

  // Control: without the retry, USB is never probed again (no periodic probing).
  Rig control;
  control.cam.fail_usb_connects = 100;
  control.plug_usb();
  control.run(15000);
  CHECK(control.tm->state() == State::ConnectedWifi);
  int settled = count_calls(control.cam, "connect:usb");
  control.run(60000);
  CHECK_EQ(count_calls(control.cam, "connect:usb"), settled);
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

// ---- Session Manager additions (P56: claim at plug-in, never idle) -------------------------------

namespace {
// Start times of `call` at or after `from` (MockCamera::timed).
std::vector<int64_t> starts(const MockCamera& cam, const std::string& call, int64_t from) {
  std::vector<int64_t> out;
  for (auto& [name, at] : cam.timed)
    if (name == call && at >= from) out.push_back(at);
  return out;
}
int64_t first_at(const MockCamera& cam, const std::string& call, int64_t from) {
  auto v = starts(cam, call, from);
  return v.empty() ? -1 : v.front();
}
int count_wifi_ready(const CaptureSink& log) {
  int n = 0;
  for (auto& e : log.events)
    n += (e.name == "connection.ready" && e.json.find("\"transport\":\"wifi\"") != std::string::npos);
  return n;
}
}  // namespace

TEST_CASE("P56 C1(a) a plug-in closes Wi-Fi, claims USB at once, repeats through the window, then Wi-Fi at once") {
  Rig r(true);
  r.cam.usb_fail_cost_ms = 10;  // an interface that is not openable refuses within ~10 ms
  r.run(3000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.cam.fail_usb_connects = 1000;  // not openable for the whole window (not held by anyone)
  const int64_t t0 = r.clock.now;
  r.plug_usb();
  r.run(2500 + 400);
  // Wi-Fi closed and the first USB attempt within 300 ms of the event, in that order.
  int64_t closed = first_at(r.cam, "disconnect", t0);
  int64_t first = first_at(r.cam, "discover:usb", t0);
  CHECK(closed >= t0);
  CHECK(first >= closed);
  CHECK(first - t0 <= 300);
  CHECK(r.log.has_field("session.closed", "\"reason\":\"switching wifi -> usb\""));
  // Attempts repeat at <= 500 ms through the window and none starts after it.
  auto usb = starts(r.cam, "discover:usb", t0);
  CHECK(usb.size() >= 5);
  for (size_t i = 1; i < usb.size(); ++i) CHECK(usb[i] - usb[i - 1] <= 500);
  CHECK(usb.back() <= t0 + 2500);
  // No USB session by the window's end: Wi-Fi reconnect starts at once.
  int64_t wifi = first_at(r.cam, "discover:wifi", usb.back());
  CHECK(wifi >= 0);
  CHECK(wifi <= t0 + 2500 + 300);
  CHECK(r.tm->state() == State::ConnectedWifi);
  // The window's attempts are not counted: more than usb_failures_before_wifi_fallback of them
  // failed and the fallback still came only when the window ended, with the window's count.
  CHECK(int(usb.size()) > r.cfg.usb_failures_before_wifi_fallback);
  CHECK_EQ(r.log.count("usb.fallback_to_wifi"), 1);
  CHECK(r.log.has_field("usb.fallback_to_wifi", "\"usb_failures\":" + std::to_string(usb.size())));
  CHECK(r.log.has_field("usb.fallback_to_wifi", "not ready within the claim window"));
  // Wi-Fi then stays: no USB probe drops the working session.
  int window_attempts = count_calls(r.cam, "connect:usb");
  r.run(30000);
  CHECK_EQ(count_calls(r.cam, "connect:usb"), window_attempts);
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK_EQ(r.cam.max_sessions, 1);

  // A held interface ends the window at the first refusal: Wi-Fi reconnect starts at once.
  Rig h(true);
  h.cam.usb_fail_cost_ms = 10;
  h.run(3000);
  h.cam.fail_usb_connects = 1000;
  h.cam.usb_held = true;
  const int64_t h0 = h.clock.now;
  h.plug_usb();
  h.run(1000);
  CHECK(first_at(h.cam, "discover:usb", h0) - h0 <= 300);
  CHECK_EQ(count_calls(h.cam, "connect:usb"), 1);
  int64_t refused = first_at(h.cam, "connect:usb", h0) + 10;
  int64_t back = first_at(h.cam, "discover:wifi", h0);
  CHECK(back >= refused);
  CHECK(back - refused <= 300);
  CHECK_EQ(h.log.count("usb.fallback_to_wifi"), 1);
  CHECK(h.tm->state() == State::ConnectedWifi);

  // Success inside the window: a USB session, Wi-Fi not reconnected.
  Rig ok(true);
  ok.cam.usb_fail_cost_ms = 10;
  ok.run(3000);
  ok.cam.fail_usb_connects = 2;  // the interface comes up on the third attempt
  ok.plug_usb();
  ok.run(2500);
  CHECK(ok.tm->state() == State::ConnectedUsb);
  CHECK_EQ(count_calls(ok.cam, "connect:usb"), 3);
  CHECK_EQ(count_wifi_ready(ok.log), 1);
}

TEST_CASE("P56 C1(b) a plug-in during a protected operation waits for busy to clear, then claims at once") {
  Rig r(true);
  r.run(4000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.tm->set_busy(BusyReason::Exposure);
  r.run(200);
  const int64_t t0 = r.clock.now;
  r.plug_usb();
  r.run(6000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK(r.log.count("switch.deferred") >= 1);
  CHECK(r.log.has_field("switch.deferred", "\"busy\":\"exposure\""));
  CHECK_EQ(first_at(r.cam, "discover:usb", t0), int64_t(-1));
  CHECK_EQ(r.log.count("session.closed"), 0);
  const int64_t t1 = r.clock.now;
  r.tm->set_busy(BusyReason::None);
  r.run(2000);
  int64_t closed = first_at(r.cam, "disconnect", t1);
  int64_t first = first_at(r.cam, "discover:usb", t1);
  CHECK(closed >= t1);
  CHECK(first >= closed);
  CHECK(first - t1 <= 300);
  CHECK(r.tm->state() == State::ConnectedUsb);
  CHECK_EQ(r.cam.max_sessions, 1);
}

TEST_CASE("P56 C2(a) held cable: Wi-Fi for 30 minutes without USB probes, USB within 1 s of the release") {
  Rig r(true);
  r.cam.usb_fail_cost_ms = 10;
  r.cam.fail_usb_connects = 1000000;
  r.cam.usb_held = true;  // ptpcamerad holds the interface
  r.plug_usb();
  r.run(5000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK_EQ(r.log.count("usb.fallback_to_wifi"), 1);
  int64_t wifi_up = -1;
  for (auto& [name, at] : r.cam.timed)
    if (name == "connect:wifi" && wifi_up < 0) wifi_up = at;
  CHECK(wifi_up >= 0);
  CHECK(wifi_up <= 1000);  // the held refusal sends it to Wi-Fi at once
  const int usb_before = count_calls(r.cam, "connect:usb");
  const int closed_before = r.log.count("session.closed");
  const int lost_before = r.log.count("session.lost");
  const int ready_before = r.log.count("connection.ready");
  // 30 simulated minutes with the holder still there (the poller keeps reporting it).
  for (int minute = 0; minute < 30; ++minute) {
    r.tm->on_usb_holder(true);
    r.run(60000);
  }
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK_EQ(count_calls(r.cam, "connect:usb"), usb_before);
  CHECK_EQ(count_calls(r.cam, "discover:usb"), usb_before);
  CHECK_EQ(r.log.count("session.closed"), closed_before);
  CHECK_EQ(r.log.count("session.lost"), lost_before);
  CHECK_EQ(r.log.count("connection.ready"), ready_before);
  // The holder lets go: a USB attempt within 1 s (Wi-Fi closed first), a USB session.
  r.cam.fail_usb_connects = 0;
  const int64_t t1 = r.clock.now;
  r.tm->on_usb_holder(false);
  r.run(3000);
  int64_t closed = first_at(r.cam, "disconnect", t1);
  int64_t first = first_at(r.cam, "discover:usb", t1);
  CHECK(first >= 0);
  CHECK(first - t1 <= 1000);
  CHECK(closed >= t1);
  CHECK(first >= closed);
  CHECK_EQ(r.log.count("usb.released"), 1);
  CHECK(r.tm->state() == State::ConnectedUsb);
  CHECK_EQ(r.cam.max_sessions, 1);

  // A release that turns out to be short-lived (refused again): back to Wi-Fi at once.
  Rig b(true);
  b.cam.usb_fail_cost_ms = 10;
  b.cam.fail_usb_connects = 1000000;
  b.cam.usb_held = true;
  b.plug_usb();
  b.run(5000);
  CHECK(b.tm->state() == State::ConnectedWifi);
  const int64_t b1 = b.clock.now;
  b.tm->on_usb_holder(false);
  b.run(3000);
  int64_t refused = first_at(b.cam, "connect:usb", b1) + 10;
  int64_t back = first_at(b.cam, "discover:wifi", b1);
  CHECK(refused >= b1 + 10);
  CHECK(back >= refused);
  CHECK(back - refused <= 300);
  CHECK(b.tm->state() == State::ConnectedWifi);
}

TEST_CASE("P56 C2(b) never idle: a lost Wi-Fi session is retried for as long as it takes") {
  Rig r(true);
  r.cam.usb_fail_cost_ms = 10;
  r.cam.fail_usb_connects = 1000000;
  r.cam.usb_held = true;
  r.plug_usb();
  r.run(5000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.cam.wifi_visible = false;
  r.tm->on_sdk_disconnected(0x8208, r.cam.current_session);
  r.run(20 * 60000);
  const int64_t late = r.clock.now;
  r.run(10 * 60000);
  // Still trying both links in the last ten minutes: no terminal idle state.
  CHECK(starts(r.cam, "discover:wifi", late).size() >= 10);
  CHECK(starts(r.cam, "discover:usb", late).size() >= 10);
  CHECK(r.tm->state() != State::Error);
  CHECK_EQ(r.log.count("transport.idle"), 0);
  CHECK_EQ(r.log.count("usb.fallback_to_wifi"), 1);
  r.cam.wifi_visible = true;
  r.run(60000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  CHECK_EQ(r.cam.active_sessions, 1);
}

TEST_CASE("P56 holder report: only another process counts as holding the cable") {
  CHECK(usb_held_by_other({"pid 95408, ptpcamerad"}, 4242));
  CHECK(!usb_held_by_other({"pid 4242, camera-sidecar"}, 4242));
  CHECK(!usb_held_by_other({}, 4242));
  CHECK(usb_held_by_other({"pid 4242, camera-sidecar", "pid 77, ptpcamerad"}, 4242));
  CHECK(usb_held_by_other({"ptpcamerad"}, 4242));
}

TEST_CASE("P57 C7(a) cached Wi-Fi endpoint connects before discovery and failure falls back") {
  CaptureSink log;
  ConnectOptions options;
  options.ip = "192.0.2.5";
  SdkCameraAdapter adapter(log, options);
  auto direct = adapter.discover(Transport::Wifi);
  CHECK_EQ(direct.size(), static_cast<size_t>(1));
  CHECK_EQ(direct[0].address, "direct-ip:192.0.2.5");
  CHECK_EQ(log.count("discovery.completed"), 0);
  CHECK(!adapter.connect(direct[0]).ok);
  adapter.forget_cached_endpoint();
  CHECK(adapter.discover(Transport::Wifi).empty());
  CHECK_EQ(log.count("discovery.completed"), 1);

  Rig r;
  r.cam.cached_wifi = true;
  r.cam.cached_wifi_fails = true;
  r.run(100);
  CHECK(r.cam.calls.size() >= 3);
  CHECK_EQ(r.cam.calls[0], "cached:wifi");
  CHECK_EQ(r.cam.calls[1], "connect:wifi");
  CHECK_EQ(r.cam.calls[2], "forget:cached");
  r.run(5000);
  bool discovered = false;
  for (const auto& call : r.cam.calls) if (call == "discover:wifi") discovered = true;
  CHECK(discovered);
  CHECK(r.tm->state() == State::ConnectedWifi);
}

namespace {
// The `recovery_ms` of the last connection.ready, or -1.
int64_t last_recovery_ms(const CaptureSink& log) {
  int64_t found = -1;
  for (const auto& e : log.events) {
    if (e.name != "connection.ready") continue;
    auto at = e.json.find("\"recovery_ms\":");
    if (at != std::string::npos) found = std::stoll(e.json.substr(at + 14));
  }
  return found;
}
// P63 C5(a) rig: production timings, the dev Mac's measured SDK costs (Wi-Fi discovery 3.8 s,
// connect 2.2 s), on Wi-Fi first (the endpoint learned), then on the cable.
void on_cable_after_wifi(Rig& r) {
  r.cam.discover_cost_ms = 3800;
  r.cam.connect_cost_ms = 2200;
  r.cam.learn_endpoint = true;
  r.run(10000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  r.plug_usb();
  r.run(10000);
  CHECK(r.tm->state() == State::ConnectedUsb);
}
}  // namespace

TEST_CASE("P63 C5(a) unplug: Wi-Fi ready within ~6 s with the cached endpoint valid") {
  Rig r(true);
  on_cable_after_wifi(r);
  // The camera's Wi-Fi remote answers again 3.7 s after the unplug (FTP uploads arrived at
  // +3.7 s in the final hands-on check).
  r.cam.wifi_ready_at = r.clock.now + 3700;
  r.unplug_usb();
  r.run(15000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  const int64_t recovery = last_recovery_ms(r.log);
  CHECK(recovery > 0);
  CHECK(recovery <= 6100);
  CHECK_EQ(r.cam.active_sessions, 1);
}

TEST_CASE("P63 C5(a) unplug: a stale cached endpoint falls back to discovery within its bound") {
  Rig r(true);
  on_cable_after_wifi(r);
  r.cam.wifi_ready_at = r.clock.now + 3700;
  r.cam.cached_wifi_stale = true;
  r.unplug_usb();
  r.run(20000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  const int64_t recovery = last_recovery_ms(r.log);
  // The policy's bound: the cached-endpoint window, then one discovery and one connect.
  CHECK(recovery > 0);
  CHECK(recovery <= r.cfg.cached_endpoint_window_ms + 3800 + 2200 + 500);
  bool forgot = false;
  for (const auto& c : r.cam.calls) forgot = forgot || c == "forget:cached";
  CHECK(forgot);
}

TEST_CASE("P63 C5(a) unplug without a cached endpoint: an empty discovery is retried at once") {
  Rig r(true);
  on_cable_after_wifi(r);
  r.cam.learn_endpoint = false;
  r.cam.cached_wifi = false;
  r.cam.wifi_ready_at = r.clock.now + 3700;
  r.unplug_usb();
  r.run(20000);
  CHECK(r.tm->state() == State::ConnectedWifi);
  // Before P63: 1 s + 3.8 s (empty) + 1 s backoff + 3.8 s + 2.2 s = 11.8 s (the dev Mac's log).
  CHECK(last_recovery_ms(r.log) <= 1000 + 3800 + 3800 + 2200 + 300);
}

int main() { return mt_run_all(); }
