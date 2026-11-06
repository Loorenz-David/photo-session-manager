// SDK lifecycle and camera discovery (EnumCameraObjects).
#include <cstdio>

#include "sdk_internal.hpp"

namespace crlab {

extern SDK::ICrEnumCameraObjectInfo* g_last_enum;  // camera_connection.cpp

namespace {
bool g_sdk_initialized = false;
}

const char* to_string(ControlMode m) {
  switch (m) {
    case ControlMode::Remote: return "remote";
    case ControlMode::RemoteTransfer: return "remote_transfer";
    case ControlMode::ContentsTransfer: return "contents_transfer";
  }
  return "?";
}

bool CameraSession::sdk_available() { return true; }

bool CameraSession::sdk_init(EventSink& log) {
  if (g_sdk_initialized) return true;
  // Note: third-party reports say Init() after Release() in the same process
  // can crash, so crlab initializes exactly once per process.
  int64_t t0 = monotonic_ms();
  bool ok = SDK::Init(0);
  log.emit("sdk.init", Fields().boolean("success", ok).num("duration_ms", monotonic_ms() - t0).str("sdk_version", sdk_version()));
  g_sdk_initialized = ok;
  return ok;
}

void CameraSession::sdk_release() {
  if (!g_sdk_initialized) return;
  if (g_last_enum) {
    g_last_enum->Release();
    g_last_enum = nullptr;
  }
  SDK::Release();
  g_sdk_initialized = false;
}

std::string CameraSession::sdk_version() {
  // Encoding per RemoteCli sample: 0xMMmmpp00 (major, minor, patch).
  CrInt32u v = SDK::GetSDKVersion();
  char buf[32];
  std::snprintf(buf, sizeof buf, "%u.%02u.%02u", (v & 0xFF000000u) >> 24, (v & 0x00FF0000u) >> 16,
                (v & 0x0000FF00u) >> 8);
  return buf;
}

Transport transport_from_connect_type(const std::string& t) {
  // RemoteCli compares the connect type name with "IP" for network cameras;
  // everything else enumerated by the SDK is USB.
  if (t == "IP" || t == "Network" || t == "IP_SSH") return Transport::Wifi;
  if (t == "USB") return Transport::Usb;
  return t.find("IP") != std::string::npos ? Transport::Wifi : Transport::Usb;
}

std::string id_to_string(const DiscoveredCamera& cam, const CrInt8u* id, CrInt32u size) {
  if (!id || size == 0) return {};
  if (cam.transport == Transport::Wifi && size == 6) {  // MAC address
    char buf[24];
    std::snprintf(buf, sizeof buf, "%02x:%02x:%02x:%02x:%02x:%02x", id[0], id[1], id[2], id[3], id[4], id[5]);
    return buf;
  }
  // USB serials are ASCII. Observed with SDK 2.02 + ILCE-7RM5 over Wi-Fi:
  // GetId() returns binary bytes, so anything non-printable is hex-encoded.
  bool printable = true;
  CrInt32u n = 0;
  for (; n < size && id[n]; ++n)
    if (id[n] < 0x20 || id[n] > 0x7E) printable = false;
  std::string s;
  if (printable) {
    for (CrInt32u i = 0; i < n; ++i) s += static_cast<char>(id[i]);
    return s;
  }
  static const char* hx = "0123456789abcdef";
  for (CrInt32u i = 0; i < size; ++i) {
    s += hx[id[i] >> 4];
    s += hx[id[i] & 15];
  }
  return "hex:" + s;
}

std::vector<DiscoveredCamera> CameraSession::discover(EventSink& log, int timeout_s, int64_t* duration_ms) {
  std::vector<DiscoveredCamera> out;
  int64_t t0 = monotonic_ms();
  SDK::ICrEnumCameraObjectInfo* list = nullptr;
  auto err = SDK::EnumCameraObjects(&list, static_cast<CrInt8u>(timeout_s));
  int64_t dt = monotonic_ms() - t0;
  if (duration_ms) *duration_ms = dt;
  if (err == SDK::CrError_None && !list) {
    // Observed with SDK 2.02.00 and no camera attached: no error, no list.
    log.emit("discovery.completed", Fields().boolean("success", true).num("duration_ms", dt).num("count", 0)
                                        .num("timeout_s", timeout_s));
    return out;
  }
  if (err != SDK::CrError_None) {
    log.emit("discovery.completed", Fields().boolean("success", false).hex("sdk_error", err).str("error", describe_error(err))
                                        .num("duration_ms", dt).num("count", 0).num("timeout_s", timeout_s));
    if (list) list->Release();
    return out;
  }
  CrInt32u n = list->GetCount();
  for (CrInt32u i = 0; i < n; ++i) {
    const SDK::ICrCameraObjectInfo* ci = list->GetCameraObjectInfo(i);
    if (!ci) continue;
    DiscoveredCamera c;
    c.index = static_cast<int>(i);
    c.model = cr_str(ci->GetModel());
    c.connect_type = cr_str(ci->GetConnectionTypeName());
    c.adaptor = cr_str(ci->GetAdaptorName());
    c.transport = transport_from_connect_type(c.connect_type);
    c.id = id_to_string(c, ci->GetId(), ci->GetIdSize());
    c.pairing_necessity = cr_str(ci->GetPairingNecessity());
    c.usb_pid = ci->GetUsbPid();
#if CRLAB_HAS_IPAddressChar && CRLAB_HAS_MACAddressChar
    if (c.transport == Transport::Wifi) {
      c.ip_address = cr_str(ci->GetIPAddressChar());
      c.mac_address = cr_str(ci->GetMACAddressChar());
      if (!c.mac_address.empty()) c.id = c.mac_address;  // stable, readable identifier
    }
#endif
#if CRLAB_HAS_SSHsupportOn
    c.ssh_supported = (ci->GetSSHsupport() == SDK::CrSSHsupport_ON);
#endif
    out.push_back(c);
    log.emit("camera.discovered", Fields()
                                      .num("index", c.index)
                                      .str("camera_model", c.model)
                                      .str("camera_id", c.id)
                                      .str("connect_type", c.connect_type)
                                      .str("transport", to_string(c.transport))
                                      .str("adaptor", c.adaptor)
                                      .str("ip_address", c.ip_address)
                                      .boolean("usb_available", c.transport == Transport::Usb)
                                      .str("sdk_version", sdk_version())
                                      .boolean("ssh_supported", c.ssh_supported)
                                      .str("pairing_necessity", c.pairing_necessity)
                                      .hex("usb_pid", static_cast<uint64_t>(c.usb_pid) & 0xFFFF));
  }
  // Keep the list alive so connect() can copy the selected entry.
  if (g_last_enum) g_last_enum->Release();
  g_last_enum = list;
  log.emit("discovery.completed", Fields().boolean("success", true).num("duration_ms", dt)
                                      .num("count", static_cast<int64_t>(out.size())).num("timeout_s", timeout_s));
  return out;
}

}  // namespace crlab
