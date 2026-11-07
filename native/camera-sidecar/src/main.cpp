// camera-sidecar — Session Manager's camera sidecar (the Sony camera lab's `crlab` CLI).
// The app runs `camera-sidecar auto`; the other commands are kept for diagnostics.
//
// Every observation is emitted as one JSON line on stdout (see json_log.hpp).
// Human-oriented hints go to stderr. Exit codes: 0 ok, 1 operation failed,
// 2 usage error, 3 session lost, 4 SDK unavailable.
#include <atomic>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <deque>
#include <iostream>
#include <map>
#include <mutex>
#include <sstream>
#include <string>
#include <thread>
#include <vector>

#include "crlab/camera.hpp"
#include "crlab/json_log.hpp"
#include "crlab/transport.hpp"
#include "crlab/usb_monitor.hpp"

using namespace crlab;

namespace {

const char* kUsage = R"(camera-sidecar — Session Manager camera sidecar (Sony Camera Remote SDK)

usage: camera-sidecar <command> [options]

commands:
  version                    SDK version and build info
  diagnose                   competing apps, Sony USB presence, SDK init, discovery
  discover                   enumerate cameras (USB and network)
  connect                    connect, read state, disconnect   [--cycles N] [--hold-s S]
  props                      connect and read properties       [--all]
  capture                    connect and fire the shutter      [--count N] [--interval-ms M] [--wait-s S]
                                                               [--save-dir DIR --dest camera|pc+camera]
                                                               [--af] half-press, wait for AF, then release
  ftp-status                 connect and read camera FTP state via SDK properties
  store-dest --value V       set [Still Img. Save Dest.] (camera|pc+camera), verified, persistent
  session                    connect, then read commands from stdin (see below)
  auto                       transport manager: prefer USB, fall back to Wi-Fi [--no-usb] [--no-wifi]
                             [--save-dir DIR] Save-to-PC folder set on every new session
                             [--usb-product-id HEX[,HEX...]] the camera's USB product ids (default 0x0E0C)
                             first stdin line: credentials user=<pct> password=<pct> (percent-encoded)

selection / connection options:
  --transport usb|wifi|any   which transport to use (default any)
  --index N                  camera index from discovery
  --usb-serial S             connect directly by USB serial   (env CAMERA_USB_SERIAL)
  --ip A.B.C.D [--mac M]     connect directly by IP address   (env CAMERA_IP, CAMERA_MAC)
  --mode remote|remote-transfer   SDK control mode (default remote)
  --sdk-reconnect on|off     SDK auto-reconnect (default off)
  --connect-timeout S        seconds to wait for OnConnected (default 20)
  --discover-timeout S       EnumCameraObjects timeout (default 3)
  --accept-fingerprint       trust and pin a new SSH fingerprint (compare with camera first!)
  Wi-Fi Access Authentication credentials reach `auto` only as its first stdin line, never argv or env.

output options:
  --experiment ID            experiment id stamped on events (default "adhoc")
  --log FILE                 also append events to FILE

stdin commands (session/auto):  capture [id=..] [count=N] [interval_ms=M] [af=1] | props [all=1] | ftp-status
                                | store-dest value=camera|pc+camera
                                | save dir=.. dest=camera|pc+camera | status | busy reason=..|none
                                | usb-retry (auto) | sleep ms=N | quit
)";

struct Args {
  std::string cmd;
  std::map<std::string, std::string> kv;
  bool has(const std::string& k) const { return kv.count(k) != 0; }
  std::string get(const std::string& k, const std::string& def = "") const {
    auto it = kv.find(k);
    return it == kv.end() ? def : it->second;
  }
  int geti(const std::string& k, int def) const { return has(k) ? std::atoi(get(k).c_str()) : def; }
};

const char* kFlags[] = {"all", "accept-fingerprint", "no-usb", "no-wifi", "help", "af"};

bool parse_args(int argc, char** argv, Args& a) {
  if (argc < 2) return false;
  a.cmd = argv[1];
  for (int i = 2; i < argc; ++i) {
    std::string s = argv[i];
    if (s.rfind("--", 0) != 0) return false;
    s = s.substr(2);
    bool flag = false;
    for (auto* f : kFlags) flag = flag || s == f;
    if (flag) a.kv[s] = "1";
    else if (i + 1 < argc) a.kv[s] = argv[++i];
    else return false;
  }
  return true;
}

std::string env(const char* k, const std::string& def = "") {
  const char* v = std::getenv(k);
  return v && *v ? v : def;
}

std::string uuid4() {
  unsigned char b[16];
  arc4random_buf(b, sizeof b);
  b[6] = (b[6] & 0x0F) | 0x40;
  b[8] = (b[8] & 0x3F) | 0x80;
  char s[40];
  std::snprintf(s, sizeof s, "%02x%02x%02x%02x-%02x%02x-%02x%02x-%02x%02x-%02x%02x%02x%02x%02x%02x", b[0], b[1], b[2], b[3],
                b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]);
  return s;
}

Transport parse_transport(const std::string& s) {
  if (s == "usb") return Transport::Usb;
  if (s == "wifi") return Transport::Wifi;
  return Transport::None;
}

ConnectOptions options_from(const Args& a) {
  ConnectOptions o;
  o.transport = parse_transport(a.get("transport", "any"));
  o.mode = a.get("mode", "remote") == "remote-transfer" ? ControlMode::RemoteTransfer : ControlMode::Remote;
  o.connect_timeout_s = a.geti("connect-timeout", 20);
  o.sdk_reconnect = a.get("sdk-reconnect", "off") == "on";
  o.usb_serial = a.get("usb-serial", env("CAMERA_USB_SERIAL"));
  o.ip = a.get("ip", env("CAMERA_IP"));
  o.mac = a.get("mac", env("CAMERA_MAC"));
  // Access Authentication credentials are never read from the environment (R-I21): `auto`
  // receives them as its first stdin line.
  o.fingerprint_dir = env("CRLAB_FINGERPRINT_DIR", env("HOME") + "/.config/crlab/fingerprints");
  o.accept_fingerprint = a.has("accept-fingerprint");
  // Directs only make sense for their transport.
  if (o.transport == Transport::Wifi) o.usb_serial.clear();
  if (o.transport == Transport::Usb) o.ip.clear();
  return o;
}

// Line reader for stdin commands (session/auto).
class StdinQueue {
 public:
  void start() {
    std::thread([this] {
      std::string line;
      while (std::getline(std::cin, line)) push(line);
      push("quit");  // EOF
    }).detach();
  }
  bool pop(std::string& out) {
    std::lock_guard<std::mutex> lk(mu_);
    if (q_.empty()) return false;
    out = q_.front();
    q_.pop_front();
    return true;
  }
  void push(const std::string& s) {
    std::lock_guard<std::mutex> lk(mu_);
    q_.push_back(s);
  }

 private:
  std::mutex mu_;
  std::deque<std::string> q_;
};

struct Cmd {
  std::string name;
  std::map<std::string, std::string> kv;
  std::string get(const std::string& k, const std::string& d = "") const {
    auto it = kv.find(k);
    return it == kv.end() ? d : it->second;
  }
};

Cmd parse_cmd(const std::string& line) {
  Cmd c;
  std::istringstream is(line);
  is >> c.name;
  std::string tok;
  while (is >> tok) {
    auto eq = tok.find('=');
    if (eq == std::string::npos) c.kv[tok] = "1";
    else c.kv[tok.substr(0, eq)] = tok.substr(eq + 1);
  }
  return c;
}

const DiscoveredCamera* select_camera(const std::vector<DiscoveredCamera>& list, const ConnectOptions& o, int index) {
  if (index >= 0) {
    for (const auto& c : list)
      if (c.index == index) return &c;
    return nullptr;
  }
  for (const auto& c : list)
    if (o.transport == Transport::None || c.transport == o.transport) return &c;
  return nullptr;
}

void print_competitors(JsonLog& log) {
  auto procs = competing_processes();
  std::string arr = "[";
  for (size_t i = 0; i < procs.size(); ++i) arr += (i ? ",\"" : "\"") + json_escape(procs[i]) + "\"";
  arr += "]";
  log.emit("host.competing_processes", Fields().raw("processes", arr).num("count", static_cast<int64_t>(procs.size())));
  if (!procs.empty())
    std::fprintf(stderr, "[crlab] WARNING: processes that may hold the camera are running (see host.competing_processes). "
                         "Quit Imaging Edge / Capture One before SDK tests.\n");
}

// Connect using discovery or direct options. Returns true on success.
bool open_session(JsonLog& log, CameraSession& s, const Args& a, ConnectOptions& o) {
  print_competitors(log);
  int64_t dt = 0;
  auto list = CameraSession::discover(log, a.geti("discover-timeout", 3), &dt);
  const DiscoveredCamera* cam = select_camera(list, o, a.geti("index", -1));
  if (!cam && o.usb_serial.empty() && o.ip.empty()) {
    std::fprintf(stderr, "[crlab] no camera found for transport '%s'. Check camera settings (docs/setup.md) "
                         "or connect directly with --usb-serial / --ip.\n", to_string(o.transport));
    return false;
  }
  if (cam) o.transport = cam->transport;
  auto r = s.connect(cam, o);
  if (!r.ok) {
    std::fprintf(stderr, "[crlab] connect failed: %s\n", r.message.c_str());
    if (o.transport == Transport::Usb) {
      for (const auto& owner : report_usb_claim(log))
        std::fprintf(stderr, "[crlab] USB interface held by: %s\n", owner.c_str());
    }
  }
  if (r.ok) {
    log.set_default("camera_id", s.camera_model());
    log.set_default("transport", to_string(s.transport()));
  }
  return r.ok;
}

void run_capture(JsonLog& log, CameraSession& s, const std::string& id_in, int count, int interval_ms, bool af = false) {
  for (int i = 0; i < count; ++i) {
    std::string id = (count == 1 && !id_in.empty()) ? id_in : (id_in.empty() ? uuid4() : id_in + "-" + std::to_string(i + 1));
    s.capture(id, af);
    if (i + 1 < count) std::this_thread::sleep_for(std::chrono::milliseconds(interval_ms));
  }
  (void)log;
}

void wait_ms(int64_t ms) { std::this_thread::sleep_for(std::chrono::milliseconds(ms)); }

// --------------------------------------------------------------- commands

int cmd_version(JsonLog& log) {
  log.emit("crlab.version", Fields().str("crlab", "0.1.0").boolean("sdk_compiled_in", CameraSession::sdk_available())
                                .str("sdk_version", CameraSession::sdk_available() ? CameraSession::sdk_version() : "unavailable"));
  return 0;
}

int cmd_diagnose(JsonLog& log, const Args& a) {
  print_competitors(log);
  UsbMonitor mon;
  std::vector<UsbDeviceInfo> seen;
  std::mutex mu;
  bool ok = mon.start([&](bool arrived, const UsbDeviceInfo& d) {
    std::lock_guard<std::mutex> lk(mu);
    if (arrived) seen.push_back(d);
  });
  wait_ms(300);
  mon.stop();
  for (const auto& d : seen)
    log.emit("usb.present", Fields().hex("product_id", d.product_id).str("product", d.product).boolean("serial_present", !d.serial.empty()));
  log.emit("usb.summary", Fields().boolean("monitor_ok", ok).num("sony_devices", static_cast<int64_t>(seen.size())));
  if (!CameraSession::sdk_init(log)) return 4;
  int64_t dt = 0;
  CameraSession::discover(log, a.geti("discover-timeout", 3), &dt);
  return 0;
}

int cmd_discover(JsonLog& log, const Args& a) {
  int64_t dt = 0;
  auto list = CameraSession::discover(log, a.geti("discover-timeout", 3), &dt);
  std::fprintf(stderr, "[crlab] %zu camera(s) found in %lld ms\n", list.size(), static_cast<long long>(dt));
  for (const auto& c : list)
    std::fprintf(stderr, "  [%d] %-12s %-5s id=%s ip=%s type=%s ssh=%s\n", c.index, c.model.c_str(), to_string(c.transport),
                 c.id.c_str(), c.ip_address.empty() ? "-" : c.ip_address.c_str(), c.connect_type.c_str(), c.ssh_supported ? "yes" : "no");
  return list.empty() ? 1 : 0;
}

int cmd_connect(JsonLog& log, const Args& a) {
  int cycles = std::max(1, a.geti("cycles", 1));
  int hold_s = a.geti("hold-s", 0);
  int failures = 0;
  for (int i = 0; i < cycles; ++i) {
    CameraSession s(log);
    ConnectOptions o = options_from(a);
    int64_t t0 = monotonic_ms();
    bool ok = open_session(log, s, a, o);
    log.emit("connect.cycle", Fields().num("cycle", i + 1).boolean("success", ok).num("duration_ms", monotonic_ms() - t0)
                                  .str("transport", to_string(o.transport)));
    if (!ok) {
      ++failures;
      continue;
    }
    Fields st;
    s.read_properties(st, false);
    log.emit("camera.state", st);
    if (hold_s > 0) wait_ms(hold_s * 1000LL);
    s.disconnect();
    if (i + 1 < cycles) wait_ms(1000);
  }
  return failures ? 1 : 0;
}

int cmd_props(JsonLog& log, const Args& a) {
  CameraSession s(log);
  ConnectOptions o = options_from(a);
  if (!open_session(log, s, a, o)) return 1;
  Fields st;
  auto r = s.read_properties(st, a.has("all"));
  log.emit("camera.state", st);
  s.disconnect();
  return r.ok ? 0 : 1;
}

int cmd_capture(JsonLog& log, const Args& a) {
  CameraSession s(log);
  ConnectOptions o = options_from(a);
  if (!open_session(log, s, a, o)) return 1;
  if (a.has("save-dir") || a.has("dest")) s.configure_save(a.get("save-dir"), a.get("dest"));
  Fields before;
  s.read_properties(before, false);
  log.emit("camera.state", before.str("phase", "before_capture"));
  run_capture(log, s, a.get("id"), std::max(1, a.geti("count", 1)), a.geti("interval-ms", 1500), a.has("af"));
  int wait_s = a.geti("wait-s", 5);
  std::fprintf(stderr, "[crlab] waiting %d s for SDK callbacks (downloads, property changes)\n", wait_s);
  wait_ms(wait_s * 1000LL);
  Fields after;
  s.read_properties(after, false);
  log.emit("camera.state", after.str("phase", "after_capture"));
  s.disconnect();
  return 0;
}

int cmd_store_dest(JsonLog& log, const Args& a) {
  std::string v = a.get("value");
  if (v != "camera" && v != "pc+camera") {  // Host PC alone is never written (R-I19)
    std::fprintf(stderr, "usage: camera-sidecar store-dest --value camera|pc+camera [--transport ...]\n");
    return 2;
  }
  CameraSession s(log);
  ConnectOptions o = options_from(a);
  if (!open_session(log, s, a, o)) return 1;
  bool ok = s.set_store_destination(v).verified;
  s.disconnect();
  return ok ? 0 : 1;
}

int cmd_ftp_status(JsonLog& log, const Args& a) {
  CameraSession s(log);
  ConnectOptions o = options_from(a);
  if (!open_session(log, s, a, o)) return 1;
  Fields f;
  auto r = s.read_ftp_status(f);
  log.emit("camera.ftp_state", f);
  wait_ms(2000);  // allow OnNotifyFTPTransferResult to arrive
  s.disconnect();
  return r.ok ? 0 : 1;
}

int cmd_session(JsonLog& log, const Args& a) {
  CameraSession s(log);
  ConnectOptions o = options_from(a);
  std::atomic<bool> lost{false};
  s.set_disconnect_handler([&](uint32_t err, uint64_t sid) {
    lost = true;
    log.emit("session.lost", Fields().hex("sdk_error", err).unum("session_id", sid));
  });
  if (!open_session(log, s, a, o)) return 1;
  if (a.has("save-dir") || a.has("dest")) s.configure_save(a.get("save-dir"), a.get("dest"));
  log.emit("session.ready", Fields().str("transport", to_string(s.transport())).str("camera_model", s.camera_model()));
  StdinQueue in;
  in.start();
  std::string line;
  while (true) {
    if (lost) {
      s.disconnect();
      return 3;
    }
    if (!in.pop(line)) {
      wait_ms(20);
      continue;
    }
    Cmd c = parse_cmd(line);
    if (c.name.empty()) continue;
    log.emit("cmd.received", Fields().str("cmd", c.name).str("cmd_id", c.get("cmd_id")));
    if (c.name == "quit") break;
    if (c.name == "capture") {
      run_capture(log, s, c.get("id"), std::max(1, std::atoi(c.get("count", "1").c_str())),
                  std::atoi(c.get("interval_ms", "1500").c_str()), c.get("af") == "1");
    } else if (c.name == "store-dest") {
      s.set_store_destination(c.get("value"));
    } else if (c.name == "props") {
      Fields st;
      s.read_properties(st, c.get("all") == "1");
      log.emit("camera.state", st.str("cmd_id", c.get("cmd_id")));
    } else if (c.name == "ftp-status") {
      Fields f;
      s.read_ftp_status(f);
      log.emit("camera.ftp_state", f.str("cmd_id", c.get("cmd_id")));
    } else if (c.name == "save") {
      s.configure_save(c.get("dir"), c.get("dest"));
    } else if (c.name == "status") {
      log.emit("session.status", Fields().boolean("connected", s.connected()).str("transport", to_string(s.transport())));
    } else if (c.name == "sleep") {
      wait_ms(std::atoi(c.get("ms", "0").c_str()));
    } else {
      log.emit("cmd.error", Fields().str("cmd", c.name).str("error", "unknown command"));
    }
    log.emit("cmd.done", Fields().str("cmd", c.name).str("cmd_id", c.get("cmd_id")));
  }
  s.disconnect();
  return 0;
}

BusyReason parse_busy(const std::string& s) {
  if (s == "exposure") return BusyReason::Exposure;
  if (s == "bracketing") return BusyReason::Bracketing;
  if (s == "pixel_shift") return BusyReason::PixelShift;
  if (s == "file_transfer") return BusyReason::FileTransfer;
  if (s == "none" || s.empty()) return BusyReason::None;
  return BusyReason::Other;
}

// `--usb-product-id 0x0E0C[,0x....]`: comma-separated hex ids; the A7R V's 0x0E0C when absent or
// unparsable (parse_args keeps only the last occurrence of a repeated flag).
std::vector<uint16_t> parse_product_ids(const std::string& s) {
  std::vector<uint16_t> ids;
  std::stringstream ss(s);
  std::string tok;
  while (std::getline(ss, tok, ',')) {
    char* end = nullptr;
    unsigned long v = std::strtoul(tok.c_str(), &end, 16);
    if (end != tok.c_str() && v > 0 && v <= 0xFFFF) ids.push_back(static_cast<uint16_t>(v));
  }
  if (ids.empty()) ids.push_back(0x0E0C);
  return ids;
}

constexpr int64_t kTransferExpectedMs = 10000;  // busy Transfer from a USB exposure to its download (R-I20)

int cmd_auto(JsonLog& log, const Args& a) {
  ConnectOptions base = options_from(a);
  base.transport = Transport::None;
  base.sdk_reconnect = a.get("sdk-reconnect", "off") == "on";
  base.save_dir = a.get("save-dir");
  base.usb_product_ids = parse_product_ids(a.get("usb-product-id", "0x0E0C"));
  if (base.sdk_reconnect)
    std::fprintf(stderr, "[crlab] note: with SDK reconnect ON, disconnects may surface only after the SDK gives up "
                         "(minutes, per Sony docs). USB removal is still detected via IOKit.\n");

  // R-I21: the first stdin line, read synchronously before the adapter copies `base`.
  // `credentials user=<pct> password=<pct>` sets the Access Authentication credentials; its
  // cmd.received carries no field but `cmd`. EOF first = quit (no connect). Any other first line
  // is the first ordinary command.
  std::string first;
  bool first_is_command = false;
  if (!std::getline(std::cin, first)) return 0;
  if (first.rfind("credentials ", 0) == 0) {
    Cmd c = parse_cmd(first);
    base.ssh_user = percent_decode(c.get("user"));
    base.ssh_password = percent_decode(c.get("password"));
    log.emit("cmd.received", Fields().str("cmd", "credentials"));
  } else {
    first_is_command = true;
  }

  SdkCameraAdapter adapter(log, base, a.geti("discover-timeout", 3));
  TransportConfig cfg;
  cfg.usb_enabled = !a.has("no-usb");
  cfg.wifi_enabled = !a.has("no-wifi");
  cfg.usb_settle_ms = a.geti("usb-settle-ms", 2500);
  TransportManager tm(adapter, log, cfg);

  adapter.session().set_disconnect_handler([&](uint32_t err, uint64_t sid) { tm.on_sdk_disconnected(err, sid); });

  // Busy Transfer (R-I20/R-I22): a USB exposure (app or camera-initiated) holds FileTransfer until
  // its download completes or 10 s pass; it only defers a voluntary stop.
  std::atomic<int64_t> transfer_deadline{-1};  // monotonic ms, -1 = none
  std::atomic<bool> usb_exposure_seen{false};
  adapter.session().set_exposure_handler([&](Transport t) {
    if (t != Transport::Usb) return;
    usb_exposure_seen = true;
    transfer_deadline = monotonic_ms() + kTransferExpectedMs;
    tm.set_busy(BusyReason::FileTransfer);
  });
  adapter.session().set_download_handler([&] {
    if (transfer_deadline.exchange(-1) >= 0) tm.set_busy(BusyReason::None);
  });

  UsbMonitor mon(UsbMonitor::kSonyVendorId, base.usb_product_ids);
  std::mutex usb_mu;
  int usb_count = 0;
  bool mon_ok = mon.start([&](bool arrived, const UsbDeviceInfo& d) {
    std::lock_guard<std::mutex> lk(usb_mu);
    usb_count += arrived ? 1 : -1;
    if (usb_count < 0) usb_count = 0;
    if (arrived && !d.serial.empty()) adapter.set_usb_serial_hint(d.serial);
    char detail[96];
    std::snprintf(detail, sizeof detail, "pid=0x%04X %s", d.product_id, d.product.c_str());
    tm.on_usb_presence(usb_count > 0, detail);
  });
  if (mon_ok) log.emit("usb.initial", Fields().boolean("present", mon.present_count() > 0));
  else log.emit("usb.monitor.error", Fields().str("message", "IOKit monitor failed; USB arrival will not be detected"));
  print_competitors(log);

  StdinQueue in;
  if (first_is_command) in.push(first);
  in.start();
  std::string line;
  bool running = true;
  while (running) {
    running = tm.step();
    while (running && in.pop(line)) {
      Cmd c = parse_cmd(line);
      if (c.name.empty()) continue;
      log.emit("cmd.received", Fields().str("cmd", c.name).str("cmd_id", c.get("cmd_id")));
      State st = tm.state();
      bool connected = st == State::ConnectedUsb || st == State::ConnectedWifi;
      if (c.name == "quit") {
        tm.request_stop();
      } else if (c.name == "status") {
        log.emit("transport.status", Fields().str("state", to_string(st)).str("transport", to_string(tm.transport()))
                                         .num("failures", tm.consecutive_failures()));
      } else if (c.name == "busy") {
        tm.set_busy(parse_busy(c.get("reason")));
      } else if (c.name == "usb-retry") {
        tm.usb_retry();  // accepted in any state (R-I23)
      } else if (!connected) {
        log.emit("cmd.rejected", Fields().str("cmd", c.name).str("reason", std::string("not connected (") + to_string(st) + ")"));
      } else if (c.name == "reconnect") {
        // R-I29: offered only on Wi-Fi; on USB the session is never left voluntarily (R-I23).
        if (tm.transport() == Transport::Usb)
          log.emit("cmd.rejected", Fields().str("cmd", c.name).str("reason", "reconnect only on wifi"));
        else
          tm.reconnect();
      } else if (c.name == "capture") {
        // Mark busy across the exposure so a pending USB switch waits.
        usb_exposure_seen = false;
        tm.set_busy(BusyReason::Exposure);
        tm.step();  // drain busy flag before the blocking capture
        run_capture(log, adapter.session(), c.get("id"), std::max(1, std::atoi(c.get("count", "1").c_str())),
                    std::atoi(c.get("interval_ms", "1500").c_str()), c.get("af") == "1");
        // A USB exposure during this command already set FileTransfer: keep it until the download.
        if (!usb_exposure_seen) tm.set_busy(BusyReason::None);
      } else if (c.name == "props") {
        Fields f;
        adapter.session().read_properties(f, c.get("all") == "1");
        log.emit("camera.state", f);
      } else if (c.name == "ftp-status") {
        Fields f;
        adapter.session().read_ftp_status(f);
        log.emit("camera.ftp_state", f);
      } else {
        log.emit("cmd.error", Fields().str("cmd", c.name).str("error", "unknown command"));
      }
      log.emit("cmd.done", Fields().str("cmd", c.name).str("cmd_id", c.get("cmd_id")));
    }
    int64_t deadline = transfer_deadline.load();
    if (deadline >= 0 && monotonic_ms() >= deadline && transfer_deadline.compare_exchange_strong(deadline, -1))
      tm.set_busy(BusyReason::None);  // no download within 10 s: back to the safe point
    wait_ms(100);
  }
  mon.stop();
  return 0;
}

}  // namespace

int main(int argc, char** argv) {
  Args a;
  if (!parse_args(argc, argv, a) || a.cmd == "help" || a.has("help")) {
    std::fputs(kUsage, stderr);
    return a.cmd == "help" ? 0 : 2;
  }
  JsonLog log(a.get("experiment", env("EXPERIMENT_ID", "adhoc")), "sdk");
  if (a.has("log") && !log.open_file(a.get("log"))) {
    std::fprintf(stderr, "cannot open log file %s\n", a.get("log").c_str());
    return 2;
  }
  if (a.cmd == "version") return cmd_version(log);

  if (a.cmd != "diagnose") {
    if (!CameraSession::sdk_available()) {
      log.emit("sdk.unavailable", Fields().str("error", "camera-sidecar was built without the Sony SDK"));
      std::fprintf(stderr, "[camera-sidecar] this build has no Sony SDK (stub). Build on a machine with the SDK: tools/sidecar/build.sh --sdk <dir>.\n");
      return 4;
    }
    if (!CameraSession::sdk_init(log)) return 4;
  }

  int rc = 2;
  if (a.cmd == "diagnose") rc = cmd_diagnose(log, a);
  else if (a.cmd == "discover") rc = cmd_discover(log, a);
  else if (a.cmd == "connect") rc = cmd_connect(log, a);
  else if (a.cmd == "props") rc = cmd_props(log, a);
  else if (a.cmd == "capture") rc = cmd_capture(log, a);
  else if (a.cmd == "ftp-status") rc = cmd_ftp_status(log, a);
  else if (a.cmd == "store-dest") rc = cmd_store_dest(log, a);
  else if (a.cmd == "session") rc = cmd_session(log, a);
  else if (a.cmd == "auto") rc = cmd_auto(log, a);
  else std::fputs(kUsage, stderr);

  CameraSession::sdk_release();
  return rc;
}
