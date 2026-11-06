// Stub backend used when the Sony SDK is not installed (CRLAB_WITH_SDK=OFF).
// Every operation fails loudly with an explanation; nothing is simulated, so
// a stub build can never be mistaken for real camera evidence.
#include "crlab/camera.hpp"

namespace crlab {

namespace {
const char* kMsg = "built without the Sony Camera Remote SDK (see native/camera-sidecar/README.md)";
}

const char* to_string(ControlMode m) {
  switch (m) {
    case ControlMode::Remote: return "remote";
    case ControlMode::RemoteTransfer: return "remote_transfer";
    case ControlMode::ContentsTransfer: return "contents_transfer";
  }
  return "?";
}

struct CameraSession::Impl {
  explicit Impl(EventSink& l) : log(l) {}
  EventSink& log;
};

bool CameraSession::sdk_available() { return false; }
bool CameraSession::sdk_init(EventSink& log) {
  log.emit("sdk.init", Fields().boolean("success", false).str("error", kMsg));
  return false;
}
void CameraSession::sdk_release() {}
std::string CameraSession::sdk_version() { return "unavailable"; }
std::vector<DiscoveredCamera> CameraSession::discover(EventSink& log, int, int64_t* d) {
  if (d) *d = 0;
  log.emit("discovery.completed", Fields().boolean("success", false).str("error", kMsg));
  return {};
}
CameraSession::CameraSession(EventSink& log) : impl_(std::make_unique<Impl>(log)) {}
CameraSession::~CameraSession() = default;
ConnectResult CameraSession::connect(const DiscoveredCamera*, const ConnectOptions&) {
  ConnectResult r;
  r.fatal = true;
  r.message = kMsg;
  return r;
}
void CameraSession::disconnect() {}
bool CameraSession::connected() const { return false; }
Transport CameraSession::transport() const { return Transport::None; }
std::string CameraSession::camera_model() const { return {}; }
std::string CameraSession::camera_id() const { return {}; }
PropertyReadResult CameraSession::read_properties(Fields&, bool) { return {}; }
PropertyReadResult CameraSession::read_ftp_status(Fields&) { return {}; }
CaptureResult CameraSession::capture(const std::string&, bool, int) {
  CaptureResult r;
  r.message = kMsg;
  return r;
}
bool CameraSession::configure_save(const std::string&, const std::string&) { return false; }
StoreDestinationResult CameraSession::set_store_destination(const std::string&, int) { return {}; }
void CameraSession::set_disconnect_handler(std::function<void(uint32_t, uint64_t)>) {}
void CameraSession::set_exposure_handler(std::function<void(Transport)>) {}
void CameraSession::set_download_handler(std::function<void()>) {}
uint64_t CameraSession::session_id() const { return 0; }

}  // namespace crlab
