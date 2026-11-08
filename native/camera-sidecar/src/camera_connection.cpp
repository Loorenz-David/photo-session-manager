// Connection lifecycle: Connect / OnConnected / OnError / OnDisconnected /
// Disconnect / ReleaseDevice, SSH fingerprint pinning, error description.
#include <arpa/inet.h>
#include <sys/stat.h>
#include <unistd.h>

#include <cctype>
#include <cerrno>
#include <chrono>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <sstream>
#include <vector>
#include <atomic>

#include "sdk_internal.hpp"

namespace crlab {

static std::atomic<bool> g_accept_fingerprint{false};
static std::atomic<bool> g_replace_fingerprint{false};
void accept_fingerprint_on_next_connect(bool replace) {
  g_accept_fingerprint = true;
  g_replace_fingerprint = replace;
}

// Last enumeration, kept alive so connect() can copy the selected entry.
// (The SDK object list is owned by the SDK until Release() is called.)
SDK::ICrEnumCameraObjectInfo* g_last_enum = nullptr;

std::string describe_error(uint32_t err) {
  if (err == 0) return "none";
#if CRLAB_HAS_ErrSSHUserAuth
  if (err == SDK::CrError_Connect_SSH_UserAuthenticationFailed) return "ssh user authentication failed (check Access Authen. user/password)";
#endif
#if CRLAB_HAS_ErrSSHServerAuth
  if (err == SDK::CrError_Connect_SSH_ServerAuthenticationFailed) return "ssh server authentication failed (fingerprint mismatch)";
#endif
#if CRLAB_HAS_ErrFailRejected
  if (err == SDK::CrError_Connect_FailRejected) return "connection rejected by camera (pairing/remote shooting setting?)";
#endif
#if CRLAB_HAS_ErrSessionAlreadyOpened
  if (err == SDK::CrError_Connect_SessionAlreadyOpened) return "session already opened (another app or a stale session holds the camera)";
#endif
  switch (err & 0xFF00u) {  // category ranges from CrError.h (CrError_Connect = 0x8200, ...)
    case 0x8000: return "generic error";
    case 0x8100: return "file error";
    case 0x8200: return "connection error";
    case 0x8300: return "memory error";
    case 0x8400: return "api error";
    case 0x8500: return "init error";
    case 0x8600: return "polling error";
    case 0x8700: return "adaptor error (CrAdapter libraries missing or failed)";
    case 0x8800: return "device error";
    case 0x8900: return "application error";
    default: return "unknown error";
  }
}

bool is_fatal_connect_error(uint32_t err) {
  // Wrong SSH credentials: retrying can lock the camera (Sony API reference),
  // so these must stop automatic retries.
#if CRLAB_HAS_ErrSSHUserAuth
  if (err == SDK::CrError_Connect_SSH_UserAuthenticationFailed) return true;
#endif
#if CRLAB_HAS_ErrSSHServerAuth
  if (err == SDK::CrError_Connect_SSH_ServerAuthenticationFailed) return true;
#endif
  (void)err;
  return false;
}

// ----------------------------------------------------------- callbacks

void CameraSession::Impl::OnConnected(CrlabDeviceConnectionVersion version) {
  {
    std::lock_guard<std::mutex> lk(mu);
    conn = Conn::Connected;
  }
  cv.notify_all();
  log.emit("sdk.on_connected", Fields().num("connection_version", static_cast<int64_t>(version)).unum("session_id", session_id));
}

void CameraSession::Impl::OnDisconnected(CrInt32u error) {
  bool was_connected = false;
  bool intentional = false;
  std::function<void(uint32_t, uint64_t)> cb;
  uint64_t sid = 0;
  {
    std::lock_guard<std::mutex> lk(mu);
    was_connected = (conn == Conn::Connected);
    intentional = closing;
    if (conn == Conn::Connecting) conn = Conn::Failed;
    else conn = Conn::Closed;
    last_error = error;
    cb = on_disconnect;
    sid = session_id;
  }
  cv.notify_all();
  log.emit("sdk.on_disconnected", Fields().hex("sdk_error", error).str("error", describe_error(error))
                                      .boolean("intentional", intentional).unum("session_id", sid));
  if (was_connected && !intentional && cb) cb(error, sid);
}

void CameraSession::Impl::OnError(CrInt32u error) {
  bool connecting = false;
  {
    std::lock_guard<std::mutex> lk(mu);
    connecting = (conn == Conn::Connecting);
    if (connecting) {
      conn = Conn::Failed;
      last_error = error;
    }
  }
  cv.notify_all();
  log.emit("sdk.on_error", Fields().hex("sdk_error", error).str("error", describe_error(error)).boolean("during_connect", connecting));
}

void CameraSession::Impl::OnWarning(CrInt32u warning) {
#if CRLAB_HAS_NotifyCapturedEvent
  if (warning == static_cast<CrInt32u>(SDK::CrNotify_Captured_Event)) {
    // Camera-side notification that an exposure was captured. Observed with
    // SDK 2.02 + ILCE-7RM5. Associated FIFO with pending capture requests
    // (provisional: bracketing/Pixel Shift may emit several per command).
    int64_t now = monotonic_ms();
    std::string cid;
    int64_t since_request = -1, since_release = -1;
    std::vector<std::string> missing;
    {
      std::lock_guard<std::mutex> lk(mu);
      // Most recent release at or before now; earlier unmatched releases were
      // overtaken by a later exposure, so they did not expose (provisional).
      int pick = -1;
      for (int i = static_cast<int>(awaiting_exposure.size()) - 1; i >= 0; --i)
        if (awaiting_exposure[i].released_ms <= now) {
          pick = i;
          break;
        }
      if (pick >= 0) {
        cid = awaiting_exposure[pick].capture_id;
        since_request = now - awaiting_exposure[pick].requested_ms;
        since_release = now - awaiting_exposure[pick].released_ms;
        for (int i = 0; i < pick; ++i) missing.push_back(awaiting_exposure[i].capture_id);
        awaiting_exposure.erase(awaiting_exposure.begin(), awaiting_exposure.begin() + pick + 1);
      }
    }
    for (const auto& m : missing)
      log.emit("capture.exposure_missing", Fields().str("capture_id", m).boolean("success", false)
                                               .str("reason", "a later capture was exposed first; this release produced no capture notification")
                                               .str("association", "latest_release_provisional"));
    Fields f;
    f.boolean("success", true).str("transport", to_string(transport)).str("source_notification", "CrNotify_Captured_Event");
    if (!cid.empty())
      f.str("capture_id", cid).num("ms_since_capture_request", since_request).num("ms_since_release", since_release)
          .str("association", "latest_release_provisional");
    else
      f.str("association", "none: no SDK capture pending (camera-initiated?)");
    log.emit("capture.exposed", f);
    std::function<void(Transport)> cb;
    Transport t;
    {
      std::lock_guard<std::mutex> lk(mu);
      cb = on_exposure;
      t = transport;
    }
    if (cb) cb(t);  // app and camera-initiated exposures alike (busy Transfer on USB, R-I20)
    return;
  }
#endif
  // Names for the generic warnings seen on hardware. The enumerators exist in every 2.x CrError.h,
  // so they are compared through the SDK enum rather than hard-coded numbers.
  const char* name = nullptr;
  switch (warning) {
    case SDK::CrWarning_Connect_Reconnected: name = "Connect_Reconnected"; break;
    case SDK::CrWarning_Connect_Reconnecting: name = "Connect_Reconnecting"; break;
    case SDK::CrWarning_NetworkErrorOccurred: name = "NetworkErrorOccurred"; break;  // seen on USB plug-in (Exp 05)
    case SDK::CrWarning_NetworkErrorRecovered: name = "NetworkErrorRecovered"; break;
    default: break;
  }
  Fields f;
  f.hex("warning", warning);
  if (name) f.str("name", name);
  log.emit("sdk.on_warning", f);
}

#if CRLAB_HAS_OnWarningExt
void CameraSession::Impl::OnWarningExt(CrInt32u warning, CrInt32 p1, CrInt32 p2, CrInt32 p3) {
#if CRLAB_HAS_WarningExtAFStatus
  if (warning == static_cast<CrInt32u>(SDK::CrWarningExt_AFStatus)) {
    {
      std::lock_guard<std::mutex> lk(mu);
      af_status = p1;
      ++af_status_seq;
    }
    cv.notify_all();
    const char* name = p1 == 1 ? "unlocked" : p1 == 2 ? "focused" : p1 == 3 ? "not_focused" : "other";
    log.emit("camera.af_status", Fields().str("status", name).num("param", p1));
    return;
  }
#endif
  log.emit("sdk.on_warning_ext", Fields().hex("warning", warning).num("p1", p1).num("p2", p2).num("p3", p3));
}
#endif

void CameraSession::Impl::OnPropertyChanged() {}
void CameraSession::Impl::OnLvPropertyChanged() {}

void CameraSession::Impl::OnPropertyChangedCodes(CrInt32u num, CrInt32u* codes) {
  {
    std::lock_guard<std::mutex> lk(mu);
    if (!props_synced) {
      props_synced = true;
      props_initial_count = num;
    }
  }
  cv.notify_all();
  std::string arr = "[";
  for (CrInt32u i = 0; i < num && codes; ++i) {
    char b[16];
    std::snprintf(b, sizeof b, "%s\"0x%04X\"", i ? "," : "", codes[i]);
    arr += b;
  }
  arr += "]";
  log.emit("sdk.property_changed", Fields().num("count", num).raw("codes", arr));
}

#if CRLAB_HAS_OnNotifyContentsTransfer
void CameraSession::Impl::OnNotifyContentsTransfer(CrInt32u notify, SDK::CrContentHandle h, CrChar* filename) {
  log.emit("sdk.contents_transfer", Fields().hex("notify", notify).unum("content_handle", static_cast<uint64_t>(h))
                                        .str("filename", cr_str(filename)));
}
#endif

#if CRLAB_HAS_OnNotifyFTPTransferResult
void CameraSession::Impl::OnNotifyFTPTransferResult(CrInt32u notify, CrInt32u num_ok, CrInt32u num_ng) {
  // Camera-side FTP result counters, reported over the SDK session. Useful
  // evidence for experiment 04 (SDK session alive while camera FTP runs).
  log.emit("sdk.ftp_transfer_result", Fields().hex("notify", notify).num("num_success", num_ok).num("num_fail", num_ng));
}
#endif

#if CRLAB_HAS_OnNotifyRemoteTransferResultFile
void CameraSession::Impl::OnNotifyRemoteTransferResult(CrInt32u notify, CrInt32u per, CrChar* filename) {
  log.emit("sdk.remote_transfer_result", Fields().hex("notify", notify).num("percent", per).str("filename", cr_str(filename)));
}
#endif

#if CRLAB_HAS_OnNotifyRemoteTransferResultData
void CameraSession::Impl::OnNotifyRemoteTransferResult(CrInt32u notify, CrInt32u per, CrInt8u*, CrInt64u size) {
  log.emit("sdk.remote_transfer_result", Fields().hex("notify", notify).num("percent", per).unum("size_bytes", size));
}
#endif

#if CRLAB_HAS_OnNotifyRemoteTransferContentsListChanged
void CameraSession::Impl::OnNotifyRemoteTransferContentsListChanged(CrInt32u notify, CrInt32u slot, CrInt32u add) {
  // In RemoteTransfer ("Background File Transfer") mode the camera reports
  // newly recorded contents. This is an observable "file written" signal.
  log.emit("sdk.remote_contents_changed", Fields().hex("notify", notify).num("slot", slot).num("added", add));
}
#endif

// ----------------------------------------------------------- fingerprint pinning

namespace {

std::string sanitize(const std::string& s) {
  std::string o;
  for (char c : s) o += (std::isalnum(static_cast<unsigned char>(c)) ? c : '_');
  return o.empty() ? "unknown" : o;
}
std::string fingerprint_for_display(const std::string& raw) {
  // Sony 2.02 reports printable colon-separated hex; accept raw bytes too so the event contract
  // stays stable if a later SDK changes that representation.
  std::string text = raw;
  while (!text.empty() && text.back() == '\0') text.pop_back();
  bool printable_hex = !text.empty();
  for (unsigned char c : text)
    printable_hex = printable_hex && (std::isxdigit(c) || c == ':');
  if (printable_hex) {
    for (char& c : text) c = static_cast<char>(std::tolower(static_cast<unsigned char>(c)));
    return text;
  }
  const char* digits = "0123456789abcdef";
  std::string out;
  for (unsigned char c : raw) {
    if (!out.empty()) out += ':';
    out += digits[c >> 4]; out += digits[c & 15];
  }
  return out;
}

// Returns true if the connection may proceed with this fingerprint.
// mkdir -p with private permissions.
bool mkdir_p(const std::string& dir) {
  std::string cur;
  for (size_t i = 0; i <= dir.size(); ++i) {
    if (i == dir.size() || dir[i] == '/') {
      if (!cur.empty() && ::mkdir(cur.c_str(), 0700) != 0 && errno != EEXIST) return false;
    }
    if (i < dir.size()) cur += dir[i];
  }
  struct stat st{};
  return ::stat(dir.c_str(), &st) == 0 && S_ISDIR(st.st_mode);
}

bool check_fingerprint(EventSink& log, const ConnectOptions& opt, const std::string& camera_id, const std::string& fp) {
  if (opt.fingerprint_dir.empty()) {
    log.emit("ssh.fingerprint", Fields().str("status", "not_pinned").str("note", "no fingerprint dir configured; accepting"));
    return true;
  }
  if (!mkdir_p(opt.fingerprint_dir))
    log.emit("ssh.fingerprint", Fields().str("status", "dir_create_failed").str("dir", opt.fingerprint_dir));
  std::string path = opt.fingerprint_dir + "/" + sanitize(camera_id) + ".fingerprint";
  if (g_replace_fingerprint.exchange(false)) ::unlink(path.c_str());
  std::ifstream in(path, std::ios::binary);
  if (in) {
    std::stringstream ss;
    ss << in.rdbuf();
    bool match = ss.str() == fp;
    log.emit("ssh.fingerprint", Fields().str("status", match ? "match" : "MISMATCH").boolean("success", match).str("fingerprint", fingerprint_for_display(fp)));
    if (!match)
      std::fprintf(stderr, "[crlab] SSH fingerprint mismatch for %s. Compare with the camera's Access Authen. Info.\n", camera_id.c_str());
    return match;
  }
  if (!opt.accept_fingerprint && !g_accept_fingerprint.exchange(false)) {
    log.emit("ssh.fingerprint", Fields().str("status", "unknown").boolean("success", false)
                                    .str("fingerprint", fingerprint_for_display(fp)));
    return false;
  }
  std::ofstream out(path, std::ios::binary | std::ios::trunc);
  out << fp;
  out.close();
  bool saved = static_cast<bool>(out);
  if (saved) ::chmod(path.c_str(), 0600);
  // The operator accepted this fingerprint for this connection either way;
  // a failed save only means they will be asked again next time.
  log.emit("ssh.fingerprint", Fields().str("status", saved ? "pinned" : "accepted_not_saved").boolean("success", true)
                                  .boolean("saved", saved).str("path", path).str("fingerprint", fingerprint_for_display(fp)));
  if (!saved) std::fprintf(stderr, "[crlab] WARNING: could not save fingerprint to %s\n", path.c_str());
  return true;
}

bool parse_mac(const std::string& s, CrInt8u out[6]) {
  unsigned v[6];
  if (std::sscanf(s.c_str(), "%x:%x:%x:%x:%x:%x", &v[0], &v[1], &v[2], &v[3], &v[4], &v[5]) != 6) return false;
  for (int i = 0; i < 6; ++i) out[i] = static_cast<CrInt8u>(v[i]);
  return true;
}

}  // namespace

// ----------------------------------------------------------- CameraSession

CameraSession::CameraSession(EventSink& log) : impl_(std::make_unique<Impl>(log)) {}
CameraSession::~CameraSession() { disconnect(); }

bool CameraSession::connected() const {
  std::lock_guard<std::mutex> lk(impl_->mu);
  return impl_->conn == Impl::Conn::Connected;
}
Transport CameraSession::transport() const { return impl_->transport; }
std::string CameraSession::camera_model() const { return impl_->model; }
std::string CameraSession::camera_id() const { return impl_->id; }
uint64_t CameraSession::session_id() const { return impl_->session_id; }

void CameraSession::set_disconnect_handler(std::function<void(uint32_t, uint64_t)> fn) {
  std::lock_guard<std::mutex> lk(impl_->mu);
  impl_->on_disconnect = std::move(fn);
}

void CameraSession::set_exposure_handler(std::function<void(Transport)> fn) {
  std::lock_guard<std::mutex> lk(impl_->mu);
  impl_->on_exposure = std::move(fn);
}

void CameraSession::set_download_handler(std::function<void()> fn) {
  std::lock_guard<std::mutex> lk(impl_->mu);
  impl_->on_download = std::move(fn);
}

ConnectResult CameraSession::connect(const DiscoveredCamera* cam, const ConnectOptions& opt) {
  static std::atomic<uint64_t> next_session{1};
  ConnectResult r;
  auto& im = *impl_;
  if (im.handle != 0) {
    r.message = "session already open in this process";
    return r;
  }

  // 1. Build our own ICrCameraObjectInfo.
  SDK::ICrCameraObjectInfo* info = nullptr;
  Transport transport = Transport::None;
  std::string model, id;
  if (cam && g_last_enum && cam->index >= 0 && static_cast<CrInt32u>(cam->index) < g_last_enum->GetCount()) {
    const SDK::ICrCameraObjectInfo* ci = g_last_enum->GetCameraObjectInfo(static_cast<CrInt32u>(cam->index));
    // Copy, as the RemoteCli sample does, so the enumeration can be released.
    info = SDK::CreateCameraObjectInfo(ci->GetName(), ci->GetModel(), ci->GetUsbPid(), ci->GetIdType(), ci->GetIdSize(),
                                       ci->GetId(), ci->GetConnectionTypeName(), ci->GetAdaptorName(),
                                       ci->GetPairingNecessity(), ci->GetSSHsupport());
    transport = cam->transport;
    model = cam->model;
    id = cam->id;
  } else if (!opt.usb_serial.empty()) {
#if CRLAB_HAS_UsbConnectionFactory && CRLAB_HAS_ModelILCE7RM5
    std::string serial = opt.usb_serial;
    auto err = SDK::CreateCameraObjectInfoUSBConnection(&info, SDK::CrCameraDeviceModel_ILCE_7RM5,
                                                         reinterpret_cast<CrInt8u*>(serial.data()));
    if (err != SDK::CrError_None) info = nullptr;
    transport = Transport::Usb;
    model = "ILCE-7RM5";
    id = serial;
#else
    r.message = "CreateCameraObjectInfoUSBConnection or CrCameraDeviceModel_ILCE_7RM5 not in installed SDK";
    return r;
#endif
  } else if (!opt.ip.empty()) {
#if CRLAB_HAS_EthernetConnectionFactory && CRLAB_HAS_ModelILCE7RM5
    CrInt8u mac[6] = {0, 0, 0, 0, 0, 0};
    if (!opt.mac.empty() && !parse_mac(opt.mac, mac)) {
      r.message = "invalid --mac (expected aa:bb:cc:dd:ee:ff)";
      return r;
    }
    in_addr a{};
    if (inet_pton(AF_INET, opt.ip.c_str(), &a) != 1) {
      r.message = "invalid --ip";
      return r;
    }
    // VERIFY against installed headers/RemoteCli: byte order of ipAddress.
    // Here: the in_addr value as stored in memory (network order bytes).
    CrInt32u ip = static_cast<CrInt32u>(a.s_addr);
    CrInt32u ssh = opt.ssh_user.empty() ? 0u : 1u;
    auto err = SDK::CreateCameraObjectInfoEthernetConnection(&info, SDK::CrCameraDeviceModel_ILCE_7RM5, ip, mac, ssh);
    if (err != SDK::CrError_None) info = nullptr;
    transport = Transport::Wifi;
    model = "ILCE-7RM5";
    id = opt.mac.empty() ? opt.ip : opt.mac;
#else
    r.message = "CreateCameraObjectInfoEthernetConnection or CrCameraDeviceModel_ILCE_7RM5 not in installed SDK";
    return r;
#endif
  } else {
    r.message = "no camera selected (discover first, or pass --usb-serial / --ip)";
    return r;
  }
  if (!info) {
    r.message = "failed to create camera object info";
    return r;
  }

  // 2. SSH fingerprint (Wi-Fi access point connections with Access Authen. On).
  std::string fp;
  bool use_ssh = (transport == Transport::Wifi) && !opt.ssh_user.empty();
#if CRLAB_HAS_GetFingerprint
  if (use_ssh) {
    char buf[512] = {0};
    CrInt32u size = sizeof buf;
    auto err = SDK::GetFingerprint(info, buf, &size);
    if (err != SDK::CrError_None) {
      info->Release();
      r.sdk_error = err;
      r.message = "GetFingerprint failed: " + describe_error(err);
      im.log.emit("ssh.fingerprint", Fields().str("status", "fetch_failed").hex("sdk_error", err));
      return r;
    }
    fp.assign(buf, size);
    if (!check_fingerprint(im.log, opt, id, fp)) {
      info->Release();
      // Retry on the sidecar's normal backoff; confirmation arms the next connect. Authentication
      // failures after this check remain fatal and are never retried.
      r.fatal = false;
      r.message = "SSH fingerprint not trusted";
      return r;
    }
  }
#endif

  // 3. Connect (asynchronous: completes with OnConnected or OnError).
  SDK::CrSdkControlMode mode = SDK::CrSdkControlMode_Remote;
  if (opt.mode == ControlMode::RemoteTransfer) {
#if CRLAB_HAS_ControlModeRemoteTransfer
    mode = SDK::CrSdkControlMode_RemoteTransfer;
#else
    info->Release();
    r.message = "CrSdkControlMode_RemoteTransfer not in installed SDK";
    return r;
#endif
  } else if (opt.mode == ControlMode::ContentsTransfer) {
#if CRLAB_HAS_ControlModeContentsTransfer
    mode = SDK::CrSdkControlMode_ContentsTransfer;
#else
    info->Release();
    r.message = "CrSdkControlMode_ContentsTransfer not in installed SDK";
    return r;
#endif
  }
  SDK::CrReconnectingSet reconnect = SDK::CrReconnecting_ON;
#if CRLAB_HAS_ReconnectingOff
  if (!opt.sdk_reconnect) reconnect = SDK::CrReconnecting_OFF;
#endif

  {
    std::lock_guard<std::mutex> lk(im.mu);
    im.conn = Impl::Conn::Connecting;
    im.last_error = 0;
    im.closing = false;
    im.props_synced = false;
    im.props_initial_count = 0;
    im.info = info;
    im.transport = transport;
    im.model = model;
    im.id = id;
    im.session_id = next_session++;
  }
  im.log.emit("connect.attempt", Fields().str("transport", to_string(transport)).str("camera_model", model)
                                     .str("camera_id", id).str("mode", to_string(opt.mode))
                                     .boolean("sdk_reconnect", opt.sdk_reconnect).boolean("ssh", use_ssh)
                                     .unum("session_id", im.session_id));
  int64_t t0 = monotonic_ms();
  SDK::CrDeviceHandle handle = 0;
  auto err = SDK::Connect(info, &im, &handle, mode, reconnect,
                          use_ssh ? opt.ssh_user.c_str() : nullptr,
                          use_ssh ? opt.ssh_password.c_str() : nullptr,
                          use_ssh && !fp.empty() ? fp.c_str() : nullptr,
                          use_ssh ? static_cast<CrInt32u>(fp.size()) : 0u);
  if (err != SDK::CrError_None) {
    {
      std::lock_guard<std::mutex> lk(im.mu);
      im.conn = Impl::Conn::Failed;
      im.info = nullptr;
    }
    info->Release();
    r.sdk_error = err;
    r.fatal = is_fatal_connect_error(err);
    r.message = "Connect() returned error: " + describe_error(err);
    im.log.emit("connect.failed", Fields().str("transport", to_string(transport)).hex("sdk_error", err)
                                      .str("error", describe_error(err)).num("duration_ms", monotonic_ms() - t0));
    return r;
  }
  im.handle = handle;

  // 4. Wait for the callback.
  std::unique_lock<std::mutex> lk(im.mu);
  bool done = im.cv.wait_for(lk, std::chrono::seconds(opt.connect_timeout_s),
                             [&] { return im.conn == Impl::Conn::Connected || im.conn == Impl::Conn::Failed || im.conn == Impl::Conn::Closed; });
  bool ok = done && im.conn == Impl::Conn::Connected;
  uint32_t cb_err = im.last_error;
  lk.unlock();
  int64_t dt = monotonic_ms() - t0;

  if (!ok) {
    r.sdk_error = cb_err;
    r.fatal = is_fatal_connect_error(cb_err);
    r.message = done ? "connection failed: " + describe_error(cb_err) : "timed out waiting for OnConnected";
    im.log.emit("connect.failed", Fields().str("transport", to_string(transport)).hex("sdk_error", cb_err)
                                      .str("error", r.message).num("duration_ms", dt).boolean("timeout", !done));
    disconnect();
    return r;
  }
  // Wait for the SDK's initial property sync so the first read is complete.
  {
    int64_t s0 = monotonic_ms();
    std::unique_lock<std::mutex> plk(im.mu);
    bool synced = im.cv.wait_for(plk, std::chrono::seconds(5), [&] { return im.props_synced; });
    uint32_t n = im.props_initial_count;
    plk.unlock();
    im.log.emit("props.initial_sync", Fields().boolean("success", synced).num("duration_ms", monotonic_ms() - s0)
                                          .num("codes", n).str("transport", to_string(transport)));
  }
  r.ok = true;
  r.session_id = im.session_id;
  im.log.emit("connect.succeeded", Fields().str("transport", to_string(transport)).str("camera_model", model)
                                       .str("camera_id", id).num("duration_ms", dt).unum("session_id", im.session_id));
  return r;
}

void CameraSession::disconnect() {
  auto& im = *impl_;
  SDK::CrDeviceHandle h;
  SDK::ICrCameraObjectInfo* info;
  {
    std::lock_guard<std::mutex> lk(im.mu);
    h = im.handle;
    info = im.info;
    im.closing = true;
  }
  if (h == 0 && !info) return;
  int64_t t0 = monotonic_ms();
  if (h != 0) {
    // Nothing is written here: a lost session also ends here, and a write to a dead session would
    // stall recovery (R-I19). The store destination is persistent; quit writes it beforehand
    // through CameraAdapter::before_voluntary_stop.
    auto err = SDK::Disconnect(h);
    // Wait briefly for OnDisconnected before releasing the handle (RemoteCli pattern).
    std::unique_lock<std::mutex> lk(im.mu);
    im.cv.wait_for(lk, std::chrono::seconds(5), [&] { return im.conn != Impl::Conn::Connected && im.conn != Impl::Conn::Connecting; });
    lk.unlock();
    auto rerr = SDK::ReleaseDevice(h);
    im.log.emit("session.disconnect", Fields().str("transport", to_string(im.transport)).hex("disconnect_error", err)
                                          .hex("release_error", rerr).num("duration_ms", monotonic_ms() - t0)
                                          .unum("session_id", im.session_id));
  }
  if (info) info->Release();
  std::lock_guard<std::mutex> lk(im.mu);
  im.handle = 0;
  im.info = nullptr;
  im.conn = Impl::Conn::Idle;
  im.awaiting_download.clear();
  im.awaiting_exposure.clear();
}

}  // namespace crlab
