// Remote shutter (SendCommand Release Down/Up, optional S1 half-press with AF
// wait) and the "Save to PC" SDK transfer baseline (SetSaveInfo +
// StillImageStoreDestination + OnCompleteDownload).
//
// A successful SendCommand only means the SDK accepted the command. Observed
// on the ILCE-7RM5 (2026-11-05): an accepted Release produced no exposure when
// AF had not locked. Exposure is evidenced separately by CrNotify_Captured_Event
// (event capture.exposed); file existence by OnCompleteDownload / FTP.
#include <chrono>
#include <thread>
#include <vector>

#include "crlab/sha256.hpp"
#include "sdk_internal.hpp"

namespace crlab {

// ------------------------------------------------------------- property helpers

bool CameraSession::Impl::read_property(uint32_t code, uint64_t* value, uint32_t* type) {
  SDK::CrDeviceProperty* props = nullptr;
  CrInt32 n = 0;
  bool found = false;
  if (SDK::GetDeviceProperties(handle, &props, &n) == SDK::CrError_None && props) {
    for (CrInt32 i = 0; i < n; ++i) {
      if (props[i].GetCode() == code) {
        if (value) *value = props[i].GetCurrentValue();
        if (type) *type = static_cast<uint32_t>(props[i].GetValueType());
        found = true;
        break;
      }
    }
    SDK::ReleaseDeviceProperties(handle, props);
  }
  return found;
}

bool CameraSession::Impl::set_property_verified(uint32_t code, uint64_t value, uint32_t type, int timeout_ms,
                                                int64_t* waited_ms, uint32_t* sdk_error, uint64_t* last_read,
                                                bool* read) {
  SDK::CrDeviceProperty p;
  p.SetCode(code);
  p.SetCurrentValue(value);
  p.SetValueType(static_cast<SDK::CrDataType>(type));
  int64_t t0 = monotonic_ms();
  auto err = SDK::SetDeviceProperty(handle, &p);
  if (sdk_error) *sdk_error = err;
  if (err != SDK::CrError_None) {
    if (waited_ms) *waited_ms = monotonic_ms() - t0;
    return false;
  }
  while (monotonic_ms() - t0 < timeout_ms) {
    uint64_t cur = 0;
    bool ok = read_property(code, &cur, nullptr);
    if (ok) {
      if (last_read) *last_read = cur;
      if (read) *read = true;
    }
    if (ok && cur == value) {
      if (waited_ms) *waited_ms = monotonic_ms() - t0;
      return true;
    }
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
  }
  if (waited_ms) *waited_ms = monotonic_ms() - t0;
  return false;
}

// ------------------------------------------------------------- capture

CaptureResult CameraSession::capture(const std::string& capture_id, bool af, int af_timeout_ms) {
  CaptureResult r;
  auto& im = *impl_;
  if (!connected()) {
    r.message = "not connected";
    im.log.emit("capture.rejected", Fields().str("capture_id", capture_id).str("reason", r.message).boolean("success", false));
    return r;
  }
  std::string requested_at = iso_utc_now();
  im.log.emit("capture.requested", Fields().str("capture_id", capture_id).str("transport", to_string(im.transport))
                                       .boolean("af", af));
  int64_t t0 = monotonic_ms();
  {
    std::lock_guard<std::mutex> lk(im.mu);
    im.awaiting_download.emplace_back(capture_id, t0);
    while (im.awaiting_download.size() > 64) im.awaiting_download.pop_front();
  }

  bool s1_locked = false;
#if CRLAB_HAS_PROP_S1 && CRLAB_HAS_LockIndicatorLocked
  if (af) {
    // Half-press: S1 lock (RemoteCli af_shutter pattern), then wait for the
    // camera's AF verdict instead of a fixed delay.
    uint64_t seq0;
    {
      std::lock_guard<std::mutex> lk(im.mu);
      seq0 = im.af_status_seq;
    }
    SDK::CrDeviceProperty s1;
    s1.SetCode(SDK::CrDeviceProperty_S1);
    s1.SetCurrentValue(SDK::CrLockIndicator_Locked);
    s1.SetValueType(SDK::CrDataType_UInt16);
    auto e = SDK::SetDeviceProperty(im.handle, &s1);
    s1_locked = (e == SDK::CrError_None);
    int af_result = 0;
    {
      std::unique_lock<std::mutex> lk(im.mu);
      im.cv.wait_for(lk, std::chrono::milliseconds(af_timeout_ms),
                     [&] { return im.af_status_seq != seq0 && (im.af_status == 2 || im.af_status == 3); });
      af_result = (im.af_status_seq != seq0) ? im.af_status : 0;
    }
    const char* name = af_result == 2 ? "focused" : af_result == 3 ? "not_focused" : "no_af_report";
    im.log.emit("capture.af", Fields().str("capture_id", capture_id).hex("s1_error", e).str("af_result", name)
                                  .num("duration_ms", monotonic_ms() - t0)
                                  .str("note", "no_af_report is expected in manual focus"));
  }
#else
  if (af) im.log.emit("capture.af", Fields().str("capture_id", capture_id).str("af_result", "unsupported_by_sdk_headers"));
#endif

  // Same press/release timing as the RemoteCli sample.
  int64_t t_press = monotonic_ms();
  {
    std::lock_guard<std::mutex> lk(im.mu);
    im.awaiting_exposure.push_back({capture_id, t0, t_press});
    while (im.awaiting_exposure.size() > 64) im.awaiting_exposure.pop_front();
  }
  auto e1 = SDK::SendCommand(im.handle, SDK::CrCommandId::CrCommandId_Release, SDK::CrCommandParam::CrCommandParam_Down);
  std::this_thread::sleep_for(std::chrono::milliseconds(35));
  auto e2 = SDK::SendCommand(im.handle, SDK::CrCommandId::CrCommandId_Release, SDK::CrCommandParam::CrCommandParam_Up);
  r.ack_latency_ms = monotonic_ms() - t_press;
  r.sdk_error = (e1 != SDK::CrError_None) ? e1 : e2;
  r.ok = (e1 == SDK::CrError_None && e2 == SDK::CrError_None);
  im.log.emit("capture.acknowledged", Fields()
                                          .str("capture_id", capture_id)
                                          .str("capture_requested_at", requested_at)
                                          .boolean("success", r.ok)
                                          .hex("sdk_error", r.sdk_error)
                                          .hex("sdk_error_down", e1)
                                          .hex("sdk_error_up", e2)
                                          .num("duration_ms", r.ack_latency_ms)
                                          .num("total_ms", monotonic_ms() - t0)
                                          .str("note", "SDK acceptance only; see capture.exposed for the camera's confirmation"));
#if CRLAB_HAS_PROP_S1 && CRLAB_HAS_LockIndicatorLocked
  if (s1_locked) {
    std::this_thread::sleep_for(std::chrono::milliseconds(1000));
    SDK::CrDeviceProperty s1;
    s1.SetCode(SDK::CrDeviceProperty_S1);
    s1.SetCurrentValue(SDK::CrLockIndicator_Unlocked);
    s1.SetValueType(SDK::CrDataType_UInt16);
    SDK::SetDeviceProperty(im.handle, &s1);
  }
#endif
  return r;
}

#if CRLAB_HAS_OnCompleteDownload2
void CameraSession::Impl::OnCompleteDownload(CrChar* filename, CrInt32u type) {
  download_completed(cr_str(filename), static_cast<int64_t>(type));
}
#else
void CameraSession::Impl::OnCompleteDownload(CrChar* filename) { download_completed(cr_str(filename), -1); }
#endif

void CameraSession::Impl::download_completed(const std::string& filename, int64_t type) {
  int64_t now = monotonic_ms();
  std::string provisional;
  int64_t since = -1;
  {
    std::lock_guard<std::mutex> lk(mu);
    // Provisional association only: the most recent capture requested before
    // this download. RAW+JPEG, bracketing and Pixel Shift produce several
    // downloads per shutter command, so the ledger must not treat this as 1:1.
    if (!awaiting_download.empty()) {
      provisional = awaiting_download.back().first;
      since = now - awaiting_download.back().second;
    }
  }
  long long size = file_size(filename);
  Fields f;
  f.str("filename", filename).num("size_bytes", size).boolean("success", size >= 0).str("transport", to_string(transport));
  if (type >= 0) f.hex("type", static_cast<uint64_t>(type));
  if (!provisional.empty()) f.str("provisional_capture_id", provisional).num("ms_since_capture_request", since);
  log.emit("sdk.download.completed", f);
  std::function<void()> cb;
  {
    std::lock_guard<std::mutex> lk(mu);
    cb = on_download;
  }
  if (cb) cb();
  EventSink* sink = &log;
  std::thread([sink, filename] {
    int64_t t0 = monotonic_ms();
    std::string h = sha256_file(filename);
    sink->emit("sdk.download.hashed", Fields().str("filename", filename).str("sha256", h).num("hash_ms", monotonic_ms() - t0)
                                          .boolean("success", !h.empty()));
  }).detach();
}

// ------------------------------------------------------------- save destination

namespace {
// R-I19: the app never writes Host PC alone; only Memory Card or Host PC + Memory Card.
bool dest_value(const std::string& dest, uint64_t* out) {
#if CRLAB_HAS_StoreDestHostPCAndCard && CRLAB_HAS_StoreDestCard
  if (dest == "pc+camera") *out = SDK::CrStillImageStoreDestination_HostPCAndMemoryCard;
  else if (dest == "camera") *out = SDK::CrStillImageStoreDestination_MemoryCard;
  else return false;
  return true;
#else
  (void)dest;
  (void)out;
  return false;
#endif
}
}  // namespace

bool CameraSession::configure_save(const std::string& save_dir, const std::string& dest) {
  auto& im = *impl_;
  if (!connected()) return false;
  bool ok = true;
#if CRLAB_HAS_SetSaveInfo
  if (!save_dir.empty()) {
    std::vector<CrChar> path(save_dir.begin(), save_dir.end());
    path.push_back('\0');
    CrChar prefix[1] = {'\0'};
    // Verified against RemoteCli 2.02: SetSaveInfo(h, path, "", -1).
    auto err = SDK::SetSaveInfo(im.handle, path.data(), prefix, -1);
    im.save_dir = save_dir;
    im.log.emit("sdk.save_info", Fields().str("save_dir", save_dir).hex("sdk_error", err).boolean("success", err == SDK::CrError_None));
    ok = ok && err == SDK::CrError_None;
  }
#else
  if (!save_dir.empty()) {
    im.log.emit("sdk.save_info", Fields().boolean("success", false).str("error", "SetSaveInfo not in installed SDK"));
    ok = false;
  }
#endif
  if (!dest.empty()) ok = set_store_destination(dest).verified && ok;
  return ok;
}

StoreDestinationResult CameraSession::set_store_destination(const std::string& dest, int timeout_ms) {
  StoreDestinationResult res;
  auto& im = *impl_;
  if (!connected()) return res;
#if CRLAB_HAS_PROP_StillImageStoreDestination
  uint64_t want = 0;
  if (!dest_value(dest, &want)) {
    im.log.emit("sdk.store_destination.set", Fields().str("requested", dest).boolean("success", false)
                                                 .str("error", "dest must be camera|pc+camera"));
    return res;
  }
  const uint32_t code = static_cast<uint32_t>(SDK::CrDeviceProperty_StillImageStoreDestination);
  uint64_t current = 0;
  uint32_t vtype = 0;
  if (!im.read_property(code, &current, &vtype)) {
    im.log.emit("sdk.store_destination.set", Fields().str("requested", dest).boolean("success", false)
                                                 .str("error", "property not reported by camera"));
    return res;
  }
  res.read = true;
  res.value = current;
  int64_t waited = 0;
  uint32_t err = 0;
  res.verified = (current == want) ||
                 im.set_property_verified(code, want, vtype, timeout_ms, &waited, &err, &res.value, &res.read);
  // Persistent: nothing is restored on disconnect (R-I19).
  im.log.emit("sdk.store_destination.set", Fields().str("requested", dest).hex("previous", current).hex("value", want)
                                               .hex("read_back", res.value).hex("sdk_error", err)
                                               .boolean("verified", res.verified).num("verify_ms", waited)
                                               .boolean("success", res.verified));
  return res;
#else
  im.log.emit("sdk.store_destination.set", Fields().str("requested", dest).boolean("success", false)
                                               .str("error", "StillImageStoreDestination not in installed SDK"));
  (void)timeout_ms;
  return res;
#endif
}

}  // namespace crlab
