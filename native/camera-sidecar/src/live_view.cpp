// Live view through the Camera Remote SDK pull API (Experiment 07).
//
// The SDK hands over one JPEG per GetLiveViewImage call (RemoteCli
// get_live_view_only: GetLiveViewImageInfo for the buffer size, then
// GetLiveViewImage; CrWarning_Frame_NotUpdated when the camera has no newer
// frame). Nothing here assumes a frame rate, a size or that the camera stops
// its own live view when nobody pulls: those are what the experiment measures.
#include <chrono>
#include <cstdlib>
#include <cstring>
#include <mutex>
#include <string>
#include <thread>
#include <vector>

#include "sdk_internal.hpp"

namespace crlab {

namespace {
#if CRLAB_HAS_PROP_LiveView_Image_Quality && CRLAB_HAS_LvQualityLow && CRLAB_HAS_LvQualityHigh
constexpr uint32_t kQualityCode = static_cast<uint32_t>(SDK::CrDeviceProperty_LiveView_Image_Quality);
#endif
}  // namespace

LiveViewState CameraSession::live_view_state() {
  LiveViewState s;
  if (!connected()) return s;
  auto& im = *impl_;
  s.ok = true;
  (void)im;
  uint64_t v = 0;
#if CRLAB_HAS_PROP_LiveViewStatus
  if (im.read_property(static_cast<uint32_t>(SDK::CrDeviceProperty_LiveViewStatus), &v, nullptr)) s.status = static_cast<int64_t>(v);
#endif
#if CRLAB_HAS_PROP_LiveView_Image_Quality
  if (im.read_property(static_cast<uint32_t>(SDK::CrDeviceProperty_LiveView_Image_Quality), &v, nullptr)) s.quality = static_cast<int64_t>(v);
#endif
#if CRLAB_HAS_PROP_LiveViewProtocol
  if (im.read_property(static_cast<uint32_t>(SDK::CrDeviceProperty_LiveViewProtocol), &v, nullptr)) s.protocol = static_cast<int64_t>(v);
#endif
  (void)v;
  return s;
}





LiveViewFrame CameraSession::pull_live_view_frame() {
  LiveViewFrame f;
  if (!connected()) return f;
#if CRLAB_HAS_GetLiveViewImage && CRLAB_HAS_GetLiveViewImageInfo && CRLAB_HAS_LvFrameNotUpdated
  auto& im = *impl_;
  int64_t t0 = monotonic_ms();
  SDK::CrImageInfo info;
  auto err = SDK::GetLiveViewImageInfo(im.handle, &info);
  if (err != SDK::CrError_None) {
    f.sdk_error = err;
    f.status = err == SDK::CrWarning_Frame_NotUpdated ? LvPull::NotUpdated : LvPull::Error;
    f.pull_ms = monotonic_ms() - t0;
    return f;
  }
  CrInt32u size = info.GetBufferSize();
  if (size < 1) {
    f.pull_ms = monotonic_ms() - t0;
    return f;  // Error, sdk_error 0: "buffer size 0"
  }
  std::vector<uint8_t> buf(size);
  SDK::CrImageDataBlock block;
  block.SetSize(size);
  block.SetData(buf.data());
  err = SDK::GetLiveViewImage(im.handle, &block);
  f.pull_ms = monotonic_ms() - t0;
  f.sdk_error = err;
  if (err == SDK::CrWarning_Frame_NotUpdated) {
    f.status = LvPull::NotUpdated;
    return f;
  }
  if (err != SDK::CrError_None) return f;
  CrInt32u n = block.GetImageSize();
  CrInt8u* p = block.GetImageData();
  if (n == 0 || !p) return f;
#if CRLAB_HAS_LvFrameNo
  f.frame_no = block.GetFrameNo();
#endif
  f.jpeg.assign(p, p + n);
  f.status = LvPull::Ok;
  return f;
#else
  f.status = LvPull::Unsupported;
  return f;
#endif
}




// --- Focus from the live picture (Experiment 10) ---------------------------------------------------------------
namespace {
struct PropInfo {
  bool found = false;
  uint64_t value = 0;
  uint32_t type = 0;
  bool writable = false;
  std::vector<int64_t> options;
};

PropInfo read_prop_info(SDK::CrDeviceHandle h, uint32_t code) {
  PropInfo r;
  SDK::CrDeviceProperty* props = nullptr;
  CrInt32 n = 0;
  if (SDK::GetDeviceProperties(h, &props, &n) == SDK::CrError_None && props) {
    for (CrInt32 i = 0; i < n; ++i) {
      if (props[i].GetCode() != code) continue;
      r.found = true;
      r.value = props[i].GetCurrentValue();
      r.type = static_cast<uint32_t>(props[i].GetValueType());
      r.writable = props[i].IsSetEnableCurrentValue();
      CrInt8u* v = props[i].GetValues();
      CrInt32u size = props[i].GetValueSize();
      if (v && size >= 2 && size % 2 == 0 && size <= 512) {  // UInt16 arrays (FocusArea, FocusMode)
        for (CrInt32u k = 0; k + 1 < size; k += 2) {
          uint16_t x;
          std::memcpy(&x, v + k, 2);
          r.options.push_back(x);
        }
      }
      break;
    }
    SDK::ReleaseDeviceProperties(h, props);
  }
  return r;
}
}  // namespace

FocusState CameraSession::focus_state() {
  FocusState f;
  if (!connected()) return f;
  auto& im = *impl_;
  f.ok = true;
#if CRLAB_HAS_PROP_FocusMode
  { auto p = read_prop_info(im.handle, static_cast<uint32_t>(SDK::CrDeviceProperty_FocusMode)); if (p.found) f.focus_mode = static_cast<int64_t>(p.value); }
#endif
#if CRLAB_HAS_PROP_FocusArea
  {
    auto p = read_prop_info(im.handle, static_cast<uint32_t>(SDK::CrDeviceProperty_FocusArea));
    if (p.found) { f.focus_area = static_cast<int64_t>(p.value); f.focus_area_writable = p.writable; f.focus_area_options = p.options; }
  }
#endif
#if CRLAB_HAS_PROP_AF_Area_Position
  {
    auto p = read_prop_info(im.handle, static_cast<uint32_t>(SDK::CrDeviceProperty_AF_Area_Position));
    if (p.found) { f.af_area_position = static_cast<int64_t>(p.value); f.af_position_writable = p.writable; }
  }
#endif
#if CRLAB_HAS_PROP_PriorityKeySettings
  { auto p = read_prop_info(im.handle, static_cast<uint32_t>(SDK::CrDeviceProperty_PriorityKeySettings)); if (p.found) f.priority_key = static_cast<int64_t>(p.value); }
#endif
#if CRLAB_HAS_PROP_FocusIndication
  { auto p = read_prop_info(im.handle, static_cast<uint32_t>(SDK::CrDeviceProperty_FocusIndication)); if (p.found) f.focus_indication = static_cast<int64_t>(p.value); }
#endif
  (void)im;
  return f;
}

bool CameraSession::set_focus_point(int x, int y, int64_t* verify_ms, uint32_t* sdk_error) {
  if (verify_ms) *verify_ms = 0;
  if (sdk_error) *sdk_error = 0;
  if (!connected() || x < 0 || x > 639 || y < 0 || y > 479) return false;
#if CRLAB_HAS_PROP_AF_Area_Position
  auto& im = *impl_;
  const uint64_t xy = (static_cast<uint64_t>(x) << 16) | static_cast<uint64_t>(y);
  // RemoteCli execute_pos_xy: UInt32, x<<16|y. The device-property read-back never reflects the new point on the
  // a7R V (Experiment 10, 2026-11-26), but the live-view focus frame does: confirm against the frame instead.
  int64_t t0 = monotonic_ms();
  auto err = im.write_camera(WriteIntent::FocusPoint, xy, static_cast<uint32_t>(SDK::CrDataType_UInt32));
  if (sdk_error) *sdk_error = err;
  if (err != SDK::CrError_None) return false;
  if (verify_ms) *verify_ms = -1;  // -1: frame never confirmed within 1 s
  while (monotonic_ms() - t0 < 1000) {
    { std::lock_guard<std::mutex> lk(im.mu); if (im.cancel_focus) return false; }
    bool ok = false;
    for (const auto& f : focus_frames(&ok)) {
      if (!f.x_den || !f.y_den) continue;
      long fx = static_cast<long>(f.x_num) * 640 / f.x_den, fy = static_cast<long>(f.y_num) * 480 / f.y_den;
      if (std::labs(fx - x) <= 3 && std::labs(fy - y) <= 3) {
        if (verify_ms) *verify_ms = monotonic_ms() - t0;
        return true;
      }
    }
    std::this_thread::sleep_for(std::chrono::milliseconds(15));
  }
  return true;  // accepted by the SDK; frame confirmation timed out (verify_ms = -1)
#else
  return false;
#endif
}



std::string CameraSession::autofocus(int timeout_ms, int64_t* duration_ms, uint32_t* s1_error) {
  if (duration_ms) *duration_ms = 0;
  if (s1_error) *s1_error = 0;
  if (!connected()) return "not_connected";
#if CRLAB_HAS_PROP_S1 && CRLAB_HAS_LockIndicatorLocked && CRLAB_HAS_UnlockIndicator
  auto& im = *impl_;
  int64_t t0 = monotonic_ms();
  uint64_t seq0;
  {
    std::lock_guard<std::mutex> lk(im.mu);
    seq0 = im.af_status_seq;
  }
  auto e = im.write_camera(WriteIntent::HalfPress, SDK::CrLockIndicator_Locked, static_cast<uint32_t>(SDK::CrDataType_UInt16));
  if (s1_error) *s1_error = e;
  int af = 0;
  {
    std::unique_lock<std::mutex> lk(im.mu);
    im.cv.wait_for(lk, std::chrono::milliseconds(timeout_ms),
                   [&] { return im.cancel_focus || (im.af_status_seq != seq0 && (im.af_status == 2 || im.af_status == 3)); });
    af = (!im.cancel_focus && im.af_status_seq != seq0) ? im.af_status : 0;
  }
  if (duration_ms) *duration_ms = monotonic_ms() - t0;
  im.write_camera(WriteIntent::HalfPress, SDK::CrLockIndicator_Unlocked, static_cast<uint32_t>(SDK::CrDataType_UInt16));
  return af == 2 ? "focused" : af == 3 ? "not_focused" : "no_af_report";
#else
  return "unsupported";
#endif
}

std::vector<FocusFrame> CameraSession::focus_frames(bool* ok) {
  std::vector<FocusFrame> out;
  if (ok) *ok = false;
  if (!connected()) return out;
#if CRLAB_HAS_GetSelectLiveViewProperties && CRLAB_HAS_LvPropAFAreaPosition && CRLAB_HAS_FocusFrameInfoType
  auto& im = *impl_;
  CrInt32 num = 0;
  SDK::CrLiveViewProperty* lv = nullptr;
  CrInt32u code = SDK::CrLiveViewProperty_AF_Area_Position;
  auto err = SDK::GetSelectLiveViewProperties(im.handle, 1, &code, &lv, &num);
  if (err != SDK::CrError_None) return out;
  if (ok) *ok = true;
  if (lv && num >= 1 && lv[0].GetFrameInfoType() == SDK::CrFrameInfoType_FocusFrameInfo) {
    CrInt32u size = lv[0].GetValueSize();
    auto* fi = reinterpret_cast<SDK::CrFocusFrameInfo*>(lv[0].GetValue());
    if (fi && size >= sizeof(SDK::CrFocusFrameInfo)) {
      for (CrInt32u i = 0; i < size / sizeof(SDK::CrFocusFrameInfo); ++i) {
        FocusFrame f;
        f.type = fi[i].type; f.state = fi[i].state; f.priority = fi[i].priority;
        f.x_num = fi[i].xNumerator; f.x_den = fi[i].xDenominator; f.y_num = fi[i].yNumerator; f.y_den = fi[i].yDenominator;
        f.width = fi[i].width; f.height = fi[i].height;
        out.push_back(f);
      }
    }
  }
  if (lv) SDK::ReleaseLiveViewProperties(im.handle, lv);
#endif
  return out;
}

}  // namespace crlab
