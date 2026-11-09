// SDK-agnostic facade over one Sony Camera Remote SDK device session.
//
// No Sony types leak through this header, so the CLI and the transport
// manager compile identically against the real SDK, the compile-check shim,
// or the "no SDK" stub.
#pragma once

#include <cstdint>
#include <functional>
#include <memory>
#include <optional>
#include <string>
#include <vector>

#include "crlab/json_log.hpp"
#include "crlab/transport.hpp"

namespace crlab {

struct DiscoveredCamera {
  int index = -1;
  std::string model;
  std::string id;            // USB serial or network MAC, as reported by the SDK
  std::string connect_type;  // SDK connect type name (e.g. "USB" / "IP")
  std::string adaptor;
  Transport transport = Transport::None;
  bool ssh_supported = false;
  std::string pairing_necessity;
  int usb_pid = 0;
  std::string ip_address;   // network endpoint, when reported (Wi-Fi)
  std::string mac_address;
};

enum class ControlMode { Remote, RemoteTransfer, ContentsTransfer };
const char* to_string(ControlMode m);

struct ConnectOptions {
  Transport transport = Transport::None;  // None = first camera found
  ControlMode mode = ControlMode::Remote;
  int connect_timeout_s = 20;
  bool sdk_reconnect = false;  // CrReconnecting_ON/OFF. OFF so disconnects surface promptly.
  // Direct (non-enumerated) connection fallbacks:
  std::string usb_serial;      // CreateCameraObjectInfoUSBConnection
  std::string ip;              // CreateCameraObjectInfoEthernetConnection
  std::string mac;             // aa:bb:cc:dd:ee:ff
  // Access Authentication (SSH) for Wi-Fi access-point connections:
  std::string ssh_user;
  std::string ssh_password;
  std::string fingerprint_dir;  // pinned fingerprints (trust on first use)
  bool accept_fingerprint = false;
  // `auto` only (R-I19/R-I20/R-I23):
  std::string save_dir;                   // Save-to-PC folder set on every new session (SetSaveInfo)
  std::vector<uint16_t> usb_product_ids;  // the camera's USB product ids (IOKit match); empty = any Sony device
};

// Outcome of a verified [Still Img. Save Dest.] write.
struct StoreDestinationResult {
  bool verified = false;  // the camera reported the requested value within the timeout
  bool read = false;      // `value` holds a value read back from the camera
  uint64_t value = 0;     // the value last read back
};

struct PropertyReadResult {
  bool ok = false;
  uint32_t sdk_error = 0;
  int64_t duration_ms = 0;
  int count = 0;
};

struct CaptureResult {
  bool ok = false;
  uint32_t sdk_error = 0;
  int64_t ack_latency_ms = 0;  // SendCommand(Release Down..Up) round trip
  std::string message;
};

class CameraSession {
 public:
  // --- process-wide SDK lifecycle (Init once, Release once, same thread) ----
  static bool sdk_available();  // false in the no-SDK stub build
  static bool sdk_init(EventSink& log);
  static void sdk_release();
  static std::string sdk_version();
  static std::vector<DiscoveredCamera> discover(EventSink& log, int timeout_s, int64_t* duration_ms);

  explicit CameraSession(EventSink& log);
  ~CameraSession();
  CameraSession(const CameraSession&) = delete;
  CameraSession& operator=(const CameraSession&) = delete;

  ConnectResult connect(const DiscoveredCamera* cam, const ConnectOptions& opt);
  void disconnect();  // idempotent; Disconnect + ReleaseDevice
  bool connected() const;
  Transport transport() const;
  std::string camera_model() const;
  std::string camera_id() const;

  // Read the curated property set (only properties present in both the
  // installed SDK headers and the camera's reply). all=true dumps every code.
  PropertyReadResult read_properties(Fields& out, bool all = false);
  PropertyReadResult read_ftp_status(Fields& out);

  // Remote shutter. capture_id is echoed in events for ledger correlation.
  // af=true: half-press (S1 lock), wait for the camera's AF status (focused /
  // not focused, up to af_timeout_ms), full press, release, S1 unlock —
  // the RemoteCli "af_shutter" sequence. af=false: full press/release only.
  CaptureResult capture(const std::string& capture_id, bool af = false, int af_timeout_ms = 3000);

  // Set [Still Img. Save Dest.] ("camera" | "pc+camera"; Host PC alone is never written, R-I19)
  // and verify by reading it back for up to timeout_ms. The write is persistent: nothing is
  // restored on disconnect.
  StoreDestinationResult set_store_destination(const std::string& dest, int timeout_ms = 3000);

  // "Save to PC" (SDK image transfer): sets SetSaveInfo (when save_dir is not empty) and, when
  // dest is not empty, the store destination as set_store_destination() does (persistent).
  bool configure_save(const std::string& save_dir, const std::string& dest);

  // Invoked (from an SDK thread) when the camera session drops.
  void set_disconnect_handler(std::function<void(uint32_t sdk_error, uint64_t session_id)> fn);
  // Invoked (from an SDK thread) after `capture.exposed` is emitted, with the session's transport.
  void set_exposure_handler(std::function<void(Transport)> fn);
  // Invoked (from an SDK thread) after `sdk.download.completed` is emitted.
  void set_download_handler(std::function<void()> fn);
  uint64_t session_id() const;

  struct Impl;

 private:
  std::unique_ptr<Impl> impl_;
};

// CameraAdapter for TransportManager, backed by CameraSession.
class SdkCameraAdapter : public CameraAdapter {
 public:
  SdkCameraAdapter(EventSink& log, ConnectOptions base, int discover_timeout_s = 3);
  std::vector<CameraEndpoint> discover(Transport t) override;
  void forget_cached_endpoint() override;
  ConnectResult connect(const CameraEndpoint& ep) override;
  void disconnect() override;
  bool read_state(Fields& out) override;
  // Quit during a USB session: one write of "camera" (Memory Card), at most 1 s (R-I19).
  void before_voluntary_stop(Transport t) override;
  bool last_discovery_saw(Transport t) const override {
    for (const auto& c : last_found_)
      if (c.transport == t) return true;
    return false;
  }
  CameraSession& session() { return session_; }
  void set_usb_serial_hint(const std::string& serial) { usb_serial_hint_ = serial; }

 private:
  // Connect sequence after a successful session connect (R-I19): SetSaveInfo(save_dir), then the
  // store destination for the link, verified; the outcome goes into `r` for connection.ready.
  void after_connect(ConnectResult& r, Transport t);

  EventSink& log_;
  ConnectOptions base_;
  int discover_timeout_s_;
  CameraSession session_;
  std::vector<DiscoveredCamera> last_found_;
  std::string usb_serial_hint_;
};

// Host diagnostics (no SDK): processes known to compete for the camera.
std::vector<std::string> competing_processes();

}  // namespace crlab
