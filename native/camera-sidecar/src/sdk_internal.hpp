// Internal: the only header that includes Sony SDK headers.
//
// API names follow the Camera Remote SDK API Reference (v2.00.00) and the
// RemoteCli sample conventions. Anything not guaranteed across SDK versions
// is guarded by CRLAB_HAS_* macros generated from the INSTALLED headers
// (see sdk_symbols.def / CMakeLists.txt).
#pragma once

#include <atomic>
#include <condition_variable>
#include <deque>
#include <mutex>
#include <string>

#include "CameraRemote_SDK.h"
#include "CrDeviceProperty.h"
#include "ICrCameraObjectInfo.h"
#include "IDeviceCallback.h"
#include "crlab/camera.hpp"
#include "crlab_sdk_features.h"

namespace SDK = SCRSDK;

#if CRLAB_HAS_DCVersionTypo
using CrlabDeviceConnectionVersion = SDK::DeviceConnectionVersioin;  // spelling used by some SDK releases
#elif CRLAB_HAS_DCVersion
using CrlabDeviceConnectionVersion = SDK::DeviceConnectionVersion;
#else
#error "Neither SCRSDK::DeviceConnectionVersioin nor SCRSDK::DeviceConnectionVersion found in the installed SDK headers."
#endif

namespace crlab {

inline std::string cr_str(const CrChar* s) { return s ? std::string(s) : std::string(); }  // CrChar == char on macOS
std::string id_to_string(const DiscoveredCamera& cam, const CrInt8u* id, CrInt32u size);
Transport transport_from_connect_type(const std::string& t);
bool is_fatal_connect_error(uint32_t err);
std::string describe_error(uint32_t err);

// final: SDK::IDeviceCallback has no virtual destructor (2.02 headers), so
// Impl must never be deleted through a base pointer.
struct CameraSession::Impl final : public SDK::IDeviceCallback {
  explicit Impl(EventSink& l) : log(l) {}
  ~Impl() = default;

  EventSink& log;

  // --- session state (guarded by mu) --------------------------------------
  std::mutex mu;
  std::condition_variable cv;
  enum class Conn { Idle, Connecting, Connected, Failed, Closed } conn = Conn::Idle;
  uint32_t last_error = 0;
  SDK::CrDeviceHandle handle = 0;
  SDK::ICrCameraObjectInfo* info = nullptr;  // our own copy (CreateCameraObjectInfo*)
  Transport transport = Transport::None;
  std::string model;
  std::string id;
  uint64_t session_id = 0;
  bool closing = false;  // intentional disconnect in progress
  // Observed (SDK 2.02 + ILCE-7RM5, Wi-Fi): right after OnConnected the SDK's
  // property cache holds ~1 property; it is filled asynchronously and announced
  // with a first OnPropertyChangedCodes (~410 codes). Reads before that are incomplete.
  bool props_synced = false;
  uint32_t props_initial_count = 0;
  std::function<void(uint32_t, uint64_t)> on_disconnect;
  std::function<void(Transport)> on_exposure;  // after capture.exposed (busy Transfer, R-I20)
  std::function<void()> on_download;           // after sdk.download.completed

  // --- capture/download correlation ----------------------------------------
  std::deque<std::pair<std::string, int64_t>> awaiting_download;  // (capture_id, requested mono ms)
  // Captures whose shutter was released and that await CrNotify_Captured_Event.
  // Matched to the MOST RECENT release before the notification (Exp 04 showed
  // FIFO matching goes off by one when a command does not fire).
  struct ExposureWait {
    std::string capture_id;
    int64_t requested_ms;
    int64_t released_ms;
  };
  std::deque<ExposureWait> awaiting_exposure;
  int af_status = 0;          // last CrWarningExt_AFStatus param (1 unlocked, 2 focused, 3 not focused)
  uint64_t af_status_seq = 0; // increments on every AF status notification
  std::string save_dir;

  // --- IDeviceCallback ---------------------------------------------------------
  void OnConnected(CrlabDeviceConnectionVersion version) override;
  void OnDisconnected(CrInt32u error) override;
  void OnPropertyChanged() override;
  void OnLvPropertyChanged() override;
#if CRLAB_HAS_OnCompleteDownload2
  void OnCompleteDownload(CrChar* filename, CrInt32u type) override;
#else
  void OnCompleteDownload(CrChar* filename) override;
#endif
  void OnWarning(CrInt32u warning) override;
  void OnError(CrInt32u error) override;
  void OnPropertyChangedCodes(CrInt32u num, CrInt32u* codes) override;
#if CRLAB_HAS_OnLvPropertyChangedCodes
  void OnLvPropertyChangedCodes(CrInt32u, CrInt32u*) override {}
#endif
#if CRLAB_HAS_OnNotifyContentsTransfer
  void OnNotifyContentsTransfer(CrInt32u notify, SDK::CrContentHandle h, CrChar* filename) override;
#endif
#if CRLAB_HAS_OnWarningExt
  void OnWarningExt(CrInt32u warning, CrInt32 p1, CrInt32 p2, CrInt32 p3) override;
#endif
#if CRLAB_HAS_OnNotifyFTPTransferResult
  void OnNotifyFTPTransferResult(CrInt32u notify, CrInt32u num_ok, CrInt32u num_ng) override;
#endif
#if CRLAB_HAS_OnNotifyRemoteTransferResultFile
  void OnNotifyRemoteTransferResult(CrInt32u notify, CrInt32u per, CrChar* filename) override;
#endif
#if CRLAB_HAS_OnNotifyRemoteTransferResultData
  void OnNotifyRemoteTransferResult(CrInt32u notify, CrInt32u per, CrInt8u* data, CrInt64u size) override;
#endif
#if CRLAB_HAS_OnNotifyRemoteTransferContentsListChanged
  void OnNotifyRemoteTransferContentsListChanged(CrInt32u notify, CrInt32u slot, CrInt32u add) override;
#endif

  void download_completed(const std::string& filename, int64_t type);
  // SetDeviceProperty, then poll GetDeviceProperties until the camera reports
  // the value (SetDeviceProperty success only means "request accepted").
  // `last_read` (optional) receives the value last read back; `read` whether any read succeeded.
  bool set_property_verified(uint32_t code, uint64_t value, uint32_t type, int timeout_ms, int64_t* waited_ms,
                             uint32_t* sdk_error, uint64_t* last_read = nullptr, bool* read = nullptr);
  bool read_property(uint32_t code, uint64_t* value, uint32_t* type);
};

}  // namespace crlab
